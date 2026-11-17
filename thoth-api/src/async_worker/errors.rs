//! The worker's own reading of engine errors (spec section 6.4): a fixed code
//! for every `ThothError` the engine answers, the refusal reason of an
//! `ASYNC_JOB_TRANSITION_REFUSED`, and the class that decides what the
//! runtime does next. It uses no test-only engine accessor, and nothing here
//! returns free text: a refusal reason the worker does not know is `OTHER`.

use thoth_errors::ThothError;

/// The fixed code of an engine error, as logs and fallbacks record it.
pub fn error_code(error: &ThothError) -> &'static str {
    match error {
        ThothError::AsyncJobNotFound => "ASYNC_JOB_NOT_FOUND",
        ThothError::AsyncJobKindUnknown => "ASYNC_JOB_KIND_UNKNOWN",
        ThothError::AsyncJobKindAdmissionClosed => "ASYNC_JOB_KIND_ADMISSION_CLOSED",
        ThothError::AsyncJobKindNotAuthorized => "ASYNC_JOB_KIND_NOT_AUTHORIZED",
        ThothError::AsyncJobPayloadInvalid(_) => "ASYNC_JOB_PAYLOAD_INVALID",
        ThothError::AsyncJobPayloadVersionUnsupported => "ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED",
        ThothError::AsyncJobIdempotencyConflict => "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
        ThothError::AsyncJobStaleClaim => "ASYNC_JOB_STALE_CLAIM",
        ThothError::AsyncJobLeaseExpired => "ASYNC_JOB_LEASE_EXPIRED",
        ThothError::AsyncJobTransitionRefused(_) => "ASYNC_JOB_TRANSITION_REFUSED",
        ThothError::AsyncJobEffectWindowInsufficient => "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
        ThothError::AsyncJobEvidenceInvalid(_) => "ASYNC_JOB_EVIDENCE_INVALID",
        ThothError::AsyncJobIsolationInvalid => "ASYNC_JOB_ISOLATION_INVALID",
        ThothError::AsyncJobIntegrityViolation => "ASYNC_JOB_INTEGRITY_VIOLATION",
        ThothError::AsyncJobBusy => "ASYNC_JOB_BUSY",
        ThothError::AsyncJobDuplicateClaim => "ASYNC_JOB_DUPLICATE_CLAIM",
        ThothError::AsyncCommandRequestConflict => "ASYNC_COMMAND_REQUEST_CONFLICT",
        _ => "OTHER_ERROR",
    }
}

/// The refusal reasons the engine records (`AsyncJobTransitionRefused`),
/// mapped to static strings so that no engine text is ever copied.
const REFUSAL_REASONS: &[&str] = &[
    "PHASE",
    "KIND_POLICY",
    "CLASS",
    "EFFECT_CONFIRMED",
    "RECONCILIATION_IN_PROGRESS",
    "NOT_RECONCILIATION_REQUIRED",
    "DEADLINE_EXCEEDED",
    "EVIDENCE_INVALID",
    "KIND_UNKNOWN",
    "NOT_A_REGRESSION",
    "NOT_CANCELLABLE",
    "OUTCOME_NOT_ALLOWED",
    "RETRY_NOT_ALLOWED",
];

/// The refusal reason of an `ASYNC_JOB_TRANSITION_REFUSED`, as a static
/// string (`OTHER` for a reason this binary does not know); `None` for every
/// other error.
pub fn refusal_reason(error: &ThothError) -> Option<&'static str> {
    match error {
        ThothError::AsyncJobTransitionRefused(reason) => Some(
            REFUSAL_REASONS
                .iter()
                .copied()
                .find(|known| *known == reason.as_str())
                .unwrap_or("OTHER"),
        ),
        _ => None,
    }
}

/// True for an `ASYNC_JOB_TRANSITION_REFUSED` with reason `PHASE`.
pub fn is_phase_refusal(error: &ThothError) -> bool {
    refusal_reason(error) == Some("PHASE")
}

/// What an engine answer to a final action means for the runtime (spec
/// section 6.4 steps 4 and 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// `ASYNC_JOB_BUSY`: retry.
    Busy,
    /// `ASYNC_JOB_STALE_CLAIM`: the token is no longer the job's.
    StaleClaim,
    /// `ASYNC_JOB_LEASE_EXPIRED`.
    LeaseExpired,
    /// `ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT`.
    WindowInsufficient,
    /// The engine refused the action as invalid: a handler fault
    /// (`ASYNC_JOB_TRANSITION_REFUSED`, `ASYNC_JOB_PAYLOAD_INVALID`,
    /// `ASYNC_JOB_EVIDENCE_INVALID`).
    ActionInvalid,
    /// `ASYNC_JOB_KIND_NOT_AUTHORIZED`: a runtime fault that also stops the
    /// worker with exit code 1.
    NotAuthorized,
    /// Any other answer: a runtime or database fault, not the handler's.
    Fault,
}

/// The class of an engine error.
pub fn classify(error: &ThothError) -> ErrorClass {
    match error {
        ThothError::AsyncJobBusy => ErrorClass::Busy,
        ThothError::AsyncJobStaleClaim => ErrorClass::StaleClaim,
        ThothError::AsyncJobLeaseExpired => ErrorClass::LeaseExpired,
        ThothError::AsyncJobEffectWindowInsufficient => ErrorClass::WindowInsufficient,
        ThothError::AsyncJobTransitionRefused(_)
        | ThothError::AsyncJobPayloadInvalid(_)
        | ThothError::AsyncJobEvidenceInvalid(_) => ErrorClass::ActionInvalid,
        ThothError::AsyncJobKindNotAuthorized => ErrorClass::NotAuthorized,
        _ => ErrorClass::Fault,
    }
}

/// The code a log line records for an error: the refusal reason after the
/// engine code when there is one.
pub fn log_code(error: &ThothError) -> String {
    match refusal_reason(error) {
        Some(reason) => format!("{}({reason})", error_code(error)),
        None => error_code(error).to_string(),
    }
}
