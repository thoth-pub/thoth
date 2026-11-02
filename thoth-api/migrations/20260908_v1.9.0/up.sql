-- MET-WP1-12: Metrics registry administration audit foundation (issue #894).
--
-- Additive and initially inactive. Creates the two closed audit enums and the
-- single append-only metric_registry_history table that the protected
-- SUPERUSER-only registry administration surface writes atomically with every
-- committed metric_platform, metric_measure and metric_platform_measure
-- create/update. No existing table, column, constraint, index, trigger or row
-- is touched, and no row is seeded: the table is empty until an administrator
-- invokes one of the six approved mutations.
--
-- Audit-model decision (Amendment 1 section I): this is the Metrics-local
-- append-only before/after audit, following the newer publisher
-- service-configuration coordinator precedent rather than the older per-entity
-- *_history convention. It is deliberately NOT a generic cross-programme audit
-- abstraction.
--
-- Shape decisions, all reviewed:
--
--   * entity_id carries NO foreign key. The row is polymorphic append-only
--     evidence across three canonical registry tables, and audit evidence must
--     not be cascade-deleted with the registry state it describes.
--   * There is no updated_at column and therefore no diesel_manage_updated_at
--     trigger: the table is append-only and nothing rewrites a row.
--   * There is no DELETE action value because the administration surface
--     exposes no delete mutation.
--   * No secondary index is created. No approved audit-history query plan
--     exists yet, so the primary key is the complete intended index set; a
--     later bounded slice must derive any index from a concrete access path.
--   * metric_platform_measure gains no timestamps. The audit created_at
--     records mutation time without changing the approved design section 6.3
--     canonical row shape.

CREATE TYPE public.metric_registry_history_entity AS ENUM (
    'PLATFORM',
    'MEASURE',
    'PLATFORM_MEASURE'
);

CREATE TYPE public.metric_registry_history_action AS ENUM (
    'CREATE',
    'UPDATE'
);

-- The two CHECK constraints are named explicitly rather than left to
-- PostgreSQL's positional <table>_check naming, so the application error
-- boundary can map them deterministically and a future column addition cannot
-- silently renumber them.
CREATE TABLE public.metric_registry_history (
    metric_registry_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    entity public.metric_registry_history_entity NOT NULL,
    entity_id uuid NOT NULL,
    action public.metric_registry_history_action NOT NULL,
    actor text NOT NULL,
    before_state jsonb,
    after_state jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_registry_history_pkey PRIMARY KEY (metric_registry_history_id),
    CONSTRAINT metric_registry_history_actor_check CHECK (actor ~ '[^[:space:]]'),
    CONSTRAINT metric_registry_history_action_before_state_check CHECK (
        (action = 'CREATE' AND before_state IS NULL)
        OR (action = 'UPDATE' AND before_state IS NOT NULL)
    )
);
