-- MET-WP1-08: OPERAS mapping persistence foundation (issue #882).
--
-- Additive and initially inactive. Creates the single `metric_operas_mapping`
-- table: the canonical durable configuration that names, for one registered
-- Metrics platform/measure pair, the OPERAS event, measure and uploader URIs
-- to use and whether that mapping is enabled. The approved Metrics design
-- treats `event_uri` as pre-registered mapping configuration rather than a
-- per-record value, and later outbound eligibility requires an enabled OPERAS
-- mapping for the platform/measure.
--
-- This slice stores mapping structure only. There is deliberately no OPERAS
-- export, import or reconciliation ledger, no payload construction, no
-- delivery, no claiming, lease, attempt, retry, backoff or status state
-- machine, no remote event ID, request hash or delivery error, no inbound
-- synchronization or loop prevention, no reconciliation, no cursor/snapshot
-- discovery, and no outbound eligibility or capability enforcement: those
-- belong to the later bounded WP5/WP9 work. The module exposes no GraphQL,
-- authorization or administration surface.
--
-- Deferred ledger boundary (reviewed): `metric_operas_export`,
-- `metric_operas_import`, `metric_reconciliation_run` and
-- `metric_reconciliation_issue` remain approved future architecture and are
-- deliberately NOT created here. The approved design fixes their conceptual
-- purpose but not their relational keys, status vocabularies, claim protocol
-- or reconciliation semantics, and this persistence-only slice must not
-- invent them.
--
-- Identity decision (reviewed): `mapping_id UUID PRIMARY KEY` is a stable
-- surrogate identity using the repository-standard Metrics UUID default. The
-- approved design's shorthand `metric_operas_mapping(...)` list names no
-- primary key, but its later conceptual `metric_operas_export` row refers to
-- a `mapping_id`. A surrogate key supplies that referential target directly,
-- without forcing a future export row to repeat mutable configuration text.
--
-- Uniqueness decision (reviewed): `UNIQUE(platform_id, measure_id)` permits
-- at most one canonical OPERAS mapping per registered platform/measure pair.
-- The approved design describes singular mapping configuration for one
-- platform/measure and phrases outbound eligibility as "the platform/measure
-- has an enabled OPERAS mapping"; permitting several simultaneously canonical
-- mappings would make both enabled-state and later `mapping_id` selection
-- ambiguous, and no version, priority or effective-date model is defined.
--
-- Referential-identity decision (reviewed): a mapping must configure a
-- platform/measure combination already admitted to the Metrics registry, so
-- the relationship is one composite foreign key over
-- `(platform_id, measure_id)` against the existing MET-WP1-01
-- `metric_platform_measure (platform_id, measure_id)` unique key. Two
-- independent single-column keys would admit a mapping naming a real platform
-- and a real measure that are not registered together as a supported pair.
-- Both columns are NOT NULL, so the MATCH SIMPLE composite key is always
-- enforced. No redundant standalone platform or measure foreign key is added,
-- because the composite key already supplies the complete intended
-- relationship.
--
-- The foreign key is deliberately non-cascading, matching every other Metrics
-- foreign key: deleting a registry pair that still has OPERAS mapping
-- configuration must fail rather than silently erase interoperability
-- configuration a later export path may depend on.
--
-- Direct-collection decision (reviewed): `metric_platform_measure`
-- .direct_collection remains the canonical flag preventing directly collected
-- platform/measure combinations being imported back from OPERAS. It is
-- deliberately NOT duplicated onto this table, where it could drift from the
-- registry.
--
-- URI decision (reviewed): `event_uri`, `measure_uri` and `uploader_uri` are
-- required configuration TEXT carrying only the existing Metrics required-text
-- integrity idiom, which rejects blank and whitespace-only values. No stronger
-- URI semantics are invented at this stage: no scheme restriction, no URI
-- parsing or normalization, no hostname allowlist, no trailing-slash rule, no
-- remote validation and no uniqueness on any URI column. Real OPERAS event,
-- measure and uploader URI values remain unapproved external inputs and are
-- recorded as unresolved in `docs/metrics/source-inventory.md`.
--
-- `enabled` is required BOOLEAN with deliberately NO database default: a
-- mapping's activation state must be stated explicitly by whatever later
-- reviewed administrative write path creates it, not silently resolved here.
--
-- Timestamp decision (reviewed): the approved mapping shorthand contains no
-- `created_at`/`updated_at`, and none is added merely to mirror other tables.
--
-- Constraint-naming decision (reviewed): the primary key, the uniqueness key,
-- the composite foreign key and the three required-text checks all use the
-- PostgreSQL default naming shape
-- `<table>_<columns>_<pkey|key|fkey|check>`, matching the merged MET-WP1-01
-- registry keys such as `metric_platform_measure_platform_id_measure_id_key`.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- primary-key index on `mapping_id` and the index PostgreSQL creates to
-- enforce `UNIQUE(platform_id, measure_id)`. No speculative index on
-- `enabled`, on any URI column or for any future export access path is
-- created: PostgreSQL builds no index for the referencing side of a foreign
-- key, and an access path must not be encoded before the WP9 export query and
-- its query-plan evidence are approved.
--
-- No mapping row is seeded, and no existing table, row, enum, index or
-- constraint is modified. In particular the two MET-WP1-01 `metric_measure`
-- seed rows are untouched.

CREATE TABLE public.metric_operas_mapping (
    mapping_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    event_uri text NOT NULL,
    measure_uri text NOT NULL,
    uploader_uri text NOT NULL,
    enabled boolean NOT NULL,
    CONSTRAINT metric_operas_mapping_pkey PRIMARY KEY (mapping_id),
    CONSTRAINT metric_operas_mapping_platform_id_measure_id_key
        UNIQUE (platform_id, measure_id),
    CONSTRAINT metric_operas_mapping_platform_id_measure_id_fkey
        FOREIGN KEY (platform_id, measure_id)
        REFERENCES public.metric_platform_measure (platform_id, measure_id),
    CONSTRAINT metric_operas_mapping_event_uri_check
        CHECK (event_uri ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_mapping_measure_uri_check
        CHECK (measure_uri ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_mapping_uploader_uri_check
        CHECK (uploader_uri ~ '[^[:space:]]')
);
