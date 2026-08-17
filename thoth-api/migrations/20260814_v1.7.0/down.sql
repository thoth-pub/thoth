-- BE-04 reversal.
--
-- Child-first, so the foreign keys do not block the drops. This is
-- reversibility evidence for a disposable database; dropping a **populated**
-- distribution_job, distribution_job_target or distribution_job_attempt in any
-- deployed environment destroys operational audit evidence of what was
-- attempted against external platforms and requires separate explicit
-- authorization (specification section 23.3).

DROP TABLE IF EXISTS public.distribution_job_attempt;
DROP TABLE IF EXISTS public.distribution_job_target;
DROP TABLE IF EXISTS public.distribution_job;

DROP TYPE IF EXISTS public.distribution_job_cancellation_reason;
DROP TYPE IF EXISTS public.distribution_job_attempt_result;
DROP TYPE IF EXISTS public.distribution_job_status;
DROP TYPE IF EXISTS public.distribution_job_kind;
