//! GraphQL types of the async job engine's protected API
//! (THOTH-ASYNC-01-ENGINE-01B, spec section 14).
//!
//! Every type here carries an engine value across the API. The enums mirror
//! the engine's database labels, the inputs convert into engine requests and
//! the outputs are built from engine rows. Payloads, checkpoints and evidence
//! cross as JSON strings, every `bigint` as `BigInt`, and no output type
//! except [`ClaimedAsyncJob`] has a claim token.
//!
//! [`AsyncJobClaimMemo`] is the per-request claim memo of spec section 14.1.

use std::fmt;
use std::sync::Mutex;

use diesel::pg::PgConnection;
use juniper::{FieldResult, IntoFieldError};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::graphql::scalars::BigInt;
use crate::model::async_job::executor::{instant, payload_versions, timestamp};
use crate::model::async_job::{self as engine, get_job, job_attempts, job_transitions};
use crate::model::Timestamp;

// ---------------------------------------------------------------------------
// Enums: one GraphQL enum per engine enum, with the same labels
// ---------------------------------------------------------------------------

/// A GraphQL enum mirroring one engine enum variant for variant, with the
/// conversions both ways. Juniper renders each variant in SCREAMING_SNAKE_CASE,
/// which is the engine's database label (a test compares them).
macro_rules! mirror_enum {
    (
        $(#[$meta:meta])*
        $name:ident => engine::$engine:ident {
            $($(#[$variant_meta:meta])* $variant:ident),+ $(,)?
        }
    ) => {
        #[derive(juniper::GraphQLEnum, Debug, Clone, Copy, PartialEq, Eq)]
        $(#[$meta])*
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
        }

        impl From<engine::$engine> for $name {
            fn from(value: engine::$engine) -> Self {
                match value {
                    $(engine::$engine::$variant => $name::$variant,)+
                }
            }
        }

        impl From<$name> for engine::$engine {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => engine::$engine::$variant,)+
                }
            }
        }
    };
}

mirror_enum! {
    #[graphql(description = "Lifecycle state of an async job")]
    AsyncJobStatus => engine::AsyncJobStatus {
        #[graphql(description = "Created and not claimed")]
        Pending,
        #[graphql(description = "Claimed by an executor")]
        Running,
        #[graphql(description = "Checkpointed and yielded until its resume time")]
        Waiting,
        #[graphql(description = "Failed before its effect started and scheduled for another attempt")]
        RetryScheduled,
        #[graphql(description = "Finished successfully")]
        Succeeded,
        #[graphql(description = "Finished without success")]
        Failed,
        #[graphql(description = "Cancelled by staff before its effect started")]
        Cancelled,
        #[graphql(description = "Its effect may have started, and a reconciler or staff must establish what happened")]
        ReconciliationRequired,
    }
}

mirror_enum! {
    #[graphql(description = "How far an async job's external effect has progressed. It only advances")]
    AsyncEffectPhase => engine::AsyncEffectPhase {
        #[graphql(description = "No external effect has started")]
        PreWrite,
        #[graphql(description = "The external effect has started")]
        EffectStarted,
        #[graphql(description = "The provider acknowledged the external effect")]
        EffectConfirmed,
    }
}

mirror_enum! {
    #[graphql(description = "Whether an async job kind writes an external effect")]
    AsyncJobEffectClass => engine::EffectClass {
        #[graphql(description = "The kind writes no external effect")]
        ReadOnly,
        #[graphql(description = "The kind writes an external effect")]
        Effectful,
    }
}

mirror_enum! {
    #[graphql(description = "How an async job kind's applied evidence is ordered")]
    AsyncJobRevisionMode => engine::RevisionMode {
        #[graphql(description = "Evidence is a fingerprint of the target's current state")]
        CurrentState,
        #[graphql(description = "Evidence is an ordered revision of the job's source")]
        RevisionBound,
    }
}

mirror_enum! {
    #[graphql(description = "Whether an async job is original or a staff retry of another job")]
    AsyncJobLineage => engine::AsyncJobLineage {
        #[graphql(description = "Created by its owning domain")]
        Original,
        #[graphql(description = "Created by a staff retry of its parent job")]
        ManualRetry,
    }
}

mirror_enum! {
    #[graphql(description = "What an async job attempt was opened for")]
    AsyncAttemptPurpose => engine::AsyncAttemptPurpose {
        #[graphql(description = "An executor ran the job")]
        Execution,
        #[graphql(description = "An executor reconciled the job")]
        Reconciliation,
        #[graphql(description = "Staff resolved the job's reconciliation")]
        StaffReconciliation,
    }
}

mirror_enum! {
    #[graphql(description = "How an async job attempt closed")]
    AsyncAttemptResult => engine::AsyncAttemptResult {
        #[graphql(description = "The attempt finished the job successfully")]
        Succeeded,
        #[graphql(description = "The attempt failed before its effect started and the job may be retried")]
        RetryableFailure,
        #[graphql(description = "The attempt failed permanently")]
        PermanentFailure,
        #[graphql(description = "The attempt ended without proof of whether its effect holds")]
        Indeterminate,
        #[graphql(description = "The attempt checkpointed the job and yielded")]
        Waiting,
        #[graphql(description = "The executor yielded the claim before any effect started")]
        Released,
        #[graphql(description = "The claim's lease expired")]
        LeaseExpired,
        #[graphql(description = "Staff stopped the attempt")]
        Cancelled,
        #[graphql(description = "The reconciler could not establish the outcome yet")]
        StillRequired,
    }
}

mirror_enum! {
    #[graphql(description = "Which operator command a command record is")]
    AsyncControlCommandKind => engine::AsyncControlCommandKind {
        #[graphql(description = "cancelAsyncJob")]
        CancelJob,
        #[graphql(description = "retryAsyncJob")]
        RetryJob,
        #[graphql(description = "resolveAsyncJobReconciliationAsStaff")]
        ResolveReconciliation,
        #[graphql(description = "redactAsyncJobDiagnostics")]
        RedactJobDiagnostics,
    }
}

mirror_enum! {
    #[graphql(description = "The recorded decision of an operator command")]
    AsyncControlCommandOutcome => engine::AsyncControlCommandOutcome {
        #[graphql(description = "The command was applied")]
        Applied,
        #[graphql(description = "The command was refused and changed no job. The refusal is recorded")]
        Refused,
    }
}

mirror_enum! {
    #[graphql(description = "What a completion reports about the job's effect")]
    AsyncJobCompletionOutcome => engine::CompletionOutcome {
        #[graphql(description = "The effect was written at the provider. Entries carry the applied evidence")]
        EffectApplied,
        #[graphql(description = "Before any effect started, the provider was found to hold the job's state already. Entries carry the observed evidence")]
        NoOp,
        #[graphql(description = "Before any effect started, a newer revision was found applied to the job's target. Entries are key-only")]
        Superseded,
        #[graphql(description = "A read-only job finished. Entries are key-only")]
        Completed,
    }
}

mirror_enum! {
    #[graphql(description = "How an executor classifies a failure")]
    AsyncJobFailureClass => engine::FailureClass {
        #[graphql(description = "A failure before the effect started that may be retried")]
        RetryablePreWrite,
        #[graphql(description = "A failure that must not be retried")]
        Permanent,
        #[graphql(description = "A failure whose effect at the provider is unknown")]
        Indeterminate,
    }
}

mirror_enum! {
    #[graphql(description = "The outcomes an executor reconciler may record")]
    AsyncJobReconcilerOutcome => engine::ReconcilerOutcome {
        #[graphql(description = "The job's effect holds at the provider. Entries carry the applied evidence")]
        Succeeded,
        #[graphql(description = "No effect exists at the provider, which only a job at EFFECT_STARTED can record. Evidence proves it")]
        Failed,
        #[graphql(description = "As FAILED, recorded as cancelled")]
        Cancelled,
        #[graphql(description = "Check again later, from checkpoint at resumeAt")]
        Waiting,
        #[graphql(description = "The outcome cannot be established yet, and the job stays RECONCILIATION_REQUIRED")]
        StillRequired,
    }
}

mirror_enum! {
    #[graphql(description = "The outcomes a staff reconciliation may record, as far as the job's kind lists them")]
    AsyncJobStaffOutcome => engine::StaffOutcome {
        #[graphql(description = "The job's effect holds at the provider. Entries carry the applied evidence")]
        Succeeded,
        #[graphql(description = "No effect exists at the provider, or a confirmed effect does not hold there. Evidence proves it")]
        Failed,
        #[graphql(description = "No effect exists at the provider, recorded as cancelled. Evidence proves it")]
        Cancelled,
        #[graphql(description = "Check again later, from checkpoint at resumeAt")]
        Waiting,
        #[graphql(description = "The provider holds this job's own revision although the stored evidence is newer or conflicts. Evidence proves it")]
        AcceptRegression,
    }
}

// ---------------------------------------------------------------------------
// Executor inputs
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "One kind and the payload versions the executor supports for it")]
pub struct AsyncJobKindVersionsInput {
    #[graphql(description = "The kind code")]
    pub kind: String,
    #[graphql(description = "The supported payload versions, at least one")]
    pub payload_versions: Vec<i32>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "Which kinds and payload versions to claim, how many jobs and for how long. The executor's domain, risk class, capabilities and revision modes come from its credentials, never from this input"
)]
pub struct ClaimAsyncJobsInput {
    #[graphql(
        description = "The executor name recorded on the claim, non-blank and at most 128 characters"
    )]
    pub claimed_by: String,
    #[graphql(
        description = "The kinds to claim, each with the payload versions the executor supports. Kinds this server does not know are reported in skippedKinds"
    )]
    pub kinds: Vec<AsyncJobKindVersionsInput>,
    #[graphql(description = "How many jobs to claim at most, from 1 to 50")]
    pub max_jobs: i32,
    #[graphql(
        description = "The requested lease in seconds, clamped to each kind's bounds. The kind's default when absent"
    )]
    pub lease_seconds: Option<i32>,
}

impl ClaimAsyncJobsInput {
    /// The engine request. Domain, risk class, capabilities and revision
    /// modes are not in it: they come from the executor map.
    pub(crate) fn claim_request(&self) -> ThothResult<engine::ClaimRequest> {
        let kinds = self
            .kinds
            .iter()
            .map(|kind| -> ThothResult<(String, Vec<i16>)> {
                Ok((kind.kind.clone(), payload_versions(&kind.payload_versions)?))
            })
            .collect::<ThothResult<Vec<_>>>()?;
        Ok(engine::ClaimRequest {
            kinds,
            max_jobs: self.max_jobs,
            lease_seconds: self.lease_seconds,
        })
    }
}

/// A claimed job and its claim token. Its `Debug` never prints the token.
#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claimed job and the claim token its claim returned")]
pub struct AsyncJobClaimInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
}

impl fmt::Debug for AsyncJobClaimInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncJobClaimInput")
            .field("async_job_id", &self.async_job_id)
            .field("claim_token", &"<redacted>")
            .finish()
    }
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claim to renew and the requested lease")]
pub struct RenewAsyncJobLeaseInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
    #[graphql(
        description = "The requested lease in seconds, clamped to the kind's bounds and never past the attempt's runtime deadline. The kind's default when absent"
    )]
    pub lease_seconds: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claim whose external effect the provider acknowledged")]
pub struct MarkAsyncJobEffectConfirmedInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
    #[graphql(
        description = "The provider's reference for the effect: free text, stripped of control characters and truncated to 512 characters. Never include credentials"
    )]
    pub provider_ref: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claim that checkpoints its job and yields it until a later time")]
pub struct WaitAsyncJobInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
    #[graphql(
        description = "A JSON object, as a string, that the next attempt resumes from, at most 65536 bytes"
    )]
    pub checkpoint: String,
    #[graphql(
        description = "When the job may be claimed again, within the kind's poll interval and waiting deadline"
    )]
    pub resume_at: Timestamp,
    #[graphql(
        description = "The provider's reference for the effect: free text, stripped of control characters and truncated to 512 characters. Never include credentials"
    )]
    pub provider_ref: Option<String>,
}

impl WaitAsyncJobInput {
    pub(crate) fn wait_request(&self) -> engine::WaitRequest {
        engine::WaitRequest {
            checkpoint: self.checkpoint.clone(),
            resume_at: instant(self.resume_at),
            provider_ref: self.provider_ref.clone(),
        }
    }
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "One target's completion evidence. Which fields an entry carries depends on the outcome and the kind's revision mode"
)]
pub struct AsyncJobEvidenceEntryInput {
    #[graphql(description = "The target key. The entries name every job target exactly once")]
    pub target_key: String,
    #[graphql(
        description = "The applied revision order, for REVISION_BOUND kinds only. It must be in the PostgreSQL bigint range"
    )]
    pub order: Option<BigInt>,
    #[graphql(description = "The applied state's fingerprint, absent from key-only entries")]
    pub fingerprint: Option<String>,
    #[graphql(description = "The applied source reference, where the outcome lists one")]
    pub source_ref: Option<String>,
}

impl From<&AsyncJobEvidenceEntryInput> for engine::EvidenceEntry {
    /// `order` reaches the engine as `i128`, without truncation: the engine
    /// makes the `bigint` range check.
    fn from(entry: &AsyncJobEvidenceEntryInput) -> Self {
        engine::EvidenceEntry {
            target_key: entry.target_key.clone(),
            order: entry.order.map(BigInt::value),
            fingerprint: entry.fingerprint.clone(),
            source_ref: entry.source_ref.clone(),
        }
    }
}

fn evidence_entries(entries: &[AsyncJobEvidenceEntryInput]) -> Vec<engine::EvidenceEntry> {
    entries.iter().map(engine::EvidenceEntry::from).collect()
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claimed job's completion and its evidence")]
pub struct CompleteAsyncJobInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
    #[graphql(description = "What the completion reports")]
    pub outcome: AsyncJobCompletionOutcome,
    #[graphql(
        description = "One entry per job target, none for a job without targets: evidence for EFFECT_APPLIED and NO_OP, key-only for SUPERSEDED and COMPLETED"
    )]
    pub entries: Vec<AsyncJobEvidenceEntryInput>,
    #[graphql(
        description = "The kind's applied evidence object as a JSON string. Optional with EFFECT_APPLIED and NO_OP, refused with any other outcome"
    )]
    pub evidence: Option<String>,
}

impl CompleteAsyncJobInput {
    pub(crate) fn complete_request(&self) -> engine::CompleteRequest {
        engine::CompleteRequest {
            outcome: self.outcome.into(),
            entries: evidence_entries(&self.entries),
            evidence: self.evidence.clone(),
        }
    }
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claimed job's failure")]
pub struct FailAsyncJobInput {
    #[graphql(description = "The claimed job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the claim returned")]
    pub claim_token: Uuid,
    #[graphql(description = "How the failure is classified")]
    pub failure_class: AsyncJobFailureClass,
    #[graphql(description = "A machine-readable error code")]
    pub error_code: String,
    #[graphql(
        description = "Free text, stripped of control characters and truncated to 2048 characters. Never include credentials"
    )]
    pub error_detail: Option<String>,
    #[graphql(
        description = "True to record that the started effect provably does not exist: PERMANENT at EFFECT_STARTED only, for a kind that allows it, with evidence"
    )]
    pub no_effect_proven: bool,
    #[graphql(
        description = "The kind's no-effect evidence object as a JSON string. Required with noEffectProven and refused without it"
    )]
    pub evidence: Option<String>,
}

impl FailAsyncJobInput {
    pub(crate) fn fail_request(&self) -> engine::FailRequest {
        engine::FailRequest {
            class: self.failure_class.into(),
            error_code: self.error_code.clone(),
            error_detail: self.error_detail.clone(),
            no_effect_proven: self.no_effect_proven,
            evidence: self.evidence.clone(),
        }
    }
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A claimed reconciliation's resolution and its evidence")]
pub struct ResolveAsyncJobReconciliationInput {
    #[graphql(description = "The job whose reconciliation is claimed")]
    pub async_job_id: Uuid,
    #[graphql(description = "The token the reconciliation claim returned")]
    pub claim_token: Uuid,
    #[graphql(description = "What the reconciler established")]
    pub outcome: AsyncJobReconcilerOutcome,
    #[graphql(
        description = "One entry per job target, required for SUCCEEDED and refused with any other outcome"
    )]
    pub entries: Option<Vec<AsyncJobEvidenceEntryInput>>,
    #[graphql(
        description = "The kind's evidence object as a JSON string: applied evidence, optional, for SUCCEEDED; no-effect evidence, required, for FAILED and CANCELLED; refused otherwise"
    )]
    pub evidence: Option<String>,
    #[graphql(
        description = "WAITING only: a JSON object, as a string, that the next attempt resumes from"
    )]
    pub checkpoint: Option<String>,
    #[graphql(description = "WAITING only: when the job may be claimed again")]
    pub resume_at: Option<Timestamp>,
}

impl ResolveAsyncJobReconciliationInput {
    pub(crate) fn resolution(&self) -> engine::ResolveReconciliation {
        engine::ResolveReconciliation {
            outcome: self.outcome.into(),
            entries: self.entries.as_deref().map(evidence_entries),
            evidence: self.evidence.clone(),
            checkpoint: self.checkpoint.clone(),
            resume_at: self.resume_at.map(instant),
        }
    }
}

// ---------------------------------------------------------------------------
// Operator inputs
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "An operator command on one async job")]
pub struct AsyncJobCommandInput {
    #[graphql(description = "The job")]
    pub async_job_id: Uuid,
    #[graphql(
        description = "Why staff issue the command: audit text kept for the life of the record, non-blank and at most 1000 characters. Do not paste provider diagnostics or personal data into it"
    )]
    pub reason: String,
    #[graphql(
        description = "A new ID for each command attempt. A repeat with the same ID and inputs returns the stored outcome, REFUSED included, for ever, so a new attempt needs a new ID"
    )]
    pub request_id: Uuid,
}

#[derive(juniper::GraphQLInputObject, Clone, PartialEq, Eq)]
#[graphql(description = "A staff resolution of an async job's reconciliation and its evidence")]
pub struct ResolveAsyncJobReconciliationAsStaffInput {
    #[graphql(description = "The job, which must be RECONCILIATION_REQUIRED")]
    pub async_job_id: Uuid,
    #[graphql(description = "One of the outcomes the job's kind lists for staff")]
    pub outcome: AsyncJobStaffOutcome,
    #[graphql(
        description = "One entry per job target, required for SUCCEEDED and refused with any other outcome"
    )]
    pub entries: Option<Vec<AsyncJobEvidenceEntryInput>>,
    #[graphql(
        description = "The kind's evidence object as a JSON string: applied evidence for SUCCEEDED (optional) and ACCEPT_REGRESSION (required), no-effect evidence for FAILED and CANCELLED at EFFECT_STARTED, not-applied evidence for FAILED at EFFECT_CONFIRMED; refused for WAITING"
    )]
    pub evidence: Option<String>,
    #[graphql(
        description = "WAITING only: a JSON object, as a string, that the next attempt resumes from"
    )]
    pub checkpoint: Option<String>,
    #[graphql(description = "WAITING only: when the job may be claimed again")]
    pub resume_at: Option<Timestamp>,
    #[graphql(
        description = "Why staff issue the command: audit text kept for the life of the record, non-blank and at most 1000 characters. Do not paste provider diagnostics or personal data into it"
    )]
    pub reason: String,
    #[graphql(
        description = "A new ID for each command attempt. A repeat with the same ID and inputs returns the stored outcome, REFUSED included, for ever, so a new attempt needs a new ID"
    )]
    pub request_id: Uuid,
}

impl ResolveAsyncJobReconciliationAsStaffInput {
    pub(crate) fn resolution(&self) -> engine::StaffResolution {
        engine::StaffResolution {
            outcome: self.outcome.into(),
            entries: self.entries.as_deref().map(evidence_entries),
            evidence: self.evidence.clone(),
            checkpoint: self.checkpoint.clone(),
            resume_at: self.resume_at.map(instant),
        }
    }
}

// ---------------------------------------------------------------------------
// Read inputs
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "Filters of the async job list: a list matches any of its values, and filters combine with AND. An absent or empty filter matches every job"
)]
pub struct AsyncJobFilterInput {
    #[graphql(description = "Only jobs of these kinds")]
    pub kinds: Option<Vec<String>>,
    #[graphql(description = "Only jobs in these statuses")]
    pub statuses: Option<Vec<AsyncJobStatus>>,
    #[graphql(description = "Only jobs with this target key")]
    pub target_key: Option<String>,
    #[graphql(description = "Only jobs with this correlation ID")]
    pub correlation_id: Option<Uuid>,
}

impl From<&AsyncJobFilterInput> for engine::AsyncJobFilter {
    fn from(filter: &AsyncJobFilterInput) -> Self {
        engine::AsyncJobFilter {
            kinds: filter.kinds.clone().unwrap_or_default(),
            statuses: filter
                .statuses
                .iter()
                .flatten()
                .map(|status| (*status).into())
                .collect(),
            target_key: filter.target_key.clone(),
            correlation_id: filter.correlation_id,
        }
    }
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "A position in the async job list: the createdAt and asyncJobId of the last job of the previous page"
)]
pub struct AsyncJobCursorInput {
    #[graphql(description = "createdAt of the last job of the previous page")]
    pub created_at: Timestamp,
    #[graphql(description = "asyncJobId of the last job of the previous page")]
    pub async_job_id: Uuid,
}

impl From<&AsyncJobCursorInput> for engine::JobCursor {
    fn from(cursor: &AsyncJobCursorInput) -> Self {
        engine::JobCursor {
            created_at: instant(cursor.created_at),
            async_job_id: cursor.async_job_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Executor outputs
// ---------------------------------------------------------------------------

/// A claimed job. Its `Debug` never prints the claim token.
#[derive(juniper::GraphQLObject, Clone, PartialEq)]
#[graphql(
    description = "A job together with the claim just granted on it. Only a claim returns a claim token"
)]
pub struct ClaimedAsyncJob {
    #[graphql(description = "ID of the job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The job's kind")]
    pub kind: String,
    #[graphql(description = "The payload's version")]
    pub payload_version: i32,
    #[graphql(description = "The payload as a JSON string")]
    pub payload: String,
    #[graphql(description = "The source revision's order, for a REVISION_BOUND job")]
    pub source_order: Option<BigInt>,
    #[graphql(description = "The source revision's fingerprint, for a REVISION_BOUND job")]
    pub source_fingerprint: Option<String>,
    #[graphql(description = "The source revision's reference, for a REVISION_BOUND job")]
    pub source_ref: Option<String>,
    #[graphql(description = "The job's target keys, ascending")]
    pub target_keys: Vec<String>,
    #[graphql(
        description = "Present this token on every write and read for this claim. It is returned only here"
    )]
    pub claim_token: Uuid,
    #[graphql(description = "When this claim's lease expires unless renewed")]
    pub lease_expires_at: Timestamp,
    #[graphql(description = "When this attempt must end, whatever its renewals")]
    pub attempt_runtime_deadline: Timestamp,
    #[graphql(description = "Ordinal of the attempt this claim opened")]
    pub attempt_number: i32,
    #[graphql(description = "The job's effect phase as the attempt starts")]
    pub effect_phase: AsyncEffectPhase,
    #[graphql(description = "The checkpoint the attempt resumes from, as a JSON string, if any")]
    pub checkpoint: Option<String>,
    #[graphql(description = "How many checkpoints the job has written")]
    pub checkpoint_version: i32,
    #[graphql(description = "The provider's reference for the job's effect, if any")]
    pub provider_ref: Option<String>,
}

impl fmt::Debug for ClaimedAsyncJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaimedAsyncJob")
            .field("async_job_id", &self.async_job_id)
            .field("kind", &self.kind)
            .field("payload_version", &self.payload_version)
            .field("claim_token", &"<redacted>")
            .field("attempt_number", &self.attempt_number)
            .field("effect_phase", &self.effect_phase)
            .finish_non_exhaustive()
    }
}

impl From<engine::ClaimedJob> for ClaimedAsyncJob {
    fn from(job: engine::ClaimedJob) -> Self {
        ClaimedAsyncJob {
            async_job_id: job.async_job_id,
            kind: job.kind,
            payload_version: i32::from(job.payload_version),
            payload: job.payload,
            source_order: job.source_order.map(BigInt::from),
            source_fingerprint: job.source_fingerprint,
            source_ref: job.source_ref,
            target_keys: job.target_keys,
            claim_token: job.claim_token,
            lease_expires_at: timestamp(job.lease_expires_at),
            attempt_runtime_deadline: timestamp(job.attempt_runtime_deadline),
            attempt_number: job.attempt_number,
            effect_phase: job.effect_phase.into(),
            checkpoint: job.checkpoint,
            checkpoint_version: job.checkpoint_version,
            provider_ref: job.provider_ref,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(description = "The jobs one claim granted")]
pub struct AsyncJobClaimResult {
    #[graphql(description = "The claimed jobs, in claim order")]
    pub jobs: Vec<ClaimedAsyncJob>,
    #[graphql(
        description = "Requested kinds this server does not know. None of their jobs was claimed"
    )]
    pub skipped_kinds: Vec<String>,
}

impl From<engine::ClaimResult> for AsyncJobClaimResult {
    fn from(result: engine::ClaimResult) -> Self {
        AsyncJobClaimResult {
            jobs: result.jobs.into_iter().map(ClaimedAsyncJob::from).collect(),
            skipped_kinds: result.skipped_kinds,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A renewed lease")]
pub struct AsyncJobLeaseRenewal {
    #[graphql(description = "When the renewed lease expires unless renewed again")]
    pub lease_expires_at: Timestamp,
    #[graphql(description = "When this attempt must end, whatever its renewals")]
    pub attempt_runtime_deadline: Timestamp,
}

impl From<engine::LeaseRenewal> for AsyncJobLeaseRenewal {
    fn from(renewal: engine::LeaseRenewal) -> Self {
        AsyncJobLeaseRenewal {
            lease_expires_at: timestamp(renewal.lease_expires_at),
            attempt_runtime_deadline: timestamp(renewal.attempt_runtime_deadline),
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "The applied evidence of one of a claimed job's targets. Every field after targetKey is null when no evidence was ever written for the target"
)]
pub struct AsyncJobTargetState {
    #[graphql(description = "The target key")]
    pub target_key: String,
    #[graphql(description = "The applied revision order, for a REVISION_BOUND target")]
    pub applied_order: Option<BigInt>,
    #[graphql(description = "The applied state's fingerprint")]
    pub applied_fingerprint: Option<String>,
    #[graphql(description = "The applied source reference")]
    pub applied_source_ref: Option<String>,
    #[graphql(description = "The job that last wrote the target's evidence")]
    pub last_job_id: Option<Uuid>,
    #[graphql(
        description = "When staff accepted a regression of this target, until an ordinary write reaches the revision it replaced"
    )]
    pub regressed_at: Option<Timestamp>,
    #[graphql(description = "The order the accepted regression replaced")]
    pub regressed_from_order: Option<BigInt>,
    #[graphql(description = "The fingerprint the accepted regression replaced")]
    pub regressed_from_fingerprint: Option<String>,
    #[graphql(description = "When the target's evidence last changed")]
    pub updated_at: Option<Timestamp>,
}

impl From<engine::TargetState> for AsyncJobTargetState {
    fn from(state: engine::TargetState) -> Self {
        AsyncJobTargetState {
            target_key: state.target_key,
            applied_order: state.applied_order.map(BigInt::from),
            applied_fingerprint: state.applied_fingerprint,
            applied_source_ref: state.applied_source_ref,
            last_job_id: state.last_job_id,
            regressed_at: state.regressed_at.map(timestamp),
            regressed_from_order: state.regressed_from_order.map(BigInt::from),
            regressed_from_fingerprint: state.regressed_from_fingerprint,
            updated_at: state.updated_at.map(timestamp),
        }
    }
}

// ---------------------------------------------------------------------------
// Jobs, attempts and transitions
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(
    description = "An async job and the policy snapshot it was created with. No claim token is ever returned here"
)]
pub struct AsyncJob {
    #[graphql(description = "ID of the job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The job's kind")]
    pub kind: String,
    #[graphql(description = "The payload's version")]
    pub payload_version: i32,
    #[graphql(description = "The payload as a JSON string")]
    pub payload: String,
    #[graphql(description = "The key that makes the job's creation idempotent")]
    pub idempotency_key: String,
    #[graphql(description = "Whether the job writes an external effect")]
    pub effect_class: AsyncJobEffectClass,
    #[graphql(description = "How the job's applied evidence is ordered")]
    pub revision_mode: AsyncJobRevisionMode,
    #[graphql(description = "How many failures before the effect started the job may retry")]
    pub retry_budget: i32,
    #[graphql(description = "How many execution claims the job may receive")]
    pub max_execution_claims: i32,
    #[graphql(description = "How many reconciliation claims the job may receive")]
    pub max_reconciliation_attempts: i32,
    #[graphql(description = "The base of the job's retry backoff, in seconds")]
    pub backoff_base_seconds: i32,
    #[graphql(description = "The cap of the job's retry backoff, in seconds")]
    pub backoff_max_seconds: i32,
    #[graphql(description = "The source revision's order, for a REVISION_BOUND job")]
    pub source_order: Option<BigInt>,
    #[graphql(description = "The source revision's fingerprint, for a REVISION_BOUND job")]
    pub source_fingerprint: Option<String>,
    #[graphql(description = "The source revision's reference, for a REVISION_BOUND job")]
    pub source_ref: Option<String>,
    #[graphql(description = "The job's lifecycle state")]
    pub status: AsyncJobStatus,
    #[graphql(description = "The job's priority within its kind, from -100 to 100")]
    pub priority: i32,
    #[graphql(description = "When the job may next be claimed")]
    pub available_at: Timestamp,
    #[graphql(description = "How far the job's external effect has progressed")]
    pub effect_phase: AsyncEffectPhase,
    #[graphql(description = "How many attempts the job has had")]
    pub attempt_count: i32,
    #[graphql(description = "How many execution claims the job has received")]
    pub execution_claim_count: i32,
    #[graphql(description = "How many failures the job has had before its effect started")]
    pub prewrite_failure_count: i32,
    #[graphql(description = "How many reconciliation claims the job has received")]
    pub reconciliation_attempt_count: i32,
    #[graphql(description = "The name of the executor holding the current claim")]
    pub claimed_by: Option<String>,
    #[graphql(description = "The authenticated identity holding the current claim")]
    pub executor_principal: Option<String>,
    #[graphql(description = "When the current claim was granted")]
    pub claimed_at: Option<Timestamp>,
    #[graphql(description = "When the current claim's lease expires")]
    pub lease_expires_at: Option<Timestamp>,
    #[graphql(description = "When the current attempt must end")]
    pub attempt_runtime_deadline: Option<Timestamp>,
    #[graphql(description = "When the job was first claimed")]
    pub first_claimed_at: Option<Timestamp>,
    #[graphql(description = "When the job's waiting ends, if it has waited")]
    pub waiting_deadline: Option<Timestamp>,
    #[graphql(description = "The job's checkpoint as a JSON string, if any")]
    pub checkpoint: Option<String>,
    #[graphql(description = "How many checkpoints the job has written")]
    pub checkpoint_version: i32,
    #[graphql(description = "The provider's reference for the job's effect, if any")]
    pub provider_ref: Option<String>,
    #[graphql(description = "When the job reached a terminal state")]
    pub terminal_at: Option<Timestamp>,
    #[graphql(description = "The outcome code of the job's latest transition that recorded one")]
    pub last_outcome_code: Option<String>,
    #[graphql(description = "When the job's diagnostics were redacted")]
    pub redacted_at: Option<Timestamp>,
    #[graphql(description = "Whether the job is original or a staff retry")]
    pub lineage: AsyncJobLineage,
    #[graphql(description = "The job a staff retry was made from")]
    pub parent_job_id: Option<Uuid>,
    #[graphql(description = "The correlation ID its creator gave the job")]
    pub correlation_id: Option<Uuid>,
    #[graphql(description = "What caused the job's creation, as its creator recorded it")]
    pub causation_ref: Option<String>,
    #[graphql(description = "Who created the job")]
    pub created_by: String,
    #[graphql(description = "When the job was created")]
    pub created_at: Timestamp,
    #[graphql(description = "When the job last changed")]
    pub updated_at: Timestamp,
    #[graphql(description = "The job's target keys, ascending")]
    pub target_keys: Vec<String>,
}

impl From<engine::AsyncJob> for AsyncJob {
    fn from(job: engine::AsyncJob) -> Self {
        AsyncJob {
            async_job_id: job.async_job_id,
            kind: job.kind,
            payload_version: i32::from(job.payload_version),
            payload: job.payload,
            idempotency_key: job.idempotency_key,
            effect_class: job.effect_class.into(),
            revision_mode: job.revision_mode.into(),
            retry_budget: i32::from(job.retry_budget),
            max_execution_claims: i32::from(job.max_execution_claims),
            max_reconciliation_attempts: i32::from(job.max_reconciliation_attempts),
            backoff_base_seconds: job.backoff_base_seconds,
            backoff_max_seconds: job.backoff_max_seconds,
            source_order: job.source_order.map(BigInt::from),
            source_fingerprint: job.source_fingerprint,
            source_ref: job.source_ref,
            status: job.status.into(),
            priority: i32::from(job.priority),
            available_at: timestamp(job.available_at),
            effect_phase: job.effect_phase.into(),
            attempt_count: job.attempt_count,
            execution_claim_count: i32::from(job.execution_claim_count),
            prewrite_failure_count: i32::from(job.prewrite_failure_count),
            reconciliation_attempt_count: i32::from(job.reconciliation_attempt_count),
            claimed_by: job.claimed_by,
            executor_principal: job.executor_principal,
            claimed_at: job.claimed_at.map(timestamp),
            lease_expires_at: job.lease_expires_at.map(timestamp),
            attempt_runtime_deadline: job.attempt_runtime_deadline.map(timestamp),
            first_claimed_at: job.first_claimed_at.map(timestamp),
            waiting_deadline: job.waiting_deadline.map(timestamp),
            checkpoint: job.checkpoint,
            checkpoint_version: job.checkpoint_version,
            provider_ref: job.provider_ref,
            terminal_at: job.terminal_at.map(timestamp),
            last_outcome_code: job.last_outcome_code,
            redacted_at: job.redacted_at.map(timestamp),
            lineage: job.lineage.into(),
            parent_job_id: job.parent_job_id,
            correlation_id: job.correlation_id,
            causation_ref: job.causation_ref,
            created_by: job.created_by,
            created_at: timestamp(job.created_at),
            updated_at: timestamp(job.updated_at),
            target_keys: job.target_keys,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(
    description = "One recorded attempt of an async job. No claim token is ever returned here"
)]
pub struct AsyncJobAttempt {
    #[graphql(description = "ID of the attempt")]
    pub async_job_attempt_id: Uuid,
    #[graphql(description = "The attempt's job")]
    pub async_job_id: Uuid,
    #[graphql(description = "Ordinal of the attempt within its job")]
    pub attempt_number: i32,
    #[graphql(description = "What the attempt was opened for")]
    pub purpose: AsyncAttemptPurpose,
    #[graphql(description = "The executor name, or the staff label for a staff attempt")]
    pub claimed_by: String,
    #[graphql(description = "The authenticated identity behind the attempt")]
    pub executor_principal: String,
    #[graphql(description = "The operator command behind a staff attempt")]
    pub control_command_id: Option<Uuid>,
    #[graphql(description = "When the attempt opened")]
    pub started_at: Timestamp,
    #[graphql(description = "When the attempt last renewed its lease")]
    pub last_heartbeat_at: Timestamp,
    #[graphql(description = "The job's effect phase when the attempt opened")]
    pub phase_at_start: AsyncEffectPhase,
    #[graphql(description = "When the attempt started the effect")]
    pub effect_started_at: Option<Timestamp>,
    #[graphql(description = "When the attempt recorded the provider's acknowledgement")]
    pub effect_confirmed_at: Option<Timestamp>,
    #[graphql(description = "When the attempt closed; null while it is open")]
    pub finished_at: Option<Timestamp>,
    #[graphql(description = "How the attempt closed")]
    pub result: Option<AsyncAttemptResult>,
    #[graphql(description = "The job's status after the attempt closed")]
    pub job_status_after: Option<AsyncJobStatus>,
    #[graphql(description = "The attempt's outcome code")]
    pub outcome_code: Option<String>,
    #[graphql(description = "The error code the executor reported")]
    pub error_code: Option<String>,
    #[graphql(description = "The error detail the executor reported, until redacted")]
    pub error_detail: Option<String>,
    #[graphql(description = "The provider reference the attempt recorded, until redacted")]
    pub provider_ref: Option<String>,
    #[graphql(
        description = "The evidence object the attempt closed with, as a JSON string, until redacted"
    )]
    pub evidence: Option<String>,
    #[graphql(description = "When the attempt's diagnostics were redacted")]
    pub redacted_at: Option<Timestamp>,
    #[graphql(description = "The operator command that redacted the attempt")]
    pub redaction_command_id: Option<Uuid>,
}

impl From<engine::AsyncJobAttempt> for AsyncJobAttempt {
    fn from(attempt: engine::AsyncJobAttempt) -> Self {
        AsyncJobAttempt {
            async_job_attempt_id: attempt.async_job_attempt_id,
            async_job_id: attempt.async_job_id,
            attempt_number: attempt.attempt_number,
            purpose: attempt.purpose.into(),
            claimed_by: attempt.claimed_by,
            executor_principal: attempt.executor_principal,
            control_command_id: attempt.control_command_id,
            started_at: timestamp(attempt.started_at),
            last_heartbeat_at: timestamp(attempt.last_heartbeat_at),
            phase_at_start: attempt.phase_at_start.into(),
            effect_started_at: attempt.effect_started_at.map(timestamp),
            effect_confirmed_at: attempt.effect_confirmed_at.map(timestamp),
            finished_at: attempt.finished_at.map(timestamp),
            result: attempt.result.map(AsyncAttemptResult::from),
            job_status_after: attempt.job_status_after.map(AsyncJobStatus::from),
            outcome_code: attempt.outcome_code,
            error_code: attempt.error_code,
            error_detail: attempt.error_detail,
            provider_ref: attempt.provider_ref,
            evidence: attempt.evidence,
            redacted_at: attempt.redacted_at.map(timestamp),
            redaction_command_id: attempt.redaction_command_id,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(description = "One status change of an async job")]
pub struct AsyncJobTransition {
    #[graphql(description = "ID of the transition, ascending in the order they happened")]
    pub async_job_transition_id: BigInt,
    #[graphql(description = "The transition's job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The status before; null for the job's creation")]
    pub from_status: Option<AsyncJobStatus>,
    #[graphql(description = "The status after")]
    pub to_status: AsyncJobStatus,
    #[graphql(description = "The engine transition that caused the change")]
    pub cause: String,
    #[graphql(description = "The attempt behind the change, if any")]
    pub async_job_attempt_id: Option<Uuid>,
    #[graphql(description = "The operator command behind the change, if any")]
    pub async_control_command_id: Option<Uuid>,
    #[graphql(description = "When the change happened")]
    pub occurred_at: Timestamp,
}

impl From<engine::AsyncJobTransition> for AsyncJobTransition {
    fn from(transition: engine::AsyncJobTransition) -> Self {
        AsyncJobTransition {
            async_job_transition_id: BigInt::from(transition.async_job_transition_id),
            async_job_id: transition.async_job_id,
            from_status: transition.from_status.map(AsyncJobStatus::from),
            to_status: transition.to_status.into(),
            cause: transition.cause,
            async_job_attempt_id: transition.async_job_attempt_id,
            async_control_command_id: transition.async_control_command_id,
            occurred_at: timestamp(transition.occurred_at),
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(description = "One async job with its attempts and its status changes")]
pub struct AsyncJobDetail {
    #[graphql(description = "The job")]
    pub job: AsyncJob,
    #[graphql(description = "The job's attempts, by attempt number")]
    pub attempts: Vec<AsyncJobAttempt>,
    #[graphql(description = "The job's status changes, in order")]
    pub transitions: Vec<AsyncJobTransition>,
}

impl AsyncJobDetail {
    /// The job, its attempts and its transitions, in the caller's
    /// transaction; `ASYNC_JOB_NOT_FOUND` when the job does not exist.
    pub(crate) fn read(conn: &mut PgConnection, async_job_id: Uuid) -> ThothResult<Self> {
        let job = get_job(conn, async_job_id)?;
        let attempts = job_attempts(conn, async_job_id)?;
        let transitions = job_transitions(conn, async_job_id)?;
        Ok(AsyncJobDetail {
            job: job.into(),
            attempts: attempts.into_iter().map(AsyncJobAttempt::from).collect(),
            transitions: transitions
                .into_iter()
                .map(AsyncJobTransition::from)
                .collect(),
        })
    }
}

impl From<engine::CompleteOutcome> for AsyncJob {
    /// `SUCCEEDED` (T6) or `RECONCILIATION_REQUIRED` (T6x): the job's status
    /// tells them apart.
    fn from(outcome: engine::CompleteOutcome) -> Self {
        match outcome {
            engine::CompleteOutcome::Succeeded(job)
            | engine::CompleteOutcome::ReconciliationRequired(job) => job.into(),
        }
    }
}

impl From<engine::WaitOutcome> for AsyncJob {
    /// `WAITING` (T10), or `RECONCILIATION_REQUIRED` or `FAILED` when the
    /// waiting deadline had passed (T10d): the job's status tells them apart.
    fn from(outcome: engine::WaitOutcome) -> Self {
        match outcome {
            engine::WaitOutcome::Waiting(job) | engine::WaitOutcome::DeadlineExceeded(job) => {
                job.into()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Operator outputs
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(
    description = "The recorded decision of an operator command. A refusal is a recorded result, not an error"
)]
pub struct AsyncJobCommandResult {
    #[graphql(description = "ID of the command record")]
    pub async_control_command_id: Uuid,
    #[graphql(description = "Which command was recorded")]
    pub command_kind: AsyncControlCommandKind,
    #[graphql(description = "Whether the command was applied or refused")]
    pub outcome: AsyncControlCommandOutcome,
    #[graphql(description = "The outcome code, which names the reason for a refusal")]
    pub outcome_code: String,
    #[graphql(description = "The command's job as it stands after the command")]
    pub job: AsyncJob,
    #[graphql(description = "The new job an applied retry created")]
    pub result_job: Option<AsyncJob>,
    #[graphql(
        description = "True when this is the stored decision of an earlier call with the same request ID and inputs"
    )]
    pub replayed: bool,
}

impl AsyncJobCommandResult {
    /// The command's decision, with the job a retry created read in the
    /// caller's transaction.
    pub(crate) fn read(
        conn: &mut PgConnection,
        command: engine::CommandResult,
    ) -> ThothResult<Self> {
        let result_job = command
            .result_job_id
            .map(|async_job_id| get_job(conn, async_job_id))
            .transpose()?;
        Ok(AsyncJobCommandResult {
            async_control_command_id: command.async_control_command_id,
            command_kind: command.command_kind.into(),
            outcome: command.outcome.into(),
            outcome_code: command.outcome_code,
            job: command.job.into(),
            result_job: result_job.map(AsyncJob::from),
            replayed: command.replayed,
        })
    }
}

// ---------------------------------------------------------------------------
// Operational signals (spec section 15)
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A ready job that waits because another job holds one of its targets")]
pub struct AsyncJobBlockedJob {
    #[graphql(description = "The waiting job")]
    pub async_job_id: Uuid,
    #[graphql(description = "The held target key")]
    pub target_key: String,
    #[graphql(description = "The job holding the key")]
    pub holder_job_id: Uuid,
    #[graphql(description = "The holding job's kind")]
    pub holder_kind: String,
}

impl From<engine::BlockedJob> for AsyncJobBlockedJob {
    fn from(blocked: engine::BlockedJob) -> Self {
        AsyncJobBlockedJob {
            async_job_id: blocked.async_job_id,
            target_key: blocked.target_key,
            holder_job_id: blocked.holder_job_id,
            holder_kind: blocked.holder_kind,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "A target with an accepted regression, awaiting an ordinary write that reaches the revision the regression replaced"
)]
pub struct AsyncJobRegressedTarget {
    #[graphql(description = "The target key")]
    pub target_key: String,
    #[graphql(description = "The applied revision order")]
    pub applied_order: Option<BigInt>,
    #[graphql(description = "The applied state's fingerprint")]
    pub applied_fingerprint: String,
    #[graphql(description = "When staff accepted the regression")]
    pub regressed_at: Timestamp,
    #[graphql(description = "The order the accepted regression replaced")]
    pub regressed_from_order: Option<BigInt>,
    #[graphql(description = "The fingerprint the accepted regression replaced")]
    pub regressed_from_fingerprint: Option<String>,
}

impl From<engine::RegressedTarget> for AsyncJobRegressedTarget {
    fn from(target: engine::RegressedTarget) -> Self {
        AsyncJobRegressedTarget {
            target_key: target.target_key,
            applied_order: target.applied_order.map(BigInt::from),
            applied_fingerprint: target.applied_fingerprint,
            regressed_at: timestamp(target.regressed_at),
            regressed_from_order: target.regressed_from_order.map(BigInt::from),
            regressed_from_fingerprint: target.regressed_from_fingerprint,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq)]
#[graphql(
    description = "The operational signals of one kind present in this server's registry or in the job table. Counts are exact and lists are bounded"
)]
pub struct AsyncJobKindSignals {
    #[graphql(description = "The kind")]
    pub kind: String,
    #[graphql(description = "Whether this server's registry knows the kind")]
    pub known: bool,
    #[graphql(description = "Jobs claimable now")]
    pub ready_count: BigInt,
    #[graphql(description = "When the oldest claimable job became claimable")]
    pub oldest_ready_at: Option<Timestamp>,
    #[graphql(description = "How long the oldest claimable job has been claimable, in seconds")]
    pub oldest_ready_age_seconds: Option<BigInt>,
    #[graphql(description = "Running jobs")]
    pub running_count: BigInt,
    #[graphql(description = "Running jobs past their lease")]
    pub running_past_lease_count: BigInt,
    #[graphql(description = "Jobs scheduled for another attempt")]
    pub retry_scheduled_count: BigInt,
    #[graphql(description = "Waiting jobs")]
    pub waiting_count: BigInt,
    #[graphql(description = "The earliest waiting deadline of a waiting job")]
    pub oldest_waiting_deadline: Option<Timestamp>,
    #[graphql(description = "Jobs that require reconciliation")]
    pub reconciliation_required_count: BigInt,
    #[graphql(
        description = "How long the oldest job requiring reconciliation has required it, in seconds"
    )]
    pub oldest_reconciliation_required_age_seconds: Option<BigInt>,
    #[graphql(
        description = "Unclaimed jobs requiring reconciliation at their reconciliation cap (RECONCILIATION_EXHAUSTED), which only staff can resolve"
    )]
    pub reconciliation_exhausted_count: BigInt,
    #[graphql(description = "Claimable jobs blocked by a target another job holds")]
    pub blocked_count: BigInt,
    #[graphql(description = "At most 100 of the blocked jobs, each with the key's holder")]
    pub blocked: Vec<AsyncJobBlockedJob>,
    #[graphql(description = "Jobs past their lease or waiting deadline for more than 10 minutes")]
    pub overdue_count: BigInt,
    #[graphql(description = "Jobs whose last recovery failed")]
    pub recovery_failure_count: BigInt,
    #[graphql(description = "When the oldest current recovery failure first happened")]
    pub oldest_recovery_failure_at: Option<Timestamp>,
    #[graphql(description = "The error codes of the current recovery failures")]
    pub recovery_failure_codes: Vec<String>,
    #[graphql(
        description = "Claimable jobs whose payload version no executor polled for in the last hour"
    )]
    pub ready_unpolled_count: BigInt,
    #[graphql(description = "Those jobs' payload versions")]
    pub unpolled_versions: Vec<i32>,
    #[graphql(description = "Unfinished jobs whose kind this server's registry does not know")]
    pub non_terminal_unknown_kind_count: BigInt,
    #[graphql(description = "Unfinished jobs whose payload version this server cannot execute")]
    pub non_terminal_version_not_executable_count: BigInt,
    #[graphql(description = "Targets last written by this kind with an accepted regression")]
    pub regressed_target_count: BigInt,
    #[graphql(description = "At most 100 of those targets, ascending by target key")]
    pub regressed_targets: Vec<AsyncJobRegressedTarget>,
    #[graphql(description = "Jobs that succeeded in the last 24 hours")]
    pub succeeded_last_24h: BigInt,
    #[graphql(description = "Jobs that failed in the last 24 hours")]
    pub failed_last_24h: BigInt,
    #[graphql(description = "Jobs cancelled in the last 24 hours")]
    pub cancelled_last_24h: BigInt,
    #[graphql(
        description = "The median duration of attempts finished in the last 24 hours, in seconds"
    )]
    pub attempt_duration_p50_seconds: Option<f64>,
    #[graphql(
        description = "The 95th percentile duration of attempts finished in the last 24 hours, in seconds"
    )]
    pub attempt_duration_p95_seconds: Option<f64>,
}

impl From<engine::KindSignals> for AsyncJobKindSignals {
    fn from(signals: engine::KindSignals) -> Self {
        AsyncJobKindSignals {
            kind: signals.kind,
            known: signals.known,
            ready_count: signals.ready_count.into(),
            oldest_ready_at: signals.oldest_ready_at.map(timestamp),
            oldest_ready_age_seconds: signals.oldest_ready_age_seconds.map(BigInt::from),
            running_count: signals.running_count.into(),
            running_past_lease_count: signals.running_past_lease_count.into(),
            retry_scheduled_count: signals.retry_scheduled_count.into(),
            waiting_count: signals.waiting_count.into(),
            oldest_waiting_deadline: signals.oldest_waiting_deadline.map(timestamp),
            reconciliation_required_count: signals.reconciliation_required_count.into(),
            oldest_reconciliation_required_age_seconds: signals
                .oldest_reconciliation_required_age_seconds
                .map(BigInt::from),
            reconciliation_exhausted_count: signals.reconciliation_exhausted_count.into(),
            blocked_count: signals.blocked_count.into(),
            blocked: signals
                .blocked
                .into_iter()
                .map(AsyncJobBlockedJob::from)
                .collect(),
            overdue_count: signals.overdue_count.into(),
            recovery_failure_count: signals.recovery_failure_count.into(),
            oldest_recovery_failure_at: signals.oldest_recovery_failure_at.map(timestamp),
            recovery_failure_codes: signals.recovery_failure_codes,
            ready_unpolled_count: signals.ready_unpolled_count.into(),
            unpolled_versions: signals
                .unpolled_versions
                .into_iter()
                .map(i32::from)
                .collect(),
            non_terminal_unknown_kind_count: signals.non_terminal_unknown_kind_count.into(),
            non_terminal_version_not_executable_count: signals
                .non_terminal_version_not_executable_count
                .into(),
            regressed_target_count: signals.regressed_target_count.into(),
            regressed_targets: signals
                .regressed_targets
                .into_iter()
                .map(AsyncJobRegressedTarget::from)
                .collect(),
            succeeded_last_24h: signals.succeeded_last_24h.into(),
            failed_last_24h: signals.failed_last_24h.into(),
            cancelled_last_24h: signals.cancelled_last_24h.into(),
            attempt_duration_p50_seconds: signals.attempt_duration_p50_seconds,
            attempt_duration_p95_seconds: signals.attempt_duration_p95_seconds,
        }
    }
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "When one executor last polled, claimed and renewed a lease for one kind")]
pub struct AsyncJobExecutorActivity {
    #[graphql(description = "The executor name")]
    pub executor_name: String,
    #[graphql(description = "The authenticated identity behind the executor")]
    pub executor_principal: String,
    #[graphql(description = "The kind")]
    pub kind: String,
    #[graphql(description = "When the executor last asked to claim jobs of the kind")]
    pub last_execution_poll_at: Option<Timestamp>,
    #[graphql(description = "When the executor last claimed a job of the kind")]
    pub last_execution_claim_at: Option<Timestamp>,
    #[graphql(description = "When the executor last asked to claim reconciliations of the kind")]
    pub last_reconciliation_poll_at: Option<Timestamp>,
    #[graphql(
        description = "The latest lease renewal of the executor's open attempts of the kind"
    )]
    pub latest_open_heartbeat_at: Option<Timestamp>,
}

impl From<engine::ExecutorActivity> for AsyncJobExecutorActivity {
    fn from(activity: engine::ExecutorActivity) -> Self {
        AsyncJobExecutorActivity {
            executor_name: activity.executor_name,
            executor_principal: activity.executor_principal,
            kind: activity.kind,
            last_execution_poll_at: activity.last_execution_poll_at.map(timestamp),
            last_execution_claim_at: activity.last_execution_claim_at.map(timestamp),
            last_reconciliation_poll_at: activity.last_reconciliation_poll_at.map(timestamp),
            latest_open_heartbeat_at: activity.latest_open_heartbeat_at.map(timestamp),
        }
    }
}

// ---------------------------------------------------------------------------
// The per-request claim memo (spec section 14.1)
// ---------------------------------------------------------------------------

/// The two claim fields. A request runs at most one claim, of either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncJobClaimField {
    /// `claimAsyncJobs`.
    Execution,
    /// `claimAsyncJobReconciliations`.
    Reconciliation,
}

/// The claim a request ran: its field, its arguments and its result, an
/// error result included.
struct StoredClaim {
    field: AsyncJobClaimField,
    input: ClaimAsyncJobsInput,
    result: FieldResult<AsyncJobClaimResult>,
}

/// One request's claim memo. `Context` is built per request, so the memo
/// never crosses requests.
///
/// The first claim field the executor runs, whichever it is, runs the claim
/// with the mutex held for the whole synchronous call and stores its field,
/// arguments and result. Every later claim field of the request waits for
/// the mutex, then returns the stored result when its field and arguments
/// are equal and is refused with `ASYNC_JOB_DUPLICATE_CLAIM` when they
/// differ, in both cases without running a claim. A mutex poisoned by a
/// panic in the first claim refuses every later claim field with
/// `ASYNC_JOB_BUSY`.
#[derive(Default)]
pub(crate) struct AsyncJobClaimMemo {
    claim: Mutex<Option<StoredClaim>>,
}

impl AsyncJobClaimMemo {
    /// Run `claim` if this is the request's first claim field; otherwise
    /// answer from the stored claim.
    pub(crate) fn claim_once(
        &self,
        field: AsyncJobClaimField,
        input: &ClaimAsyncJobsInput,
        claim: impl FnOnce() -> FieldResult<AsyncJobClaimResult>,
    ) -> FieldResult<AsyncJobClaimResult> {
        let Ok(mut stored) = self.claim.lock() else {
            return Err(ThothError::AsyncJobBusy.into_field_error());
        };
        if let Some(earlier) = stored.as_ref() {
            return if earlier.field == field && earlier.input == *input {
                earlier.result.clone()
            } else {
                Err(ThothError::AsyncJobDuplicateClaim.into_field_error())
            };
        }
        let result = claim();
        *stored = Some(StoredClaim {
            field,
            input: input.clone(),
            result: result.clone(),
        });
        result
    }

    /// True while a claim runs under the memo's mutex.
    #[cfg(test)]
    pub(crate) fn is_claiming(&self) -> bool {
        matches!(
            self.claim.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::thread;

    use super::*;

    fn input(kind: &str) -> ClaimAsyncJobsInput {
        ClaimAsyncJobsInput {
            claimed_by: "executor-1".to_string(),
            kinds: vec![AsyncJobKindVersionsInput {
                kind: kind.to_string(),
                payload_versions: vec![1],
            }],
            max_jobs: 1,
            lease_seconds: None,
        }
    }

    fn result(kind: &str) -> AsyncJobClaimResult {
        AsyncJobClaimResult {
            jobs: Vec::new(),
            skipped_kinds: vec![kind.to_string()],
        }
    }

    fn error_type(error: &juniper::FieldError) -> String {
        format!("{:?}", error.extensions())
    }

    /// Whichever field runs first claims; every later field answers from the
    /// memo, in either order of the two argument sets.
    #[test]
    fn the_first_claim_field_run_claims_and_later_fields_answer_from_the_memo() {
        for (first, second) in [("KIND_A", "KIND_B"), ("KIND_B", "KIND_A")] {
            let memo = AsyncJobClaimMemo::default();
            let runs = AtomicUsize::new(0);
            let claim = |kind: &'static str| {
                let runs = &runs;
                move || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    Ok(result(kind))
                }
            };
            let stored = memo
                .claim_once(AsyncJobClaimField::Execution, &input(first), claim(first))
                .expect("the first claim runs");
            assert_eq!(stored, result(first));
            // An identical field (a duplicated response key or an alias)
            // returns the stored result without running a claim.
            let repeated = memo
                .claim_once(AsyncJobClaimField::Execution, &input(first), claim(first))
                .expect("the stored result");
            assert_eq!(repeated, stored);
            // Other arguments, or the other claim field, are refused before
            // any claim runs.
            for (field, kind) in [
                (AsyncJobClaimField::Execution, second),
                (AsyncJobClaimField::Reconciliation, first),
                (AsyncJobClaimField::Reconciliation, second),
            ] {
                let refused = memo
                    .claim_once(field, &input(kind), claim(kind))
                    .expect_err("a different claim is refused");
                assert!(
                    error_type(&refused).contains("ASYNC_JOB_DUPLICATE_CLAIM"),
                    "{refused:?}"
                );
            }
            assert_eq!(runs.load(Ordering::SeqCst), 1, "{first} then {second}");
        }
    }

    #[test]
    fn a_failed_claim_is_stored_and_repeated_without_a_second_claim() {
        let memo = AsyncJobClaimMemo::default();
        let runs = AtomicUsize::new(0);
        let failing = || {
            runs.fetch_add(1, Ordering::SeqCst);
            Err(ThothError::AsyncJobKindNotAuthorized.into_field_error())
        };
        let first = memo
            .claim_once(
                AsyncJobClaimField::Reconciliation,
                &input("KIND_A"),
                failing,
            )
            .expect_err("the claim fails");
        let second = memo
            .claim_once(AsyncJobClaimField::Reconciliation, &input("KIND_A"), || {
                runs.fetch_add(1, Ordering::SeqCst);
                Ok(result("KIND_A"))
            })
            .expect_err("the stored error");
        assert_eq!(first, second);
        assert!(error_type(&second).contains("ASYNC_JOB_KIND_NOT_AUTHORIZED"));
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }

    /// The mutex is held for the whole claim: a second field waits for it and
    /// then answers from the memo. Synchronized by channels, never by sleeps.
    #[test]
    fn the_memo_mutex_is_held_for_the_whole_claim_and_a_later_field_waits_for_it() {
        let memo = Arc::new(AsyncJobClaimMemo::default());
        let runs = Arc::new(AtomicUsize::new(0));
        let (entered, claim_entered) = channel();
        let (release, claim_released) = channel::<()>();
        let first = {
            let (memo, runs) = (Arc::clone(&memo), Arc::clone(&runs));
            thread::spawn(move || {
                memo.claim_once(AsyncJobClaimField::Execution, &input("KIND_A"), || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    entered.send(()).expect("announce the claim");
                    claim_released.recv().expect("wait for the release");
                    Ok(result("KIND_A"))
                })
            })
        };
        claim_entered.recv().expect("the first claim runs");
        assert!(memo.is_claiming(), "the mutex is held while the claim runs");
        let (started, second_started) = channel();
        let second = {
            let (memo, runs) = (Arc::clone(&memo), Arc::clone(&runs));
            thread::spawn(move || {
                started.send(()).expect("announce the second field");
                memo.claim_once(AsyncJobClaimField::Execution, &input("KIND_A"), || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    Ok(result("KIND_B"))
                })
            })
        };
        second_started.recv().expect("the second field starts");
        assert!(
            memo.is_claiming(),
            "still held: the second field cannot claim"
        );
        release.send(()).expect("release the first claim");
        let first = first.join().expect("the first field").expect("claimed");
        let second = second.join().expect("the second field").expect("stored");
        assert_eq!(first, result("KIND_A"));
        assert_eq!(second, first);
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        assert!(!memo.is_claiming());
    }

    #[test]
    fn a_poisoned_memo_refuses_every_later_claim_field_as_busy_without_a_claim() {
        let memo = AsyncJobClaimMemo::default();
        let panicked = catch_unwind(AssertUnwindSafe(|| {
            memo.claim_once(AsyncJobClaimField::Execution, &input("KIND_A"), || {
                panic!("a claim that panics")
            })
        }));
        assert!(panicked.is_err());
        let runs = AtomicUsize::new(0);
        for (field, kind) in [
            (AsyncJobClaimField::Execution, "KIND_A"),
            (AsyncJobClaimField::Reconciliation, "KIND_B"),
        ] {
            let refused = memo
                .claim_once(field, &input(kind), || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    Ok(result(kind))
                })
                .expect_err("a poisoned memo refuses");
            assert!(
                error_type(&refused).contains("ASYNC_JOB_BUSY"),
                "{refused:?}"
            );
        }
        assert_eq!(runs.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn debug_never_prints_a_claim_token() {
        let token = Uuid::new_v4();
        let claim = AsyncJobClaimInput {
            async_job_id: Uuid::new_v4(),
            claim_token: token,
        };
        let job = ClaimedAsyncJob {
            async_job_id: Uuid::new_v4(),
            kind: "KIND_A".to_string(),
            payload_version: 1,
            payload: "{}".to_string(),
            source_order: None,
            source_fingerprint: None,
            source_ref: None,
            target_keys: Vec::new(),
            claim_token: token,
            lease_expires_at: timestamp(chrono::Utc::now()),
            attempt_runtime_deadline: timestamp(chrono::Utc::now()),
            attempt_number: 1,
            effect_phase: AsyncEffectPhase::PreWrite,
            checkpoint: None,
            checkpoint_version: 0,
            provider_ref: None,
        };
        let result = AsyncJobClaimResult {
            jobs: vec![job.clone()],
            skipped_kinds: Vec::new(),
        };
        for printed in [
            format!("{claim:?}"),
            format!("{job:?}"),
            format!("{result:?}"),
        ] {
            assert!(!printed.contains(&token.to_string()), "{printed}");
            assert!(printed.contains("<redacted>"), "{printed}");
        }
    }
}
