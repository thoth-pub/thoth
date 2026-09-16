-- BE-06 snapshot queries (thoth-pub/thoth#848)
--
-- Read-only state snapshots for the BE-06 release gates of R52B section 27.3, as modified by Amendments 1-3. Every
-- statement here only reads. The file opens a READ ONLY transaction and ends with ROLLBACK, so running it whole
-- cannot write. Run it with psql against the database under inspection, for example:
--
--   psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f docs/engineering/ai-delivery/BE-06-snapshot-queries.sql
--
-- These queries record state. They do not replace a gate's authorization, and none of them grants anything.
-- Where the API already provides a report, the report is authoritative and the query here is a cross-check:
--   * the census:                seedCrossrefWorkUpsert(data: {publisherId, limit: 0}) { remainingUncovered }
--   * blocking permits / drain:  crossrefBlockingWritePermitCount, crossrefDrained, crossrefUnresolvedPermits
--   * residue:                   workUpsertResidue, workUpsertResolutions, workUpsertCaptureLag
--
-- The census query in section 5 evaluates only the SQL clauses of Crossref eligibility (R52B section 14.2: E4, E5
-- and the row-level E6 clauses). Abstract normalisation is evaluated in Rust by the API, so this SQL census is an
-- upper bound: it can count a Work that fails only the abstract clause, which the API's census does not.

BEGIN TRANSACTION READ ONLY;

\echo '== 1. Migration ledger and the reserved Diesel versions (gates 3-11)'
SELECT version, run_on
  FROM __diesel_schema_migrations
 ORDER BY version DESC
 LIMIT 5;

SELECT (SELECT count(*) FROM __diesel_schema_migrations WHERE version = '20260910') AS migration_1_applied,
       (SELECT count(*) FROM __diesel_schema_migrations WHERE version = '20260911') AS migration_2_applied;

\echo '== 2. Object inventory (Amendment 3 section 3.3; tests G2-G4)'
SELECT (SELECT count(*) FROM pg_tables WHERE schemaname = 'public') AS tables,
       (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND NOT t.tgisinternal) AS user_triggers,
       (SELECT count(*) FROM pg_indexes WHERE schemaname = 'public') AS indexes,
       (SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
         WHERE n.nspname = 'public') AS functions,
       (SELECT count(*) FROM pg_type t JOIN pg_namespace n ON n.oid = t.typnamespace
         WHERE n.nspname = 'public' AND t.typtype = 'e') AS enum_types;

SELECT c.relname AS table_name, t.tgname AS trigger_name, t.tgenabled AS enabled
  FROM pg_trigger t
  JOIN pg_class c ON c.oid = t.tgrelid
  JOIN pg_namespace n ON n.oid = c.relnamespace
 WHERE n.nspname = 'public' AND NOT t.tgisinternal
   AND (t.tgname LIKE 'work_upsert%' OR t.tgname LIKE 'crossref%' OR t.tgname LIKE 'distribution_job_work%')
 ORDER BY c.relname, t.tgname;

-- No trigger may be disabled and the session must be in origin mode (R52B section 27.4).
SELECT current_setting('session_replication_role') AS session_replication_role,
       (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND NOT t.tgisinternal AND t.tgenabled <> 'O') AS disabled_user_triggers;

\echo '== 3. The inert state (gates 10 and 11): no BE-06 row other than generation rows'
SELECT execution_profile, capture_enabled, execution_enabled, updated_at
  FROM work_upsert_control
 ORDER BY execution_profile;

SELECT floor_value, updated_at FROM work_crossref_version_floor;

SELECT (SELECT count(*) FROM work_upsert_generation) AS generation_rows,
       (SELECT count(*) FROM work_upsert_capture_queue) AS capture_queue_rows,
       (SELECT count(*) FROM work_upsert_admission) AS admission_rows,
       (SELECT count(*) FROM crossref_write_permit) AS permit_rows,
       (SELECT count(*) FROM crossref_write_permit_doi) AS permit_membership_rows,
       (SELECT count(*) FROM crossref_version_floor_audit) AS floor_audit_rows,
       (SELECT count(*) FROM distribution_job WHERE kind = 'WORK_UPSERT') AS work_upsert_jobs;

-- No BE-06 label or column on a released row.
SELECT (SELECT count(*) FROM distribution_job
         WHERE execution_profile IS NOT NULL OR created_generation IS NOT NULL OR job_ordinal IS NOT NULL
            OR predecessor_job_id IS NOT NULL OR superseded_by_job_id IS NOT NULL
            OR cancellation_reason IN ('BINDING_SUPERSEDED', 'WORK_DELETED')) AS jobs_with_be06_columns,
       (SELECT count(*) FROM distribution_job_attempt
         WHERE claimed_generation IS NOT NULL OR fenced_at IS NOT NULL OR recovery_cleared_at IS NOT NULL)
         AS attempts_with_be06_columns;

\echo '== 4. Generation lifecycle (R52B sections 8.6 and 8.7)'
-- An orphaned generation row: a row whose Work no longer exists. Zero through every supported path.
SELECT count(*) AS orphaned_generation_rows
  FROM work_upsert_generation g
 WHERE NOT EXISTS (SELECT 1 FROM work w WHERE w.work_id = g.work_id);

-- A queue row outlives only an uncommitted transaction; outside one it is residue of a defect.
SELECT entry_kind, count(*) AS rows
  FROM work_upsert_capture_queue
 GROUP BY entry_kind
 ORDER BY entry_kind;

\echo '== 5. Coverage and the census, per publisher with an enabled CROSSREF assignment (gate 13)'
-- The census counts an eligible Work with no generation row, or with a 0 row, as uncovered (R52B section 21.3).
-- SQL-evaluable eligibility only: see the header note on abstract normalisation.
SELECT i.publisher_id,
       count(*) AS sql_eligible_works,
       count(*) FILTER (WHERE g.work_id IS NULL OR g.source_generation = 0) AS uncovered_upper_bound
  FROM work w
  JOIN imprint i ON i.imprint_id = w.imprint_id
  LEFT JOIN work_upsert_generation g ON g.work_id = w.work_id AND g.execution_profile = 'CROSSREF'
 WHERE EXISTS (SELECT 1 FROM publisher_distribution_platform a
                WHERE a.publisher_id = i.publisher_id AND a.platform = 'CROSSREF' AND a.enabled)
   AND (w.doi IS NOT NULL OR EXISTS (SELECT 1 FROM work_relation r JOIN work c ON c.work_id = r.related_work_id
                                      WHERE r.relator_work_id = w.work_id AND r.relation_type = 'has-child'
                                        AND c.doi IS NOT NULL))
   AND (w.work_status IN ('active', 'withdrawn') OR (w.work_status = 'forthcoming' AND w.publication_date IS NOT NULL))
   AND w.publication_date IS NOT NULL
   AND EXISTS (SELECT 1 FROM publication p WHERE p.work_id = w.work_id AND p.isbn IS NOT NULL)
   AND (w.doi IS NULL OR w.landing_page IS NOT NULL)
   AND EXISTS (SELECT 1 FROM title t WHERE t.work_id = w.work_id)
   AND NOT EXISTS (SELECT 1 FROM work_relation r JOIN work c ON c.work_id = r.related_work_id
                    WHERE r.relator_work_id = w.work_id AND r.relation_type = 'has-child' AND c.doi IS NOT NULL
                      AND (c.landing_page IS NULL OR c.edition IS NOT NULL
                           OR NOT EXISTS (SELECT 1 FROM title ct WHERE ct.work_id = c.work_id)))
 GROUP BY i.publisher_id
 ORDER BY i.publisher_id;

-- Admissions, with whether each names the publisher's current CROSSREF activation.
SELECT ad.publisher_id, ad.activation_id, ad.admitted_at, ad.actor,
       (ad.activation_id = a.activation_id AND a.enabled) AS names_current_activation
  FROM work_upsert_admission ad
  LEFT JOIN publisher_distribution_platform a
         ON a.publisher_id = ad.publisher_id AND a.platform = 'CROSSREF'
 ORDER BY ad.publisher_id, ad.admitted_at;

\echo '== 6. Residue and work-level jobs (gates 13, 16 and 20)'
-- Residue: a generation above its resolution (R52B section 9.1).
SELECT g.execution_profile,
       count(*) FILTER (WHERE g.source_generation > public.work_upsert_resolution(g.work_id, g.execution_profile))
         AS residue_works,
       count(*) AS generation_rows
  FROM work_upsert_generation g
 GROUP BY g.execution_profile;

SELECT status, cancellation_reason, count(*) AS jobs
  FROM distribution_job
 WHERE kind = 'WORK_UPSERT'
 GROUP BY status, cancellation_reason
 ORDER BY status, cancellation_reason;

-- More than one actionable job per Work and profile is a defect (the single creation helper).
SELECT work_id, execution_profile, count(*) AS actionable_jobs
  FROM distribution_job
 WHERE kind = 'WORK_UPSERT' AND status IN ('PENDING', 'RUNNING')
 GROUP BY work_id, execution_profile
HAVING count(*) > 1;

-- Fenced abandonments not yet cleared: they keep their jobs unclaimable (R52B section 11.7).
SELECT j.distribution_job_id, j.work_identity, a.distribution_job_attempt_id, a.result, a.fenced_at
  FROM distribution_job_attempt a
  JOIN distribution_job j ON j.distribution_job_id = a.distribution_job_id
 WHERE j.kind = 'WORK_UPSERT' AND a.fenced_at IS NOT NULL AND a.result = 'ABANDONED'
   AND a.recovery_cleared_at IS NULL
 ORDER BY a.fenced_at;

\echo '== 7. Permits (gates 15-18 and the section 27.4 stop conditions)'
SELECT route, state, reconciliation_state, count(*) AS permits
  FROM crossref_write_permit
 GROUP BY route, state, reconciliation_state
 ORDER BY route, state, reconciliation_state;

SELECT public.crossref_blocking_write_permit_count() AS blocking_permits,
       public.crossref_is_drained() AS drained;

-- Stop condition: a permit that reached AUTHORIZED, or any later provider state, without a bound payload digest.
SELECT permit_id, route, state
  FROM crossref_write_permit
 WHERE state NOT IN ('RESERVED', 'VOIDED') AND payload_digest IS NULL;

-- Stop condition: membership that no longer agrees with the permit's recorded cardinality.
SELECT p.permit_id, p.doi_set_cardinality, count(d.doi) AS membership_rows
  FROM crossref_write_permit p
  LEFT JOIN crossref_write_permit_doi d ON d.permit_id = p.permit_id
 GROUP BY p.permit_id, p.doi_set_cardinality
HAVING count(d.doi) <> p.doi_set_cardinality;

-- Stop condition before gate 16: any MANUAL_RECOVERY reservation.
SELECT count(*) AS manual_recovery_permits FROM crossref_write_permit WHERE route = 'MANUAL_RECOVERY';

-- Abandoned reservations: RESERVED permits whose job-linked attempt is already closed (gate 18).
SELECT p.permit_id, p.route, p.job_identity, p.attempt_identity, a.result AS attempt_result, p.issued_at
  FROM crossref_write_permit p
  JOIN distribution_job_attempt a ON a.distribution_job_attempt_id = p.attempt_identity
 WHERE p.state = 'RESERVED' AND a.finished_at IS NOT NULL
 ORDER BY p.issued_at;

-- Unresolved permits: every permit the blocking predicate counts (gates 17, 18 and 20).
SELECT permit_id, route, state, reconciliation_state, crossref_timestamp, authorized_at, provider_reported_at
  FROM crossref_write_permit
 WHERE public.crossref_is_blocking_write_permit(state, reconciliation_state)
 ORDER BY issued_at;

\echo '== 8. Version floor audit (gate 17)'
SELECT audit_id, mutation_kind, before_value, after_value, g6_attempt_id, observation_id,
       g7_authorization_reference, actor, occurred_at
  FROM crossref_version_floor_audit
 ORDER BY occurred_at;

ROLLBACK;
