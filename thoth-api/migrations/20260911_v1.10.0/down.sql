-- BE-06 Migration 2 (20260911_v1.10.0) down.sql: a stage-1 rollback only (R52B §23.3, §23.5).
--
-- It drops every BE-06 object -- the 35 triggers, the 27 functions, the 8 tables (the capture queue and the generation
-- table among them), the actionable-uniqueness index, and the constraints and columns added to the two released tables --
-- restores distribution_job_work_id_fkey to ON DELETE CASCADE, drops the 5 enum types, and leaves the three Migration 1
-- labels in place. Dropping the profile tables requires dropping their TRUNCATE and permanence guards first, in the same
-- transaction.
--
-- This is NOT a safe rollback once jobs or permits exist: permits are provider evidence and job rows are lifecycle
-- evidence (R52B §23.5). `thoth migrate --revert` is revert_all_migrations and is never a BE-06 rollback.
DROP TRIGGER IF EXISTS work_upsert_capture_flush ON public.work_upsert_capture_queue;
DROP FUNCTION IF EXISTS public.work_upsert_capture_flush();
DROP TRIGGER IF EXISTS crossref_version_floor_audit_no_truncate ON public.crossref_version_floor_audit;
DROP TRIGGER IF EXISTS crossref_version_floor_audit_append_only ON public.crossref_version_floor_audit;
DROP TRIGGER IF EXISTS work_crossref_version_floor_no_truncate ON public.work_crossref_version_floor;
DROP TRIGGER IF EXISTS work_crossref_version_floor_guard ON public.work_crossref_version_floor;
DROP TRIGGER IF EXISTS crossref_permit_membership_agreement_d ON public.crossref_write_permit_doi;
DROP TRIGGER IF EXISTS crossref_permit_membership_agreement_p ON public.crossref_write_permit;
DROP TRIGGER IF EXISTS crossref_write_permit_doi_no_truncate ON public.crossref_write_permit_doi;
DROP TRIGGER IF EXISTS crossref_write_permit_doi_immutable ON public.crossref_write_permit_doi;
DROP TRIGGER IF EXISTS crossref_write_permit_no_truncate ON public.crossref_write_permit;
DROP TRIGGER IF EXISTS crossref_write_permit_fsm ON public.crossref_write_permit;
DROP TRIGGER IF EXISTS crossref_write_permit_insert_guard ON public.crossref_write_permit;
DROP TRIGGER IF EXISTS work_upsert_target_set_target ON public.distribution_job_target;
DROP TRIGGER IF EXISTS work_upsert_target_set_job ON public.distribution_job;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.work;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.title;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.abstract;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.contribution;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.contributor;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.affiliation;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.institution;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.publication;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.location;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.funding;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.issue;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.series;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.imprint;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.publisher;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.reference;
DROP TRIGGER IF EXISTS work_upsert_capture ON public.work_relation;

DROP TRIGGER IF EXISTS work_upsert_admission_no_truncate ON public.work_upsert_admission;
DROP TRIGGER IF EXISTS work_upsert_control_no_truncate ON public.work_upsert_control;
DROP TRIGGER IF EXISTS work_upsert_admission_guard ON public.work_upsert_admission;
DROP TRIGGER IF EXISTS work_upsert_control_guard ON public.work_upsert_control;
DROP TRIGGER IF EXISTS distribution_job_work_reference_guard ON public.distribution_job;
DROP TABLE public.crossref_write_permit_doi, public.crossref_write_permit, public.crossref_version_floor_audit,
           public.work_crossref_version_floor, public.work_upsert_admission,
           public.work_upsert_control, public.work_upsert_generation, public.work_upsert_capture_queue;
DROP FUNCTION public.crossref_refuse_truncate(), public.crossref_write_permit_doi_immutable(), public.crossref_permit_membership_agreement(),
              public.work_crossref_version_floor_guard(), public.crossref_version_floor_audit_append_only(),
              public.work_upsert_refuse_truncate(), public.crossref_write_permit_insert_guard(), public.crossref_write_permit_fsm(),
              public.work_upsert_target_set_check(), public.work_upsert_capture(), public.work_upsert_admission_guard(),
              public.work_upsert_control_guard(), public.distribution_job_work_reference_guard(),
              public.work_upsert_resolution(uuid, public.distribution_platform), public.crossref_is_drained(),
              public.crossref_blocking_write_permit_count(),
              public.crossref_is_blocking_write_permit(public.crossref_write_permit_state, public.crossref_reconciliation_state),
              public.crossref_deposit_membership(uuid), public.crossref_roots(uuid), public.crossref_doi_set_digest(text[]),
              public.crossref_allocate_timestamp(bigint, bigint, bigint), public.crossref_ts_now(), public.crossref_ts_next(bigint),
              public.crossref_ts_decode(bigint), public.crossref_ts_encode(timestamptz), public.crossref_canonical_doi(text);
DROP INDEX IF EXISTS public.distribution_job_one_actionable_work_upsert_idx;
ALTER TABLE public.distribution_job
    DROP CONSTRAINT distribution_job_work_id_fkey,
    ADD  CONSTRAINT distribution_job_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE,
    DROP CONSTRAINT distribution_job_successor_fkey, DROP CONSTRAINT distribution_job_predecessor_fkey,
    DROP CONSTRAINT distribution_job_no_self_successor_check, DROP CONSTRAINT distribution_job_no_self_predecessor_check,
    DROP CONSTRAINT distribution_job_work_upsert_dedup_formula_check, DROP CONSTRAINT distribution_job_actionable_work_present_check,
    DROP CONSTRAINT distribution_job_work_identity_agreement_check, DROP CONSTRAINT distribution_job_work_upsert_ordinal_check,
    DROP CONSTRAINT distribution_job_work_upsert_generation_check, DROP CONSTRAINT distribution_job_work_upsert_identity_check,
    DROP CONSTRAINT distribution_job_work_upsert_profile_check,
    DROP COLUMN superseded_by_job_id, DROP COLUMN predecessor_job_id, DROP COLUMN job_ordinal, DROP COLUMN created_generation,
    DROP COLUMN work_identity, DROP COLUMN execution_profile;
ALTER TABLE public.distribution_job_attempt
    DROP COLUMN recovery_clearance_reference, DROP COLUMN recovery_cleared_at, DROP COLUMN fenced_at, DROP COLUMN claimed_generation;
DROP TYPE public.crossref_void_reason, public.crossref_write_scope,
          public.crossref_reconciliation_state, public.crossref_write_permit_state, public.crossref_write_route;
