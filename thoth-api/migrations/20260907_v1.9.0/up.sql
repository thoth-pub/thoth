-- MET-WP1-11: reconciliation ledger persistence foundation (issue #890).
--
-- Additive and initially inactive. Creates the two tightly coupled tables that
-- close the currently design-named WP1 persistence surface:
-- `metric_reconciliation_run`, one durable reconciliation execution, and
-- `metric_reconciliation_issue`, one machine-readable finding belonging to
-- exactly one run. They land together because an issue belongs to a run and
-- the two objects form one durable audit unit; splitting them would add a
-- migration and review cycle without creating an independently useful
-- persistence boundary.
--
-- This slice stores reconciliation structure only. There is deliberately no
-- reconciliation execution, no comparison of source manifests, canonical
-- records, rollups or OPERAS ledgers, no runtime creation of a run or an
-- issue, no issue classification, no closed status/type/severity vocabulary,
-- no resolution or reopening workflow, no OPERAS loop prevention, no
-- divergence handling, no snapshot or rolling scan, no completeness
-- determination, no claim, lease, retry or backoff behaviour, and no
-- `recordMetricReconciliation` or any other API operation: those belong to the
-- later bounded WP9 work. Nothing reads or writes a reconciliation row at
-- runtime, and the slice exposes no GraphQL, authorization or administration
-- surface.
--
-- Ledger boundary (reviewed): every OPERAS ledger this reconciliation contract
-- refers to is already merged and owned elsewhere. `metric_operas_mapping`
-- belongs to MET-WP1-08 (`20260904_v1.9.0`), `metric_operas_export` to
-- MET-WP1-09 (`20260905_v1.9.0`) and `metric_operas_import` to MET-WP1-10
-- (`20260906_v1.9.0`). This migration creates none of them and modifies none
-- of them.
--
-- COMPLETENESS BOUNDARY (section 15.5 — reviewed and load-bearing). Persisting
-- reconciliation runs and issues does NOT solve, weaken or narrow the OPERAS
-- inbound-completeness blocker, and no reader of this migration may treat it
-- as doing so. Guaranteed inbound completeness remains EXTERNALLY BLOCKED
-- without an adequate cursor/created-at event stream, replication, a complete
-- snapshot or export, or an equivalent reliable incremental mechanism. This
-- migration therefore adds NO completeness flag, coverage assertion, cursor,
-- scan or snapshot identifier, and implements NO completeness determination.
-- A populated reconciliation ledger would be evidence only of the comparisons
-- a later WP9 runtime actually performed — never evidence that reconciliation
-- was complete. WP9 owns completeness reporting and must surface unverified
-- completeness rather than claim it.
--
-- Identity decisions (reviewed): `run_id` and `issue_id` are repository-
-- standard Metrics UUID surrogate primary keys with the standard
-- `uuid_generate_v4()` default, matching the merged MET-WP1-09
-- `metric_operas_export.export_id`. The approved design names both IDs and
-- defines no natural composite identity for either, and no externally supplied
-- reconciliation-run identifier is part of the approved contract. No secondary
-- natural-key uniqueness is invented: in particular there is NO uniqueness over
-- `(run_id, issue_type, record_id, remote_event_id)` or any other inferred
-- issue identity, because runtime deduplication and reopening semantics are
-- not design-fixed and remain WP9-owned.
--
-- Structured-field decisions (reviewed): `scope`, `summary` and `details` are
-- JSONB, matching the established Metrics representation for structured
-- evidence and configuration — the MET-WP1-02 `metric_source_account
-- .configuration`, the MET-WP1-03 `metric_import.manifest` and the MET-WP1-04
-- `metric_record_provenance.details`. `scope` is NOT NULL with NO default,
-- because a run must state what it covered and this slice defines no default
-- coverage; `summary` and `details` are NOT NULL DEFAULT '{}', matching the
-- merged manifest/details idiom, so a row exists before its machine-readable
-- outcome is known. None of the three carries a database-level JSON schema,
-- required key, source/platform/measure identifier or completeness flag, and
-- an empty object is given no semantic interpretation. Reconciliation may
-- cover different combinations of source, canonical, rollup and OPERAS state,
-- so forcing a scalar code or promoting scope/summary columns would invent
-- runtime semantics that belong to WP9.
--
-- Required-text decisions (reviewed): `status`, `issue_type` and `severity`
-- are required TEXT carrying only the existing Metrics required-text CHECK,
-- which rejects blank and whitespace-only values. There is deliberately NO
-- PostgreSQL enum, NO CHECK enumerating values, NO default, NO trigger or
-- stored procedure and NO transition graph for any of them. The approved
-- design names the fields but fixes no closed vocabulary: its section 15.6
-- prose examples — missing export, unexpected remote record, value divergence,
-- unmapped measure, unresolved work, late source change, stale coverage — are
-- illustrative and must NOT be promoted to an exhaustive enum, and the
-- operational mention of high-severity reconciliation alerts must NOT be
-- turned into an `INFO`/`WARNING`/`ERROR`/`HIGH`/`CRITICAL` domain or mapped
-- onto another repository enum. WP9 owns those semantics.
--
-- Run timestamp decisions (reviewed, amended): `started_at` is
-- TIMESTAMPTZ NOT NULL with NO DEFAULT. This deliberately departs from the
-- repository-standard current-time default used for `created_at` elsewhere in
-- Metrics, because `started_at` is the actual reconciliation-execution start
-- supplied by the writer, and the approved design does not establish that
-- Thoth's durable insertion time and the reconciliation execution start are
-- the same event. The database must therefore never silently substitute
-- CURRENT_TIMESTAMP: an INSERT omitting `started_at` must fail, and an
-- explicitly supplied timestamp must round-trip exactly. This decision is
-- specific to the Metrics reconciliation contract and creates no repository-
-- wide rule for every column named `started_at`. `completed_at` is nullable
-- with no default, because the row exists before completion, and there is
-- deliberately NO status/timestamp state-machine CHECK tying `completed_at` to
-- `status`: the status vocabulary and its transition graph are not
-- design-fixed and remain WP9-owned. No `created_at`, `updated_at`, retry,
-- lease or heartbeat column is added.
--
-- Run relationship (reviewed): `metric_reconciliation_issue.run_id` is
-- required and a single-column non-cascading foreign key to
-- `metric_reconciliation_run (run_id)`. Every reconciliation issue is durable
-- evidence belonging to exactly one run, and deleting a run while its issues
-- exist must fail rather than silently cascade-delete that evidence. No
-- assumption is made that runs are ever deleted by normal runtime behaviour.
--
-- Canonical record relationship (reviewed): `record_id` is nullable with a
-- single-column non-cascading foreign key to `metric_record (record_id)`.
-- Nullable, because several design-named issue categories can exist before or
-- without a canonical record — unexpected remote records, unmapped measures
-- and unresolved works — so requiring `record_id` would make legitimate
-- reconciliation evidence unrepresentable. Referentially enforced when
-- present, so a named record cannot be a nonexistent one. Non-cascading,
-- matching every other Metrics foreign key. No key to `metric_record_revision`
-- is added, because the approved shorthand names `record_id`, not
-- `record_revision_id`.
--
-- NO REMOTE-EVENT FOREIGN KEY (reviewed and load-bearing): `remote_event_id`
-- is nullable opaque TEXT carrying the nullable form of the required-text
-- idiom — NULL, or at least one non-whitespace character — and NOTHING else.
-- There is deliberately NO foreign key from it to `metric_operas_import` or
-- `metric_operas_export`, and NO global uniqueness on it. The merged MET-WP1-10
-- inbound ledger established canonical remote identity as the composite
-- `(remote_instance, remote_event_id)` and deliberately established that a bare
-- `remote_event_id` is NOT globally unique. The approved reconciliation
-- shorthand contains no `remote_instance`, and reconciliation may also refer to
-- outbound or legacy remote evidence, so a single-column key to the inbound
-- ledger would contradict that merged identity contract and could make later
-- reconciliation ambiguous or falsely authoritative. `remote_instance`,
-- `operas_import_id`, `export_id`, `mapping_id` and `record_revision_id` are
-- likewise NOT added: no relationship may be manufactured merely to produce
-- referential integrity the approved design does not name. A later WP9 runtime
-- may carry richer evidence inside `details`, and may propose an additive
-- amendment only from a concrete demonstrated access or integrity requirement.
--
-- Resolution decision (reviewed): `resolved_at` is nullable TIMESTAMPTZ with no
-- default, exactly reflecting the design-named optional resolution timestamp.
-- No `resolved_by`, `resolution`, issue `status`, `updated_at`, reopening
-- counter or cross-column invariant is invented. This slice defines neither
-- what "resolved" means nor whether an issue may reopen.
--
-- Constraint-naming decision (reviewed): both primary keys, both foreign keys
-- and the four CHECKs use the PostgreSQL default naming shape
-- `<table>_<columns>_<pkey|fkey|check>`, matching the merged MET-WP1-10
-- `metric_operas_import_import_id_fkey` and MET-WP1-09
-- `metric_operas_export_record_revision_id_fkey` keys.
--
-- Index decision (reviewed, amended and closed): the complete intended index
-- inventory is exactly the primary-key index on `metric_reconciliation_run
-- (run_id)` and the primary-key index on `metric_reconciliation_issue
-- (issue_id)`. NO secondary index is created on `run_id`, `record_id`,
-- `remote_event_id`, `issue_type`, `severity`, `resolved_at` or any other
-- reconciliation column. PostgreSQL does not require a child-side referencing
-- index to enforce these foreign keys during ordinary child INSERT/UPDATE; the
-- closest merged Metrics parent/child evidence precedent,
-- `metric_import_error (import_id)`, deliberately carries no child-side FK
-- index; the approved design's section 14.4 specifies several concrete Metrics
-- operational indexes but none for reconciliation; and this slice is additive,
-- empty and has no reconciliation reader or writer, so no query plan exists to
-- justify one. WP9 may add reconciliation indexes only from an actual access
-- pattern with query-plan evidence.
--
-- Parent-before-child ordering: `metric_reconciliation_run` is created first
-- because `metric_reconciliation_issue` references it.
--
-- No reconciliation row is seeded, and no existing table, row, enum, index or
-- constraint is modified. In particular the MET-WP1-01 `metric_measure` seed
-- rows, the MET-WP1-03 `metric_import` schema and index, the MET-WP1-04
-- canonical record/revision/provenance schema and enums, the MET-WP1-05
-- coverage schema, the MET-WP1-06 publisher-platform approval schema, the
-- MET-WP1-07 rollup-delta schema, the MET-WP1-08 `metric_operas_mapping`
-- configuration and the MET-WP1-09/10 OPERAS export and import ledgers are
-- untouched, and no real reconciliation scope, status, issue type, severity,
-- remote event identifier or OPERAS mapping is approved or seeded.

CREATE TABLE public.metric_reconciliation_run (
    run_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    scope jsonb NOT NULL,
    status text NOT NULL,
    started_at timestamp with time zone NOT NULL,
    completed_at timestamp with time zone,
    summary jsonb DEFAULT '{}'::jsonb NOT NULL,
    CONSTRAINT metric_reconciliation_run_pkey PRIMARY KEY (run_id),
    CONSTRAINT metric_reconciliation_run_status_check
        CHECK (status ~ '[^[:space:]]')
);

CREATE TABLE public.metric_reconciliation_issue (
    issue_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    run_id uuid NOT NULL,
    issue_type text NOT NULL,
    severity text NOT NULL,
    record_id uuid,
    remote_event_id text,
    details jsonb DEFAULT '{}'::jsonb NOT NULL,
    resolved_at timestamp with time zone,
    CONSTRAINT metric_reconciliation_issue_pkey PRIMARY KEY (issue_id),
    CONSTRAINT metric_reconciliation_issue_run_id_fkey
        FOREIGN KEY (run_id)
        REFERENCES public.metric_reconciliation_run (run_id),
    CONSTRAINT metric_reconciliation_issue_record_id_fkey
        FOREIGN KEY (record_id)
        REFERENCES public.metric_record (record_id),
    CONSTRAINT metric_reconciliation_issue_issue_type_check
        CHECK (issue_type ~ '[^[:space:]]'),
    CONSTRAINT metric_reconciliation_issue_severity_check
        CHECK (severity ~ '[^[:space:]]'),
    CONSTRAINT metric_reconciliation_issue_remote_event_id_check
        CHECK (remote_event_id IS NULL OR remote_event_id ~ '[^[:space:]]')
);
