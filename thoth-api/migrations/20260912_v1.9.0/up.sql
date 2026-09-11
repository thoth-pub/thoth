-- MET-WP1-13: Metrics source and source-account administration (issue #904).
--
-- Additive and initially inactive. Implements exactly the approved persistence
-- consequences of the protected SUPERUSER-only source/source-account
-- administration surface and nothing else: no source, source-account,
-- platform, publisher-approval or checkpoint row is seeded or rewritten, no
-- existing configuration JSON is touched, and no data cleanup runs.
--
-- 1. The DRIVER / driver_key invariant on metric_source, enforced at the
--    database boundary as one explicit named CHECK:
--
--        acquisition_type = DRIVER  -> driver_key contains a non-space character
--        acquisition_type <> DRIVER -> driver_key IS NULL
--
--    No driver registry, driver-key uniqueness or approved driver value is
--    introduced. The constraint is validated against existing rows when it is
--    added: a pre-existing row that violates the invariant makes this
--    migration fail closed rather than being silently rewritten.
--
-- 2. The bounded Metrics source-administration audit: two closed enums and
--    one append-only metric_source_registry_history table, written atomically
--    with every committed metric_source / metric_source_account create and
--    update. This deliberately does NOT extend metric_registry_history, whose
--    entity inventory MET-WP1-12 closed to PLATFORM, MEASURE and
--    PLATFORM_MEASURE; it is a parallel Metrics-local audit with the same
--    shape, not a generic cross-programme abstraction.
--
-- Audit shape decisions, all reviewed under #904 section 9:
--
--   * entity_id carries NO foreign key: the row is polymorphic append-only
--     evidence across two canonical tables and must not be cascade-deleted
--     with the state it describes.
--   * There is no updated_at column and therefore no timestamp trigger.
--   * There is no DELETE action value because no delete mutation exists.
--   * No secondary index is created: no approved audit access path exists.
--   * metric_source and metric_source_account gain no timestamp column.

-- The explicit IS NOT NULL matters: under SQL three-valued logic a NULL
-- driver_key makes `driver_key ~ '...'` NULL rather than FALSE, and a CHECK
-- whose whole expression is NULL is treated as satisfied. Without it a DRIVER
-- row with no key would pass.
ALTER TABLE public.metric_source
    ADD CONSTRAINT metric_source_driver_key_check CHECK (
        (acquisition_type = 'DRIVER'
            AND driver_key IS NOT NULL
            AND driver_key ~ '[^[:space:]]')
        OR (acquisition_type <> 'DRIVER' AND driver_key IS NULL)
    );

CREATE TYPE public.metric_source_registry_history_entity AS ENUM (
    'SOURCE',
    'SOURCE_ACCOUNT'
);

CREATE TYPE public.metric_source_registry_history_action AS ENUM (
    'CREATE',
    'UPDATE'
);

-- Both CHECK constraints are named explicitly so the application error
-- boundary can map them deterministically.
CREATE TABLE public.metric_source_registry_history (
    metric_source_registry_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    entity public.metric_source_registry_history_entity NOT NULL,
    entity_id uuid NOT NULL,
    action public.metric_source_registry_history_action NOT NULL,
    actor text NOT NULL,
    before_state jsonb,
    after_state jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_source_registry_history_pkey
        PRIMARY KEY (metric_source_registry_history_id),
    CONSTRAINT metric_source_registry_history_actor_check
        CHECK (actor ~ '[^[:space:]]'),
    CONSTRAINT metric_source_registry_history_action_before_state_check CHECK (
        (action = 'CREATE' AND before_state IS NULL)
        OR (action = 'UPDATE' AND before_state IS NOT NULL)
    )
);
