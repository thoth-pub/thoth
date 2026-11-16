//! The protected API's side of the async job engine
//! (THOTH-ASYNC-01-ENGINE-01B, spec sections 14 and 16): the code-owned
//! executor map, the adapters the GraphQL resolvers call, and the access-log
//! body predicate.
//!
//! Nothing here decides an engine rule. The map turns an authenticated
//! machine role into the engine's [`Executor`] identity, the adapters convert
//! API values into engine values and back, and the predicate decides which
//! `/graphql` request bodies the API server's access log omits.

use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::registry::{AsyncJobRegistry, ExecutorDomain, RiskClass};
use super::signals::{executor_activity, kind_signals, ExecutorActivity, KindSignals};
use super::{Executor, RevisionMode, WorkerClaim};
use crate::graphql::GraphQLRequest;
use crate::model::Timestamp;
use crate::policy::{PolicyContext, Role};

// ---------------------------------------------------------------------------
// The executor map (spec 14.1)
// ---------------------------------------------------------------------------

/// The capability the dissemination executor holds.
pub const DISSEMINATION_CAPABILITY: &str = "dissemination";

/// What the executor map gives an authenticated machine role: the executor
/// domain, risk class, capabilities and accepted revision modes. They never
/// come from request input; a request only narrows kinds and payload
/// versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorProfile {
    pub domain: ExecutorDomain,
    pub risk_class: RiskClass,
    pub capabilities: &'static [&'static str],
    pub revision_modes: &'static [RevisionMode],
}

/// The code-owned executor map: a function over the existing [`Role`] enum
/// that adds no role variant.
///
/// Release N maps exactly one role, `DISSEMINATION_WORKER`, to the
/// `Dissemination` domain with risk class `Trusted`, capability
/// `dissemination` and revision modes `CURRENT_STATE` and `REVISION_BOUND`.
/// Every other role is no executor: `SUPERUSER` never satisfies a machine
/// role (ADR-0008), and `ThothWorker` kinds have no role at all, so they are
/// never claimable through GraphQL. The match names every variant, so a new
/// role must be placed here explicitly.
pub(crate) fn executor_profile(role: Role) -> Option<ExecutorProfile> {
    match role {
        Role::DisseminationWorker => Some(ExecutorProfile {
            domain: ExecutorDomain::Dissemination,
            risk_class: RiskClass::Trusted,
            capabilities: &[DISSEMINATION_CAPABILITY],
            revision_modes: &[RevisionMode::CurrentState, RevisionMode::RevisionBound],
        }),
        Role::Superuser
        | Role::PublisherAdmin
        | Role::PublisherUser
        | Role::WorkLifecycle
        | Role::CdnWrite
        | Role::MetricsIngestService
        | Role::MetricsReadService => None,
    }
}

/// The profile of the role that `require_dissemination_worker` admits. A map
/// without it fails closed, as an unauthorized caller.
pub(crate) fn dissemination_profile() -> ThothResult<ExecutorProfile> {
    executor_profile(Role::DisseminationWorker).ok_or(ThothError::Unauthorised)
}

/// Authorize an executor operation and return the caller's profile.
///
/// Every executor operation calls this before the engine runs. Release N's
/// one executor role is `DISSEMINATION_WORKER`, checked by the existing
/// `require_dissemination_worker` guard: an anonymous caller, `SUPERUSER`,
/// the Metrics machine roles and every publisher-scoped role are refused
/// with `Unauthorized`. The engine then checks the profile's domain against
/// each kind's executor or reconciler domain.
pub(crate) fn authorize_executor<C: PolicyContext>(context: &C) -> ThothResult<ExecutorProfile> {
    context.require_dissemination_worker()?;
    dissemination_profile()
}

impl ExecutorProfile {
    /// The engine executor of a claim: `name` is the request's `claimedBy`
    /// (the executor name) and `principal` the authenticated token's user id.
    pub(crate) fn executor(&self, name: &str, principal: &str) -> Executor {
        Executor {
            name: name.to_string(),
            principal: principal.to_string(),
            domain: self.domain,
            risk_class: self.risk_class,
            capabilities: self
                .capabilities
                .iter()
                .map(|capability| capability.to_string())
                .collect(),
            revision_modes: self.revision_modes.to_vec(),
        }
    }

    /// What a token-fenced write presents: the job and the claim token from
    /// the request, and the caller's domain from the map. The engine checks
    /// that domain against the job's kind on every call, so a token leaked
    /// to another domain's executor writes nothing.
    pub(crate) fn worker_claim(&self, async_job_id: Uuid, claim_token: Uuid) -> WorkerClaim {
        WorkerClaim {
            async_job_id,
            claim_token,
            domain: self.domain,
        }
    }
}

// ---------------------------------------------------------------------------
// Adapters
// ---------------------------------------------------------------------------

/// The `Timestamp` scalar of an engine instant.
pub(crate) fn timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp(value)
}

/// The engine instant of a `Timestamp` input.
pub(crate) fn instant(value: Timestamp) -> DateTime<Utc> {
    value.0
}

/// Payload versions from the API's `Int` to the engine's `smallint`. A value
/// outside the `smallint` range names no version any kind can declare, and is
/// `ASYNC_JOB_PAYLOAD_INVALID`.
pub(crate) fn payload_versions(values: &[i32]) -> ThothResult<Vec<i16>> {
    values
        .iter()
        .map(|value| {
            i16::try_from(*value).map_err(|_| {
                ThothError::AsyncJobPayloadInvalid(
                    "a payload version is outside the smallint range".to_string(),
                )
            })
        })
        .collect()
}

/// The signals of the requested kinds, or of every kind present in the
/// registry or in the table when `kinds` is empty (spec 15). A requested kind
/// that is in neither has no signals and is not listed.
pub(crate) fn kind_signals_of(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    kinds: &[String],
) -> ThothResult<Vec<KindSignals>> {
    let signals = kind_signals(conn, registry)?;
    if kinds.is_empty() {
        return Ok(signals);
    }
    Ok(signals
        .into_iter()
        .filter(|signal| kinds.contains(&signal.kind))
        .collect())
}

/// The executor activity rows (spec 15) with at least one recorded instant
/// at or after `since`; every row when `since` is absent.
pub(crate) fn executor_activity_since(
    conn: &mut PgConnection,
    since: Option<DateTime<Utc>>,
) -> ThothResult<Vec<ExecutorActivity>> {
    let rows = executor_activity(conn)?;
    let Some(since) = since else {
        return Ok(rows);
    };
    Ok(rows
        .into_iter()
        .filter(|row| {
            [
                row.last_execution_poll_at,
                row.last_execution_claim_at,
                row.last_reconciliation_poll_at,
                row.latest_open_heartbeat_at,
            ]
            .into_iter()
            .flatten()
            .any(|at| at >= since)
        })
        .collect())
}

// ---------------------------------------------------------------------------
// The access-log body rule (spec 16)
// ---------------------------------------------------------------------------

/// The substring every async job query and mutation field name contains, and
/// no other `QueryRoot` or `MutationRoot` field name does.
pub const ASYNC_JOB_FIELD_MARKER: &str = "syncJob";

/// What the API server's access log writes in place of an omitted body.
pub const OMITTED_REQUEST_BODY: &str = "[async job operation body omitted]";

/// Whether the access log must omit this `/graphql` request body.
///
/// The body is decoded into [`GraphQLRequest`], the type the `/graphql`
/// handler deserializes, so every body shape the handler accepts (a
/// positional JSON array included) is matched on the query text the server
/// would execute, and a JSON escape such as `claimAsyncJobs` cannot
/// hide a field name. GraphQL names have no escape syntax, so aliases and
/// fragments still contain the field name. A body that type cannot decode is
/// refused by the handler and runs no operation; it is omitted when its raw
/// text contains the marker, so a malformed body around a live claim token is
/// not logged either. Every other body is logged exactly as before.
pub fn access_log_omits_body(body: &[u8]) -> bool {
    match serde_json::from_slice::<GraphQLRequest>(body) {
        Ok(request) => request.query.contains(ASYNC_JOB_FIELD_MARKER),
        Err(_) => String::from_utf8_lossy(body).contains(ASYNC_JOB_FIELD_MARKER),
    }
}
