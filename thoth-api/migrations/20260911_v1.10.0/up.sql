-- BE-06 Migration 2 (20260911_v1.10.0): the work-level WORK_UPSERT substrate and the Crossref execution profile.
--
-- Specification: docs/publisher-services/specifications/BE-06-R52B.md (SHA-256 584683ca...) as amended by #848
-- Amendments 1-3 (Replacement Amendment 3, SHA-256 d9c04dec...). The executable order is R52B §23.3 as amended by
-- Amendment 3 §3.3. Amendment 3 removes the baseline table, its enum, its guard and its code (CTO R-3 decision,
-- #848 comment 5686341130) and adds the two generic TRUNCATE guards (R-8).
--
-- Inventory created here (Amendment 3 §3.3): 8 tables, 5 enum types, 27 functions, 35 triggers, 18 indexes
-- (8 primary keys and 10 others), 75 columns, and 2 seed rows. One released object changes:
-- distribution_job_work_id_fkey becomes ON DELETE SET NULL. Migration 1 (20260910_v1.10.0) added the three enum labels
-- this migration uses; PostgreSQL forbids using a label added in the same transaction, hence two directories.
--
-- Every function body schema-qualifies its callees (public.*), so behaviour does not depend on search_path.

-- =====================================================================================================================
-- Step 1. The nine functions that read no BE-06 object, with their invariant assertions (R52B §23.3 step 1).
-- =====================================================================================================================

-- §16.3: canonical DOI identity, total for every value the released Doi parser accepts and accepting nothing it rejects.
CREATE FUNCTION public.crossref_canonical_doi(raw text) RETURNS text
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE ident text;
BEGIN
    IF raw IS NULL OR raw = '' THEN RETURN NULL; END IF;
    -- Strip exactly the optional prefix Doi::from_str accepts, case-insensitively.
    -- The 'i' flag scopes case-insensitivity to the prefix, exactly as the live (?i:) group does.
    ident := regexp_replace(raw, '^(https?://)?(www\.)?(dx\.)?doi\.org/', '', 'i');
    -- Accept exactly the live identifier class -- a CLOSED class, not "any non-whitespace".
    IF ident !~ '^10\.[0-9]{4,9}/[-._;()/:a-zA-Z0-9<>+\[\]]+$' THEN RETURN NULL; END IF;
    RETURN 'https://doi.org/' || lower(ident);
END $$;

-- §17.2: exact rendering of an instant, and the one domain guard: no instant at or after
-- 10000-01-01T00:00:00.000Z has a 17-digit encoding, because PostgreSQL's YYYY renders a fifth digit.
CREATE FUNCTION public.crossref_ts_encode(t timestamptz) RETURNS bigint
LANGUAGE plpgsql IMMUTABLE AS $$
BEGIN
    IF t >= TIMESTAMPTZ '10000-01-01 00:00:00+00' THEN
        RAISE EXCEPTION 'CROSSREF_TIMESTAMP_OVERFLOW: %', t;
    END IF;
    RETURN to_char(t AT TIME ZONE 'UTC', 'YYYYMMDDHH24MISSMS')::bigint;
END $$;

-- §17.2: STRICT decode. A value is valid only if it is the exact rendering of the instant it decodes to.
-- make_timestamptz silently NORMALISES second 60 and hour 24, so leniency would admit values that encode no instant.
-- Round-trip identity is the validity test. The round trip runs inside the handler, so a value that normalises past
-- the domain (second 60 or hour 24 on 9999-12-31) makes crossref_ts_encode raise and is refused like every other
-- non-encoding.
CREATE FUNCTION public.crossref_ts_decode(v bigint) RETURNS timestamptz
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE s text; t timestamptz; BEGIN
    IF v IS NULL OR v < 10000000000000000 OR v > 99999999999999999 THEN RETURN NULL; END IF;
    s := v::text;
    BEGIN
        t := make_timestamptz(substr(s,1,4)::int, substr(s,5,2)::int, substr(s,7,2)::int,
                              substr(s,9,2)::int, substr(s,11,2)::int,
                              substr(s,13,2)::numeric + substr(s,15,3)::numeric/1000, 'UTC');
        IF public.crossref_ts_encode(t) <> v THEN RETURN NULL; END IF;
    EXCEPTION WHEN others THEN RETURN NULL; END;
    RETURN t;
END $$;

-- §17.2: the calendar successor, exactly one millisecond later. 99991231235959999 has none:
-- crossref_ts_encode raises CROSSREF_TIMESTAMP_OVERFLOW.
CREATE FUNCTION public.crossref_ts_next(v bigint) RETURNS bigint
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE t timestamptz; BEGIN
    t := public.crossref_ts_decode(v);
    IF t IS NULL THEN RAISE EXCEPTION 'CROSSREF_TIMESTAMP_NOT_DECODABLE: %', v; END IF;
    RETURN public.crossref_ts_encode(t + interval '1 millisecond');
END $$;

-- §17.3: the only clock. UTC, millisecond, re-evaluated on every call.
CREATE FUNCTION public.crossref_ts_now() RETURNS bigint LANGUAGE sql VOLATILE AS $$
    SELECT public.crossref_ts_encode(clock_timestamp())
$$;

-- §17.3: the pure allocation rule, testable with any clock value.
CREATE FUNCTION public.crossref_allocate_timestamp(history bigint, floor_value bigint, now_17 bigint)
RETURNS bigint LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE cand bigint;
BEGIN
    IF floor_value NOT IN (0, 99999999999999) THEN RAISE EXCEPTION 'CROSSREF_VERSION_FLOOR_DOMAIN'; END IF;
    IF public.crossref_ts_decode(now_17) IS NULL THEN RAISE EXCEPTION 'CROSSREF_TIMESTAMP_NOT_DECODABLE'; END IF;
    IF history IS NOT NULL AND public.crossref_ts_decode(history) IS NULL THEN
        RAISE EXCEPTION 'CROSSREF_TIMESTAMP_NOT_DECODABLE'; END IF;
    cand := now_17;
    IF history IS NOT NULL AND cand <= history THEN
        cand := public.crossref_ts_next(history);          -- calendar successor; CROSSREF_TIMESTAMP_OVERFLOW at 99991231235959999
    END IF;
    IF cand <= floor_value OR (history IS NOT NULL AND cand <= history) THEN
        RAISE EXCEPTION 'CROSSREF_TIMESTAMP_NOT_INCREASING'; END IF;
    RETURN cand;
END $$;

-- §16.3: the lower-case hex SHA-256 of the UTF-8 concatenation of the canonical DOIs, sorted by code point, each
-- followed by a newline.
CREATE FUNCTION public.crossref_doi_set_digest(dois text[]) RETURNS text LANGUAGE sql IMMUTABLE AS $$
    SELECT encode(sha256(convert_to(coalesce((SELECT string_agg(x || E'\n', '' ORDER BY x COLLATE "C")
                                               FROM unnest(dois) x), ''), 'UTF8')), 'hex')
$$;

-- §8.4 rule 2 and §8.5: roots(x) = {x} and every HAS_CHILD parent of x. The Crossref profile's root rule. 'has-child'
-- is the label the released relation_type enum stores; HAS_CHILD is its GraphQL name.
CREATE FUNCTION public.crossref_roots(w uuid) RETURNS uuid[] LANGUAGE sql STABLE AS $$
    SELECT array_agg(r ORDER BY r) FROM (
        SELECT w AS r
        UNION
        SELECT relator_work_id FROM public.work_relation WHERE related_work_id = w AND relation_type = 'has-child') s
$$;

-- §14.3: the registration membership of a deposit, API-derived and serializer-exact.
CREATE FUNCTION public.crossref_deposit_membership(root uuid) RETURNS text[] LANGUAGE sql STABLE AS $$
    SELECT coalesce(array_agg(d ORDER BY d COLLATE "C"), ARRAY[]::text[]) FROM (
        SELECT public.crossref_canonical_doi(w.doi) AS d
          FROM public.work w
         WHERE w.work_id = root AND w.doi IS NOT NULL AND w.landing_page IS NOT NULL
        UNION
        SELECT public.crossref_canonical_doi(c.doi)
          FROM public.work_relation r JOIN public.work c ON c.work_id = r.related_work_id
         WHERE r.relator_work_id = root AND r.relation_type = 'has-child' AND c.doi IS NOT NULL
    ) s WHERE d IS NOT NULL
$$;

-- §23.3 migration-time assertions over the step-1 functions. Each is a hard failure.
DO $$
DECLARE r record; got text;
BEGIN
    -- crossref_canonical_doi is total and exact over the released-grammar corpus of §16.3: 40 values, of which 22 are
    -- accepted by the released Doi::from_str (expected: the lower-cased released stored form) and 18 are rejected
    -- (expected: NULL). Rust test BE-06 T116 reads this block and asserts parity with Doi::from_str.
    FOR r IN SELECT * FROM (VALUES
        -- BE06_DOI_CORPUS_BEGIN
        (E'10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'10.1234/ABC', 'https://doi.org/10.1234/abc'),
        (E'10.123456789/x', 'https://doi.org/10.123456789/x'),
        (E'https://doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'http://doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'https://dx.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'http://dx.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'https://www.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'http://www.dx.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'dx.doi.org/10.12345/AbC', 'https://doi.org/10.12345/abc'),
        (E'www.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'HTTPS://DOI.ORG/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'Https://Dx.Doi.Org/10.12345/MiXeD', 'https://doi.org/10.12345/mixed'),
        (E'10.12345/a-b.c_d;e(f)g/h:i', 'https://doi.org/10.12345/a-b.c_d;e(f)g/h:i'),
        (E'10.12345/<tag>+[x]', 'https://doi.org/10.12345/<tag>+[x]'),
        (E'10.12345/9780262525831', 'https://doi.org/10.12345/9780262525831'),
        (E'https://doi.org/10.11647/OBP.0001', 'https://doi.org/10.11647/obp.0001'),
        (E'10.12345/a/b/c', 'https://doi.org/10.12345/a/b/c'),
        (E'www.dx.doi.org/10.12345/abc', 'https://doi.org/10.12345/abc'),
        (E'HTTP://WWW.DOI.ORG/10.99999/Z', 'https://doi.org/10.99999/z'),
        (E'10.1234/_', 'https://doi.org/10.1234/_'),
        (E'', NULL),
        (E'10.123/abc', NULL),
        (E'10.1234567890/abc', NULL),
        (E'10.12345/a b', NULL),
        (E'10.12345/a%20b', NULL),
        (E'10.12345/a#b', NULL),
        (E'10.12345/é', NULL),
        (E' 10.12345/abc', NULL),
        (E'10.12345/abc ', NULL),
        (E'11.12345/abc', NULL),
        (E'10.12345/', NULL),
        (E'ftp://doi.org/10.12345/abc', NULL),
        (E'https://doi.org/10.12345/abc?x=1', NULL),
        (E'https://example.org/10.12345/abc', NULL),
        (E'doi:10.12345/abc', NULL),
        (E'https://doi.org//10.12345/abc', NULL),
        (E'10.12345/abc\n', NULL),
        (E'10.12345/a"b', NULL)
        -- BE06_DOI_CORPUS_END
    ) AS corpus(raw, expected) LOOP
        got := public.crossref_canonical_doi(r.raw);
        IF got IS DISTINCT FROM r.expected THEN
            RAISE EXCEPTION 'M2 assertion: crossref_canonical_doi';
        END IF;
    END LOOP;

    -- crossref_ts_next returns a valid 17-digit instant exactly one millisecond later for the 13 boundary vectors of
    -- §17.2 (T194): second, minute, hour, day, 30- and 31-day month, year and leap-day rollovers, a century
    -- non-leap and a 400-year leap February, and two in-second vectors. Naive decimal +1 is invalid for 11 of them.
    FOR r IN SELECT * FROM (VALUES
        -- BE06_TS_VECTORS_BEGIN
        (20260904120059999::bigint, 20260904120100000::bigint),
        (20260904125959999::bigint, 20260904130000000::bigint),
        (20260904235959999::bigint, 20260905000000000::bigint),
        (20260930235959999::bigint, 20261001000000000::bigint),
        (20260131235959999::bigint, 20260201000000000::bigint),
        (20261231235959999::bigint, 20270101000000000::bigint),
        (20250228235959999::bigint, 20250301000000000::bigint),
        (20240228235959999::bigint, 20240229000000000::bigint),
        (20240229235959999::bigint, 20240301000000000::bigint),
        (21000228235959999::bigint, 21000301000000000::bigint),
        (20000228235959999::bigint, 20000229000000000::bigint),
        (20260904120000123::bigint, 20260904120000124::bigint),
        (20260904120009999::bigint, 20260904120010000::bigint)
        -- BE06_TS_VECTORS_END
    ) AS vectors(input, successor) LOOP
        IF public.crossref_ts_next(r.input) <> r.successor
           OR public.crossref_ts_decode(r.successor) IS NULL
           OR public.crossref_ts_decode(r.successor) - public.crossref_ts_decode(r.input) <> interval '1 millisecond' THEN
            RAISE EXCEPTION 'M2 assertion: crossref_ts_next';
        END IF;
    END LOOP;

    -- crossref_ts_decode refuses the six non-encodings of §17.2 (second 60, hour 24, month 13, day 32, 29 February in a
    -- non-leap year, the 14-digit floor sentinel), and returns NULL, without raising, for two values past the domain.
    IF public.crossref_ts_decode(20260904120060000) IS NOT NULL OR public.crossref_ts_decode(20260904240000000) IS NOT NULL
       OR public.crossref_ts_decode(20261304120000000) IS NOT NULL OR public.crossref_ts_decode(20260932120000000) IS NOT NULL
       OR public.crossref_ts_decode(20250229120000000) IS NOT NULL OR public.crossref_ts_decode(99999999999999) IS NOT NULL
       OR public.crossref_ts_decode(99991231235960000) IS NOT NULL OR public.crossref_ts_decode(99991231240000000) IS NOT NULL THEN
        RAISE EXCEPTION 'M2 assertion: crossref_ts_decode';
    END IF;

    -- the last successor, and the two overflows of R52B-23
    IF public.crossref_ts_next(99991231235959998) <> 99991231235959999 THEN
        RAISE EXCEPTION 'M2 assertion: crossref_ts_next at the domain maximum';
    END IF;
    BEGIN
        PERFORM public.crossref_ts_next(99991231235959999);
        RAISE EXCEPTION 'M2 assertion: no overflow from crossref_ts_next';
    EXCEPTION WHEN raise_exception THEN
        IF SQLERRM NOT LIKE 'CROSSREF_TIMESTAMP_OVERFLOW:%' THEN RAISE; END IF;
    END;
    BEGIN
        PERFORM public.crossref_ts_encode(TIMESTAMPTZ '10000-01-01 00:00:00+00');
        RAISE EXCEPTION 'M2 assertion: no overflow from crossref_ts_encode';
    EXCEPTION WHEN raise_exception THEN
        IF SQLERRM NOT LIKE 'CROSSREF_TIMESTAMP_OVERFLOW:%' THEN RAISE; END IF;
    END;
END $$;

-- =====================================================================================================================
-- Step 2. The five new enum types (R52B §18.7 as amended by Amendment 3 §3.1: the baseline residual-class enum removed).
-- =====================================================================================================================

CREATE TYPE public.crossref_write_route AS ENUM ('WORK_UPSERT', 'PUBLISHER_BACK_CATALOGUE', 'LEGACY_SCHEDULED', 'MANUAL_RECOVERY');
CREATE TYPE public.crossref_write_permit_state AS ENUM ('RESERVED', 'AUTHORIZED', 'INDETERMINATE', 'ACCEPTED', 'NONE_ATTEMPTED', 'VOIDED');
CREATE TYPE public.crossref_reconciliation_state AS ENUM ('RECONCILED', 'RECONCILIATION_REQUIRED', 'RECONCILIATION_IMPOSSIBLE');
CREATE TYPE public.crossref_write_scope AS ENUM ('SINGLE_ROOT_WORK');
CREATE TYPE public.crossref_void_reason AS ENUM ('SOURCE_CHANGED_DURING_PREPARATION', 'DOI_MEMBERSHIP_CHANGED', 'ARTIFACT_DOI_SET_MISMATCH',
    'ARTIFACT_BATCH_ID_MISMATCH', 'ARTIFACT_TIMESTAMP_MISMATCH', 'EXECUTION_NOT_PERMITTED', 'PROFILE_NOT_ADMITTED', 'INELIGIBLE',
    'BINDING_SUPERSEDED', 'ASSIGNMENT_DISABLED', 'NO_WORK', 'OWNER_ABANDONED', 'OPERATOR_CLEANUP');

-- =====================================================================================================================
-- Step 3. New columns on the released tables (R52B §18.1, §18.2). Every one is nullable, so no table is rewritten.
-- =====================================================================================================================

ALTER TABLE public.distribution_job
    ADD COLUMN execution_profile public.distribution_platform,
    ADD COLUMN work_identity uuid,
    ADD COLUMN created_generation bigint,
    ADD COLUMN job_ordinal integer,
    ADD COLUMN predecessor_job_id uuid,
    ADD COLUMN superseded_by_job_id uuid;
ALTER TABLE public.distribution_job_attempt
    ADD COLUMN claimed_generation bigint,
    ADD COLUMN fenced_at timestamptz,
    ADD COLUMN recovery_cleared_at timestamptz,
    ADD COLUMN recovery_clearance_reference text;

-- =====================================================================================================================
-- Step 4. The one changed released object (R52B §13.2, §18.1): the decision BE-04 explicitly deferred to this task.
-- =====================================================================================================================

ALTER TABLE public.distribution_job
  DROP CONSTRAINT distribution_job_work_id_fkey,
  ADD  CONSTRAINT distribution_job_work_id_fkey
       FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE SET NULL;

-- =====================================================================================================================
-- Step 5. New CHECK constraints, foreign keys and the guard trigger on the released tables (R52B §18.1, §18.2).
-- Every added CHECK is conditional on kind = 'WORK_UPSERT' or on the new nullable columns, so it holds vacuously for
-- every released PUBLISHER_BACK_CATALOGUE row.
-- =====================================================================================================================

ALTER TABLE public.distribution_job
  ADD CONSTRAINT distribution_job_work_upsert_profile_check
      CHECK ((kind = 'WORK_UPSERT') = (execution_profile IS NOT NULL)),
  ADD CONSTRAINT distribution_job_work_upsert_identity_check
      CHECK ((kind = 'WORK_UPSERT') = (work_identity IS NOT NULL)),
  ADD CONSTRAINT distribution_job_work_upsert_generation_check
      CHECK ((kind = 'WORK_UPSERT') = (created_generation IS NOT NULL)
             AND (created_generation IS NULL OR created_generation >= 1)),
  ADD CONSTRAINT distribution_job_work_upsert_ordinal_check
      CHECK ((kind = 'WORK_UPSERT') = (job_ordinal IS NOT NULL)
             AND (job_ordinal IS NULL OR job_ordinal >= 1)),
  -- work_id and work_identity agree while both are present
  ADD CONSTRAINT distribution_job_work_identity_agreement_check
      CHECK (work_id IS NULL OR work_identity IS NULL OR work_id = work_identity),
  -- BACKSTOP: an actionable work-level job can never reference a deleted Work
  ADD CONSTRAINT distribution_job_actionable_work_present_check
      CHECK (kind <> 'WORK_UPSERT' OR status NOT IN ('PENDING','RUNNING') OR work_id IS NOT NULL),
  ADD CONSTRAINT distribution_job_work_upsert_dedup_formula_check CHECK (
      kind <> 'WORK_UPSERT'
      OR deduplication_key = 'WORK_UPSERT:' || publisher_id::text
                              || ':' || work_identity::text
                              || ':' || execution_profile::text
                              || ':' || activation_id::text
                              || ':' || created_generation::text
                              || ':' || job_ordinal::text ),
  ADD CONSTRAINT distribution_job_no_self_predecessor_check
      CHECK (predecessor_job_id IS DISTINCT FROM distribution_job_id),
  ADD CONSTRAINT distribution_job_no_self_successor_check
      CHECK (superseded_by_job_id IS DISTINCT FROM distribution_job_id),
  ADD CONSTRAINT distribution_job_predecessor_fkey
      FOREIGN KEY (predecessor_job_id) REFERENCES public.distribution_job(distribution_job_id) ON DELETE SET NULL,
  ADD CONSTRAINT distribution_job_successor_fkey
      FOREIGN KEY (superseded_by_job_id) REFERENCES public.distribution_job(distribution_job_id) ON DELETE SET NULL;

ALTER TABLE public.distribution_job_attempt
    ADD CONSTRAINT distribution_job_attempt_claimed_generation_check
        CHECK (claimed_generation IS NULL OR claimed_generation >= 1),
    ADD CONSTRAINT distribution_job_attempt_recovery_clearance_pairing_check
        CHECK ((recovery_cleared_at IS NULL) = (recovery_clearance_reference IS NULL)),
    ADD CONSTRAINT distribution_job_attempt_recovery_requires_fence_check
        CHECK (recovery_cleared_at IS NULL OR fenced_at IS NOT NULL),
    ADD CONSTRAINT distribution_job_attempt_recovery_reference_check
        CHECK (recovery_clearance_reference IS NULL OR recovery_clearance_reference ~ '[^[:space:]]');

-- §18.1: work_identity is immutable, and a work_id set NULL by a Work deletion is never restored.
CREATE FUNCTION public.distribution_job_work_reference_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.work_identity IS DISTINCT FROM OLD.work_identity THEN
        RAISE EXCEPTION 'DISTRIBUTION_JOB_WORK_IDENTITY_IMMUTABLE';
    END IF;
    IF OLD.work_id IS NULL AND NEW.work_id IS NOT NULL THEN
        RAISE EXCEPTION 'DISTRIBUTION_JOB_WORK_REFERENCE_NOT_RESTORABLE';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER distribution_job_work_reference_guard BEFORE UPDATE ON public.distribution_job
    FOR EACH ROW EXECUTE FUNCTION public.distribution_job_work_reference_guard();

-- =====================================================================================================================
-- Step 6. The eight new tables (Amendment 3 §3.3), then the control and admission guards, including the two generic
-- TRUNCATE guards of Amendment 3 R-8.
-- =====================================================================================================================

-- §18.3: no foreign key to work; the lifecycle that replaces it is the flush of step 8 (CTO decision 5646180203).
CREATE TABLE public.work_upsert_generation (
    work_id           uuid NOT NULL,
    execution_profile public.distribution_platform NOT NULL,
    source_generation bigint NOT NULL DEFAULT 0 CHECK (source_generation >= 0),
    updated_at        timestamptz NOT NULL DEFAULT current_timestamp,
    PRIMARY KEY (work_id, execution_profile)
);

-- §18.3a: the transaction-local record of a transaction's capture obligations. No row survives its transaction.
CREATE TABLE public.work_upsert_capture_queue (
    entry_id   bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    txid       xid8 NOT NULL DEFAULT pg_current_xact_id(),
    entry_kind text NOT NULL CHECK (entry_kind IN ('OWNERS', 'DELETED', 'ARM', 'PROBE')),
    work_ids   uuid[] CHECK ((entry_kind IN ('OWNERS', 'DELETED')) = (work_ids IS NOT NULL))
);

-- §18.4
CREATE TABLE public.work_upsert_control (
    execution_profile public.distribution_platform PRIMARY KEY,
    capture_enabled   boolean NOT NULL DEFAULT false,
    execution_enabled boolean NOT NULL DEFAULT false,
    updated_at        timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT work_upsert_control_execution_requires_capture_check
        CHECK (NOT execution_enabled OR capture_enabled)
);

-- §18.5
CREATE TABLE public.work_upsert_admission (
    execution_profile  public.distribution_platform NOT NULL,
    publisher_id       uuid NOT NULL REFERENCES public.publisher(publisher_id) ON DELETE CASCADE,
    activation_id      uuid NOT NULL,
    evidence_reference text NOT NULL CONSTRAINT work_upsert_admission_evidence_reference_check
                            CHECK (evidence_reference ~ '[^[:space:]]'),
    admitted_at        timestamptz NOT NULL DEFAULT current_timestamp,
    actor              text NOT NULL CONSTRAINT work_upsert_admission_actor_check CHECK (actor ~ '[^[:space:]]'),
    PRIMARY KEY (execution_profile, publisher_id, activation_id)
);

-- §18.6
CREATE TABLE public.crossref_write_permit (
    permit_id            uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    route                public.crossref_write_route NOT NULL,
    scope                public.crossref_write_scope NOT NULL,
    state                public.crossref_write_permit_state NOT NULL DEFAULT 'RESERVED',
    reconciliation_state public.crossref_reconciliation_state,
    reservation_token    uuid NOT NULL DEFAULT public.uuid_generate_v4(),
    publisher_id         uuid REFERENCES public.publisher(publisher_id) ON DELETE SET NULL,
    publisher_identity   uuid NOT NULL,
    root_work_identity   uuid NOT NULL,
    distribution_job_id  uuid REFERENCES public.distribution_job(distribution_job_id) ON DELETE SET NULL,
    distribution_job_attempt_id uuid
        REFERENCES public.distribution_job_attempt(distribution_job_attempt_id) ON DELETE SET NULL,
    job_identity         uuid,
    attempt_identity     uuid,
    permit_generation    bigint,
    source_generation_witness bigint NOT NULL CHECK (source_generation_witness >= 0),
    doi_set_digest       text NOT NULL CHECK (doi_set_digest ~ '^[0-9a-f]{64}$'),
    doi_set_cardinality  integer NOT NULL CHECK (doi_set_cardinality >= 1),
    crossref_timestamp   bigint NOT NULL
        CHECK (crossref_timestamp BETWEEN 10000000000000000 AND 99999999999999999),
    doi_batch_id         text NOT NULL CHECK (doi_batch_id ~ '[^[:space:]]'),
    payload_digest       text CONSTRAINT crossref_write_permit_payload_digest_check
                              CHECK (payload_digest IS NULL OR payload_digest ~ '^[0-9a-f]{64}$'),
    operator_authorization_reference       text,
    reconciliation_annotation_reference    text,
    reconciliation_authorization_reference text,
    void_reason          public.crossref_void_reason,
    void_detail          text,
    void_authorization_reference text,
    issued_at            timestamptz NOT NULL DEFAULT clock_timestamp(),
    authorized_at        timestamptz,
    provider_reported_at timestamptz,
    reconciliation_annotated_at timestamptz,
    reconciled_at        timestamptz,
    closed_at            timestamptz,
    CONSTRAINT crossref_write_permit_authorized_manifest_check CHECK (
        state IN ('RESERVED','VOIDED') OR (payload_digest IS NOT NULL AND authorized_at IS NOT NULL)),
    CONSTRAINT crossref_write_permit_reserved_check CHECK (
        state <> 'RESERVED' OR (payload_digest IS NULL AND authorized_at IS NULL AND provider_reported_at IS NULL
                                AND void_reason IS NULL)),
    CONSTRAINT crossref_write_permit_void_check CHECK (
        (state = 'VOIDED') = (void_reason IS NOT NULL)
        AND (state <> 'VOIDED' OR (payload_digest IS NULL AND authorized_at IS NULL AND provider_reported_at IS NULL))),
    CONSTRAINT crossref_write_permit_operator_void_check CHECK (
        (void_reason IS NOT DISTINCT FROM 'OPERATOR_CLEANUP') = (void_authorization_reference IS NOT NULL)
        AND (void_authorization_reference IS NULL OR void_authorization_reference ~ '[^[:space:]]')),
    CONSTRAINT crossref_write_permit_explicit_void_detail_check CHECK (
        void_reason IS NULL OR void_reason NOT IN ('OWNER_ABANDONED','OPERATOR_CLEANUP')
        OR coalesce(void_detail, '') ~ '[^[:space:]]'),
    CONSTRAINT crossref_write_permit_closed_check CHECK (
        (state IN ('ACCEPTED','NONE_ATTEMPTED','VOIDED')) = (closed_at IS NOT NULL)),
    CONSTRAINT crossref_write_permit_report_check CHECK (
        provider_reported_at IS NULL
        OR (payload_digest IS NOT NULL AND state IN ('ACCEPTED','INDETERMINATE','NONE_ATTEMPTED'))),
    CONSTRAINT crossref_write_permit_reconciliation_state_check CHECK (
        (reconciliation_state IS DISTINCT FROM 'RECONCILED' OR state IN ('ACCEPTED','NONE_ATTEMPTED'))
        AND (reconciliation_state IS NULL OR reconciliation_state = 'RECONCILED' OR state = 'INDETERMINATE')),
    CONSTRAINT crossref_write_permit_reconciliation_evidence_check CHECK (
        (reconciliation_state IS NOT DISTINCT FROM 'RECONCILED') = (reconciliation_authorization_reference IS NOT NULL)
        AND (reconciliation_authorization_reference IS NULL) = (reconciled_at IS NULL)
        AND (reconciliation_annotation_reference IS NULL) = (reconciliation_annotated_at IS NULL)
        AND (reconciliation_annotation_reference IS NULL OR reconciliation_state IS NOT NULL)
        AND (reconciliation_state IS NULL OR reconciliation_state = 'RECONCILED'
             OR reconciliation_annotation_reference IS NOT NULL)
        AND coalesce(reconciliation_annotation_reference, 'x') ~ '[^[:space:]]'
        AND coalesce(reconciliation_authorization_reference, 'x') ~ '[^[:space:]]'),
    CONSTRAINT crossref_write_permit_work_route_check CHECK (
        route <> 'WORK_UPSERT'
        OR (permit_generation IS NOT NULL AND job_identity IS NOT NULL
            AND attempt_identity IS NOT NULL AND scope = 'SINGLE_ROOT_WORK')),
    CONSTRAINT crossref_write_permit_unit_route_check CHECK (
        route <> 'PUBLISHER_BACK_CATALOGUE'
        OR (job_identity IS NOT NULL AND attempt_identity IS NOT NULL AND permit_generation IS NULL
            AND scope = 'SINGLE_ROOT_WORK')),
    CONSTRAINT crossref_write_permit_jobless_route_check CHECK (
        route NOT IN ('LEGACY_SCHEDULED','MANUAL_RECOVERY')
        OR (distribution_job_id IS NULL AND job_identity IS NULL AND attempt_identity IS NULL
            AND permit_generation IS NULL)),
    CONSTRAINT crossref_write_permit_manual_route_check CHECK (
        (route = 'MANUAL_RECOVERY') = (operator_authorization_reference IS NOT NULL)
        AND (operator_authorization_reference IS NULL OR operator_authorization_reference ~ '[^[:space:]]'))
);

-- §16.3: the membership foreign key is NO ACTION, since a permit can never be deleted.
CREATE TABLE public.crossref_write_permit_doi (
    permit_id uuid NOT NULL REFERENCES public.crossref_write_permit(permit_id),
    doi       text NOT NULL,
    PRIMARY KEY (permit_id, doi),
    CONSTRAINT crossref_write_permit_doi_canonical_check CHECK (
        doi ~ '^https://doi\.org/10\.[0-9]{4,9}/[-._;()/:a-zA-Z0-9<>+\[\]]+$' AND doi = lower(doi))
);

-- §17.4
CREATE TABLE public.work_crossref_version_floor (
    floor_id    boolean PRIMARY KEY DEFAULT true CHECK (floor_id),
    floor_value bigint NOT NULL DEFAULT 0,
    updated_at  timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT work_crossref_version_floor_domain_check
        CHECK (floor_value = 0 OR floor_value = 99999999999999)
);

-- §22.5
CREATE TABLE public.crossref_version_floor_audit (
    audit_id                      uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    mutation_kind                 text NOT NULL,
    before_value                  bigint,
    after_value                   bigint,
    g6_attempt_id                 uuid,
    observation_id                uuid,
    g7_authorization_reference    text,
    authorization_register_digest text,
    actor                         text NOT NULL,
    occurred_at                   timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT crossref_version_floor_audit_shape_check CHECK (
        CASE WHEN mutation_kind = 'ADVANCE_VERSION_FLOOR' THEN
                 before_value IS NOT NULL AND after_value IS NOT NULL
             AND after_value > before_value
             AND g6_attempt_id IS NOT NULL AND observation_id IS NOT NULL
             AND authorization_register_digest ~ '^[0-9a-f]{64}$'
             AND g7_authorization_reference ~ '[^[:space:]]'
             ELSE g6_attempt_id IS NULL AND observation_id IS NULL
              AND g7_authorization_reference IS NULL
              AND authorization_register_digest IS NULL
        END )
);

-- §21.1: the control state machine. The CHECK of the table refuses (false, true).
CREATE FUNCTION public.work_upsert_control_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN RAISE EXCEPTION 'WORK_UPSERT_CONTROL_ROW_IS_PERMANENT'; END IF;
    IF NEW.execution_profile IS DISTINCT FROM OLD.execution_profile THEN RAISE EXCEPTION 'WORK_UPSERT_CONTROL_KEY_IMMUTABLE'; END IF;
    IF OLD.capture_enabled AND NOT NEW.capture_enabled THEN RAISE EXCEPTION 'WORK_UPSERT_CAPTURE_IS_MONOTONE'; END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER work_upsert_control_guard BEFORE UPDATE OR DELETE ON public.work_upsert_control
    FOR EACH ROW EXECUTE FUNCTION public.work_upsert_control_guard();

-- §18.5: an admission row is immutable, and a delete passes only once its publisher row is already gone, which is
-- exactly the referential cascade of a publisher deletion.
CREATE FUNCTION public.work_upsert_admission_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN RAISE EXCEPTION 'WORK_UPSERT_ADMISSION_IMMUTABLE'; END IF;
    IF EXISTS (SELECT 1 FROM public.publisher WHERE publisher_id = OLD.publisher_id) THEN
        RAISE EXCEPTION 'WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE';
    END IF;
    RETURN OLD;
END $$;
CREATE TRIGGER work_upsert_admission_guard BEFORE UPDATE OR DELETE ON public.work_upsert_admission
    FOR EACH ROW EXECUTE FUNCTION public.work_upsert_admission_guard();

-- Amendment 3 §3.2 (R-8): statement-level TRUNCATE guards for the two permanent generic operational tables. A statement
-- trigger never fires for the row-level referential cascade a publisher DELETE performs, so the admission guard's
-- publisher-cascade exception is untouched. The function is generic because Crossref state stays out of the generic
-- substrate (frozen rule 12).
CREATE FUNCTION public.work_upsert_refuse_truncate() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION '%', TG_ARGV[0]; END $$;
CREATE TRIGGER work_upsert_control_no_truncate BEFORE TRUNCATE ON public.work_upsert_control
    FOR EACH STATEMENT EXECUTE FUNCTION public.work_upsert_refuse_truncate('WORK_UPSERT_CONTROL_ROW_IS_PERMANENT');
CREATE TRIGGER work_upsert_admission_no_truncate BEFORE TRUNCATE ON public.work_upsert_admission
    FOR EACH STATEMENT EXECUTE FUNCTION public.work_upsert_refuse_truncate('WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE');

-- =====================================================================================================================
-- Step 7. The ten indexes of R52B §18.9, then the four functions that read BE-06 types, columns or tables, with their
-- assertions. PostgreSQL validates a LANGUAGE sql body at CREATE FUNCTION, so none of these can precede its objects.
-- =====================================================================================================================

CREATE INDEX work_upsert_generation_profile_idx
    ON public.work_upsert_generation (execution_profile, work_id);
CREATE INDEX work_upsert_capture_queue_txid_idx ON public.work_upsert_capture_queue (txid, entry_kind);
CREATE INDEX crossref_write_permit_doi_lookup_idx
    ON public.crossref_write_permit_doi (doi, permit_id);
CREATE UNIQUE INDEX distribution_job_one_actionable_work_upsert_idx
    ON public.distribution_job (work_id, execution_profile)
    WHERE kind = 'WORK_UPSERT' AND status IN ('PENDING','RUNNING');
CREATE INDEX crossref_write_permit_blocking_partial_idx
    ON public.crossref_write_permit (issued_at, permit_id)
    WHERE state IN ('RESERVED','AUTHORIZED','INDETERMINATE');
CREATE UNIQUE INDEX crossref_write_permit_one_per_work_upsert_attempt_idx
    ON public.crossref_write_permit (attempt_identity)
    WHERE route = 'WORK_UPSERT';
CREATE INDEX crossref_write_permit_job_idx     ON public.crossref_write_permit (job_identity);
CREATE INDEX crossref_write_permit_root_idx    ON public.crossref_write_permit (root_work_identity);
CREATE INDEX crossref_write_permit_attempt_idx ON public.crossref_write_permit (attempt_identity);
CREATE UNIQUE INDEX crossref_version_floor_audit_one_advance_per_g6_attempt_idx
    ON public.crossref_version_floor_audit (g6_attempt_id)
    WHERE mutation_kind = 'ADVANCE_VERSION_FLOOR';

-- §16.9: CALLED ON NULL INPUT is load-bearing. A STRICT variant returns NULL for AUTHORIZED + NULL, which must block.
CREATE FUNCTION public.crossref_is_blocking_write_permit(
    state public.crossref_write_permit_state,
    reconciliation_state public.crossref_reconciliation_state)
RETURNS boolean LANGUAGE sql IMMUTABLE CALLED ON NULL INPUT AS $$
    SELECT state IN ('RESERVED','AUTHORIZED','INDETERMINATE')
       AND (reconciliation_state IS DISTINCT FROM 'RECONCILED')
$$;

-- §16.9: argument-free by contract.
CREATE FUNCTION public.crossref_blocking_write_permit_count() RETURNS bigint LANGUAGE sql STABLE AS $$
    SELECT count(*) FROM public.crossref_write_permit p WHERE public.crossref_is_blocking_write_permit(p.state, p.reconciliation_state)
$$;
CREATE FUNCTION public.crossref_is_drained() RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT public.crossref_blocking_write_permit_count() = 0
$$;

-- §9.1: derived resolution, max(D, H), over rows whose work_id is non-NULL.
CREATE FUNCTION public.work_upsert_resolution(p_work uuid, p_profile public.distribution_platform) RETURNS bigint
LANGUAGE sql STABLE AS $$
    WITH g AS (
        SELECT j.status, j.cancellation_reason,
               COALESCE((SELECT max(a.claimed_generation) FROM public.distribution_job_attempt a
                          WHERE a.distribution_job_id = j.distribution_job_id), j.created_generation) AS gen
          FROM public.distribution_job j
         WHERE j.kind = 'WORK_UPSERT' AND j.work_id = p_work AND j.execution_profile = p_profile)
    SELECT GREATEST(COALESCE((SELECT max(gen) FROM g WHERE status = 'SUCCEEDED'), 0),
                    COALESCE((SELECT max(gen) FROM g WHERE status = 'FAILED'
                                   OR (status = 'CANCELLED' AND cancellation_reason = 'ADMINISTRATIVE')), 0))
$$;

DO $$
BEGIN
    IF NOT public.crossref_is_blocking_write_permit('AUTHORIZED', NULL)
       OR NOT public.crossref_is_blocking_write_permit('RESERVED', NULL) THEN
        RAISE EXCEPTION 'M2 assertion: crossref_is_blocking_write_permit';
    END IF;
END $$;

-- =====================================================================================================================
-- Step 8. Capture, and the one commit-time flush (R52B §8.4, §8.6; CTO decision 5646180203, Alternative A).
-- =====================================================================================================================

-- §8.4: the sixteen row triggers resolve an event's owners and record them. They take no generation row, no Work row and
-- no other lock; the flush below applies the transaction's obligations once, at transaction completion.
CREATE FUNCTION public.work_upsert_capture() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE owners uuid[] := ARRAY[]::uuid[];
BEGIN
    IF TG_TABLE_NAME IN ('work', 'title', 'abstract', 'contribution', 'publication', 'funding', 'issue', 'reference') THEN
        IF TG_OP <> 'INSERT' THEN owners := owners || OLD.work_id; END IF;
        IF TG_OP <> 'DELETE' THEN owners := owners || NEW.work_id; END IF;
    ELSIF TG_TABLE_NAME = 'work_relation' THEN
        IF TG_OP <> 'INSERT' THEN owners := owners || OLD.relator_work_id || OLD.related_work_id; END IF;
        IF TG_OP <> 'DELETE' THEN owners := owners || NEW.relator_work_id || NEW.related_work_id; END IF;
    ELSIF TG_TABLE_NAME = 'affiliation' THEN
        owners := ARRAY(SELECT c.work_id FROM public.contribution c WHERE c.contribution_id IN (
                        CASE WHEN TG_OP <> 'INSERT' THEN OLD.contribution_id END, CASE WHEN TG_OP <> 'DELETE' THEN NEW.contribution_id END));
    ELSIF TG_TABLE_NAME = 'location' THEN
        owners := ARRAY(SELECT p.work_id FROM public.publication p WHERE p.publication_id IN (
                        CASE WHEN TG_OP <> 'INSERT' THEN OLD.publication_id END, CASE WHEN TG_OP <> 'DELETE' THEN NEW.publication_id END));
    ELSIF TG_TABLE_NAME = 'contributor' THEN
        owners := ARRAY(SELECT c.work_id FROM public.contribution c WHERE c.contributor_id IN (
                        CASE WHEN TG_OP <> 'INSERT' THEN OLD.contributor_id END, CASE WHEN TG_OP <> 'DELETE' THEN NEW.contributor_id END));
    ELSIF TG_TABLE_NAME = 'institution' THEN
        owners := ARRAY(SELECT c.work_id FROM public.affiliation a JOIN public.contribution c ON c.contribution_id = a.contribution_id
                         WHERE a.institution_id IN (CASE WHEN TG_OP <> 'INSERT' THEN OLD.institution_id END,
                                                    CASE WHEN TG_OP <> 'DELETE' THEN NEW.institution_id END)
                        UNION
                        SELECT f.work_id FROM public.funding f
                         WHERE f.institution_id IN (CASE WHEN TG_OP <> 'INSERT' THEN OLD.institution_id END,
                                                    CASE WHEN TG_OP <> 'DELETE' THEN NEW.institution_id END));
    ELSIF TG_TABLE_NAME = 'series' THEN
        owners := ARRAY(SELECT i.work_id FROM public.issue i WHERE i.series_id IN (
                        CASE WHEN TG_OP <> 'INSERT' THEN OLD.series_id END, CASE WHEN TG_OP <> 'DELETE' THEN NEW.series_id END));
    ELSIF TG_TABLE_NAME = 'imprint' THEN
        owners := ARRAY(SELECT w.work_id FROM public.work w WHERE w.imprint_id IN (
                        CASE WHEN TG_OP <> 'INSERT' THEN OLD.imprint_id END, CASE WHEN TG_OP <> 'DELETE' THEN NEW.imprint_id END));
    ELSIF TG_TABLE_NAME = 'publisher' THEN
        owners := ARRAY(SELECT w.work_id FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id
                         WHERE i.publisher_id = NEW.publisher_id);
    END IF;
    owners := ARRAY(SELECT DISTINCT o FROM unnest(owners) o WHERE o IS NOT NULL ORDER BY o);
    IF TG_TABLE_NAME = 'work' AND TG_OP = 'DELETE' THEN
        INSERT INTO public.work_upsert_capture_queue (entry_kind, work_ids) VALUES ('DELETED', ARRAY[OLD.work_id]);
    ELSIF cardinality(owners) = 0 THEN
        RETURN NULL;
    END IF;
    IF cardinality(owners) > 0 THEN
        INSERT INTO public.work_upsert_capture_queue (entry_kind, work_ids) VALUES ('OWNERS', owners);
    END IF;
    IF coalesce(current_setting('be06.capture_armed', true), '') <> 'on' THEN
        INSERT INTO public.work_upsert_capture_queue (entry_kind) VALUES ('ARM');
        PERFORM set_config('be06.capture_armed', 'on', true);
    END IF;
    RETURN NULL;
END $$;

-- §8.2: the sixteen capture triggers, one per matrix row, AFTER ... FOR EACH ROW and not deferred. Written against the
-- physical table names (title and abstract are aliased work_title and work_abstract in schema.rs).
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF work_type, work_status, doi, edition, publication_date, withdrawn_date, place, license, landing_page, first_page, last_page, imprint_id ON public.work FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.title FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.abstract FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.contribution FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF orcid ON public.contributor FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.affiliation FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF institution_name, ror, institution_doi ON public.institution FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF publication_type, isbn, work_id ON public.publication FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF canonical, full_text_url, location_platform, publication_id ON public.location FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF grant_number, work_id, institution_id ON public.funding FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF issue_number, issue_ordinal, series_id, work_id ON public.issue FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF series_name, issn_print, issn_digital ON public.series FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE OF crossmark_doi, publisher_id ON public.imprint FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER UPDATE OF publisher_name ON public.publisher FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.reference FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();
CREATE TRIGGER work_upsert_capture AFTER INSERT OR DELETE OR UPDATE ON public.work_relation FOR EACH ROW EXECUTE FUNCTION public.work_upsert_capture();

-- §8.6: the one flush. A DEFERRABLE INITIALLY DEFERRED constraint trigger fires at transaction completion. The probe
-- establishes that this firing is the transaction's completion: a probe row's own event can only fire inside this INSERT
-- when this constraint is currently IMMEDIATE, which is exactly when the firing is not transaction completion.
CREATE FUNCTION public.work_upsert_capture_flush() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE me xid8 := pg_current_xact_id(); owners uuid[]; deleted uuid[]; roots uuid[]; wrote uuid[];
BEGIN
    IF NEW.entry_kind = 'PROBE' THEN
        PERFORM set_config('be06.capture_probe', 'fired', true);
        DELETE FROM public.work_upsert_capture_queue WHERE entry_id = NEW.entry_id;
        RETURN NULL;
    END IF;
    PERFORM set_config('be06.capture_probe', '', true);
    INSERT INTO public.work_upsert_capture_queue (entry_kind) VALUES ('PROBE');
    IF coalesce(current_setting('be06.capture_probe', true), '') = 'fired' THEN
        -- a caller has made deferrable constraints IMMEDIATE: this firing is not transaction completion. Restore this
        -- constraint's deferral, consume this arming row and arm again, so the flush still happens at completion.
        SET CONSTRAINTS public.work_upsert_capture_flush DEFERRED;
        DELETE FROM public.work_upsert_capture_queue WHERE entry_id = NEW.entry_id;
        INSERT INTO public.work_upsert_capture_queue (entry_kind) VALUES ('ARM');
        RETURN NULL;
    END IF;
    SELECT coalesce(array_agg(DISTINCT u ORDER BY u), ARRAY[]::uuid[]) INTO deleted
      FROM public.work_upsert_capture_queue q, unnest(q.work_ids) u WHERE q.txid = me AND q.entry_kind = 'DELETED';
    SELECT coalesce(array_agg(DISTINCT u ORDER BY u), ARRAY[]::uuid[]) INTO owners
      FROM public.work_upsert_capture_queue q, unnest(q.work_ids) u WHERE q.txid = me AND q.entry_kind = 'OWNERS';
    -- 1. end of generation, before anything is advanced: the generation of every Work this transaction deleted ends
    --    with it, taken ascending
    PERFORM 1 FROM public.work_upsert_generation WHERE work_id = ANY (deleted)
        ORDER BY work_id, execution_profile FOR UPDATE;
    DELETE FROM public.work_upsert_generation WHERE work_id = ANY (deleted);
    -- 2. the transaction's root set: the owners and their HAS_CHILD parents, resolved by MVCC, and the surviving roots
    --    upserted once each, ascending by work_id, in one statement
    roots := ARRAY(SELECT DISTINCT r FROM unnest(owners) o, unnest(public.crossref_roots(o)) r ORDER BY r);
    WITH written AS (
        INSERT INTO public.work_upsert_generation (work_id, execution_profile, source_generation)
        SELECT w.work_id, 'CROSSREF', 1 FROM public.work w WHERE w.work_id = ANY (roots) ORDER BY w.work_id
        ON CONFLICT (work_id, execution_profile)
        DO UPDATE SET source_generation = public.work_upsert_generation.source_generation + 1, updated_at = current_timestamp
        RETURNING work_id)
    SELECT coalesce(array_agg(work_id), ARRAY[]::uuid[]) INTO wrote FROM written;
    -- 3. a root whose Work vanished while the upsert waited keeps no generation row (a fresh READ COMMITTED snapshot)
    DELETE FROM public.work_upsert_generation g WHERE g.work_id = ANY (wrote) AND g.execution_profile = 'CROSSREF'
       AND NOT EXISTS (SELECT 1 FROM public.work w WHERE w.work_id = g.work_id);
    DELETE FROM public.work_upsert_capture_queue WHERE txid = me;
    PERFORM set_config('be06.capture_armed', '', true);
    RETURN NULL;
END $$;
CREATE CONSTRAINT TRIGGER work_upsert_capture_flush AFTER INSERT ON public.work_upsert_capture_queue
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (NEW.entry_kind IN ('ARM', 'PROBE'))
    EXECUTE FUNCTION public.work_upsert_capture_flush();

-- =====================================================================================================================
-- Step 9. The generic target-set constraint triggers of §11.4, and the eleven Crossref profile triggers (Amendment 3
-- §3.3; the baseline guard is removed).
-- =====================================================================================================================

-- §11.4: one deferred constraint-trigger function, armed on both tables, evaluated at COMMIT. It governs WORK_UPSERT jobs
-- only: a PUBLISHER_BACK_CATALOGUE job keeps its released target set.
CREATE FUNCTION public.work_upsert_target_set_check() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE jid uuid; k public.distribution_job_kind; prof public.distribution_platform; n bigint; ok boolean;
BEGIN
    jid := CASE WHEN TG_OP = 'DELETE' THEN OLD.distribution_job_id ELSE NEW.distribution_job_id END;
    SELECT j.kind, j.execution_profile INTO k, prof FROM public.distribution_job j WHERE j.distribution_job_id = jid;
    IF k IS DISTINCT FROM 'WORK_UPSERT' THEN RETURN NULL; END IF;
    CASE prof
      WHEN 'CROSSREF' THEN
        SELECT count(*), bool_and(t.platform = 'CROSSREF') INTO n, ok FROM public.distribution_job_target t WHERE t.distribution_job_id = jid;
        IF n <> 1 OR NOT ok THEN RAISE EXCEPTION 'WORK_UPSERT_TARGET_SET_MISMATCH' USING ERRCODE = 'check_violation'; END IF;
      ELSE
        RAISE EXCEPTION 'WORK_UPSERT_PROFILE_NOT_IMPLEMENTED' USING ERRCODE = 'check_violation';
    END CASE;
    RETURN NULL;
END $$;
CREATE CONSTRAINT TRIGGER work_upsert_target_set_job AFTER INSERT OR UPDATE ON public.distribution_job
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.work_upsert_target_set_check();
CREATE CONSTRAINT TRIGGER work_upsert_target_set_target AFTER INSERT OR UPDATE OR DELETE ON public.distribution_job_target
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.work_upsert_target_set_check();

-- §16.2 rule I1: a permit can only be inserted as a clean RESERVED row.
CREATE FUNCTION public.crossref_write_permit_insert_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF NEW.state <> 'RESERVED' OR NEW.payload_digest IS NOT NULL OR NEW.authorized_at IS NOT NULL
     OR NEW.provider_reported_at IS NOT NULL OR NEW.closed_at IS NOT NULL OR NEW.reconciliation_state IS NOT NULL
     OR NEW.reconciliation_annotation_reference IS NOT NULL OR NEW.reconciliation_annotated_at IS NOT NULL
     OR NEW.reconciliation_authorization_reference IS NOT NULL OR NEW.reconciled_at IS NOT NULL
     OR NEW.void_reason IS NOT NULL OR NEW.void_detail IS NOT NULL OR NEW.void_authorization_reference IS NOT NULL THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_INITIAL_STATE_INVALID';
  END IF;
  RETURN NEW;
END $$;
CREATE TRIGGER crossref_write_permit_insert_guard BEFORE INSERT ON public.crossref_write_permit
  FOR EACH ROW EXECUTE FUNCTION public.crossref_write_permit_insert_guard();

-- §16.2 rules U1-U3, S1-S3, T, A and D: the permit state machine, enforced against every statement in the API's role.
CREATE FUNCTION public.crossref_write_permit_fsm() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE g bigint; history bigint; floor_now bigint; reported boolean; annotated boolean; reconciled boolean;
BEGIN
  IF TG_OP = 'DELETE' THEN RAISE EXCEPTION 'CROSSREF_PERMIT_DELETE_REFUSED'; END IF;
  -- U1: immutable for life
  IF (NEW.permit_id, NEW.route, NEW.scope, NEW.reservation_token, NEW.publisher_identity, NEW.root_work_identity,
      NEW.job_identity, NEW.attempt_identity, NEW.permit_generation, NEW.source_generation_witness,
      NEW.crossref_timestamp, NEW.doi_batch_id, NEW.doi_set_digest, NEW.doi_set_cardinality, NEW.issued_at,
      NEW.operator_authorization_reference)
     IS DISTINCT FROM
     (OLD.permit_id, OLD.route, OLD.scope, OLD.reservation_token, OLD.publisher_identity, OLD.root_work_identity,
      OLD.job_identity, OLD.attempt_identity, OLD.permit_generation, OLD.source_generation_witness,
      OLD.crossref_timestamp, OLD.doi_batch_id, OLD.doi_set_digest, OLD.doi_set_cardinality, OLD.issued_at,
      OLD.operator_authorization_reference) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_EVIDENCE_IMMUTABLE';
  END IF;
  -- U2: operational links may only change to NULL
  IF (NEW.publisher_id IS NOT NULL AND NEW.publisher_id IS DISTINCT FROM OLD.publisher_id)
     OR (NEW.distribution_job_id IS NOT NULL AND NEW.distribution_job_id IS DISTINCT FROM OLD.distribution_job_id)
     OR (NEW.distribution_job_attempt_id IS NOT NULL
         AND NEW.distribution_job_attempt_id IS DISTINCT FROM OLD.distribution_job_attempt_id) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_LINK_NOT_RESTORABLE';
  END IF;
  -- U3: write-once fields never change once set
  IF (OLD.payload_digest IS NOT NULL AND NEW.payload_digest IS DISTINCT FROM OLD.payload_digest)
     OR (OLD.authorized_at IS NOT NULL AND NEW.authorized_at IS DISTINCT FROM OLD.authorized_at)
     OR (OLD.provider_reported_at IS NOT NULL AND NEW.provider_reported_at IS DISTINCT FROM OLD.provider_reported_at)
     OR (OLD.reconciliation_annotation_reference IS NOT NULL
         AND NEW.reconciliation_annotation_reference IS DISTINCT FROM OLD.reconciliation_annotation_reference)
     OR (OLD.reconciliation_annotated_at IS NOT NULL
         AND NEW.reconciliation_annotated_at IS DISTINCT FROM OLD.reconciliation_annotated_at)
     OR (OLD.reconciliation_authorization_reference IS NOT NULL
         AND NEW.reconciliation_authorization_reference IS DISTINCT FROM OLD.reconciliation_authorization_reference)
     OR (OLD.reconciled_at IS NOT NULL AND NEW.reconciled_at IS DISTINCT FROM OLD.reconciled_at)
     OR (OLD.closed_at IS NOT NULL AND NEW.closed_at IS DISTINCT FROM OLD.closed_at)
     OR (OLD.void_reason IS NOT NULL AND NEW.void_reason IS DISTINCT FROM OLD.void_reason)
     OR (OLD.void_detail IS NOT NULL AND NEW.void_detail IS DISTINCT FROM OLD.void_detail)
     OR (OLD.void_authorization_reference IS NOT NULL
         AND NEW.void_authorization_reference IS DISTINCT FROM OLD.void_authorization_reference) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  reported   := OLD.provider_reported_at IS NULL AND NEW.provider_reported_at IS NOT NULL;
  annotated  := (OLD.reconciliation_annotation_reference IS NULL AND NEW.reconciliation_annotation_reference IS NOT NULL)
             OR (OLD.reconciliation_annotated_at IS NULL AND NEW.reconciliation_annotated_at IS NOT NULL);
  reconciled := (OLD.reconciliation_authorization_reference IS NULL AND NEW.reconciliation_authorization_reference IS NOT NULL)
             OR (OLD.reconciled_at IS NULL AND NEW.reconciled_at IS NOT NULL);
  -- U3: each write-once field moves from NULL to a value only inside the act that owns it
  IF ((OLD.payload_digest IS NULL AND NEW.payload_digest IS NOT NULL) OR (OLD.authorized_at IS NULL AND NEW.authorized_at IS NOT NULL))
     AND NOT (OLD.state = 'RESERVED' AND NEW.state = 'AUTHORIZED') THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  IF ((OLD.void_reason IS NULL AND NEW.void_reason IS NOT NULL) OR (OLD.void_detail IS NULL AND NEW.void_detail IS NOT NULL)
      OR (OLD.void_authorization_reference IS NULL AND NEW.void_authorization_reference IS NOT NULL))
     AND NOT (OLD.state = 'RESERVED' AND NEW.state = 'VOIDED') THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  IF reported AND NOT (OLD.state = 'AUTHORIZED' AND NEW.state IN ('ACCEPTED','INDETERMINATE','NONE_ATTEMPTED')) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  IF annotated AND NOT (OLD.state IN ('AUTHORIZED','INDETERMINATE') AND NEW.state = 'INDETERMINATE'
                        AND OLD.reconciliation_state IS NULL
                        AND NEW.reconciliation_state IN ('RECONCILIATION_REQUIRED','RECONCILIATION_IMPOSSIBLE')) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  IF reconciled AND NOT (NEW.state IN ('ACCEPTED','NONE_ATTEMPTED') AND NEW.reconciliation_state = 'RECONCILED'
                         AND OLD.reconciliation_state IS DISTINCT FROM 'RECONCILED') THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;
  IF OLD.closed_at IS NULL AND NEW.closed_at IS NOT NULL
     AND NOT (OLD.state NOT IN ('ACCEPTED','NONE_ATTEMPTED','VOIDED') AND NEW.state IN ('ACCEPTED','NONE_ATTEMPTED','VOIDED')) THEN
    RAISE EXCEPTION 'CROSSREF_PERMIT_WRITE_ONCE_FIELD';
  END IF;

  -- S1-S3: the only same-state writes
  IF NEW.state = OLD.state THEN
    IF NOT reported AND NOT annotated AND NOT reconciled
       AND NEW.reconciliation_state IS NOT DISTINCT FROM OLD.reconciliation_state
       AND (NEW.payload_digest, NEW.authorized_at, NEW.closed_at, NEW.void_reason, NEW.void_detail, NEW.void_authorization_reference)
           IS NOT DISTINCT FROM
           (OLD.payload_digest, OLD.authorized_at, OLD.closed_at, OLD.void_reason, OLD.void_detail, OLD.void_authorization_reference) THEN
      RETURN NEW;                                              -- S1: operational links set NULL, nothing else
    END IF;
    IF OLD.state = 'INDETERMINATE' AND OLD.reconciliation_state IS NULL
       AND NEW.reconciliation_state IN ('RECONCILIATION_REQUIRED','RECONCILIATION_IMPOSSIBLE')
       AND NEW.reconciliation_annotation_reference IS NOT NULL AND NOT reconciled AND NOT reported THEN
      RETURN NEW;                                              -- S2: the annotation act, once
    END IF;
    IF OLD.state IN ('ACCEPTED','NONE_ATTEMPTED') AND OLD.reconciliation_state IS NULL
       AND NEW.reconciliation_state = 'RECONCILED' AND NEW.reconciliation_authorization_reference IS NOT NULL
       AND NOT annotated AND NOT reported THEN
      RETURN NEW;                                              -- S3: confirmation
    END IF;
    RAISE EXCEPTION 'CROSSREF_PERMIT_ILLEGAL_TRANSITION';
  END IF;

  -- T: the seven state-changing edges, and A: authorization preconditions re-proved at RESERVED -> AUTHORIZED
  IF OLD.state = 'RESERVED' AND NEW.state = 'AUTHORIZED' THEN
    IF NEW.payload_digest IS NULL OR NEW.authorized_at IS NULL THEN
      RAISE EXCEPTION 'CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_MANIFEST';
    END IF;
    SELECT source_generation INTO g FROM public.work_upsert_generation
     WHERE work_id = NEW.root_work_identity AND execution_profile = 'CROSSREF';
    IF g IS DISTINCT FROM NEW.source_generation_witness THEN RAISE EXCEPTION 'CROSSREF_ARTIFACT_SOURCE_CHANGED'; END IF;
    IF NEW.route = 'WORK_UPSERT' AND NOT EXISTS (
         SELECT 1 FROM public.distribution_job_attempt a
          WHERE a.distribution_job_attempt_id = NEW.attempt_identity AND a.finished_at IS NULL AND a.fenced_at IS NOT NULL) THEN
      RAISE EXCEPTION 'CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE';
    END IF;
    -- NEW is the permit being authorised; it overlaps its own membership and carries its own timestamp, so both
    -- queries exclude it by identity.
    IF EXISTS (SELECT 1
                 FROM public.crossref_write_permit_doi mine
                 JOIN public.crossref_write_permit_doi o ON o.doi = mine.doi
                 JOIN public.crossref_write_permit p ON p.permit_id = o.permit_id
                WHERE mine.permit_id = NEW.permit_id
                  AND p.permit_id <> NEW.permit_id
                  AND public.crossref_is_blocking_write_permit(p.state, p.reconciliation_state)) THEN
        RAISE EXCEPTION 'CROSSREF_PERMIT_BLOCKED';
    END IF;
    SELECT max(p.crossref_timestamp) INTO history
      FROM public.crossref_write_permit_doi mine
      JOIN public.crossref_write_permit_doi o ON o.doi = mine.doi
      JOIN public.crossref_write_permit p ON p.permit_id = o.permit_id
     WHERE mine.permit_id = NEW.permit_id
       AND p.permit_id <> NEW.permit_id
       AND p.state <> 'VOIDED';
    SELECT floor_value INTO floor_now FROM public.work_crossref_version_floor;
    IF NEW.crossref_timestamp <= floor_now
       OR (history IS NOT NULL AND NEW.crossref_timestamp <= history) THEN
        RAISE EXCEPTION 'CROSSREF_TIMESTAMP_NOT_INCREASING';
    END IF;
    RETURN NEW;
  ELSIF OLD.state = 'RESERVED' AND NEW.state = 'VOIDED' THEN
    RETURN NEW;
  ELSIF OLD.state = 'AUTHORIZED' AND NEW.state IN ('ACCEPTED','INDETERMINATE','NONE_ATTEMPTED') THEN
    IF reported AND NEW.reconciliation_state IS NULL THEN RETURN NEW; END IF;                 -- the outcome report
    IF NOT reported AND NEW.state IN ('ACCEPTED','NONE_ATTEMPTED')
       AND NEW.reconciliation_state = 'RECONCILED' AND reconciled THEN RETURN NEW; END IF;      -- a resolution
    IF NOT reported AND NEW.state = 'INDETERMINATE' AND annotated
       AND NEW.reconciliation_state IN ('RECONCILIATION_REQUIRED','RECONCILIATION_IMPOSSIBLE') THEN
      RETURN NEW;                                                                              -- an annotation
    END IF;
    RAISE EXCEPTION 'CROSSREF_PERMIT_ILLEGAL_TRANSITION';
  ELSIF OLD.state = 'INDETERMINATE' AND NEW.state IN ('ACCEPTED','NONE_ATTEMPTED') THEN
    IF NEW.reconciliation_state IS DISTINCT FROM 'RECONCILED' OR NOT reconciled THEN
      RAISE EXCEPTION 'CROSSREF_RECONCILIATION_REQUIRES_REFERENCE';
    END IF;
    RETURN NEW;
  END IF;
  RAISE EXCEPTION 'CROSSREF_PERMIT_ILLEGAL_TRANSITION';
END $$;
CREATE TRIGGER crossref_write_permit_fsm BEFORE UPDATE OR DELETE ON public.crossref_write_permit
  FOR EACH ROW EXECUTE FUNCTION public.crossref_write_permit_fsm();

-- §16.2 rule D: TRUNCATE of the permit table or of the membership table is refused CROSSREF_PERMIT_DELETE_REFUSED (§25.1).
CREATE FUNCTION public.crossref_refuse_truncate() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION '%', TG_ARGV[0]; END $$;
CREATE TRIGGER crossref_write_permit_no_truncate BEFORE TRUNCATE ON public.crossref_write_permit
    FOR EACH STATEMENT EXECUTE FUNCTION public.crossref_refuse_truncate('CROSSREF_PERMIT_DELETE_REFUSED');

-- §16.3: membership is immutable from the instant the reservation commits.
CREATE FUNCTION public.crossref_write_permit_doi_immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE'; END $$;
CREATE TRIGGER crossref_write_permit_doi_immutable BEFORE UPDATE OR DELETE ON public.crossref_write_permit_doi
    FOR EACH ROW EXECUTE FUNCTION public.crossref_write_permit_doi_immutable();
CREATE TRIGGER crossref_write_permit_doi_no_truncate BEFORE TRUNCATE ON public.crossref_write_permit_doi
    FOR EACH STATEMENT EXECUTE FUNCTION public.crossref_refuse_truncate('CROSSREF_PERMIT_DELETE_REFUSED');

-- §16.3: the deferred agreement of membership with the permit's digest and cardinality, armed on both tables. It must be
-- DEFERRABLE INITIALLY DEFERRED: a reservation inserts the permit row before its membership rows.
CREATE FUNCTION public.crossref_permit_membership_agreement() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE pid uuid; pd text; pc integer; d text; c bigint;
BEGIN
    pid := CASE WHEN TG_OP = 'DELETE' THEN OLD.permit_id ELSE NEW.permit_id END;
    SELECT doi_set_digest, doi_set_cardinality INTO pd, pc FROM public.crossref_write_permit WHERE permit_id = pid;
    IF NOT FOUND THEN RETURN NULL; END IF;
    SELECT public.crossref_doi_set_digest(array_agg(doi)), count(*) INTO d, c FROM public.crossref_write_permit_doi WHERE permit_id = pid;
    IF c <> pc OR d IS DISTINCT FROM pd THEN RAISE EXCEPTION 'CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH'; END IF;
    RETURN NULL;
END $$;
CREATE CONSTRAINT TRIGGER crossref_permit_membership_agreement_p AFTER INSERT OR UPDATE ON public.crossref_write_permit
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.crossref_permit_membership_agreement();
CREATE CONSTRAINT TRIGGER crossref_permit_membership_agreement_d AFTER INSERT OR UPDATE OR DELETE ON public.crossref_write_permit_doi
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.crossref_permit_membership_agreement();

-- §17.4: the version floor is monotone and permanent against every writer.
CREATE FUNCTION public.work_crossref_version_floor_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN RAISE EXCEPTION 'CROSSREF_VERSION_FLOOR_PERMANENT'; END IF;
    IF NEW.floor_value < OLD.floor_value THEN RAISE EXCEPTION 'CROSSREF_VERSION_FLOOR_NOT_DECREASING'; END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER work_crossref_version_floor_guard BEFORE UPDATE OR DELETE ON public.work_crossref_version_floor
    FOR EACH ROW EXECUTE FUNCTION public.work_crossref_version_floor_guard();
CREATE TRIGGER work_crossref_version_floor_no_truncate BEFORE TRUNCATE ON public.work_crossref_version_floor
    FOR EACH STATEMENT EXECUTE FUNCTION public.crossref_refuse_truncate('CROSSREF_VERSION_FLOOR_PERMANENT');

-- §22.5: the floor audit is append-only and permanent.
CREATE FUNCTION public.crossref_version_floor_audit_append_only() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY'; END $$;
CREATE TRIGGER crossref_version_floor_audit_append_only BEFORE UPDATE OR DELETE ON public.crossref_version_floor_audit
    FOR EACH ROW EXECUTE FUNCTION public.crossref_version_floor_audit_append_only();
CREATE TRIGGER crossref_version_floor_audit_no_truncate BEFORE TRUNCATE ON public.crossref_version_floor_audit
    FOR EACH STATEMENT EXECUTE FUNCTION public.crossref_refuse_truncate('CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY');

-- =====================================================================================================================
-- Step 10. The two seed rows, and their assertions. They are the only rows this migration writes.
-- =====================================================================================================================

INSERT INTO public.work_upsert_control (execution_profile, capture_enabled, execution_enabled) VALUES ('CROSSREF', false, false);
INSERT INTO public.work_crossref_version_floor (floor_id, floor_value) VALUES (true, 0);
DO $$
BEGIN
    IF (SELECT count(*) FROM public.work_upsert_control) <> 1
       OR NOT EXISTS (SELECT 1 FROM public.work_upsert_control
                       WHERE execution_profile = 'CROSSREF' AND NOT capture_enabled AND NOT execution_enabled)
       OR (SELECT count(*) FROM public.work_crossref_version_floor) <> 1
       OR (SELECT floor_value FROM public.work_crossref_version_floor) <> 0 THEN
        RAISE EXCEPTION 'M2 assertion: control row or floor';
    END IF;
END $$;
