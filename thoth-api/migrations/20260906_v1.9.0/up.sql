-- MET-WP1-10: OPERAS import ledger persistence foundation (issue #888).
--
-- Additive and initially inactive. Creates the single `metric_operas_import`
-- table: the canonical durable record of one remote OPERAS event observed on
-- one remote OPERAS instance, the identity of the payload that event carried,
-- and an optional link to the canonical MET-WP1-03 import that normalized it.
-- The approved Metrics design assigns the OPERAS synchronization ledgers to
-- Thoth, and section 15.4 requires each remote event ID and payload hash to be
-- stored BEFORE normalization; this row is that durable remote-event evidence.
--
-- This slice stores inbound-ledger structure only. There is deliberately no
-- OPERAS network or API access, no provider or runtime inspection, no
-- discovery cursor, rolling scan or snapshot import, no remote polling or
-- scheduling, no normalization or metric ingestion, no automatic creation or
-- completion of a `metric_import`, no `direct_collection` eligibility
-- enforcement, no configured-uploader matching, no Thoth-export echo
-- detection, linking or skipping, no loop-prevention behaviour, no
-- payload-divergence handling, no reconciliation run or issue, no inbound
-- status vocabulary or transition graph, and no worker claim, lease, retry or
-- `FOR UPDATE SKIP LOCKED` logic: those belong to the later bounded WP9 work.
-- Nothing reads or writes an inbound-ledger row at runtime, and the slice
-- exposes no GraphQL, authorization or administration surface.
--
-- Deferred ledger boundary (reviewed): `metric_reconciliation_run` and
-- `metric_reconciliation_issue` remain approved future architecture and are
-- deliberately NOT created here. The outbound `metric_operas_export` ledger is
-- already owned by MET-WP1-09 and created by migration `20260905_v1.9.0`.
--
-- INBOUND-COMPLETENESS BOUNDARY (section 15.5 — reviewed and load-bearing).
-- Creating this ledger does NOT imply guaranteed inbound discovery, and no
-- reader of this migration may treat it as solving section 15.5. Guaranteed
-- inbound completeness remains EXTERNALLY BLOCKED without an adequate
-- cursor/created-at event stream, replication, a complete snapshot or export,
-- or an equivalent reliable incremental mechanism; nothing in this slice
-- removes that blocker. Accordingly this migration adds NO cursor field, NO
-- remote-created-at field and NO scan or snapshot identifier; it implements NO
-- rolling-scan or snapshot behaviour; and it performs NO provider or API
-- access. WP9 owns inbound discovery modes, loop prevention, reconciliation
-- and completeness reporting, and must surface unverified completeness rather
-- than claim it. A populated `metric_operas_import` is evidence only of the
-- remote events that were actually observed and recorded — never evidence that
-- all of them were.
--
-- INDEXING RECONCILIATION (section 14.4 — reviewed). The design's generic
-- requirement for import status and creation-time indexing is ALREADY
-- SATISFIED by the merged `metric_import_status_created_at_idx` on
-- `metric_import`, created by MET-WP1-03 migration `20260828_v1.9.0`. There is
-- therefore NO outstanding MET-WP1-10 OPERAS-import operational index
-- requirement, and this migration adds no speculative secondary index. WP9 may
-- add operational indexes only from actual query and query-plan evidence.
--
-- Identity decision (reviewed): the primary key is exactly the composite
-- `(remote_instance, remote_event_id)`. The approved design's section 6.14
-- names no surrogate inbound-ledger ID and deliberately carries
-- `remote_instance` alongside `remote_event_id`, so a bare remote event
-- identifier is NOT established as globally unique. One remote event observed
-- repeatedly must resolve to the same durable row rather than create duplicate
-- remote-event evidence, while the same `remote_event_id` stays representable
-- for two distinct remote instances. No surrogate UUID column, no global
-- uniqueness on `remote_event_id` and no additional identity column is added.
--
-- Required-text decisions (reviewed): `remote_instance`, `remote_event_id`,
-- `payload_hash` and `status` are required TEXT carrying only the existing
-- Metrics required-text CHECK, which rejects blank and whitespace-only values.
-- No URI, hostname, tenant/environment enum, registry, normalization or
-- case-folding rule constrains `remote_instance`; no syntax, length, UUID/URI
-- or global-uniqueness rule constrains `remote_event_id`; no algorithm,
-- encoding, case, length or uniqueness rule constrains `payload_hash`; and
-- there is NO PostgreSQL enum, NO CHECK enumerating status values, NO default,
-- NO trigger or stored procedure and NO cross-column rule tying `status` to
-- `import_id` or `payload_hash`. The approved design names these fields but
-- fixes none of those vocabularies, so none is invented; WP9 owns them.
--
-- Payload-hash cardinality decision (reviewed): `payload_hash` is deliberately
-- NOT unique. Two genuinely different remote events may legitimately carry
-- equal payload content, and forbidding that would make ordinary duplicate
-- content unrepresentable. The hash is payload identity evidence recorded
-- before normalization; how a changed payload for an already-known remote
-- identity becomes a divergence/reconciliation outcome is WP9-owned.
--
-- Import-linkage decisions (reviewed): `import_id` is nullable and non-unique,
-- with a single-column non-cascading foreign key to `metric_import`. It is
-- nullable because section 15.4 requires the remote event and its payload hash
-- to be recorded BEFORE normalization, and because an event that is linked or
-- skipped for loop prevention may never require a canonical import job of its
-- own; a durable remote-event row must therefore be able to exist before any
-- `metric_import` does. It is non-unique because one `metric_import` may
-- represent an API response or batch containing many distinct remote events,
-- so several inbound-ledger rows must be able to reference the same import.
-- The database decides neither when `import_id` is populated nor whether a
-- particular `status` requires it. The key is non-cascading, matching every
-- other Metrics foreign key: deleting a canonical import while durable
-- remote-event evidence references it must fail rather than silently erase the
-- evidence. PostgreSQL builds no index for the referencing side of a foreign
-- key; per the indexing note above, that is accepted at this inactive
-- foundation stage rather than pre-empted with a speculative index.
--
-- Timestamp decision (reviewed): `created_at` is required with the
-- repository-standard current-time default and records when Thoth stored the
-- row. It is deliberately the ONLY timestamp: no remote-created-at, discovery,
-- scan, snapshot, normalized-at, updated-at or completion timestamp is added,
-- because section 6.14 names none and because such a field would imply the
-- inbound discovery semantics section 15.5 leaves externally blocked.
--
-- No export-ledger relationship (reviewed): there is deliberately NO foreign
-- key or stored relationship to `metric_operas_export`, and no duplicated
-- export, platform, measure or mapping identifier. The approved inbound
-- shorthand contains none, and loop prevention is defined by WP9 matching
-- remote event and uploader evidence at runtime rather than by a stored
-- relational export identity on the inbound row.
--
-- Constraint-naming decision (reviewed): the composite primary key, the
-- foreign key and the four CHECKs all use the PostgreSQL default naming shape
-- `<table>_<columns>_<pkey|fkey|check>`, matching the merged MET-WP1-09
-- `metric_operas_export_record_revision_id_fkey` and MET-WP1-08
-- `metric_operas_mapping_platform_id_measure_id_fkey` keys.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- composite primary-key index on `(remote_instance, remote_event_id)`. No
-- index on `status`, `created_at`, `import_id`, `payload_hash`, bare
-- `remote_event_id` or any scan/cursor field may exist before WP9 approves the
-- actual query and its query-plan evidence.
--
-- No inbound-ledger row is seeded, and no existing table, row, enum, index or
-- constraint is modified. In particular the MET-WP1-01 `metric_measure` seed
-- rows, the MET-WP1-03 `metric_import` schema and its
-- `metric_import_status_created_at_idx` index, the MET-WP1-04 canonical
-- record/revision/provenance schema and enums, the MET-WP1-08
-- `metric_operas_mapping` configuration and the MET-WP1-09
-- `metric_operas_export` ledger are untouched, and no real OPERAS instance,
-- event identifier, payload hash or status value is approved or seeded.

CREATE TABLE public.metric_operas_import (
    remote_instance text NOT NULL,
    remote_event_id text NOT NULL,
    payload_hash text NOT NULL,
    import_id uuid,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_operas_import_pkey
        PRIMARY KEY (remote_instance, remote_event_id),
    CONSTRAINT metric_operas_import_import_id_fkey
        FOREIGN KEY (import_id)
        REFERENCES public.metric_import (import_id),
    CONSTRAINT metric_operas_import_remote_instance_check
        CHECK (remote_instance ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_remote_event_id_check
        CHECK (remote_event_id ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_payload_hash_check
        CHECK (payload_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_status_check
        CHECK (status ~ '[^[:space:]]')
);
