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
--        acquisition_type = DRIVER  -> driver_key contains a non-whitespace character
--        acquisition_type <> DRIVER -> driver_key IS NULL
--
--    "Whitespace" is one explicit, locale-independent set: the 25 code points
--    of the Unicode White_Space property, U+0009..U+000D, U+0020, U+0085,
--    U+00A0, U+1680, U+2000..U+200A, U+2028, U+2029, U+202F, U+205F and
--    U+3000. The application coordinator consults exactly the same set
--    (DRIVER_KEY_WHITESPACE in thoth-api/src/model/metric_source/mod.rs).
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
--
-- The nonblank test is a negated bracket expression listing every whitespace
-- code point as a \uXXXX escape: it matches when the key holds any character
-- outside that set. It deliberately does not use [:space:] or \s, whose
-- membership depends on LC_CTYPE (under the C locale they miss U+00A0 and
-- U+2003, so a key of only those characters would pass the database while
-- the coordinator refuses it). The pattern contains no character class, no
-- range and no case-insensitive flag, so no locale or collation participates.
ALTER TABLE public.metric_source
    ADD CONSTRAINT metric_source_driver_key_check CHECK (
        (acquisition_type = 'DRIVER'
            AND driver_key IS NOT NULL
            AND driver_key ~ '[^\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000]')
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
