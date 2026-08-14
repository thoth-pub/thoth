//! Durable publisher back-catalogue distribution jobs (`BE-04`).
//!
//! This module owns the durable **job state** layer of the approved three-layer
//! model: work that must be performed, retried or audited. It owns neither
//! desired configuration (`BE-01`/`BE-02`/`BE-03`) nor observed delivery state,
//! which is deferred work that `BE-04` does not store and never infers.
//!
//! `BE-04` performs **no dissemination**. Nothing here contacts a distribution
//! platform, reads or writes a publication file, generates a feed or deposit, or
//! invokes an adapter; `thoth-dissemination` remains the execution engine and
//! consumes these jobs under its own separately specified `DIS-02` task.
//!
//! Everything in this module is **Publisher-Services-specific**
//! ([ADR-0008] section 3.4). There is deliberately no generic `Job`, `Queue`,
//! `Lease`, `Worker` or `ServiceRole` type, no trait abstracting job kinds across
//! programmes, no shared scheduler and no shared claim protocol. These tables,
//! types and lifecycle APIs must not be reused cross-programme by analogy: a
//! future reusable generic job or queue abstraction requires its own explicit
//! cross-programme ADR (ADR-0008 section 3.5).
//!
//! `Crud` is deliberately **not** implemented for any of the three entities:
//! there is no generic create/update/delete surface for durable jobs, and the
//! only supported writes are the named domain functions in [`crud`].
//!
//! [ADR-0008]: ../../../docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::Timestamp;

/// The hard maximum number of execution attempts one job may consume.
///
/// This is a **correctness property**, not a runtime knob: it is tied to the
/// migration's `distribution_job_attempt_count_check` upper bound, so raising
/// the budget is a migration *and* a constant change reviewed together.
pub(crate) const DISTRIBUTION_JOB_MAX_ATTEMPTS: i32 = 5;
/// Lease length granted when a worker requests none.
pub(crate) const DISTRIBUTION_JOB_LEASE_DEFAULT_SECONDS: i32 = 900;
/// Shortest lease a worker may be granted; shorter requests clamp up.
pub(crate) const DISTRIBUTION_JOB_LEASE_MIN_SECONDS: i32 = 60;
/// Longest lease a worker may be granted; longer requests clamp down.
pub(crate) const DISTRIBUTION_JOB_LEASE_MAX_SECONDS: i32 = 3600;
/// Batch size claimed when a worker requests none.
pub(crate) const DISTRIBUTION_JOB_CLAIM_DEFAULT_BATCH: i32 = 10;
/// Largest batch one claim call may take; larger requests clamp down.
pub(crate) const DISTRIBUTION_JOB_CLAIM_MAX_BATCH: i32 = 50;
/// Bounded amount of lease recovery one claim call performs.
pub(crate) const DISTRIBUTION_JOB_LEASE_RECOVERY_BATCH: i32 = 50;
/// First term of the retry backoff curve, in seconds.
pub(crate) const DISTRIBUTION_JOB_RETRY_BASE_SECONDS: i64 = 300;
/// Cap of the retry backoff curve, in seconds.
pub(crate) const DISTRIBUTION_JOB_RETRY_MAX_SECONDS: i64 = 21_600;
/// Maximum length of a worker-reported classification code.
pub(crate) const DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS: usize = 64;
/// Maximum length of a stored diagnostic, in Unicode scalar values.
pub(crate) const DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS: usize = 2048;

/// What kind of durable distribution work a job represents.
///
/// The inventory is closed. There is deliberately no `OTHER`, no `UNKNOWN` and
/// no `Default`: an unrecognised database, serde, string or GraphQL value must
/// fail rather than resolve to a nearest kind.
///
/// Exactly one value exists today. The enum exists so that **this programme's**
/// own deferred job kinds reuse one relation and one deduplication index under
/// their own approved tasks, and so the deduplication key is kind-scoped from
/// the first row. It is not an extension point for another programme.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Kind of durable distribution work a job represents"),
    ExistingTypePath = "crate::schema::sql_types::DistributionJobKind"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionJobKind {
    #[cfg_attr(
        feature = "backend",
        db_rename = "PUBLISHER_BACK_CATALOGUE",
        graphql(
            description = "Onboarding of a publisher's existing back catalogue to newly activated destinations"
        )
    )]
    PublisherBackCatalogue,
}

/// The lifecycle state of one durable distribution job.
///
/// The inventory is closed, with no `OTHER`, `UNKNOWN`, `NONE`, `NOT_STARTED`
/// or `Default`. In particular there is deliberately **no value meaning "no
/// job"**: a publisher with no durable job is represented by a null job and by
/// nothing else (specification section 17.3).
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Lifecycle state of one durable distribution job"),
    ExistingTypePath = "crate::schema::sql_types::DistributionJobStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionJobStatus {
    #[cfg_attr(
        feature = "backend",
        db_rename = "PENDING",
        graphql(description = "Durable work exists and is not claimed")
    )]
    Pending,
    #[cfg_attr(
        feature = "backend",
        db_rename = "RUNNING",
        graphql(description = "Claimed by one worker under a live or expired lease")
    )]
    Running,
    #[cfg_attr(
        feature = "backend",
        db_rename = "SUCCEEDED",
        graphql(description = "Terminal; a worker reported success")
    )]
    Succeeded,
    #[cfg_attr(
        feature = "backend",
        db_rename = "FAILED",
        graphql(
            description = "Terminal; a worker reported a non-retryable failure, or the attempt budget was exhausted"
        )
    )]
    Failed,
    #[cfg_attr(
        feature = "backend",
        db_rename = "CANCELLED",
        graphql(description = "Terminal; withdrawn administratively or by assignment disable")
    )]
    Cancelled,
}

impl DistributionJobStatus {
    /// Whether this status is terminal, so no further transition is permitted.
    pub(crate) fn is_terminal(self) -> bool {
        matches!(
            self,
            DistributionJobStatus::Succeeded
                | DistributionJobStatus::Failed
                | DistributionJobStatus::Cancelled
        )
    }
}

/// How one recorded execution attempt ended.
///
/// This is deliberately **not** [`DistributionJobStatus`] restricted to terminal
/// values: a job's status and an attempt's result are different facts. A
/// `FAILED` attempt routinely leaves the job `PENDING` for retry, and
/// `ABANDONED` — a lease that expired with no worker response — has no
/// job-status counterpart at all.
///
/// The value names **the event that closed the attempt**, which is what keeps
/// the four unambiguous.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "How one recorded execution attempt ended"),
    ExistingTypePath = "crate::schema::sql_types::DistributionJobAttemptResult"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionJobAttemptResult {
    #[cfg_attr(
        feature = "backend",
        db_rename = "SUCCEEDED",
        graphql(description = "The worker reported success")
    )]
    Succeeded,
    #[cfg_attr(
        feature = "backend",
        db_rename = "FAILED",
        graphql(description = "The worker reported a failure")
    )]
    Failed,
    #[cfg_attr(
        feature = "backend",
        db_rename = "CANCELLED",
        graphql(description = "A cancellation closed the attempt")
    )]
    Cancelled,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ABANDONED",
        graphql(description = "The lease expired with no worker response")
    )]
    Abandoned,
}

/// Why a distribution job was cancelled.
///
/// This exists so an operator can distinguish a deliberate administrative
/// cancellation from one caused by the publisher's own destination being
/// disabled. Without it the staff report cannot answer "why is this cancelled",
/// which is exactly the question a cancelled onboarding job raises.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Why a distribution job was cancelled"),
    ExistingTypePath = "crate::schema::sql_types::DistributionJobCancellationReason"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionJobCancellationReason {
    #[cfg_attr(
        feature = "backend",
        db_rename = "ADMINISTRATIVE",
        graphql(description = "An operator cancelled the job deliberately")
    )]
    Administrative,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ASSIGNMENT_DISABLED",
        graphql(
            description = "The publisher's assignment for a destination of this job was disabled"
        )
    )]
    AssignmentDisabled,
}

/// Whether a qualifying activation may create a durable distribution job.
///
/// `Off` does **not** mean "skip the job": a `SUPERUSER_API` configuration
/// transaction that would qualify for a job fails closed while this is `Off`
/// (specification section 9.4). Treating `Off` as "commit the activation, skip
/// the job" would strand that activation without an onboarding job for ever,
/// because nothing afterwards repairs it.
///
/// This is one named control for one behaviour. It is Publisher-Services-
/// specific and is **not** a feature-flag subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistributionJobCreation {
    #[default]
    Off,
    On,
}

impl DistributionJobCreation {
    /// Whether automatic creation is currently permitted.
    pub(crate) fn is_on(self) -> bool {
        self == DistributionJobCreation::On
    }
}

impl FromStr for DistributionJobCreation {
    type Err = String;

    /// Accepts exactly `OFF` and `ON` and fails on anything else.
    ///
    /// An unparseable value is never silently resolved to a nearest value: the
    /// process fails to start.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "OFF" => Ok(DistributionJobCreation::Off),
            "ON" => Ok(DistributionJobCreation::On),
            other => Err(format!(
                "Invalid distribution job creation setting: {other}. Expected OFF or ON."
            )),
        }
    }
}

impl std::fmt::Display for DistributionJobCreation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rendered = match self {
            DistributionJobCreation::Off => "OFF",
            DistributionJobCreation::On => "ON",
        };
        formatter.write_str(rendered)
    }
}

/// One durable unit of distribution work.
///
/// `claim_token` is a column of this row but is **never** exposed through any
/// GraphQL surface: it appears only on a freshly granted
/// [`ClaimedDistributionJob`]. Exposing it on the job type would let any caller
/// who can read a job steal the live claim, which would defeat the whole lease
/// model.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Queryable, diesel::QueryableByName),
    diesel(table_name = crate::schema::distribution_job)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionJob {
    pub distribution_job_id: Uuid,
    pub kind: DistributionJobKind,
    pub publisher_id: Uuid,
    /// Reserved for future work-level jobs. `BE-04` never populates it, and
    /// `distribution_job_back_catalogue_work_check` makes that a database
    /// property for the only current kind.
    pub work_id: Option<Uuid>,
    /// The `BE-02` activation this job was created for.
    ///
    /// This is a first-class column rather than only a component of the
    /// deduplication key, because claim eligibility needs it to decide whether
    /// a job's targets are still enabled *under the activation that created the
    /// job* — which is what stops a disable/re-enable cycle producing two live
    /// jobs for one destination.
    pub activation_id: Uuid,
    pub status: DistributionJobStatus,
    pub deduplication_key: String,
    pub attempt_count: i32,
    pub available_at: Timestamp,
    pub claim_token: Option<Uuid>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<Timestamp>,
    pub lease_expires_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub cancellation_reason: Option<DistributionJobCancellationReason>,
    /// The most recent **worker-reported** failure classification of this job.
    ///
    /// Set by `T3`/`T4`, cleared by `T2`, and left untouched by lease-expiry
    /// recovery (`T5a`, `T5b`) and by every form of cancellation (`T6`, `T7`,
    /// `T8`). It is therefore **not** a mirror of the newest attempt row: a job
    /// whose newest attempt is `ABANDONED` or `CANCELLED` carries whatever the
    /// last *reported* failure was, or nothing at all. A `FAILED` job no worker
    /// ever reported a failure for legitimately has `None` here, and that is
    /// correct rather than missing data. Attempt history is the authoritative
    /// record of how a job ended.
    pub last_error_code: Option<String>,
    /// The bounded sanitized diagnostic paired with [`Self::last_error_code`].
    pub last_error_detail: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DistributionJob {
    /// The deduplication key one `PUBLISHER_BACK_CATALOGUE` activation yields.
    ///
    /// This is the single site at which the key is computed, and
    /// `distribution_job_deduplication_key_formula_check` proves the stored
    /// value equals it, so a code defect that computed the wrong key fails the
    /// insert rather than silently creating a second job.
    pub(crate) fn back_catalogue_deduplication_key(
        publisher_id: Uuid,
        activation_id: Uuid,
    ) -> String {
        let kind = DistributionJobKind::PublisherBackCatalogue;
        format!("{kind}:{publisher_id}:{activation_id}")
    }
}

/// One destination of a distribution job.
///
/// Target rows are **immutable**: nothing updates or deletes one for any
/// lifecycle reason — not failure, not retry, not cancellation, not disabling
/// the assignment. They disappear only with the job row, by cascade.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Queryable, diesel::QueryableByName),
    diesel(table_name = crate::schema::distribution_job_target)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionJobTarget {
    pub distribution_job_id: Uuid,
    pub platform: DistributionPlatform,
    pub created_at: Timestamp,
}

/// One recorded execution attempt of a distribution job.
///
/// The row is inserted open and closed once; there is no third write, and
/// nothing deletes a row except the job cascade. `claim_token` is stored for
/// audit and to bind the attempt to exactly one claim for all time; it is
/// **never** exposed through any GraphQL surface.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Queryable, diesel::QueryableByName),
    diesel(table_name = crate::schema::distribution_job_attempt)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionJobAttempt {
    pub distribution_job_attempt_id: Uuid,
    pub distribution_job_id: Uuid,
    pub attempt_number: i32,
    pub claim_token: Uuid,
    pub claimed_by: String,
    pub started_at: Timestamp,
    pub finished_at: Option<Timestamp>,
    pub result: Option<DistributionJobAttemptResult>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
}

/// One durable job as the GraphQL `DistributionJob` type exposes it.
///
/// The nested `targets` and `attempts` collections are resolved by **two
/// different bounded mechanisms**, and which one applies is a property of the
/// producing path rather than of the field:
///
/// - the **worker claim path** pre-resolves both with its own set-based
///   statements, because that path must stay a constant four statements for a
///   claim of any size and deliberately does not use `RequestLoaders`;
/// - every other path leaves them absent, and the field resolvers batch through
///   the request-local `ADR-0007` loaders, which is what keeps the staff
///   report's statement count constant in the page size.
///
/// The claim token is **not** part of this type. It is returned only on
/// [`ClaimedDistributionJob`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionJobPayload {
    pub job: DistributionJob,
    pub preloaded_targets: Option<Vec<DistributionJobTarget>>,
    pub preloaded_attempts: Option<Vec<DistributionJobAttempt>>,
}

impl DistributionJobPayload {
    /// A payload whose children resolve through the request-local loaders.
    pub(crate) fn lazy(job: DistributionJob) -> Self {
        Self {
            job,
            preloaded_targets: None,
            preloaded_attempts: None,
        }
    }

    /// A payload whose children were already resolved set-based by the producing
    /// path.
    pub(crate) fn preloaded(
        job: DistributionJob,
        targets: Vec<DistributionJobTarget>,
        attempts: Vec<DistributionJobAttempt>,
    ) -> Self {
        Self {
            job,
            preloaded_targets: Some(targets),
            preloaded_attempts: Some(attempts),
        }
    }
}

/// A distribution job together with the claim it was just granted.
///
/// The claim token is returned **only** here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedDistributionJob {
    pub job: DistributionJobPayload,
    pub claim_token: Uuid,
    pub lease_expires_at: Timestamp,
    pub attempt_number: i32,
}

// ---------------------------------------------------------------------------
// Worker and operator inputs
// ---------------------------------------------------------------------------

/// How many jobs to claim, for how long, and of which kinds.
///
/// `limit` and `leaseSeconds` are **clamped rather than rejected**, and the
/// field descriptions say so: a long-running automated worker that asks for
/// slightly too much should still make bounded progress, because erroring would
/// put it into a retry loop that delivers nothing. Fail-closed applies to
/// authorization and to state transitions, not to a sizing argument.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "How many distribution jobs to claim, for how long, and of which kinds")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDistributionJobsInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = 10,
            description = "Maximum jobs to claim. Values above 50 are clamped to 50; values at or below 0 claim nothing"
        )
    )]
    pub limit: Option<i32>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = 900,
            description = "Requested lease duration in seconds, clamped to the range 60 to 3600"
        )
    )]
    pub lease_seconds: Option<i32>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = vec![],
            description = "If set, only claims jobs of these kinds. An empty list claims any kind"
        )
    )]
    pub kinds: Option<Vec<DistributionJobKind>>,
}

/// Which claimed job succeeded, and under which claim.
///
/// This deliberately carries **no** error fields. A success reports no error, so
/// this operation is structurally incapable of presenting a malformed
/// classification code, and no `errorCode` field is added to it for symmetry.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which claimed distribution job succeeded, and under which claim")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteDistributionJobInput {
    pub distribution_job_id: Uuid,
    pub claim_token: Uuid,
}

/// Which claimed job failed, how, and whether it may be retried.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which claimed distribution job failed, how, and whether to retry it")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailDistributionJobInput {
    pub distribution_job_id: Uuid,
    pub claim_token: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Stable machine-readable classification, matching ^[A-Z][A-Z0-9_]*$, at most 64 characters. A value outside that shape is rejected with INVALID_DISTRIBUTION_JOB_ERROR_CODE and changes no job or attempt state"
        )
    )]
    pub error_code: String,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Bounded human-readable diagnostic, truncated to 2048 characters. Must contain no credential, token, signed URL, response body or personal data"
        )
    )]
    pub error_detail: Option<String>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = true,
            description = "Whether the failure may be retried. A retryable failure returns the job to PENDING until the attempt budget is exhausted"
        )
    )]
    pub retryable: bool,
}

/// Which job to withdraw administratively.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which distribution job to cancel")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelDistributionJobInput {
    pub distribution_job_id: Uuid,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
mod tests;
