//! The shared PostgreSQL job engine of THOTH-ASYNC-01 (ADR-0012, Release N,
//! task THOTH-ASYNC-01-ENGINE-01A).
//!
//! The engine owns durable jobs, their attempts, target holds, applied
//! evidence, operator commands and the transition log. It ships **inert**: no
//! production path creates a job and the production kind registry
//! ([`registry::PRODUCTION_REGISTRY`]) is empty.
//!
//! Write functions end in `_on`, take `&mut PgConnection` and run inside the
//! caller's READ COMMITTED transaction (the BE-04 `_on` pattern). Every `_on`
//! function checks that it runs inside an open READ COMMITTED transaction,
//! validates every bounded input before any write, defers the engine's three
//! constraint triggers while it works and forces them before it returns, and
//! resets the `thoth.async_*` transaction settings it uses for the transition
//! log. [`with_engine_transaction`] is the thin wrapper that opens its own READ
//! COMMITTED transaction with `SET LOCAL lock_timeout = '5s'` and maps a failed
//! commit.
//!
//! Lock order (spec section 11): per-kind advisory locks (claims only), then
//! job rows, then inserts that reference only held or created rows, then hold
//! advisory locks under rule H, hold inserts, applied-state rows ascending by
//! target key, hold deletes, attempt, transition and command writes, and poll
//! upserts. Claims and recovery only ever try the hold locks; every other path
//! takes them in the waiting form after its job row lock.
//!
//! No read returns a claim token, and no engine path logs a payload,
//! checkpoint, evidence, error detail, claim token or environment value.

use std::sync::LazyLock;

use chrono::{DateTime, Duration, Utc};
use diesel::connection::{AnsiTransactionManager, SimpleConnection, TransactionManager};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::{
    Array, BigInt, Bool, Int2, Int4, Int8, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::{sql_query, QueryableByName};
use regex::Regex;
use serde::{Deserialize, Serialize};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;

pub mod commands;
pub mod engine;
pub mod registry;
pub mod signals;
pub mod targets;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use commands::{
    cancel_job_command_on, redact_job_diagnostics_command_on, resolve_reconciliation_command_on,
    retry_job_command_on, CommandResult, StaffActor, StaffResolution,
};
pub use engine::{
    claim_on, claim_reconciliation_on, complete_on, create_job_on, fail_on,
    mark_effect_confirmed_on, mark_effect_started_on, read_target_state_on, recover_on, release_on,
    renew_on, resolve_reconciliation_on, wait_on, ClaimRequest, ClaimResult, ClaimedJob,
    CompleteOutcome, CompleteRequest, CompletionOutcome, CreateOutcome, Executor, FailRequest,
    FailureClass, LeaseRenewal, NewAsyncJob, ReconcilerOutcome, RecoveryReport,
    ResolveReconciliation, WaitOutcome, WaitRequest, WorkerClaim,
};
pub use registry::{
    AsyncJobKindPolicy, AsyncJobRegistry, CancelPolicy, EvidenceContext, EvidenceUse,
    ExecutorDomain, PayloadError, RiskClass, StaffOutcome, PRODUCTION_REGISTRY,
};
pub use signals::{
    executor_activity, get_job, job_attempts, job_transitions, kind_signals, list_jobs,
    AsyncJobFilter, BlockedJob, ExecutorActivity, JobCursor, KindSignals, RegressedTarget,
};
pub use targets::{EvidenceEntry, TargetState};

// ---------------------------------------------------------------------------
// Advisory lock namespaces (two-key form, first key). Release R reserves a
// fourth namespace for routing; it is not used in Release N.
// ---------------------------------------------------------------------------

/// Per-kind execution claim lock: `pg_advisory_xact_lock(NS_KIND, hashtext(kind))`.
pub(crate) const NS_KIND: i32 = 1095981390;
/// Per-kind reconciliation claim lock.
pub(crate) const NS_RECON: i32 = 1095981391;
/// Per-target hold lock (rule H): `(NS_HOLD, hashtext(target_key))`.
pub(crate) const NS_HOLD: i32 = 1095981392;

// ---------------------------------------------------------------------------
// Bounds (spec sections 7, 10 and 16)
// ---------------------------------------------------------------------------

pub(crate) const PAYLOAD_MAX_BYTES: i32 = 16384;
pub(crate) const CHECKPOINT_MAX_BYTES: i32 = 65536;
pub(crate) const EVIDENCE_MAX_BYTES: i32 = 16384;
pub(crate) const IDENTIFIER_MAX_CHARS: usize = 256;
pub(crate) const CODE_MAX_CHARS: usize = 64;
pub(crate) const CREATED_BY_MAX_CHARS: usize = 256;
pub(crate) const CLAIMED_BY_MAX_CHARS: usize = 128;
pub(crate) const PRINCIPAL_MAX_CHARS: usize = 256;
pub(crate) const REASON_MAX_CHARS: usize = 1000;
pub(crate) const ERROR_DETAIL_MAX_CHARS: usize = 2048;
pub(crate) const PROVIDER_REF_MAX_CHARS: usize = 512;
pub(crate) const EFFECT_IDENTITY_MAX_CHARS: usize = 400;
pub(crate) const TARGET_KEY_VALUE_MAX_CHARS: usize = 400;
pub(crate) const MAX_TARGETS: usize = 32;
pub(crate) const PRIORITY_MIN: i16 = -100;
pub(crate) const PRIORITY_MAX: i16 = 100;
pub(crate) const CLAIM_MAX_JOBS: i32 = 50;
pub(crate) const CLAIM_RECOVERY_LIMIT: i64 = 50;
pub(crate) const RECOVERY_LIMIT_MAX: i64 = 1000;
pub(crate) const LIST_LIMIT_MAX: i64 = 100;
/// The executor name recorded on a staff reconciliation attempt.
pub(crate) const STAFF_ATTEMPT_LABEL: &str = "staff";
/// `lock_timeout` of an engine-owned transaction.
pub(crate) const ENGINE_LOCK_TIMEOUT: &str = "5s";

// ---------------------------------------------------------------------------
// Exact server messages recognised as `ASYNC_JOB_BUSY` (section 10). Diesel
// 2.3 exposes no SQLSTATE; these assume an English `lc_messages`, and a test
// fails loudly otherwise.
// ---------------------------------------------------------------------------

pub(crate) const LOCK_TIMEOUT_TEXT: &str = "canceling statement due to lock timeout";
pub(crate) const STATEMENT_TIMEOUT_TEXT: &str = "canceling statement due to statement timeout";
pub(crate) const DEADLOCK_TEXT: &str = "deadlock detected";
pub(crate) const QUERY_CANCEL_TEXT: &str = "canceling statement due to user request";
pub(crate) const ADMIN_TERMINATION_TEXT: &str =
    "terminating connection due to administrator command";

/// The three deferred constraint triggers (section 7.10).
pub(crate) const ALL_DEFERRED_CHECKS: &str =
    "async_job_hold_consistency, async_job_attempt_consistency, async_control_command_decided";
pub(crate) const HOLD_AND_ATTEMPT_CHECKS: &str =
    "async_job_hold_consistency, async_job_attempt_consistency";

/// The credential key guard of section 16.
static CREDENTIAL_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(password|passwd|secret|token|api[_-]?key|credential|authorization|private[_-]?key|access[_-]?key)",
    )
    .expect("the credential key pattern is valid")
});

// ---------------------------------------------------------------------------
// Enum types (section 7.1)
// ---------------------------------------------------------------------------

macro_rules! async_enum {
    (
        $(#[$meta:meta])*
        $name:ident => $path:literal { $($variant:ident = $label:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            diesel_derive_enum::DbEnum,
        )]
        #[ExistingTypePath = $path]
        pub enum $name {
            $(
                #[db_rename = $label]
                #[serde(rename = $label)]
                $variant,
            )+
        }

        impl $name {
            /// The database label.
            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $label,)+
                }
            }
        }
    };
}

async_enum! {
    /// The lifecycle state of a job (section 9.1).
    AsyncJobStatus => "crate::schema::sql_types::AsyncJobStatus" {
        Pending = "PENDING",
        Running = "RUNNING",
        Waiting = "WAITING",
        RetryScheduled = "RETRY_SCHEDULED",
        Succeeded = "SUCCEEDED",
        Failed = "FAILED",
        Cancelled = "CANCELLED",
        ReconciliationRequired = "RECONCILIATION_REQUIRED",
    }
}

async_enum! {
    /// How far a job's external effect has progressed. Ordered: it only
    /// advances.
    AsyncEffectPhase => "crate::schema::sql_types::AsyncEffectPhase" {
        PreWrite = "PRE_WRITE",
        EffectStarted = "EFFECT_STARTED",
        EffectConfirmed = "EFFECT_CONFIRMED",
    }
}

async_enum! {
    /// Whether a kind writes an external effect.
    EffectClass => "crate::schema::sql_types::AsyncJobEffectClass" {
        ReadOnly = "READ_ONLY",
        Effectful = "EFFECTFUL",
    }
}

async_enum! {
    /// How a kind's applied evidence is ordered.
    RevisionMode => "crate::schema::sql_types::AsyncJobRevisionMode" {
        CurrentState = "CURRENT_STATE",
        RevisionBound = "REVISION_BOUND",
    }
}

async_enum! {
    /// Whether a job is original or a staff manual retry.
    AsyncJobLineage => "crate::schema::sql_types::AsyncJobLineage" {
        Original = "ORIGINAL",
        ManualRetry = "MANUAL_RETRY",
    }
}

async_enum! {
    /// What an attempt was opened for.
    AsyncAttemptPurpose => "crate::schema::sql_types::AsyncAttemptPurpose" {
        Execution = "EXECUTION",
        Reconciliation = "RECONCILIATION",
        StaffReconciliation = "STAFF_RECONCILIATION",
    }
}

async_enum! {
    /// How an attempt closed.
    AsyncAttemptResult => "crate::schema::sql_types::AsyncAttemptResult" {
        Succeeded = "SUCCEEDED",
        RetryableFailure = "RETRYABLE_FAILURE",
        PermanentFailure = "PERMANENT_FAILURE",
        Indeterminate = "INDETERMINATE",
        Waiting = "WAITING",
        Released = "RELEASED",
        LeaseExpired = "LEASE_EXPIRED",
        Cancelled = "CANCELLED",
        StillRequired = "STILL_REQUIRED",
    }
}

async_enum! {
    /// The four operator commands.
    AsyncControlCommandKind => "crate::schema::sql_types::AsyncControlCommandKind" {
        CancelJob = "CANCEL_JOB",
        RetryJob = "RETRY_JOB",
        ResolveReconciliation = "RESOLVE_RECONCILIATION",
        RedactJobDiagnostics = "REDACT_JOB_DIAGNOSTICS",
    }
}

async_enum! {
    /// A command's recorded decision.
    AsyncControlCommandOutcome => "crate::schema::sql_types::AsyncControlCommandOutcome" {
        Applied = "APPLIED",
        Refused = "REFUSED",
    }
}

impl AsyncJobStatus {
    /// `SUCCEEDED`, `FAILED` or `CANCELLED`.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            AsyncJobStatus::Succeeded | AsyncJobStatus::Failed | AsyncJobStatus::Cancelled
        )
    }
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

/// The column list every job read selects. JSON columns are returned as the
/// stored `jsonb` text, never re-serialised.
pub(crate) const JOB_COLUMNS: &str = "j.async_job_id, j.kind, j.payload_version, \
     j.payload::text AS payload, j.idempotency_key, j.effect_class, j.revision_mode, \
     j.retry_budget, j.max_execution_claims, j.max_reconciliation_attempts, \
     j.backoff_base_seconds, j.backoff_max_seconds, j.source_order, j.source_fingerprint, \
     j.source_ref, j.status, j.priority, j.available_at, j.effect_phase, j.attempt_count, \
     j.execution_claim_count, j.prewrite_failure_count, j.reconciliation_attempt_count, \
     j.claim_token, j.claimed_by, j.executor_principal, j.claimed_at, j.lease_expires_at, \
     j.attempt_runtime_deadline, j.first_claimed_at, j.waiting_deadline, \
     j.checkpoint::text AS checkpoint, j.checkpoint_version, j.provider_ref, j.terminal_at, \
     j.last_outcome_code, j.redacted_at, j.lineage, j.parent_job_id, j.correlation_id, \
     j.causation_ref, j.created_by, j.created_at, j.updated_at, \
     ARRAY(SELECT t.target_key FROM public.async_job_target t \
           WHERE t.async_job_id = j.async_job_id ORDER BY t.target_key) AS target_keys";

/// One job row as the engine reads it under its row lock. Internal: it carries
/// the claim token, which no read returns.
#[derive(Debug, Clone, PartialEq, QueryableByName)]
pub(crate) struct JobRow {
    #[diesel(sql_type = SqlUuid)]
    pub async_job_id: Uuid,
    #[diesel(sql_type = Text)]
    pub kind: String,
    #[diesel(sql_type = Int2)]
    pub payload_version: i16,
    #[diesel(sql_type = Text)]
    pub payload: String,
    #[diesel(sql_type = Text)]
    pub idempotency_key: String,
    #[diesel(sql_type = crate::schema::sql_types::AsyncJobEffectClass)]
    pub effect_class: EffectClass,
    #[diesel(sql_type = crate::schema::sql_types::AsyncJobRevisionMode)]
    pub revision_mode: RevisionMode,
    #[diesel(sql_type = Int2)]
    pub retry_budget: i16,
    #[diesel(sql_type = Int2)]
    pub max_execution_claims: i16,
    #[diesel(sql_type = Int2)]
    pub max_reconciliation_attempts: i16,
    #[diesel(sql_type = Int4)]
    pub backoff_base_seconds: i32,
    #[diesel(sql_type = Int4)]
    pub backoff_max_seconds: i32,
    #[diesel(sql_type = Nullable<Int8>)]
    pub source_order: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    pub source_fingerprint: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub source_ref: Option<String>,
    #[diesel(sql_type = crate::schema::sql_types::AsyncJobStatus)]
    pub status: AsyncJobStatus,
    #[diesel(sql_type = Int2)]
    pub priority: i16,
    #[diesel(sql_type = Timestamptz)]
    pub available_at: DateTime<Utc>,
    #[diesel(sql_type = crate::schema::sql_types::AsyncEffectPhase)]
    pub effect_phase: AsyncEffectPhase,
    #[diesel(sql_type = Int4)]
    pub attempt_count: i32,
    #[diesel(sql_type = Int2)]
    pub execution_claim_count: i16,
    #[diesel(sql_type = Int2)]
    pub prewrite_failure_count: i16,
    #[diesel(sql_type = Int2)]
    pub reconciliation_attempt_count: i16,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub claim_token: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    pub claimed_by: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub executor_principal: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub claimed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub lease_expires_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub attempt_runtime_deadline: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub first_claimed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub waiting_deadline: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    pub checkpoint: Option<String>,
    #[diesel(sql_type = Int4)]
    pub checkpoint_version: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub provider_ref: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub terminal_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_outcome_code: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub redacted_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = crate::schema::sql_types::AsyncJobLineage)]
    pub lineage: AsyncJobLineage,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub parent_job_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub correlation_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    pub causation_ref: Option<String>,
    #[diesel(sql_type = Text)]
    pub created_by: String,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
    #[diesel(sql_type = Array<Text>)]
    pub target_keys: Vec<String>,
}

/// A job as every read returns it: every column except the claim token (S4).
/// `payload` and `checkpoint` are the stored `jsonb` text.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AsyncJob {
    pub async_job_id: Uuid,
    pub kind: String,
    pub payload_version: i16,
    pub payload: String,
    pub idempotency_key: String,
    pub effect_class: EffectClass,
    pub revision_mode: RevisionMode,
    pub retry_budget: i16,
    pub max_execution_claims: i16,
    pub max_reconciliation_attempts: i16,
    pub backoff_base_seconds: i32,
    pub backoff_max_seconds: i32,
    pub source_order: Option<i64>,
    pub source_fingerprint: Option<String>,
    pub source_ref: Option<String>,
    pub status: AsyncJobStatus,
    pub priority: i16,
    pub available_at: DateTime<Utc>,
    pub effect_phase: AsyncEffectPhase,
    pub attempt_count: i32,
    pub execution_claim_count: i16,
    pub prewrite_failure_count: i16,
    pub reconciliation_attempt_count: i16,
    pub claimed_by: Option<String>,
    pub executor_principal: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub attempt_runtime_deadline: Option<DateTime<Utc>>,
    pub first_claimed_at: Option<DateTime<Utc>>,
    pub waiting_deadline: Option<DateTime<Utc>>,
    pub checkpoint: Option<String>,
    pub checkpoint_version: i32,
    pub provider_ref: Option<String>,
    pub terminal_at: Option<DateTime<Utc>>,
    pub last_outcome_code: Option<String>,
    pub redacted_at: Option<DateTime<Utc>>,
    pub lineage: AsyncJobLineage,
    pub parent_job_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub causation_ref: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// The job's target keys, ascending.
    pub target_keys: Vec<String>,
}

impl From<JobRow> for AsyncJob {
    fn from(row: JobRow) -> Self {
        AsyncJob {
            async_job_id: row.async_job_id,
            kind: row.kind,
            payload_version: row.payload_version,
            payload: row.payload,
            idempotency_key: row.idempotency_key,
            effect_class: row.effect_class,
            revision_mode: row.revision_mode,
            retry_budget: row.retry_budget,
            max_execution_claims: row.max_execution_claims,
            max_reconciliation_attempts: row.max_reconciliation_attempts,
            backoff_base_seconds: row.backoff_base_seconds,
            backoff_max_seconds: row.backoff_max_seconds,
            source_order: row.source_order,
            source_fingerprint: row.source_fingerprint,
            source_ref: row.source_ref,
            status: row.status,
            priority: row.priority,
            available_at: row.available_at,
            effect_phase: row.effect_phase,
            attempt_count: row.attempt_count,
            execution_claim_count: row.execution_claim_count,
            prewrite_failure_count: row.prewrite_failure_count,
            reconciliation_attempt_count: row.reconciliation_attempt_count,
            claimed_by: row.claimed_by,
            executor_principal: row.executor_principal,
            claimed_at: row.claimed_at,
            lease_expires_at: row.lease_expires_at,
            attempt_runtime_deadline: row.attempt_runtime_deadline,
            first_claimed_at: row.first_claimed_at,
            waiting_deadline: row.waiting_deadline,
            checkpoint: row.checkpoint,
            checkpoint_version: row.checkpoint_version,
            provider_ref: row.provider_ref,
            terminal_at: row.terminal_at,
            last_outcome_code: row.last_outcome_code,
            redacted_at: row.redacted_at,
            lineage: row.lineage,
            parent_job_id: row.parent_job_id,
            correlation_id: row.correlation_id,
            causation_ref: row.causation_ref,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            target_keys: row.target_keys,
        }
    }
}

/// One attempt as reads return it: every column except the claim token.
#[derive(Debug, Clone, PartialEq, Serialize, QueryableByName)]
pub struct AsyncJobAttempt {
    #[diesel(sql_type = SqlUuid)]
    pub async_job_attempt_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    pub async_job_id: Uuid,
    #[diesel(sql_type = Int4)]
    pub attempt_number: i32,
    #[diesel(sql_type = crate::schema::sql_types::AsyncAttemptPurpose)]
    pub purpose: AsyncAttemptPurpose,
    #[diesel(sql_type = Text)]
    pub claimed_by: String,
    #[diesel(sql_type = Text)]
    pub executor_principal: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub control_command_id: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    pub started_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub last_heartbeat_at: DateTime<Utc>,
    #[diesel(sql_type = crate::schema::sql_types::AsyncEffectPhase)]
    pub phase_at_start: AsyncEffectPhase,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub effect_started_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub effect_confirmed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub finished_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::AsyncAttemptResult>)]
    pub result: Option<AsyncAttemptResult>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::AsyncJobStatus>)]
    pub job_status_after: Option<AsyncJobStatus>,
    #[diesel(sql_type = Nullable<Text>)]
    pub outcome_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub error_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub error_detail: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub provider_ref: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub evidence: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub redacted_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub redaction_command_id: Option<Uuid>,
}

/// The attempt column list reads select (no claim token).
pub(crate) const ATTEMPT_COLUMNS: &str = "a.async_job_attempt_id, a.async_job_id, \
     a.attempt_number, a.purpose, a.claimed_by, a.executor_principal, a.control_command_id, \
     a.started_at, a.last_heartbeat_at, a.phase_at_start, a.effect_started_at, \
     a.effect_confirmed_at, a.finished_at, a.result, a.job_status_after, a.outcome_code, \
     a.error_code, a.error_detail, a.provider_ref, a.evidence::text AS evidence, \
     a.redacted_at, a.redaction_command_id";

/// One transition log row.
#[derive(Debug, Clone, PartialEq, Serialize, QueryableByName)]
pub struct AsyncJobTransition {
    #[diesel(sql_type = BigInt)]
    pub async_job_transition_id: i64,
    #[diesel(sql_type = SqlUuid)]
    pub async_job_id: Uuid,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::AsyncJobStatus>)]
    pub from_status: Option<AsyncJobStatus>,
    #[diesel(sql_type = crate::schema::sql_types::AsyncJobStatus)]
    pub to_status: AsyncJobStatus,
    #[diesel(sql_type = Text)]
    pub cause: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub async_job_attempt_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub async_control_command_id: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    pub occurred_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Error mapping (section 10)
// ---------------------------------------------------------------------------

/// True when a database error is one of the `ASYNC_JOB_BUSY` class: a lock or
/// statement timeout, a deadlock, a query cancel, an administrator
/// termination, a closed connection or a failed send.
pub(crate) fn is_busy_error(error: &DieselError) -> bool {
    match error {
        DieselError::DatabaseError(kind, info) => {
            matches!(
                kind,
                DatabaseErrorKind::ClosedConnection | DatabaseErrorKind::UnableToSendCommand
            ) || {
                let message = info.message();
                message == LOCK_TIMEOUT_TEXT
                    || message == STATEMENT_TIMEOUT_TEXT
                    || message == DEADLOCK_TEXT
                    || message == QUERY_CANCEL_TEXT
                    || message.starts_with(ADMIN_TERMINATION_TEXT)
            }
        }
        DieselError::RollbackErrorOnCommit {
            rollback_error,
            commit_error,
        } => is_busy_error(commit_error) || is_busy_error(rollback_error),
        _ => false,
    }
}

/// True when the connection itself is gone, so nothing further can run on it.
pub(crate) fn is_connection_lost(error: &DieselError) -> bool {
    match error {
        DieselError::DatabaseError(kind, info) => {
            matches!(
                kind,
                DatabaseErrorKind::ClosedConnection | DatabaseErrorKind::UnableToSendCommand
            ) || info.message().starts_with(ADMIN_TERMINATION_TEXT)
        }
        DieselError::RollbackErrorOnCommit {
            rollback_error,
            commit_error,
        } => is_connection_lost(commit_error) || is_connection_lost(rollback_error),
        _ => false,
    }
}

/// Map any database error an engine function or wrapper meets: the busy class
/// to `ASYNC_JOB_BUSY`, everything else to `ASYNC_JOB_INTEGRITY_VIOLATION`
/// with its fixed message. The constraint name and the server's primary
/// message go to the server log; row content (the error's detail) never does.
pub(crate) fn map_db_error(error: DieselError) -> ThothError {
    if is_busy_error(&error) {
        return ThothError::AsyncJobBusy;
    }
    match &error {
        DieselError::DatabaseError(_, info) => log::error!(
            "async job engine: database error mapped to ASYNC_JOB_INTEGRITY_VIOLATION \
             (constraint: {}): {}",
            info.constraint_name().unwrap_or("none"),
            info.message()
        ),
        DieselError::RollbackErrorOnCommit { .. } => log::error!(
            "async job engine: commit and rollback both failed; mapped to \
             ASYNC_JOB_INTEGRITY_VIOLATION"
        ),
        other => log::error!(
            "async job engine: non-database error mapped to ASYNC_JOB_INTEGRITY_VIOLATION: {}",
            error_kind_name(other)
        ),
    }
    ThothError::AsyncJobIntegrityViolation
}

fn error_kind_name(error: &DieselError) -> &'static str {
    match error {
        DieselError::NotFound => "not found",
        DieselError::DeserializationError(_) => "deserialization",
        DieselError::SerializationError(_) => "serialization",
        DieselError::AlreadyInTransaction => "already in transaction",
        DieselError::NotInTransaction => "not in transaction",
        DieselError::BrokenTransactionManager => "broken transaction manager",
        DieselError::QueryBuilderError(_) => "query builder",
        _ => "other",
    }
}

/// The `error_code` a recovery-failure entry records for a mapped error.
pub(crate) fn error_code_of(error: &ThothError) -> &'static str {
    match error {
        ThothError::AsyncJobBusy => "ASYNC_JOB_BUSY",
        _ => "ASYNC_JOB_INTEGRITY_VIOLATION",
    }
}

/// `map_db_error` as a closure-friendly function.
pub(crate) fn db<T>(result: QueryResult<T>) -> ThothResult<T> {
    result.map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Session discipline of every `_on` function (sections 7.10, 7.8 and 10)
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
struct TextValue {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(QueryableByName)]
pub(crate) struct NowRow {
    #[diesel(sql_type = Timestamptz)]
    pub now: DateTime<Utc>,
}

/// Refuse with `ASYNC_JOB_ISOLATION_INVALID` unless the transaction runs READ
/// COMMITTED.
pub(crate) fn check_isolation(conn: &mut PgConnection) -> ThothResult<()> {
    let isolation = db(
        sql_query("SELECT current_setting('transaction_isolation') AS value")
            .get_result::<TextValue>(conn),
    )?;
    if isolation.value != "read committed" {
        return Err(ThothError::AsyncJobIsolationInvalid);
    }
    Ok(())
}

/// Refuse with `ASYNC_JOB_ISOLATION_INVALID` unless Diesel's transaction
/// manager has an open transaction on this connection.
pub(crate) fn check_in_transaction(conn: &mut PgConnection) -> ThothResult<()> {
    let open = matches!(
        AnsiTransactionManager::transaction_manager_status_mut(conn).transaction_depth(),
        Ok(Some(_))
    );
    if !open {
        return Err(ThothError::AsyncJobIsolationInvalid);
    }
    Ok(())
}

/// The start of every `_on` function: an open READ COMMITTED transaction, and
/// the three constraint triggers deferred so a caller's `SET CONSTRAINTS ALL
/// IMMEDIATE` cannot fire them mid-operation.
pub(crate) fn begin_op(conn: &mut PgConnection) -> ThothResult<()> {
    check_in_transaction(conn)?;
    check_isolation(conn)?;
    db(conn.batch_execute(&format!("SET CONSTRAINTS {ALL_DEFERRED_CHECKS} DEFERRED")))
}

/// Force the hold and attempt checks (end of each claim candidate, recovery
/// row and internal creation).
pub(crate) fn force_hold_and_attempt_checks(conn: &mut PgConnection) -> QueryResult<()> {
    conn.batch_execute(&format!(
        "SET CONSTRAINTS {HOLD_AND_ATTEMPT_CHECKS} IMMEDIATE; \
         SET CONSTRAINTS {HOLD_AND_ATTEMPT_CHECKS} DEFERRED"
    ))
}

/// The end of every `_on` function: force all three checks, then reset the
/// transition-log settings, so a violation surfaces inside the call and the
/// caller's later statements are never attributed to the engine.
pub(crate) fn finish_op(conn: &mut PgConnection) -> ThothResult<()> {
    db(conn.batch_execute(&format!(
        "SET CONSTRAINTS {ALL_DEFERRED_CHECKS} IMMEDIATE; \
         SET CONSTRAINTS {ALL_DEFERRED_CHECKS} DEFERRED"
    )))?;
    db(reset_cause(conn))
}

/// Run an `_on` body between [`begin_op`] and [`finish_op`]. When the body
/// fails, the settings are still reset if the transaction is usable.
pub(crate) fn run_op<T>(
    conn: &mut PgConnection,
    body: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    begin_op(conn)?;
    match body(conn) {
        Ok(value) => {
            finish_op(conn)?;
            Ok(value)
        }
        Err(error) => {
            // Best effort: an aborted transaction refuses the statement, and
            // its settings die with it anyway.
            let _ = reset_cause(conn);
            Err(error)
        }
    }
}

/// Attribute the next status changes to an engine cause, attempt and command.
pub(crate) fn set_cause(
    conn: &mut PgConnection,
    cause: &str,
    attempt_id: Option<Uuid>,
    command_id: Option<Uuid>,
) -> QueryResult<()> {
    sql_query(
        "SELECT set_config('thoth.async_cause', $1, true), \
                set_config('thoth.async_attempt_id', $2, true), \
                set_config('thoth.async_command_id', $3, true)",
    )
    .bind::<Text, _>(cause)
    .bind::<Text, _>(attempt_id.map(|id| id.to_string()).unwrap_or_default())
    .bind::<Text, _>(command_id.map(|id| id.to_string()).unwrap_or_default())
    .execute(conn)
    .map(|_| ())
}

pub(crate) fn reset_cause(conn: &mut PgConnection) -> QueryResult<()> {
    set_cause(conn, "", None, None)
}

/// `clock_timestamp()`, read in its own statement after the locks it follows.
pub(crate) fn clock_now(conn: &mut PgConnection) -> QueryResult<DateTime<Utc>> {
    sql_query("SELECT clock_timestamp() AS now")
        .get_result::<NowRow>(conn)
        .map(|row| row.now)
}

/// Run `f` inside a savepoint; the savepoint is rolled back when `f` fails.
pub(crate) fn in_savepoint<T, E>(
    conn: &mut PgConnection,
    f: impl FnOnce(&mut PgConnection) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<DieselError>,
{
    conn.transaction(f)
}

// ---------------------------------------------------------------------------
// JSON inputs (section 10): one text bound to the pre-check, digest and write
// ---------------------------------------------------------------------------

/// Which input a JSON text is, which fixes its byte limit and its answer to a
/// fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonInput {
    Payload,
    Checkpoint,
    Evidence,
}

impl JsonInput {
    pub(crate) fn max_bytes(self) -> i32 {
        match self {
            JsonInput::Payload => PAYLOAD_MAX_BYTES,
            JsonInput::Checkpoint => CHECKPOINT_MAX_BYTES,
            JsonInput::Evidence => EVIDENCE_MAX_BYTES,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            JsonInput::Payload => "payload",
            JsonInput::Checkpoint => "checkpoint",
            JsonInput::Evidence => "evidence",
        }
    }
}

/// Why a JSON input was refused before any database question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonFault {
    /// It does not parse.
    Parse,
    /// It parses, but is not an object.
    NotObject,
    /// A string or key contains U+0000, which `jsonb` cannot store.
    Nul,
    /// A key at some depth matches the credential key guard.
    CredentialKey,
    /// `jsonb` refused it, or it exceeds its byte limit.
    Database,
    /// Its `jsonb` rendering exceeds the limit.
    TooLarge,
}

impl JsonFault {
    pub(crate) fn describe(self, input: JsonInput) -> String {
        let name = input.name();
        match self {
            JsonFault::Parse => format!("the {name} is not valid JSON"),
            JsonFault::NotObject => format!("the {name} must be a JSON object"),
            JsonFault::Nul => format!("the {name} contains U+0000"),
            JsonFault::CredentialKey => format!("the {name} has a credential-named key"),
            JsonFault::Database => format!("the {name} is not storable as jsonb"),
            JsonFault::TooLarge => format!("the {name} exceeds its byte limit"),
        }
    }
}

/// Parse a JSON text and run the Rust-side checks: it parses, is an object,
/// has no U+0000 anywhere, and (when `guard_credentials`) no credential-named
/// key at any depth. Serde only validates: the original text is what is
/// stored.
pub(crate) fn parse_json_object(
    text: &str,
    guard_credentials: bool,
) -> Result<serde_json::Value, JsonFault> {
    let value = parse_json(text)?;
    check_json_object(&value, guard_credentials)?;
    Ok(value)
}

/// Parse a JSON text and refuse U+0000 in any string or key: the two faults
/// that leave a command input without a canonical form.
pub(crate) fn parse_json(text: &str) -> Result<serde_json::Value, JsonFault> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| JsonFault::Parse)?;
    if contains_nul(&value) {
        return Err(JsonFault::Nul);
    }
    Ok(value)
}

/// The value is an object and, when `guard_credentials`, has no
/// credential-named key at any depth.
pub(crate) fn check_json_object(
    value: &serde_json::Value,
    guard_credentials: bool,
) -> Result<(), JsonFault> {
    if !value.is_object() {
        return Err(JsonFault::NotObject);
    }
    if guard_credentials && has_credential_key(value) {
        return Err(JsonFault::CredentialKey);
    }
    Ok(())
}

/// True when any string or object key in `value` contains U+0000.
pub(crate) fn contains_nul(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(s) => s.contains('\0'),
        serde_json::Value::Array(items) => items.iter().any(contains_nul),
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(key, item)| key.contains('\0') || contains_nul(item)),
        _ => false,
    }
}

/// True when an object key at any depth matches the credential key guard.
pub(crate) fn has_credential_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().any(has_credential_key),
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(key, item)| CREDENTIAL_KEY.is_match(key) || has_credential_key(item)),
        _ => false,
    }
}

#[derive(QueryableByName)]
struct JsonMeasure {
    #[diesel(sql_type = Int4)]
    bytes: i32,
    #[diesel(sql_type = Text)]
    digest: String,
}

/// The `jsonb` byte measure and canonical digest of one JSON text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JsonMeasured {
    /// `octet_length($1::jsonb::text)`.
    pub bytes: i32,
    /// SHA-256 hex of the canonical form, `$1::jsonb::text`.
    pub digest: String,
}

/// Ask the database, inside a savepoint so neither statement can abort the
/// caller's transaction, for `octet_length($1::jsonb::text)` and the SHA-256
/// of the canonical form. `Ok(None)` means `jsonb` refused the value; a
/// busy-class error (timeout, deadlock, cancel, closed connection, failed
/// send) is `ASYNC_JOB_BUSY`.
pub(crate) fn json_measure(
    conn: &mut PgConnection,
    text: &str,
) -> ThothResult<Option<JsonMeasured>> {
    let measured = in_savepoint(conn, |conn| {
        sql_query(
            "SELECT octet_length($1::jsonb::text) AS bytes, \
                    encode(sha256(convert_to($1::jsonb::text, 'UTF8')), 'hex') AS digest",
        )
        .bind::<Text, _>(text)
        .get_result::<JsonMeasure>(conn)
    });
    match measured {
        Ok(measure) => Ok(Some(JsonMeasured {
            bytes: measure.bytes,
            digest: measure.digest,
        })),
        Err(error) if is_busy_error(&error) => Err(ThothError::AsyncJobBusy),
        Err(_) => Ok(None),
    }
}

/// [`json_measure`] against `input`'s byte limit: `JsonFault::Database` when
/// `jsonb` refuses the value, `JsonFault::TooLarge` over the limit; otherwise
/// the canonical digest.
pub(crate) fn measure_json(
    conn: &mut PgConnection,
    text: &str,
    input: JsonInput,
) -> ThothResult<Result<String, JsonFault>> {
    Ok(match json_measure(conn, text)? {
        None => Err(JsonFault::Database),
        Some(measured) if measured.bytes > input.max_bytes() => Err(JsonFault::TooLarge),
        Some(measured) => Ok(measured.digest),
    })
}

// ---------------------------------------------------------------------------
// Column formats (sections 7 and 10.5)
// ---------------------------------------------------------------------------

/// The identifier format: 1 to 256 characters of printable non-space ASCII.
pub(crate) fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= IDENTIFIER_MAX_CHARS
        && value.chars().all(|c| ('!'..='~').contains(&c))
}

/// An outcome or error code: `^[A-Z][A-Z0-9_]*$`, at most 64 characters.
pub(crate) fn is_code(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && value.chars().count() <= CODE_MAX_CHARS
}

/// A kind: `^[A-Z][A-Z0-9_]{2,63}$`.
pub(crate) fn is_kind_format(value: &str) -> bool {
    let count = value.chars().count();
    (3..=64).contains(&count) && is_code(value)
}

/// A target namespace: `^[a-z][a-z0-9_]{1,31}$`.
pub(crate) fn is_namespace(value: &str) -> bool {
    let mut chars = value.chars();
    let count = value.chars().count();
    (2..=32).contains(&count)
        && matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// A target key: a namespace, a colon, then 1 to 400 characters of printable
/// non-space ASCII. Returns the namespace.
pub(crate) fn target_key_namespace(value: &str) -> Option<&str> {
    let (namespace, rest) = value.split_once(':')?;
    if !is_namespace(namespace) || rest.is_empty() {
        return None;
    }
    if rest.chars().count() > TARGET_KEY_VALUE_MAX_CHARS
        || !rest.chars().all(|c| ('!'..='~').contains(&c))
    {
        return None;
    }
    Some(namespace)
}

/// Non-blank in the database's sense (`~ '[^[:space:]]'`). Rust's
/// `char::is_whitespace` (Unicode `White_Space`) is a superset of every
/// regex strategy's space class, so this is never laxer than the check.
pub(crate) fn is_non_blank(value: &str) -> bool {
    value.chars().any(|c| !c.is_whitespace())
}

/// A text input bound as `text`: refused when it contains U+0000.
pub(crate) fn require_no_nul(value: &str, name: &str) -> ThothResult<()> {
    if value.contains('\0') {
        return Err(ThothError::AsyncJobPayloadInvalid(format!(
            "the {name} contains U+0000"
        )));
    }
    Ok(())
}

/// Free text (`error_detail`, `provider_ref`): stripped of control
/// characters, trimmed and truncated to `max` characters; never refused for
/// length. Empty text is absent.
pub(crate) fn sanitize_free_text(value: &str, max: usize) -> Option<String> {
    let stripped: String = value.chars().filter(|c| !c.is_control()).collect();
    let truncated: String = stripped.trim().chars().take(max).collect();
    if truncated.is_empty() {
        None
    } else {
        Some(truncated)
    }
}

// ---------------------------------------------------------------------------
// Backoff (section 9.3)
// ---------------------------------------------------------------------------

/// `min(max, base * 2^(n - 1))` for `n >= 1`, saturating: when `n - 1 >= 31`
/// or the product exceeds `max`, the result is `max`. Never overflows.
pub fn backoff_delay_seconds(base_seconds: i32, max_seconds: i32, n: i32) -> i64 {
    let n = n.max(1);
    let base = i64::from(base_seconds);
    let max = i64::from(max_seconds);
    // `n - 1 >= 31`: the shift itself would leave the 32-bit range.
    if n >= 32 {
        return max;
    }
    match base.checked_mul(1_i64 << (n - 1)) {
        Some(product) if product <= max => product,
        _ => max,
    }
}

/// The jitter percentage expression, in `0..=20`, over a `bigint` hash. No
/// `abs`, which overflows on the minimum `bigint`.
pub(crate) fn jitter_expression(hash: &str) -> String {
    format!("((({hash} % 21) + 21) % 21)")
}

#[derive(QueryableByName)]
struct JitterRow {
    #[diesel(sql_type = BigInt)]
    j: i64,
}

/// `j = ((hashtextextended(async_job_id::text || ':' || attempt_count, 0) % 21) + 21) % 21`.
pub(crate) fn jitter_percent(
    conn: &mut PgConnection,
    job_id: Uuid,
    attempt_count: i32,
) -> QueryResult<i64> {
    let sql = format!(
        "SELECT {} AS j",
        jitter_expression("hashtextextended($1::text || ':' || $2, 0)")
    );
    sql_query(sql)
        .bind::<SqlUuid, _>(job_id)
        .bind::<Int4, _>(attempt_count)
        .get_result::<JitterRow>(conn)
        .map(|row| row.j)
}

/// The backoff delay plus its deterministic jitter, `floor(delay * j / 100)`.
pub fn jittered_delay_seconds(delay: i64, jitter_percent: i64) -> i64 {
    delay + (delay * jitter_percent) / 100
}

/// `now + backoff(n) + jitter` for a job.
pub(crate) fn backoff_until(
    conn: &mut PgConnection,
    job: &JobRow,
    n: i32,
    now: DateTime<Utc>,
) -> QueryResult<DateTime<Utc>> {
    let delay = backoff_delay_seconds(job.backoff_base_seconds, job.backoff_max_seconds, n);
    let j = jitter_percent(conn, job.async_job_id, job.attempt_count)?;
    Ok(now + Duration::seconds(jittered_delay_seconds(delay, j)))
}

// ---------------------------------------------------------------------------
// Engine-owned transactions (section 10)
// ---------------------------------------------------------------------------

/// The error a wrapped engine transaction carries: the engine's own answer,
/// or a database error from `BEGIN`, `SET LOCAL`, the commit or the rollback.
enum WrapperError {
    Engine(ThothError),
    Database(DieselError),
}

impl From<DieselError> for WrapperError {
    fn from(error: DieselError) -> Self {
        WrapperError::Database(error)
    }
}

/// Map a failure at a wrapper's own `BEGIN`, `SET LOCAL`, commit or
/// rollback: a timeout, deadlock, cancel, closed connection or failed send is
/// `ASYNC_JOB_BUSY` (after a closed connection at commit the outcome is
/// unknown, so the caller re-reads before retrying); anything else is
/// `ASYNC_JOB_INTEGRITY_VIOLATION`.
pub(crate) fn map_commit_error(error: DieselError) -> ThothError {
    map_db_error(error)
}

/// Run `operation` in an engine-owned READ COMMITTED transaction with
/// `SET LOCAL lock_timeout = '5s'`, on a connection from `pool`. A pool that
/// cannot hand out a connection is `ASYNC_JOB_BUSY`.
pub fn with_engine_transaction<T>(
    pool: &PgPool,
    operation: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    let mut conn = pool.get().map_err(|_| ThothError::AsyncJobBusy)?;
    with_engine_transaction_on(&mut conn, operation)
}

/// [`with_engine_transaction`] on a given connection, which must not be in a
/// transaction.
pub fn with_engine_transaction_on<T>(
    conn: &mut PgConnection,
    operation: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    let result = conn
        .build_transaction()
        .read_committed()
        .run(|conn| -> Result<T, WrapperError> {
            conn.batch_execute(&format!("SET LOCAL lock_timeout = '{ENGINE_LOCK_TIMEOUT}'"))?;
            operation(conn).map_err(WrapperError::Engine)
        });
    match result {
        Ok(value) => Ok(value),
        Err(WrapperError::Engine(error)) => Err(error),
        Err(WrapperError::Database(error)) => Err(map_commit_error(error)),
    }
}

// ---------------------------------------------------------------------------
// Small shared readers
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
pub(crate) struct BoolRow {
    #[diesel(sql_type = Bool)]
    pub value: bool,
}

#[derive(QueryableByName)]
pub(crate) struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName)]
pub(crate) struct UuidRow {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
}

#[derive(QueryableByName)]
pub(crate) struct LockKeyRow {
    #[diesel(sql_type = Int4)]
    pub key: i32,
    #[diesel(sql_type = Text)]
    pub name: String,
}

/// Lock the job row `FOR UPDATE` by id.
pub(crate) fn lock_job(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<Option<JobRow>> {
    db(sql_query(format!(
        "SELECT {JOB_COLUMNS} FROM public.async_job j WHERE j.async_job_id = $1 FOR UPDATE OF j"
    ))
    .bind::<SqlUuid, _>(job_id)
    .get_result::<JobRow>(conn)
    .optional())
}

/// Read a job without locking it.
pub(crate) fn read_job(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<Option<JobRow>> {
    db(sql_query(format!(
        "SELECT {JOB_COLUMNS} FROM public.async_job j WHERE j.async_job_id = $1"
    ))
    .bind::<SqlUuid, _>(job_id)
    .get_result::<JobRow>(conn)
    .optional())
}

/// `hashtext` lock keys of `names`, ascending by key then name (every name
/// kept: the poll upserts of a claim follow this order).
pub(crate) fn lock_key_rows(
    conn: &mut PgConnection,
    names: &[String],
) -> QueryResult<Vec<LockKeyRow>> {
    sql_query(
        "SELECT DISTINCT hashtext(n) AS key, n AS name FROM unnest($1::text[]) AS n ORDER BY 1, 2",
    )
    .bind::<Array<Text>, _>(names)
    .load::<LockKeyRow>(conn)
}

/// [`lock_key_rows`] with duplicate keys removed: the order in which a
/// transaction takes advisory locks.
pub(crate) fn lock_keys(conn: &mut PgConnection, names: &[String]) -> QueryResult<Vec<LockKeyRow>> {
    let mut rows = lock_key_rows(conn, names)?;
    rows.dedup_by(|b, a| a.key == b.key);
    Ok(rows)
}
