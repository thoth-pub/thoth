//! Test support for the async job engine: test kinds and registries,
//! executors, engine shortcuts, direct-SQL fixtures and concurrency helpers.
//!
//! Concurrency helpers never sleep: they synchronize with barriers, advisory
//! locks held by a controlling connection, and bounded `pg_locks` polling.

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration as StdDuration, Instant};

use chrono::{DateTime, Duration, Utc};
use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use diesel::QueryableByName;
use serde_json::{json, Value};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::registry::{
    AsyncJobKindPolicy, AsyncJobRegistry, CancelPolicy, EvidenceContext, EvidenceUse,
    ExecutorDomain, PayloadError, RiskClass, StaffOutcome,
};
use super::{
    claim_on, claim_reconciliation_on, create_job_on, get_job, job_attempts, job_transitions,
    mark_effect_confirmed_on, mark_effect_started_on, with_engine_transaction_on, AsyncJob,
    AsyncJobAttempt, AsyncJobTransition, ClaimRequest, ClaimedJob, CreateOutcome, EffectClass,
    Executor, NewAsyncJob, RevisionMode, StaffActor, WorkerClaim,
};
use crate::model::tests::db::test_db_url;

// ---------------------------------------------------------------------------
// Test kinds
// ---------------------------------------------------------------------------

pub(crate) const TEST_READ: &str = "TEST_READ";
pub(crate) const TEST_EFFECT: &str = "TEST_EFFECT";
pub(crate) const TEST_PEER: &str = "TEST_PEER";
pub(crate) const TEST_REVISION: &str = "TEST_REVISION";
pub(crate) const TEST_POLL: &str = "TEST_POLL";
pub(crate) const TEST_UNTRUSTED: &str = "TEST_UNTRUSTED";
pub(crate) const TEST_CAPABILITY: &str = "TEST_CAPABILITY";
pub(crate) const TEST_LIMITED: &str = "TEST_LIMITED";
pub(crate) const TEST_STRICT: &str = "TEST_STRICT";
pub(crate) const TEST_RETIRED: &str = "TEST_RETIRED";
pub(crate) const TEST_VERSIONED: &str = "TEST_VERSIONED";

pub(crate) const NS_CS: &str = "test_cs";
pub(crate) const NS_RB: &str = "test_rb";
pub(crate) const NS_READ: &str = "test_read";
pub(crate) const NS_POLL: &str = "test_poll";

pub(crate) const CAPABILITY: &str = "test.capability";

/// The payload must carry a non-empty string `id` and no `invalid` key.
fn validate_payload(_version: i16, payload: &Value) -> Result<(), PayloadError> {
    match payload.get("id").and_then(Value::as_str) {
        Some(id) if !id.is_empty() => {}
        _ => return Err(PayloadError::new("the payload needs an id")),
    }
    if payload.get("invalid").is_some() {
        return Err(PayloadError::new("the payload is marked invalid"));
    }
    Ok(())
}

/// `id`, or `id@rev` when the payload names a revision; `identity` overrides
/// both (to test the 400-character bound).
fn effect_identity(_version: i16, payload: &Value) -> Result<String, PayloadError> {
    if let Some(identity) = payload.get("identity").and_then(Value::as_str) {
        return Ok(identity.to_string());
    }
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Ok(match payload.get("rev") {
        Some(rev) => format!("{id}@{rev}"),
        None => id.to_string(),
    })
}

/// The payload's `targets` array of strings.
fn targets(_version: i16, payload: &Value) -> Result<Vec<String>, PayloadError> {
    Ok(payload
        .get("targets")
        .and_then(Value::as_array)
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
    evidence: &Value,
) -> Result<(), PayloadError> {
    let valid = evidence.get("valid") == Some(&Value::Bool(true));
    match use_ {
        EvidenceUse::Applied => {
            if evidence.is_null() {
                return Ok(());
            }
            if !valid {
                return Err(PayloadError::new("applied evidence is not valid"));
            }
            if let Some(order) = context.source_order {
                if evidence.get("order").and_then(Value::as_i64) != Some(order) {
                    return Err(PayloadError::new("applied evidence names another revision"));
                }
            }
            Ok(())
        }
        EvidenceUse::NoEffect => {
            if valid
                && evidence
                    .get("providerRef")
                    .and_then(Value::as_str)
                    .is_some()
            {
                Ok(())
            } else {
                Err(PayloadError::new(
                    "no-effect evidence needs a provider reference",
                ))
            }
        }
        EvidenceUse::NotApplied => {
            if evidence.get("notApplied") == Some(&Value::Bool(true)) {
                Ok(())
            } else {
                Err(PayloadError::new("not-applied evidence is not valid"))
            }
        }
    }
}

/// Like [`validate_evidence`], but applied evidence is required.
fn validate_evidence_strict(
    use_: EvidenceUse,
    context: &EvidenceContext<'_>,
    evidence: &Value,
) -> Result<(), PayloadError> {
    if use_ == EvidenceUse::Applied && evidence.is_null() {
        return Err(PayloadError::new("applied evidence is required"));
    }
    validate_evidence(use_, context, evidence)
}

const ALL_STAFF: &[StaffOutcome] = &[
    StaffOutcome::Succeeded,
    StaffOutcome::Failed,
    StaffOutcome::Cancelled,
    StaffOutcome::Waiting,
];

/// The base effectful CURRENT_STATE kind every other test kind derives from.
pub(crate) const EFFECT_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_EFFECT,
    owning_domain: "test_domain",
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
    staff_reconciliation_outcomes: ALL_STAFF,
    validate_payload,
    effect_identity,
    targets,
    validate_evidence,
};

pub(crate) const READ_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_READ,
    executor_domain: ExecutorDomain::ThothWorker,
    reconciler_domain: ExecutorDomain::ThothWorker,
    effect_class: EffectClass::ReadOnly,
    effect_window_seconds: None,
    target_namespaces: &[NS_READ],
    staff_reconciliation_outcomes: &[],
    ..EFFECT_POLICY
};

pub(crate) const PEER_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_PEER,
    ..EFFECT_POLICY
};

pub(crate) const REVISION_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_REVISION,
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
    ..EFFECT_POLICY
};

pub(crate) const POLL_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_POLL,
    target_namespaces: &[NS_POLL],
    confirmed_effect_resumable: true,
    staff_reconciliation_outcomes: &[
        StaffOutcome::Succeeded,
        StaffOutcome::Failed,
        StaffOutcome::Waiting,
    ],
    ..EFFECT_POLICY
};

pub(crate) const UNTRUSTED_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_UNTRUSTED,
    risk_class: RiskClass::UntrustedContent,
    ..EFFECT_POLICY
};

pub(crate) const CAPABILITY_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_CAPABILITY,
    required_capabilities: &[CAPABILITY],
    ..EFFECT_POLICY
};

pub(crate) const LIMITED_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_LIMITED,
    concurrency_limit: 3,
    reconciliation_concurrency_limit: 1,
    ..EFFECT_POLICY
};

pub(crate) const STRICT_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_STRICT,
    cancellable: CancelPolicy::NONE,
    manual_retry_allowed: false,
    no_effect_after_start_allowed: false,
    staff_reconciliation_outcomes: &[StaffOutcome::Succeeded, StaffOutcome::Failed],
    validate_evidence: validate_evidence_strict,
    ..EFFECT_POLICY
};

pub(crate) const RETIRED_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_RETIRED,
    admission_open: false,
    ..EFFECT_POLICY
};

pub(crate) const VERSIONED_POLICY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: TEST_VERSIONED,
    creatable_versions: &[1],
    executable_versions: &[1, 2],
    ..EFFECT_POLICY
};

/// The registry every test serves with.
pub(crate) static REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[
    READ_POLICY,
    EFFECT_POLICY,
    PEER_POLICY,
    REVISION_POLICY,
    POLL_POLICY,
    UNTRUSTED_POLICY,
    CAPABILITY_POLICY,
    LIMITED_POLICY,
    STRICT_POLICY,
    RETIRED_POLICY,
    VERSIONED_POLICY,
]);

/// An earlier binary's registry: `TEST_RETIRED` still admitted and
/// `TEST_VERSIONED` version 2 (and 3) still creatable, for retirement tests.
pub(crate) static EARLIER_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[
    EFFECT_POLICY,
    AsyncJobKindPolicy {
        admission_open: true,
        ..RETIRED_POLICY
    },
    AsyncJobKindPolicy {
        creatable_versions: &[1, 2, 3],
        executable_versions: &[1, 2, 3],
        ..VERSIONED_POLICY
    },
]);

/// A binary that knows only `TEST_READ`: every other test kind is unknown to
/// it.
pub(crate) static NARROW_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[READ_POLICY]);

/// A later binary's registry whose `TEST_EFFECT` policy has another budget
/// and backoff: a manual retry takes its snapshot from the current registry.
pub(crate) static RETUNED_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[
    AsyncJobKindPolicy {
        retry_budget: 5,
        max_execution_claims: 6,
        backoff_base_seconds: 20,
        backoff_max_seconds: 600,
        ..EFFECT_POLICY
    },
    PEER_POLICY,
]);

/// Every registry the tests serve with (each must validate).
pub(crate) fn test_registries() -> [&'static AsyncJobRegistry; 4] {
    [
        &REGISTRY,
        &EARLIER_REGISTRY,
        &NARROW_REGISTRY,
        &RETUNED_REGISTRY,
    ]
}

// ---------------------------------------------------------------------------
// Executors and actors
// ---------------------------------------------------------------------------

/// The dissemination executor that serves every effectful test kind.
pub(crate) fn dissemination() -> Executor {
    executor_named("dissemination-1")
}

pub(crate) fn executor_named(name: &str) -> Executor {
    Executor {
        name: name.to_string(),
        principal: "machine:dissemination".to_string(),
        domain: ExecutorDomain::Dissemination,
        risk_class: RiskClass::Trusted,
        capabilities: vec![CAPABILITY.to_string()],
        revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
    }
}

/// The in-process worker that serves `TEST_READ`.
pub(crate) fn thoth_worker() -> Executor {
    Executor {
        name: "thoth-worker".to_string(),
        principal: "process:thoth-worker".to_string(),
        domain: ExecutorDomain::ThothWorker,
        risk_class: RiskClass::Trusted,
        capabilities: Vec::new(),
        revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
    }
}

pub(crate) fn staff(reason: &str) -> StaffActor {
    StaffActor {
        principal: "staff:operator@example.org".to_string(),
        reason: reason.to_string(),
        request_id: Some(Uuid::new_v4()),
    }
}

/// A claim request for `kinds`, each with versions 1 and 2.
pub(crate) fn request(kinds: &[&str], max_jobs: i32) -> ClaimRequest {
    ClaimRequest {
        kinds: kinds
            .iter()
            .map(|kind| (kind.to_string(), vec![1, 2]))
            .collect(),
        max_jobs,
        lease_seconds: None,
    }
}

// ---------------------------------------------------------------------------
// Job inputs
// ---------------------------------------------------------------------------

/// A version-1 job of `kind` with payload `{"id": id, "targets": targets}`.
pub(crate) fn new_job(kind: &str, id: &str, targets: &[&str]) -> NewAsyncJob {
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

/// A `TEST_REVISION` job for revision `order` of `id`, on one target.
pub(crate) fn revision_job(id: &str, target: &str, order: i64, fingerprint: &str) -> NewAsyncJob {
    NewAsyncJob {
        payload: json!({ "id": id, "rev": order, "targets": [target] }).to_string(),
        source_order: Some(order),
        source_fingerprint: Some(fingerprint.to_string()),
        source_ref: Some(format!("ref-{order}")),
        ..new_job(TEST_REVISION, id, &[target])
    }
}

/// A fresh target key of `namespace`.
pub(crate) fn key(namespace: &str) -> String {
    format!("{namespace}:{}", Uuid::new_v4())
}

// ---------------------------------------------------------------------------
// Connections
// ---------------------------------------------------------------------------

/// A dedicated connection, outside the shared pool.
pub(crate) fn connect() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database")
}

/// Run an engine call in its own engine-owned transaction.
pub(crate) fn tx<T>(
    conn: &mut PgConnection,
    operation: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    with_engine_transaction_on(conn, operation)
}

#[derive(QueryableByName)]
struct I64Row {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

#[derive(QueryableByName)]
struct I32Row {
    #[diesel(sql_type = Integer)]
    value: i32,
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}

/// One `bigint` value, selected `AS value`.
pub(crate) fn scalar(conn: &mut PgConnection, sql: &str) -> i64 {
    sql_query(sql)
        .get_result::<I64Row>(conn)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

/// One nullable text value, selected `AS value`.
pub(crate) fn text(conn: &mut PgConnection, sql: &str) -> Option<String> {
    sql_query(sql)
        .get_result::<TextRow>(conn)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

/// Execute SQL that must succeed.
pub(crate) fn exec(conn: &mut PgConnection, sql: &str) {
    conn.batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

/// Execute SQL that must fail, in its own transaction (a savepoint when
/// `conn` is already in one, so the outer transaction stays usable); the
/// deferred checks fire at its commit. Returns the error's constraint name.
pub(crate) fn refused_by(conn: &mut PgConnection, sql: &str) -> String {
    match conn.transaction(|c| c.batch_execute(sql)) {
        Ok(()) => panic!("expected a refusal: {sql}"),
        Err(DieselError::DatabaseError(_, info)) => info
            .constraint_name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("<no constraint: {}>", info.message())),
        Err(other) => panic!("unexpected error for {sql}: {other}"),
    }
}

/// Run `sql` in its own transaction with every ordinary trigger disabled
/// (`session_replication_role = replica`): the direct-SQL fixture that
/// bypasses the engine's triggers. Check constraints still apply.
pub(crate) fn bypass(conn: &mut PgConnection, sql: &str) {
    conn.transaction(|conn| {
        conn.batch_execute("SET LOCAL session_replication_role = replica")?;
        conn.batch_execute(sql)
    })
    .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

/// The backend process id of a connection.
pub(crate) fn backend_pid(conn: &mut PgConnection) -> i32 {
    sql_query("SELECT pg_backend_pid() AS value")
        .get_result::<I32Row>(conn)
        .expect("Failed to read the backend pid")
        .value
}

/// `pg_stat_database.deadlocks` of the test database.
pub(crate) fn deadlocks(conn: &mut PgConnection) -> i64 {
    conn.batch_execute("SELECT pg_stat_clear_snapshot()")
        .expect("Failed to clear the statistics snapshot");
    scalar(
        conn,
        "SELECT deadlocks AS value FROM pg_stat_database WHERE datname = current_database()",
    )
}

/// Wait, by bounded polling of `pg_locks` (never sleeping), until backend
/// `pid` waits for a lock.
pub(crate) fn wait_until_blocked(observer: &mut PgConnection, pid: i32) {
    let deadline = Instant::now() + StdDuration::from_secs(60);
    loop {
        let waiting = scalar(
            observer,
            &format!("SELECT count(*) AS value FROM pg_locks WHERE pid = {pid} AND NOT granted"),
        );
        if waiting > 0 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "backend {pid} never entered a lock wait"
        );
        thread::yield_now();
    }
}

/// True when backend `pid` waits for any lock now.
pub(crate) fn is_blocked(observer: &mut PgConnection, pid: i32) -> bool {
    scalar(
        observer,
        &format!("SELECT count(*) AS value FROM pg_locks WHERE pid = {pid} AND NOT granted"),
    ) > 0
}

/// Spawn `f` on its own thread with a fresh connection, after announcing the
/// connection's backend pid.
pub(crate) fn spawn_with_connection<T: Send + 'static>(
    f: impl FnOnce(&mut PgConnection) -> T + Send + 'static,
) -> (i32, thread::JoinHandle<T>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let mut conn = connect();
        sender
            .send(backend_pid(&mut conn))
            .expect("Failed to announce the backend pid");
        f(&mut conn)
    });
    let pid = receiver.recv().expect("Failed to receive the backend pid");
    (pid, handle)
}

/// One task of [`run_together`]: a closure over its own connection.
pub(crate) type Task<T> = Box<dyn FnOnce(&mut PgConnection) -> T + Send>;

/// Run `n` closures concurrently, each on its own connection, released
/// together by a barrier.
pub(crate) fn run_together<T: Send + 'static>(tasks: Vec<Task<T>>) -> Vec<T> {
    let barrier = Arc::new(Barrier::new(tasks.len()));
    let handles: Vec<_> = tasks
        .into_iter()
        .map(|task| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut conn = connect();
                barrier.wait();
                task(&mut conn)
            })
        })
        .collect();
    handles
        .into_iter()
        .map(|handle| handle.join().expect("a concurrent task panicked"))
        .collect()
}

// ---------------------------------------------------------------------------
// Engine shortcuts
// ---------------------------------------------------------------------------

pub(crate) fn create(conn: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    match tx(conn, |c| create_job_on(c, &REGISTRY, input)).expect("create") {
        CreateOutcome::Created(job) => job,
        CreateOutcome::Existing(_) => panic!("expected a new job"),
    }
}

/// Create one job with an earlier binary's registry.
pub(crate) fn create_earlier(conn: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    tx(conn, |c| create_job_on(c, &EARLIER_REGISTRY, input))
        .expect("create")
        .into_job()
}

/// Claim every ready job of `kinds` (at most 50) as `executor`.
pub(crate) fn claim(
    conn: &mut PgConnection,
    executor: &Executor,
    kinds: &[&str],
) -> Vec<ClaimedJob> {
    tx(conn, |c| {
        claim_on(c, &REGISTRY, executor, request(kinds, 50))
    })
    .expect("claim")
    .jobs
}

/// Claim exactly the one job `job_id` (the only ready job of its kind).
pub(crate) fn claim_job(conn: &mut PgConnection, job: &AsyncJob) -> WorkerClaim {
    let executor = if job.kind == TEST_READ {
        thoth_worker()
    } else {
        dissemination()
    };
    let claimed = claim(conn, &executor, &[&job.kind]);
    let claimed = claimed
        .into_iter()
        .find(|c| c.async_job_id == job.async_job_id)
        .unwrap_or_else(|| panic!("job {} was not claimed", job.async_job_id));
    claimed.worker_claim(executor.domain)
}

/// Claim the RR job `job` for reconciliation.
pub(crate) fn claim_recon(conn: &mut PgConnection, job: &AsyncJob) -> WorkerClaim {
    let executor = dissemination();
    let claimed = tx(conn, |c| {
        claim_reconciliation_on(c, &REGISTRY, &executor, request(&[&job.kind], 50))
    })
    .expect("claim reconciliation")
    .jobs
    .into_iter()
    .find(|c| c.async_job_id == job.async_job_id)
    .unwrap_or_else(|| {
        panic!(
            "job {} was not claimed for reconciliation",
            job.async_job_id
        )
    });
    claimed.worker_claim(executor.domain)
}

/// Create and claim a job.
pub(crate) fn running(conn: &mut PgConnection, input: NewAsyncJob) -> (AsyncJob, WorkerClaim) {
    let job = create(conn, input);
    let claim = claim_job(conn, &job);
    (job, claim)
}

/// Create, claim and mark the effect started.
pub(crate) fn started(conn: &mut PgConnection, input: NewAsyncJob) -> (AsyncJob, WorkerClaim) {
    let (job, claim) = running(conn, input);
    tx(conn, |c| mark_effect_started_on(c, &REGISTRY, &claim)).expect("mark effect started");
    (job, claim)
}

/// Create, claim, start and confirm the effect.
pub(crate) fn confirmed(conn: &mut PgConnection, input: NewAsyncJob) -> (AsyncJob, WorkerClaim) {
    let (job, claim) = started(conn, input);
    tx(conn, |c| {
        mark_effect_confirmed_on(c, &REGISTRY, &claim, Some("provider-1"))
    })
    .expect("mark effect confirmed");
    (job, claim)
}

pub(crate) fn job(conn: &mut PgConnection, id: Uuid) -> AsyncJob {
    get_job(conn, id).expect("job")
}

pub(crate) fn attempts(conn: &mut PgConnection, id: Uuid) -> Vec<AsyncJobAttempt> {
    job_attempts(conn, id).expect("attempts")
}

pub(crate) fn transitions(conn: &mut PgConnection, id: Uuid) -> Vec<AsyncJobTransition> {
    job_transitions(conn, id).expect("transitions")
}

/// The causes of a job's transitions, in order.
pub(crate) fn causes(conn: &mut PgConnection, id: Uuid) -> Vec<String> {
    transitions(conn, id).into_iter().map(|t| t.cause).collect()
}

/// The keys a job holds, ascending.
pub(crate) fn holds(conn: &mut PgConnection, id: Uuid) -> Vec<String> {
    #[derive(QueryableByName)]
    struct Key {
        #[diesel(sql_type = Text)]
        target_key: String,
    }
    sql_query(format!(
        "SELECT target_key FROM public.async_target_hold \
          WHERE async_job_id = '{id}' ORDER BY target_key"
    ))
    .load::<Key>(conn)
    .expect("holds")
    .into_iter()
    .map(|k| k.target_key)
    .collect()
}

/// The applied state row of a key: `(order, fingerprint, source_ref,
/// regressed_from_order, regressed_from_fingerprint, regressed)`.
#[derive(Debug, Clone, PartialEq, Eq, QueryableByName)]
pub(crate) struct StateView {
    #[diesel(sql_type = Nullable<BigInt>)]
    pub applied_order: Option<i64>,
    #[diesel(sql_type = Text)]
    pub applied_fingerprint: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub applied_source_ref: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub regressed_from_order: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    pub regressed_from_fingerprint: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub regressed: bool,
}

pub(crate) fn state(conn: &mut PgConnection, key: &str) -> Option<StateView> {
    sql_query(format!(
        "SELECT applied_order, applied_fingerprint, applied_source_ref, regressed_from_order, \
                regressed_from_fingerprint, regressed_at IS NOT NULL AS regressed \
           FROM public.async_effect_target_state WHERE target_key = '{key}'"
    ))
    .get_result::<StateView>(conn)
    .optional()
    .expect("state")
}

/// The number of command rows.
pub(crate) fn command_rows(conn: &mut PgConnection) -> i64 {
    scalar(
        conn,
        "SELECT count(*) AS value FROM public.async_control_command",
    )
}

/// No engine path wrote an `UNSPECIFIED` transition cause.
pub(crate) fn assert_no_unspecified(conn: &mut PgConnection) {
    assert_eq!(
        scalar(
            conn,
            "SELECT count(*) AS value FROM public.async_job_transition WHERE cause = 'UNSPECIFIED'"
        ),
        0,
        "an engine path wrote an UNSPECIFIED transition cause"
    );
}

// ---------------------------------------------------------------------------
// Time fixtures: move a job's clock instead of sleeping
// ---------------------------------------------------------------------------

/// Expire the job's lease (allowed by the triggers: same status).
pub(crate) fn expire_lease(conn: &mut PgConnection, id: Uuid) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job SET lease_expires_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Expire the job's lease more than `minutes` ago.
pub(crate) fn expire_lease_ago(conn: &mut PgConnection, id: Uuid, minutes: i64) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job \
                SET lease_expires_at = clock_timestamp() - interval '{minutes} minutes' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Make the job due now (`available_at` in the past).
pub(crate) fn make_available(conn: &mut PgConnection, id: Uuid) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job SET available_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Move the job's write-once waiting deadline `seconds` into the past (a
/// trigger-bypassing fixture).
pub(crate) fn expire_waiting_deadline(conn: &mut PgConnection, id: Uuid, seconds: i64) {
    bypass(
        conn,
        &format!(
            "UPDATE public.async_job \
                SET waiting_deadline = clock_timestamp() - interval '{seconds} seconds' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// `now` as the database sees it.
pub(crate) fn db_now(conn: &mut PgConnection) -> DateTime<Utc> {
    super::clock_now(conn).expect("clock")
}

/// A resume time inside the WAITING bounds of the test kinds.
pub(crate) fn resume_in(conn: &mut PgConnection, seconds: i64) -> DateTime<Utc> {
    db_now(conn) + Duration::seconds(seconds)
}

// ---------------------------------------------------------------------------
// Error helpers
// ---------------------------------------------------------------------------

/// The GraphQL code of an engine error.
pub(crate) fn code(error: &ThothError) -> &'static str {
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
        _ => "OTHER",
    }
}

/// Assert an engine result is the error `expected` (by code).
#[track_caller]
pub(crate) fn assert_code<T: std::fmt::Debug>(result: ThothResult<T>, expected: &str) {
    match result {
        Ok(value) => panic!("expected {expected}, got Ok({value:?})"),
        Err(error) => assert_eq!(code(&error), expected, "unexpected error: {error:?}"),
    }
}

/// Assert a transition refusal with its reason.
#[track_caller]
pub(crate) fn assert_refused<T: std::fmt::Debug>(result: ThothResult<T>, reason: &str) {
    match result {
        Err(ThothError::AsyncJobTransitionRefused(actual)) => assert_eq!(actual, reason),
        other => panic!("expected ASYNC_JOB_TRANSITION_REFUSED({reason}), got {other:?}"),
    }
}
