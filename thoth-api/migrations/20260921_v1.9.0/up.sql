-- MET-WP7-PREREQ-02: durable unresolved-DOI quarantine (issue #930).
--
-- Additive only. Creates the single `metric_identifier_quarantine` table:
-- durable evidence of one eligible CloudFront observation whose DOI is
-- syntactically valid but resolves to no Thoth work. No row is seeded or
-- backfilled, and no existing table, constraint, index or row is touched.
--
-- Quarantine is not canonical acceptance. The observation it preserves stays a
-- `REJECTED` / `UNKNOWN_DOI` row in `metric_record_provenance`, which remains
-- the sole authoritative per-row classification ledger; it creates no
-- `metric_record`, `metric_record_revision` or rollup delta, and it is counted
-- in `metric_import.invalid_count`. This table only keeps the reduced,
-- lossless normalized observation so that a later, separately bounded
-- reconciliation can revalidate it once its DOI resolves. Nothing here
-- resolves, reconciles or clears a quarantine row.
--
-- Stored columns (reviewed): exactly the normalized observation fields the
-- approved CloudFront methodology needs, with the source account, platform and
-- measure resolved to the locked canonical rows the coordinator validated
-- against. `publication_isbn`, `publication_type`, `institution_ror`,
-- `source_record_id` and `source_row_number` are deliberately absent: an
-- observation carrying any of them is never quarantined, so their absence here
-- loses nothing. No request identity is stored: no IP address, user agent,
-- cookie, query string, referrer, request identifier, raw log row, session
-- identity, provider routing or credential.
--
-- DOI decision (reviewed): `work_doi` holds the observation's DOI exactly as
-- supplied to the coordinator after application-level `Doi::from_str`
-- validation: not lowercased, not re-prefixed, not rewritten. There is
-- deliberately no DOI-format CHECK. The application accepts bare and
-- `http(s)://(www.)(dx.)doi.org/` forms case-insensitively and matches its
-- registrant digits with Unicode-aware `\d`, so neither the `work` table's
-- canonical-form CHECK nor any obvious regular expression is equivalent to it,
-- and a stricter database rule would turn an application-valid row into a
-- request-level database failure.
--
-- Nonblank decision (reviewed): `schema_version`, `work_doi` and
-- `methodology_version` use the same locale-independent negated bracket as
-- `metric_source_driver_key_check`, listing every Unicode White_Space code
-- point as a \uXXXX escape. That is the exact twin of the coordinator's
-- `trim().is_empty()` test, so no value the coordinator accepts is refused
-- here, whatever the database's LC_CTYPE.
--
-- Foreign keys (reviewed): all four are non-cascading. Deleting a provenance
-- row, source account, platform or measure that still has quarantine evidence
-- fails instead of silently erasing it. The UNIQUE provenance key makes a
-- quarantine row the evidence of exactly one provenance row, and it is also
-- the index the checkpoint evidence join uses. No other index is created: no
-- other access path is approved.

CREATE TABLE public.metric_identifier_quarantine (
    identifier_quarantine_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_provenance_id uuid NOT NULL,
    source_account_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    schema_version text NOT NULL,
    work_doi text NOT NULL,
    period_start date NOT NULL,
    period_end date NOT NULL,
    reporting_grain public.metric_reporting_grain NOT NULL,
    country_code text,
    value bigint NOT NULL,
    methodology_version text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_identifier_quarantine_pkey PRIMARY KEY (identifier_quarantine_id),
    -- Exactly one quarantine row per provenance row.
    CONSTRAINT metric_identifier_quarantine_record_provenance_id_key
        UNIQUE (record_provenance_id),
    CONSTRAINT metric_identifier_quarantine_schema_version_check CHECK (
        schema_version ~ '[^\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000]'
    ),
    CONSTRAINT metric_identifier_quarantine_work_doi_check CHECK (
        work_doi ~ '[^\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000]'
    ),
    CONSTRAINT metric_identifier_quarantine_methodology_version_check CHECK (
        methodology_version ~ '[^\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000]'
    ),
    -- Half-open calendar-date period, exactly as on metric_record.
    CONSTRAINT metric_identifier_quarantine_period_check CHECK (period_end > period_start),
    CONSTRAINT metric_identifier_quarantine_record_provenance_id_fkey
        FOREIGN KEY (record_provenance_id)
        REFERENCES public.metric_record_provenance(record_provenance_id),
    CONSTRAINT metric_identifier_quarantine_source_account_id_fkey
        FOREIGN KEY (source_account_id)
        REFERENCES public.metric_source_account(source_account_id),
    CONSTRAINT metric_identifier_quarantine_platform_id_fkey
        FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_identifier_quarantine_measure_id_fkey
        FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);
