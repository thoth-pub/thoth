-- BE-06 Migration 1 (20260910_v1.10.0): enum ADD VALUE only, three statements (R52B §18.8, §23.2).
--
-- PostgreSQL 17 forbids using a value added by ALTER TYPE ... ADD VALUE before that transaction commits, and Diesel runs
-- each migration in its own transaction, so the labels are added here and first used by Migration 2
-- (20260911_v1.10.0). The labels are inert until Migration 2 creates anything that uses them.
ALTER TYPE public.distribution_job_kind                ADD VALUE 'WORK_UPSERT';
ALTER TYPE public.distribution_job_cancellation_reason ADD VALUE 'BINDING_SUPERSEDED';
ALTER TYPE public.distribution_job_cancellation_reason ADD VALUE 'WORK_DELETED';
