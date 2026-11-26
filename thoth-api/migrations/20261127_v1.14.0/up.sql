-- CITATIONS-01 C1 (thoth-pub/thoth#1074): storage for citation links.
--
-- Five new tables and two enum types; no existing table changes and nothing
-- is seeded. The registry (platform, source, source account) is created after
-- deploy through the registry write path, so `metric_registry_history`
-- records it. All foreign keys are non-cascading: a work with citation rows
-- cannot be deleted.
--
-- `lock_timeout` and `statement_timeout` bound the migration; a bound that
-- fires fails it closed.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

CREATE TYPE public.metric_citation_match_status AS ENUM ('MATCHED', 'NOT_FOUND');
CREATE TYPE public.metric_citation_link_status AS ENUM ('ACTIVE', 'WITHDRAWN');
CREATE TYPE public.metric_citation_refresh_status AS ENUM (
    'RUNNING', 'COMPLETED', 'COMPLETED_WITH_ERRORS', 'FAILED'
);

-- One row per (cited work, source): the outcome of the last attempt that
-- resolved the DOI, and the last MATCHED values, which a later NOT_FOUND or
-- failed attempt never erases.
CREATE TABLE public.metric_citation_work_state (
    cited_work_id uuid NOT NULL,
    source_id uuid NOT NULL,
    match_status public.metric_citation_match_status NOT NULL,
    last_matched_at timestamp with time zone,
    external_work_id text,
    reported_count integer,
    enumerated_count integer,
    guard_held_count integer,
    unconfirmed boolean DEFAULT false NOT NULL,
    last_attempted_at timestamp with time zone,
    last_refreshed_at timestamp with time zone,
    last_error_at timestamp with time zone,
    last_refresh_id uuid,
    CONSTRAINT metric_citation_work_state_pkey PRIMARY KEY (cited_work_id, source_id),
    CONSTRAINT metric_citation_work_state_cited_work_id_fkey
        FOREIGN KEY (cited_work_id) REFERENCES public.work(work_id),
    CONSTRAINT metric_citation_work_state_source_id_fkey
        FOREIGN KEY (source_id) REFERENCES public.metric_source(source_id),
    CONSTRAINT metric_citation_work_state_external_work_id_check
        CHECK (external_work_id IS NULL
               OR (external_work_id ~ '[^[:space:]]' AND char_length(external_work_id) <= 200)),
    CONSTRAINT metric_citation_work_state_reported_count_check
        CHECK (reported_count IS NULL OR reported_count >= 0),
    CONSTRAINT metric_citation_work_state_enumerated_count_check
        CHECK (enumerated_count IS NULL OR enumerated_count >= 0),
    CONSTRAINT metric_citation_work_state_guard_held_count_check
        CHECK (guard_held_count IS NULL OR guard_held_count >= 0),
    -- A MATCHED row always carries its identity and time.
    CONSTRAINT metric_citation_work_state_matched_check
        CHECK (match_status <> 'MATCHED'
               OR (last_matched_at IS NOT NULL AND external_work_id IS NOT NULL)),
    -- A work never matched holds no source values.
    CONSTRAINT metric_citation_work_state_never_matched_check
        CHECK (last_matched_at IS NOT NULL
               OR (external_work_id IS NULL AND reported_count IS NULL))
);

CREATE INDEX metric_citation_work_state_source_attempted_idx
    ON public.metric_citation_work_state (source_id, last_attempted_at);

-- One row per (cited work, source, citing work).
CREATE TABLE public.metric_citation_link (
    link_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    cited_work_id uuid NOT NULL,
    source_id uuid NOT NULL,
    citing_external_id text NOT NULL,
    citing_doi text,
    citing_title text,
    citing_type text,
    citing_publication_year smallint,
    citing_publication_date date,
    citing_sort_date date NOT NULL,
    citing_venue text,
    citing_authors jsonb,
    citing_url text,
    status public.metric_citation_link_status DEFAULT 'ACTIVE' NOT NULL,
    first_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    withdrawn_at timestamp with time zone,
    last_refresh_id uuid,
    CONSTRAINT metric_citation_link_pkey PRIMARY KEY (link_id),
    CONSTRAINT metric_citation_link_cited_work_id_fkey
        FOREIGN KEY (cited_work_id) REFERENCES public.work(work_id),
    CONSTRAINT metric_citation_link_source_id_fkey
        FOREIGN KEY (source_id) REFERENCES public.metric_source(source_id),
    CONSTRAINT metric_citation_link_citing_external_id_key
        UNIQUE (cited_work_id, source_id, citing_external_id),
    CONSTRAINT metric_citation_link_citing_external_id_check
        CHECK (citing_external_id ~ '[^[:space:]]' AND char_length(citing_external_id) <= 200),
    CONSTRAINT metric_citation_link_citing_doi_check
        CHECK (citing_doi IS NULL
               OR (citing_doi ~ '^10\.[^[:space:]]+$' AND citing_doi = lower(citing_doi)
                   AND char_length(citing_doi) <= 300)),
    CONSTRAINT metric_citation_link_citing_title_check
        CHECK (citing_title IS NULL OR char_length(citing_title) <= 1000),
    CONSTRAINT metric_citation_link_citing_type_check
        CHECK (citing_type IS NULL OR char_length(citing_type) <= 64),
    CONSTRAINT metric_citation_link_citing_venue_check
        CHECK (citing_venue IS NULL OR char_length(citing_venue) <= 500),
    CONSTRAINT metric_citation_link_citing_url_check
        CHECK (citing_url IS NULL
               OR (citing_url ~ '^https://[^/@[:space:]]+(/[^[:space:]]*)?$'
                   AND char_length(citing_url) <= 2048)),
    CONSTRAINT metric_citation_link_citing_authors_check
        CHECK (citing_authors IS NULL
               OR (jsonb_typeof(citing_authors) = 'object'
                   AND citing_authors ?& ARRAY['names', 'total']
                   AND (citing_authors - 'names' - 'total') = '{}'::jsonb
                   AND jsonb_typeof(citing_authors -> 'names') = 'array'
                   AND jsonb_array_length(citing_authors -> 'names') <= 5
                   AND jsonb_typeof(citing_authors -> 'total') = 'number'
                   AND pg_column_size(citing_authors) <= 2048)),
    CONSTRAINT metric_citation_link_withdrawn_check
        CHECK ((status = 'ACTIVE' AND withdrawn_at IS NULL)
               OR (status = 'WITHDRAWN' AND withdrawn_at IS NOT NULL))
);

-- One citing work counts once for a cited work and source while ACTIVE, even
-- when the source re-ids or merges it.
CREATE UNIQUE INDEX metric_citation_link_active_doi_key
    ON public.metric_citation_link (cited_work_id, source_id, citing_doi)
    WHERE citing_doi IS NOT NULL AND status = 'ACTIVE';

CREATE INDEX metric_citation_link_cited_order_idx
    ON public.metric_citation_link (cited_work_id, status, citing_sort_date DESC, link_id DESC);

CREATE INDEX metric_citation_link_source_seen_idx
    ON public.metric_citation_link (source_id, last_seen_at);

-- One row per claimed refresh run: audit, and the bound on what a completion
-- may sweep.
CREATE TABLE public.metric_citation_refresh (
    refresh_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_account_id uuid NOT NULL,
    claim_token_hash text NOT NULL,
    status public.metric_citation_refresh_status DEFAULT 'RUNNING' NOT NULL,
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    works_seen integer DEFAULT 0 NOT NULL,
    works_matched integer DEFAULT 0 NOT NULL,
    works_not_found integer DEFAULT 0 NOT NULL,
    links_upserted integer DEFAULT 0 NOT NULL,
    links_withdrawn integer DEFAULT 0 NOT NULL,
    links_refused integer DEFAULT 0 NOT NULL,
    links_merged integer DEFAULT 0 NOT NULL,
    last_error text,
    CONSTRAINT metric_citation_refresh_pkey PRIMARY KEY (refresh_id),
    CONSTRAINT metric_citation_refresh_source_account_id_fkey
        FOREIGN KEY (source_account_id) REFERENCES public.metric_source_account(source_account_id),
    CONSTRAINT metric_citation_refresh_claim_token_hash_check
        CHECK (claim_token_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_citation_refresh_counters_check
        CHECK (works_seen >= 0 AND works_matched >= 0 AND works_not_found >= 0
               AND links_upserted >= 0 AND links_withdrawn >= 0
               AND links_refused >= 0 AND links_merged >= 0),
    CONSTRAINT metric_citation_refresh_last_error_check
        CHECK (last_error IS NULL OR char_length(last_error) <= 1000),
    CONSTRAINT metric_citation_refresh_terminal_check
        CHECK ((status = 'RUNNING' AND completed_at IS NULL)
               OR (status <> 'RUNNING' AND completed_at IS NOT NULL))
);

-- At most one RUNNING refresh per account.
CREATE UNIQUE INDEX metric_citation_refresh_running_key
    ON public.metric_citation_refresh (source_account_id)
    WHERE status = 'RUNNING';

CREATE INDEX metric_citation_refresh_account_started_idx
    ON public.metric_citation_refresh (source_account_id, started_at DESC);

-- The citing ids seen per (refresh, work), refused and merged ones included.
-- Written by every batch, read by the completeness check and the sweep, and
-- deleted when its refresh reaches a terminal status.
CREATE TABLE public.metric_citation_refresh_seen (
    refresh_id uuid NOT NULL,
    cited_work_id uuid NOT NULL,
    citing_external_id text NOT NULL,
    CONSTRAINT metric_citation_refresh_seen_pkey
        PRIMARY KEY (refresh_id, cited_work_id, citing_external_id),
    CONSTRAINT metric_citation_refresh_seen_refresh_id_fkey
        FOREIGN KEY (refresh_id) REFERENCES public.metric_citation_refresh(refresh_id),
    CONSTRAINT metric_citation_refresh_seen_cited_work_id_fkey
        FOREIGN KEY (cited_work_id) REFERENCES public.work(work_id),
    CONSTRAINT metric_citation_refresh_seen_citing_external_id_check
        CHECK (char_length(citing_external_id) <= 200)
);

-- Replay safety: the same (refresh, batch key) with the same payload hash
-- returns its committed outcome; with another hash it is refused.
CREATE TABLE public.metric_citation_batch (
    refresh_id uuid NOT NULL,
    batch_key text NOT NULL,
    payload_hash text NOT NULL,
    outcome jsonb NOT NULL,
    committed_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_citation_batch_pkey PRIMARY KEY (refresh_id, batch_key),
    CONSTRAINT metric_citation_batch_refresh_id_fkey
        FOREIGN KEY (refresh_id) REFERENCES public.metric_citation_refresh(refresh_id),
    CONSTRAINT metric_citation_batch_batch_key_check
        CHECK (batch_key ~ '[^[:space:]]' AND char_length(batch_key) <= 200),
    CONSTRAINT metric_citation_batch_payload_hash_check
        CHECK (payload_hash ~ '[^[:space:]]')
);
