-- THOTH-ASYNC-01-ENGINE-01A (E1): revert the shared job engine's storage.
--
-- For disposable and development databases only. Once production holds any
-- `async_*` row, schema rollback is a forward-repair decision (ADR-0012
-- section 10), never a routine down migration.
--
-- Drops exactly what `up.sql` created, in dependency order, without CASCADE.

DROP TABLE public.async_job_transition;
DROP TABLE public.async_effect_target_state;
DROP TABLE public.async_target_hold;
DROP TABLE public.async_recovery_failure;
DROP TABLE public.async_job_attempt;
DROP TABLE public.async_control_command;
DROP TABLE public.async_job_target;
DROP TABLE public.async_executor_poll;
DROP TABLE public.async_job;

DROP FUNCTION public.async_job_before_insert();
DROP FUNCTION public.async_job_before_update();
DROP FUNCTION public.async_job_record_transition();
DROP FUNCTION public.async_job_target_guard();
DROP FUNCTION public.async_target_hold_guard();
DROP FUNCTION public.async_effect_target_state_guard();
DROP FUNCTION public.async_job_attempt_guard();
DROP FUNCTION public.async_control_command_guard();
DROP FUNCTION public.async_job_transition_guard();
DROP FUNCTION public.async_job_hold_check();
DROP FUNCTION public.async_job_attempt_check();
DROP FUNCTION public.async_control_command_decided_check();

DROP TYPE public.async_control_command_outcome;
DROP TYPE public.async_control_command_kind;
DROP TYPE public.async_attempt_result;
DROP TYPE public.async_attempt_purpose;
DROP TYPE public.async_job_lineage;
DROP TYPE public.async_job_revision_mode;
DROP TYPE public.async_job_effect_class;
DROP TYPE public.async_effect_phase;
DROP TYPE public.async_job_status;
