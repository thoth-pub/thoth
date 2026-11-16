//! `THOTH-ASYNC-01-ENGINE-01B` (E2) evidence: the protected GraphQL API of
//! the async job engine (specification sections 14 to 17 and the E2 items of
//! section 19.2).
//!
//! These tests run the real production schema and resolvers against the
//! disposable test database, with test kinds injected through
//! [`Context::with_async_registry`]. Every request gets a fresh `Context`, as
//! production builds one per request, so no claim memo crosses requests.
//! Engine fixtures run in their own engine transactions on a dedicated
//! connection. The one concurrency test synchronizes with a held advisory
//! lock and bounded `pg_locks` polling, never with a sleep.

#![cfg(all(test, feature = "backend"))]

use std::collections::{BTreeSet, HashMap};
use std::ptr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration as StdDuration, Instant};

use chrono::{Duration, SecondsFormat, Utc};
use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::sql_types::{BigInt as SqlBigInt, Nullable, Text};
use diesel::{sql_query, Connection, QueryableByName, RunQueryDsl};
use juniper::{DefaultScalarValue, FromInputValue, InputValue, IntoFieldError, ToInputValue};
use regex::Regex;
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::types::async_job as api;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::async_job::executor::{
    executor_profile, ExecutorProfile, ASYNC_JOB_FIELD_MARKER, DISSEMINATION_CAPABILITY,
};
use crate::model::async_job::{
    claim_on, claim_reconciliation_on, complete_on, create_job_on, fail_on, mark_effect_started_on,
    renew_on, with_engine_transaction_on, AsyncAttemptPurpose, AsyncAttemptResult,
    AsyncControlCommandKind, AsyncControlCommandOutcome, AsyncEffectPhase, AsyncJob,
    AsyncJobKindPolicy, AsyncJobLineage, AsyncJobRegistry, AsyncJobStatus, CancelPolicy,
    ClaimRequest, CompleteOutcome, CompleteRequest, CompletionOutcome, EffectClass,
    EvidenceContext, EvidenceEntry, EvidenceUse, Executor, ExecutorDomain, FailRequest,
    FailureClass, NewAsyncJob, PayloadError, ReconcilerOutcome, RevisionMode, RiskClass,
    StaffOutcome, WorkerClaim, NS_KIND, PRODUCTION_REGISTRY,
};
use crate::model::tests::db as test_db;
use crate::policy::Role;

// ---------------------------------------------------------------------------
// Test kinds and registries
// ---------------------------------------------------------------------------

/// Effectful, CURRENT_STATE, executed and reconciled by `Dissemination`.
const API_EFFECT: &str = "API_EFFECT";
/// Effectful, REVISION_BOUND, listing `ACCEPT_REGRESSION` for staff.
const API_REVISION: &str = "API_REVISION";
/// Requires the `dissemination` capability, which only the executor map gives.
const API_CAPABLE: &str = "API_CAPABLE";
/// Untrusted content, which the map's `Trusted` executor never runs.
const API_UNTRUSTED: &str = "API_UNTRUSTED";
/// Executed by `Dissemination`, reconciled by the in-process `ThothWorker`.
const API_SPLIT: &str = "API_SPLIT";
/// Executed and reconciled by the in-process `ThothWorker` only.
const API_LOCAL: &str = "API_LOCAL";

const NS_CS: &str = "api_cs";
const NS_RB: &str = "api_rb";
const NS_LOCAL: &str = "api_local";

/// The payload must carry a non-empty string `id`.
fn validate_payload(_version: i16, payload: &JsonValue) -> Result<(), PayloadError> {
    match payload.get("id").and_then(JsonValue::as_str) {
        Some(id) if !id.is_empty() => Ok(()),
        _ => Err(PayloadError::new("the payload needs an id")),
    }
}

/// `id`, or `id@rev` when the payload names a revision.
fn effect_identity(_version: i16, payload: &JsonValue) -> Result<String, PayloadError> {
    let id = payload
        .get("id")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    Ok(match payload.get("rev") {
        Some(rev) => format!("{id}@{rev}"),
        None => id.to_string(),
    })
}

/// The payload's `targets` array of strings.
fn targets(_version: i16, payload: &JsonValue) -> Result<Vec<String>, PayloadError> {
    Ok(payload
        .get("targets")
        .and_then(JsonValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

/// Applied evidence is optional; when present it is `{"valid": true}` and,
/// for a REVISION_BOUND job, names the job's own `order`. No-effect evidence
/// is `{"valid": true, "providerRef": "..."}`. Not-applied evidence is
/// `{"notApplied": true}`.
fn validate_evidence(
    use_: EvidenceUse,
    context: &EvidenceContext<'_>,
    evidence: &JsonValue,
) -> Result<(), PayloadError> {
    let valid = evidence.get("valid") == Some(&JsonValue::Bool(true));
    match use_ {
        EvidenceUse::Applied if evidence.is_null() => Ok(()),
        EvidenceUse::Applied => {
            let names_own_revision = match context.source_order {
                Some(order) => evidence.get("order").and_then(JsonValue::as_i64) == Some(order),
                None => true,
            };
            if valid && names_own_revision {
                Ok(())
            } else {
                Err(PayloadError::new("the applied evidence is not valid"))
            }
        }
        EvidenceUse::NoEffect => {
            let names_provider = evidence
                .get("providerRef")
                .and_then(JsonValue::as_str)
                .is_some();
            if valid && names_provider {
                Ok(())
            } else {
                Err(PayloadError::new(
                    "no-effect evidence needs a provider reference",
                ))
            }
        }
        EvidenceUse::NotApplied => {
            if evidence.get("notApplied") == Some(&JsonValue::Bool(true)) {
                Ok(())
            } else {
                Err(PayloadError::new("not-applied evidence is not valid"))
            }
        }
    }
}

const STAFF_OUTCOMES: &[StaffOutcome] = &[
    StaffOutcome::Succeeded,
    StaffOutcome::Failed,
    StaffOutcome::Cancelled,
    StaffOutcome::Waiting,
];

const API_EFFECT_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_EFFECT,
    owning_domain: "api_test",
    executor_domain: ExecutorDomain::Dissemination,
    reconciler_domain: ExecutorDomain::Dissemination,
    risk_class: RiskClass::Trusted,
    required_capabilities: &[],
    creatable_versions: &[1],
    executable_versions: &[1],
    effect_class: EffectClass::Effectful,
    revision_mode: RevisionMode::CurrentState,
    target_namespaces: &[NS_CS],
    admission_open: true,
    retry_budget: 3,
    max_execution_claims: 4,
    max_reconciliation_attempts: 2,
    backoff_base_seconds: 10,
    backoff_max_seconds: 300,
    poll_interval_min_seconds: 5,
    lease_default_seconds: 60,
    lease_min_seconds: 15,
    lease_max_seconds: 300,
    max_attempt_runtime_seconds: 600,
    effect_window_seconds: Some(30),
    waiting_horizon_seconds: 3600,
    concurrency_limit: 10,
    reconciliation_concurrency_limit: 5,
    max_targets: 4,
    cancellable: CancelPolicy::ALL,
    manual_retry_allowed: true,
    no_effect_after_start_allowed: true,
    confirmed_effect_resumable: false,
    staff_reconciliation_outcomes: STAFF_OUTCOMES,
    validate_payload,
    effect_identity,
    targets,
    validate_evidence,
};

const API_REVISION_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_REVISION,
    revision_mode: RevisionMode::RevisionBound,
    target_namespaces: &[NS_RB],
    max_targets: 1,
    staff_reconciliation_outcomes: &[
        StaffOutcome::Succeeded,
        StaffOutcome::Failed,
        StaffOutcome::Cancelled,
        StaffOutcome::Waiting,
        StaffOutcome::AcceptRegression,
    ],
    ..API_EFFECT_POLICY
};

const API_CAPABLE_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_CAPABLE,
    required_capabilities: &[DISSEMINATION_CAPABILITY],
    ..API_EFFECT_POLICY
};

const API_UNTRUSTED_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_UNTRUSTED,
    risk_class: RiskClass::UntrustedContent,
    ..API_EFFECT_POLICY
};

const API_SPLIT_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_SPLIT,
    reconciler_domain: ExecutorDomain::ThothWorker,
    ..API_EFFECT_POLICY
};

const API_LOCAL_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: API_LOCAL,
    executor_domain: ExecutorDomain::ThothWorker,
    reconciler_domain: ExecutorDomain::ThothWorker,
    target_namespaces: &[NS_LOCAL],
    ..API_EFFECT_POLICY
};

/// The registry every test request serves with.
static API_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[
    API_EFFECT_POLICY,
    API_REVISION_POLICY,
    API_CAPABLE_POLICY,
    API_UNTRUSTED_POLICY,
    API_SPLIT_POLICY,
    API_LOCAL_POLICY,
]);

/// A binary that knows only `API_REVISION`.
static NARROW_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[API_REVISION_POLICY]);

// ---------------------------------------------------------------------------
// Callers
// ---------------------------------------------------------------------------

const WORKER_ID: &str = "machine:dissemination-1";
const SECOND_WORKER_ID: &str = "machine:dissemination-2";
const STAFF_ID: &str = "staff:operator-1";

const PUBLISHER_ROLES: [Role; 4] = [
    Role::PublisherAdmin,
    Role::PublisherUser,
    Role::WorkLifecycle,
    Role::CdnWrite,
];

/// A caller, described rather than built: `IntrospectedUser` is not `Clone`,
/// so each request builds its own.
#[derive(Debug, Clone)]
struct Caller {
    label: &'static str,
    id: Option<&'static str>,
    roles: Vec<Role>,
}

impl Caller {
    fn new(label: &'static str, id: &'static str, roles: &[Role]) -> Self {
        Caller {
            label,
            id: Some(id),
            roles: roles.to_vec(),
        }
    }

    fn anonymous() -> Self {
        Caller {
            label: "an unauthenticated caller",
            id: None,
            roles: Vec::new(),
        }
    }

    fn user(&self) -> Option<IntrospectedUser> {
        self.id.map(|id| user(id, &self.roles))
    }
}

/// A user holding `roles`, each scoped to `org-1`.
fn user(id: &str, roles: &[Role]) -> IntrospectedUser {
    let mut project_roles: HashMap<String, HashMap<String, String>> = HashMap::new();
    for role in roles {
        project_roles
            .entry(role.as_ref().to_string())
            .or_default()
            .insert("org-1".to_string(), "role".to_string());
    }
    IntrospectedUser {
        user_id: id.to_string(),
        username: None,
        name: None,
        given_name: None,
        family_name: None,
        preferred_username: None,
        email: None,
        email_verified: None,
        locale: None,
        project_roles: Some(project_roles),
        metadata: None,
    }
}

fn worker() -> Caller {
    Caller::new(
        "DISSEMINATION_WORKER",
        WORKER_ID,
        &[Role::DisseminationWorker],
    )
}

fn staff() -> Caller {
    Caller::new("SUPERUSER", STAFF_ID, &[Role::Superuser])
}

/// A superuser who also holds the machine role: both checks pass for it.
fn superworker() -> Caller {
    Caller::new(
        "SUPERUSER and DISSEMINATION_WORKER",
        "staff:superworker",
        &[Role::Superuser, Role::DisseminationWorker],
    )
}

/// Callers refused by every new operation: no authentication, no role, each
/// publisher-scoped role, all of them, and each Metrics machine role.
fn outsiders() -> Vec<Caller> {
    vec![
        Caller::anonymous(),
        Caller::new("an authenticated user with no role", "user:no-role", &[]),
        Caller::new(
            "PUBLISHER_USER",
            "user:publisher-user",
            &[Role::PublisherUser],
        ),
        Caller::new(
            "PUBLISHER_ADMIN",
            "user:publisher-admin",
            &[Role::PublisherAdmin],
        ),
        Caller::new(
            "WORK_LIFECYCLE",
            "user:work-lifecycle",
            &[Role::WorkLifecycle],
        ),
        Caller::new("CDN_WRITE", "user:cdn-write", &[Role::CdnWrite]),
        Caller::new(
            "every publisher-scoped role",
            "user:every-publisher-role",
            &PUBLISHER_ROLES,
        ),
        Caller::new(
            "METRICS_INGEST_SERVICE",
            "machine:metrics-ingest",
            &[Role::MetricsIngestService],
        ),
        Caller::new(
            "METRICS_READ_SERVICE",
            "machine:metrics-read",
            &[Role::MetricsReadService],
        ),
    ]
}

/// Callers refused by every executor operation: the outsiders and
/// `SUPERUSER`, alone and with every role but `DISSEMINATION_WORKER`.
fn refused_executors() -> Vec<Caller> {
    let mut callers = outsiders();
    callers.push(staff());
    callers.push(Caller::new(
        "SUPERUSER with every role but DISSEMINATION_WORKER",
        "staff:every-role-but-worker",
        &[
            Role::Superuser,
            Role::PublisherAdmin,
            Role::PublisherUser,
            Role::WorkLifecycle,
            Role::CdnWrite,
            Role::MetricsIngestService,
            Role::MetricsReadService,
        ],
    ));
    callers
}

/// Callers refused by every operator command and read: the outsiders and
/// `DISSEMINATION_WORKER`, alone and with every role but `SUPERUSER`.
fn refused_operators() -> Vec<Caller> {
    let mut callers = outsiders();
    callers.push(worker());
    callers.push(Caller::new(
        "DISSEMINATION_WORKER with every role but SUPERUSER",
        "machine:every-role-but-superuser",
        &[
            Role::DisseminationWorker,
            Role::PublisherAdmin,
            Role::PublisherUser,
            Role::WorkLifecycle,
            Role::CdnWrite,
            Role::MetricsIngestService,
            Role::MetricsReadService,
        ],
    ));
    callers
}

// ---------------------------------------------------------------------------
// Requests and responses
// ---------------------------------------------------------------------------

/// A context as `Context::new` builds it: with the production registry.
fn production_context(pool: &Arc<PgPool>, caller: &Caller) -> Context {
    match caller.user() {
        Some(user) => test_db::test_context_with_user(Arc::clone(pool), user),
        None => test_db::test_context_anonymous(Arc::clone(pool)),
    }
}

fn context(pool: &Arc<PgPool>, caller: &Caller, registry: &'static AsyncJobRegistry) -> Context {
    production_context(pool, caller).with_async_registry(registry)
}

async fn execute(
    schema: &Schema,
    context: &Context,
    query: &str,
    variables: JsonValue,
) -> JsonValue {
    let request: GraphQLRequest =
        serde_json::from_value(json!({ "query": query, "variables": variables }))
            .expect("build the GraphQL request");
    serde_json::to_value(request.execute(schema, context).await)
        .expect("serialize the GraphQL response")
}

/// One request as `caller`, in its own context, serving the test registry.
async fn run(
    schema: &Schema,
    pool: &Arc<PgPool>,
    caller: &Caller,
    query: &str,
    variables: JsonValue,
) -> JsonValue {
    execute(
        schema,
        &context(pool, caller, &API_REGISTRY),
        query,
        variables,
    )
    .await
}

fn errors(response: &JsonValue) -> Vec<JsonValue> {
    response
        .get("errors")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The non-null value of `field`, in a response without errors.
#[track_caller]
fn data<'a>(response: &'a JsonValue, field: &str) -> &'a JsonValue {
    assert!(
        errors(response).is_empty(),
        "unexpected GraphQL errors: {response}"
    );
    let value = &response["data"][field];
    assert!(!value.is_null(), "`{field}` is null: {response}");
    value
}

/// One GraphQL error: its `extensions.type`, message, path and, for a
/// transition refusal, its reason.
struct FieldFailure {
    kind: String,
    message: String,
    path: Vec<String>,
    reason: Option<String>,
}

fn failure(error: &JsonValue) -> FieldFailure {
    FieldFailure {
        kind: error["extensions"]["type"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        message: error["message"].as_str().unwrap_or_default().to_string(),
        path: error["path"]
            .as_array()
            .map(|path| {
                path.iter()
                    .map(|step| step.as_str().unwrap_or_default().to_string())
                    .collect()
            })
            .unwrap_or_default(),
        reason: error["extensions"]["reason"].as_str().map(str::to_string),
    }
}

#[track_caller]
fn only_error(response: &JsonValue) -> FieldFailure {
    let errors = errors(response);
    assert_eq!(errors.len(), 1, "expected exactly one error: {response}");
    failure(&errors[0])
}

#[track_caller]
fn assert_error(response: &JsonValue, kind: &str, message: &str) {
    let error = only_error(response);
    assert_eq!(
        (error.kind.as_str(), error.message.as_str()),
        (kind, message),
        "{response}"
    );
}

/// A fail-closed denial of `field`, with no data.
#[track_caller]
fn assert_unauthorized(response: &JsonValue, field: &str, label: &str) {
    let error = only_error(response);
    assert_eq!(
        (error.kind.as_str(), error.message.as_str()),
        ("NO_ACCESS", "Unauthorized"),
        "{label} must be refused `{field}`: {response}"
    );
    assert_eq!(error.path, [field], "{label}: {response}");
    assert!(
        response["data"].is_null() || response["data"][field].is_null(),
        "{label} got data from `{field}`: {response}"
    );
}

/// A GraphQL input error, raised before any resolver runs.
#[track_caller]
fn assert_input_error(response: &JsonValue) {
    let errors = errors(response);
    assert!(!errors.is_empty(), "expected an input error: {response}");
    assert!(response["data"].is_null(), "{response}");
    for error in &errors {
        let kind = error["extensions"]["type"].as_str().unwrap_or_default();
        assert!(
            !kind.starts_with("ASYNC"),
            "the engine must not have run: {response}"
        );
    }
}

fn uuid_of(value: &JsonValue) -> Uuid {
    value
        .as_str()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| panic!("not a UUID: {value}"))
}

fn ids(jobs: &[JsonValue]) -> Vec<String> {
    jobs.iter()
        .map(|job| job["asyncJobId"].as_str().expect("asyncJobId").to_string())
        .collect()
}

fn claim_input(claimed_by: &str, kinds: &[&str], max_jobs: i32) -> JsonValue {
    json!({
        "claimedBy": claimed_by,
        "kinds": kinds
            .iter()
            .map(|kind| json!({ "kind": kind, "payloadVersions": [1] }))
            .collect::<Vec<_>>(),
        "maxJobs": max_jobs,
    })
}

fn claim_vars(claimed_by: &str, kinds: &[&str], max_jobs: i32) -> JsonValue {
    json!({ "input": claim_input(claimed_by, kinds, max_jobs) })
}

fn command_vars(job: Uuid, reason: &str, request_id: Uuid) -> JsonValue {
    json!({ "input": { "asyncJobId": job, "reason": reason, "requestId": request_id } })
}

/// A staff FAILED resolution with no-effect evidence.
fn staff_failed_vars(job: Uuid, reason: &str, request_id: Uuid) -> JsonValue {
    json!({ "input": {
        "asyncJobId": job,
        "outcome": "FAILED",
        "evidence": no_effect_evidence(),
        "reason": reason,
        "requestId": request_id,
    } })
}

fn applied_evidence(order: i64) -> String {
    json!({ "valid": true, "order": order }).to_string()
}

fn no_effect_evidence() -> String {
    json!({ "valid": true, "providerRef": "provider-1" }).to_string()
}

/// A resume time inside every test kind's WAITING bounds.
fn resume_at() -> String {
    (Utc::now() + Duration::seconds(120)).to_rfc3339_opts(SecondsFormat::Secs, true)
}

// ---------------------------------------------------------------------------
// GraphQL documents
// ---------------------------------------------------------------------------

const CLAIM: &str = "mutation Claim($input: ClaimAsyncJobsInput!) { \
     claimAsyncJobs(input: $input) { \
     jobs { asyncJobId kind claimToken attemptNumber effectPhase targetKeys } skippedKinds } }";

const CLAIM_RECONCILIATIONS: &str = "mutation Reconcile($input: ClaimAsyncJobsInput!) { \
     claimAsyncJobReconciliations(input: $input) { \
     jobs { asyncJobId kind claimToken attemptNumber effectPhase targetKeys } skippedKinds } }";

const CLAIM_FIELDS: [(&str, &str); 2] = [
    ("claimAsyncJobs", CLAIM),
    ("claimAsyncJobReconciliations", CLAIM_RECONCILIATIONS),
];

const RENEW: &str = "mutation Renew($input: RenewAsyncJobLeaseInput!) { \
     renewAsyncJobLease(input: $input) { leaseExpiresAt attemptRuntimeDeadline } }";

const MARK_STARTED: &str = "mutation Start($input: AsyncJobClaimInput!) { \
     markAsyncJobEffectStarted(input: $input) { asyncJobId status effectPhase } }";

const MARK_CONFIRMED: &str = "mutation Confirm($input: MarkAsyncJobEffectConfirmedInput!) { \
     markAsyncJobEffectConfirmed(input: $input) { asyncJobId status effectPhase providerRef } }";

const WAIT: &str = "mutation Wait($input: WaitAsyncJobInput!) { \
     waitAsyncJob(input: $input) { asyncJobId status effectPhase checkpoint } }";

const COMPLETE: &str = "mutation Complete($input: CompleteAsyncJobInput!) { \
     completeAsyncJob(input: $input) { asyncJobId status effectPhase lastOutcomeCode } }";

const FAIL: &str = "mutation Fail($input: FailAsyncJobInput!) { \
     failAsyncJob(input: $input) { asyncJobId status effectPhase lastOutcomeCode } }";

const RELEASE: &str = "mutation Release($input: AsyncJobClaimInput!) { \
     releaseAsyncJob(input: $input) { asyncJobId status effectPhase } }";

const RESOLVE: &str = "mutation Resolve($input: ResolveAsyncJobReconciliationInput!) { \
     resolveAsyncJobReconciliation(input: $input) { \
     asyncJobId status effectPhase lastOutcomeCode } }";

const TARGET_STATE: &str = "query State($input: AsyncJobClaimInput!) { \
     asyncJobTargetState(input: $input) { targetKey appliedOrder appliedFingerprint \
     appliedSourceRef lastJobId regressedAt regressedFromOrder regressedFromFingerprint } }";

/// An operator command document, selecting the whole command result.
macro_rules! command_document {
    ($operation:literal, $field:literal, $input_type:literal) => {
        concat!(
            "mutation ",
            $operation,
            "($input: ",
            $input_type,
            "!) { ",
            $field,
            "(input: $input) { ...Command } } ",
            "fragment Command on AsyncJobCommandResult { asyncControlCommandId commandKind ",
            "outcome outcomeCode replayed job { asyncJobId status effectPhase lastOutcomeCode } ",
            "resultJob { asyncJobId status lineage parentJobId } }"
        )
    };
}

const CANCEL: &str = command_document!("Cancel", "cancelAsyncJob", "AsyncJobCommandInput");
const RETRY: &str = command_document!("Retry", "retryAsyncJob", "AsyncJobCommandInput");
const REDACT: &str = command_document!(
    "Redact",
    "redactAsyncJobDiagnostics",
    "AsyncJobCommandInput"
);
const RESOLVE_AS_STAFF: &str = command_document!(
    "Staff",
    "resolveAsyncJobReconciliationAsStaff",
    "ResolveAsyncJobReconciliationAsStaffInput"
);

const READ_JOB: &str = "query Job($id: Uuid!) { asyncJob(asyncJobId: $id) { \
     job { asyncJobId kind status effectPhase claimedBy executorPrincipal lastOutcomeCode \
     redactedAt } \
     attempts { attemptNumber purpose claimedBy executorPrincipal result outcomeCode errorDetail } \
     transitions { fromStatus toStatus cause } } }";

const LIST_JOBS: &str = "query Jobs($filter: AsyncJobFilterInput, $limit: Int!, \
     $after: AsyncJobListPositionInput) { asyncJobs(filter: $filter, limit: $limit, after: $after) { \
     asyncJobId kind status createdAt targetKeys correlationId } }";

const KIND_SIGNALS: &str =
    "query Signals($kinds: [String!]) { asyncJobKindSignals(kinds: $kinds) { \
     kind known readyCount runningCount reconciliationRequiredCount regressedTargetCount \
     regressedTargets { targetKey appliedOrder appliedFingerprint regressedAt \
     regressedFromOrder regressedFromFingerprint } } }";

const EXECUTOR_ACTIVITY: &str = "query Activity($since: Timestamp) { \
     asyncJobExecutorActivity(since: $since) { executorName executorPrincipal kind \
     lastExecutionPollAt lastExecutionClaimAt lastReconciliationPollAt latestOpenHeartbeatAt } }";

/// The nine operations a claim token fences, in fixture order.
const FENCED: [&str; 9] = [
    "renewAsyncJobLease",
    "markAsyncJobEffectStarted",
    "markAsyncJobEffectConfirmed",
    "waitAsyncJob",
    "completeAsyncJob",
    "failAsyncJob",
    "releaseAsyncJob",
    "resolveAsyncJobReconciliation",
    "asyncJobTargetState",
];

/// The document and variables of a token-fenced operation on `fixture`,
/// with arguments that succeed for a job in the state the matrix fixture
/// gives it.
fn fenced(field: &str, fixture: &Fixture) -> (&'static str, JsonValue) {
    let mut input = fixture.input();
    let document = match field {
        "renewAsyncJobLease" => {
            input["leaseSeconds"] = json!(60);
            RENEW
        }
        "markAsyncJobEffectStarted" => MARK_STARTED,
        "markAsyncJobEffectConfirmed" => {
            input["providerRef"] = json!("provider-1");
            MARK_CONFIRMED
        }
        "waitAsyncJob" => {
            input["checkpoint"] = json!(json!({ "step": 1 }).to_string());
            input["resumeAt"] = json!(resume_at());
            WAIT
        }
        "completeAsyncJob" => {
            input["outcome"] = json!("EFFECT_APPLIED");
            input["entries"] = json!([{ "targetKey": fixture.key, "fingerprint": "h1" }]);
            COMPLETE
        }
        "failAsyncJob" => {
            input["failureClass"] = json!("PERMANENT");
            input["errorCode"] = json!("TEST_FAILURE");
            input["errorDetail"] = json!("the provider refused the request");
            input["noEffectProven"] = json!(false);
            FAIL
        }
        "releaseAsyncJob" => RELEASE,
        "resolveAsyncJobReconciliation" => {
            input["outcome"] = json!("STILL_REQUIRED");
            RESOLVE
        }
        "asyncJobTargetState" => TARGET_STATE,
        other => panic!("`{other}` is not a token-fenced field"),
    };
    (document, json!({ "input": input }))
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

/// A dedicated connection, outside the shared pool.
fn connect() -> PgConnection {
    PgConnection::establish(&test_db::test_db_url()).expect("connect to the test database")
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = SqlBigInt)]
    value: i64,
}

/// One nullable text value, selected `AS value`.
fn text(c: &mut PgConnection, sql: &str) -> Option<String> {
    sql_query(sql)
        .get_result::<TextRow>(c)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

/// One `bigint` value, selected `AS value`.
fn count(c: &mut PgConnection, sql: &str) -> i64 {
    sql_query(sql)
        .get_result::<CountRow>(c)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

fn exec(c: &mut PgConnection, sql: &str) {
    c.batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

/// Every table the engine writes.
const ASYNC_TABLES: [&str; 9] = [
    "async_job",
    "async_job_target",
    "async_target_hold",
    "async_effect_target_state",
    "async_job_attempt",
    "async_control_command",
    "async_job_transition",
    "async_executor_poll",
    "async_recovery_failure",
];

/// Every row of every async table, as text: equal footprints mean nothing
/// was written in between.
fn footprint(c: &mut PgConnection) -> String {
    ASYNC_TABLES
        .iter()
        .map(|table| {
            let rows = text(
                c,
                &format!(
                    "SELECT string_agg(r::text, chr(10) ORDER BY r::text) AS value \
                       FROM public.{table} r"
                ),
            )
            .unwrap_or_default();
            format!("[{table}]\n{rows}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `pg_stat_database.deadlocks` of the test database.
fn deadlocks(c: &mut PgConnection) -> i64 {
    exec(c, "SELECT pg_stat_clear_snapshot()");
    count(
        c,
        "SELECT deadlocks AS value FROM pg_stat_database WHERE datname = current_database()",
    )
}

fn status(c: &mut PgConnection, id: Uuid) -> String {
    job_column(c, id, "status").expect("the job exists")
}

fn job_column(c: &mut PgConnection, id: Uuid, column: &str) -> Option<String> {
    text(
        c,
        &format!(
            "SELECT {column}::text AS value FROM public.async_job WHERE async_job_id = '{id}'"
        ),
    )
}

fn job_row(c: &mut PgConnection, id: Uuid) -> String {
    text(
        c,
        &format!("SELECT r::text AS value FROM public.async_job r WHERE async_job_id = '{id}'"),
    )
    .expect("the job exists")
}

fn running(c: &mut PgConnection) -> i64 {
    count(
        c,
        "SELECT count(*) AS value FROM public.async_job WHERE status = 'RUNNING'",
    )
}

fn attempts(c: &mut PgConnection, purpose: &str) -> i64 {
    count(
        c,
        &format!(
            "SELECT count(*) AS value FROM public.async_job_attempt WHERE purpose = '{purpose}'"
        ),
    )
}

fn commands(c: &mut PgConnection) -> i64 {
    count(
        c,
        "SELECT count(*) AS value FROM public.async_control_command",
    )
}

fn holds(c: &mut PgConnection, id: Uuid) -> i64 {
    count(
        c,
        &format!(
            "SELECT count(*) AS value FROM public.async_target_hold WHERE async_job_id = '{id}'"
        ),
    )
}

/// The audit columns of one command row, `|`-separated.
fn command_row(c: &mut PgConnection, id: Uuid) -> String {
    text(
        c,
        &format!(
            "SELECT concat_ws('|', command_kind::text, request_id::text, actor_principal, reason, \
                    async_job_id::text, outcome::text, outcome_code, \
                    coalesce(result_job_id::text, '-')) AS value \
               FROM public.async_control_command WHERE async_control_command_id = '{id}'"
        ),
    )
    .expect("the command row")
}

/// `regressed_at|regressed_from_order|regressed_from_fingerprint` of a
/// target's state row; `||` when all three are null.
fn regressed_columns(c: &mut PgConnection, key: &str) -> String {
    text(
        c,
        &format!(
            "SELECT coalesce(regressed_at::text, '') || '|' || \
                    coalesce(regressed_from_order::text, '') || '|' || \
                    coalesce(regressed_from_fingerprint, '') AS value \
               FROM public.async_effect_target_state WHERE target_key = '{key}'"
        ),
    )
    .expect("the target's state row")
}

// ---------------------------------------------------------------------------
// Engine fixtures
// ---------------------------------------------------------------------------

fn tx<T>(
    c: &mut PgConnection,
    operation: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    with_engine_transaction_on(c, operation)
}

/// A version-1 job of `kind` with payload `{"id": id, "targets": targets}`.
fn new_job(kind: &str, id: &str, targets: &[&str]) -> NewAsyncJob {
    NewAsyncJob {
        kind: kind.to_string(),
        payload_version: 1,
        payload: json!({ "id": id, "targets": targets }).to_string(),
        priority: 0,
        source_order: None,
        source_fingerprint: None,
        source_ref: None,
        correlation_id: None,
        causation_ref: None,
        created_by: "test:creator".to_string(),
    }
}

/// An `API_REVISION` job for revision `order` of `id`, on one target.
fn revision_job(id: &str, target: &str, order: i64, fingerprint: &str) -> NewAsyncJob {
    NewAsyncJob {
        payload: json!({ "id": id, "rev": order, "targets": [target] }).to_string(),
        source_order: Some(order),
        source_fingerprint: Some(fingerprint.to_string()),
        source_ref: Some(format!("ref-{order}")),
        ..new_job(API_REVISION, id, &[target])
    }
}

/// A fresh target key of `namespace`.
fn key(namespace: &str) -> String {
    format!("{namespace}:{}", Uuid::new_v4())
}

fn create(c: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    tx(c, |c| create_job_on(c, &API_REGISTRY, input))
        .expect("create the fixture job")
        .into_job()
}

/// The dissemination executor exactly as the executor map builds it.
fn dissemination_executor(name: &str) -> Executor {
    executor_profile(Role::DisseminationWorker)
        .expect("the dissemination profile")
        .executor(name, WORKER_ID)
}

/// The in-process executor, which no GraphQL caller can be.
fn local_executor() -> Executor {
    Executor {
        name: "thoth-worker".to_string(),
        principal: "process:thoth-worker".to_string(),
        domain: ExecutorDomain::ThothWorker,
        risk_class: RiskClass::Trusted,
        capabilities: Vec::new(),
        revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
    }
}

fn claim_request(kinds: &[&str], max_jobs: i32) -> ClaimRequest {
    ClaimRequest {
        kinds: kinds
            .iter()
            .map(|kind| (kind.to_string(), vec![1]))
            .collect(),
        max_jobs,
        lease_seconds: None,
    }
}

/// A claimed job: its ID, its live claim token and its (first) target key.
#[derive(Debug, Clone)]
struct Fixture {
    id: Uuid,
    token: Uuid,
    key: String,
}

impl Fixture {
    fn input(&self) -> JsonValue {
        json!({ "asyncJobId": self.id, "claimToken": self.token })
    }

    fn worker_claim(&self, domain: ExecutorDomain) -> WorkerClaim {
        WorkerClaim {
            async_job_id: self.id,
            claim_token: self.token,
            domain,
        }
    }
}

/// Create `inputs` (all of one kind) and claim them as `executor`; every
/// ready job of that kind is claimed, so callers create other ready jobs of
/// it only afterwards. The fixtures come back in input order.
fn claimed(c: &mut PgConnection, executor: &Executor, inputs: Vec<NewAsyncJob>) -> Vec<Fixture> {
    let kind = inputs.first().expect("at least one job").kind.clone();
    let jobs: Vec<AsyncJob> = inputs.into_iter().map(|input| create(c, input)).collect();
    let claims = tx(c, |c| {
        claim_on(
            c,
            &API_REGISTRY,
            executor,
            claim_request(&[kind.as_str()], 50),
        )
    })
    .expect("claim the fixture jobs")
    .jobs;
    jobs.iter()
        .map(|job| {
            let claim = claims
                .iter()
                .find(|claim| claim.async_job_id == job.async_job_id)
                .expect("every fixture job is claimed");
            Fixture {
                id: job.async_job_id,
                token: claim.claim_token,
                key: job.target_keys.first().cloned().unwrap_or_default(),
            }
        })
        .collect()
}

/// Claim the RECONCILIATION_REQUIRED `fixture` job for reconciliation as
/// `executor`; the fixture comes back with the new token.
fn reconciling(
    c: &mut PgConnection,
    executor: &Executor,
    kind: &str,
    fixture: &Fixture,
) -> Fixture {
    let claims = tx(c, |c| {
        claim_reconciliation_on(c, &API_REGISTRY, executor, claim_request(&[kind], 50))
    })
    .expect("claim the reconciliation")
    .jobs;
    let claim = claims
        .iter()
        .find(|claim| claim.async_job_id == fixture.id)
        .expect("the job is claimed for reconciliation");
    Fixture {
        token: claim.claim_token,
        ..fixture.clone()
    }
}

fn start(c: &mut PgConnection, fixture: &Fixture, domain: ExecutorDomain) {
    let claim = fixture.worker_claim(domain);
    tx(c, |c| mark_effect_started_on(c, &API_REGISTRY, &claim))
        .expect("start the fixture job's effect");
}

fn fail(c: &mut PgConnection, fixture: &Fixture, domain: ExecutorDomain, class: FailureClass) {
    let claim = fixture.worker_claim(domain);
    let request = FailRequest {
        class,
        error_code: "TEST_FAILURE".to_string(),
        error_detail: Some("the provider refused the request".to_string()),
        no_effect_proven: false,
        evidence: None,
    };
    tx(c, |c| fail_on(c, &API_REGISTRY, &claim, request)).expect("fail the fixture job");
}

/// Move a running fixture job to RECONCILIATION_REQUIRED, unclaimed: its
/// effect starts, then the worker reports an INDETERMINATE failure.
fn to_reconciliation(c: &mut PgConnection, fixture: &Fixture, domain: ExecutorDomain) {
    start(c, fixture, domain);
    fail(c, fixture, domain, FailureClass::Indeterminate);
    assert_eq!(status(c, fixture.id), "RECONCILIATION_REQUIRED");
}

fn rb_entry(target: &str, order: i64, fingerprint: &str) -> EvidenceEntry {
    EvidenceEntry {
        target_key: target.to_string(),
        order: Some(i128::from(order)),
        fingerprint: Some(fingerprint.to_string()),
        source_ref: None,
    }
}

/// Claim, start and complete an `API_REVISION` job for revision `order` of
/// `target` (fingerprint `h{order}`) through the engine.
fn complete_revision(
    c: &mut PgConnection,
    executor: &Executor,
    target: &str,
    order: i64,
) -> (Fixture, CompleteOutcome) {
    let fingerprint = format!("h{order}");
    let id = format!("revision-{order}-{}", Uuid::new_v4());
    let fixture = claimed(
        c,
        executor,
        vec![revision_job(&id, target, order, &fingerprint)],
    )
    .remove(0);
    start(c, &fixture, ExecutorDomain::Dissemination);
    let claim = fixture.worker_claim(ExecutorDomain::Dissemination);
    let request = CompleteRequest {
        outcome: CompletionOutcome::EffectApplied,
        entries: vec![rb_entry(target, order, &fingerprint)],
        evidence: None,
    };
    let outcome = tx(c, |c| complete_on(c, &API_REGISTRY, &claim, request))
        .expect("complete the revision job");
    (fixture, outcome)
}

/// An ordinary forward write: revision `order` reaches `target`.
fn forward_write(c: &mut PgConnection, executor: &Executor, target: &str, order: i64) {
    let (_fixture, outcome) = complete_revision(c, executor, target, order);
    assert!(
        matches!(outcome, CompleteOutcome::Succeeded(_)),
        "{outcome:?}"
    );
}

/// The nine matrix fixtures, in `FENCED` order, each in the state its
/// operation needs: running at PRE_WRITE, with the effect started for the
/// confirmation and the completion, and reconciliation-claimed for the
/// resolution.
fn fenced_fixtures(c: &mut PgConnection) -> Vec<Fixture> {
    let executor = dissemination_executor("fixture-executor");
    let inputs = FENCED
        .iter()
        .map(|field| new_job(API_EFFECT, field, &[&key(NS_CS)]))
        .collect();
    let mut fixtures = claimed(c, &executor, inputs);
    let at = |field: &str| {
        FENCED
            .iter()
            .position(|fenced| *fenced == field)
            .expect("a fenced field")
    };
    for field in ["markAsyncJobEffectConfirmed", "completeAsyncJob"] {
        start(c, &fixtures[at(field)], ExecutorDomain::Dissemination);
    }
    let resolve = at("resolveAsyncJobReconciliation");
    to_reconciliation(c, &fixtures[resolve], ExecutorDomain::Dissemination);
    fixtures[resolve] = reconciling(c, &executor, API_EFFECT, &fixtures[resolve]);
    fixtures
}

/// The jobs the operator commands act on.
struct CommandJobs {
    /// PENDING, to cancel.
    pending: Uuid,
    /// PENDING, for the superuser who also holds the machine role.
    pending_too: Uuid,
    /// FAILED at PRE_WRITE, to retry.
    failed: Uuid,
    /// RECONCILIATION_REQUIRED at EFFECT_STARTED, unclaimed, to resolve.
    reconciliation: Uuid,
    /// FAILED with diagnostics, to redact.
    terminal: Uuid,
}

fn command_jobs(c: &mut PgConnection) -> CommandJobs {
    let executor = dissemination_executor("fixture-executor");
    let fixtures = claimed(
        c,
        &executor,
        vec![
            new_job(API_EFFECT, "failed", &[&key(NS_CS)]),
            new_job(API_EFFECT, "reconciliation", &[&key(NS_CS)]),
            new_job(API_EFFECT, "terminal", &[&key(NS_CS)]),
        ],
    );
    let domain = ExecutorDomain::Dissemination;
    fail(c, &fixtures[0], domain, FailureClass::Permanent);
    to_reconciliation(c, &fixtures[1], domain);
    fail(c, &fixtures[2], domain, FailureClass::Permanent);
    CommandJobs {
        pending: create(c, new_job(API_EFFECT, "pending", &[&key(NS_CS)])).async_job_id,
        pending_too: create(c, new_job(API_EFFECT, "pending-too", &[&key(NS_CS)])).async_job_id,
        failed: fixtures[0].id,
        reconciliation: fixtures[1].id,
        terminal: fixtures[2].id,
    }
}

async fn signals_as_staff(
    schema: &Schema,
    pool: &Arc<PgPool>,
    variables: JsonValue,
) -> Vec<JsonValue> {
    let response = run(schema, pool, &staff(), KIND_SIGNALS, variables).await;
    data(&response, "asyncJobKindSignals")
        .as_array()
        .expect("a list")
        .clone()
}

async fn revision_signals(schema: &Schema, pool: &Arc<PgPool>) -> JsonValue {
    let rows = signals_as_staff(schema, pool, json!({ "kinds": [API_REVISION] })).await;
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows[0].clone()
}

async fn list_as_staff(
    schema: &Schema,
    pool: &Arc<PgPool>,
    variables: JsonValue,
) -> Vec<JsonValue> {
    let response = run(schema, pool, &staff(), LIST_JOBS, variables).await;
    data(&response, "asyncJobs")
        .as_array()
        .expect("a list")
        .clone()
}

async fn activity_as_staff(
    schema: &Schema,
    pool: &Arc<PgPool>,
    since: JsonValue,
) -> Vec<JsonValue> {
    let response = run(
        schema,
        pool,
        &staff(),
        EXECUTOR_ACTIVITY,
        json!({ "since": since }),
    )
    .await;
    data(&response, "asyncJobExecutorActivity")
        .as_array()
        .expect("a list")
        .clone()
}

// ===========================================================================
// 14.1: the executor map and the registry the resolvers serve with
// ===========================================================================

#[test]
fn the_test_registries_validate_and_only_dissemination_worker_maps_to_an_executor() {
    for registry in [&API_REGISTRY, &NARROW_REGISTRY, &PRODUCTION_REGISTRY] {
        assert_eq!(registry.validate(), Ok(()));
    }
    assert!(PRODUCTION_REGISTRY.is_empty());

    let roles = [
        Role::Superuser,
        Role::PublisherAdmin,
        Role::PublisherUser,
        Role::WorkLifecycle,
        Role::CdnWrite,
        Role::DisseminationWorker,
        Role::MetricsIngestService,
        Role::MetricsReadService,
    ];
    for role in roles {
        let expected = (role == Role::DisseminationWorker).then_some(ExecutorProfile {
            domain: ExecutorDomain::Dissemination,
            risk_class: RiskClass::Trusted,
            capabilities: &["dissemination"],
            revision_modes: &[RevisionMode::CurrentState, RevisionMode::RevisionBound],
        });
        assert_eq!(executor_profile(role), expected, "{}", role.as_ref());
    }

    // The engine identity takes the executor name from the request and the
    // principal from the token; everything else comes from the map.
    assert_eq!(
        dissemination_executor("exec-1"),
        Executor {
            name: "exec-1".to_string(),
            principal: WORKER_ID.to_string(),
            domain: ExecutorDomain::Dissemination,
            risk_class: RiskClass::Trusted,
            capabilities: vec![DISSEMINATION_CAPABILITY.to_string()],
            revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
        }
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn with_the_empty_production_registry_the_worker_claims_nothing_and_every_other_caller_is_refused(
) {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    // A row of a kind the production binary does not register: it is left
    // alone.
    let job = create(&mut c, new_job(API_EFFECT, "inert", &[&key(NS_CS)]));
    let before = footprint(&mut c);

    for (field, document) in CLAIM_FIELDS {
        let context = production_context(&pool, &worker());
        assert!(ptr::eq(context.async_registry, &PRODUCTION_REGISTRY));
        let response = execute(
            &schema,
            &context,
            document,
            claim_vars("exec-1", &[API_EFFECT, "ANOTHER_KIND"], 5),
        )
        .await;
        let result = data(&response, field);
        assert_eq!(result["jobs"], json!([]));
        assert_eq!(result["skippedKinds"], json!([API_EFFECT, "ANOTHER_KIND"]));

        for caller in refused_executors() {
            let response = execute(
                &schema,
                &production_context(&pool, &caller),
                document,
                claim_vars("exec-1", &[API_EFFECT], 5),
            )
            .await;
            assert_unauthorized(&response, field, caller.label);
        }
    }

    assert_eq!(footprint(&mut c), before, "the inert engine wrote");
    assert_eq!(status(&mut c, job.async_job_id), "PENDING");
}

// ===========================================================================
// 19.2 API and authorization: every operation, every caller
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn both_claim_fields_admit_only_the_dissemination_worker() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let rr = claimed(
        &mut c,
        &executor,
        vec![new_job(API_EFFECT, "claims-rr", &[&key(NS_CS)])],
    )
    .remove(0);
    to_reconciliation(&mut c, &rr, ExecutorDomain::Dissemination);
    let pending: BTreeSet<Uuid> = (0..2)
        .map(|i| {
            create(
                &mut c,
                new_job(API_EFFECT, &format!("claims-{i}"), &[&key(NS_CS)]),
            )
            .async_job_id
        })
        .collect();

    let before = footprint(&mut c);
    for (field, document) in CLAIM_FIELDS {
        for caller in refused_executors() {
            let response = run(
                &schema,
                &pool,
                &caller,
                document,
                claim_vars("exec-1", &[API_EFFECT], 5),
            )
            .await;
            assert_unauthorized(&response, field, caller.label);
        }
    }
    assert_eq!(footprint(&mut c), before, "a refused claim wrote");

    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-1", &[API_EFFECT], 5),
    )
    .await;
    let jobs = data(&response, "claimAsyncJobs")["jobs"]
        .as_array()
        .expect("jobs")
        .clone();
    let claimed_ids: BTreeSet<Uuid> = jobs.iter().map(|job| uuid_of(&job["asyncJobId"])).collect();
    assert_eq!(claimed_ids, pending);
    for job in &jobs {
        assert!(!job["claimToken"].is_null());
        assert_eq!(job["effectPhase"], json!("PRE_WRITE"));
    }

    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM_RECONCILIATIONS,
        claim_vars("exec-1", &[API_EFFECT], 5),
    )
    .await;
    let jobs = data(&response, "claimAsyncJobReconciliations")["jobs"]
        .as_array()
        .expect("jobs")
        .clone();
    assert_eq!(jobs.len(), 1, "{response}");
    assert_eq!(uuid_of(&jobs[0]["asyncJobId"]), rr.id);
    assert_eq!(jobs[0]["effectPhase"], json!("EFFECT_STARTED"));

    // A superuser who also holds the machine role passes the machine-role
    // check (there is nothing left to claim).
    for (field, document) in CLAIM_FIELDS {
        let response = run(
            &schema,
            &pool,
            &superworker(),
            document,
            claim_vars("exec-2", &[API_EFFECT], 5),
        )
        .await;
        assert_eq!(data(&response, field)["jobs"], json!([]));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_token_fenced_operation_admits_only_the_dissemination_worker() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let fixtures = fenced_fixtures(&mut c);

    let before = footprint(&mut c);
    for (field, fixture) in FENCED.iter().zip(&fixtures) {
        let (document, variables) = fenced(field, fixture);
        for caller in refused_executors() {
            let response = run(&schema, &pool, &caller, document, variables.clone()).await;
            assert_unauthorized(&response, field, caller.label);
        }
    }
    assert_eq!(
        footprint(&mut c),
        before,
        "a refused caller changed a job with a live claim token"
    );

    // The superuser who also holds the machine role reads the target state.
    let target = &fixtures[8];
    let (document, variables) = fenced("asyncJobTargetState", target);
    let response = run(&schema, &pool, &superworker(), document, variables).await;
    let states = data(&response, "asyncJobTargetState");
    assert_eq!(states.as_array().map(Vec::len), Some(1), "{response}");
    assert_eq!(states[0]["targetKey"], json!(target.key));
    assert!(states[0]["appliedFingerprint"].is_null());

    // The worker runs every operation.
    for (field, fixture) in FENCED.iter().zip(&fixtures) {
        let (document, variables) = fenced(field, fixture);
        let response = run(&schema, &pool, &worker(), document, variables).await;
        let value = data(&response, field);
        let expected = match *field {
            "markAsyncJobEffectStarted" => Some(("RUNNING", "EFFECT_STARTED")),
            "markAsyncJobEffectConfirmed" => Some(("RUNNING", "EFFECT_CONFIRMED")),
            "waitAsyncJob" => Some(("WAITING", "PRE_WRITE")),
            "completeAsyncJob" => Some(("SUCCEEDED", "EFFECT_STARTED")),
            "failAsyncJob" => Some(("FAILED", "PRE_WRITE")),
            "releaseAsyncJob" => Some(("PENDING", "PRE_WRITE")),
            "resolveAsyncJobReconciliation" => Some(("RECONCILIATION_REQUIRED", "EFFECT_STARTED")),
            _ => None,
        };
        if let Some((job_status, phase)) = expected {
            assert_eq!(
                (value["status"].clone(), value["effectPhase"].clone()),
                (json!(job_status), json!(phase)),
                "{field}: {response}"
            );
            assert_eq!(status(&mut c, fixture.id), job_status);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_operator_command_admits_only_the_superuser() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let jobs = command_jobs(&mut c);
    let operations = [
        (
            "cancelAsyncJob",
            CANCEL,
            command_vars(jobs.pending, "cancel the job", Uuid::new_v4()),
            "CANCELLED",
        ),
        (
            "retryAsyncJob",
            RETRY,
            command_vars(jobs.failed, "retry the job", Uuid::new_v4()),
            "RETRY_CREATED",
        ),
        (
            "resolveAsyncJobReconciliationAsStaff",
            RESOLVE_AS_STAFF,
            staff_failed_vars(jobs.reconciliation, "no effect exists", Uuid::new_v4()),
            "NO_EFFECT_PROVEN",
        ),
        (
            "redactAsyncJobDiagnostics",
            REDACT,
            command_vars(jobs.terminal, "redact the job", Uuid::new_v4()),
            "REDACTED",
        ),
    ];

    let before = footprint(&mut c);
    for (field, document, variables, _) in &operations {
        for caller in refused_operators() {
            let response = run(&schema, &pool, &caller, document, variables.clone()).await;
            assert_unauthorized(&response, field, caller.label);
        }
    }
    assert_eq!(
        footprint(&mut c),
        before,
        "a refused caller wrote a command row or changed a job"
    );

    for (field, document, variables, code) in &operations {
        let response = run(&schema, &pool, &staff(), document, variables.clone()).await;
        let result = data(&response, field);
        assert_eq!(result["outcome"], json!("APPLIED"), "{field}: {response}");
        assert_eq!(result["outcomeCode"], json!(code), "{field}: {response}");
        assert_eq!(result["replayed"], json!(false));
    }

    // A superuser who also holds the machine role is still a superuser.
    let response = run(
        &schema,
        &pool,
        &superworker(),
        CANCEL,
        command_vars(jobs.pending_too, "cancel the job", Uuid::new_v4()),
    )
    .await;
    assert_eq!(
        data(&response, "cancelAsyncJob")["outcomeCode"],
        json!("CANCELLED")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_operator_read_admits_only_the_superuser_and_pages_newest_first() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let correlation = Uuid::new_v4();
    let claimed_job = claimed(
        &mut c,
        &executor,
        vec![NewAsyncJob {
            correlation_id: Some(correlation),
            ..new_job(API_EFFECT, "read-0", &[&key(NS_CS)])
        }],
    )
    .remove(0);
    let pending: Vec<AsyncJob> = (1..4)
        .map(|i| {
            create(
                &mut c,
                new_job(API_EFFECT, &format!("read-{i}"), &[&key(NS_CS)]),
            )
        })
        .collect();
    let revision = create(&mut c, revision_job("read-revision", &key(NS_RB), 1, "h1"));

    let reads = [
        ("asyncJob", READ_JOB, json!({ "id": claimed_job.id })),
        ("asyncJobs", LIST_JOBS, json!({ "limit": 10 })),
        ("asyncJobKindSignals", KIND_SIGNALS, json!({})),
        ("asyncJobExecutorActivity", EXECUTOR_ACTIVITY, json!({})),
    ];
    let before = footprint(&mut c);
    for (field, document, variables) in &reads {
        for caller in refused_operators() {
            let response = run(&schema, &pool, &caller, document, variables.clone()).await;
            assert_unauthorized(&response, field, caller.label);
        }
        for caller in [staff(), superworker()] {
            let response = run(&schema, &pool, &caller, document, variables.clone()).await;
            data(&response, field);
        }
    }
    assert_eq!(footprint(&mut c), before, "a read wrote");

    // One job, with its attempts and status changes.
    let response = run(
        &schema,
        &pool,
        &staff(),
        READ_JOB,
        json!({ "id": claimed_job.id }),
    )
    .await;
    let detail = data(&response, "asyncJob");
    assert_eq!(detail["job"]["status"], json!("RUNNING"));
    assert_eq!(detail["job"]["claimedBy"], json!("fixture-executor"));
    assert_eq!(detail["job"]["executorPrincipal"], json!(WORKER_ID));
    let attempts = detail["attempts"].as_array().expect("attempts");
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0]["attemptNumber"], json!(1));
    assert_eq!(attempts[0]["purpose"], json!("EXECUTION"));
    assert_eq!(attempts[0]["claimedBy"], json!("fixture-executor"));
    assert_eq!(attempts[0]["executorPrincipal"], json!(WORKER_ID));
    assert!(attempts[0]["result"].is_null());
    assert_eq!(
        detail["transitions"],
        json!([
            { "fromStatus": null, "toStatus": "PENDING", "cause": "T1_CREATE" },
            { "fromStatus": "PENDING", "toStatus": "RUNNING", "cause": "T2_CLAIM" },
        ])
    );

    // The list: newest first, by creation time and then ID, a page at a
    // time.
    let expected = text(
        &mut c,
        "SELECT string_agg(async_job_id::text, ',' ORDER BY created_at DESC, async_job_id DESC) \
             AS value FROM public.async_job",
    )
    .expect("jobs");
    let all = list_as_staff(&schema, &pool, json!({ "limit": 100 })).await;
    assert_eq!(ids(&all).join(","), expected);
    let mut paged = Vec::new();
    let mut after = JsonValue::Null;
    loop {
        let page = list_as_staff(&schema, &pool, json!({ "limit": 2, "after": after })).await;
        assert!(page.len() <= 2);
        let Some(last) = page.last() else { break };
        after = json!({ "createdAt": last["createdAt"], "asyncJobId": last["asyncJobId"] });
        paged.extend(page);
    }
    assert_eq!(ids(&paged).join(","), expected);

    // Filters combine with AND; a list filter matches any of its values.
    let filtered = |filter: JsonValue| {
        list_as_staff(&schema, &pool, json!({ "filter": filter, "limit": 100 }))
    };
    let only = |job: Uuid| vec![job.to_string()];
    assert_eq!(
        ids(&filtered(json!({ "kinds": [API_REVISION] })).await),
        only(revision.async_job_id)
    );
    assert_eq!(
        ids(&filtered(json!({ "statuses": ["RUNNING"] })).await),
        only(claimed_job.id)
    );
    assert_eq!(
        ids(&filtered(json!({ "targetKey": pending[1].target_keys[0] })).await),
        only(pending[1].async_job_id)
    );
    assert_eq!(
        ids(&filtered(json!({ "correlationId": correlation })).await),
        only(claimed_job.id)
    );
    let pending_ids: BTreeSet<String> = pending
        .iter()
        .map(|job| job.async_job_id.to_string())
        .collect();
    let listed: BTreeSet<String> = ids(&filtered(
        json!({ "kinds": [API_EFFECT, "NOT_A_KIND"], "statuses": ["PENDING"] }),
    )
    .await)
    .into_iter()
    .collect();
    assert_eq!(listed, pending_ids);

    for limit in [0, 101] {
        let response = run(
            &schema,
            &pool,
            &staff(),
            LIST_JOBS,
            json!({ "limit": limit }),
        )
        .await;
        assert_error(
            &response,
            "ASYNC_JOB_PAYLOAD_INVALID",
            &ThothError::AsyncJobPayloadInvalid("limit must be between 1 and 100".to_string())
                .to_string(),
        );
    }

    // Signals: every registry kind when the list is absent or empty.
    let registry_kinds: BTreeSet<String> = API_REGISTRY
        .policies()
        .iter()
        .map(|policy| policy.kind.to_string())
        .collect();
    for variables in [json!({}), json!({ "kinds": [] })] {
        let rows = signals_as_staff(&schema, &pool, variables).await;
        let kinds: BTreeSet<String> = rows
            .iter()
            .map(|row| row["kind"].as_str().expect("kind").to_string())
            .collect();
        assert_eq!(kinds, registry_kinds);
        assert!(rows.iter().all(|row| row["known"] == json!(true)));
    }
    let rows = signals_as_staff(&schema, &pool, json!({ "kinds": [API_EFFECT] })).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (
            rows[0]["readyCount"].clone(),
            rows[0]["runningCount"].clone()
        ),
        (json!("3"), json!("1"))
    );
    let rows = signals_as_staff(&schema, &pool, json!({ "kinds": ["NOT_A_KIND"] })).await;
    assert!(rows.is_empty());

    // Activity: the fixture executor's poll and claim.
    let rows = activity_as_staff(&schema, &pool, JsonValue::Null).await;
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["executorName"], json!("fixture-executor"));
    assert_eq!(rows[0]["executorPrincipal"], json!(WORKER_ID));
    assert_eq!(rows[0]["kind"], json!(API_EFFECT));
}

// ===========================================================================
// 19.2: another domain's kinds, even with a valid token
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_dissemination_executor_cannot_claim_or_write_another_domains_job_even_with_a_valid_token(
) {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let not_authorized = ThothError::AsyncJobKindNotAuthorized.to_string();
    create(
        &mut c,
        new_job(API_LOCAL, "local-pending", &[&key(NS_LOCAL)]),
    );
    let effect = create(
        &mut c,
        new_job(API_EFFECT, "effect-pending", &[&key(NS_CS)]),
    );

    // A claim naming a ThothWorker kind, alone or beside a Dissemination
    // kind, is refused whole: nothing is claimed.
    let before = footprint(&mut c);
    for (field, document) in CLAIM_FIELDS {
        for kinds in [&[API_LOCAL][..], &[API_EFFECT, API_LOCAL][..]] {
            let response = run(
                &schema,
                &pool,
                &worker(),
                document,
                claim_vars("exec-1", kinds, 5),
            )
            .await;
            assert_error(&response, "ASYNC_JOB_KIND_NOT_AUTHORIZED", &not_authorized);
            assert!(response["data"][field].is_null());
        }
    }
    // A kind whose reconciler is the in-process worker is never reconciled
    // through GraphQL.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM_RECONCILIATIONS,
        claim_vars("exec-1", &[API_SPLIT], 5),
    )
    .await;
    assert_error(&response, "ASYNC_JOB_KIND_NOT_AUTHORIZED", &not_authorized);
    assert_eq!(footprint(&mut c), before, "a refused claim wrote");
    assert_eq!(status(&mut c, effect.async_job_id), "PENDING");

    // Valid tokens of the in-process worker's claims: an execution claim of
    // a ThothWorker kind, a reconciliation claim of one, and a
    // reconciliation claim of a kind Dissemination executes.
    let local = local_executor();
    let fixtures = claimed(
        &mut c,
        &local,
        vec![
            new_job(API_LOCAL, "local-run", &[&key(NS_LOCAL)]),
            new_job(API_LOCAL, "local-rr", &[&key(NS_LOCAL)]),
        ],
    );
    let local_run = fixtures[0].clone();
    to_reconciliation(&mut c, &fixtures[1], ExecutorDomain::ThothWorker);
    let local_rr = reconciling(&mut c, &local, API_LOCAL, &fixtures[1]);
    let split = claimed(
        &mut c,
        &dissemination_executor("fixture-executor"),
        vec![new_job(API_SPLIT, "split-rr", &[&key(NS_CS)])],
    )
    .remove(0);
    to_reconciliation(&mut c, &split, ExecutorDomain::Dissemination);
    let split_rr = reconciling(&mut c, &local, API_SPLIT, &split);

    let before = footprint(&mut c);
    for (label, fixture) in [
        ("a ThothWorker kind's execution token", &local_run),
        ("a ThothWorker kind's reconciliation token", &local_rr),
        (
            "a ThothWorker reconciliation token of a Dissemination-executed kind",
            &split_rr,
        ),
    ] {
        for field in FENCED {
            let (document, variables) = fenced(field, fixture);
            let response = run(&schema, &pool, &worker(), document, variables).await;
            let error = only_error(&response);
            assert_eq!(
                (error.kind.as_str(), error.message.as_str()),
                ("ASYNC_JOB_KIND_NOT_AUTHORIZED", not_authorized.as_str()),
                "`{field}` with {label}: {response}"
            );
        }
    }
    assert_eq!(
        footprint(&mut c),
        before,
        "another domain's job was written"
    );

    // Each token is still live for its own domain.
    for fixture in [&local_run, &local_rr, &split_rr] {
        let claim = fixture.worker_claim(ExecutorDomain::ThothWorker);
        tx(&mut c, |c| renew_on(c, &API_REGISTRY, &claim, None)).expect("the token is live");
    }

    // The split kind's execution is Dissemination's: that claim is admitted.
    create(&mut c, new_job(API_SPLIT, "split-run", &[&key(NS_CS)]));
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-1", &[API_SPLIT], 5),
    )
    .await;
    assert_eq!(
        data(&response, "claimAsyncJobs")["jobs"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

// ===========================================================================
// 14.1: the executor identity comes from the map and the token
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_executor_identity_comes_from_the_role_map_and_the_token_never_from_input() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    create(&mut c, new_job(API_UNTRUSTED, "untrusted", &[&key(NS_CS)]));
    let capable = create(&mut c, new_job(API_CAPABLE, "capable", &[&key(NS_CS)]));

    // The map's risk class is TRUSTED: an untrusted-content kind is refused.
    let before = footprint(&mut c);
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-capable", &[API_UNTRUSTED], 1),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_KIND_NOT_AUTHORIZED",
        &ThothError::AsyncJobKindNotAuthorized.to_string(),
    );
    assert_eq!(footprint(&mut c), before);

    // The map gives the `dissemination` capability, so a kind requiring it
    // is claimed.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-capable", &[API_CAPABLE], 1),
    )
    .await;
    let jobs = data(&response, "claimAsyncJobs")["jobs"]
        .as_array()
        .expect("jobs")
        .clone();
    assert_eq!(jobs.len(), 1);
    assert_eq!(uuid_of(&jobs[0]["asyncJobId"]), capable.async_job_id);

    // `claimedBy` is the executor name; the principal is the token's user.
    let id = capable.async_job_id;
    assert_eq!(
        job_column(&mut c, id, "claimed_by").as_deref(),
        Some("exec-capable")
    );
    assert_eq!(
        job_column(&mut c, id, "executor_principal").as_deref(),
        Some(WORKER_ID)
    );
    let response = run(&schema, &pool, &staff(), READ_JOB, json!({ "id": id })).await;
    let detail = data(&response, "asyncJob");
    for record in [&detail["job"], &detail["attempts"][0]] {
        assert_eq!(record["claimedBy"], json!("exec-capable"));
        assert_eq!(record["executorPrincipal"], json!(WORKER_ID));
    }

    // Another token's user polling under the same executor name is another
    // activity row.
    let second = Caller::new(
        "a second DISSEMINATION_WORKER",
        SECOND_WORKER_ID,
        &[Role::DisseminationWorker],
    );
    let response = run(
        &schema,
        &pool,
        &second,
        CLAIM,
        claim_vars("exec-capable", &[API_CAPABLE], 1),
    )
    .await;
    assert_eq!(data(&response, "claimAsyncJobs")["jobs"], json!([]));

    let rows = activity_as_staff(&schema, &pool, JsonValue::Null).await;
    let row = |principal: &str| {
        rows.iter()
            .find(|row| row["executorPrincipal"] == json!(principal))
            .unwrap_or_else(|| panic!("no activity row for {principal}: {rows:?}"))
            .clone()
    };
    assert_eq!(rows.len(), 2, "{rows:?}");
    for principal in [WORKER_ID, SECOND_WORKER_ID] {
        let found = row(principal);
        assert_eq!(found["executorName"], json!("exec-capable"));
        assert_eq!(found["kind"], json!(API_CAPABLE));
        assert!(!found["lastExecutionPollAt"].is_null());
        assert!(found["lastReconciliationPollAt"].is_null());
    }
    let first = row(WORKER_ID);
    assert!(!first["lastExecutionClaimAt"].is_null());
    assert!(!first["latestOpenHeartbeatAt"].is_null());
    let polled_only = row(SECOND_WORKER_ID);
    assert!(polled_only["lastExecutionClaimAt"].is_null());
    assert!(polled_only["latestOpenHeartbeatAt"].is_null());

    let hour_ago = (Utc::now() - Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Secs, true);
    let hour_ahead = (Utc::now() + Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Secs, true);
    assert_eq!(
        activity_as_staff(&schema, &pool, json!(hour_ago))
            .await
            .len(),
        2
    );
    assert!(activity_as_staff(&schema, &pool, json!(hour_ahead))
        .await
        .is_empty());
}

// ===========================================================================
// 19.2 API additions: one claim per request
// ===========================================================================

const DUPLICATED_KEY: &str = "mutation Twice($input: ClaimAsyncJobsInput!) { \
     claimAsyncJobs(input: $input) { jobs { asyncJobId claimToken } skippedKinds } \
     claimAsyncJobs(input: $input) { jobs { asyncJobId claimToken } skippedKinds } }";

const ALIASED: &str = "mutation Aliased($input: ClaimAsyncJobsInput!) { \
     a: claimAsyncJobs(input: $input) { jobs { asyncJobId claimToken } skippedKinds } \
     b: claimAsyncJobs(input: $input) { jobs { asyncJobId claimToken } skippedKinds } }";

const DUPLICATED_RECONCILIATION_KEY: &str = "mutation Twice($input: ClaimAsyncJobsInput!) { \
     claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId claimToken } skippedKinds } \
     claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId claimToken } skippedKinds } }";

const ALIASED_RECONCILIATION: &str = "mutation Aliased($input: ClaimAsyncJobsInput!) { \
     a: claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId claimToken } skippedKinds } \
     b: claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId claimToken } skippedKinds } }";

const X_THEN_Y: &str = "mutation Order($x: ClaimAsyncJobsInput!, $y: ClaimAsyncJobsInput!) { \
     x: claimAsyncJobs(input: $x) { jobs { asyncJobId } skippedKinds } \
     y: claimAsyncJobs(input: $y) { jobs { asyncJobId } skippedKinds } }";

const Y_THEN_X: &str = "mutation Order($x: ClaimAsyncJobsInput!, $y: ClaimAsyncJobsInput!) { \
     y: claimAsyncJobs(input: $y) { jobs { asyncJobId } skippedKinds } \
     x: claimAsyncJobs(input: $x) { jobs { asyncJobId } skippedKinds } }";

const EXECUTION_THEN_RECONCILIATION: &str = "mutation Mixed($input: ClaimAsyncJobsInput!) { \
     execution: claimAsyncJobs(input: $input) { jobs { asyncJobId } skippedKinds } \
     reconciliation: claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId } skippedKinds } }";

const RECONCILIATION_THEN_EXECUTION: &str = "mutation Mixed($input: ClaimAsyncJobsInput!) { \
     reconciliation: claimAsyncJobReconciliations(input: $input) { jobs { asyncJobId } skippedKinds } \
     execution: claimAsyncJobs(input: $input) { jobs { asyncJobId } skippedKinds } }";

const STORED_FAILURE: &str = "mutation Stored($same: ClaimAsyncJobsInput!, \
     $other: ClaimAsyncJobsInput!) { \
     first: claimAsyncJobs(input: $same) { jobs { asyncJobId } skippedKinds } \
     repeated: claimAsyncJobs(input: $same) { jobs { asyncJobId } skippedKinds } \
     other: claimAsyncJobs(input: $other) { jobs { asyncJobId } skippedKinds } }";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_duplicated_claim_key_and_an_aliased_identical_claim_return_one_stored_result_and_claim_once(
) {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();

    // Execution claims.
    for i in 0..3 {
        create(
            &mut c,
            new_job(API_EFFECT, &format!("memo-{i}"), &[&key(NS_CS)]),
        );
    }
    let response = run(
        &schema,
        &pool,
        &worker(),
        DUPLICATED_KEY,
        claim_vars("exec-1", &[API_EFFECT, "UNKNOWN_KIND"], 2),
    )
    .await;
    let result = data(&response, "claimAsyncJobs");
    assert_eq!(
        result["jobs"].as_array().map(Vec::len),
        Some(2),
        "{response}"
    );
    assert_eq!(result["skippedKinds"], json!(["UNKNOWN_KIND"]));
    assert_eq!(running(&mut c), 2, "the duplicated key claimed twice");
    assert_eq!(attempts(&mut c, "EXECUTION"), 2);

    let response = run(
        &schema,
        &pool,
        &worker(),
        ALIASED,
        claim_vars("exec-1", &[API_EFFECT], 1),
    )
    .await;
    let (a, b) = (data(&response, "a"), data(&response, "b"));
    assert_eq!(a, b, "the alias returns the stored result, token included");
    assert_eq!(a["jobs"].as_array().map(Vec::len), Some(1));
    assert_eq!(running(&mut c), 3, "the alias claimed again");
    assert_eq!(attempts(&mut c, "EXECUTION"), 3);

    // Reconciliation claims.
    let executor = dissemination_executor("fixture-executor");
    let fixtures = claimed(
        &mut c,
        &executor,
        vec![
            new_job(API_CAPABLE, "memo-rr-0", &[&key(NS_CS)]),
            new_job(API_CAPABLE, "memo-rr-1", &[&key(NS_CS)]),
        ],
    );
    for fixture in &fixtures {
        to_reconciliation(&mut c, fixture, ExecutorDomain::Dissemination);
    }
    let response = run(
        &schema,
        &pool,
        &worker(),
        DUPLICATED_RECONCILIATION_KEY,
        claim_vars("exec-1", &[API_CAPABLE], 1),
    )
    .await;
    let result = data(&response, "claimAsyncJobReconciliations");
    assert_eq!(
        result["jobs"].as_array().map(Vec::len),
        Some(1),
        "{response}"
    );
    assert_eq!(attempts(&mut c, "RECONCILIATION"), 1);

    let response = run(
        &schema,
        &pool,
        &worker(),
        ALIASED_RECONCILIATION,
        claim_vars("exec-1", &[API_CAPABLE], 1),
    )
    .await;
    let (a, b) = (data(&response, "a"), data(&response, "b"));
    assert_eq!(a, b);
    assert_eq!(a["jobs"].as_array().map(Vec::len), Some(1));
    assert_eq!(attempts(&mut c, "RECONCILIATION"), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_claim_field_with_other_arguments_is_refused_whichever_claim_field_runs_first() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let mut first_runs = BTreeSet::new();

    for document in [X_THEN_Y, Y_THEN_X] {
        test_db::reset_db(&pool).expect("reset the test database");
        for i in 0..3 {
            create(
                &mut c,
                new_job(API_EFFECT, &format!("order-{i}"), &[&key(NS_CS)]),
            );
        }
        let variables = json!({
            "x": claim_input("exec-x", &[API_EFFECT], 1),
            "y": claim_input("exec-y", &[API_EFFECT], 2),
        });
        let response = run(&schema, &pool, &worker(), document, variables).await;

        let error = only_error(&response);
        assert_eq!(error.kind, "ASYNC_JOB_DUPLICATE_CLAIM", "{response}");
        assert_eq!(
            error.message,
            ThothError::AsyncJobDuplicateClaim.to_string()
        );
        let refused = error.path.first().expect("a path").clone();
        let ran = if refused == "x" { "y" } else { "x" };
        // The refusal nulls only its own field.
        assert!(response["data"][&refused].is_null(), "{response}");
        let jobs = response["data"][ran]["jobs"]
            .as_array()
            .expect("the claim field that ran reports its jobs");
        let (name, other_name, max) = if ran == "x" {
            ("exec-x", "exec-y", 1)
        } else {
            ("exec-y", "exec-x", 2)
        };
        assert_eq!(jobs.len(), max, "{response}");
        let max = i64::try_from(max).expect("a small count");
        assert_eq!(running(&mut c), max);
        assert_eq!(attempts(&mut c, "EXECUTION"), max);
        assert_eq!(
            count(
                &mut c,
                &format!(
                    "SELECT count(*) AS value FROM public.async_job \
                      WHERE status = 'RUNNING' AND claimed_by = '{name}'"
                ),
            ),
            max
        );
        assert_eq!(
            count(
                &mut c,
                &format!(
                    "SELECT count(*) AS value FROM public.async_executor_poll \
                      WHERE executor_name = '{other_name}'"
                ),
            ),
            0,
            "the refused field reached the engine"
        );
        first_runs.insert(name);
    }

    assert_eq!(
        first_runs,
        BTreeSet::from(["exec-x", "exec-y"]),
        "both execution orders must be exercised"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_claim_and_a_reconciliation_claim_in_one_request_claim_once_whichever_runs_first() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let mut first_runs = BTreeSet::new();

    for document in [EXECUTION_THEN_RECONCILIATION, RECONCILIATION_THEN_EXECUTION] {
        test_db::reset_db(&pool).expect("reset the test database");
        let rr = claimed(
            &mut c,
            &executor,
            vec![new_job(API_EFFECT, "mixed-rr", &[&key(NS_CS)])],
        )
        .remove(0);
        to_reconciliation(&mut c, &rr, ExecutorDomain::Dissemination);
        let pending = create(&mut c, new_job(API_EFFECT, "mixed-pending", &[&key(NS_CS)]));

        let response = run(
            &schema,
            &pool,
            &worker(),
            document,
            claim_vars("exec-1", &[API_EFFECT], 5),
        )
        .await;
        let error = only_error(&response);
        assert_eq!(error.kind, "ASYNC_JOB_DUPLICATE_CLAIM", "{response}");
        let refused = error.path.first().expect("a path").clone();
        let ran = if refused == "execution" {
            "reconciliation"
        } else {
            "execution"
        };
        assert!(response["data"][&refused].is_null(), "{response}");
        let jobs = response["data"][ran]["jobs"]
            .as_array()
            .expect("the claim field that ran reports its jobs");
        assert_eq!(jobs.len(), 1, "{response}");

        if ran == "execution" {
            assert_eq!(uuid_of(&jobs[0]["asyncJobId"]), pending.async_job_id);
            assert_eq!(status(&mut c, pending.async_job_id), "RUNNING");
            assert_eq!(job_column(&mut c, rr.id, "claimed_by"), None);
        } else {
            assert_eq!(uuid_of(&jobs[0]["asyncJobId"]), rr.id);
            assert_eq!(status(&mut c, pending.async_job_id), "PENDING");
            assert_eq!(
                job_column(&mut c, rr.id, "claimed_by").as_deref(),
                Some("exec-1")
            );
        }
        first_runs.insert(ran);
    }

    assert_eq!(
        first_runs,
        BTreeSet::from(["execution", "reconciliation"]),
        "both execution orders must be exercised"
    );
}

/// The backend waiting for an advisory lock in the per-kind claim
/// namespace, found by bounded polling of `pg_locks` (never sleeping).
fn kind_lock_waiter(observer: &mut PgConnection) -> i64 {
    let deadline = Instant::now() + StdDuration::from_secs(60);
    loop {
        let pid = count(
            observer,
            &format!(
                "SELECT coalesce(min(pid), 0)::bigint AS value FROM pg_locks \
                  WHERE locktype = 'advisory' AND NOT granted \
                    AND classid = '{NS_KIND}'::oid AND objsubid = 2"
            ),
        );
        if pid != 0 {
            return pid;
        }
        assert!(
            Instant::now() < deadline,
            "the claim never waited for the kind lock"
        );
        thread::yield_now();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_identical_claim_field_after_a_failed_claim_returns_the_stored_error_without_claiming() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let job = create(
        &mut c,
        new_job(API_EFFECT, "stored-failure", &[&key(NS_CS)]),
    );
    let deadlocks_before = deadlocks(&mut c);
    let before = footprint(&mut c);

    // A blocker holds the per-kind claim lock, so the first claim field
    // waits for it. Once the claim is seen waiting, its statement is
    // cancelled (ASYNC_JOB_BUSY) and the lock is released: a second claim
    // after that point would claim the job.
    let lock = format!("{NS_KIND}, hashtext('{API_EFFECT}')");
    let mut blocker = connect();
    exec(&mut blocker, &format!("SELECT pg_advisory_lock({lock})"));
    let observer = tokio::task::spawn_blocking(move || {
        let mut observer = connect();
        let pid = kind_lock_waiter(&mut observer);
        exec(&mut observer, &format!("SELECT pg_cancel_backend({pid})"));
        exec(&mut blocker, &format!("SELECT pg_advisory_unlock({lock})"));
        pid
    });
    let variables = json!({
        "same": claim_input("exec-1", &[API_EFFECT], 1),
        "other": claim_input("exec-2", &[API_EFFECT], 1),
    });
    let response = run(&schema, &pool, &worker(), STORED_FAILURE, variables).await;
    let pid = observer.await.expect("the observer");
    assert!(pid > 0);

    // Juniper runs top-level fields in document order, so `first` claimed.
    let failures: HashMap<String, FieldFailure> = errors(&response)
        .iter()
        .map(|error| {
            let failure = failure(error);
            (failure.path.join("."), failure)
        })
        .collect();
    assert_eq!(failures.len(), 3, "{response}");
    let busy = ThothError::AsyncJobBusy.to_string();
    for alias in ["first", "repeated"] {
        assert_eq!(
            (
                failures[alias].kind.as_str(),
                failures[alias].message.as_str()
            ),
            ("ASYNC_JOB_BUSY", busy.as_str()),
            "{alias}: {response}"
        );
    }
    assert_eq!(failures["other"].kind, "ASYNC_JOB_DUPLICATE_CLAIM");
    for alias in ["first", "repeated", "other"] {
        assert!(response["data"][alias].is_null(), "{response}");
    }

    assert_eq!(status(&mut c, job.async_job_id), "PENDING");
    assert_eq!(attempts(&mut c, "EXECUTION"), 0);
    assert_eq!(footprint(&mut c), before, "a second claim ran");
    assert_eq!(deadlocks(&mut c), deadlocks_before);

    // The lock is free again: a new request claims the job.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-1", &[API_EFFECT], 1),
    )
    .await;
    assert_eq!(
        data(&response, "claimAsyncJobs")["jobs"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

// ===========================================================================
// 19.2: operator command audit rows
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_operator_command_writes_one_audit_row_and_a_repeat_replays_it_without_another() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let jobs = command_jobs(&mut c);
    let cases = [
        // A refusal is recorded too: a RECONCILIATION_REQUIRED job cannot be
        // cancelled.
        (
            "cancelAsyncJob",
            CANCEL,
            jobs.reconciliation,
            "CANCEL_JOB",
            "REFUSED",
            "NOT_CANCELLABLE",
        ),
        (
            "cancelAsyncJob",
            CANCEL,
            jobs.pending,
            "CANCEL_JOB",
            "APPLIED",
            "CANCELLED",
        ),
        (
            "retryAsyncJob",
            RETRY,
            jobs.failed,
            "RETRY_JOB",
            "APPLIED",
            "RETRY_CREATED",
        ),
        (
            "resolveAsyncJobReconciliationAsStaff",
            RESOLVE_AS_STAFF,
            jobs.reconciliation,
            "RESOLVE_RECONCILIATION",
            "APPLIED",
            "NO_EFFECT_PROVEN",
        ),
        (
            "redactAsyncJobDiagnostics",
            REDACT,
            jobs.terminal,
            "REDACT_JOB_DIAGNOSTICS",
            "APPLIED",
            "REDACTED",
        ),
    ];

    for (field, document, job, kind, outcome, code) in cases {
        let request_id = Uuid::new_v4();
        let reason = format!("{kind} audit reason");
        let variables = if field == "resolveAsyncJobReconciliationAsStaff" {
            staff_failed_vars(job, &reason, request_id)
        } else {
            command_vars(job, &reason, request_id)
        };
        let rows = commands(&mut c);
        let job_before = job_row(&mut c, job);

        let response = run(&schema, &pool, &staff(), document, variables.clone()).await;
        let result = data(&response, field).clone();
        assert_eq!(
            (
                result["commandKind"].clone(),
                result["outcome"].clone(),
                result["outcomeCode"].clone(),
                result["replayed"].clone()
            ),
            (json!(kind), json!(outcome), json!(code), json!(false)),
            "{response}"
        );
        assert_eq!(commands(&mut c), rows + 1, "{field} wrote one command row");
        let command_id = uuid_of(&result["asyncControlCommandId"]);
        let result_job = result["resultJob"]["asyncJobId"]
            .as_str()
            .unwrap_or("-")
            .to_string();
        assert_eq!(
            command_row(&mut c, command_id),
            format!("{kind}|{request_id}|{STAFF_ID}|{reason}|{job}|{outcome}|{code}|{result_job}")
        );
        if outcome == "REFUSED" {
            assert_eq!(job_row(&mut c, job), job_before, "a refusal changes no job");
        }
        if field == "retryAsyncJob" {
            assert_eq!(result["resultJob"]["lineage"], json!("MANUAL_RETRY"));
            assert_eq!(result["resultJob"]["parentJobId"], json!(job.to_string()));
            assert_eq!(result["resultJob"]["status"], json!("PENDING"));
        }

        // The same request ID and inputs return the stored decision, with no
        // new row.
        let replay = run(&schema, &pool, &staff(), document, variables).await;
        let replayed = data(&replay, field);
        assert_eq!(replayed["replayed"], json!(true), "{replay}");
        assert_eq!(
            replayed["asyncControlCommandId"],
            result["asyncControlCommandId"]
        );
        assert_eq!(replayed["outcomeCode"], result["outcomeCode"]);
        assert_eq!(commands(&mut c), rows + 1, "a replay wrote a row");
    }

    // The same request ID for another job is a conflict, and writes nothing.
    let request_id = Uuid::new_v4();
    let response = run(
        &schema,
        &pool,
        &staff(),
        REDACT,
        command_vars(jobs.terminal, "redact again", request_id),
    )
    .await;
    assert_eq!(
        data(&response, "redactAsyncJobDiagnostics")["outcomeCode"],
        json!("NOTHING_TO_REDACT")
    );
    let before = footprint(&mut c);
    let response = run(
        &schema,
        &pool,
        &staff(),
        REDACT,
        command_vars(jobs.failed, "redact again", request_id),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_COMMAND_REQUEST_CONFLICT",
        &ThothError::AsyncCommandRequestConflict.to_string(),
    );
    assert_eq!(footprint(&mut c), before);

    // The redaction is visible on the superuser read.
    let response = run(
        &schema,
        &pool,
        &staff(),
        READ_JOB,
        json!({ "id": jobs.terminal }),
    )
    .await;
    let detail = data(&response, "asyncJob");
    assert!(!detail["job"]["redactedAt"].is_null());
    assert!(detail["attempts"][0]["errorDetail"].is_null());
}

// ===========================================================================
// 14.4 and 19.2: error codes
// ===========================================================================

#[test]
fn every_async_error_maps_to_its_stable_graphql_type_and_its_display_message() {
    let cases = [
        (ThothError::AsyncJobNotFound, "ASYNC_JOB_NOT_FOUND"),
        (ThothError::AsyncJobKindUnknown, "ASYNC_JOB_KIND_UNKNOWN"),
        (
            ThothError::AsyncJobKindAdmissionClosed,
            "ASYNC_JOB_KIND_ADMISSION_CLOSED",
        ),
        (
            ThothError::AsyncJobKindNotAuthorized,
            "ASYNC_JOB_KIND_NOT_AUTHORIZED",
        ),
        (
            ThothError::AsyncJobPayloadInvalid("a reason".to_string()),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            ThothError::AsyncJobPayloadVersionUnsupported,
            "ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED",
        ),
        (
            ThothError::AsyncJobIdempotencyConflict,
            "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
        ),
        (ThothError::AsyncJobStaleClaim, "ASYNC_JOB_STALE_CLAIM"),
        (ThothError::AsyncJobLeaseExpired, "ASYNC_JOB_LEASE_EXPIRED"),
        (
            ThothError::AsyncJobTransitionRefused("PHASE".to_string()),
            "ASYNC_JOB_TRANSITION_REFUSED",
        ),
        (
            ThothError::AsyncJobEffectWindowInsufficient,
            "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
        ),
        (
            ThothError::AsyncJobEvidenceInvalid("a reason".to_string()),
            "ASYNC_JOB_EVIDENCE_INVALID",
        ),
        (
            ThothError::AsyncJobIsolationInvalid,
            "ASYNC_JOB_ISOLATION_INVALID",
        ),
        (
            ThothError::AsyncJobIntegrityViolation,
            "ASYNC_JOB_INTEGRITY_VIOLATION",
        ),
        (ThothError::AsyncJobBusy, "ASYNC_JOB_BUSY"),
        (
            ThothError::AsyncJobDuplicateClaim,
            "ASYNC_JOB_DUPLICATE_CLAIM",
        ),
        (
            ThothError::AsyncCommandRequestConflict,
            "ASYNC_COMMAND_REQUEST_CONFLICT",
        ),
    ];
    for (error, code) in cases {
        let message = error.to_string();
        let field_error = error.into_field_error();
        assert_eq!(field_error.message(), message);
        let extensions =
            serde_json::to_value(field_error.extensions()).expect("serialize the extensions");
        assert_eq!(extensions["type"], json!(code));
        let reason = (code == "ASYNC_JOB_TRANSITION_REFUSED").then(|| json!("PHASE"));
        assert_eq!(extensions.get("reason").cloned(), reason, "{code}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_reachable_error_code_reaches_the_client_with_its_type_and_message() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let fixtures = claimed(
        &mut c,
        &executor,
        (0..6)
            .map(|i| new_job(API_EFFECT, &format!("errors-{i}"), &[&key(NS_CS)]))
            .collect(),
    );

    // ASYNC_JOB_NOT_FOUND.
    let response = run(
        &schema,
        &pool,
        &staff(),
        READ_JOB,
        json!({ "id": Uuid::new_v4() }),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_NOT_FOUND",
        &ThothError::AsyncJobNotFound.to_string(),
    );

    // ASYNC_JOB_STALE_CLAIM: another token, and a job that does not exist.
    let stale = ThothError::AsyncJobStaleClaim.to_string();
    for fixture in [
        Fixture {
            token: Uuid::new_v4(),
            ..fixtures[0].clone()
        },
        Fixture {
            id: Uuid::new_v4(),
            ..fixtures[0].clone()
        },
    ] {
        let (document, variables) = fenced("renewAsyncJobLease", &fixture);
        let response = run(&schema, &pool, &worker(), document, variables).await;
        assert_error(&response, "ASYNC_JOB_STALE_CLAIM", &stale);
    }

    // ASYNC_JOB_KIND_UNKNOWN: a binary that does not know the job's kind.
    let (document, variables) = fenced("renewAsyncJobLease", &fixtures[0]);
    let response = execute(
        &schema,
        &context(&pool, &worker(), &NARROW_REGISTRY),
        document,
        variables,
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_KIND_UNKNOWN",
        &ThothError::AsyncJobKindUnknown.to_string(),
    );

    // ASYNC_JOB_KIND_NOT_AUTHORIZED.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-1", &[API_LOCAL], 1),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_KIND_NOT_AUTHORIZED",
        &ThothError::AsyncJobKindNotAuthorized.to_string(),
    );

    // ASYNC_JOB_PAYLOAD_INVALID: a bound out of range, and a payload version
    // outside `smallint`.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        claim_vars("exec-1", &[API_EFFECT], 0),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_PAYLOAD_INVALID",
        &ThothError::AsyncJobPayloadInvalid("max_jobs must be between 1 and 50".to_string())
            .to_string(),
    );
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM,
        json!({ "input": {
            "claimedBy": "exec-1",
            "kinds": [{ "kind": API_EFFECT, "payloadVersions": [40000] }],
            "maxJobs": 1,
        } }),
    )
    .await;
    assert_error(
        &response,
        "ASYNC_JOB_PAYLOAD_INVALID",
        &ThothError::AsyncJobPayloadInvalid(
            "a payload version is outside the smallint range".to_string(),
        )
        .to_string(),
    );

    // ASYNC_JOB_LEASE_EXPIRED.
    exec(
        &mut c,
        &format!(
            "UPDATE public.async_job SET lease_expires_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{}'",
            fixtures[1].id
        ),
    );
    let (document, variables) = fenced("renewAsyncJobLease", &fixtures[1]);
    let response = run(&schema, &pool, &worker(), document, variables).await;
    assert_error(
        &response,
        "ASYNC_JOB_LEASE_EXPIRED",
        &ThothError::AsyncJobLeaseExpired.to_string(),
    );

    // ASYNC_JOB_TRANSITION_REFUSED with its reason: proven no-effect at
    // PRE_WRITE. The claim stays live.
    let mut input = fixtures[2].input();
    input["failureClass"] = json!("PERMANENT");
    input["errorCode"] = json!("TEST_FAILURE");
    input["noEffectProven"] = json!(true);
    input["evidence"] = json!(no_effect_evidence());
    let response = run(&schema, &pool, &worker(), FAIL, json!({ "input": input })).await;
    let error = only_error(&response);
    assert_eq!(
        (
            error.kind.as_str(),
            error.message.clone(),
            error.reason.as_deref()
        ),
        (
            "ASYNC_JOB_TRANSITION_REFUSED",
            ThothError::AsyncJobTransitionRefused("PHASE".to_string()).to_string(),
            Some("PHASE")
        ),
        "{response}"
    );

    // ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT: 20 seconds of lease, a 30-second
    // window.
    exec(
        &mut c,
        &format!(
            "UPDATE public.async_job SET lease_expires_at = clock_timestamp() + interval '20 seconds' \
              WHERE async_job_id = '{}'",
            fixtures[3].id
        ),
    );
    let (document, variables) = fenced("markAsyncJobEffectStarted", &fixtures[3]);
    let response = run(&schema, &pool, &worker(), document, variables).await;
    assert_error(
        &response,
        "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
        &ThothError::AsyncJobEffectWindowInsufficient.to_string(),
    );

    // ASYNC_JOB_EVIDENCE_INVALID: evidence on a failure that proves nothing.
    let mut input = fixtures[4].input();
    input["failureClass"] = json!("PERMANENT");
    input["errorCode"] = json!("TEST_FAILURE");
    input["noEffectProven"] = json!(false);
    input["evidence"] = json!(no_effect_evidence());
    let response = run(&schema, &pool, &worker(), FAIL, json!({ "input": input })).await;
    let error = only_error(&response);
    assert_eq!(error.kind, "ASYNC_JOB_EVIDENCE_INVALID", "{response}");
    assert!(
        error
            .message
            .starts_with("The async job evidence is invalid: "),
        "{response}"
    );

    // Both refusals left their claims live.
    for fixture in [&fixtures[2], &fixtures[4]] {
        assert_eq!(status(&mut c, fixture.id), "RUNNING");
        let (document, variables) = fenced("renewAsyncJobLease", fixture);
        let response = run(&schema, &pool, &worker(), document, variables).await;
        data(&response, "renewAsyncJobLease");
    }

    // ASYNC_JOB_BUSY: no database connection can be had.
    let unreachable = Arc::new(test_db::failing_pool());
    let busy = ThothError::AsyncJobBusy.to_string();
    let (renew, renew_variables) = fenced("renewAsyncJobLease", &fixtures[5]);
    for (caller, document, variables) in [
        (worker(), CLAIM, claim_vars("exec-1", &[API_EFFECT], 1)),
        (worker(), renew, renew_variables),
        (staff(), READ_JOB, json!({ "id": fixtures[5].id })),
        (
            staff(),
            CANCEL,
            command_vars(fixtures[5].id, "cancel", Uuid::new_v4()),
        ),
    ] {
        let response = execute(
            &schema,
            &context(&unreachable, &caller, &API_REGISTRY),
            document,
            variables,
        )
        .await;
        assert_error(&response, "ASYNC_JOB_BUSY", &busy);
    }
}

// ===========================================================================
// 14.1: `BigInt` evidence orders
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_bigint_order_reaches_the_engine_untruncated_and_only_the_scalar_refuses_malformed_text()
{
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let target = key(NS_RB);
    let fixture = claimed(
        &mut c,
        &executor,
        vec![revision_job("bigint", &target, 5, "h5")],
    )
    .remove(0);
    start(&mut c, &fixture, ExecutorDomain::Dissemination);
    let complete = |order: &str| {
        let mut input = fixture.input();
        input["outcome"] = json!("EFFECT_APPLIED");
        input["entries"] = json!([{ "targetKey": target, "order": order, "fingerprint": "h5" }]);
        json!({ "input": input })
    };

    // Outside `bigint`: the engine's shape fault, nothing written, claim live.
    let before = footprint(&mut c);
    for order in ["9223372036854775808", "-9223372036854775809"] {
        let response = run(&schema, &pool, &worker(), COMPLETE, complete(order)).await;
        let error = only_error(&response);
        assert_eq!(
            error.kind, "ASYNC_JOB_EVIDENCE_INVALID",
            "{order}: {response}"
        );
        assert!(error
            .message
            .starts_with("The async job evidence is invalid: "));
        assert_eq!(footprint(&mut c), before);
    }
    // Not canonical, or outside `i128`: the scalar refuses it during input
    // coercion, before the engine runs.
    for order in [
        "007",
        "1e3",
        "+5",
        " 5",
        "170141183460469231731687303715884105728",
    ] {
        let response = run(&schema, &pool, &worker(), COMPLETE, complete(order)).await;
        assert_input_error(&response);
        assert_eq!(footprint(&mut c), before);
    }
    // The claim is still live: the job's own order completes it.
    let response = run(&schema, &pool, &worker(), COMPLETE, complete("5")).await;
    assert_eq!(
        data(&response, "completeAsyncJob")["status"],
        json!("SUCCEEDED")
    );
    assert_eq!(
        text(
            &mut c,
            &format!(
                "SELECT applied_order::text AS value FROM public.async_effect_target_state \
                  WHERE target_key = '{target}'"
            ),
        )
        .as_deref(),
        Some("5")
    );

    // A staff command: malformed text is an input error with no command row,
    // and an order outside `bigint` is the engine's committed REFUSED.
    let other = key(NS_RB);
    let rr = claimed(
        &mut c,
        &executor,
        vec![revision_job("bigint-rr", &other, 7, "h7")],
    )
    .remove(0);
    to_reconciliation(&mut c, &rr, ExecutorDomain::Dissemination);
    let staff_vars = |order: &str| {
        json!({ "input": {
            "asyncJobId": rr.id,
            "outcome": "SUCCEEDED",
            "entries": [{ "targetKey": other, "order": order, "fingerprint": "h7" }],
            "reason": "the provider holds revision 7",
            "requestId": Uuid::new_v4(),
        } })
    };
    let rows = commands(&mut c);
    let response = run(
        &schema,
        &pool,
        &staff(),
        RESOLVE_AS_STAFF,
        staff_vars("007"),
    )
    .await;
    assert_input_error(&response);
    assert_eq!(commands(&mut c), rows);
    let response = run(
        &schema,
        &pool,
        &staff(),
        RESOLVE_AS_STAFF,
        staff_vars("9223372036854775808"),
    )
    .await;
    let result = data(&response, "resolveAsyncJobReconciliationAsStaff");
    assert_eq!(
        (result["outcome"].clone(), result["outcomeCode"].clone()),
        (json!("REFUSED"), json!("EVIDENCE_INVALID")),
        "{response}"
    );
    assert_eq!(commands(&mut c), rows + 1);
    assert_eq!(status(&mut c, rr.id), "RECONCILIATION_REQUIRED");
}

// ===========================================================================
// 19.2 Accepted regression (E2): through the API
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_accepted_regression_is_recorded_through_the_api_and_cleared_only_by_ordinary_progress()
{
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let mut c = connect();
    let executor = dissemination_executor("fixture-executor");
    let target = key(NS_RB);
    forward_write(&mut c, &executor, &target, 9);

    // The older revision 4 writes its effect: its completion is T6x.
    let older = claimed(
        &mut c,
        &executor,
        vec![revision_job("older-4", &target, 4, "h4")],
    )
    .remove(0);
    start(&mut c, &older, ExecutorDomain::Dissemination);
    let mut input = older.input();
    input["outcome"] = json!("EFFECT_APPLIED");
    input["entries"] = json!([{ "targetKey": target, "order": "4", "fingerprint": "h4" }]);
    let response = run(
        &schema,
        &pool,
        &worker(),
        COMPLETE,
        json!({ "input": input }),
    )
    .await;
    assert_eq!(
        data(&response, "completeAsyncJob")["status"],
        json!("RECONCILIATION_REQUIRED")
    );

    // An executor reconciler's regressing SUCCEEDED is refused: nothing is
    // written and the claim stays live.
    let response = run(
        &schema,
        &pool,
        &worker(),
        CLAIM_RECONCILIATIONS,
        claim_vars("exec-1", &[API_REVISION], 1),
    )
    .await;
    let claimed_job = data(&response, "claimAsyncJobReconciliations")["jobs"][0].clone();
    assert_eq!(uuid_of(&claimed_job["asyncJobId"]), older.id);
    let reconciliation = Fixture {
        token: uuid_of(&claimed_job["claimToken"]),
        ..older.clone()
    };
    let before = footprint(&mut c);
    let mut input = reconciliation.input();
    input["outcome"] = json!("SUCCEEDED");
    input["entries"] = json!([{ "targetKey": target, "order": "4", "fingerprint": "h4" }]);
    input["evidence"] = json!(applied_evidence(4));
    let response = run(
        &schema,
        &pool,
        &worker(),
        RESOLVE,
        json!({ "input": input }),
    )
    .await;
    assert_eq!(
        only_error(&response).kind,
        "ASYNC_JOB_EVIDENCE_INVALID",
        "{response}"
    );
    assert_eq!(footprint(&mut c), before);
    let (document, variables) = fenced("asyncJobTargetState", &reconciliation);
    let response = run(&schema, &pool, &worker(), document, variables).await;
    let state = &data(&response, "asyncJobTargetState")[0];
    assert_eq!(
        (
            state["appliedOrder"].clone(),
            state["appliedFingerprint"].clone()
        ),
        (json!("9"), json!("h9"))
    );
    assert!(state["regressedAt"].is_null());
    let (document, variables) = fenced("resolveAsyncJobReconciliation", &reconciliation);
    let response = run(&schema, &pool, &worker(), document, variables).await;
    assert_eq!(
        data(&response, "resolveAsyncJobReconciliation")["status"],
        json!("RECONCILIATION_REQUIRED")
    );

    // Staff accept the regression with applied evidence of revision 4.
    let accept = |job: Uuid, order: i64| {
        json!({ "input": {
            "asyncJobId": job,
            "outcome": "ACCEPT_REGRESSION",
            "evidence": applied_evidence(order),
            "reason": "the provider holds the older revision",
            "requestId": Uuid::new_v4(),
        } })
    };
    let response = run(
        &schema,
        &pool,
        &staff(),
        RESOLVE_AS_STAFF,
        accept(older.id, 4),
    )
    .await;
    let result = data(&response, "resolveAsyncJobReconciliationAsStaff");
    assert_eq!(
        (
            result["outcome"].clone(),
            result["outcomeCode"].clone(),
            result["job"]["status"].clone()
        ),
        (
            json!("APPLIED"),
            json!("REGRESSION_ACCEPTED"),
            json!("SUCCEEDED")
        ),
        "{response}"
    );
    assert_eq!(holds(&mut c, older.id), 0, "the holds are released");

    // The signals list the target with the replaced revision.
    let signals = revision_signals(&schema, &pool).await;
    assert_eq!(signals["regressedTargetCount"], json!("1"));
    let regressed = &signals["regressedTargets"][0];
    assert_eq!(regressed["targetKey"], json!(target));
    assert_eq!(
        (
            regressed["appliedOrder"].clone(),
            regressed["appliedFingerprint"].clone(),
            regressed["regressedFromOrder"].clone(),
            regressed["regressedFromFingerprint"].clone()
        ),
        (json!("4"), json!("h4"), json!("9"), json!("h9"))
    );
    let regressed_at = regressed["regressedAt"].clone();
    assert!(!regressed_at.is_null());
    let columns = regressed_columns(&mut c, &target);

    // An ordinary write below the pre-acceptance revision keeps it.
    forward_write(&mut c, &executor, &target, 6);
    let signals = revision_signals(&schema, &pool).await;
    assert_eq!(signals["regressedTargetCount"], json!("1"));
    let regressed = &signals["regressedTargets"][0];
    assert_eq!(
        (
            regressed["appliedOrder"].clone(),
            regressed["regressedAt"].clone(),
            regressed["regressedFromOrder"].clone()
        ),
        (json!("6"), regressed_at, json!("9"))
    );
    assert_eq!(regressed_columns(&mut c, &target), columns);

    // One above it clears all three columns.
    forward_write(&mut c, &executor, &target, 10);
    let signals = revision_signals(&schema, &pool).await;
    assert_eq!(signals["regressedTargetCount"], json!("0"));
    assert_eq!(signals["regressedTargets"], json!([]));
    assert_eq!(regressed_columns(&mut c, &target), "||");

    // So does an equal order with the pre-acceptance fingerprint, shown on a
    // second target.
    let second = key(NS_RB);
    forward_write(&mut c, &executor, &second, 8);
    let (older, outcome) = complete_revision(&mut c, &executor, &second, 3);
    assert!(
        matches!(outcome, CompleteOutcome::ReconciliationRequired(_)),
        "{outcome:?}"
    );
    let response = run(
        &schema,
        &pool,
        &staff(),
        RESOLVE_AS_STAFF,
        accept(older.id, 3),
    )
    .await;
    assert_eq!(
        data(&response, "resolveAsyncJobReconciliationAsStaff")["outcomeCode"],
        json!("REGRESSION_ACCEPTED")
    );
    assert_eq!(
        revision_signals(&schema, &pool).await["regressedTargetCount"],
        json!("1")
    );
    forward_write(&mut c, &executor, &second, 8);
    assert_eq!(
        revision_signals(&schema, &pool).await["regressedTargetCount"],
        json!("0")
    );
    assert_eq!(regressed_columns(&mut c, &second), "||");
}

// ===========================================================================
// 17 and 19.2: the generated SDL
// ===========================================================================

/// The 40 declarations the API adds; the base SDL names nothing `Async`.
const NEW_DECLARATIONS: [&str; 40] = [
    "enum AsyncAttemptPurpose {",
    "enum AsyncAttemptResult {",
    "enum AsyncControlCommandKind {",
    "enum AsyncControlCommandOutcome {",
    "enum AsyncEffectPhase {",
    "enum AsyncJobCompletionOutcome {",
    "enum AsyncJobEffectClass {",
    "enum AsyncJobFailureClass {",
    "enum AsyncJobLineage {",
    "enum AsyncJobReconcilerOutcome {",
    "enum AsyncJobRevisionMode {",
    "enum AsyncJobStaffOutcome {",
    "enum AsyncJobStatus {",
    "input AsyncJobClaimInput {",
    "input AsyncJobCommandInput {",
    "input AsyncJobEvidenceEntryInput {",
    "input AsyncJobFilterInput {",
    "input AsyncJobKindVersionsInput {",
    "input AsyncJobListPositionInput {",
    "input ClaimAsyncJobsInput {",
    "input CompleteAsyncJobInput {",
    "input FailAsyncJobInput {",
    "input MarkAsyncJobEffectConfirmedInput {",
    "input RenewAsyncJobLeaseInput {",
    "input ResolveAsyncJobReconciliationAsStaffInput {",
    "input ResolveAsyncJobReconciliationInput {",
    "input WaitAsyncJobInput {",
    "type AsyncJob {",
    "type AsyncJobAttempt {",
    "type AsyncJobBlockedJob {",
    "type AsyncJobClaimResult {",
    "type AsyncJobCommandResult {",
    "type AsyncJobDetail {",
    "type AsyncJobExecutorActivity {",
    "type AsyncJobKindSignals {",
    "type AsyncJobLeaseRenewal {",
    "type AsyncJobRegressedTarget {",
    "type AsyncJobTargetState {",
    "type AsyncJobTransition {",
    "type ClaimedAsyncJob {",
];

/// The 19 root fields the API adds: 14 mutations and 5 queries.
const NEW_ROOT_FIELDS: [&str; 19] = [
    "claimAsyncJobs",
    "renewAsyncJobLease",
    "markAsyncJobEffectStarted",
    "markAsyncJobEffectConfirmed",
    "waitAsyncJob",
    "completeAsyncJob",
    "failAsyncJob",
    "releaseAsyncJob",
    "claimAsyncJobReconciliations",
    "resolveAsyncJobReconciliation",
    "cancelAsyncJob",
    "retryAsyncJob",
    "resolveAsyncJobReconciliationAsStaff",
    "redactAsyncJobDiagnostics",
    "asyncJobTargetState",
    "asyncJob",
    "asyncJobs",
    "asyncJobKindSignals",
    "asyncJobExecutorActivity",
];

const ROOTS: [&str; 2] = ["type QueryRoot {", "type MutationRoot {"];

/// The SHA-256 of each BE-04 declaration's SDL chunk, computed on the base
/// commit; a byte change in any of them fails.
const BE04_DECLARATIONS: [(&str, &str); 12] = [
    (
        "enum DistributionJobAttemptResult {",
        "55f1739a1a248b3cfa931e227cf1e8512bb146de29f99d284f2f454cd278d02d",
    ),
    (
        "enum DistributionJobCancellationReason {",
        "a9f516acc48a202519a92c988bd930f1cc82f3192a1c25ee48fe31e5b7c3a6ce",
    ),
    (
        "enum DistributionJobKind {",
        "8837f6bd1307810c320467c4121037748b0e11bf2c9b7d47cd34319eeaf9867a",
    ),
    (
        "enum DistributionJobStatus {",
        "7a95563d346d1f195dc988d71de404a8ab44a1f06fadf52c0fc7b689a2244e94",
    ),
    (
        "input CancelDistributionJobInput {",
        "c83a0dab628655884fe743f7403a86e404dbc068022b588474126ece712df065",
    ),
    (
        "input ClaimDistributionJobsInput {",
        "cc608c2e1af86ef986d9aaf9b6cce979cb37f14b6cba400617da96199ac3d024",
    ),
    (
        "input CompleteDistributionJobInput {",
        "15f99ec4031a16f01656a0189a2383b0e3d1c1411b303fb6098b9df7f5f51dba",
    ),
    (
        "input FailDistributionJobInput {",
        "220e35b2b890060ae0585462fd64a498e199ba0cefb5aea89bbd6cc21a50b518",
    ),
    (
        "type ClaimedDistributionJob {",
        "27058c957a3000f3592350d46485fcd3382fa88a58d5b65e5ec5001b13d33bd6",
    ),
    (
        "type DistributionJob {",
        "58a51485abb547e94fa4946ffa725526726251d7fbe1fc5f81b7e6abd82fe713",
    ),
    (
        "type DistributionJobAttempt {",
        "928cdfb43f5dcfdaefd5ccc88d160479720a10fe15c0ae74e74bb19c51f592f4",
    ),
    (
        "type DistributionJobTarget {",
        "801096add3111e0f86c831636eb463d124b02bcb9861df452a8abf1b7358aecd",
    ),
];

/// The SHA-256 of each BE-04 field's description and declaration lines,
/// computed on the base commit.
const BE04_FIELDS: [(&str, &str, &str); 7] = [
    (
        "type MutationRoot {",
        "claimDistributionJobs(",
        "772771061bf58cef7b66ee34a9a294ac9c1a5648fe0046de404358d696b078f5",
    ),
    (
        "type MutationRoot {",
        "completeDistributionJob(",
        "b1a7d04e44f7a60ee48b46f38d7b840dafe10e056ae8baa8764c8fa523212429",
    ),
    (
        "type MutationRoot {",
        "failDistributionJob(",
        "bb3786b991fb2210e0b01bdb14e849cc4d44b1ae4fe5009ed033e28a724be058",
    ),
    (
        "type MutationRoot {",
        "cancelDistributionJob(",
        "db83faa10603adf61c4cec51046e0351cefff9fec2a9876ece9a8116383e3fa0",
    ),
    (
        "type QueryRoot {",
        "publisherServiceConfigurations(",
        "be6fd342f04afe6602138e47a164cd889d3dba2552b350f466a74d9e32aac576",
    ),
    (
        "type QueryRoot {",
        "publisherServiceConfigurationCount(",
        "7d3d4f6937aa275ed914bc447c445ed9e11f3bb1281014c7a512d1d444c038bf",
    ),
    (
        "type PublisherServiceConfigurationSummary {",
        "latestBackCatalogueJob:",
        "7578d3d56dce81936ba4cc266f094818106665ed6fa0e5552c548158e626d3e3",
    ),
];

fn sdl() -> String {
    create_schema().as_sdl()
}

/// `text` with every quoted string emptied, so a name in a description is
/// never taken for a reference.
fn without_strings(text: &str) -> String {
    Regex::new(r#""(?:[^"\\]|\\.)*""#)
        .expect("the string pattern")
        .replace_all(text, "\"\"")
        .into_owned()
}

fn word(name: &str) -> Regex {
    Regex::new(&format!(r"\b{name}\b")).expect("a word pattern")
}

/// Every type, input, enum, scalar, union or interface declaration line.
fn declaration_lines(sdl: &str) -> Vec<&str> {
    sdl.lines()
        .filter(|line| {
            [
                "type ",
                "input ",
                "enum ",
                "scalar ",
                "union ",
                "interface ",
            ]
            .iter()
            .any(|keyword| line.starts_with(keyword))
        })
        .collect()
}

/// The (description, declaration) line pairs of a root type's fields.
fn root_fields(sdl: &str, root: &str) -> Vec<(String, String)> {
    let block = sdl_block(sdl, root);
    assert!(
        !block.contains("\"\"\""),
        "a block string would hide field lines"
    );
    let lines: Vec<&str> = block.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with("  ") && !line.trim_start().starts_with('"'))
        .map(|(index, line)| {
            let description = index
                .checked_sub(1)
                .map(|before| lines[before])
                .filter(|before| before.trim_start().starts_with('"'))
                .unwrap_or_default();
            (description.to_string(), line.to_string())
        })
        .collect()
}

fn field_name(line: &str) -> &str {
    line.trim_start()
        .split(['(', ':'])
        .next()
        .unwrap_or_default()
        .trim()
}

/// The (description, member) pairs of an input object's fields or an enum's
/// values. The SDL writes such a member's description on the member's own
/// line, before it.
fn inline_members(sdl: &str, declaration: &str) -> Vec<(String, String)> {
    let block = sdl_block(sdl, declaration);
    assert!(
        !block.contains("\"\"\""),
        "a block string would hide member lines"
    );
    let described = Regex::new(r#"^  ("(?:[^"\\]|\\.)*") (\S.*)$"#).expect("the member pattern");
    block
        .lines()
        .filter(|line| line.starts_with("  "))
        .map(|line| match described.captures(line) {
            Some(parts) => (parts[1].to_string(), parts[2].to_string()),
            None => {
                assert!(
                    !line.trim_start().starts_with('"'),
                    "{declaration}: a description apart from its member: {line}"
                );
                (String::new(), line.trim().to_string())
            }
        })
        .collect()
}

/// The named type of a root field's declaration line, without list or
/// non-null marks.
fn return_type(line: &str) -> String {
    let stripped = without_strings(line);
    let written = match stripped.rfind("): ") {
        Some(at) => &stripped[at + 3..],
        None => stripped.split_once(": ").expect("a field type").1,
    };
    written
        .trim()
        .trim_matches(|mark| mark == '[' || mark == ']' || mark == '!')
        .to_string()
}

/// The output types reachable from `start` through the types their fields
/// name. Input, enum and scalar names are not followed.
fn reachable_output_types(sdl: &str, start: BTreeSet<String>) -> BTreeSet<String> {
    let declared: BTreeSet<&str> = declaration_lines(sdl)
        .into_iter()
        .filter_map(|line| line.strip_prefix("type ")?.strip_suffix(" {"))
        .collect();
    let type_name = Regex::new(r"\b[A-Z][A-Za-z0-9_]*\b").expect("the type name pattern");
    let mut reached = BTreeSet::new();
    let mut queue: Vec<String> = start.into_iter().collect();
    while let Some(name) = queue.pop() {
        if !declared.contains(name.as_str()) || !reached.insert(name.clone()) {
            continue;
        }
        let block = without_strings(sdl_block(sdl, &format!("type {name} {{")));
        queue.extend(
            type_name
                .find_iter(&block)
                .map(|found| found.as_str().to_string()),
        );
    }
    reached
}

/// The chunk of the SDL (blank-line separated) holding `declaration` and
/// its description.
fn sdl_chunk<'a>(sdl: &'a str, declaration: &str) -> &'a str {
    let found: Vec<&str> = sdl
        .split("\n\n")
        .filter(|chunk| chunk.lines().any(|line| line == declaration))
        .collect();
    assert_eq!(found.len(), 1, "{declaration}");
    found[0]
}

fn sha256(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

#[test]
fn the_generated_sdl_declares_exactly_the_forty_new_async_job_types() {
    let sdl = sdl();
    let declared: BTreeSet<&str> = declaration_lines(&sdl)
        .into_iter()
        .filter(|line| line.contains("Async"))
        .collect();
    let expected: BTreeSet<&str> = NEW_DECLARATIONS.into_iter().collect();
    assert_eq!(expected.len(), 40);
    assert_eq!(declared, expected);
}

#[test]
fn every_new_root_field_names_an_async_job_and_no_other_root_field_does() {
    let sdl = sdl();
    let mut marked = BTreeSet::new();
    for root in ROOTS {
        for (_, line) in root_fields(&sdl, root) {
            let name = field_name(&line).to_string();
            let stripped = without_strings(&line);
            let signature =
                &stripped[stripped.find(name.as_str()).expect("the name") + name.len()..];
            let touches_async_type = signature.contains("Async");
            let has_marker = name.contains(ASYNC_JOB_FIELD_MARKER);
            assert_eq!(
                has_marker, touches_async_type,
                "{root} `{name}`: an async job field must contain `{ASYNC_JOB_FIELD_MARKER}` \
                 and no other field may"
            );
            if has_marker {
                marked.insert(name);
            }
        }
    }
    let expected: BTreeSet<String> = NEW_ROOT_FIELDS
        .iter()
        .map(|field| field.to_string())
        .collect();
    assert_eq!(marked, expected);
}

#[test]
fn only_the_two_claim_fields_are_nullable() {
    let sdl = sdl();
    let mut seen = 0;
    for root in ROOTS {
        for (_, line) in root_fields(&sdl, root) {
            let name = field_name(&line);
            if !name.contains(ASYNC_JOB_FIELD_MARKER) {
                continue;
            }
            let stripped = without_strings(&line);
            let return_type = match stripped.rfind("): ") {
                Some(at) => &stripped[at + 3..],
                None => stripped.split_once(": ").expect("a field type").1,
            };
            let nullable = !return_type.trim_end().ends_with('!');
            let claim_field = name == "claimAsyncJobs" || name == "claimAsyncJobReconciliations";
            assert_eq!(nullable, claim_field, "{line}");
            seen += 1;
        }
    }
    assert_eq!(seen, NEW_ROOT_FIELDS.len());
}

#[test]
fn only_a_claim_result_carries_a_claim_token_and_only_the_claim_fields_return_one() {
    let sdl = sdl();
    let claim_token = word("claimToken");

    // The new root fields reach exactly the new output types, so the API
    // reaches no claim token of an older type (such as a BE-04 or metrics
    // claim), and among the new types only the claim result carries one.
    let mut returned = BTreeSet::new();
    let mut returning_claims = BTreeSet::new();
    for root in ROOTS {
        for (_, line) in root_fields(&sdl, root) {
            let name = field_name(&line).to_string();
            if word("AsyncJobClaimResult").is_match(&without_strings(&line)) {
                returning_claims.insert(name.clone());
            }
            if name.contains(ASYNC_JOB_FIELD_MARKER) {
                returned.insert(return_type(&line));
            }
        }
    }
    let new_output_types: BTreeSet<String> = NEW_DECLARATIONS
        .into_iter()
        .filter_map(|declaration| declaration.strip_prefix("type ")?.strip_suffix(" {"))
        .map(str::to_string)
        .collect();
    let reached = reachable_output_types(&sdl, returned);
    assert_eq!(reached, new_output_types);
    for name in &reached {
        let block = without_strings(sdl_block(&sdl, &format!("type {name} {{")));
        assert_eq!(
            claim_token.is_match(&block),
            name == "ClaimedAsyncJob",
            "`{name}` and claimToken"
        );
    }

    // Output type -> the only type allowed to refer to it, checked over every
    // output type of the schema.
    let references = [
        ("ClaimedAsyncJob", "AsyncJobClaimResult"),
        ("AsyncJobClaimResult", "MutationRoot"),
        ("AsyncJobAttempt", "AsyncJobDetail"),
        ("AsyncJobTransition", "AsyncJobDetail"),
    ];
    for declaration in declaration_lines(&sdl)
        .into_iter()
        .filter(|line| line.starts_with("type "))
    {
        let name = declaration.split_whitespace().nth(1).expect("a type name");
        let block = without_strings(sdl_block(&sdl, declaration));
        for (referenced, allowed) in references {
            if word(referenced).is_match(&block) {
                assert_eq!(name, allowed, "`{name}` refers to `{referenced}`");
            }
        }
    }
    assert_eq!(
        returning_claims,
        BTreeSet::from([
            "claimAsyncJobReconciliations".to_string(),
            "claimAsyncJobs".to_string()
        ])
    );
}

#[test]
fn the_be04_sdl_elements_are_byte_identical_to_the_base() {
    let sdl = sdl();
    for (declaration, expected) in BE04_DECLARATIONS {
        assert_eq!(
            sha256(sdl_chunk(&sdl, declaration)),
            expected,
            "{declaration}"
        );
    }
    for (owner, prefix, expected) in BE04_FIELDS {
        let lines: Vec<&str> = sdl_chunk(&sdl, owner).lines().collect();
        let at: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with(&format!("  {prefix}")))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(at.len(), 1, "{owner} {prefix}");
        let description = lines[at[0] - 1];
        assert!(description.starts_with("  \""), "{owner} {prefix}");
        assert_eq!(
            sha256(&format!("{description}\n{}", lines[at[0]])),
            expected,
            "{owner} {prefix}"
        );
    }
}

#[test]
fn the_new_sdl_names_no_generic_worker_or_service_role_or_field() {
    let pattern = Regex::new(r"\b(?i:worker|service)s?\b").expect("the pattern");
    // The machine role the descriptions name is not a generic word.
    assert!(!pattern.is_match("Requires the DISSEMINATION_WORKER role"));
    assert!(pattern.is_match("a WORKER role") && pattern.is_match("generic services"));

    let sdl = sdl();
    for declaration in NEW_DECLARATIONS {
        let chunk = sdl_chunk(&sdl, declaration);
        assert!(!pattern.is_match(chunk), "{chunk}");
    }
    for root in ROOTS {
        for (description, line) in root_fields(&sdl, root) {
            if field_name(&line).contains(ASYNC_JOB_FIELD_MARKER) {
                assert!(!pattern.is_match(&description), "{description}");
                assert!(!pattern.is_match(&line), "{line}");
            }
        }
    }
}

/// The labels of `$api` <-> `$engine`, checking both conversions and the
/// GraphQL rendering of every variant (and the engine's `as_str` where it
/// has one).
macro_rules! mirrored_labels {
    ($api:ty, $engine:ident, as_str, [$($variant:ident => $label:literal),+ $(,)?]) => {{
        $(assert_eq!($engine::$variant.as_str(), $label);)+
        mirrored_labels!($api, $engine, [$($variant => $label),+])
    }};
    ($api:ty, $engine:ident, [$($variant:ident => $label:literal),+ $(,)?]) => {{
        let mut labels: Vec<&'static str> = Vec::new();
        $(
            let engine_value = $engine::$variant;
            let api_value = <$api as From<$engine>>::from(engine_value);
            assert_eq!($engine::from(api_value), engine_value);
            // juniper renders an enum input value as its label and parses the
            // label written as a GraphQL enum value.
            let rendered: InputValue<DefaultScalarValue> = api_value.to_input_value();
            assert_eq!(rendered, InputValue::scalar($label.to_string()));
            let parsed = <$api as FromInputValue<DefaultScalarValue>>::from_input_value(
                &InputValue::enum_value($label),
            )
            .unwrap_or_else(|error| panic!("`{}` must parse: {error}", $label));
            assert_eq!($engine::from(parsed), engine_value);
            labels.push($label);
        )+
        labels
    }};
}

#[test]
fn every_async_enum_mirrors_its_engine_enum_label_for_label() {
    let enums = [
        (
            "enum AsyncJobStatus {",
            mirrored_labels!(api::AsyncJobStatus, AsyncJobStatus, as_str, [
                Pending => "PENDING",
                Running => "RUNNING",
                Waiting => "WAITING",
                RetryScheduled => "RETRY_SCHEDULED",
                Succeeded => "SUCCEEDED",
                Failed => "FAILED",
                Cancelled => "CANCELLED",
                ReconciliationRequired => "RECONCILIATION_REQUIRED",
            ]),
        ),
        (
            "enum AsyncEffectPhase {",
            mirrored_labels!(api::AsyncEffectPhase, AsyncEffectPhase, as_str, [
                PreWrite => "PRE_WRITE",
                EffectStarted => "EFFECT_STARTED",
                EffectConfirmed => "EFFECT_CONFIRMED",
            ]),
        ),
        (
            "enum AsyncJobEffectClass {",
            mirrored_labels!(api::AsyncJobEffectClass, EffectClass, as_str, [
                ReadOnly => "READ_ONLY",
                Effectful => "EFFECTFUL",
            ]),
        ),
        (
            "enum AsyncJobRevisionMode {",
            mirrored_labels!(api::AsyncJobRevisionMode, RevisionMode, as_str, [
                CurrentState => "CURRENT_STATE",
                RevisionBound => "REVISION_BOUND",
            ]),
        ),
        (
            "enum AsyncJobLineage {",
            mirrored_labels!(api::AsyncJobLineage, AsyncJobLineage, as_str, [
                Original => "ORIGINAL",
                ManualRetry => "MANUAL_RETRY",
            ]),
        ),
        (
            "enum AsyncAttemptPurpose {",
            mirrored_labels!(api::AsyncAttemptPurpose, AsyncAttemptPurpose, as_str, [
                Execution => "EXECUTION",
                Reconciliation => "RECONCILIATION",
                StaffReconciliation => "STAFF_RECONCILIATION",
            ]),
        ),
        (
            "enum AsyncAttemptResult {",
            mirrored_labels!(api::AsyncAttemptResult, AsyncAttemptResult, as_str, [
                Succeeded => "SUCCEEDED",
                RetryableFailure => "RETRYABLE_FAILURE",
                PermanentFailure => "PERMANENT_FAILURE",
                Indeterminate => "INDETERMINATE",
                Waiting => "WAITING",
                Released => "RELEASED",
                LeaseExpired => "LEASE_EXPIRED",
                Cancelled => "CANCELLED",
                StillRequired => "STILL_REQUIRED",
            ]),
        ),
        (
            "enum AsyncControlCommandKind {",
            mirrored_labels!(api::AsyncControlCommandKind, AsyncControlCommandKind, as_str, [
                CancelJob => "CANCEL_JOB",
                RetryJob => "RETRY_JOB",
                ResolveReconciliation => "RESOLVE_RECONCILIATION",
                RedactJobDiagnostics => "REDACT_JOB_DIAGNOSTICS",
            ]),
        ),
        (
            "enum AsyncControlCommandOutcome {",
            mirrored_labels!(api::AsyncControlCommandOutcome, AsyncControlCommandOutcome, as_str, [
                Applied => "APPLIED",
                Refused => "REFUSED",
            ]),
        ),
        (
            "enum AsyncJobCompletionOutcome {",
            mirrored_labels!(api::AsyncJobCompletionOutcome, CompletionOutcome, as_str, [
                EffectApplied => "EFFECT_APPLIED",
                NoOp => "NO_OP",
                Superseded => "SUPERSEDED",
                Completed => "COMPLETED",
            ]),
        ),
        (
            "enum AsyncJobFailureClass {",
            mirrored_labels!(api::AsyncJobFailureClass, FailureClass, [
                RetryablePreWrite => "RETRYABLE_PRE_WRITE",
                Permanent => "PERMANENT",
                Indeterminate => "INDETERMINATE",
            ]),
        ),
        (
            "enum AsyncJobReconcilerOutcome {",
            mirrored_labels!(api::AsyncJobReconcilerOutcome, ReconcilerOutcome, [
                Succeeded => "SUCCEEDED",
                Failed => "FAILED",
                Cancelled => "CANCELLED",
                Waiting => "WAITING",
                StillRequired => "STILL_REQUIRED",
            ]),
        ),
        (
            "enum AsyncJobStaffOutcome {",
            mirrored_labels!(api::AsyncJobStaffOutcome, StaffOutcome, as_str, [
                Succeeded => "SUCCEEDED",
                Failed => "FAILED",
                Cancelled => "CANCELLED",
                Waiting => "WAITING",
                AcceptRegression => "ACCEPT_REGRESSION",
            ]),
        ),
    ];

    let sdl = sdl();
    let declared: BTreeSet<&str> = NEW_DECLARATIONS
        .into_iter()
        .filter(|declaration| declaration.starts_with("enum "))
        .collect();
    let checked: BTreeSet<&str> = enums.iter().map(|(declaration, _)| *declaration).collect();
    assert_eq!(checked, declared, "every new enum is checked");
    for (declaration, labels) in enums {
        let listed: BTreeSet<String> = inline_members(&sdl, declaration)
            .into_iter()
            .map(|(_, value)| value)
            .collect();
        let expected: BTreeSet<String> = labels.into_iter().map(str::to_string).collect();
        assert_eq!(listed, expected, "{declaration}");
    }
}

#[test]
fn the_reason_fields_warn_staff_and_the_claim_input_names_no_executor_identity() {
    let sdl = sdl();
    for declaration in [
        "input AsyncJobCommandInput {",
        "input ResolveAsyncJobReconciliationAsStaffInput {",
    ] {
        let reasons: Vec<String> = inline_members(&sdl, declaration)
            .into_iter()
            .filter(|(_, member)| member == "reason: String!")
            .map(|(description, _)| description)
            .collect();
        assert_eq!(reasons.len(), 1, "{declaration}: one reason field");
        assert!(
            reasons[0].contains("Do not paste provider diagnostics or personal data into it"),
            "{declaration}: {}",
            reasons[0]
        );
    }

    // The request narrows kinds and versions; the executor's domain, risk
    // class, capabilities and revision modes are never input.
    for (declaration, expected) in [
        (
            "input ClaimAsyncJobsInput {",
            BTreeSet::from(["claimedBy", "kinds", "maxJobs", "leaseSeconds"]),
        ),
        (
            "input AsyncJobKindVersionsInput {",
            BTreeSet::from(["kind", "payloadVersions"]),
        ),
    ] {
        let members = inline_members(&sdl, declaration);
        let fields: BTreeSet<&str> = members
            .iter()
            .map(|(_, member)| field_name(member))
            .collect();
        assert_eq!(fields, expected, "{declaration}");
    }
}
