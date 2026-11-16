//! Operator commands (spec section 10.5): `CANCEL_JOB` (T14, T14w, T14r and
//! T14e), `RETRY_JOB`, `RESOLVE_RECONCILIATION` (T18) and
//! `REDACT_JOB_DIAGNOSTICS`.
//!
//! Every command validates its inputs, locks its job row, inserts its command
//! row (idempotent by `(command_kind, request_id)`), checks section 9 and the
//! kind policy, applies its change and records the decision on the command
//! row, all in the caller's transaction. A `REFUSED` outcome is a committed,
//! successful call returned as data, so the audit row survives. The command
//! `request` holds only scalar inputs and SHA-256 digests of the canonical
//! form of entries, evidence and checkpoints, never their content.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Bool, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::QueryableByName;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::engine::{
    apply_reconciliation_lease_expiry, claimed_attempt_id, close_attempt, evidence_context,
    insert_attempt, insert_planned_job, plan_creation, stored_payload, validate_targets, write_job,
    AttemptClose, CheckpointWrite, CreateOutcome, CreationPlan, JobChange, NewAsyncJob, NewAttempt,
};
use super::registry::{AsyncJobKindPolicy, AsyncJobRegistry, EvidenceUse, StaffOutcome};
use super::targets::{
    self, check_entries_semantic, check_entry_shapes, entries_json, EntryRule, EvidenceEntry,
    StateRow, StateWrite,
};
use super::{
    check_json_object, clock_now, db, force_hold_and_attempt_checks, is_non_blank, json_measure,
    lock_job, parse_json, read_job, require_no_nul, run_op, set_cause, AsyncAttemptPurpose,
    AsyncAttemptResult, AsyncControlCommandKind, AsyncControlCommandOutcome, AsyncEffectPhase,
    AsyncJob, AsyncJobLineage, AsyncJobStatus, EffectClass, JobRow, JsonFault, JsonInput,
    RevisionMode, UuidRow, PRINCIPAL_MAX_CHARS, REASON_MAX_CHARS, STAFF_ATTEMPT_LABEL,
};

// ---------------------------------------------------------------------------
// Transition causes of the commands (section 7.8)
// ---------------------------------------------------------------------------

pub(crate) const T14_CANCEL: &str = "T14_CANCEL";
pub(crate) const T14W_CANCEL_WAITING: &str = "T14W_CANCEL_WAITING";
pub(crate) const T14R_CANCEL_RUNNING: &str = "T14R_CANCEL_RUNNING";
pub(crate) const T14E_OPERATOR_STOP: &str = "T14E_OPERATOR_STOP";
pub(crate) const T18_STAFF_RESOLVE: &str = "T18_STAFF_RESOLVE";
pub(crate) const RETRY_JOB_CREATE: &str = "RETRY_JOB_CREATE";

// ---------------------------------------------------------------------------
// Inputs and results
// ---------------------------------------------------------------------------

/// The staff member behind a command. `principal` comes from the
/// authenticated token, never from input. `reason` is staff-authored audit
/// text (non-blank, at most 1000 characters) kept for the life of the record:
/// staff must not paste provider diagnostics or personal data into it.
/// `request_id` makes the command idempotent: a repeat with the same id and
/// inputs returns the stored outcome for ever, so a new attempt needs a new
/// id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffActor {
    pub principal: String,
    pub reason: String,
    pub request_id: Option<Uuid>,
}

/// The input of [`resolve_reconciliation_command_on`]. Which of `entries`
/// and `evidence` each outcome takes is section 13.3's evidence-argument
/// table; `WAITING` takes `checkpoint` and `resume_at` instead.
#[derive(Clone, PartialEq, Eq)]
pub struct StaffResolution {
    pub outcome: StaffOutcome,
    pub entries: Option<Vec<EvidenceEntry>>,
    /// A JSON object as text.
    pub evidence: Option<String>,
    /// A JSON object as text.
    pub checkpoint: Option<String>,
    pub resume_at: Option<DateTime<Utc>>,
}

impl std::fmt::Debug for StaffResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Evidence and checkpoints are never printed.
        f.debug_struct("StaffResolution")
            .field("outcome", &self.outcome)
            .field("entries", &self.entries.as_ref().map(Vec::len))
            .field("evidence", &self.evidence.is_some())
            .field("checkpoint", &self.checkpoint.is_some())
            .field("resume_at", &self.resume_at)
            .finish()
    }
}

/// A command's decision: `APPLIED`, or a committed `REFUSED` with its code,
/// and the job as it stands after the command.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub async_control_command_id: Uuid,
    pub command_kind: AsyncControlCommandKind,
    pub outcome: AsyncControlCommandOutcome,
    pub outcome_code: String,
    /// The new job of an applied `RETRY_JOB`.
    pub result_job_id: Option<Uuid>,
    /// The target job after the command (on a replay, as it is now).
    pub job: AsyncJob,
    /// True when the call returned the stored decision of an earlier call
    /// with the same request id and inputs.
    pub replayed: bool,
}

impl CommandResult {
    pub fn is_applied(&self) -> bool {
        self.outcome == AsyncControlCommandOutcome::Applied
    }
}

struct Decision {
    outcome: AsyncControlCommandOutcome,
    code: String,
    result_job_id: Option<Uuid>,
}

impl Decision {
    fn applied(code: &str) -> Self {
        Decision {
            outcome: AsyncControlCommandOutcome::Applied,
            code: code.to_string(),
            result_job_id: None,
        }
    }

    fn refused(code: &str) -> Self {
        Decision {
            outcome: AsyncControlCommandOutcome::Refused,
            code: code.to_string(),
            result_job_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// The shared command steps (section 10.5 steps 1 to 3)
// ---------------------------------------------------------------------------

/// The `StaffActor` checks: every fault is `ASYNC_JOB_PAYLOAD_INVALID`
/// before any write, because the command row could not be stored.
fn validate_actor(actor: &StaffActor) -> ThothResult<Uuid> {
    require_no_nul(&actor.principal, "actor principal")?;
    require_no_nul(&actor.reason, "reason")?;
    if !is_non_blank(&actor.principal) || actor.principal.chars().count() > PRINCIPAL_MAX_CHARS {
        return Err(ThothError::AsyncJobPayloadInvalid(
            "the actor principal must be non-blank and at most 256 characters".to_string(),
        ));
    }
    if !is_non_blank(&actor.reason) || actor.reason.chars().count() > REASON_MAX_CHARS {
        return Err(ThothError::AsyncJobPayloadInvalid(
            "the reason must be non-blank and at most 1000 characters".to_string(),
        ));
    }
    actor.request_id.ok_or_else(|| {
        ThothError::AsyncJobPayloadInvalid("the command needs a request id".to_string())
    })
}

/// A locked job with a new, undecided command row.
struct CommandContext {
    command_id: Uuid,
    kind: AsyncControlCommandKind,
    job: JobRow,
    now: DateTime<Utc>,
}

enum Opened {
    New(CommandContext),
    Replay(CommandResult),
}

#[derive(QueryableByName)]
struct StoredCommand {
    #[diesel(sql_type = SqlUuid)]
    async_control_command_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    async_job_id: Uuid,
    #[diesel(sql_type = Bool)]
    same_request: bool,
    #[diesel(sql_type = Nullable<Text>)]
    outcome: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    outcome_code: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    result_job_id: Option<Uuid>,
}

/// Steps 1 and 2: lock the job (`ASYNC_JOB_NOT_FOUND` when it does not
/// exist, nothing written), read `now`, insert the command row. On a
/// `(command_kind, request_id)` conflict, replay the stored decision when the
/// job and `request` are equal, else `ASYNC_COMMAND_REQUEST_CONFLICT`.
fn open_command(
    conn: &mut PgConnection,
    kind: AsyncControlCommandKind,
    actor: &StaffActor,
    request_id: Uuid,
    job_id: Uuid,
    request: &str,
) -> ThothResult<Opened> {
    let job = lock_job(conn, job_id)?.ok_or(ThothError::AsyncJobNotFound)?;
    let now = db(clock_now(conn))?;
    let inserted = db(sql_query(
        "INSERT INTO public.async_control_command \
             (command_kind, request_id, actor_principal, reason, async_job_id, request) \
         VALUES ($1::public.async_control_command_kind, $2, $3, $4, $5, $6::jsonb) \
         ON CONFLICT (command_kind, request_id) DO NOTHING \
         RETURNING async_control_command_id AS id",
    )
    .bind::<Text, _>(kind.as_str())
    .bind::<SqlUuid, _>(request_id)
    .bind::<Text, _>(&actor.principal)
    .bind::<Text, _>(&actor.reason)
    .bind::<SqlUuid, _>(job_id)
    .bind::<Text, _>(request)
    .get_result::<UuidRow>(conn)
    .optional())?;
    if let Some(row) = inserted {
        return Ok(Opened::New(CommandContext {
            command_id: row.id,
            kind,
            job,
            now,
        }));
    }
    let stored = db(sql_query(
        "SELECT async_control_command_id, async_job_id, (request = $3::jsonb) AS same_request, \
                outcome::text AS outcome, outcome_code, result_job_id \
           FROM public.async_control_command \
          WHERE command_kind = $1::public.async_control_command_kind AND request_id = $2",
    )
    .bind::<Text, _>(kind.as_str())
    .bind::<SqlUuid, _>(request_id)
    .bind::<Text, _>(request)
    .get_result::<StoredCommand>(conn))?;
    if stored.async_job_id != job_id || !stored.same_request {
        return Err(ThothError::AsyncCommandRequestConflict);
    }
    let outcome = match stored.outcome.as_deref() {
        Some("APPLIED") => AsyncControlCommandOutcome::Applied,
        Some("REFUSED") => AsyncControlCommandOutcome::Refused,
        // A committed command always has an outcome.
        _ => return Err(ThothError::AsyncJobIntegrityViolation),
    };
    Ok(Opened::Replay(CommandResult {
        async_control_command_id: stored.async_control_command_id,
        command_kind: kind,
        outcome,
        outcome_code: stored
            .outcome_code
            .ok_or(ThothError::AsyncJobIntegrityViolation)?,
        result_job_id: stored.result_job_id,
        job: job.into(),
        replayed: true,
    }))
}

/// Step 3's end: record the decision on the command row and return it with
/// the job as it now stands.
fn record_decision(
    conn: &mut PgConnection,
    context: &CommandContext,
    decision: Decision,
) -> ThothResult<CommandResult> {
    let updated = db(sql_query(
        "UPDATE public.async_control_command SET \
             outcome = $2::public.async_control_command_outcome, \
             outcome_code = $3, \
             decided_at = $4, \
             result_job_id = $5 \
         WHERE async_control_command_id = $1",
    )
    .bind::<SqlUuid, _>(context.command_id)
    .bind::<Text, _>(decision.outcome.as_str())
    .bind::<Text, _>(&decision.code)
    .bind::<Timestamptz, _>(context.now)
    .bind::<Nullable<SqlUuid>, _>(decision.result_job_id)
    .execute(conn))?;
    if updated != 1 {
        return Err(ThothError::AsyncJobIntegrityViolation);
    }
    let job =
        read_job(conn, context.job.async_job_id)?.ok_or(ThothError::AsyncJobIntegrityViolation)?;
    Ok(CommandResult {
        async_control_command_id: context.command_id,
        command_kind: context.kind,
        outcome: decision.outcome,
        outcome_code: decision.code,
        result_job_id: decision.result_job_id,
        job: job.into(),
        replayed: false,
    })
}

/// Run one command: open it, decide it unless it is a replay, record the
/// decision. The body runs inside `run_op`, so the three deferred checks are
/// forced once, at the end.
fn run_command(
    conn: &mut PgConnection,
    kind: AsyncControlCommandKind,
    actor: &StaffActor,
    request_id: Uuid,
    job_id: Uuid,
    request: &str,
    decide: impl FnOnce(&mut PgConnection, &mut CommandContext) -> ThothResult<Decision>,
) -> ThothResult<CommandResult> {
    match open_command(conn, kind, actor, request_id, job_id, request)? {
        Opened::Replay(result) => Ok(result),
        Opened::New(mut context) => {
            let decision = decide(conn, &mut context)?;
            record_decision(conn, &context, decision)
        }
    }
}

/// The `request` of a command with no input but the job: `{}`.
const EMPTY_REQUEST: &str = "{}";

// ---------------------------------------------------------------------------
// CANCEL_JOB (T14, T14w, T14r, T14e)
// ---------------------------------------------------------------------------

/// Cancel a job (T14 from `PENDING` or `RETRY_SCHEDULED`, T14w from `WAITING`
/// and T14r from `RUNNING` at `PRE_WRITE`, as the kind's `cancellable`
/// policy allows), or stop it after effect start (T14e: `RUNNING` or
/// `WAITING` at `EFFECT_STARTED` or later goes to `RECONCILIATION_REQUIRED`
/// whatever the policy). RR and terminal jobs are `NOT_CANCELLABLE`.
pub fn cancel_job_command_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    actor: &StaffActor,
    job_id: Uuid,
) -> ThothResult<CommandResult> {
    let request_id = validate_actor(actor)?;
    run_op(conn, |conn| {
        run_command(
            conn,
            AsyncControlCommandKind::CancelJob,
            actor,
            request_id,
            job_id,
            EMPTY_REQUEST,
            |conn, context| decide_cancel(conn, registry, context),
        )
    })
}

fn decide_cancel(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    context: &mut CommandContext,
) -> ThothResult<Decision> {
    let job = &context.job;
    let now = context.now;
    let Some(policy) = registry.get(&job.kind) else {
        return Ok(Decision::refused("KIND_UNKNOWN"));
    };
    let pre_write = job.effect_phase == AsyncEffectPhase::PreWrite;
    let allowed_by_policy = match job.status {
        AsyncJobStatus::Pending => Some(policy.cancellable.pending),
        AsyncJobStatus::RetryScheduled => Some(policy.cancellable.retry_scheduled),
        AsyncJobStatus::Waiting if pre_write => Some(policy.cancellable.waiting),
        AsyncJobStatus::Running if pre_write => Some(policy.cancellable.running),
        // T14e and the uncancellable states do not consult the policy.
        _ => None,
    };
    if allowed_by_policy == Some(false) {
        return Ok(Decision::refused("KIND_POLICY"));
    }
    let command_id = Some(context.command_id);
    let mut change = JobChange::from_row(job);
    match (job.status, pre_write) {
        (AsyncJobStatus::Pending | AsyncJobStatus::RetryScheduled, _) => {
            // T14: no holds to release.
            change.terminate(AsyncJobStatus::Cancelled, now);
            change.outcome("CANCELLED");
            db(set_cause(conn, T14_CANCEL, None, command_id))?;
            db(write_job(conn, job.async_job_id, &change))?;
            Ok(Decision::applied("CANCELLED"))
        }
        (AsyncJobStatus::Waiting, true) => {
            // T14w: release holds.
            db(targets::lock_holds_waiting(conn, &job.target_keys))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
            change.terminate(AsyncJobStatus::Cancelled, now);
            change.outcome("CANCELLED");
            db(set_cause(conn, T14W_CANCEL_WAITING, None, command_id))?;
            db(write_job(conn, job.async_job_id, &change))?;
            Ok(Decision::applied("CANCELLED"))
        }
        (AsyncJobStatus::Running, true) => {
            // T14r: the token is invalidated, the attempt closed, holds released.
            let attempt_id = db(claimed_attempt_id(conn, job))?;
            db(targets::lock_holds_waiting(conn, &job.target_keys))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
            db(close_attempt(
                conn,
                attempt_id,
                now,
                &AttemptClose::new(
                    AsyncAttemptResult::Cancelled,
                    AsyncJobStatus::Cancelled,
                    "CANCELLED",
                ),
            ))?;
            change.terminate(AsyncJobStatus::Cancelled, now);
            change.outcome("CANCELLED");
            db(set_cause(
                conn,
                T14R_CANCEL_RUNNING,
                Some(attempt_id),
                command_id,
            ))?;
            db(write_job(conn, job.async_job_id, &change))?;
            Ok(Decision::applied("CANCELLED"))
        }
        (AsyncJobStatus::Running, false) => {
            // T14e: end the attempt and hand the job to reconciliation.
            let attempt_id = db(claimed_attempt_id(conn, job))?;
            db(close_attempt(
                conn,
                attempt_id,
                now,
                &AttemptClose::new(
                    AsyncAttemptResult::Cancelled,
                    AsyncJobStatus::ReconciliationRequired,
                    "OPERATOR_STOP",
                ),
            ))?;
            change.require_reconciliation(now);
            change.outcome("OPERATOR_STOP");
            db(set_cause(
                conn,
                T14E_OPERATOR_STOP,
                Some(attempt_id),
                command_id,
            ))?;
            db(write_job(conn, job.async_job_id, &change))?;
            Ok(Decision::applied("OPERATOR_STOP"))
        }
        (AsyncJobStatus::Waiting, false) => {
            // T14e from WAITING: no claim and no open attempt; holds kept.
            change.require_reconciliation(now);
            change.outcome("OPERATOR_STOP");
            db(set_cause(conn, T14E_OPERATOR_STOP, None, command_id))?;
            db(write_job(conn, job.async_job_id, &change))?;
            Ok(Decision::applied("OPERATOR_STOP"))
        }
        // RR is resolved only through T16 or T18; terminal jobs are final.
        _ => Ok(Decision::refused("NOT_CANCELLABLE")),
    }
}

// ---------------------------------------------------------------------------
// RETRY_JOB (safe manual retry)
// ---------------------------------------------------------------------------

/// Create a manual retry of a `FAILED` or `CANCELLED` job that never started
/// its effect, or whose latest finished attempt proved no effect: same kind,
/// version, payload, source fields and targets, `MANUAL_RETRY` lineage, a
/// policy snapshot from the current registry, and the key
/// `kind:effect_identity#retry:parent_job_id`. One retry child per job.
pub fn retry_job_command_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    actor: &StaffActor,
    job_id: Uuid,
) -> ThothResult<CommandResult> {
    let request_id = validate_actor(actor)?;
    run_op(conn, |conn| {
        run_command(
            conn,
            AsyncControlCommandKind::RetryJob,
            actor,
            request_id,
            job_id,
            EMPTY_REQUEST,
            |conn, context| decide_retry(conn, registry, actor, context),
        )
    })
}

#[derive(QueryableByName)]
struct OutcomeRow {
    #[diesel(sql_type = Nullable<Text>)]
    outcome_code: Option<String>,
}

/// The outcome code of the job's latest finished attempt (highest
/// `attempt_number`, never `finished_at`).
fn latest_finished_outcome(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<Option<String>> {
    Ok(db(sql_query(
        "SELECT outcome_code FROM public.async_job_attempt \
          WHERE async_job_id = $1 AND finished_at IS NOT NULL \
          ORDER BY attempt_number DESC LIMIT 1",
    )
    .bind::<SqlUuid, _>(job_id)
    .get_result::<OutcomeRow>(conn)
    .optional())?
    .and_then(|row| row.outcome_code))
}

#[derive(QueryableByName)]
struct ExistsRow {
    #[diesel(sql_type = Bool)]
    value: bool,
}

fn has_retry_child(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<bool> {
    Ok(db(sql_query(
        "SELECT EXISTS (SELECT 1 FROM public.async_job WHERE parent_job_id = $1) AS value",
    )
    .bind::<SqlUuid, _>(job_id)
    .get_result::<ExistsRow>(conn))?
    .value)
}

/// The T1 plan of a retry child: the parent's stored payload and targets,
/// validated against the current registry, under the retry key.
fn plan_retry(
    registry: &AsyncJobRegistry,
    input: &NewAsyncJob,
    parent: &JobRow,
) -> ThothResult<CreationPlan> {
    let mut plan = plan_creation(registry, input)?;
    plan.idempotency_key = format!("{}#retry:{}", plan.idempotency_key, parent.async_job_id);
    plan.targets = validate_targets(plan.policy, &parent.target_keys)?;
    Ok(plan)
}

fn decide_retry(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    actor: &StaffActor,
    context: &mut CommandContext,
) -> ThothResult<Decision> {
    let job = &context.job;
    let Some(policy) = registry.get(&job.kind) else {
        return Ok(Decision::refused("KIND_UNKNOWN"));
    };
    if !policy.manual_retry_allowed
        || !policy.admission_open
        || !policy.is_creatable_version(job.payload_version)
    {
        return Ok(Decision::refused("KIND_POLICY"));
    }
    let input = NewAsyncJob {
        kind: job.kind.clone(),
        payload_version: job.payload_version,
        payload: job.payload.clone(),
        priority: job.priority,
        source_order: job.source_order,
        source_fingerprint: job.source_fingerprint.clone(),
        source_ref: job.source_ref.clone(),
        correlation_id: job.correlation_id,
        causation_ref: job.causation_ref.clone(),
        created_by: actor.principal.clone(),
    };
    // The current kind no longer accepts the stored job: a policy refusal.
    let Ok(plan) = plan_retry(registry, &input, job) else {
        return Ok(Decision::refused("KIND_POLICY"));
    };
    let closed = matches!(
        job.status,
        AsyncJobStatus::Failed | AsyncJobStatus::Cancelled
    );
    let replay_safe = job.effect_phase == AsyncEffectPhase::PreWrite
        || latest_finished_outcome(conn, job.async_job_id)?.as_deref() == Some("NO_EFFECT_PROVEN");
    if !closed || !replay_safe || has_retry_child(conn, job.async_job_id)? {
        return Ok(Decision::refused("RETRY_NOT_ALLOWED"));
    }
    match insert_planned_job(
        conn,
        &plan,
        &input,
        AsyncJobLineage::ManualRetry,
        Some(job.async_job_id),
        RETRY_JOB_CREATE,
        Some(context.command_id),
    ) {
        Ok(CreateOutcome::Created(child)) => {
            // The internal creation path forces only the hold and attempt
            // checks; the command's own check is forced once, at its end.
            db(force_hold_and_attempt_checks(conn))?;
            Ok(Decision {
                outcome: AsyncControlCommandOutcome::Applied,
                code: "RETRY_CREATED".to_string(),
                result_job_id: Some(child.async_job_id),
            })
        }
        Ok(CreateOutcome::Existing(_)) | Err(ThothError::AsyncJobIdempotencyConflict) => {
            Ok(Decision::refused("RETRY_NOT_ALLOWED"))
        }
        Err(error) => Err(error),
    }
}

// ---------------------------------------------------------------------------
// RESOLVE_RECONCILIATION (T18)
// ---------------------------------------------------------------------------

/// A JSON input of a staff command that parsed, has no U+0000 and that
/// `jsonb` accepts: its text, value, `jsonb` byte length and canonical
/// digest.
struct CommandJson {
    text: String,
    value: serde_json::Value,
    bytes: i32,
    digest: String,
}

/// Parse a command's JSON input: a parse failure or U+0000 is
/// `ASYNC_JOB_PAYLOAD_INVALID` (no canonical form to digest).
fn parse_command_json(text: &str, input: JsonInput) -> ThothResult<serde_json::Value> {
    parse_json(text).map_err(|fault| ThothError::AsyncJobPayloadInvalid(fault.describe(input)))
}

/// Ask `jsonb` for the byte length and canonical digest; a value `jsonb`
/// refuses is `ASYNC_JOB_PAYLOAD_INVALID` before any write.
fn measure_command_json(
    conn: &mut PgConnection,
    text: &str,
    value: serde_json::Value,
    input: JsonInput,
) -> ThothResult<CommandJson> {
    let measured = json_measure(conn, text)?
        .ok_or_else(|| ThothError::AsyncJobPayloadInvalid(JsonFault::Database.describe(input)))?;
    Ok(CommandJson {
        text: text.to_string(),
        value,
        bytes: measured.bytes,
        digest: measured.digest,
    })
}

/// Resolve an RR job by staff (T18): the kind must be known; the job is
/// unclaimed, or claimed with an expired lease (T17 is applied first and
/// commits with the command whatever its outcome); a live reconciliation
/// claim is refused with `RECONCILIATION_IN_PROGRESS`. The outcome is one of
/// the kind's `staff_reconciliation_outcomes`; the staff attempt is opened
/// and closed in this call.
pub fn resolve_reconciliation_command_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    actor: &StaffActor,
    job_id: Uuid,
    resolution: StaffResolution,
) -> ThothResult<CommandResult> {
    let request_id = validate_actor(actor)?;
    // Inputs without a canonical form refuse before any statement.
    let evidence = resolution
        .evidence
        .as_deref()
        .map(|text| parse_command_json(text, JsonInput::Evidence).map(|v| (text, v)))
        .transpose()?;
    let checkpoint = resolution
        .checkpoint
        .as_deref()
        .map(|text| parse_command_json(text, JsonInput::Checkpoint).map(|v| (text, v)))
        .transpose()?;
    if let Some(entries) = &resolution.entries {
        let nul = entries.iter().any(|entry| {
            [
                Some(entry.target_key.as_str()),
                entry.fingerprint.as_deref(),
                entry.source_ref.as_deref(),
            ]
            .iter()
            .flatten()
            .any(|value| value.contains('\0'))
        });
        if nul {
            return Err(ThothError::AsyncJobPayloadInvalid(
                "the entries contain U+0000".to_string(),
            ));
        }
    }
    run_op(conn, |conn| {
        let evidence = evidence
            .map(|(text, value)| measure_command_json(conn, text, value, JsonInput::Evidence))
            .transpose()?;
        let checkpoint = checkpoint
            .map(|(text, value)| measure_command_json(conn, text, value, JsonInput::Checkpoint))
            .transpose()?;
        let entries_digest = match &resolution.entries {
            Some(entries) => {
                let text = entries_json(entries);
                let measured = json_measure(conn, &text)?.ok_or_else(|| {
                    ThothError::AsyncJobPayloadInvalid(
                        "the entries are not storable as jsonb".to_string(),
                    )
                })?;
                Some(measured.digest)
            }
            None => None,
        };
        let request = serde_json::json!({
            "outcome": resolution.outcome.as_str(),
            "resumeAt": resolution
                .resume_at
                .map(|at| at.to_rfc3339_opts(SecondsFormat::Micros, true)),
            "entriesSha256": entries_digest,
            "evidenceSha256": evidence.as_ref().map(|e| e.digest.clone()),
            "checkpointSha256": checkpoint.as_ref().map(|c| c.digest.clone()),
        })
        .to_string();
        let inputs = ResolutionInputs {
            outcome: resolution.outcome,
            entries: resolution.entries.as_deref(),
            evidence: evidence.as_ref(),
            checkpoint: checkpoint.as_ref(),
            resume_at: resolution.resume_at,
        };
        run_command(
            conn,
            AsyncControlCommandKind::ResolveReconciliation,
            actor,
            request_id,
            job_id,
            &request,
            |conn, context| decide_resolution(conn, registry, actor, context, &inputs),
        )
    })
}

struct ResolutionInputs<'a> {
    outcome: StaffOutcome,
    entries: Option<&'a [EvidenceEntry]>,
    evidence: Option<&'a CommandJson>,
    checkpoint: Option<&'a CommandJson>,
    resume_at: Option<DateTime<Utc>>,
}

/// An evidence object's shape for a command: an object, without a
/// credential-named key, within 16384 bytes as `jsonb`.
fn command_evidence_shape(evidence: &CommandJson) -> bool {
    check_json_object(&evidence.value, true).is_ok()
        && evidence.bytes <= JsonInput::Evidence.max_bytes()
}

/// A checkpoint's shape for a command: an object, without a credential-named
/// key, within 65536 bytes as `jsonb`.
fn command_checkpoint_shape(checkpoint: &CommandJson) -> bool {
    check_json_object(&checkpoint.value, true).is_ok()
        && checkpoint.bytes <= JsonInput::Checkpoint.max_bytes()
}

/// True when the stored state of the job's one target is a regression or a
/// same-order conflict against the job's own source revision.
fn is_regression_or_conflict(
    job: &JobRow,
    states: &std::collections::BTreeMap<String, StateRow>,
) -> bool {
    let [target] = job.target_keys.as_slice() else {
        return false;
    };
    let Some(stored) = states.get(target) else {
        return false;
    };
    match (stored.applied_order, job.source_order) {
        (Some(applied), Some(source)) => {
            applied > source
                || (applied == source
                    && Some(stored.applied_fingerprint.as_str())
                        != job.source_fingerprint.as_deref())
        }
        _ => false,
    }
}

/// The planned write of an applied T18 outcome.
enum StaffApply {
    Succeeded(Vec<StateWrite>),
    AcceptRegression(StateWrite),
    /// FAILED or CANCELLED with no-effect evidence, or FAILED at
    /// `EFFECT_CONFIRMED` with not-applied evidence.
    Closed {
        status: AsyncJobStatus,
        result: AsyncAttemptResult,
        code: &'static str,
    },
    Waiting {
        resume_at: DateTime<Utc>,
    },
}

fn decide_resolution(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    actor: &StaffActor,
    context: &mut CommandContext,
    inputs: &ResolutionInputs<'_>,
) -> ThothResult<Decision> {
    let now = context.now;
    let Some(policy) = registry.get(&context.job.kind) else {
        return Ok(Decision::refused("KIND_UNKNOWN"));
    };
    if context.job.status != AsyncJobStatus::ReconciliationRequired {
        return Ok(Decision::refused("NOT_RECONCILIATION_REQUIRED"));
    }
    if context.job.claim_token.is_some() {
        let live = context
            .job
            .lease_expires_at
            .is_some_and(|lease| lease > now);
        if live {
            return Ok(Decision::refused("RECONCILIATION_IN_PROGRESS"));
        }
        // T17 first, in the same transaction; it commits with the command.
        db(apply_reconciliation_lease_expiry(conn, &context.job, now))?;
        context.job = lock_job(conn, context.job.async_job_id)?
            .ok_or(ThothError::AsyncJobIntegrityViolation)?;
    }
    let job = context.job.clone();
    let outcome = inputs.outcome;
    let accept_allowed = job.effect_class == EffectClass::Effectful
        && job.revision_mode == RevisionMode::RevisionBound;
    if !policy.allows_staff_outcome(outcome)
        || (outcome == StaffOutcome::AcceptRegression && !accept_allowed)
    {
        return Ok(Decision::refused("OUTCOME_NOT_ALLOWED"));
    }
    let confirmed = job.effect_phase == AsyncEffectPhase::EffectConfirmed;
    if outcome == StaffOutcome::Cancelled && confirmed {
        return Ok(Decision::refused("EFFECT_CONFIRMED"));
    }
    // Rule H, waiting form, then the state rows, ascending by key.
    db(targets::lock_holds_waiting(conn, &job.target_keys))?;
    let states = db(targets::lock_state_rows(conn, &job.target_keys))?;
    if outcome == StaffOutcome::AcceptRegression && !is_regression_or_conflict(&job, &states) {
        return Ok(Decision::refused("NOT_A_REGRESSION"));
    }
    if let Some(code) = wait_refusal(policy, &job, inputs, now) {
        return Ok(Decision::refused(code));
    }
    let apply = match staff_evidence(policy, &job, inputs, &states)? {
        Ok(apply) => apply,
        Err(()) => return Ok(Decision::refused("EVIDENCE_INVALID")),
    };
    apply_resolution(conn, policy, actor, context, inputs, apply)
}

/// `WAIT_INVALID`: for `WAITING`, a missing or invalid checkpoint, a missing
/// `resume_at` or one outside its bounds, or a passed waiting deadline; for
/// any other outcome, a supplied checkpoint or `resume_at`.
fn wait_refusal(
    policy: &AsyncJobKindPolicy,
    job: &JobRow,
    inputs: &ResolutionInputs<'_>,
    now: DateTime<Utc>,
) -> Option<&'static str> {
    if inputs.outcome != StaffOutcome::Waiting {
        return (inputs.checkpoint.is_some() || inputs.resume_at.is_some())
            .then_some("WAIT_INVALID");
    }
    let checkpoint_valid = inputs.checkpoint.is_some_and(command_checkpoint_shape);
    let Some(resume_at) = inputs.resume_at else {
        return Some("WAIT_INVALID");
    };
    if !checkpoint_valid || job.waiting_deadline.is_some_and(|deadline| deadline <= now) {
        return Some("WAIT_INVALID");
    }
    let earliest = now + Duration::seconds(i64::from(policy.poll_interval_min_seconds));
    let latest = job
        .waiting_deadline
        .unwrap_or(now + Duration::seconds(policy.waiting_horizon_seconds));
    if resume_at < earliest || resume_at > latest {
        return Some("WAIT_INVALID");
    }
    None
}

/// The evidence checks of section 13.3 for a T18 outcome: `Err(())` is a
/// committed `REFUSED` with `EVIDENCE_INVALID`.
fn staff_evidence(
    policy: &AsyncJobKindPolicy,
    job: &JobRow,
    inputs: &ResolutionInputs<'_>,
    states: &std::collections::BTreeMap<String, StateRow>,
) -> ThothResult<Result<StaffApply, ()>> {
    let payload = stored_payload(job)?;
    let context = evidence_context(job, &payload);
    let confirmed = job.effect_phase == AsyncEffectPhase::EffectConfirmed;
    // A supplied evidence object must be shaped; the kind then judges it.
    let validate = |use_: EvidenceUse, evidence: Option<&CommandJson>| -> bool {
        match evidence {
            Some(evidence) => {
                command_evidence_shape(evidence)
                    && (policy.validate_evidence)(use_, &context, &evidence.value).is_ok()
            }
            None => (policy.validate_evidence)(use_, &context, &serde_json::Value::Null).is_ok(),
        }
    };
    Ok(match inputs.outcome {
        StaffOutcome::Succeeded => {
            let Some(entries) = inputs.entries else {
                return Ok(Err(()));
            };
            // Carry-forward A5: the EFFECT_APPLIED row, so `sourceRef` is
            // allowed.
            if check_entry_shapes(entries, EntryRule::EffectApplied, job.revision_mode).is_err() {
                return Ok(Err(()));
            }
            if inputs.evidence.is_some_and(|e| !command_evidence_shape(e)) {
                return Ok(Err(()));
            }
            let Ok(writes) = check_entries_semantic(entries, EntryRule::EffectApplied, job, states)
            else {
                return Ok(Err(()));
            };
            if !validate(EvidenceUse::Applied, inputs.evidence) {
                return Ok(Err(()));
            }
            Ok(StaffApply::Succeeded(writes))
        }
        StaffOutcome::AcceptRegression => {
            if inputs.entries.is_some() || inputs.evidence.is_none() {
                return Ok(Err(()));
            }
            if !validate(EvidenceUse::Applied, inputs.evidence) {
                return Ok(Err(()));
            }
            let (Some(target), Some(fingerprint)) =
                (job.target_keys.first(), job.source_fingerprint.clone())
            else {
                return Err(ThothError::AsyncJobIntegrityViolation);
            };
            Ok(StaffApply::AcceptRegression(StateWrite {
                target_key: target.clone(),
                order: job.source_order,
                fingerprint,
                source_ref: job.source_ref.clone(),
            }))
        }
        StaffOutcome::Failed | StaffOutcome::Cancelled => {
            if inputs.entries.is_some() || inputs.evidence.is_none() {
                return Ok(Err(()));
            }
            let (use_, status, result, code) = match (inputs.outcome, confirmed) {
                (StaffOutcome::Failed, true) => (
                    EvidenceUse::NotApplied,
                    AsyncJobStatus::Failed,
                    AsyncAttemptResult::PermanentFailure,
                    "CONFIRMED_EFFECT_NOT_APPLIED",
                ),
                (StaffOutcome::Failed, false) => (
                    EvidenceUse::NoEffect,
                    AsyncJobStatus::Failed,
                    AsyncAttemptResult::PermanentFailure,
                    "NO_EFFECT_PROVEN",
                ),
                _ => (
                    EvidenceUse::NoEffect,
                    AsyncJobStatus::Cancelled,
                    AsyncAttemptResult::Cancelled,
                    "NO_EFFECT_PROVEN",
                ),
            };
            if !validate(use_, inputs.evidence) {
                return Ok(Err(()));
            }
            Ok(StaffApply::Closed {
                status,
                result,
                code,
            })
        }
        StaffOutcome::Waiting => {
            if inputs.entries.is_some() || inputs.evidence.is_some() {
                return Ok(Err(()));
            }
            Ok(StaffApply::Waiting {
                resume_at: inputs
                    .resume_at
                    .ok_or(ThothError::AsyncJobIntegrityViolation)?,
            })
        }
    })
}

/// Open and close the staff attempt and apply the outcome (T18).
fn apply_resolution(
    conn: &mut PgConnection,
    policy: &AsyncJobKindPolicy,
    actor: &StaffActor,
    context: &CommandContext,
    inputs: &ResolutionInputs<'_>,
    apply: StaffApply,
) -> ThothResult<Decision> {
    let job = &context.job;
    let now = context.now;
    let attempt_number = job.attempt_count + 1;
    let attempt_id = db(insert_attempt(
        conn,
        &NewAttempt {
            job_id: job.async_job_id,
            attempt_number,
            purpose: AsyncAttemptPurpose::StaffReconciliation,
            token: Uuid::new_v4(),
            claimed_by: STAFF_ATTEMPT_LABEL,
            principal: &actor.principal,
            command_id: Some(context.command_id),
            now,
            phase: job.effect_phase,
        },
    ))?;
    let evidence_text = inputs.evidence.map(|evidence| evidence.text.as_str());
    let mut change = JobChange::from_row(job);
    change.attempt_count = attempt_number;
    let code = match apply {
        StaffApply::Succeeded(writes) => {
            db(targets::write_states(
                conn,
                &writes,
                job.async_job_id,
                attempt_id,
            ))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
            let mut close = AttemptClose::new(
                AsyncAttemptResult::Succeeded,
                AsyncJobStatus::Succeeded,
                "EFFECT_APPLIED",
            );
            close.evidence = evidence_text;
            db(close_attempt(conn, attempt_id, now, &close))?;
            change.terminate(AsyncJobStatus::Succeeded, now);
            "EFFECT_APPLIED"
        }
        StaffApply::AcceptRegression(write) => {
            db(targets::write_states(
                conn,
                std::slice::from_ref(&write),
                job.async_job_id,
                attempt_id,
            ))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
            let mut close = AttemptClose::new(
                AsyncAttemptResult::Succeeded,
                AsyncJobStatus::Succeeded,
                "REGRESSION_ACCEPTED",
            );
            close.evidence = evidence_text;
            db(close_attempt(conn, attempt_id, now, &close))?;
            change.terminate(AsyncJobStatus::Succeeded, now);
            "REGRESSION_ACCEPTED"
        }
        StaffApply::Closed {
            status,
            result,
            code,
        } => {
            db(targets::delete_holds(conn, job.async_job_id))?;
            let mut close = AttemptClose::new(result, status, code);
            close.evidence = evidence_text;
            db(close_attempt(conn, attempt_id, now, &close))?;
            change.terminate(status, now);
            code
        }
        StaffApply::Waiting { resume_at } => {
            let checkpoint = inputs
                .checkpoint
                .ok_or(ThothError::AsyncJobIntegrityViolation)?;
            db(close_attempt(
                conn,
                attempt_id,
                now,
                &AttemptClose::new(
                    AsyncAttemptResult::Waiting,
                    AsyncJobStatus::Waiting,
                    "WAITING",
                ),
            ))?;
            change.status = AsyncJobStatus::Waiting;
            change.checkpoint = CheckpointWrite::Set(checkpoint.text.clone());
            change.checkpoint_version += 1;
            change.available_at = resume_at;
            change.waiting_deadline = job.waiting_deadline.or(Some(
                now + Duration::seconds(policy.waiting_horizon_seconds),
            ));
            "WAITING"
        }
    };
    change.outcome(code);
    db(set_cause(
        conn,
        T18_STAFF_RESOLVE,
        Some(attempt_id),
        Some(context.command_id),
    ))?;
    db(write_job(conn, job.async_job_id, &change))?;
    Ok(Decision::applied(code))
}

// ---------------------------------------------------------------------------
// REDACT_JOB_DIAGNOSTICS
// ---------------------------------------------------------------------------

/// Redact the job's diagnostics: one set-based update nulls `error_detail`,
/// `provider_ref` and `evidence` of every finished, unredacted attempt; a
/// terminal, unredacted job also loses its `provider_ref` and `checkpoint`.
/// An open attempt is left unchanged and reported (`OPEN_ATTEMPT_UNCHANGED`);
/// a repeat with nothing left to redact is `APPLIED` (`NOTHING_TO_REDACT`) and
/// changes no row but its own command row. Needs no kind policy, so it also
/// applies to kinds unknown to the registry.
pub fn redact_job_diagnostics_command_on(
    conn: &mut PgConnection,
    actor: &StaffActor,
    job_id: Uuid,
) -> ThothResult<CommandResult> {
    let request_id = validate_actor(actor)?;
    run_op(conn, |conn| {
        run_command(
            conn,
            AsyncControlCommandKind::RedactJobDiagnostics,
            actor,
            request_id,
            job_id,
            EMPTY_REQUEST,
            decide_redaction,
        )
    })
}

#[derive(QueryableByName)]
struct CountOpen {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

fn decide_redaction(
    conn: &mut PgConnection,
    context: &mut CommandContext,
) -> ThothResult<Decision> {
    let job = &context.job;
    let open = db(sql_query(
        "SELECT count(*) AS count FROM public.async_job_attempt \
          WHERE async_job_id = $1 AND finished_at IS NULL",
    )
    .bind::<SqlUuid, _>(job.async_job_id)
    .get_result::<CountOpen>(conn))?
    .count;
    let attempts = db(sql_query(
        "UPDATE public.async_job_attempt SET \
             error_detail = NULL, provider_ref = NULL, evidence = NULL, \
             redacted_at = $2, redaction_command_id = $3 \
         WHERE async_job_id = $1 AND finished_at IS NOT NULL AND redacted_at IS NULL",
    )
    .bind::<SqlUuid, _>(job.async_job_id)
    .bind::<Timestamptz, _>(context.now)
    .bind::<SqlUuid, _>(context.command_id)
    .execute(conn))?;
    let job_redacted = if job.status.is_terminal() && job.redacted_at.is_none() {
        db(sql_query(
            "UPDATE public.async_job SET provider_ref = NULL, checkpoint = NULL, redacted_at = $2 \
              WHERE async_job_id = $1",
        )
        .bind::<SqlUuid, _>(job.async_job_id)
        .bind::<Timestamptz, _>(context.now)
        .execute(conn))?
            == 1
    } else {
        false
    };
    Ok(Decision::applied(if open > 0 {
        "OPEN_ATTEMPT_UNCHANGED"
    } else if attempts == 0 && !job_redacted {
        "NOTHING_TO_REDACT"
    } else {
        "REDACTED"
    }))
}
