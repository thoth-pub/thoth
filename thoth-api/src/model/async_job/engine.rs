//! Engine operations (spec sections 9 to 13): creation, claims, worker
//! writes, reconciliation and recovery.
//!
//! Every operation validates its bounded inputs before any write, runs inside
//! the caller's READ COMMITTED transaction between [`run_op`]'s deferral and
//! forced checks, reads `now` with `clock_timestamp()` after its locks, and
//! attributes each status change to a named cause in the transition log.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use chrono::{DateTime, Duration, Utc};
use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sql_types::{
    Array, BigInt, Bool, Int2, Int4, Int8, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::registry::{
    AsyncJobKindPolicy, AsyncJobRegistry, EvidenceContext, EvidenceUse, ExecutorDomain, RiskClass,
};
use super::targets::{
    self, check_entries_semantic, check_entry_shapes, EntryRule, EvidenceEntry, SemanticFault,
    TargetState,
};
use super::{
    backoff_until, clock_now, db, error_code_of, force_hold_and_attempt_checks, is_busy_error,
    is_code, is_connection_lost, is_identifier, is_non_blank, lock_job, lock_key_rows,
    map_db_error, measure_json, parse_json_object, read_job, require_no_nul, run_op,
    sanitize_free_text, set_cause, target_key_namespace, AsyncAttemptPurpose, AsyncAttemptResult,
    AsyncEffectPhase, AsyncJob, AsyncJobLineage, AsyncJobStatus, BoolRow, CountRow, EffectClass,
    JobRow, JsonInput, RevisionMode, UuidRow, CLAIMED_BY_MAX_CHARS, CLAIM_MAX_JOBS,
    CLAIM_RECOVERY_LIMIT, CREATED_BY_MAX_CHARS, EFFECT_IDENTITY_MAX_CHARS, ERROR_DETAIL_MAX_CHARS,
    JOB_COLUMNS, MAX_TARGETS, NS_KIND, NS_RECON, PRINCIPAL_MAX_CHARS, PRIORITY_MAX, PRIORITY_MIN,
    PROVIDER_REF_MAX_CHARS, RECOVERY_LIMIT_MAX,
};

// ---------------------------------------------------------------------------
// Transition causes (section 7.8): every engine status change names one.
// ---------------------------------------------------------------------------

pub(crate) const T1_CREATE: &str = "T1_CREATE";
pub(crate) const T2_CLAIM: &str = "T2_CLAIM";
pub(crate) const T6_COMPLETE: &str = "T6_COMPLETE";
pub(crate) const T6X_EVIDENCE_FAULT: &str = "T6X_EVIDENCE_FAULT";
pub(crate) const T7_FAIL_RETRYABLE: &str = "T7_FAIL_RETRYABLE";
pub(crate) const T8_FAIL: &str = "T8_FAIL";
pub(crate) const T9_FAIL_AFTER_EFFECT_START: &str = "T9_FAIL_AFTER_EFFECT_START";
pub(crate) const T9N_FAIL_NO_EFFECT_PROVEN: &str = "T9N_FAIL_NO_EFFECT_PROVEN";
pub(crate) const T10_WAIT: &str = "T10_WAIT";
pub(crate) const T10D_WAIT_DEADLINE_EXCEEDED: &str = "T10D_WAIT_DEADLINE_EXCEEDED";
pub(crate) const T11_WAITING_DEADLINE_EXCEEDED: &str = "T11_WAITING_DEADLINE_EXCEEDED";
pub(crate) const T12_LEASE_EXPIRED: &str = "T12_LEASE_EXPIRED";
pub(crate) const T13_RELEASE: &str = "T13_RELEASE";
pub(crate) const T16_RESOLVE: &str = "T16_RESOLVE";

// ---------------------------------------------------------------------------
// Small error constructors
// ---------------------------------------------------------------------------

pub(crate) fn invalid(message: impl Into<String>) -> ThothError {
    ThothError::AsyncJobPayloadInvalid(message.into())
}

pub(crate) fn refused(reason: &str) -> ThothError {
    ThothError::AsyncJobTransitionRefused(reason.to_string())
}

pub(crate) fn evidence_invalid(message: impl Into<String>) -> ThothError {
    ThothError::AsyncJobEvidenceInvalid(message.into())
}

/// The error of one savepoint step (a claim candidate, a recovery row): skip
/// the step (a hold lock or hold is taken, or the row is not due), or a
/// database error the caller classifies.
pub(crate) enum StepError {
    Skip,
    Db(DieselError),
}

impl From<DieselError> for StepError {
    fn from(error: DieselError) -> Self {
        StepError::Db(error)
    }
}

// ---------------------------------------------------------------------------
// Job writes: one fixed UPDATE of every mutable column, under the row lock
// ---------------------------------------------------------------------------

/// What happens to the job's checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckpointWrite {
    Keep,
    /// The exact JSON text received, stored as `$1::jsonb`.
    Set(String),
}

/// The new values of every mutable job column, built from the locked row.
#[derive(Debug, Clone)]
pub(crate) struct JobChange {
    pub status: AsyncJobStatus,
    pub available_at: DateTime<Utc>,
    pub effect_phase: AsyncEffectPhase,
    pub attempt_count: i32,
    pub execution_claim_count: i16,
    pub prewrite_failure_count: i16,
    pub reconciliation_attempt_count: i16,
    pub claim_token: Option<Uuid>,
    pub claimed_by: Option<String>,
    pub executor_principal: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub attempt_runtime_deadline: Option<DateTime<Utc>>,
    pub first_claimed_at: Option<DateTime<Utc>>,
    pub waiting_deadline: Option<DateTime<Utc>>,
    pub checkpoint: CheckpointWrite,
    pub checkpoint_version: i32,
    pub provider_ref: Option<String>,
    pub terminal_at: Option<DateTime<Utc>>,
    pub last_outcome_code: Option<String>,
}

impl JobChange {
    pub(crate) fn from_row(job: &JobRow) -> Self {
        JobChange {
            status: job.status,
            available_at: job.available_at,
            effect_phase: job.effect_phase,
            attempt_count: job.attempt_count,
            execution_claim_count: job.execution_claim_count,
            prewrite_failure_count: job.prewrite_failure_count,
            reconciliation_attempt_count: job.reconciliation_attempt_count,
            claim_token: job.claim_token,
            claimed_by: job.claimed_by.clone(),
            executor_principal: job.executor_principal.clone(),
            claimed_at: job.claimed_at,
            lease_expires_at: job.lease_expires_at,
            attempt_runtime_deadline: job.attempt_runtime_deadline,
            first_claimed_at: job.first_claimed_at,
            waiting_deadline: job.waiting_deadline,
            checkpoint: CheckpointWrite::Keep,
            checkpoint_version: job.checkpoint_version,
            provider_ref: job.provider_ref.clone(),
            terminal_at: job.terminal_at,
            last_outcome_code: job.last_outcome_code.clone(),
        }
    }

    /// Clear the six claim columns (the token is invalidated).
    pub(crate) fn clear_claim(&mut self) {
        self.claim_token = None;
        self.claimed_by = None;
        self.executor_principal = None;
        self.claimed_at = None;
        self.lease_expires_at = None;
        self.attempt_runtime_deadline = None;
    }

    /// Enter a terminal status.
    pub(crate) fn terminate(&mut self, status: AsyncJobStatus, now: DateTime<Utc>) {
        self.clear_claim();
        self.status = status;
        self.terminal_at = Some(now);
    }

    /// Enter `RECONCILIATION_REQUIRED`: `available_at = now`, claim cleared.
    pub(crate) fn require_reconciliation(&mut self, now: DateTime<Utc>) {
        self.clear_claim();
        self.status = AsyncJobStatus::ReconciliationRequired;
        self.available_at = now;
    }

    pub(crate) fn outcome(&mut self, code: &str) {
        self.last_outcome_code = Some(code.to_string());
    }
}

const UPDATE_JOB_SQL: &str = "UPDATE public.async_job SET \
     status = $2::public.async_job_status, \
     available_at = $3, \
     effect_phase = $4::public.async_effect_phase, \
     attempt_count = $5, \
     execution_claim_count = $6, \
     prewrite_failure_count = $7, \
     reconciliation_attempt_count = $8, \
     claim_token = $9, \
     claimed_by = $10, \
     executor_principal = $11, \
     claimed_at = $12, \
     lease_expires_at = $13, \
     attempt_runtime_deadline = $14, \
     first_claimed_at = $15, \
     waiting_deadline = $16, \
     checkpoint = CASE WHEN $17 THEN $18::jsonb ELSE checkpoint END, \
     checkpoint_version = $19, \
     provider_ref = $20, \
     terminal_at = $21, \
     last_outcome_code = $22 \
     WHERE async_job_id = $1";

/// Write every mutable column of a locked job.
pub(crate) fn write_job(
    conn: &mut PgConnection,
    job_id: Uuid,
    change: &JobChange,
) -> QueryResult<()> {
    let (set_checkpoint, checkpoint) = match &change.checkpoint {
        CheckpointWrite::Keep => (false, None),
        CheckpointWrite::Set(text) => (true, Some(text.as_str())),
    };
    let updated = sql_query(UPDATE_JOB_SQL)
        .bind::<SqlUuid, _>(job_id)
        .bind::<Text, _>(change.status.as_str())
        .bind::<Timestamptz, _>(change.available_at)
        .bind::<Text, _>(change.effect_phase.as_str())
        .bind::<Int4, _>(change.attempt_count)
        .bind::<Int2, _>(change.execution_claim_count)
        .bind::<Int2, _>(change.prewrite_failure_count)
        .bind::<Int2, _>(change.reconciliation_attempt_count)
        .bind::<Nullable<SqlUuid>, _>(change.claim_token)
        .bind::<Nullable<Text>, _>(change.claimed_by.as_deref())
        .bind::<Nullable<Text>, _>(change.executor_principal.as_deref())
        .bind::<Nullable<Timestamptz>, _>(change.claimed_at)
        .bind::<Nullable<Timestamptz>, _>(change.lease_expires_at)
        .bind::<Nullable<Timestamptz>, _>(change.attempt_runtime_deadline)
        .bind::<Nullable<Timestamptz>, _>(change.first_claimed_at)
        .bind::<Nullable<Timestamptz>, _>(change.waiting_deadline)
        .bind::<Bool, _>(set_checkpoint)
        .bind::<Nullable<Text>, _>(checkpoint)
        .bind::<Int4, _>(change.checkpoint_version)
        .bind::<Nullable<Text>, _>(change.provider_ref.as_deref())
        .bind::<Nullable<Timestamptz>, _>(change.terminal_at)
        .bind::<Nullable<Text>, _>(change.last_outcome_code.as_deref())
        .execute(conn)?;
    if updated != 1 {
        return Err(DieselError::NotFound);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Attempts
// ---------------------------------------------------------------------------

/// A new attempt row.
pub(crate) struct NewAttempt<'a> {
    pub job_id: Uuid,
    pub attempt_number: i32,
    pub purpose: AsyncAttemptPurpose,
    pub token: Uuid,
    pub claimed_by: &'a str,
    pub principal: &'a str,
    pub command_id: Option<Uuid>,
    pub now: DateTime<Utc>,
    pub phase: AsyncEffectPhase,
}

pub(crate) fn insert_attempt(
    conn: &mut PgConnection,
    attempt: &NewAttempt<'_>,
) -> QueryResult<Uuid> {
    sql_query(
        "INSERT INTO public.async_job_attempt \
             (async_job_id, attempt_number, purpose, claim_token, claimed_by, \
              executor_principal, control_command_id, started_at, last_heartbeat_at, \
              phase_at_start) \
         VALUES ($1, $2, $3::public.async_attempt_purpose, $4, $5, $6, $7, $8, $8, \
                 $9::public.async_effect_phase) \
         RETURNING async_job_attempt_id AS id",
    )
    .bind::<SqlUuid, _>(attempt.job_id)
    .bind::<Int4, _>(attempt.attempt_number)
    .bind::<Text, _>(attempt.purpose.as_str())
    .bind::<SqlUuid, _>(attempt.token)
    .bind::<Text, _>(attempt.claimed_by)
    .bind::<Text, _>(attempt.principal)
    .bind::<Nullable<SqlUuid>, _>(attempt.command_id)
    .bind::<Timestamptz, _>(attempt.now)
    .bind::<Text, _>(attempt.phase.as_str())
    .get_result::<UuidRow>(conn)
    .map(|row| row.id)
}

/// How an attempt closes: its closing columns, set once together.
pub(crate) struct AttemptClose<'a> {
    pub result: AsyncAttemptResult,
    pub job_status_after: AsyncJobStatus,
    pub outcome_code: &'a str,
    pub error_code: Option<&'a str>,
    pub error_detail: Option<&'a str>,
    /// Set when `Some`; kept otherwise.
    pub provider_ref: Option<&'a str>,
    /// The exact evidence text, stored as `$1::jsonb`.
    pub evidence: Option<&'a str>,
}

impl<'a> AttemptClose<'a> {
    pub(crate) fn new(
        result: AsyncAttemptResult,
        job_status_after: AsyncJobStatus,
        outcome_code: &'a str,
    ) -> Self {
        AttemptClose {
            result,
            job_status_after,
            outcome_code,
            error_code: None,
            error_detail: None,
            provider_ref: None,
            evidence: None,
        }
    }
}

pub(crate) fn close_attempt(
    conn: &mut PgConnection,
    attempt_id: Uuid,
    now: DateTime<Utc>,
    close: &AttemptClose<'_>,
) -> QueryResult<()> {
    let updated = sql_query(
        "UPDATE public.async_job_attempt SET \
             finished_at = $2, \
             result = $3::public.async_attempt_result, \
             job_status_after = $4::public.async_job_status, \
             outcome_code = $5, \
             error_code = $6, \
             error_detail = $7, \
             provider_ref = COALESCE($8, provider_ref), \
             evidence = $9::jsonb \
         WHERE async_job_attempt_id = $1 AND finished_at IS NULL",
    )
    .bind::<SqlUuid, _>(attempt_id)
    .bind::<Timestamptz, _>(now)
    .bind::<Text, _>(close.result.as_str())
    .bind::<Text, _>(close.job_status_after.as_str())
    .bind::<Text, _>(close.outcome_code)
    .bind::<Nullable<Text>, _>(close.error_code)
    .bind::<Nullable<Text>, _>(close.error_detail)
    .bind::<Nullable<Text>, _>(close.provider_ref)
    .bind::<Nullable<Text>, _>(close.evidence)
    .execute(conn)?;
    if updated != 1 {
        return Err(DieselError::NotFound);
    }
    Ok(())
}

/// The open attempt bound to a claim token.
pub(crate) fn open_attempt_id(conn: &mut PgConnection, token: Uuid) -> QueryResult<Uuid> {
    sql_query(
        "SELECT async_job_attempt_id AS id FROM public.async_job_attempt \
          WHERE claim_token = $1 AND finished_at IS NULL",
    )
    .bind::<SqlUuid, _>(token)
    .get_result::<UuidRow>(conn)
    .map(|row| row.id)
}

/// The open attempt of a claimed job row.
pub(crate) fn claimed_attempt_id(conn: &mut PgConnection, job: &JobRow) -> QueryResult<Uuid> {
    match job.claim_token {
        Some(token) => open_attempt_id(conn, token),
        None => Err(DieselError::NotFound),
    }
}

// ---------------------------------------------------------------------------
// Creation (section 10.1, T1)
// ---------------------------------------------------------------------------

/// The input of [`create_job_on`]. Callers never supply the idempotency key:
/// it is `kind || ':' || effect_identity(version, payload)`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewAsyncJob {
    pub kind: String,
    pub payload_version: i16,
    /// A JSON object as text, stored byte-for-byte through `$1::jsonb`.
    pub payload: String,
    pub priority: i16,
    pub source_order: Option<i64>,
    pub source_fingerprint: Option<String>,
    pub source_ref: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub causation_ref: Option<String>,
    pub created_by: String,
}

/// A created job, or the existing job of an equal earlier request.
#[derive(Debug, Clone, PartialEq)]
pub enum CreateOutcome {
    Created(AsyncJob),
    Existing(AsyncJob),
}

impl CreateOutcome {
    pub fn job(&self) -> &AsyncJob {
        match self {
            CreateOutcome::Created(job) | CreateOutcome::Existing(job) => job,
        }
    }

    pub fn into_job(self) -> AsyncJob {
        match self {
            CreateOutcome::Created(job) | CreateOutcome::Existing(job) => job,
        }
    }

    pub fn is_created(&self) -> bool {
        matches!(self, CreateOutcome::Created(_))
    }
}

/// What T1 validated: the policy, the derived idempotency key and the
/// sorted target keys.
pub(crate) struct CreationPlan {
    pub policy: &'static AsyncJobKindPolicy,
    pub idempotency_key: String,
    pub targets: Vec<String>,
}

/// Every T1 check that needs no database, in the order of section 9.3: kind
/// known and admission open, version creatable, column formats, payload,
/// credential guard, effect identity, targets.
pub(crate) fn plan_creation(
    registry: &AsyncJobRegistry,
    input: &NewAsyncJob,
) -> ThothResult<CreationPlan> {
    let policy = registry
        .get(&input.kind)
        .ok_or(ThothError::AsyncJobKindUnknown)?;
    if !policy.admission_open {
        return Err(ThothError::AsyncJobKindAdmissionClosed);
    }
    if !policy.is_creatable_version(input.payload_version) {
        return Err(ThothError::AsyncJobPayloadVersionUnsupported);
    }
    validate_job_columns(policy, input)?;
    let payload = parse_json_object(&input.payload, true)
        .map_err(|fault| invalid(fault.describe(JsonInput::Payload)))?;
    (policy.validate_payload)(input.payload_version, &payload).map_err(|e| invalid(e.0))?;
    let identity =
        (policy.effect_identity)(input.payload_version, &payload).map_err(|e| invalid(e.0))?;
    if identity.is_empty()
        || identity.chars().count() > EFFECT_IDENTITY_MAX_CHARS
        || identity.contains('\0')
    {
        return Err(invalid(
            "the effect identity must be 1 to 400 characters without U+0000",
        ));
    }
    let targets = (policy.targets)(input.payload_version, &payload).map_err(|e| invalid(e.0))?;
    let targets = validate_targets(policy, &targets)?;
    Ok(CreationPlan {
        policy,
        idempotency_key: format!("{}:{}", input.kind, identity),
        targets,
    })
}

fn validate_job_columns(policy: &AsyncJobKindPolicy, input: &NewAsyncJob) -> ThothResult<()> {
    if !(PRIORITY_MIN..=PRIORITY_MAX).contains(&input.priority) {
        return Err(invalid("priority must be between -100 and 100"));
    }
    require_no_nul(&input.created_by, "created_by")?;
    if !is_non_blank(&input.created_by) || input.created_by.chars().count() > CREATED_BY_MAX_CHARS {
        return Err(invalid(
            "created_by must be non-blank and at most 256 characters",
        ));
    }
    match policy.revision_mode {
        RevisionMode::RevisionBound => {
            if input.source_order.is_none() || input.source_fingerprint.is_none() {
                return Err(invalid(
                    "a REVISION_BOUND job needs source_order and source_fingerprint",
                ));
            }
        }
        RevisionMode::CurrentState => {
            if input.source_order.is_some() || input.source_fingerprint.is_some() {
                return Err(invalid(
                    "a CURRENT_STATE job takes no source_order or source_fingerprint",
                ));
            }
        }
    }
    for (name, value) in [
        ("source_fingerprint", &input.source_fingerprint),
        ("source_ref", &input.source_ref),
        ("causation_ref", &input.causation_ref),
    ] {
        if let Some(value) = value {
            if !is_identifier(value) {
                return Err(invalid(format!(
                    "{name} must be 1 to 256 characters of printable ASCII without spaces"
                )));
            }
        }
    }
    Ok(())
}

/// Targets valid, declared, distinct and within the kind's bounds; returned
/// sorted.
pub(crate) fn validate_targets(
    policy: &AsyncJobKindPolicy,
    targets: &[String],
) -> ThothResult<Vec<String>> {
    let mut seen = BTreeSet::new();
    for key in targets {
        let namespace =
            target_key_namespace(key).ok_or_else(|| invalid("a target key is malformed"))?;
        if !policy.target_namespaces.contains(&namespace) {
            return Err(invalid("a target namespace is not declared by the kind"));
        }
        if !seen.insert(key.clone()) {
            return Err(invalid("a target key is repeated"));
        }
    }
    let count = seen.len();
    if count > policy.max_targets.min(MAX_TARGETS) {
        return Err(invalid("the job has more targets than its kind allows"));
    }
    if policy.is_effectful() && count == 0 {
        return Err(invalid("an EFFECTFUL job needs at least one target"));
    }
    if policy.is_effectful() && policy.is_revision_bound() && count != 1 {
        return Err(invalid(
            "an EFFECTFUL REVISION_BOUND job has exactly one target",
        ));
    }
    Ok(seen.into_iter().collect())
}

const INSERT_JOB_SQL: &str = "INSERT INTO public.async_job ( \
     kind, payload_version, payload, idempotency_key, effect_class, revision_mode, \
     retry_budget, max_execution_claims, max_reconciliation_attempts, \
     backoff_base_seconds, backoff_max_seconds, source_order, source_fingerprint, \
     source_ref, priority, lineage, parent_job_id, correlation_id, causation_ref, created_by) \
     VALUES ($1, $2, $3::jsonb, $4, $5::public.async_job_effect_class, \
     $6::public.async_job_revision_mode, $7, $8, $9, $10, $11, $12, $13, $14, $15, \
     $16::public.async_job_lineage, $17, $18, $19, $20) \
     ON CONFLICT (idempotency_key) DO NOTHING \
     RETURNING async_job_id AS id";

/// Insert a planned job and its targets, or compare it with the existing job
/// under the same key. Shared by [`create_job_on`] and `RETRY_JOB`.
pub(crate) fn insert_planned_job(
    conn: &mut PgConnection,
    plan: &CreationPlan,
    input: &NewAsyncJob,
    lineage: AsyncJobLineage,
    parent_job_id: Option<Uuid>,
    cause: &str,
    command_id: Option<Uuid>,
) -> ThothResult<CreateOutcome> {
    let policy = plan.policy;
    db(set_cause(conn, cause, None, command_id))?;
    let inserted = db(sql_query(INSERT_JOB_SQL)
        .bind::<Text, _>(&input.kind)
        .bind::<Int2, _>(input.payload_version)
        .bind::<Text, _>(&input.payload)
        .bind::<Text, _>(&plan.idempotency_key)
        .bind::<Text, _>(policy.effect_class.as_str())
        .bind::<Text, _>(policy.revision_mode.as_str())
        .bind::<Int2, _>(policy.retry_budget)
        .bind::<Int2, _>(policy.max_execution_claims)
        .bind::<Int2, _>(policy.max_reconciliation_attempts)
        .bind::<Int4, _>(policy.backoff_base_seconds)
        .bind::<Int4, _>(policy.backoff_max_seconds)
        .bind::<Nullable<Int8>, _>(input.source_order)
        .bind::<Nullable<Text>, _>(input.source_fingerprint.as_deref())
        .bind::<Nullable<Text>, _>(input.source_ref.as_deref())
        .bind::<Int2, _>(input.priority)
        .bind::<Text, _>(lineage.as_str())
        .bind::<Nullable<SqlUuid>, _>(parent_job_id)
        .bind::<Nullable<SqlUuid>, _>(input.correlation_id)
        .bind::<Nullable<Text>, _>(input.causation_ref.as_deref())
        .bind::<Text, _>(&input.created_by)
        .get_result::<UuidRow>(conn)
        .optional())?;
    match inserted {
        Some(row) => {
            if !plan.targets.is_empty() {
                db(sql_query(
                    "INSERT INTO public.async_job_target (async_job_id, target_key) \
                     SELECT $1, k FROM unnest($2::text[]) AS k",
                )
                .bind::<SqlUuid, _>(row.id)
                .bind::<Array<Text>, _>(&plan.targets)
                .execute(conn))?;
            }
            let job = read_job(conn, row.id)?.ok_or(ThothError::AsyncJobIntegrityViolation)?;
            Ok(CreateOutcome::Created(job.into()))
        }
        None => {
            let existing = db(sql_query(format!(
                "SELECT {JOB_COLUMNS} FROM public.async_job j WHERE j.idempotency_key = $1"
            ))
            .bind::<Text, _>(&plan.idempotency_key)
            .get_result::<JobRow>(conn))?;
            // Equal canonical forms (section 10): `jsonb` keeps number
            // scale, so `1` and `1.0` are different payloads here, as they
            // are to a handler that reads the stored text.
            let same_payload = db(sql_query(
                "SELECT (payload::text = $2::jsonb::text) AS value FROM public.async_job \
                  WHERE async_job_id = $1",
            )
            .bind::<SqlUuid, _>(existing.async_job_id)
            .bind::<Text, _>(&input.payload)
            .get_result::<BoolRow>(conn))?
            .value;
            let equal = existing.kind == input.kind
                && existing.payload_version == input.payload_version
                && same_payload
                && existing.source_order == input.source_order
                && existing.source_fingerprint == input.source_fingerprint
                && existing.source_ref == input.source_ref
                && existing.lineage == lineage
                && existing.parent_job_id == parent_job_id;
            if equal {
                Ok(CreateOutcome::Existing(existing.into()))
            } else {
                Err(ThothError::AsyncJobIdempotencyConflict)
            }
        }
    }
}

/// T1: create a job inside the caller's transaction (section 10.1). On a key
/// collision with different content the call fails closed: it returns
/// `ASYNC_JOB_IDEMPOTENCY_CONFLICT` and leaves the caller's transaction
/// aborted, so the caller's business write cannot commit without its job
/// even if the error were ignored (a later `COMMIT` rolls back).
pub fn create_job_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    new_job: NewAsyncJob,
) -> ThothResult<CreateOutcome> {
    let plan = plan_creation(registry, &new_job)?;
    let result = run_op(conn, |conn| {
        measure_json(conn, &new_job.payload, JsonInput::Payload)?
            .map_err(|fault| invalid(fault.describe(JsonInput::Payload)))?;
        insert_planned_job(
            conn,
            &plan,
            &new_job,
            AsyncJobLineage::Original,
            None,
            T1_CREATE,
            None,
        )
    });
    if matches!(result, Err(ThothError::AsyncJobIdempotencyConflict)) {
        abort_callers_transaction(conn);
    }
    result
}

/// Put the caller's transaction (or the caller's own savepoint, when it
/// opened one) into PostgreSQL's aborted state with a fixed server message
/// and no row content. The statement's own error is the intended effect.
fn abort_callers_transaction(conn: &mut PgConnection) {
    let aborted = conn.batch_execute(
        "DO $$ BEGIN RAISE EXCEPTION USING \
             ERRCODE = 'unique_violation', \
             CONSTRAINT = 'async_job_idempotency_key_key', \
             MESSAGE = 'async job idempotency conflict: the transaction is aborted'; \
         END $$",
    );
    debug_assert!(aborted.is_err(), "the abort statement always raises");
}

// ---------------------------------------------------------------------------
// Claims (sections 10.2, 10.4 and 12; T2 and T15)
// ---------------------------------------------------------------------------

/// The claiming executor. Domain, risk class, capabilities and accepted
/// revision modes come from the authenticated identity, never from input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executor {
    /// `claimed_by`: the executor name, non-blank, at most 128 characters.
    pub name: String,
    /// The authenticated principal, at most 256 characters.
    pub principal: String,
    pub domain: ExecutorDomain,
    pub risk_class: RiskClass,
    pub capabilities: Vec<String>,
    pub revision_modes: Vec<RevisionMode>,
}

/// One claim call: kinds with the payload versions the executor supports,
/// at most `max_jobs` (1 to 50) jobs, and an optional lease request clamped
/// to each kind's bounds (the kind's default when absent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRequest {
    pub kinds: Vec<(String, Vec<i16>)>,
    pub max_jobs: i32,
    pub lease_seconds: Option<i32>,
}

/// A claimed job: exactly the fields the API returns, the claim token
/// included. Its `Debug` never prints the token.
#[derive(Clone, PartialEq)]
pub struct ClaimedJob {
    pub async_job_id: Uuid,
    pub kind: String,
    pub payload_version: i16,
    pub payload: String,
    pub source_order: Option<i64>,
    pub source_fingerprint: Option<String>,
    pub source_ref: Option<String>,
    pub target_keys: Vec<String>,
    pub claim_token: Uuid,
    pub lease_expires_at: DateTime<Utc>,
    pub attempt_runtime_deadline: DateTime<Utc>,
    pub attempt_number: i32,
    pub effect_phase: AsyncEffectPhase,
    pub checkpoint: Option<String>,
    pub checkpoint_version: i32,
    pub provider_ref: Option<String>,
}

impl fmt::Debug for ClaimedJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaimedJob")
            .field("async_job_id", &self.async_job_id)
            .field("kind", &self.kind)
            .field("payload_version", &self.payload_version)
            .field("claim_token", &"<redacted>")
            .field("lease_expires_at", &self.lease_expires_at)
            .field("attempt_number", &self.attempt_number)
            .field("effect_phase", &self.effect_phase)
            .finish_non_exhaustive()
    }
}

impl ClaimedJob {
    /// The worker claim a handler passes to every follow-up write.
    pub fn worker_claim(&self, domain: ExecutorDomain) -> WorkerClaim {
        WorkerClaim {
            async_job_id: self.async_job_id,
            claim_token: self.claim_token,
            domain,
        }
    }
}

/// The claimed jobs, in claim order, and the requested kinds this binary
/// does not know.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClaimResult {
    pub jobs: Vec<ClaimedJob>,
    pub skipped_kinds: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaimPurpose {
    Execution,
    Reconciliation,
}

impl ClaimPurpose {
    fn namespace(self) -> i32 {
        match self {
            ClaimPurpose::Execution => NS_KIND,
            ClaimPurpose::Reconciliation => NS_RECON,
        }
    }

    fn domain(self, policy: &AsyncJobKindPolicy) -> ExecutorDomain {
        match self {
            ClaimPurpose::Execution => policy.executor_domain,
            ClaimPurpose::Reconciliation => policy.reconciler_domain,
        }
    }

    fn concurrency_limit(self, policy: &AsyncJobKindPolicy) -> i32 {
        match self {
            ClaimPurpose::Execution => policy.concurrency_limit,
            ClaimPurpose::Reconciliation => policy.reconciliation_concurrency_limit,
        }
    }

    fn attempt_purpose(self) -> AsyncAttemptPurpose {
        match self {
            ClaimPurpose::Execution => AsyncAttemptPurpose::Execution,
            ClaimPurpose::Reconciliation => AsyncAttemptPurpose::Reconciliation,
        }
    }
}

fn validate_executor(executor: &Executor) -> ThothResult<()> {
    require_no_nul(&executor.name, "executor name")?;
    require_no_nul(&executor.principal, "executor principal")?;
    if !is_non_blank(&executor.name) || executor.name.chars().count() > CLAIMED_BY_MAX_CHARS {
        return Err(invalid(
            "the executor name must be non-blank and at most 128 characters",
        ));
    }
    if !is_non_blank(&executor.principal)
        || executor.principal.chars().count() > PRINCIPAL_MAX_CHARS
    {
        return Err(invalid(
            "the executor principal must be non-blank and at most 256 characters",
        ));
    }
    Ok(())
}

/// Domain, risk class, capabilities and revision mode must all match.
fn authorize(
    executor: &Executor,
    policy: &AsyncJobKindPolicy,
    purpose: ClaimPurpose,
) -> ThothResult<()> {
    let authorized = executor.domain == purpose.domain(policy)
        && executor.risk_class == policy.risk_class
        && policy
            .required_capabilities
            .iter()
            .all(|needed| executor.capabilities.iter().any(|held| held == needed))
        && executor.revision_modes.contains(&policy.revision_mode);
    if authorized {
        Ok(())
    } else {
        Err(ThothError::AsyncJobKindNotAuthorized)
    }
}

/// T2: claim ready jobs of the requested kinds (section 10.2).
pub fn claim_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    executor: &Executor,
    request: ClaimRequest,
) -> ThothResult<ClaimResult> {
    claim_jobs(conn, registry, executor, request, ClaimPurpose::Execution)
}

/// T15: claim unclaimed `RECONCILIATION_REQUIRED` jobs as their reconciler
/// (section 10.4).
pub fn claim_reconciliation_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    executor: &Executor,
    request: ClaimRequest,
) -> ThothResult<ClaimResult> {
    claim_jobs(
        conn,
        registry,
        executor,
        request,
        ClaimPurpose::Reconciliation,
    )
}

struct KnownKind {
    policy: &'static AsyncJobKindPolicy,
    versions: BTreeSet<i16>,
}

fn claim_jobs(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    executor: &Executor,
    request: ClaimRequest,
    purpose: ClaimPurpose,
) -> ThothResult<ClaimResult> {
    validate_executor(executor)?;
    if !(1..=CLAIM_MAX_JOBS).contains(&request.max_jobs) {
        return Err(invalid("max_jobs must be between 1 and 50"));
    }
    let mut known: BTreeMap<String, KnownKind> = BTreeMap::new();
    let mut skipped_kinds: Vec<String> = Vec::new();
    for (kind, versions) in &request.kinds {
        if versions.is_empty() {
            return Err(invalid(
                "every requested kind names at least one payload version",
            ));
        }
        match registry.get(kind) {
            Some(policy) => known
                .entry(policy.kind.to_string())
                .or_insert_with(|| KnownKind {
                    policy,
                    versions: BTreeSet::new(),
                })
                .versions
                .extend(versions.iter().copied()),
            None => {
                if !skipped_kinds.contains(kind) {
                    skipped_kinds.push(kind.clone());
                }
            }
        }
    }
    // No partial authorization: one mismatch refuses the whole call.
    for entry in known.values() {
        authorize(executor, entry.policy, purpose)?;
    }
    run_op(conn, |conn| {
        if known.is_empty() {
            // S1: a claim naming only unknown kinds writes nothing.
            return Ok(ClaimResult {
                jobs: Vec::new(),
                skipped_kinds,
            });
        }
        let names: Vec<String> = known.keys().cloned().collect();
        let ordered = db(lock_key_rows(conn, &names))?;
        let mut previous = None;
        for row in &ordered {
            if previous != Some(row.key) {
                db(sql_query("SELECT pg_advisory_xact_lock($1, $2)")
                    .bind::<Int4, _>(purpose.namespace())
                    .bind::<Int4, _>(row.key)
                    .execute(conn))?;
                previous = Some(row.key);
            }
        }
        recover_rows(conn, registry, Some(&names), CLAIM_RECOVERY_LIMIT)?;
        let now = db(clock_now(conn))?;

        let mut queues: BTreeMap<String, (i64, VecDeque<JobRow>)> = BTreeMap::new();
        for (kind, entry) in &known {
            let claimed = count_claimed(conn, kind, purpose)?;
            let free = i64::from(purpose.concurrency_limit(entry.policy)) - claimed;
            if free <= 0 {
                continue;
            }
            let executable: Vec<i16> = entry
                .versions
                .iter()
                .copied()
                .filter(|version| entry.policy.is_executable_version(*version))
                .collect();
            if executable.is_empty() {
                continue;
            }
            let window = 4 * free.min(i64::from(request.max_jobs));
            let candidates = load_candidates(conn, kind, &executable, now, window, purpose)?;
            queues.insert(kind.clone(), (free, candidates.into()));
        }

        // Fairness (section 12): one job at a time from the kind whose best
        // remaining candidate has the oldest `available_at`.
        let mut jobs = Vec::new();
        let mut claimed_kinds = BTreeSet::new();
        while jobs.len() < request.max_jobs as usize {
            let next = queues
                .iter()
                .filter(|(_, (free, queue))| *free > 0 && !queue.is_empty())
                .min_by(|(a_kind, (_, a)), (b_kind, (_, b))| {
                    let (a, b) = (&a[0], &b[0]);
                    (
                        a.available_at,
                        a.created_at,
                        a.async_job_id,
                        a_kind.as_str(),
                    )
                        .cmp(&(
                            b.available_at,
                            b.created_at,
                            b.async_job_id,
                            b_kind.as_str(),
                        ))
                })
                .map(|(kind, _)| kind.clone());
            let Some(kind) = next else {
                break;
            };
            let (free, queue) = queues.get_mut(&kind).expect("the kind has a queue");
            let job = queue.pop_front().expect("the queue is not empty");
            let policy = known[&kind].policy;
            if let Some(claimed) = claim_candidate(
                conn,
                policy,
                executor,
                &job,
                request.lease_seconds,
                now,
                purpose,
            )? {
                *free -= 1;
                claimed_kinds.insert(kind);
                jobs.push(claimed);
            }
        }

        // Poll rows, ascending by kind lock key, under their kind locks.
        for row in &ordered {
            let entry = &known[&row.name];
            let versions: Vec<i16> = entry.versions.iter().copied().collect();
            db(upsert_poll(
                conn,
                executor,
                &row.name,
                purpose,
                &versions,
                now,
                claimed_kinds.contains(&row.name),
            ))?;
        }
        Ok(ClaimResult {
            jobs,
            skipped_kinds,
        })
    })
}

fn count_claimed(conn: &mut PgConnection, kind: &str, purpose: ClaimPurpose) -> ThothResult<i64> {
    let sql = match purpose {
        ClaimPurpose::Execution => {
            "SELECT count(*) AS count FROM public.async_job \
              WHERE kind = $1 AND status = 'RUNNING'"
        }
        ClaimPurpose::Reconciliation => {
            "SELECT count(*) AS count FROM public.async_job \
              WHERE kind = $1 AND status = 'RECONCILIATION_REQUIRED' \
                AND claim_token IS NOT NULL"
        }
    };
    db(sql_query(sql)
        .bind::<Text, _>(kind)
        .get_result::<CountRow>(conn))
    .map(|row| row.count)
}

/// Claimable candidates of one kind, locked `FOR UPDATE SKIP LOCKED`, in the
/// kind's order (section 12), skipping jobs whose targets another job holds.
fn load_candidates(
    conn: &mut PgConnection,
    kind: &str,
    versions: &[i16],
    now: DateTime<Utc>,
    window: i64,
    purpose: ClaimPurpose,
) -> ThothResult<Vec<JobRow>> {
    let sql = match purpose {
        ClaimPurpose::Execution => format!(
            "SELECT {JOB_COLUMNS} FROM public.async_job j \
              WHERE j.kind = $1 \
                AND j.payload_version = ANY($2::smallint[]) \
                AND j.available_at <= $3 \
                AND ((j.status IN ('PENDING', 'RETRY_SCHEDULED') \
                      AND j.execution_claim_count < j.max_execution_claims) \
                     OR (j.status = 'WAITING' AND j.waiting_deadline > $3)) \
                AND NOT EXISTS ( \
                      SELECT 1 FROM public.async_job_target t \
                        JOIN public.async_target_hold h ON h.target_key = t.target_key \
                       WHERE t.async_job_id = j.async_job_id \
                         AND h.async_job_id <> j.async_job_id) \
              ORDER BY j.priority DESC, j.available_at, j.created_at, j.async_job_id \
              LIMIT $4 \
              FOR UPDATE OF j SKIP LOCKED"
        ),
        ClaimPurpose::Reconciliation => format!(
            "SELECT {JOB_COLUMNS} FROM public.async_job j \
              WHERE j.kind = $1 \
                AND j.payload_version = ANY($2::smallint[]) \
                AND j.status = 'RECONCILIATION_REQUIRED' \
                AND j.claim_token IS NULL \
                AND j.available_at <= $3 \
                AND j.reconciliation_attempt_count < j.max_reconciliation_attempts \
              ORDER BY j.available_at, j.async_job_id \
              LIMIT $4 \
              FOR UPDATE OF j SKIP LOCKED"
        ),
    };
    db(sql_query(sql)
        .bind::<Text, _>(kind)
        .bind::<Array<Int2>, _>(versions)
        .bind::<Timestamptz, _>(now)
        .bind::<BigInt, _>(window)
        .load::<JobRow>(conn))
}

/// Claim one candidate inside its own savepoint (section 10.2 step 6): try
/// the hold locks, insert the missing holds, apply T2 (or T15), force the
/// deferred checks. Any refusal or violation rolls the candidate back and
/// skips it; it stays claimable.
fn claim_candidate(
    conn: &mut PgConnection,
    policy: &AsyncJobKindPolicy,
    executor: &Executor,
    job: &JobRow,
    lease_seconds: Option<i32>,
    now: DateTime<Utc>,
    purpose: ClaimPurpose,
) -> ThothResult<Option<ClaimedJob>> {
    let result = conn.transaction(|conn| -> Result<ClaimedJob, StepError> {
        if purpose == ClaimPurpose::Execution {
            let keys = targets::unheld_target_keys(conn, job.async_job_id)?;
            if !targets::try_lock_holds(conn, &keys)? {
                return Err(StepError::Skip);
            }
            if targets::insert_holds(conn, job.async_job_id, &keys)? != keys.len() {
                return Err(StepError::Skip);
            }
        }
        let claimed = open_claim(conn, policy, executor, job, lease_seconds, now, purpose)?;
        force_hold_and_attempt_checks(conn)?;
        Ok(claimed)
    });
    match result {
        Ok(claimed) => Ok(Some(claimed)),
        Err(StepError::Skip) => Ok(None),
        Err(StepError::Db(error)) if is_busy_error(&error) => Err(ThothError::AsyncJobBusy),
        Err(StepError::Db(error)) => {
            log_step_failure("claim skipped a candidate", job.async_job_id, &error);
            Ok(None)
        }
    }
}

/// Log a contained step failure by constraint name, never row content.
fn log_step_failure(what: &str, job_id: Uuid, error: &DieselError) {
    match error {
        DieselError::DatabaseError(_, info) => log::warn!(
            "async job engine: {what} {job_id} (constraint: {}): {}",
            info.constraint_name().unwrap_or("none"),
            info.message()
        ),
        _ => log::warn!("async job engine: {what} {job_id}: non-database error"),
    }
}

/// T2 or T15 on a locked candidate.
fn open_claim(
    conn: &mut PgConnection,
    policy: &AsyncJobKindPolicy,
    executor: &Executor,
    job: &JobRow,
    lease_seconds: Option<i32>,
    now: DateTime<Utc>,
    purpose: ClaimPurpose,
) -> QueryResult<ClaimedJob> {
    let token = Uuid::new_v4();
    let attempt_number = job.attempt_count + 1;
    let attempt_id = insert_attempt(
        conn,
        &NewAttempt {
            job_id: job.async_job_id,
            attempt_number,
            purpose: purpose.attempt_purpose(),
            token,
            claimed_by: &executor.name,
            principal: &executor.principal,
            command_id: None,
            now,
            phase: job.effect_phase,
        },
    )?;
    let deadline = now + Duration::seconds(i64::from(policy.max_attempt_runtime_seconds));
    let lease_expires_at = (now
        + Duration::seconds(i64::from(policy.clamp_lease_seconds(lease_seconds))))
    .min(deadline);
    let mut change = JobChange::from_row(job);
    change.claim_token = Some(token);
    change.claimed_by = Some(executor.name.clone());
    change.executor_principal = Some(executor.principal.clone());
    change.claimed_at = Some(now);
    change.lease_expires_at = Some(lease_expires_at);
    change.attempt_runtime_deadline = Some(deadline);
    change.attempt_count = attempt_number;
    match purpose {
        ClaimPurpose::Execution => {
            if job.status != AsyncJobStatus::Waiting {
                change.execution_claim_count += 1;
            }
            change.status = AsyncJobStatus::Running;
            change.first_claimed_at = job.first_claimed_at.or(Some(now));
            set_cause(conn, T2_CLAIM, Some(attempt_id), None)?;
        }
        ClaimPurpose::Reconciliation => {
            change.reconciliation_attempt_count += 1;
        }
    }
    write_job(conn, job.async_job_id, &change)?;
    Ok(ClaimedJob {
        async_job_id: job.async_job_id,
        kind: job.kind.clone(),
        payload_version: job.payload_version,
        payload: job.payload.clone(),
        source_order: job.source_order,
        source_fingerprint: job.source_fingerprint.clone(),
        source_ref: job.source_ref.clone(),
        target_keys: job.target_keys.clone(),
        claim_token: token,
        lease_expires_at,
        attempt_runtime_deadline: deadline,
        attempt_number,
        effect_phase: job.effect_phase,
        checkpoint: job.checkpoint.clone(),
        checkpoint_version: job.checkpoint_version,
        provider_ref: job.provider_ref.clone(),
    })
}

fn upsert_poll(
    conn: &mut PgConnection,
    executor: &Executor,
    kind: &str,
    purpose: ClaimPurpose,
    versions: &[i16],
    now: DateTime<Utc>,
    claimed: bool,
) -> QueryResult<()> {
    sql_query(
        "INSERT INTO public.async_executor_poll \
             (executor_name, executor_principal, kind, purpose, payload_versions, \
              last_poll_at, last_claim_at) \
         VALUES ($1, $2, $3, $4::public.async_attempt_purpose, $5::smallint[], $6, $7) \
         ON CONFLICT (executor_name, executor_principal, kind, purpose) DO UPDATE SET \
             payload_versions = EXCLUDED.payload_versions, \
             last_poll_at = EXCLUDED.last_poll_at, \
             last_claim_at = COALESCE(EXCLUDED.last_claim_at, \
                                      public.async_executor_poll.last_claim_at)",
    )
    .bind::<Text, _>(&executor.name)
    .bind::<Text, _>(&executor.principal)
    .bind::<Text, _>(kind)
    .bind::<Text, _>(purpose.attempt_purpose().as_str())
    .bind::<Array<Int2>, _>(versions)
    .bind::<Timestamptz, _>(now)
    .bind::<Nullable<Timestamptz>, _>(if claimed { Some(now) } else { None })
    .execute(conn)
    .map(|_| ())
}

// ---------------------------------------------------------------------------
// Worker writes (section 10.3; T3 to T13, T16)
// ---------------------------------------------------------------------------

/// What a worker presents on every token-fenced write: the job, the claim
/// token from its claim, and the caller's executor domain (from the
/// authenticated identity). Its `Debug` never prints the token.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WorkerClaim {
    pub async_job_id: Uuid,
    pub claim_token: Uuid,
    pub domain: ExecutorDomain,
}

impl fmt::Debug for WorkerClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerClaim")
            .field("async_job_id", &self.async_job_id)
            .field("claim_token", &"<redacted>")
            .field("domain", &self.domain)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Need {
    Execution,
    Reconciliation,
    Either,
}

/// A token-fenced write's locked state.
struct Worker {
    job: JobRow,
    policy: &'static AsyncJobKindPolicy,
    attempt_id: Uuid,
    now: DateTime<Utc>,
}

/// Lock the job row, check the kind, the executor domain and the token, then
/// read `now` (section 10.3).
fn worker_prelude(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    need: Need,
) -> ThothResult<Worker> {
    let job = lock_job(conn, claim.async_job_id)?.ok_or(ThothError::AsyncJobStaleClaim)?;
    let policy = registry
        .get(&job.kind)
        .ok_or(ThothError::AsyncJobKindUnknown)?;
    if claim.domain != policy.executor_domain && claim.domain != policy.reconciler_domain {
        return Err(ThothError::AsyncJobKindNotAuthorized);
    }
    if job.claim_token != Some(claim.claim_token) {
        return Err(ThothError::AsyncJobStaleClaim);
    }
    let reconciliation = job.status == AsyncJobStatus::ReconciliationRequired;
    let domain = if reconciliation {
        policy.reconciler_domain
    } else {
        policy.executor_domain
    };
    if claim.domain != domain {
        return Err(ThothError::AsyncJobKindNotAuthorized);
    }
    match (need, reconciliation) {
        (Need::Execution, true) => return Err(refused("RECONCILIATION_IN_PROGRESS")),
        (Need::Reconciliation, false) => return Err(refused("NOT_RECONCILIATION_REQUIRED")),
        _ => {}
    }
    let attempt_id = db(open_attempt_id(conn, claim.claim_token))?;
    let now = db(clock_now(conn))?;
    Ok(Worker {
        job,
        policy,
        attempt_id,
        now,
    })
}

fn reread(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<AsyncJob> {
    read_job(conn, job_id)?
        .map(AsyncJob::from)
        .ok_or(ThothError::AsyncJobIntegrityViolation)
}

/// A renewed lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseRenewal {
    pub lease_expires_at: DateTime<Utc>,
    pub attempt_runtime_deadline: DateTime<Utc>,
}

/// T3: renew an execution or reconciliation claim. Needs the current token
/// and an unexpired lease; the lease never passes the runtime deadline.
pub fn renew_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    lease_seconds: Option<i32>,
) -> ThothResult<LeaseRenewal> {
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Either)?;
        let (job, now) = (&worker.job, worker.now);
        let expires = job
            .lease_expires_at
            .ok_or(ThothError::AsyncJobIntegrityViolation)?;
        let deadline = job
            .attempt_runtime_deadline
            .ok_or(ThothError::AsyncJobIntegrityViolation)?;
        if expires <= now {
            return Err(ThothError::AsyncJobLeaseExpired);
        }
        let lease = (now
            + Duration::seconds(i64::from(worker.policy.clamp_lease_seconds(lease_seconds))))
        .min(deadline);
        db(sql_query(
            "UPDATE public.async_job_attempt SET last_heartbeat_at = $2 \
              WHERE async_job_attempt_id = $1",
        )
        .bind::<SqlUuid, _>(worker.attempt_id)
        .bind::<Timestamptz, _>(now)
        .execute(conn))?;
        let mut change = JobChange::from_row(job);
        change.lease_expires_at = Some(lease);
        db(write_job(conn, job.async_job_id, &change))?;
        Ok(LeaseRenewal {
            lease_expires_at: lease,
            attempt_runtime_deadline: deadline,
        })
    })
}

/// T4: mark the effect started. Needs an EFFECTFUL kind, phase `PRE_WRITE`
/// and the effect window left on both the lease and the runtime deadline.
pub fn mark_effect_started_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
) -> ThothResult<AsyncJob> {
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now) = (&worker.job, worker.now);
        if job.effect_class != EffectClass::Effectful {
            return Err(refused("KIND_POLICY"));
        }
        if job.effect_phase != AsyncEffectPhase::PreWrite {
            return Err(refused("PHASE"));
        }
        let window = Duration::seconds(i64::from(
            worker.policy.effect_window_seconds.unwrap_or(i32::MAX),
        ));
        let lease = job
            .lease_expires_at
            .ok_or(ThothError::AsyncJobIntegrityViolation)?;
        let deadline = job
            .attempt_runtime_deadline
            .ok_or(ThothError::AsyncJobIntegrityViolation)?;
        if lease - now < window || deadline - now < window {
            return Err(ThothError::AsyncJobEffectWindowInsufficient);
        }
        db(sql_query(
            "UPDATE public.async_job_attempt SET effect_started_at = $2 \
              WHERE async_job_attempt_id = $1",
        )
        .bind::<SqlUuid, _>(worker.attempt_id)
        .bind::<Timestamptz, _>(now)
        .execute(conn))?;
        let mut change = JobChange::from_row(job);
        change.effect_phase = AsyncEffectPhase::EffectStarted;
        db(write_job(conn, job.async_job_id, &change))?;
        reread(conn, job.async_job_id)
    })
}

/// T5: mark the effect confirmed, with an optional provider reference (free
/// text: control characters stripped, trimmed, truncated to 512).
pub fn mark_effect_confirmed_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    provider_ref: Option<&str>,
) -> ThothResult<AsyncJob> {
    if let Some(value) = provider_ref {
        require_no_nul(value, "provider reference")?;
    }
    let provider_ref =
        provider_ref.and_then(|value| sanitize_free_text(value, PROVIDER_REF_MAX_CHARS));
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now) = (&worker.job, worker.now);
        if job.effect_phase != AsyncEffectPhase::EffectStarted {
            return Err(refused("PHASE"));
        }
        db(sql_query(
            "UPDATE public.async_job_attempt SET effect_confirmed_at = $2, \
                    provider_ref = COALESCE($3, provider_ref) \
              WHERE async_job_attempt_id = $1",
        )
        .bind::<SqlUuid, _>(worker.attempt_id)
        .bind::<Timestamptz, _>(now)
        .bind::<Nullable<Text>, _>(provider_ref.as_deref())
        .execute(conn))?;
        let mut change = JobChange::from_row(job);
        change.effect_phase = AsyncEffectPhase::EffectConfirmed;
        if provider_ref.is_some() {
            change.provider_ref = provider_ref.clone();
        }
        db(write_job(conn, job.async_job_id, &change))?;
        reread(conn, job.async_job_id)
    })
}

/// The input of [`wait_on`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitRequest {
    /// A JSON object as text (at most 65536 bytes as `jsonb`).
    pub checkpoint: String,
    pub resume_at: DateTime<Utc>,
    pub provider_ref: Option<String>,
}

/// The result of [`wait_on`]: `WAITING` (T10), or the new state when the
/// stored waiting deadline had passed (T10d), returned as an outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum WaitOutcome {
    Waiting(AsyncJob),
    DeadlineExceeded(AsyncJob),
}

/// T10 / T10d: checkpoint and yield the claim until `resume_at`.
pub fn wait_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    request: WaitRequest,
) -> ThothResult<WaitOutcome> {
    parse_json_object(&request.checkpoint, true)
        .map_err(|fault| invalid(fault.describe(JsonInput::Checkpoint)))?;
    if let Some(value) = &request.provider_ref {
        require_no_nul(value, "provider reference")?;
    }
    let provider_ref = request
        .provider_ref
        .as_deref()
        .and_then(|value| sanitize_free_text(value, PROVIDER_REF_MAX_CHARS));
    run_op(conn, |conn| {
        measure_json(conn, &request.checkpoint, JsonInput::Checkpoint)?
            .map_err(|fault| invalid(fault.describe(JsonInput::Checkpoint)))?;
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now) = (&worker.job, worker.now);
        if job.waiting_deadline.is_some_and(|deadline| deadline <= now) {
            return apply_deadline_exceeded(conn, &worker).map(WaitOutcome::DeadlineExceeded);
        }
        let earliest = now + Duration::seconds(i64::from(worker.policy.poll_interval_min_seconds));
        let latest = job
            .waiting_deadline
            .unwrap_or(now + Duration::seconds(worker.policy.waiting_horizon_seconds));
        if request.resume_at < earliest || request.resume_at > latest {
            return Err(invalid(
                "resume_at must be at least the poll interval away and not after the waiting deadline",
            ));
        }
        let mut close = AttemptClose::new(
            AsyncAttemptResult::Waiting,
            AsyncJobStatus::Waiting,
            "WAITING",
        );
        close.provider_ref = provider_ref.as_deref();
        db(close_attempt(conn, worker.attempt_id, now, &close))?;
        let mut change = JobChange::from_row(job);
        change.clear_claim();
        change.status = AsyncJobStatus::Waiting;
        change.checkpoint = CheckpointWrite::Set(request.checkpoint.clone());
        change.checkpoint_version += 1;
        if provider_ref.is_some() {
            change.provider_ref = provider_ref.clone();
        }
        change.available_at = request.resume_at;
        change.waiting_deadline = job.waiting_deadline.or(Some(
            now + Duration::seconds(worker.policy.waiting_horizon_seconds),
        ));
        change.outcome("WAITING");
        db(set_cause(conn, T10_WAIT, Some(worker.attempt_id), None))?;
        db(write_job(conn, job.async_job_id, &change))?;
        Ok(WaitOutcome::Waiting(reread(conn, job.async_job_id)?))
    })
}

/// T10d: `wait` after the stored deadline. RR after effect start (holds
/// kept), FAILED before it (holds released).
fn apply_deadline_exceeded(conn: &mut PgConnection, worker: &Worker) -> ThothResult<AsyncJob> {
    let (job, now) = (&worker.job, worker.now);
    let after_start = job.effect_phase >= AsyncEffectPhase::EffectStarted;
    let mut change = JobChange::from_row(job);
    let close = if after_start {
        change.require_reconciliation(now);
        AttemptClose::new(
            AsyncAttemptResult::Indeterminate,
            AsyncJobStatus::ReconciliationRequired,
            "DEADLINE_EXCEEDED",
        )
    } else {
        db(targets::lock_holds_waiting(conn, &job.target_keys))?;
        db(targets::delete_holds(conn, job.async_job_id))?;
        change.terminate(AsyncJobStatus::Failed, now);
        AttemptClose::new(
            AsyncAttemptResult::PermanentFailure,
            AsyncJobStatus::Failed,
            "DEADLINE_EXCEEDED",
        )
    };
    db(close_attempt(conn, worker.attempt_id, now, &close))?;
    change.outcome("DEADLINE_EXCEEDED");
    db(set_cause(
        conn,
        T10D_WAIT_DEADLINE_EXCEEDED,
        Some(worker.attempt_id),
        None,
    ))?;
    db(write_job(conn, job.async_job_id, &change))?;
    reread(conn, job.async_job_id)
}

/// The outcome a `complete` call reports (section 13.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionOutcome {
    EffectApplied,
    NoOp,
    Superseded,
    /// Read-only kinds.
    Completed,
}

impl CompletionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            CompletionOutcome::EffectApplied => "EFFECT_APPLIED",
            CompletionOutcome::NoOp => "NO_OP",
            CompletionOutcome::Superseded => "SUPERSEDED",
            CompletionOutcome::Completed => "COMPLETED",
        }
    }
}

/// The input of [`complete_on`]: one entry per job target, and the optional
/// kind-defined evidence object (as JSON text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteRequest {
    pub outcome: CompletionOutcome,
    pub entries: Vec<EvidenceEntry>,
    pub evidence: Option<String>,
}

/// `SUCCEEDED` (T6), or `RECONCILIATION_REQUIRED` after a semantic evidence
/// fault past effect start (T6x), returned as an outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum CompleteOutcome {
    Succeeded(AsyncJob),
    ReconciliationRequired(AsyncJob),
}

/// The checks of an outcome against the phase and the kind, in the order
/// PHASE, KIND_POLICY (section 13.3).
fn completion_rule(outcome: CompletionOutcome, job: &JobRow) -> ThothResult<EntryRule> {
    let effectful = job.effect_class == EffectClass::Effectful;
    let revision_bound = job.revision_mode == RevisionMode::RevisionBound;
    let pre_write = job.effect_phase == AsyncEffectPhase::PreWrite;
    let (phase_allows, kind_allows, rule) = match outcome {
        CompletionOutcome::EffectApplied => (!pre_write, effectful, EntryRule::EffectApplied),
        CompletionOutcome::NoOp => (pre_write, effectful, EntryRule::NoOp),
        CompletionOutcome::Superseded => (
            pre_write,
            effectful && revision_bound,
            EntryRule::Superseded,
        ),
        CompletionOutcome::Completed => (true, !effectful, EntryRule::Completed),
    };
    if !phase_allows {
        return Err(refused("PHASE"));
    }
    if !kind_allows {
        return Err(refused("KIND_POLICY"));
    }
    Ok(rule)
}

/// The stored payload, parsed for a kind function.
pub(crate) fn stored_payload(job: &JobRow) -> ThothResult<serde_json::Value> {
    serde_json::from_str(&job.payload).map_err(|_| {
        log::error!(
            "async job engine: the stored payload of job {} does not parse",
            job.async_job_id
        );
        ThothError::AsyncJobIntegrityViolation
    })
}

/// The `EvidenceContext` of a job (section 8).
pub(crate) fn evidence_context<'a>(
    job: &'a JobRow,
    payload: &'a serde_json::Value,
) -> EvidenceContext<'a> {
    EvidenceContext {
        payload_version: job.payload_version,
        payload,
        source_order: job.source_order,
        source_fingerprint: job.source_fingerprint.as_deref(),
        source_ref: job.source_ref.as_deref(),
        target_keys: &job.target_keys,
        provider_ref: job.provider_ref.as_deref(),
    }
}

/// The shape checks of a worker's evidence object: it parses, has no
/// U+0000, is an object without a credential-named key, and `jsonb` takes it
/// within 16384 bytes. `Err` carries the shape fault's description.
fn evidence_shape(
    conn: &mut PgConnection,
    text: &str,
) -> ThothResult<Result<serde_json::Value, String>> {
    let value = match parse_json_object(text, true) {
        Ok(value) => value,
        Err(fault) => return Ok(Err(fault.describe(JsonInput::Evidence))),
    };
    Ok(match measure_json(conn, text, JsonInput::Evidence)? {
        Ok(_) => Ok(value),
        Err(fault) => Err(fault.describe(JsonInput::Evidence)),
    })
}

/// The semantic-fault code of T6x.
fn semantic_code(fault: &SemanticFault) -> &'static str {
    match fault {
        SemanticFault::Regression => "EVIDENCE_REGRESSION",
        SemanticFault::Invalid(_) => "EVIDENCE_INVALID",
    }
}

pub(crate) fn semantic_message(fault: &SemanticFault) -> String {
    match fault {
        SemanticFault::Regression => "the stored order is newer than the entry".to_string(),
        SemanticFault::Invalid(message) => message.clone(),
    }
}

/// T6 / T6x: complete with outcome, entries and evidence (section 13.3).
pub fn complete_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    request: CompleteRequest,
) -> ThothResult<CompleteOutcome> {
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now, policy) = (&worker.job, worker.now, worker.policy);
        // State, phase and kind checks come before every evidence fault.
        let rule = completion_rule(request.outcome, job)?;
        let after_start = job.effect_phase >= AsyncEffectPhase::EffectStarted;
        let evidence_allowed = matches!(
            request.outcome,
            CompletionOutcome::EffectApplied | CompletionOutcome::NoOp
        );

        // Shape faults: nothing written, the claim stays live.
        if request.evidence.is_some() && !evidence_allowed {
            return Err(evidence_invalid(format!(
                "{} takes no evidence object",
                request.outcome.as_str()
            )));
        }
        check_entry_shapes(&request.entries, rule, job.revision_mode).map_err(evidence_invalid)?;
        let payload = stored_payload(job)?;
        let context = evidence_context(job, &payload);
        let evidence = match &request.evidence {
            Some(text) => Some(evidence_shape(conn, text)?.map_err(evidence_invalid)?),
            None => {
                if evidence_allowed
                    && (policy.validate_evidence)(
                        EvidenceUse::Applied,
                        &context,
                        &serde_json::Value::Null,
                    )
                    .is_err()
                {
                    return Err(evidence_invalid("the kind requires an evidence object"));
                }
                None
            }
        };

        // Semantic faults, under the holds (rule H before the state rows).
        db(targets::lock_holds_waiting(conn, &job.target_keys))?;
        let states = db(targets::lock_state_rows(conn, &job.target_keys))?;
        let semantic =
            check_entries_semantic(&request.entries, rule, job, &states).and_then(|writes| {
                if let Some(value) = &evidence {
                    (policy.validate_evidence)(EvidenceUse::Applied, &context, value)
                        .map_err(|error| SemanticFault::Invalid(error.0))?;
                }
                Ok(writes)
            });
        let evidence_text = request.evidence.as_deref();
        match semantic {
            Ok(writes) => {
                db(targets::write_states(
                    conn,
                    &writes,
                    job.async_job_id,
                    worker.attempt_id,
                ))?;
                db(targets::delete_holds(conn, job.async_job_id))?;
                let mut close = AttemptClose::new(
                    AsyncAttemptResult::Succeeded,
                    AsyncJobStatus::Succeeded,
                    request.outcome.as_str(),
                );
                close.evidence = evidence_text;
                db(close_attempt(conn, worker.attempt_id, now, &close))?;
                let mut change = JobChange::from_row(job);
                change.terminate(AsyncJobStatus::Succeeded, now);
                change.outcome(request.outcome.as_str());
                db(set_cause(conn, T6_COMPLETE, Some(worker.attempt_id), None))?;
                db(write_job(conn, job.async_job_id, &change))?;
                Ok(CompleteOutcome::Succeeded(reread(conn, job.async_job_id)?))
            }
            Err(fault) if !after_start => Err(evidence_invalid(semantic_message(&fault))),
            Err(fault) => {
                let code = semantic_code(&fault);
                let mut close = AttemptClose::new(
                    AsyncAttemptResult::Indeterminate,
                    AsyncJobStatus::ReconciliationRequired,
                    code,
                );
                close.evidence = evidence_text;
                db(close_attempt(conn, worker.attempt_id, now, &close))?;
                let mut change = JobChange::from_row(job);
                change.require_reconciliation(now);
                change.outcome(code);
                db(set_cause(
                    conn,
                    T6X_EVIDENCE_FAULT,
                    Some(worker.attempt_id),
                    None,
                ))?;
                db(write_job(conn, job.async_job_id, &change))?;
                Ok(CompleteOutcome::ReconciliationRequired(reread(
                    conn,
                    job.async_job_id,
                )?))
            }
        }
    })
}

/// The class a worker reports with `fail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    RetryablePreWrite,
    Permanent,
    Indeterminate,
}

impl FailureClass {
    fn attempt_result(self) -> AsyncAttemptResult {
        match self {
            FailureClass::RetryablePreWrite => AsyncAttemptResult::RetryableFailure,
            FailureClass::Permanent => AsyncAttemptResult::PermanentFailure,
            FailureClass::Indeterminate => AsyncAttemptResult::Indeterminate,
        }
    }
}

/// The input of [`fail_on`]. `error_code` is required and validated;
/// `error_detail` is free text (control characters stripped, trimmed,
/// truncated to 2048). `evidence` is taken only with `no_effect_proven`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailRequest {
    pub class: FailureClass,
    pub error_code: String,
    pub error_detail: Option<String>,
    pub no_effect_proven: bool,
    pub evidence: Option<String>,
}

/// T7, T8, T9 and T9n: report a failure.
pub fn fail_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    request: FailRequest,
) -> ThothResult<AsyncJob> {
    require_no_nul(&request.error_code, "error code")?;
    if !is_code(&request.error_code) {
        return Err(invalid(
            "the error code must match ^[A-Z][A-Z0-9_]*$ and be at most 64 characters",
        ));
    }
    if let Some(detail) = &request.error_detail {
        require_no_nul(detail, "error detail")?;
    }
    let error_detail = request
        .error_detail
        .as_deref()
        .and_then(|detail| sanitize_free_text(detail, ERROR_DETAIL_MAX_CHARS));
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now) = (&worker.job, worker.now);
        let phase = job.effect_phase;
        let mut change = JobChange::from_row(job);
        change.clear_claim();

        if request.no_effect_proven {
            // T9n or refused, never T9: PHASE, EFFECT_CONFIRMED, KIND_POLICY,
            // CLASS, first applicable.
            let reasons = [
                (phase == AsyncEffectPhase::PreWrite, "PHASE"),
                (
                    phase == AsyncEffectPhase::EffectConfirmed,
                    "EFFECT_CONFIRMED",
                ),
                (!worker.policy.no_effect_after_start_allowed, "KIND_POLICY"),
                (request.class != FailureClass::Permanent, "CLASS"),
            ];
            if let Some((_, reason)) = reasons.iter().find(|(applies, _)| *applies) {
                return Err(refused(reason));
            }
            let Some(text) = request.evidence.as_deref() else {
                return Err(evidence_invalid(
                    "a no-effect failure requires an evidence object",
                ));
            };
            let value = evidence_shape(conn, text)?.map_err(evidence_invalid)?;
            let payload = stored_payload(job)?;
            (worker.policy.validate_evidence)(
                EvidenceUse::NoEffect,
                &evidence_context(job, &payload),
                &value,
            )
            .map_err(|error| evidence_invalid(error.0))?;
            db(targets::lock_holds_waiting(conn, &job.target_keys))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
            let mut close = AttemptClose::new(
                AsyncAttemptResult::PermanentFailure,
                AsyncJobStatus::Failed,
                "NO_EFFECT_PROVEN",
            );
            close.error_code = Some(&request.error_code);
            close.error_detail = error_detail.as_deref();
            close.evidence = Some(text);
            db(close_attempt(conn, worker.attempt_id, now, &close))?;
            change.terminate(AsyncJobStatus::Failed, now);
            change.outcome("NO_EFFECT_PROVEN");
            db(set_cause(
                conn,
                T9N_FAIL_NO_EFFECT_PROVEN,
                Some(worker.attempt_id),
                None,
            ))?;
            db(write_job(conn, job.async_job_id, &change))?;
            return reread(conn, job.async_job_id);
        }

        if request.evidence.is_some() {
            return Err(evidence_invalid(
                "a failure without noEffectProven takes no evidence object",
            ));
        }

        let (status, result, outcome, cause) = if phase == AsyncEffectPhase::PreWrite {
            // T7 / T8. INDETERMINATE at PRE_WRITE is a retryable pre-write
            // failure: the durable phase proves no effect started.
            if request.class == FailureClass::Permanent {
                change.terminate(AsyncJobStatus::Failed, now);
                (
                    AsyncJobStatus::Failed,
                    AsyncAttemptResult::PermanentFailure,
                    "PERMANENT_FAILURE",
                    T8_FAIL,
                )
            } else {
                change.prewrite_failure_count += 1;
                let room = change.prewrite_failure_count < job.retry_budget
                    && job.execution_claim_count < job.max_execution_claims;
                if room {
                    change.status = AsyncJobStatus::RetryScheduled;
                    change.available_at = db(backoff_until(
                        conn,
                        job,
                        i32::from(change.prewrite_failure_count),
                        now,
                    ))?;
                    (
                        AsyncJobStatus::RetryScheduled,
                        AsyncAttemptResult::RetryableFailure,
                        "RETRY_SCHEDULED",
                        T7_FAIL_RETRYABLE,
                    )
                } else {
                    change.terminate(AsyncJobStatus::Failed, now);
                    let code = if change.prewrite_failure_count >= job.retry_budget {
                        "RETRY_BUDGET_EXHAUSTED"
                    } else {
                        "ATTEMPT_CAP"
                    };
                    (
                        AsyncJobStatus::Failed,
                        AsyncAttemptResult::RetryableFailure,
                        code,
                        T8_FAIL,
                    )
                }
            }
        } else {
            // T9: after effect start any failure needs reconciliation.
            change.require_reconciliation(now);
            (
                AsyncJobStatus::ReconciliationRequired,
                request.class.attempt_result(),
                "RECONCILIATION_REQUIRED",
                T9_FAIL_AFTER_EFFECT_START,
            )
        };
        if status != AsyncJobStatus::ReconciliationRequired {
            db(targets::lock_holds_waiting(conn, &job.target_keys))?;
            db(targets::delete_holds(conn, job.async_job_id))?;
        }
        let mut close = AttemptClose::new(result, status, outcome);
        close.error_code = Some(&request.error_code);
        close.error_detail = error_detail.as_deref();
        db(close_attempt(conn, worker.attempt_id, now, &close))?;
        change.outcome(outcome);
        db(set_cause(conn, cause, Some(worker.attempt_id), None))?;
        db(write_job(conn, job.async_job_id, &change))?;
        reread(conn, job.async_job_id)
    })
}

/// T13: yield a pre-write claim. `PENDING` while execution claims are left,
/// otherwise `FAILED` (`ATTEMPT_CAP`); the pre-write budget is unchanged.
pub fn release_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
) -> ThothResult<AsyncJob> {
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Execution)?;
        let (job, now) = (&worker.job, worker.now);
        if job.effect_phase != AsyncEffectPhase::PreWrite {
            return Err(refused("PHASE"));
        }
        let mut change = JobChange::from_row(job);
        let (status, outcome) = if job.execution_claim_count < job.max_execution_claims {
            change.clear_claim();
            change.status = AsyncJobStatus::Pending;
            change.available_at = now;
            (AsyncJobStatus::Pending, "RELEASED")
        } else {
            change.terminate(AsyncJobStatus::Failed, now);
            (AsyncJobStatus::Failed, "ATTEMPT_CAP")
        };
        db(targets::lock_holds_waiting(conn, &job.target_keys))?;
        db(targets::delete_holds(conn, job.async_job_id))?;
        db(close_attempt(
            conn,
            worker.attempt_id,
            now,
            &AttemptClose::new(AsyncAttemptResult::Released, status, outcome),
        ))?;
        change.outcome(outcome);
        db(set_cause(conn, T13_RELEASE, Some(worker.attempt_id), None))?;
        db(write_job(conn, job.async_job_id, &change))?;
        reread(conn, job.async_job_id)
    })
}

/// The applied state of the job's targets, for a handler to check before
/// any write (section 13.2). Needs a current token (execution or
/// reconciliation).
pub fn read_target_state_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
) -> ThothResult<Vec<TargetState>> {
    run_op(conn, |conn| {
        let worker = worker_prelude(conn, registry, claim, Need::Either)?;
        db(targets::read_states(conn, worker.job.async_job_id))
    })
}

// ---------------------------------------------------------------------------
// Reconciliation resolution (T16)
// ---------------------------------------------------------------------------

/// The outcomes an executor reconciler may record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcilerOutcome {
    Succeeded,
    Failed,
    Cancelled,
    Waiting,
    StillRequired,
}

/// The input of [`resolve_reconciliation_on`]. Which of `entries` and
/// `evidence` each outcome takes is section 13.3's evidence-argument table;
/// `WAITING` takes `checkpoint` and `resume_at` instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveReconciliation {
    pub outcome: ReconcilerOutcome,
    pub entries: Option<Vec<EvidenceEntry>>,
    pub evidence: Option<String>,
    pub checkpoint: Option<String>,
    pub resume_at: Option<DateTime<Utc>>,
}

/// T16: resolve a claimed reconciliation.
pub fn resolve_reconciliation_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    claim: &WorkerClaim,
    request: ResolveReconciliation,
) -> ThothResult<AsyncJob> {
    let waiting = request.outcome == ReconcilerOutcome::Waiting;
    if waiting {
        let Some(checkpoint) = &request.checkpoint else {
            return Err(invalid("WAITING needs a checkpoint"));
        };
        parse_json_object(checkpoint, true)
            .map_err(|fault| invalid(fault.describe(JsonInput::Checkpoint)))?;
        if request.resume_at.is_none() {
            return Err(invalid("WAITING needs resume_at"));
        }
    } else if request.checkpoint.is_some() || request.resume_at.is_some() {
        return Err(invalid("only WAITING takes a checkpoint and resume_at"));
    }
    run_op(conn, |conn| {
        if let (true, Some(checkpoint)) = (waiting, &request.checkpoint) {
            measure_json(conn, checkpoint, JsonInput::Checkpoint)?
                .map_err(|fault| invalid(fault.describe(JsonInput::Checkpoint)))?;
        }
        let worker = worker_prelude(conn, registry, claim, Need::Reconciliation)?;
        let (job, now, policy) = (&worker.job, worker.now, worker.policy);
        let confirmed = job.effect_phase == AsyncEffectPhase::EffectConfirmed;
        match request.outcome {
            ReconcilerOutcome::Failed | ReconcilerOutcome::Cancelled if confirmed => {
                return Err(refused("EFFECT_CONFIRMED"));
            }
            ReconcilerOutcome::Waiting
                if job.waiting_deadline.is_some_and(|deadline| deadline <= now) =>
            {
                return Err(refused("DEADLINE_EXCEEDED"));
            }
            _ => {}
        }
        let mut change = JobChange::from_row(job);
        change.clear_claim();
        match request.outcome {
            ReconcilerOutcome::Succeeded => {
                let Some(entries) = &request.entries else {
                    return Err(evidence_invalid("SUCCEEDED requires entries"));
                };
                check_entry_shapes(entries, EntryRule::EffectApplied, job.revision_mode)
                    .map_err(evidence_invalid)?;
                let payload = stored_payload(job)?;
                let context = evidence_context(job, &payload);
                let evidence = match &request.evidence {
                    Some(text) => Some(evidence_shape(conn, text)?.map_err(evidence_invalid)?),
                    None => {
                        if (policy.validate_evidence)(
                            EvidenceUse::Applied,
                            &context,
                            &serde_json::Value::Null,
                        )
                        .is_err()
                        {
                            return Err(evidence_invalid("the kind requires an evidence object"));
                        }
                        None
                    }
                };
                db(targets::lock_holds_waiting(conn, &job.target_keys))?;
                let states = db(targets::lock_state_rows(conn, &job.target_keys))?;
                // An executor reconciler never accepts a regression.
                let writes =
                    check_entries_semantic(entries, EntryRule::EffectApplied, job, &states)
                        .map_err(|fault| evidence_invalid(semantic_message(&fault)))?;
                if let Some(value) = &evidence {
                    (policy.validate_evidence)(EvidenceUse::Applied, &context, value)
                        .map_err(|error| evidence_invalid(error.0))?;
                }
                db(targets::write_states(
                    conn,
                    &writes,
                    job.async_job_id,
                    worker.attempt_id,
                ))?;
                db(targets::delete_holds(conn, job.async_job_id))?;
                let mut close = AttemptClose::new(
                    AsyncAttemptResult::Succeeded,
                    AsyncJobStatus::Succeeded,
                    "EFFECT_APPLIED",
                );
                close.evidence = request.evidence.as_deref();
                db(close_attempt(conn, worker.attempt_id, now, &close))?;
                change.terminate(AsyncJobStatus::Succeeded, now);
                change.outcome("EFFECT_APPLIED");
            }
            ReconcilerOutcome::Failed | ReconcilerOutcome::Cancelled => {
                if request.entries.is_some() {
                    return Err(evidence_invalid("this outcome takes no entries"));
                }
                let Some(text) = request.evidence.as_deref() else {
                    return Err(evidence_invalid(
                        "this outcome requires a no-effect evidence object",
                    ));
                };
                let value = evidence_shape(conn, text)?.map_err(evidence_invalid)?;
                let payload = stored_payload(job)?;
                (policy.validate_evidence)(
                    EvidenceUse::NoEffect,
                    &evidence_context(job, &payload),
                    &value,
                )
                .map_err(|error| evidence_invalid(error.0))?;
                let (status, result) = if request.outcome == ReconcilerOutcome::Failed {
                    (AsyncJobStatus::Failed, AsyncAttemptResult::PermanentFailure)
                } else {
                    (AsyncJobStatus::Cancelled, AsyncAttemptResult::Cancelled)
                };
                db(targets::lock_holds_waiting(conn, &job.target_keys))?;
                db(targets::delete_holds(conn, job.async_job_id))?;
                let mut close = AttemptClose::new(result, status, "NO_EFFECT_PROVEN");
                close.evidence = Some(text);
                db(close_attempt(conn, worker.attempt_id, now, &close))?;
                change.terminate(status, now);
                change.outcome("NO_EFFECT_PROVEN");
            }
            ReconcilerOutcome::Waiting | ReconcilerOutcome::StillRequired => {
                if request.entries.is_some() || request.evidence.is_some() {
                    return Err(evidence_invalid(
                        "this outcome takes no entries and no evidence object",
                    ));
                }
                if waiting {
                    let resume_at = request.resume_at.expect("checked above");
                    let earliest =
                        now + Duration::seconds(i64::from(policy.poll_interval_min_seconds));
                    let latest = job
                        .waiting_deadline
                        .unwrap_or(now + Duration::seconds(policy.waiting_horizon_seconds));
                    if resume_at < earliest || resume_at > latest {
                        return Err(invalid(
                            "resume_at must be at least the poll interval away and not after \
                             the waiting deadline",
                        ));
                    }
                    db(close_attempt(
                        conn,
                        worker.attempt_id,
                        now,
                        &AttemptClose::new(
                            AsyncAttemptResult::Waiting,
                            AsyncJobStatus::Waiting,
                            "WAITING",
                        ),
                    ))?;
                    change.status = AsyncJobStatus::Waiting;
                    change.checkpoint =
                        CheckpointWrite::Set(request.checkpoint.clone().expect("checked above"));
                    change.checkpoint_version += 1;
                    change.available_at = resume_at;
                    change.waiting_deadline = job.waiting_deadline.or(Some(
                        now + Duration::seconds(policy.waiting_horizon_seconds),
                    ));
                    change.outcome("WAITING");
                } else {
                    db(close_attempt(
                        conn,
                        worker.attempt_id,
                        now,
                        &AttemptClose::new(
                            AsyncAttemptResult::StillRequired,
                            AsyncJobStatus::ReconciliationRequired,
                            "STILL_REQUIRED",
                        ),
                    ))?;
                    change.available_at = db(backoff_until(
                        conn,
                        job,
                        i32::from(job.reconciliation_attempt_count),
                        now,
                    ))?;
                    change.outcome("STILL_REQUIRED");
                }
            }
        }
        db(set_cause(conn, T16_RESOLVE, Some(worker.attempt_id), None))?;
        db(write_job(conn, job.async_job_id, &change))?;
        reread(conn, job.async_job_id)
    })
}

// ---------------------------------------------------------------------------
// Recovery (section 10.6; T11, T12 and T17)
// ---------------------------------------------------------------------------

/// The rows one recovery call recovered, deferred (a hold lock was taken)
/// and failed (rolled back to their savepoint and recorded).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub recovered: Vec<Uuid>,
    pub deferred: Vec<Uuid>,
    pub failed: Vec<Uuid>,
}

/// Apply T11, T12 and T17 to at most `limit` (1 to 1000) due rows of the
/// given kinds (all kinds when `None`).
pub fn recover_on(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    kinds: Option<&[String]>,
    limit: i64,
) -> ThothResult<RecoveryReport> {
    if !(1..=RECOVERY_LIMIT_MAX).contains(&limit) {
        return Err(invalid("limit must be between 1 and 1000"));
    }
    if let Some(kinds) = kinds {
        for kind in kinds {
            require_no_nul(kind, "kind")?;
        }
    }
    let kinds: Option<Vec<String>> = kinds.map(<[String]>::to_vec);
    run_op(conn, |conn| {
        recover_rows(conn, registry, kinds.as_deref(), limit)
    })
}

/// Recovery inside an operation that already ran `begin_op` (claims call it
/// after their kind locks).
pub(crate) fn recover_rows(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    kinds: Option<&[String]>,
    limit: i64,
) -> ThothResult<RecoveryReport> {
    // One statement locks the due rows: no current failure entry first, then
    // by the entry's last failure, then by due time and id.
    let rows = db(sql_query(format!(
        "SELECT {JOB_COLUMNS} FROM public.async_job j \
           LEFT JOIN public.async_recovery_failure f ON f.async_job_id = j.async_job_id \
          WHERE ($1::text[] IS NULL OR j.kind = ANY($1::text[])) \
            AND ((j.claim_token IS NOT NULL AND j.lease_expires_at <= clock_timestamp()) \
                 OR (j.status = 'WAITING' AND j.waiting_deadline <= clock_timestamp())) \
          ORDER BY (f.last_failed_at >= COALESCE(j.lease_expires_at, j.waiting_deadline)) IS TRUE, \
                   CASE WHEN f.last_failed_at >= COALESCE(j.lease_expires_at, j.waiting_deadline) \
                        THEN f.last_failed_at END, \
                   COALESCE(j.lease_expires_at, j.waiting_deadline), \
                   j.async_job_id \
          LIMIT $2 \
          FOR UPDATE OF j SKIP LOCKED"
    ))
    .bind::<Nullable<Array<Text>>, _>(kinds)
    .bind::<BigInt, _>(limit)
    .load::<JobRow>(conn))?;
    let now = db(clock_now(conn))?;
    let mut report = RecoveryReport::default();
    for job in rows {
        let policy = registry.get(&job.kind);
        let step = conn.transaction(|conn| -> Result<(), StepError> {
            recover_one(conn, policy, &job, now)?;
            force_hold_and_attempt_checks(conn)?;
            delete_recovery_failure(conn, job.async_job_id)?;
            Ok(())
        });
        match step {
            Ok(()) => report.recovered.push(job.async_job_id),
            Err(StepError::Skip) => report.deferred.push(job.async_job_id),
            Err(StepError::Db(error)) if is_connection_lost(&error) => {
                return Err(ThothError::AsyncJobBusy);
            }
            Err(StepError::Db(error)) => {
                log_step_failure("recovery failed for job", job.async_job_id, &error);
                let code = error_code_of(&map_db_error(error));
                db(upsert_recovery_failure(conn, job.async_job_id, now, code))?;
                report.failed.push(job.async_job_id);
            }
        }
    }
    Ok(report)
}

fn delete_recovery_failure(conn: &mut PgConnection, job_id: Uuid) -> QueryResult<()> {
    sql_query("DELETE FROM public.async_recovery_failure WHERE async_job_id = $1")
        .bind::<SqlUuid, _>(job_id)
        .execute(conn)
        .map(|_| ())
}

fn upsert_recovery_failure(
    conn: &mut PgConnection,
    job_id: Uuid,
    now: DateTime<Utc>,
    code: &str,
) -> QueryResult<()> {
    sql_query(
        "INSERT INTO public.async_recovery_failure \
             (async_job_id, first_failed_at, last_failed_at, failure_count, error_code) \
         VALUES ($1, $2, $2, 1, $3) \
         ON CONFLICT (async_job_id) DO UPDATE SET \
             failure_count = public.async_recovery_failure.failure_count + 1, \
             last_failed_at = EXCLUDED.last_failed_at, \
             error_code = EXCLUDED.error_code",
    )
    .bind::<SqlUuid, _>(job_id)
    .bind::<Timestamptz, _>(now)
    .bind::<Text, _>(code)
    .execute(conn)
    .map(|_| ())
}

/// One due row. Kinds unknown to the registry recover from the job's own
/// snapshot and are never treated as resumable.
fn recover_one(
    conn: &mut PgConnection,
    policy: Option<&'static AsyncJobKindPolicy>,
    job: &JobRow,
    now: DateTime<Utc>,
) -> Result<(), StepError> {
    let lease_due = job.claim_token.is_some() && job.lease_expires_at.is_some_and(|l| l <= now);
    match job.status {
        AsyncJobStatus::Running if lease_due => recover_lease(conn, policy, job, now),
        AsyncJobStatus::ReconciliationRequired if lease_due => {
            apply_reconciliation_lease_expiry(conn, job, now)?;
            Ok(())
        }
        AsyncJobStatus::Waiting if job.waiting_deadline.is_some_and(|d| d <= now) => {
            let mut change = JobChange::from_row(job);
            if job.effect_phase >= AsyncEffectPhase::EffectStarted {
                change.require_reconciliation(now);
            } else {
                if !targets::try_lock_holds(conn, &job.target_keys)? {
                    return Err(StepError::Skip);
                }
                targets::delete_holds(conn, job.async_job_id)?;
                change.terminate(AsyncJobStatus::Failed, now);
            }
            change.outcome("DEADLINE_EXCEEDED");
            set_cause(conn, T11_WAITING_DEADLINE_EXCEEDED, None, None)?;
            write_job(conn, job.async_job_id, &change)?;
            Ok(())
        }
        _ => Err(StepError::Skip),
    }
}

/// T12: an expired execution lease.
fn recover_lease(
    conn: &mut PgConnection,
    policy: Option<&'static AsyncJobKindPolicy>,
    job: &JobRow,
    now: DateTime<Utc>,
) -> Result<(), StepError> {
    let attempt_id = claimed_attempt_id(conn, job)?;
    let mut change = JobChange::from_row(job);
    let mut release = false;
    let outcome = match job.effect_phase {
        AsyncEffectPhase::PreWrite => {
            change.prewrite_failure_count += 1;
            release = true;
            let room = change.prewrite_failure_count < job.retry_budget
                && job.execution_claim_count < job.max_execution_claims;
            if room {
                change.clear_claim();
                change.status = AsyncJobStatus::RetryScheduled;
                change.available_at =
                    backoff_until(conn, job, i32::from(change.prewrite_failure_count), now)?;
                "LEASE_EXPIRED"
            } else {
                change.terminate(AsyncJobStatus::Failed, now);
                if change.prewrite_failure_count >= job.retry_budget {
                    "RETRY_BUDGET_EXHAUSTED"
                } else {
                    "ATTEMPT_CAP"
                }
            }
        }
        AsyncEffectPhase::EffectStarted => {
            change.require_reconciliation(now);
            "LEASE_EXPIRED"
        }
        AsyncEffectPhase::EffectConfirmed => {
            let resumable =
                policy.is_some_and(|p| p.confirmed_effect_resumable) && job.checkpoint.is_some();
            let deadline = job
                .waiting_deadline
                .or_else(|| policy.map(|p| now + Duration::seconds(p.waiting_horizon_seconds)));
            match (resumable, deadline, policy) {
                (true, Some(deadline), Some(policy)) if deadline > now => {
                    change.clear_claim();
                    change.status = AsyncJobStatus::Waiting;
                    change.available_at =
                        now + Duration::seconds(i64::from(policy.poll_interval_min_seconds));
                    change.waiting_deadline = Some(deadline);
                }
                _ => change.require_reconciliation(now),
            }
            "LEASE_EXPIRED"
        }
    };
    if release {
        if !targets::try_lock_holds(conn, &job.target_keys)? {
            return Err(StepError::Skip);
        }
        targets::delete_holds(conn, job.async_job_id)?;
    }
    close_attempt(
        conn,
        attempt_id,
        now,
        &AttemptClose::new(AsyncAttemptResult::LeaseExpired, change.status, outcome),
    )?;
    change.outcome(outcome);
    set_cause(conn, T12_LEASE_EXPIRED, Some(attempt_id), None)?;
    write_job(conn, job.async_job_id, &change)?;
    Ok(())
}

/// T17: an expired reconciliation lease. The job stays RR, unclaimed, and
/// becomes claimable again after the reconciliation backoff. Status
/// preserving: recorded on the attempt, not in the transition log.
pub(crate) fn apply_reconciliation_lease_expiry(
    conn: &mut PgConnection,
    job: &JobRow,
    now: DateTime<Utc>,
) -> QueryResult<()> {
    let attempt_id = claimed_attempt_id(conn, job)?;
    close_attempt(
        conn,
        attempt_id,
        now,
        &AttemptClose::new(
            AsyncAttemptResult::LeaseExpired,
            AsyncJobStatus::ReconciliationRequired,
            "LEASE_EXPIRED",
        ),
    )?;
    let mut change = JobChange::from_row(job);
    change.clear_claim();
    change.available_at =
        backoff_until(conn, job, i32::from(job.reconciliation_attempt_count), now)?;
    change.outcome("LEASE_EXPIRED");
    write_job(conn, job.async_job_id, &change)?;
    delete_recovery_failure(conn, job.async_job_id)
}
