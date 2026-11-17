//! The section 8 tests of THOTH-ASYNC-01-WORKER-01 (W1 to W25 and W16b; W26
//! lives in the binary crate). Each test function is named after its
//! section 8 number.
//!
//! Database tests take the shared test database lock (`setup_test_db`),
//! drive the engine directly to set up job states before a runtime starts,
//! and stop and join every runtime (and drop its pools) before the guard
//! drops. Time passes by moving `available_at`, `lease_expires_at`,
//! `waiting_deadline` or the attempt runtime deadline, or by waiting for a
//! condition with a bound; only W9 waits for real renewals.

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use serde_json::json;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::config::{
    check_aws_sources, WorkerArgs, WorkerConfig, AWS_CREDENTIAL_VARIABLES, WORKER_PRINCIPAL,
};
use super::handler::{
    ExecutionAction, ExecutionContext, HandlerError, HandlerRegistry, JobView,
    ReconciliationAction, WaitAction, PRODUCTION_HANDLERS,
};
use super::health::{
    is_healthy, liveness_text, parse_health_args, withhold_reason, write_liveness, LivenessInputs,
    LivenessLimits,
};
use super::runtime::{
    applied_migrations, compute_claim_sets, deadline_guard, due_claims, effect_lease_request,
    embedded_migrations, in_read_only, migrations_pending, panic_line, plan_resume, settled_at,
    worker_executor, ClaimSets, ClaimState, ClaimStatus, Op, Purpose, ResumePlan, TrackedPhase,
};
use super::test_support::*;
use crate::model::async_job::{
    cancel_job_command_on, claim_on, fail_on, job_transitions, mark_effect_confirmed_on,
    mark_effect_started_on, renew_on, wait_on, with_engine_transaction_on, AsyncAttemptPurpose,
    AsyncEffectPhase, AsyncJobAttempt, AsyncJobKindPolicy, AsyncJobRegistry, ClaimRequest,
    ClaimedJob, ExecutorDomain, FailRequest, FailureClass, NewAsyncJob, ReconcilerOutcome,
    RevisionMode, RiskClass, StaffActor, WaitRequest, WorkerClaim, PRODUCTION_REGISTRY,
};
use crate::model::async_job::{AsyncJobStatus, Executor};
use crate::model::tests::db::{setup_test_db, test_db_url, TestDbGuard};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type Notes = Arc<Mutex<Vec<String>>>;

fn notes() -> Notes {
    Arc::new(Mutex::new(Vec::new()))
}

fn note(notes: &Notes, text: impl Into<String>) {
    notes.lock().unwrap().push(text.into());
}

fn noted(notes: &Notes) -> Vec<String> {
    notes.lock().unwrap().clone()
}

fn setup() -> (TestDbGuard, PgConnection) {
    logger();
    let (guard, _pool) = setup_test_db();
    (guard, connect())
}

fn tx<T>(
    conn: &mut PgConnection,
    f: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    with_engine_transaction_on(conn, f)
}

/// The executor the tests use to drive the engine directly.
fn setup_executor() -> Executor {
    Executor {
        name: "wtest-setup".to_string(),
        principal: WORKER_PRINCIPAL.to_string(),
        domain: ExecutorDomain::ThothWorker,
        risk_class: RiskClass::Trusted,
        capabilities: vec![W_CAPABILITY_NAME.to_string()],
        revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
    }
}

/// A job of `kind` whose payload carries `mode` (and one target for a kind
/// with targets).
fn mode_job(conn: &mut PgConnection, kind: &str, mode: &str) -> AsyncJob {
    let policy = REGISTRY.get(kind).expect("a worker test kind");
    let targets: Vec<String> = policy
        .target_namespaces
        .first()
        .map(|namespace| vec![key(namespace)])
        .unwrap_or_default();
    create(
        conn,
        NewAsyncJob {
            payload: json!({ "id": Uuid::new_v4().to_string(), "targets": targets, "mode": mode })
                .to_string(),
            ..new_job(kind, "unused", &targets)
        },
    )
}

use crate::model::async_job::AsyncJob;

fn mode_of(job: &JobView) -> String {
    job.payload_value
        .get("mode")
        .and_then(|mode| mode.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Claim the one ready job of `kind` directly through the engine.
fn engine_claim(conn: &mut PgConnection, kind: &str) -> ClaimedJob {
    let jobs = tx(conn, |c| {
        claim_on(
            c,
            &REGISTRY,
            &setup_executor(),
            ClaimRequest {
                kinds: vec![(kind.to_string(), vec![1, 2])],
                max_jobs: 50,
                lease_seconds: None,
            },
        )
    })
    .expect("claim")
    .jobs;
    assert_eq!(jobs.len(), 1, "exactly one ready job of {kind}");
    jobs.into_iter().next().unwrap()
}

fn worker_claim(job: &ClaimedJob) -> WorkerClaim {
    job.worker_claim(ExecutorDomain::ThothWorker)
}

/// T3 with the effect lease and T4 in one transaction, as the runtime does.
fn engine_start(conn: &mut PgConnection, kind: &str, claim: &WorkerClaim) {
    let policy = REGISTRY.get(kind).unwrap();
    let request = effect_lease_request(policy);
    tx(conn, |c| {
        renew_on(c, &REGISTRY, claim, Some(request))?;
        mark_effect_started_on(c, &REGISTRY, claim)
    })
    .expect("mark effect started");
}

fn engine_fail(conn: &mut PgConnection, claim: &WorkerClaim, class: FailureClass) {
    tx(conn, |c| {
        fail_on(
            c,
            &REGISTRY,
            claim,
            FailRequest {
                class,
                error_code: "SETUP_FAILURE".to_string(),
                error_detail: None,
                no_effect_proven: false,
                evidence: None,
            },
        )
    })
    .expect("fail");
}

fn engine_wait(conn: &mut PgConnection, claim: &WorkerClaim) {
    let resume_at = db_now(conn) + chrono::Duration::seconds(30);
    tx(conn, |c| {
        wait_on(
            c,
            &REGISTRY,
            claim,
            WaitRequest {
                checkpoint: json!({ "step": 1 }).to_string(),
                resume_at,
                provider_ref: None,
            },
        )
    })
    .expect("wait");
}

/// A `RECONCILIATION_REQUIRED` job: claimed, T4, `fail` `Indeterminate`.
fn to_rr(conn: &mut PgConnection, kind: &str, mode: &str) -> Uuid {
    let job = mode_job(conn, kind, mode);
    let claimed = engine_claim(conn, kind);
    let claim = worker_claim(&claimed);
    engine_start(conn, kind, &claim);
    engine_fail(conn, &claim, FailureClass::Indeterminate);
    assert_eq!(status(conn, job.async_job_id), "RECONCILIATION_REQUIRED");
    job.async_job_id
}

/// A `WAITING` job with a checkpoint, at `phase`, resumable in 30 s.
fn to_waiting(conn: &mut PgConnection, kind: &str, mode: &str, phase: AsyncEffectPhase) -> Uuid {
    let job = mode_job(conn, kind, mode);
    let claimed = engine_claim(conn, kind);
    let claim = worker_claim(&claimed);
    if phase != AsyncEffectPhase::PreWrite {
        engine_start(conn, kind, &claim);
    }
    if phase == AsyncEffectPhase::EffectConfirmed {
        tx(conn, |c| {
            mark_effect_confirmed_on(c, &REGISTRY, &claim, Some("provider-setup"))
        })
        .expect("confirm");
    }
    engine_wait(conn, &claim);
    assert_eq!(status(conn, job.async_job_id), "WAITING");
    job.async_job_id
}

fn execution_attempts(conn: &mut PgConnection, id: Uuid) -> Vec<AsyncJobAttempt> {
    attempts(conn, id)
        .into_iter()
        .filter(|attempt| attempt.purpose == AsyncAttemptPurpose::Execution)
        .collect()
}

fn last_execution(conn: &mut PgConnection, id: Uuid) -> AsyncJobAttempt {
    execution_attempts(conn, id)
        .pop()
        .expect("an execution attempt")
}

/// Start a runtime on the test database for `kinds`.
fn start(
    kinds: &[&str],
    slots: usize,
    execute: ExecuteFn,
    reconcile: ReconcileFn,
    tweak: impl FnOnce(&mut Timings),
) -> TestWorker {
    start_hooked(kinds, slots, execute, reconcile, tweak, Arc::default())
}

fn start_hooked(
    kinds: &[&str],
    slots: usize,
    execute: ExecuteFn,
    reconcile: ReconcileFn,
    tweak: impl FnOnce(&mut Timings),
    hooks: Arc<TestHooks>,
) -> TestWorker {
    let config = config(kinds, slots, &test_db_url());
    let mut timings = fast_timings(&config);
    tweak(&mut timings);
    TestWorker::start_with(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, execute, reconcile),
        timings,
        hooks,
    )
}

use super::runtime::Timings;

fn no_tweak(_: &mut Timings) {}

/// The effect sequence of a well-behaved handler: the right fence for the
/// phase at claim, T5, then `Complete`.
fn run_effect(job: &JobView, ctx: &mut ExecutionContext<'_>) -> ExecutionAction {
    let policy = REGISTRY.get(&job.kind).unwrap();
    if !policy.is_effectful() {
        return complete(job);
    }
    let fence = match job.effect_phase {
        AsyncEffectPhase::PreWrite => ctx.mark_effect_started(),
        _ => ctx.confirm_claim(),
    };
    if fence.is_err() {
        return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
    }
    if job.effect_phase != AsyncEffectPhase::EffectConfirmed
        && ctx.mark_effect_confirmed(Some("provider-1")).is_err()
    {
        return fail(FailureClass::Indeterminate, "CONFIRM_REFUSED");
    }
    complete(job)
}

fn effectful() -> ExecuteFn {
    exec_fn(run_effect)
}

fn seconds(duration: chrono::Duration) -> i64 {
    duration.num_seconds()
}

// ---------------------------------------------------------------------------
// W1: configuration
// ---------------------------------------------------------------------------

#[test]
fn w1_configuration_defaults_bounds_and_redaction() {
    const URL_SENTINEL: &str = "W1URLSENTINEL";
    let url = format!("postgres://thoth:{URL_SENTINEL}@127.0.0.1:1/thoth");
    let defaults = WorkerArgs::with_defaults(Some(url.clone()));
    let config = WorkerConfig::from_args(&defaults).expect("defaults are valid");
    assert_eq!(config.worker_name, "thoth-worker");
    assert_eq!(config.slots, 4);
    assert_eq!(config.poll_seconds, 5);
    assert_eq!(config.shutdown_grace_seconds, 20);
    assert_eq!(
        config.liveness_file.to_str(),
        Some("/tmp/thoth-worker-liveness")
    );
    assert!(config.kinds.is_empty() && config.capabilities.is_empty());
    assert!(config.is_idle(), "empty --kinds is idle");

    let with = |f: &dyn Fn(&mut WorkerArgs)| {
        let mut args = WorkerArgs::with_defaults(Some(url.clone()));
        f(&mut args);
        args
    };
    let cases: Vec<(WorkerArgs, &str)> = vec![
        (WorkerArgs::with_defaults(None), "--database-url"),
        (
            WorkerArgs::with_defaults(Some(String::new())),
            "--database-url",
        ),
        (with(&|a| a.worker_name = "Bad".into()), "--worker-name"),
        (with(&|a| a.worker_name = "ab".into()), "--worker-name"),
        (
            with(&|a| a.worker_name = format!("a{}", "b".repeat(64))),
            "--worker-name",
        ),
        (with(&|a| a.kinds = "lower_case".into()), "--kinds"),
        (with(&|a| a.kinds = "AB".into()), "--kinds"),
        (
            with(&|a| a.kinds = "WTEST_READ,WTEST_READ".into()),
            "--kinds",
        ),
        (
            with(&|a| a.capabilities = "Bad.cap".into()),
            "--capabilities",
        ),
        (
            with(&|a| a.capabilities = "a.b,a.b".into()),
            "--capabilities",
        ),
        (with(&|a| a.slots = "0".into()), "--slots"),
        (with(&|a| a.slots = "17".into()), "--slots"),
        (with(&|a| a.slots = "four".into()), "--slots"),
        (with(&|a| a.poll_seconds = "0".into()), "--poll-seconds"),
        (with(&|a| a.poll_seconds = "61".into()), "--poll-seconds"),
        (
            with(&|a| a.shutdown_grace_seconds = "0".into()),
            "--shutdown-grace-seconds",
        ),
        (
            with(&|a| a.shutdown_grace_seconds = "81".into()),
            "--shutdown-grace-seconds",
        ),
        (
            with(&|a| a.liveness_file = "relative/file".into()),
            "--liveness-file",
        ),
        (
            with(&|a| a.liveness_file = String::new()),
            "--liveness-file",
        ),
        (with(&|a| a.liveness_file = "/".into()), "--liveness-file"),
    ];
    for (args, argument) in cases {
        let problems = WorkerConfig::from_args(&args).expect_err(argument);
        assert_eq!(problems.len(), 1, "{argument}: {problems:?}");
        assert!(problems[0].contains(argument), "{problems:?}");
        assert!(!problems[0].contains(URL_SENTINEL));
    }
    let duplicate =
        WorkerConfig::from_args(&with(&|a| a.kinds = "WTEST_READ, WTEST_READ".into())).unwrap_err();
    assert!(duplicate[0].contains("more than once"));
    // Every problem at once, each naming its argument.
    let all = WorkerArgs {
        database_url: None,
        worker_name: "X".into(),
        kinds: "x".into(),
        capabilities: "X".into(),
        slots: "0".into(),
        poll_seconds: "0".into(),
        shutdown_grace_seconds: "0".into(),
        liveness_file: "x".into(),
    };
    assert_eq!(WorkerConfig::from_args(&all).unwrap_err().len(), 8);

    // Debug redacts the database URL.
    assert!(!format!("{config:?}").contains(URL_SENTINEL));
    assert!(!format!("{defaults:?}").contains(URL_SENTINEL));

    // No startup line names the database URL. Idle: no connection is tried.
    logger();
    let worker = TestWorker::start(
        config_with_url(&[], &url),
        &REGISTRY,
        handlers_for(&REGISTRY, effectful(), still_required()),
        fast_timings(&config),
    );
    let hooks = Arc::clone(&worker.hooks);
    assert!(wait_for(|| hooks.liveness().len() >= 3));
    assert_eq!(worker.stop(), 0);
    assert!(logger().containing(URL_SENTINEL).is_empty());
    assert!(!logger().containing("async worker: starting").is_empty());
}

fn config_with_url(kinds: &[&str], url: &str) -> WorkerConfig {
    config(kinds, 2, url)
}

// ---------------------------------------------------------------------------
// W2: AWS credential sources
// ---------------------------------------------------------------------------

fn lookup(map: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<OsString> {
    move |name| map.get(name).map(OsString::from)
}

#[test]
fn w2_aws_credential_sources_refuse_startup() {
    const SENTINEL: &str = "W2AWSSENTINEL";
    for name in AWS_CREDENTIAL_VARIABLES {
        let problems =
            check_aws_sources(lookup(HashMap::from([(name, SENTINEL)]))).expect_err(name);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains(name));
        assert!(!problems[0].contains(SENTINEL));
    }
    let all: HashMap<&'static str, &'static str> = AWS_CREDENTIAL_VARIABLES
        .iter()
        .map(|name| (*name, SENTINEL))
        .collect();
    let problems = check_aws_sources(lookup(all)).unwrap_err();
    assert_eq!(problems.len(), 6);
    assert!(problems.iter().all(|problem| !problem.contains(SENTINEL)));

    let empty: HashMap<&'static str, &'static str> = AWS_CREDENTIAL_VARIABLES
        .iter()
        .map(|name| (*name, ""))
        .collect();
    assert!(check_aws_sources(lookup(empty)).is_ok());
    let allowed = HashMap::from([
        (
            "AWS_CONTAINER_CREDENTIALS_RELATIVE_URI",
            "/v2/credentials/x",
        ),
        ("AWS_REGION", "eu-west-1"),
    ]);
    assert!(check_aws_sources(lookup(allowed)).is_ok());
}

// ---------------------------------------------------------------------------
// W3: handler registry validation
// ---------------------------------------------------------------------------

fn leak_registry(policies: Vec<AsyncJobKindPolicy>) -> &'static AsyncJobRegistry {
    Box::leak(Box::new(AsyncJobRegistry::new(Box::leak(
        policies.into_boxed_slice(),
    ))))
}

fn plain(kind: &'static str, versions: &'static [i16]) -> &'static dyn super::handler::JobHandler {
    handler(kind, versions, effectful(), still_required())
}

fn problems_of(
    handlers: Vec<&'static dyn super::handler::JobHandler>,
    registry: &AsyncJobRegistry,
) -> Vec<String> {
    handler_registry(handlers)
        .validate(registry)
        .err()
        .unwrap_or_default()
}

#[test]
fn w3_handler_registry_validation() {
    assert!(REGISTRY.validate().is_ok());
    assert!(OLDER_REGISTRY.validate().is_ok());
    assert!(handlers_for(&REGISTRY, effectful(), still_required())
        .validate(&REGISTRY)
        .is_ok());
    assert!(PRODUCTION_HANDLERS.is_empty() && PRODUCTION_REGISTRY.is_empty());
    assert!(PRODUCTION_HANDLERS.validate(&PRODUCTION_REGISTRY).is_ok());

    let read = leak_registry(vec![READ]);
    // 1. Served twice.
    let found = problems_of(vec![plain(W_READ, &[1]), plain(W_READ, &[1])], read);
    assert!(
        found.iter().any(|p| p.contains("served twice")),
        "{found:?}"
    );
    // 2. Not in the registry.
    let found = problems_of(
        vec![plain(W_READ, &[1]), plain("WTEST_UNKNOWN", &[1])],
        read,
    );
    assert!(found
        .iter()
        .any(|p| p.contains("WTEST_UNKNOWN") && p.contains("not in the registry")));
    // 3. Neither domain is ThothWorker.
    let found = problems_of(
        vec![plain(W_DISSEMINATION, &[1])],
        leak_registry(vec![DISSEMINATION]),
    );
    assert!(found
        .iter()
        .any(|p| p.contains("neither the executor nor the reconciler")));
    // 4. Not Trusted.
    let found = problems_of(
        vec![plain(W_UNTRUSTED, &[1])],
        leak_registry(vec![UNTRUSTED]),
    );
    assert!(found
        .iter()
        .any(|p| p.contains("risk class is not Trusted")));
    // 5. Versions empty or not a subset.
    for versions in [&[][..], &[2][..]] {
        let found = problems_of(vec![plain(W_READ, versions)], read);
        assert!(
            found.iter().any(|p| p.contains("payload versions")),
            "{found:?}"
        );
    }
    // 6. Completeness.
    let found = problems_of(vec![plain(W_READ, &[1])], leak_registry(vec![READ, EFFECT]));
    assert!(found
        .iter()
        .any(|p| p.contains(W_EFFECT) && p.contains("has no handler")));
    // 7. The effect lease cannot cover the window plus the margin.
    let tight = AsyncJobKindPolicy {
        lease_default_seconds: 30,
        lease_max_seconds: 35,
        effect_window_seconds: Some(30),
        ..EFFECT
    };
    let tight_registry = leak_registry(vec![tight]);
    assert!(tight_registry.validate().is_ok(), "a valid engine policy");
    let found = problems_of(vec![plain(W_EFFECT, &[1])], tight_registry);
    assert!(
        found
            .iter()
            .any(|p| p.contains("exceeds lease_max_seconds")),
        "{found:?}"
    );
}

// ---------------------------------------------------------------------------
// W4: claim sets
// ---------------------------------------------------------------------------

#[test]
fn w4_claim_sets_leave_out_with_one_line_each() {
    let handlers = handler_registry(
        REGISTRY
            .policies()
            .iter()
            .filter(|policy| policy.kind != W_REVISION)
            .map(|policy| plain(policy.kind, policy.executable_versions))
            .collect(),
    );
    let args = WorkerArgs::with_defaults(Some("postgres://unused".into()));
    let executor = worker_executor(&WorkerConfig::from_args(&args).unwrap());
    let listed: Vec<String> = [
        W_READ,
        W_RECONCILER,
        W_CAPABILITY,
        W_UNTRUSTED,
        W_DISSEMINATION,
        W_REVISION,
        "WTEST_NOPE",
        W_EFFECT,
    ]
    .iter()
    .map(|kind| kind.to_string())
    .collect();
    let (sets, notes) = compute_claim_sets(&listed, &REGISTRY, handlers, &executor);
    let kinds = |set: &[(String, Vec<i16>)]| set.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>();
    assert_eq!(kinds(&sets.execution), vec![W_EFFECT, W_READ]);
    assert_eq!(
        kinds(&sets.reconciliation),
        vec![W_EFFECT, W_READ, W_RECONCILER]
    );
    assert!(sets
        .execution
        .iter()
        .all(|(_, versions)| versions == &vec![1]));
    assert_eq!(notes.len(), 5, "{notes:?}");
    for left_out in [
        W_CAPABILITY,
        W_UNTRUSTED,
        W_DISSEMINATION,
        W_REVISION,
        "WTEST_NOPE",
    ] {
        assert_eq!(
            notes.iter().filter(|n| n.contains(left_out)).count(),
            1,
            "{left_out}: {notes:?}"
        );
    }
    assert_eq!(sets.union_kinds(), vec![W_EFFECT, W_READ, W_RECONCILER]);
    assert_eq!(
        sets.describe(Purpose::Execution),
        format!("{W_EFFECT}, {W_READ}")
    );
    assert!(ClaimSets::default().is_empty());
}

// ---------------------------------------------------------------------------
// W5: inert default
// ---------------------------------------------------------------------------

#[test]
fn w5_inert_default_builds_no_pool_and_changes_nothing() {
    let (_guard, mut conn) = setup();
    let jobs: Vec<Uuid> = (0..3)
        .map(|_| create_kind(&mut conn, W_READ).async_job_id)
        .collect();
    let url = url_with_application("wtest_w5_idle");
    let config = config(&[], 2, &url);
    let timings = fast_timings(&config);
    let worker = TestWorker::start(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, effectful(), still_required()),
        timings,
    );
    let hooks = Arc::clone(&worker.hooks);
    let file = worker.liveness_file();
    assert!(wait_for(|| hooks.liveness().len() >= 10));
    assert!(is_healthy(&file, 15, super::health::unix_now()));
    assert_eq!(
        scalar(
            &mut conn,
            "SELECT count(*) AS value FROM pg_stat_activity WHERE application_name = 'wtest_w5_idle'"
        ),
        0
    );
    assert_eq!(hooks.pools_built(), 0);
    assert!(!worker.shared().pools_built());
    assert_eq!(worker.stop(), 0);
    for id in jobs {
        let job = job(&mut conn, id);
        assert_eq!(job.status, AsyncJobStatus::Pending);
        assert_eq!(job.execution_claim_count, 0);
    }
    assert_eq!(
        scalar(
            &mut conn,
            "SELECT count(*) AS value FROM public.async_executor_poll"
        ),
        0
    );
}

// ---------------------------------------------------------------------------
// W6: success
// ---------------------------------------------------------------------------

#[test]
fn w6_success_read_only_and_effectful() {
    let (_guard, mut conn) = setup();
    let reached = Flag::new();
    let go = Flag::new();
    let (reached_h, go_h) = (Arc::clone(&reached), Arc::clone(&go));
    let execute = exec_fn(move |job, ctx| {
        if job.kind == W_READ {
            return complete(job);
        }
        if ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        reached_h.set();
        go_h.wait();
        if ctx.mark_effect_confirmed(Some("provider-1")).is_err() {
            return fail(FailureClass::Indeterminate, "CONFIRM_REFUSED");
        }
        complete(job)
    });
    let args = WorkerArgs {
        worker_name: "wtest-worker".into(),
        kinds: format!("{W_READ},{W_EFFECT}"),
        slots: "2".into(),
        liveness_file: liveness_path().to_string_lossy().into_owned(),
        ..WorkerArgs::with_defaults(Some(test_db_url()))
    };
    let config = WorkerConfig::from_args(&args).unwrap();
    let timings = fast_timings(&config);
    let read = create_kind(&mut conn, W_READ).async_job_id;
    let effect = create_kind(&mut conn, W_EFFECT);
    let worker = TestWorker::start(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, execute, still_required()),
        timings,
    );
    assert!(reached.wait());
    // The first renew_on + mark_effect_started_on pairing in one transaction.
    let row = job(&mut conn, effect.async_job_id);
    let attempt = last_execution(&mut conn, effect.async_job_id);
    assert_eq!(row.effect_phase, AsyncEffectPhase::EffectStarted);
    let started_at = attempt.effect_started_at.expect("effect_started_at");
    assert!(
        attempt.last_heartbeat_at > attempt.started_at,
        "the paired renew_on ran"
    );
    let lease = row.lease_expires_at.unwrap();
    assert!(
        seconds(lease - started_at) >= 30,
        "the lease covers the window"
    );
    assert!(
        seconds(lease - db_now(&mut conn)) >= 50,
        "the effect lease E = 60"
    );
    go.set();
    assert!(wait_status(&mut conn, read, "SUCCEEDED"));
    assert!(wait_status(&mut conn, effect.async_job_id, "SUCCEEDED"));
    // One poll row per kind and purpose of the claim sets, once the loop has
    // made a claim call of each purpose.
    let expected = {
        let sets = worker.shared().sets();
        i64::try_from(sets.execution.len() + sets.reconciliation.len()).unwrap()
    };
    assert_eq!(
        expected, 4,
        "both kinds are executed and reconciled by ThothWorker"
    );
    let poll_rows = |conn: &mut PgConnection| {
        scalar(
            conn,
            "SELECT count(*) AS value FROM public.async_executor_poll \
              WHERE executor_name = 'wtest-worker' AND executor_principal = 'process:thoth-worker'",
        )
    };
    assert!(wait_for(|| poll_rows(&mut conn) == expected));
    assert_eq!(worker.stop(), 0);

    assert_eq!(
        job(&mut conn, read).last_outcome_code.as_deref(),
        Some("COMPLETED")
    );
    let done = job(&mut conn, effect.async_job_id);
    assert_eq!(done.last_outcome_code.as_deref(), Some("EFFECT_APPLIED"));
    let target = &effect.target_keys[0];
    assert_eq!(
        scalar(
            &mut conn,
            &format!(
                "SELECT count(*) AS value FROM public.async_effect_target_state \
                  WHERE target_key = '{target}' AND last_job_id = '{}'",
                effect.async_job_id
            )
        ),
        1
    );
    for id in [read, effect.async_job_id] {
        let attempt = last_execution(&mut conn, id);
        assert_eq!(attempt.claimed_by, "wtest-worker");
        assert_eq!(attempt.executor_principal, "process:thoth-worker");
    }
    assert_eq!(
        poll_rows(&mut conn),
        expected,
        "one row per kind and purpose"
    );
    assert_eq!(
        scalar(
            &mut conn,
            "SELECT count(DISTINCT (kind, purpose)) AS value FROM public.async_executor_poll"
        ),
        expected
    );
}

// ---------------------------------------------------------------------------
// W7: retryable and permanent failures
// ---------------------------------------------------------------------------

#[test]
fn w7_retryable_then_success_budget_exhausted_and_permanent() {
    let (_guard, mut conn) = setup();
    let execute = exec_fn(|job, _ctx| match mode_of(job).as_str() {
        "retry-once" if job.attempt_number == 1 => {
            fail(FailureClass::RetryablePreWrite, "PROVIDER_UNAVAILABLE")
        }
        "retry-once" => complete(job),
        "retry-always" => fail(FailureClass::RetryablePreWrite, "PROVIDER_UNAVAILABLE"),
        _ => fail(FailureClass::Permanent, "BAD_INPUT"),
    });
    let once = mode_job(&mut conn, W_READ, "retry-once").async_job_id;
    let always = mode_job(&mut conn, W_READ, "retry-always").async_job_id;
    let permanent = mode_job(&mut conn, W_READ, "permanent").async_job_id;
    let worker = start(&[W_READ], 3, execute, still_required(), no_tweak);

    assert!(wait_status(&mut conn, permanent, "FAILED"));
    assert_eq!(execution_attempts(&mut conn, permanent).len(), 1);

    assert!(wait_status(&mut conn, once, "RETRY_SCHEDULED"));
    make_available(&mut conn, once);
    assert!(wait_status(&mut conn, once, "SUCCEEDED"));
    assert_eq!(execution_attempts(&mut conn, once).len(), 2);

    let mut rounds = 0;
    while status(&mut conn, always) != "FAILED" {
        assert!(wait_for(|| {
            let now = status(&mut conn, always);
            now == "RETRY_SCHEDULED" || now == "FAILED"
        }));
        if status(&mut conn, always) == "RETRY_SCHEDULED" {
            make_available(&mut conn, always);
            assert!(wait_for(|| status(&mut conn, always) != "RETRY_SCHEDULED"
                || job(&mut conn, always).available_at > Utc::now()));
        }
        rounds += 1;
        assert!(rounds < 20);
    }
    assert_eq!(worker.stop(), 0);
    assert_eq!(
        job(&mut conn, always).last_outcome_code.as_deref(),
        Some("RETRY_BUDGET_EXHAUSTED")
    );
    assert_eq!(
        job(&mut conn, permanent).last_outcome_code.as_deref(),
        Some("PERMANENT_FAILURE")
    );
    assert_eq!(
        last_execution(&mut conn, permanent).error_code.as_deref(),
        Some("BAD_INPUT")
    );
    assert_eq!(execution_attempts(&mut conn, permanent).len(), 1);
}

// ---------------------------------------------------------------------------
// W8: wait and resume
// ---------------------------------------------------------------------------

#[test]
fn w8_wait_then_resume_with_checkpoint_and_confirm_claim() {
    let (_guard, mut conn) = setup();
    let seen = notes();
    let seen_h = Arc::clone(&seen);
    let execute = exec_fn(move |job, ctx| {
        if job.checkpoint.is_none() {
            if ctx.mark_effect_started().is_err() {
                return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
            }
            if ctx.mark_effect_confirmed(Some("provider-1")).is_err() {
                return fail(FailureClass::Indeterminate, "CONFIRM_REFUSED");
            }
            return ExecutionAction::Wait(WaitAction {
                checkpoint: json!({ "step": 1 }).to_string(),
                resume_after: Duration::ZERO,
                provider_ref: None,
            });
        }
        note(
            &seen_h,
            format!(
                "{}|{}|{:?}|{:?}",
                job.checkpoint
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
                    .unwrap_or_default(),
                job.checkpoint_version,
                job.effect_phase,
                ctx.confirm_claim()
            ),
        );
        complete(job)
    });
    let id = mode_job(&mut conn, W_RESUMABLE, "wait").async_job_id;
    let worker = start(&[W_RESUMABLE], 1, execute, still_required(), no_tweak);
    assert!(wait_status(&mut conn, id, "WAITING"));
    let waiting = job(&mut conn, id);
    let attempt = last_execution(&mut conn, id);
    // resume_at = clock + poll_min + margin, the clock read in the wait's
    // transaction just before the attempt closed.
    let delay = (waiting.available_at - attempt.finished_at.unwrap()).num_milliseconds();
    assert!((14_000..=15_000).contains(&delay), "{delay} ms");
    make_available(&mut conn, id);
    assert!(wait_status(&mut conn, id, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
    assert_eq!(
        noted(&seen),
        vec![format!("{}|1|EffectConfirmed|Ok(())", json!({ "step": 1 }))]
    );
}

// ---------------------------------------------------------------------------
// W9: heartbeat
// ---------------------------------------------------------------------------

fn test_claim_state(
    policy: &'static AsyncJobKindPolicy,
    expiry_seconds: i64,
    renewed_ago: Duration,
    lost: bool,
) -> Arc<ClaimState> {
    let now = Instant::now();
    Arc::new(ClaimState {
        claim: WorkerClaim {
            async_job_id: Uuid::new_v4(),
            claim_token: Uuid::new_v4(),
            domain: ExecutorDomain::ThothWorker,
        },
        job_id: Uuid::new_v4(),
        policy,
        purpose: Purpose::Execution,
        payload_version: 1,
        attempt_number: 1,
        renewal: Mutex::new(None),
        status: Mutex::new(ClaimStatus {
            phase: TrackedPhase::PreWrite,
            claim_lost: lost,
            lease_expired: false,
            lost_at: None,
            finishing: false,
            handler_returned: false,
            ended: false,
            last_renewal: now.checked_sub(renewed_ago).unwrap_or(now),
            lease_expires_at: Utc::now() + chrono::Duration::seconds(expiry_seconds),
            deadline_estimate: now + Duration::from_secs(600),
            shutdown_after_effect_mark: false,
            slot: None,
        }),
    })
}

#[test]
fn w9_heartbeat_renews_soonest_expiry_first() {
    let config = config(&[W_SHORT], 1, "postgres://unused");
    let timings = fast_timings(&config);
    let late = test_claim_state(&SHORT, 30, Duration::from_secs(10), false);
    let soon = test_claim_state(&SHORT, 10, Duration::from_secs(10), false);
    let middle = test_claim_state(&SHORT, 20, Duration::from_secs(10), false);
    let lost = test_claim_state(&SHORT, 5, Duration::from_secs(10), true);
    let fresh = test_claim_state(&SHORT, 1, Duration::ZERO, false);
    let due = due_claims(
        vec![late.clone(), lost, soon.clone(), fresh, middle.clone()],
        Instant::now(),
        &timings,
    );
    let order: Vec<Uuid> = due.iter().map(|state| state.job_id).collect();
    assert_eq!(order, vec![soon.job_id, middle.job_id, late.job_id]);
    assert_eq!(timings.renew_every(&SHORT), Duration::from_secs(5));
}

#[test]
fn w9_heartbeat_extends_the_lease_never_past_the_deadline() {
    let (_guard, mut conn) = setup();
    let started = Flag::new();
    let yielded = Flag::new();
    let go = Flag::new();
    let elapsed = notes();
    let (started_h, yielded_h, go_h, elapsed_h) = (
        Arc::clone(&started),
        Arc::clone(&yielded),
        Arc::clone(&go),
        Arc::clone(&elapsed),
    );
    let execute = exec_fn(move |job, ctx| {
        let at = Instant::now();
        started_h.set();
        assert!(!ctx.should_yield(), "20 s left, horizon 15 s");
        while !ctx.should_yield() && at.elapsed() < Duration::from_secs(19) {
            thread::sleep(Duration::from_millis(50));
        }
        note(&elapsed_h, at.elapsed().as_millis().to_string());
        yielded_h.set();
        go_h.wait();
        complete(job)
    });
    let id = create_kind(&mut conn, W_SHORT).async_job_id;
    let worker = start(&[W_SHORT], 1, execute, still_required(), |t| {
        t.renew_interval = None;
    });
    assert!(started.wait());
    let first = job(&mut conn, id);
    let first_lease = first.lease_expires_at.unwrap();
    let deadline = first.attempt_runtime_deadline.unwrap();
    // A third of the 15 s lease: the first renewal comes after about 5 s.
    assert!(wait_for(
        || job(&mut conn, id).lease_expires_at.unwrap() > first_lease
    ));
    let attempt = last_execution(&mut conn, id);
    assert!(attempt.last_heartbeat_at > attempt.started_at);
    // About 5 s plus the 15 s lease reaches the 20 s deadline: capped.
    assert!(job(&mut conn, id).lease_expires_at.unwrap() <= deadline);
    let first_heartbeat = attempt.last_heartbeat_at;
    assert!(yielded.wait());
    let waited: u128 = noted(&elapsed)[0].parse().unwrap();
    assert!(
        waited >= 4_000,
        "should_yield turned true near the deadline ({waited} ms)"
    );
    // The next renewal (about 10 s in) asks past the deadline and is capped
    // at it.
    assert!(wait_for(
        || last_execution(&mut conn, id).last_heartbeat_at > first_heartbeat
    ));
    assert_eq!(job(&mut conn, id).lease_expires_at.unwrap(), deadline);
    go.set();
    assert!(wait_status(&mut conn, id, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
}

// ---------------------------------------------------------------------------
// W10: the effect lease
// ---------------------------------------------------------------------------

#[test]
fn w10_effect_lease_covers_the_window_with_a_heartbeat_renewal_in_flight() {
    let (_guard, mut conn) = setup();
    let (ready, go, calling, started, done) = (
        Flag::new(),
        Flag::new(),
        Flag::new(),
        Flag::new(),
        Flag::new(),
    );
    let results = notes();
    let flags = (
        Arc::clone(&ready),
        Arc::clone(&go),
        Arc::clone(&calling),
        Arc::clone(&started),
        Arc::clone(&done),
        Arc::clone(&results),
    );
    let execute = exec_fn(move |job, ctx| {
        let (ready, go, calling, started, done, results) = &flags;
        ready.set();
        go.wait();
        calling.set();
        let result = ctx.mark_effect_started();
        note(results, format!("{result:?}"));
        started.set();
        done.wait();
        if ctx.mark_effect_confirmed(None).is_err() {
            return fail(FailureClass::Indeterminate, "CONFIRM_REFUSED");
        }
        complete(job)
    });
    let hooks = Arc::new(TestHooks::default());
    hooks.heartbeat_gate.arm();
    let id = create_kind(&mut conn, W_LONG).async_job_id;
    let worker = start_hooked(
        &[W_LONG],
        1,
        execute,
        still_required(),
        |t| t.renew_interval = Some(Duration::from_millis(300)),
        Arc::clone(&hooks),
    );
    assert!(ready.wait());
    // A heartbeat renewal with request None is in flight, holding the lock.
    assert!(hooks.heartbeat_gate.wait_reached());
    go.set();
    assert!(calling.wait());
    // Ordering aid only: let the handler reach the renewal lock.
    thread::sleep(Duration::from_millis(100));
    hooks.heartbeat_gate.release();
    assert!(started.wait());
    assert_eq!(noted(&results), vec!["Ok(())".to_string()]);
    let effect_started = last_execution(&mut conn, id).effect_started_at.unwrap();
    let mut heartbeat = last_execution(&mut conn, id).last_heartbeat_at;
    for _ in 0..4 {
        let row = job(&mut conn, id);
        let lease = row.lease_expires_at.unwrap();
        assert!(
            seconds(lease - effect_started) >= 90,
            "lease covers T4 + window"
        );
        assert!(
            seconds(lease - db_now(&mut conn)) >= 90,
            "renewals after T4 send Some(E = 100)"
        );
        assert!(wait_for(
            || last_execution(&mut conn, id).last_heartbeat_at > heartbeat
        ));
        heartbeat = last_execution(&mut conn, id).last_heartbeat_at;
    }
    done.set();
    assert!(wait_status(&mut conn, id, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
}

#[test]
fn w10_resumed_fence_and_window_insufficient() {
    let (_guard, mut conn) = setup();
    // Set up before the runtime starts: two resumed jobs at EFFECT_STARTED.
    let resumed = to_waiting(&mut conn, W_LONG, "resume", AsyncEffectPhase::EffectStarted);
    let insufficient = to_waiting(
        &mut conn,
        W_LONG,
        "resume-short",
        AsyncEffectPhase::EffectStarted,
    );
    // One (reached, go) pair per handler step; a Flag stays set once set.
    let pair = || (Flag::new(), Flag::new());
    let (resume_reached, resume_go) = pair();
    let (confirmed_reached, confirmed_go) = pair();
    let (pre_reached, pre_go) = pair();
    let (short_reached, short_go) = pair();
    let results = notes();
    let flags = (
        [
            Arc::clone(&resume_reached),
            Arc::clone(&resume_go),
            Arc::clone(&confirmed_reached),
            Arc::clone(&confirmed_go),
            Arc::clone(&pre_reached),
            Arc::clone(&pre_go),
            Arc::clone(&short_reached),
            Arc::clone(&short_go),
        ],
        Arc::clone(&results),
    );
    let execute = exec_fn(move |job, ctx| {
        let (
            [resume_reached, resume_go, confirmed_reached, confirmed_go, pre_reached, pre_go, short_reached, short_go],
            results,
        ) = &flags;
        match mode_of(job).as_str() {
            "resume" => {
                resume_reached.set();
                resume_go.wait();
                let result = ctx.confirm_claim();
                note(results, format!("resume {result:?}"));
                confirmed_reached.set();
                confirmed_go.wait();
                if result.is_err() || ctx.mark_effect_confirmed(None).is_err() {
                    return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
                }
                complete(job)
            }
            "resume-short" => {
                short_reached.set();
                short_go.wait();
                let result = ctx.confirm_claim();
                note(results, format!("resume-short {result:?}"));
                fail(FailureClass::Indeterminate, "WINDOW_INSUFFICIENT")
            }
            _ => {
                pre_reached.set();
                pre_go.wait();
                let result = ctx.mark_effect_started();
                note(results, format!("start {result:?}"));
                match result {
                    Err(HandlerError::EffectWindowInsufficient) => ExecutionAction::Release,
                    _ => fail(FailureClass::Indeterminate, "UNEXPECTED"),
                }
            }
        }
    });
    let worker = start(&[W_LONG], 1, execute, still_required(), no_tweak);

    // A resumed attempt's confirm_claim at EFFECT_STARTED sends Some(E).
    make_available(&mut conn, resumed);
    assert!(resume_reached.wait());
    resume_go.set();
    assert!(confirmed_reached.wait());
    let lease = job(&mut conn, resumed).lease_expires_at.unwrap();
    assert!(
        seconds(lease - db_now(&mut conn)) >= 90,
        "the fence plus the window"
    );
    confirmed_go.set();
    assert!(wait_status(&mut conn, resumed, "SUCCEEDED"));

    // The deadline too close: mark_effect_started answers
    // EffectWindowInsufficient and the Release leaves PENDING.
    let pre = create_kind(&mut conn, W_LONG).async_job_id;
    assert!(pre_reached.wait());
    set_runtime_deadline(&mut conn, pre, 60);
    worker.shared().set_remaining(Duration::from_secs(60));
    pre_go.set();
    assert!(wait_status(&mut conn, pre, "PENDING"));
    let row = job(&mut conn, pre);
    assert_eq!(row.effect_phase, AsyncEffectPhase::PreWrite);
    assert_eq!(row.prewrite_failure_count, 0);

    // A post-effect confirm_claim with less than the window plus the margin
    // left answers EffectWindowInsufficient before any call.
    make_available(&mut conn, insufficient);
    assert!(short_reached.wait());
    worker.shared().set_remaining(Duration::from_secs(95));
    short_go.set();
    assert!(wait_status(
        &mut conn,
        insufficient,
        "RECONCILIATION_REQUIRED"
    ));
    assert_eq!(worker.stop(), 0);
    let results = noted(&results);
    assert!(
        results.contains(&"resume Ok(())".to_string()),
        "{results:?}"
    );
    assert!(
        results.contains(&"start Err(EffectWindowInsufficient)".to_string()),
        "{results:?}"
    );
    assert!(results.contains(&"resume-short Err(EffectWindowInsufficient)".to_string()));
}

// ---------------------------------------------------------------------------
// W11: claim loss and lease expiry
// ---------------------------------------------------------------------------

#[test]
fn w11_claim_taken_away_by_recovery_or_cancel() {
    let (_guard, mut conn) = setup();
    let ready = Flag::new();
    let results = notes();
    let (ready_h, results_h) = (Arc::clone(&ready), Arc::clone(&results));
    let execute = exec_fn(move |job, ctx| {
        ready_h.set();
        let lost = wait_for(|| ctx.claim_lost());
        let fence = ctx.mark_effect_started();
        note(&results_h, format!("{} {lost} {fence:?}", mode_of(job)));
        complete(job)
    });
    let recovered = mode_job(&mut conn, W_EFFECT, "recovered").async_job_id;
    let worker = start(&[W_EFFECT], 1, execute, still_required(), |t| {
        t.renew_interval = Some(Duration::from_millis(100));
    });
    assert!(ready.wait());
    // Another claim call recovers the job (T12) in the same transaction as
    // the lease moves into the past.
    tx(&mut conn, |c| {
        c.batch_execute(&format!(
            "UPDATE public.async_job SET lease_expires_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{recovered}'"
        ))
        .map_err(|_| ThothError::AsyncJobIntegrityViolation)?;
        claim_on(
            c,
            &REGISTRY,
            &setup_executor(),
            ClaimRequest {
                kinds: vec![(W_EFFECT.to_string(), vec![1])],
                max_jobs: 1,
                lease_seconds: None,
            },
        )
    })
    .expect("a peer claim call");
    assert!(wait_for(|| noted(&results).len() == 1));
    assert!(wait_for(|| worker.shared().in_flight().is_empty()));
    assert_eq!(
        noted(&results),
        vec!["recovered true Err(ClaimLost)".to_string()]
    );
    let row = job(&mut conn, recovered);
    assert_eq!(row.status, AsyncJobStatus::RetryScheduled);
    assert_eq!(row.effect_phase, AsyncEffectPhase::PreWrite);
    assert!(last_execution(&mut conn, recovered)
        .effect_started_at
        .is_none());
    assert!(!logger()
        .containing(&format!("job {recovered} "))
        .iter()
        .filter(|line| line.contains("-> ASYNC_JOB_STALE_CLAIM"))
        .collect::<Vec<_>>()
        .is_empty());

    // An operator cancel at pre-write.
    let ready_again = Arc::clone(&ready);
    let _ = ready_again;
    let cancelled = mode_job(&mut conn, W_EFFECT, "cancelled").async_job_id;
    assert!(wait_for(|| status(&mut conn, cancelled) == "RUNNING"));
    assert!(wait_for(
        || noted(&results).len() == 1 && worker.shared().in_flight().len() == 1
    ));
    tx(&mut conn, |c| {
        cancel_job_command_on(
            c,
            &REGISTRY,
            &StaffActor {
                principal: "staff:operator@example.org".to_string(),
                reason: "test cancel".to_string(),
                request_id: Some(Uuid::new_v4()),
            },
            cancelled,
        )
    })
    .expect("cancel");
    assert!(wait_for(|| noted(&results).len() == 2));
    assert!(wait_for(|| worker.shared().in_flight().is_empty()));
    assert_eq!(noted(&results)[1], "cancelled true Err(ClaimLost)");
    assert_eq!(status(&mut conn, cancelled), "CANCELLED");
    assert!(worker.shared().counters.lost_claims.load(Ordering::Relaxed) >= 2);
    assert_eq!(worker.stop(), 0);
}

#[test]
fn w11_lease_expired_refuses_t4_but_complete_after_t5_commits() {
    let (_guard, mut conn) = setup();
    let ready = notes();
    let results = notes();
    let (ready_h, results_h) = (Arc::clone(&ready), Arc::clone(&results));
    let execute = exec_fn(move |job, ctx| {
        let mode = mode_of(job);
        if mode == "after-t5"
            && (ctx.mark_effect_started().is_err() || ctx.mark_effect_confirmed(None).is_err())
        {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        note(&ready_h, mode.clone());
        let expired = wait_for(|| ctx.lease_expired());
        if mode == "after-t5" {
            note(&results_h, format!("after-t5 {expired}"));
            return complete(job);
        }
        let fence = ctx.mark_effect_started();
        note(&results_h, format!("pre {expired} {fence:?}"));
        ExecutionAction::Release
    });
    let pre = mode_job(&mut conn, W_EFFECT, "pre").async_job_id;
    let after = mode_job(&mut conn, W_EFFECT, "after-t5").async_job_id;
    let worker = start(&[W_EFFECT], 2, execute, still_required(), |t| {
        t.renew_interval = Some(Duration::from_millis(100));
    });
    assert!(wait_for(|| noted(&ready).len() == 2));
    // No recovery: every claim and recovery call answers busy from now.
    for op in [Op::Claim, Op::ClaimReconciliation, Op::Recover] {
        worker
            .hooks
            .push_faults(op, (0..100_000).map(|_| Fault::busy()).collect());
    }
    expire_lease(&mut conn, pre);
    expire_lease(&mut conn, after);
    assert!(wait_for(|| noted(&results).len() == 2));
    assert!(wait_status(&mut conn, after, "SUCCEEDED"));
    assert!(wait_for(|| worker.shared().in_flight().is_empty()));
    let results = noted(&results);
    assert!(
        results.contains(&"pre true Err(LeaseExpired)".to_string()),
        "{results:?}"
    );
    assert!(results.contains(&"after-t5 true".to_string()));
    let row = job(&mut conn, pre);
    assert_eq!(row.effect_phase, AsyncEffectPhase::PreWrite);
    assert!(last_execution(&mut conn, pre).effect_started_at.is_none());
    assert!(
        worker
            .shared()
            .counters
            .expired_leases
            .load(Ordering::Relaxed)
            >= 2
    );
    assert_eq!(worker.stop(), 0);
}

// ---------------------------------------------------------------------------
// W12, W13, W14: shutdown
// ---------------------------------------------------------------------------

#[test]
fn w12_shutdown_at_pre_write() {
    let (_guard, mut conn) = setup();
    let running = Arc::new(AtomicUsize::new(0));
    let release = Flag::new();
    let (running_h, release_h) = (Arc::clone(&running), Arc::clone(&release));
    let execute = exec_fn(move |job, ctx| {
        running_h.fetch_add(1, Ordering::SeqCst);
        if mode_of(job) == "uncooperative" {
            release_h.wait();
        } else {
            wait_for(|| ctx.shutdown_requested());
        }
        ExecutionAction::Release
    });
    let cooperative = mode_job(&mut conn, W_EFFECT, "cooperative").async_job_id;
    let uncooperative = mode_job(&mut conn, W_EFFECT, "uncooperative").async_job_id;
    let capped = mode_job(&mut conn, W_EFFECT, "cooperative").async_job_id;
    bypass(
        &mut conn,
        &format!(
            "UPDATE public.async_job SET execution_claim_count = 3, attempt_count = 3, \
              first_claimed_at = clock_timestamp() \
              WHERE async_job_id = '{capped}'"
        ),
    );
    let worker = start(&[W_EFFECT], 3, execute, still_required(), |t| {
        t.grace = Duration::from_millis(500);
    });
    let counters_rt = Arc::clone(worker.shared());
    assert!(wait_for(|| running.load(Ordering::SeqCst) == 3));
    let code = worker.stop_then(|| release.set());
    assert_eq!(code, 0);
    for id in [cooperative, uncooperative] {
        let row = job(&mut conn, id);
        assert_eq!(row.status, AsyncJobStatus::Pending, "{id}");
        assert_eq!(row.prewrite_failure_count, 0);
        assert_eq!(row.execution_claim_count, 1);
    }
    let row = job(&mut conn, capped);
    assert_eq!(row.status, AsyncJobStatus::Failed);
    assert_eq!(row.last_outcome_code.as_deref(), Some("ATTEMPT_CAP"));
    assert_eq!(
        counters_rt
            .counters
            .shutdown_fallbacks
            .load(Ordering::Relaxed),
        1
    );
}

#[test]
fn w13_shutdown_after_effect_start_and_resumable_confirmed() {
    let (_guard, mut conn) = setup();
    let confirmed = to_waiting(
        &mut conn,
        W_RESUMABLE,
        "block",
        AsyncEffectPhase::EffectConfirmed,
    );
    let running = Arc::new(AtomicUsize::new(0));
    let release = Flag::new();
    let (running_h, release_h) = (Arc::clone(&running), Arc::clone(&release));
    let execute = exec_fn(move |job, ctx| {
        let mode = mode_of(job);
        if mode == "block" {
            running_h.fetch_add(1, Ordering::SeqCst);
            release_h.wait();
            return complete(job);
        }
        if ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        running_h.fetch_add(1, Ordering::SeqCst);
        if mode == "uncooperative" {
            release_h.wait();
            return complete(job);
        }
        wait_for(|| ctx.shutdown_requested());
        if ctx.mark_effect_confirmed(None).is_err() {
            return fail(FailureClass::Indeterminate, "CONFIRM_REFUSED");
        }
        complete(job)
    });
    let cooperative = mode_job(&mut conn, W_EFFECT, "cooperative").async_job_id;
    let uncooperative = mode_job(&mut conn, W_EFFECT, "uncooperative").async_job_id;
    make_available(&mut conn, confirmed);
    let worker = start(
        &[W_EFFECT, W_RESUMABLE],
        3,
        execute,
        still_required(),
        |t| {
            t.grace = Duration::from_millis(500);
        },
    );
    assert!(wait_for(|| running.load(Ordering::SeqCst) == 3));
    assert_eq!(worker.stop_then(|| release.set()), 0);
    assert_eq!(status(&mut conn, cooperative), "SUCCEEDED");
    assert_eq!(status(&mut conn, uncooperative), "RECONCILIATION_REQUIRED");
    let attempt = last_execution(&mut conn, uncooperative);
    assert_eq!(attempt.error_code.as_deref(), Some("WORKER_SHUTDOWN"));
    assert!(job_transitions(&mut conn, uncooperative)
        .unwrap()
        .iter()
        .all(|t| t.to_status != AsyncJobStatus::RetryScheduled));
    assert_eq!(status(&mut conn, confirmed), "WAITING");
    let waiting = job(&mut conn, confirmed);
    assert_eq!(waiting.effect_phase, AsyncEffectPhase::EffectConfirmed);
    assert!(waiting.checkpoint.is_some());

    // A second runtime resumes and completes the confirmed job.
    make_available(&mut conn, confirmed);
    let second = start(&[W_RESUMABLE], 1, effectful(), still_required(), no_tweak);
    assert!(wait_status(&mut conn, confirmed, "SUCCEEDED"));
    assert_eq!(second.stop(), 0);
}

#[test]
fn w14_fences_refuse_after_the_signal() {
    let (_guard, mut conn) = setup();
    let results = notes();
    let ready = Flag::new();
    let (results_h, ready_h) = (Arc::clone(&results), Arc::clone(&ready));
    let execute = exec_fn(move |_job, ctx| {
        ready_h.set();
        wait_for(|| ctx.shutdown_requested());
        let start = ctx.mark_effect_started();
        let confirm = ctx.confirm_claim();
        note(
            &results_h,
            format!("{start:?} {confirm:?} {:?}", ctx.tracked_phase()),
        );
        ExecutionAction::Release
    });
    let id = mode_job(&mut conn, W_EFFECT, "cooperative").async_job_id;
    let worker = start(&[W_EFFECT], 1, execute, still_required(), no_tweak);
    assert!(ready.wait());
    assert_eq!(worker.stop(), 0);
    assert_eq!(
        noted(&results),
        vec!["Err(ShutdownInProgress) Err(ShutdownInProgress) PreWrite".to_string()]
    );
    let row = job(&mut conn, id);
    assert_eq!(row.status, AsyncJobStatus::Pending);
    assert_eq!(row.effect_phase, AsyncEffectPhase::PreWrite);
}

#[test]
fn w14_t4_committed_before_the_fallback_release_gets_indeterminate() {
    let (_guard, mut conn) = setup();
    let ready = Flag::new();
    let ready_h = Arc::clone(&ready);
    let results = notes();
    let results_h = Arc::clone(&results);
    let execute = exec_fn(move |_job, ctx| {
        ready_h.set();
        let start = ctx.mark_effect_started();
        note(&results_h, format!("{start:?} {:?}", ctx.tracked_phase()));
        ExecutionAction::Release
    });
    let hooks = Arc::new(TestHooks::default());
    let id = mode_job(&mut conn, W_EFFECT, "x").async_job_id;
    let worker = start_hooked(
        &[W_EFFECT],
        1,
        execute,
        still_required(),
        |t| t.grace = Duration::from_millis(200),
        Arc::clone(&hooks),
    );
    // T4 commits; before the context sees the answer, the signal comes and
    // the grace phase ends, so the fallback finds the tracked phase PRE_WRITE.
    let handle = worker.handle.clone();
    let observer_url = test_db_url();
    hooks.push_faults(
        Op::EffectStart,
        vec![Fault::pass_then(move || {
            handle.request_shutdown();
            let mut observer = PgConnection::establish(&observer_url).unwrap();
            wait_for(|| status(&mut observer, id) == "RECONCILIATION_REQUIRED");
        })],
    );
    assert!(ready.wait());
    let shared = Arc::clone(worker.shared());
    let mut worker = worker;
    let code = worker.wait_code();
    worker.join_threads();
    assert_eq!(code, 0);
    let row = job(&mut conn, id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    let attempt = last_execution(&mut conn, id);
    assert_eq!(attempt.error_code.as_deref(), Some("WORKER_SHUTDOWN"));
    assert!(logger()
        .containing(&format!("job {id} "))
        .iter()
        .any(|line| line.contains("SHUTDOWN_FALLBACK WORKER_SHUTDOWN")));
    assert_eq!(
        noted(&results),
        vec!["Err(ShutdownInProgress) EffectStarted".to_string()]
    );
    assert_eq!(
        shared.counters.shutdown_fallbacks.load(Ordering::Relaxed),
        1
    );
}

/// Carry OPW4-1: after a committed T4, `mark_effect_started` answers
/// `ShutdownInProgress` (tracked `EFFECT_STARTED`); the handler's `Release`,
/// refused with `PHASE` while the signal is set, gets `fail_on`
/// `Indeterminate` `WORKER_SHUTDOWN`, counted as a shutdown fallback.
#[test]
fn w14_opw4_1_release_after_shutdown_in_progress_is_a_shutdown_fallback() {
    let (_guard, mut conn) = setup();
    let results = notes();
    let results_h = Arc::clone(&results);
    let execute = exec_fn(move |_job, ctx| {
        let start = ctx.mark_effect_started();
        note(&results_h, format!("{start:?} {:?}", ctx.tracked_phase()));
        ExecutionAction::Release
    });
    let hooks = Arc::new(TestHooks::default());
    let id = mode_job(&mut conn, W_EFFECT, "x").async_job_id;
    let worker = start_hooked(
        &[W_EFFECT],
        1,
        execute,
        still_required(),
        no_tweak,
        Arc::clone(&hooks),
    );
    let handle = worker.handle.clone();
    hooks.push_faults(
        Op::EffectStart,
        vec![Fault::pass_then(move || handle.request_shutdown())],
    );
    let shared = Arc::clone(worker.shared());
    let mut worker = worker;
    let code = worker.wait_code();
    worker.join_threads();
    assert_eq!(code, 0);
    assert_eq!(
        noted(&results),
        vec!["Err(ShutdownInProgress) EffectStarted".to_string()]
    );
    assert_eq!(status(&mut conn, id), "RECONCILIATION_REQUIRED");
    let attempt = last_execution(&mut conn, id);
    assert_eq!(attempt.error_code.as_deref(), Some("WORKER_SHUTDOWN"));
    assert_eq!(
        shared.counters.shutdown_fallbacks.load(Ordering::Relaxed),
        1
    );
    assert_eq!(shared.counters.action_fallbacks.load(Ordering::Relaxed), 0);
}

// ---------------------------------------------------------------------------
// W15: the unrun rule
// ---------------------------------------------------------------------------

#[test]
fn w15_unrun_rule_after_the_signal() {
    let (_guard, mut conn) = setup();
    // (5) A pre-write job that waited once, then was retried: checkpoint and
    // waiting deadline kept, the deadline moved into the past.
    let retried = mode_job(&mut conn, W_EFFECT, "retried").async_job_id;
    let claim = worker_claim(&engine_claim(&mut conn, W_EFFECT));
    engine_wait(&mut conn, &claim);
    make_available(&mut conn, retried);
    let claim = worker_claim(&engine_claim(&mut conn, W_EFFECT));
    engine_fail(&mut conn, &claim, FailureClass::RetryablePreWrite);
    assert_eq!(status(&mut conn, retried), "RETRY_SCHEDULED");
    set_waiting_deadline(&mut conn, retried, -60);
    // (2) Pre-write WAITING with a checkpoint.
    let pre_waiting = to_waiting(&mut conn, W_EFFECT, "w", AsyncEffectPhase::PreWrite);
    // (3) Resumed post-effect WAITING with a checkpoint.
    let post_waiting = to_waiting(
        &mut conn,
        W_RESUMABLE,
        "w",
        AsyncEffectPhase::EffectConfirmed,
    );
    // (6) Resumable EFFECT_CONFIRMED whose deadline passes after the claim.
    let confirmed_late = to_waiting(
        &mut conn,
        W_RESUMABLE,
        "w",
        AsyncEffectPhase::EffectConfirmed,
    );
    // (4) A reconciliation claim (another kind, claimed by a second runtime).
    let reconciliation = to_rr(&mut conn, W_CAPABILITY, "rr");
    // (1) A plain pre-write job.
    let plain_pre = mode_job(&mut conn, W_EFFECT, "plain").async_job_id;
    for id in [retried, pre_waiting, post_waiting, confirmed_late] {
        make_available(&mut conn, id);
    }
    let before: HashMap<Uuid, AsyncJob> = [
        retried,
        pre_waiting,
        post_waiting,
        confirmed_late,
        reconciliation,
        plain_pre,
    ]
    .iter()
    .map(|id| (*id, job(&mut conn, *id)))
    .collect();

    let calls = Arc::new(AtomicUsize::new(0));
    let (calls_e, calls_r) = (Arc::clone(&calls), Arc::clone(&calls));
    let execute = exec_fn(move |job, _| {
        calls_e.fetch_add(1, Ordering::SeqCst);
        complete(job)
    });
    let reconcile = rec_fn(move |_, _| {
        calls_r.fetch_add(1, Ordering::SeqCst);
        ReconciliationAction::still_required()
    });
    let hooks = Arc::new(TestHooks::default());
    hooks.claim_gate.arm();
    let worker = start_hooked(
        &[W_EFFECT, W_RESUMABLE],
        8,
        Arc::clone(&execute),
        Arc::clone(&reconcile),
        no_tweak,
        Arc::clone(&hooks),
    );
    assert!(hooks.claim_gate.wait_reached());
    set_waiting_deadline(&mut conn, confirmed_late, -1);
    worker.handle.request_shutdown();
    hooks.claim_gate.release();
    let shared = Arc::clone(worker.shared());
    assert_eq!(worker.stop(), 0);
    assert_eq!(shared.counters.unrun.load(Ordering::Relaxed), 5);

    let after = |conn: &mut PgConnection, id: Uuid| job(conn, id);
    let row = after(&mut conn, plain_pre);
    assert_eq!(row.status, AsyncJobStatus::Pending);
    assert_eq!(
        row.execution_claim_count,
        before[&plain_pre].execution_claim_count + 1
    );
    assert_eq!(row.prewrite_failure_count, 0);
    for id in [pre_waiting, post_waiting] {
        let row = after(&mut conn, id);
        assert_eq!(row.status, AsyncJobStatus::Waiting, "{id}");
        assert_eq!(row.checkpoint, before[&id].checkpoint);
        assert_eq!(
            row.execution_claim_count, before[&id].execution_claim_count,
            "a claim from WAITING spends no execution claim"
        );
    }
    let row = after(&mut conn, retried);
    assert_eq!(
        row.status,
        AsyncJobStatus::Pending,
        "released, never DEADLINE_EXCEEDED"
    );
    assert_eq!(
        row.execution_claim_count,
        before[&retried].execution_claim_count + 1
    );
    let row = after(&mut conn, confirmed_late);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(
        last_execution(&mut conn, confirmed_late)
            .error_code
            .as_deref(),
        Some("WORKER_SHUTDOWN")
    );
    assert_eq!(status(&mut conn, reconciliation), "RECONCILIATION_REQUIRED");

    // The reconciliation claim, unrun: StillRequired.
    let hooks = Arc::new(TestHooks::default());
    hooks.claim_gate.arm();
    let worker = start_hooked(
        &[W_CAPABILITY],
        2,
        execute,
        reconcile,
        no_tweak,
        Arc::clone(&hooks),
    );
    assert!(hooks.claim_gate.wait_reached());
    worker.handle.request_shutdown();
    hooks.claim_gate.release();
    assert_eq!(worker.stop(), 0);
    let row = job(&mut conn, reconciliation);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert!(row.claimed_by.is_none());
    assert_eq!(
        row.reconciliation_attempt_count,
        before[&reconciliation].reconciliation_attempt_count + 1
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the handler is never called"
    );
}

// ---------------------------------------------------------------------------
// W16: abrupt termination; W16b: connection loss
// ---------------------------------------------------------------------------

#[test]
fn w16_abrupt_termination_is_recovered_by_a_second_runtime() {
    let (_guard, mut conn) = setup();
    let confirmed = to_waiting(
        &mut conn,
        W_RESUMABLE,
        "block",
        AsyncEffectPhase::EffectConfirmed,
    );
    make_available(&mut conn, confirmed);
    let running = Arc::new(AtomicUsize::new(0));
    let release = Flag::new();
    let (running_h, release_h) = (Arc::clone(&running), Arc::clone(&release));
    let execute = exec_fn(move |job, ctx| {
        if mode_of(job) == "started" && ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        running_h.fetch_add(1, Ordering::SeqCst);
        release_h.wait();
        complete(job)
    });
    let pre = mode_job(&mut conn, W_EFFECT, "pre").async_job_id;
    let started = mode_job(&mut conn, W_EFFECT, "started").async_job_id;
    let worker = start(
        &[W_EFFECT, W_RESUMABLE],
        3,
        execute,
        still_required(),
        no_tweak,
    );
    assert!(wait_for(|| running.load(Ordering::SeqCst) == 3));
    assert_eq!(worker.hard_stop_then(|| release.set()), 1);
    for id in [pre, started, confirmed] {
        assert_eq!(
            status(&mut conn, id),
            "RUNNING",
            "no final write, no fallback"
        );
        expire_lease(&mut conn, id);
    }

    let reconcile = rec_fn(|job, _ctx| ReconciliationAction::succeeded(applied_entries(job), None));
    let second = start(
        &[W_EFFECT, W_RESUMABLE],
        3,
        effectful(),
        reconcile,
        no_tweak,
    );
    assert!(wait_for(
        || status(&mut conn, pre) == "RETRY_SCHEDULED" || status(&mut conn, pre) == "SUCCEEDED"
    ));
    make_available(&mut conn, pre);
    assert!(wait_status(&mut conn, pre, "SUCCEEDED"));
    assert!(wait_status(&mut conn, started, "SUCCEEDED"));
    assert!(attempts(&mut conn, started)
        .iter()
        .any(|a| a.purpose == AsyncAttemptPurpose::Reconciliation));
    assert!(wait_for(
        || status(&mut conn, confirmed) == "WAITING" || status(&mut conn, confirmed) == "SUCCEEDED"
    ));
    make_available(&mut conn, confirmed);
    assert!(wait_status(&mut conn, confirmed, "SUCCEEDED"));
    assert!(job_transitions(&mut conn, confirmed)
        .unwrap()
        .iter()
        .any(|t| t.cause == "T12_LEASE_EXPIRED" && t.to_status == AsyncJobStatus::Waiting));
    assert_eq!(second.stop(), 0);
}

#[test]
fn w16b_connection_loss_is_survived() {
    let (_guard, mut conn) = setup();
    let ready = Flag::new();
    let go = Flag::new();
    let results = notes();
    let flags = (Arc::clone(&ready), Arc::clone(&go), Arc::clone(&results));
    let execute = exec_fn(move |job, ctx| {
        let (ready, go, results) = &flags;
        ready.set();
        // Blocked behind the test's row lock while the backends die.
        let read = ctx.read_target_state().map(|_| ());
        note(results, format!("{read:?}"));
        go.wait();
        run_effect(job, ctx)
    });
    let id = mode_job(&mut conn, W_EFFECT, "x").async_job_id;
    let config = config(&[W_EFFECT], 1, &url_with_application("wtest_w16b"));
    let mut timings = fast_timings(&config);
    timings.renew_interval = Some(Duration::from_millis(200));
    let worker = TestWorker::start(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, execute, still_required()),
        timings,
    );
    assert!(ready.wait());
    conn.batch_execute("BEGIN").unwrap();
    exec(
        &mut conn,
        &format!("SELECT 1 FROM public.async_job WHERE async_job_id = '{id}' FOR UPDATE"),
    );
    let mut observer = connect();
    assert!(wait_for(|| scalar(
        &mut observer,
        "SELECT count(*) AS value FROM pg_stat_activity \
          WHERE application_name = 'wtest_w16b' AND wait_event_type = 'Lock'"
    ) > 0));
    let terminated = scalar(
        &mut observer,
        "SELECT count(*) AS value FROM (SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
          WHERE application_name = 'wtest_w16b') AS killed",
    );
    assert!(terminated >= 2, "the engine and heartbeat connections");
    conn.batch_execute("ROLLBACK").unwrap();
    assert!(wait_for(|| noted(&results).len() == 1));
    assert_eq!(noted(&results), vec!["Err(Busy)".to_string()]);
    // The heartbeat pool replaces its connection and renewals go on.
    let heartbeat = last_execution(&mut conn, id).last_heartbeat_at;
    assert!(wait_for(
        || last_execution(&mut conn, id).last_heartbeat_at > heartbeat
    ));
    let heartbeat = last_execution(&mut conn, id).last_heartbeat_at;
    assert!(wait_for(
        || last_execution(&mut conn, id).last_heartbeat_at > heartbeat
    ));
    go.set();
    assert!(wait_status(&mut conn, id, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
}

// ---------------------------------------------------------------------------
// W17: two runtimes
// ---------------------------------------------------------------------------

#[test]
fn w17_two_runtimes_share_the_queue_exactly_once() {
    let (_guard, mut conn) = setup();
    let ids: Vec<Uuid> = (0..40)
        .map(|_| create_kind(&mut conn, W_LIMITED).async_job_id)
        .collect();
    let executions: Arc<Mutex<HashMap<Uuid, usize>>> = Arc::default();
    let max_running = Arc::new(AtomicUsize::new(0));
    let (executions_h, max_h) = (Arc::clone(&executions), Arc::clone(&max_running));
    let execute = exec_fn(move |job, ctx| {
        *executions_h
            .lock()
            .unwrap()
            .entry(job.async_job_id)
            .or_default() += 1;
        if let Ok(mut handler_conn) = ctx.pool().get() {
            let running = scalar(
                &mut handler_conn,
                &format!(
                    "SELECT count(*) AS value FROM public.async_job \
                      WHERE kind = '{W_LIMITED}' AND status = 'RUNNING'"
                ),
            );
            max_h.fetch_max(usize::try_from(running).unwrap_or(0), Ordering::SeqCst);
        }
        thread::sleep(Duration::from_millis(20));
        complete(job)
    });
    let first = start(
        &[W_LIMITED],
        4,
        Arc::clone(&execute),
        still_required(),
        no_tweak,
    );
    let second = start(&[W_LIMITED], 4, execute, still_required(), no_tweak);
    assert!(wait_for(|| scalar(
        &mut conn,
        &format!(
            "SELECT count(*) AS value FROM public.async_job \
              WHERE kind = '{W_LIMITED}' AND status = 'SUCCEEDED'"
        )
    ) == 40));
    let (a, b) = (Arc::clone(first.shared()), Arc::clone(second.shared()));
    assert_eq!(first.stop(), 0);
    assert_eq!(second.stop(), 0);
    let executions = executions.lock().unwrap();
    assert_eq!(executions.len(), 40);
    assert!(executions.values().all(|count| *count == 1));
    assert!(
        max_running.load(Ordering::SeqCst) <= 3,
        "concurrency_limit 3"
    );
    assert!(max_running.load(Ordering::SeqCst) >= 1);
    for shared in [a, b] {
        assert_eq!(shared.counters.lost_claims.load(Ordering::Relaxed), 0);
    }
    for id in ids {
        assert_eq!(execution_attempts(&mut conn, id).len(), 1);
        assert!(logger()
            .containing(&format!("job {id} "))
            .iter()
            .all(|line| !line.contains("ASYNC_JOB_STALE_CLAIM")));
    }
}

// ---------------------------------------------------------------------------
// W18: mixed versions
// ---------------------------------------------------------------------------

#[test]
fn w18_mixed_versions_older_runtime_leaves_newer_work() {
    let (_guard, mut conn) = setup();
    let x1 = create_kind(&mut conn, W_X).async_job_id;
    let x2 = create(
        &mut conn,
        NewAsyncJob {
            payload_version: 2,
            ..new_job(W_X, "x-two", &[])
        },
    )
    .async_job_id;
    let y = create_kind(&mut conn, W_Y).async_job_id;
    let y_expired = create_kind(&mut conn, W_Y).async_job_id;
    let claimed = tx(&mut conn, |c| {
        claim_on(
            c,
            &REGISTRY,
            &setup_executor(),
            ClaimRequest {
                kinds: vec![(W_Y.to_string(), vec![1])],
                max_jobs: 2,
                lease_seconds: None,
            },
        )
    })
    .unwrap();
    // Leave `y` unclaimed: put the job not under test back by recovery.
    for job in &claimed.jobs {
        expire_lease(&mut conn, job.async_job_id);
    }
    let y_ids: HashSet<Uuid> = claimed.jobs.iter().map(|j| j.async_job_id).collect();
    assert!(y_ids.contains(&y_expired) || y_ids.contains(&y));

    let older_config = config(&[W_X, W_Y], 2, &test_db_url());
    let older_timings = fast_timings(&older_config);
    let older = TestWorker::start(
        older_config,
        &OLDER_REGISTRY,
        handlers_for(&OLDER_REGISTRY, effectful(), still_required()),
        older_timings,
    );
    assert!(wait_status(&mut conn, x1, "SUCCEEDED"));
    let ticks = older.hooks.calls(Op::Claim);
    assert!(wait_for(|| older.hooks.calls(Op::Claim) >= ticks + 10));
    assert_eq!(status(&mut conn, x2), "PENDING");
    for id in &y_ids {
        assert_eq!(
            status(&mut conn, *id),
            "RUNNING",
            "never recovered by the older"
        );
    }
    assert!(!logger()
        .containing(&format!("kind {W_Y} left out"))
        .is_empty());
    assert_eq!(older.stop(), 0);

    let newer = start(&[W_X, W_Y], 2, effectful(), still_required(), no_tweak);
    assert!(wait_status(&mut conn, x2, "SUCCEEDED"));
    for id in [y, y_expired] {
        assert!(wait_for(|| {
            let now = status(&mut conn, id);
            if now == "RETRY_SCHEDULED" {
                make_available(&mut conn, id);
            }
            now == "SUCCEEDED"
        }));
    }
    assert_eq!(newer.stop(), 0);
}

// ---------------------------------------------------------------------------
// W19: reconciliation
// ---------------------------------------------------------------------------

/// Compile-time exhaustive: a sixth outcome fails to compile here.
fn outcome_name(outcome: ReconcilerOutcome) -> &'static str {
    match outcome {
        ReconcilerOutcome::Succeeded => "SUCCEEDED",
        ReconcilerOutcome::Failed => "FAILED",
        ReconcilerOutcome::Cancelled => "CANCELLED",
        ReconcilerOutcome::Waiting => "WAITING",
        ReconcilerOutcome::StillRequired => "STILL_REQUIRED",
    }
}

#[test]
fn w19_reconciler_outcomes_are_exactly_five() {
    let names: HashSet<&str> = [
        ReconcilerOutcome::Succeeded,
        ReconcilerOutcome::Failed,
        ReconcilerOutcome::Cancelled,
        ReconcilerOutcome::Waiting,
        ReconcilerOutcome::StillRequired,
    ]
    .into_iter()
    .map(outcome_name)
    .collect();
    assert_eq!(names.len(), 5);
}

fn test_attempt(
    purpose: AsyncAttemptPurpose,
    phase_at_start: AsyncEffectPhase,
    started: bool,
    finished_ago: Option<i64>,
) -> AsyncJobAttempt {
    let now = Utc::now();
    AsyncJobAttempt {
        async_job_attempt_id: Uuid::new_v4(),
        async_job_id: Uuid::new_v4(),
        attempt_number: 1,
        purpose,
        claimed_by: "w".into(),
        executor_principal: "p".into(),
        control_command_id: None,
        started_at: now,
        last_heartbeat_at: now,
        phase_at_start,
        effect_started_at: started.then_some(now),
        effect_confirmed_at: None,
        finished_at: finished_ago.map(|ago| now - chrono::Duration::seconds(ago)),
        result: None,
        job_status_after: None,
        outcome_code: None,
        error_code: None,
        error_detail: None,
        provider_ref: None,
        evidence: None,
        redacted_at: None,
        redaction_command_id: None,
    }
}

#[test]
fn w19_effect_settled_at_is_none_without_an_effect_start() {
    let clock = Utc::now();
    let pre = test_attempt(
        AsyncAttemptPurpose::Execution,
        AsyncEffectPhase::PreWrite,
        false,
        Some(5),
    );
    let reconciliation = test_attempt(
        AsyncAttemptPurpose::Reconciliation,
        AsyncEffectPhase::EffectStarted,
        false,
        Some(1),
    );
    assert_eq!(
        settled_at(&[pre.clone(), reconciliation], clock, &EFFECT),
        None
    );
    let started = test_attempt(
        AsyncAttemptPurpose::Execution,
        AsyncEffectPhase::PreWrite,
        true,
        Some(10),
    );
    let resumed = test_attempt(
        AsyncAttemptPurpose::Execution,
        AsyncEffectPhase::EffectStarted,
        false,
        None,
    );
    assert_eq!(
        settled_at(&[pre.clone(), started.clone()], clock, &EFFECT),
        Some(started.finished_at.unwrap() + chrono::Duration::seconds(30))
    );
    assert_eq!(
        settled_at(&[pre, started, resumed], clock, &EFFECT),
        Some(clock + chrono::Duration::seconds(30)),
        "an open attempt counts at the clock"
    );
}

#[test]
fn w19_reconciliation_outcomes_and_effect_settled_at() {
    let (_guard, mut conn) = setup();
    let succeeded = to_rr(&mut conn, W_EFFECT, "succeeded");
    let still = to_rr(&mut conn, W_EFFECT, "still");
    let waiting = to_rr(&mut conn, W_EFFECT, "waiting");
    let capped = to_rr(&mut conn, W_EFFECT, "waiting-capped");
    set_waiting_deadline(&mut conn, capped, 100);
    let failed = to_rr(&mut conn, W_EFFECT, "failed");
    let cancelled = to_rr(&mut conn, W_EFFECT, "cancelled");
    // A resumed attempt: claimed at EFFECT_STARTED, confirm_claim, then
    // Fail Indeterminate.
    let resumed = to_waiting(
        &mut conn,
        W_EFFECT,
        "resumed",
        AsyncEffectPhase::EffectStarted,
    );
    make_available(&mut conn, resumed);
    let claim = worker_claim(&engine_claim(&mut conn, W_EFFECT));
    tx(&mut conn, |c| renew_on(c, &REGISTRY, &claim, Some(60))).unwrap();
    engine_fail(&mut conn, &claim, FailureClass::Indeterminate);

    let settled: Arc<Mutex<HashMap<Uuid, Option<chrono::DateTime<Utc>>>>> = Arc::default();
    let settled_h = Arc::clone(&settled);
    let reconcile = rec_fn(move |job, ctx| {
        settled_h
            .lock()
            .unwrap()
            .insert(job.async_job_id, ctx.effect_settled_at());
        let not_applied = Some(json!({ "valid": true }).to_string());
        match mode_of(job).as_str() {
            "succeeded" | "resumed" => ReconciliationAction::succeeded(applied_entries(job), None),
            "waiting" => {
                ReconciliationAction::waiting(json!({ "recheck": 1 }).to_string(), Duration::ZERO)
            }
            "waiting-capped" => ReconciliationAction::waiting(
                json!({ "recheck": 1 }).to_string(),
                Duration::from_secs(7200),
            ),
            "failed" => ReconciliationAction::failed(not_applied),
            "cancelled" => ReconciliationAction::cancelled(not_applied),
            _ => ReconciliationAction::still_required(),
        }
    });
    let worker = start(&[W_EFFECT], 4, effectful(), reconcile, no_tweak);
    for (id, expected) in [
        (succeeded, "SUCCEEDED"),
        (waiting, "WAITING"),
        (capped, "WAITING"),
        (failed, "FAILED"),
        (cancelled, "CANCELLED"),
        (resumed, "SUCCEEDED"),
    ] {
        assert!(wait_status(&mut conn, id, expected), "{id} {expected}");
    }
    assert!(wait_for(|| job(&mut conn, still)
        .reconciliation_attempt_count
        == 1
        && job(&mut conn, still).claimed_by.is_none()));
    assert_eq!(worker.stop(), 0);
    assert_eq!(status(&mut conn, still), "RECONCILIATION_REQUIRED");

    // Waiting: resume_at from the database clock, at least poll_min + margin.
    let row = job(&mut conn, waiting);
    let closed = attempts(&mut conn, waiting)
        .pop()
        .unwrap()
        .finished_at
        .unwrap();
    // The clock was read in the same transaction, just before the attempt
    // closed.
    let delay = (row.available_at - closed).num_milliseconds();
    assert!((14_000..=15_000).contains(&delay), "{delay} ms");
    // Carry FBW4-1: capped at the waiting deadline.
    let row = job(&mut conn, capped);
    assert_eq!(Some(row.available_at), row.waiting_deadline);

    let settled = settled.lock().unwrap();
    let window = chrono::Duration::seconds(30);
    let t4 = execution_attempts(&mut conn, succeeded);
    assert_eq!(
        settled[&succeeded],
        Some(t4[0].finished_at.unwrap() + window)
    );
    let resumed_attempts = execution_attempts(&mut conn, resumed);
    assert_eq!(resumed_attempts.len(), 2);
    assert_eq!(
        resumed_attempts[1].phase_at_start,
        AsyncEffectPhase::EffectStarted
    );
    assert_eq!(
        settled[&resumed],
        Some(resumed_attempts[1].finished_at.unwrap() + window)
    );
}

// ---------------------------------------------------------------------------
// W20: panics, invalid actions and faults
// ---------------------------------------------------------------------------

#[test]
fn w20_panics_and_invalid_actions() {
    let (_guard, mut conn) = setup();
    let reconcile_panic = to_rr(&mut conn, W_CAPABILITY, "panic");
    let execute = exec_fn(|job, ctx| {
        let mode = mode_of(job);
        if mode.ends_with("-post") && ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        match mode.as_str() {
            "panic-pre" | "panic-post" => panic!("handler panic in a test"),
            "release-post" => ExecutionAction::Release,
            "bad-code-pre" => fail(FailureClass::RetryablePreWrite, "not a code!"),
            "beyond-horizon-pre" => ExecutionAction::Wait(WaitAction {
                checkpoint: json!({ "step": 1 }).to_string(),
                resume_after: Duration::from_secs(3601 + 60),
                provider_ref: None,
            }),
            _ => complete(job),
        }
    });
    let reconcile = rec_fn(|job, _| {
        if mode_of(job) == "panic" {
            panic!("reconciler panic in a test");
        }
        ReconciliationAction::still_required()
    });
    let modes = [
        ("panic-pre", "FAILED", "HANDLER_PANIC"),
        ("panic-post", "RECONCILIATION_REQUIRED", "HANDLER_PANIC"),
        (
            "release-post",
            "RECONCILIATION_REQUIRED",
            "HANDLER_ACTION_INVALID",
        ),
        ("bad-code-pre", "FAILED", "HANDLER_ACTION_INVALID"),
        // Carry FBW4-2: a first wait past the waiting horizon is a handler
        // fault.
        ("beyond-horizon-pre", "FAILED", "HANDLER_ACTION_INVALID"),
    ];
    let ids: Vec<(Uuid, &str, &str)> = modes
        .iter()
        .map(|(mode, status, code)| {
            (
                mode_job(&mut conn, W_EFFECT, mode).async_job_id,
                *status,
                *code,
            )
        })
        .collect();
    let worker = start(&[W_EFFECT, W_CAPABILITY], 6, execute, reconcile, no_tweak);
    for (id, expected, code) in &ids {
        assert!(
            wait_for(|| status(&mut conn, *id) == *expected),
            "{id} {expected}"
        );
        assert_eq!(
            last_execution(&mut conn, *id).error_code.as_deref(),
            Some(*code)
        );
    }
    assert!(wait_for(|| job(&mut conn, reconcile_panic)
        .reconciliation_attempt_count
        == 1
        && job(&mut conn, reconcile_panic).claimed_by.is_none()));
    let shared = Arc::clone(worker.shared());
    assert_eq!(worker.stop(), 0);
    assert_eq!(
        status(&mut conn, reconcile_panic),
        "RECONCILIATION_REQUIRED"
    );
    assert_eq!(
        scalar(
            &mut conn,
            "SELECT count(*) AS value FROM public.async_job WHERE status = 'RUNNING'"
        ),
        0,
        "no job is left RUNNING"
    );
    assert!(shared.counters.action_fallbacks.load(Ordering::Relaxed) >= 3);
}

#[test]
fn w20_fault_fallback_and_wait_resume_at() {
    let (_guard, mut conn) = setup();
    let execute = exec_fn(|job, ctx| {
        let mode = mode_of(job);
        if mode.ends_with("-post") && ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        let wait = |after: Duration| {
            ExecutionAction::Wait(WaitAction {
                checkpoint: json!({ "step": 1 }).to_string(),
                resume_after: after,
                provider_ref: None,
            })
        };
        match mode.as_str() {
            "wait" | "wait-too-close" => wait(Duration::ZERO),
            "wait-capped" => wait(Duration::from_secs(7200)),
            "complete-post" => {
                let _ = ctx.mark_effect_confirmed(None);
                complete(job)
            }
            _ => complete(job),
        }
    });
    // Two jobs waited once at pre-write through the engine, set up before
    // the runtime starts so it cannot claim them first; each is due later
    // with its own waiting deadline.
    let capped = to_waiting(
        &mut conn,
        W_EFFECT,
        "wait-capped",
        AsyncEffectPhase::PreWrite,
    );
    let close = to_waiting(
        &mut conn,
        W_EFFECT,
        "wait-too-close",
        AsyncEffectPhase::PreWrite,
    );
    let hooks = Arc::new(TestHooks::default());
    let worker = start_hooked(
        &[W_EFFECT, W_READ],
        1,
        execute,
        still_required(),
        no_tweak,
        Arc::clone(&hooks),
    );

    // The fault fallback: RETRY_SCHEDULED at pre-write, RR after T4.
    hooks.push_faults(Op::Complete, vec![Fault::integrity()]);
    let pre = mode_job(&mut conn, W_READ, "complete").async_job_id;
    assert!(wait_status(&mut conn, pre, "RETRY_SCHEDULED"));
    assert_eq!(
        last_execution(&mut conn, pre).error_code.as_deref(),
        Some("WORKER_ENGINE_FAULT")
    );
    hooks.push_faults(Op::Complete, vec![Fault::integrity()]);
    let post = mode_job(&mut conn, W_EFFECT, "complete-post").async_job_id;
    assert!(wait_status(&mut conn, post, "RECONCILIATION_REQUIRED"));
    assert_eq!(
        last_execution(&mut conn, post).error_code.as_deref(),
        Some("WORKER_ENGINE_FAULT")
    );
    for id in [pre, post] {
        assert!(logger()
            .containing(&format!("job {id} "))
            .iter()
            .all(|line| !line.contains("HANDLER_ACTION_INVALID")));
    }

    // A Wait busy twice (no commit) commits on the third try.
    hooks.push_faults(Op::Wait, vec![Fault::busy(), Fault::busy()]);
    let calls = hooks.calls(Op::Wait);
    let waited = mode_job(&mut conn, W_EFFECT, "wait").async_job_id;
    assert!(wait_status(&mut conn, waited, "WAITING"));
    assert_eq!(hooks.calls(Op::Wait), calls + 3);
    let row = job(&mut conn, waited);
    let closed = last_execution(&mut conn, waited).finished_at.unwrap();
    // resume_at = clock + max(0, poll_interval_min_seconds 5 + 10); the
    // attempt closed in the same transaction, just after the clock read.
    let delay = (row.available_at - closed).num_milliseconds();
    assert!((14_000..=15_000).contains(&delay), "{delay} ms");
    assert!(row.available_at <= row.waiting_deadline.unwrap());

    // resume_after past the waiting deadline is capped to it.
    due_later(&mut conn, capped, 100);
    let row = job(&mut conn, capped);
    assert_eq!(row.status, AsyncJobStatus::Waiting);
    assert_eq!(Some(row.available_at), row.waiting_deadline);

    // A deadline closer than poll_min + margin: WAIT_DEADLINE_TOO_CLOSE.
    let shared = Arc::clone(worker.shared());
    let failures = || shared.counters.failures.load(Ordering::Relaxed);
    assert_eq!(failures(), 0, "no handler Fail in this test");
    due_later(&mut conn, close, 8);
    assert_eq!(status(&mut conn, close), "RETRY_SCHEDULED");
    assert_eq!(
        last_execution(&mut conn, close).error_code.as_deref(),
        Some("WAIT_DEADLINE_TOO_CLOSE")
    );
    assert!(
        wait_for(|| failures() >= 1),
        "a wait turned into fail_on is counted as a failure"
    );
    assert_eq!(worker.stop(), 0);
    // The two fault fallbacks have their own counter.
    assert_eq!(shared.counters.fault_fallbacks.load(Ordering::Relaxed), 2);
    assert_eq!(shared.counters.action_fallbacks.load(Ordering::Relaxed), 0);
}

/// Make a job waited once through the engine due now, with its waiting
/// deadline `deadline_seconds` from now; returns once the running worker
/// settled it.
fn due_later(conn: &mut PgConnection, id: Uuid, deadline_seconds: i64) {
    set_waiting_deadline(conn, id, deadline_seconds);
    make_available(conn, id);
    let settled = |conn: &mut PgConnection| {
        let row = job(conn, id);
        row.status != AsyncJobStatus::Running
            && (row.status != AsyncJobStatus::Waiting || row.checkpoint_version >= 2)
    };
    assert!(wait_for(|| settled(conn)), "{id}");
}

#[test]
fn w20_runtime_thread_panic_exits_1_and_withholds_liveness() {
    let (_guard, mut conn) = setup();
    // A handler held in flight keeps the runtime in its grace phase after
    // the panic, so the ticker runs with the panic recorded.
    let entered = Flag::new();
    let release = Flag::new();
    let flags = (Arc::clone(&entered), Arc::clone(&release));
    let execute = exec_fn(move |job, _ctx| {
        flags.0.set();
        flags.1.wait();
        complete(job)
    });
    let hooks = Arc::new(TestHooks::default());
    let mut worker = start_hooked(
        &[W_READ],
        1,
        execute,
        still_required(),
        no_tweak,
        Arc::clone(&hooks),
    );
    let id = mode_job(&mut conn, W_READ, "held").async_job_id;
    assert!(entered.wait());
    let in_flight_lines = || logger().containing("claim call was still in flight").len();
    let before = in_flight_lines();
    hooks.inject_claim_loop_panic();
    assert!(wait_for(|| hooks
        .liveness()
        .contains(&Some("RUNTIME_THREAD_PANICKED"))));
    release.set();
    assert_eq!(worker.wait_code(), 1);
    worker.join_threads();
    assert_eq!(
        status(&mut conn, id),
        "SUCCEEDED",
        "the held handler finished"
    );
    assert_eq!(
        in_flight_lines(),
        before,
        "a panicked claim loop counts as exited: no wait for the join bound"
    );
}

#[test]
fn w20_panic_line_has_the_location_only() {
    let location = std::panic::Location::caller();
    let line = panic_line(Some(location));
    assert_eq!(
        line,
        format!(
            "async worker: panic at {}:{}",
            location.file(),
            location.line()
        )
    );
    assert_eq!(
        panic_line(None),
        "async worker: panic at an unknown location"
    );
}

// ---------------------------------------------------------------------------
// W21: no secrets in logs
// ---------------------------------------------------------------------------

#[test]
fn w21_no_secret_reaches_a_log_record() {
    const SENTINEL: &str = "W21SECRETSENTINEL";
    let (_guard, mut conn) = setup();
    let release = Flag::new();
    let release_h = Arc::clone(&release);
    let execute = exec_fn(move |job, ctx| {
        let mode = mode_of(job);
        if mode.ends_with("-post") && ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        let secret = |name: &str| format!("{name}-{SENTINEL}");
        match mode.as_str() {
            "fail-pre" => ExecutionAction::Fail(FailRequest {
                class: FailureClass::RetryablePreWrite,
                error_code: "PROVIDER_UNAVAILABLE".into(),
                error_detail: Some(secret("detail")),
                no_effect_proven: false,
                evidence: None,
            }),
            "bad-fail-pre" => ExecutionAction::Fail(FailRequest {
                class: FailureClass::Permanent,
                error_code: secret("code"),
                error_detail: Some(secret("detail")),
                no_effect_proven: false,
                evidence: None,
            }),
            "wait-post" => {
                let _ = ctx.mark_effect_confirmed(Some(&secret("provider")));
                ExecutionAction::Wait(WaitAction {
                    checkpoint: json!({ "note": secret("checkpoint") }).to_string(),
                    resume_after: Duration::ZERO,
                    provider_ref: Some(secret("provider")),
                })
            }
            "evidence-post" => {
                let _ = ctx.mark_effect_confirmed(None);
                let mut request = complete_request(&EFFECT, job);
                request.evidence =
                    Some(json!({ "valid": false, "s": secret("evidence") }).to_string());
                ExecutionAction::Complete(request)
            }
            "panic-pre" => panic!("{}", secret("panic")),
            "block-post" => {
                release_h.wait();
                complete(job)
            }
            _ => complete(job),
        }
    });
    let secret_job = |conn: &mut PgConnection, kind: &str, mode: &str| {
        let target = key(REGISTRY.get(kind).unwrap().target_namespaces[0]);
        let target = format!("{target}-{}", SENTINEL.to_lowercase());
        create(
            conn,
            NewAsyncJob {
                payload: json!({
                    "id": Uuid::new_v4().to_string(),
                    "targets": [target],
                    "mode": mode,
                    "note": SENTINEL,
                })
                .to_string(),
                ..new_job(kind, "unused", &[])
            },
        )
        .async_job_id
    };
    let modes = [
        "complete",
        "fail-pre",
        "bad-fail-pre",
        "wait-post",
        "evidence-post",
        "panic-pre",
        "block-post",
    ];
    let ids: Vec<Uuid> = modes
        .iter()
        .map(|mode| secret_job(&mut conn, W_EFFECT, mode))
        .collect();
    let worker = start(&[W_EFFECT], 8, execute, still_required(), |t| {
        t.grace = Duration::from_millis(300);
    });
    assert!(wait_for(|| ids
        .iter()
        .filter(|id| status(&mut conn, **id) != "RUNNING" && status(&mut conn, **id) != "PENDING")
        .count()
        >= modes.len() - 1));
    assert_eq!(worker.stop_then(|| release.set()), 0);

    // A hard stop and the recovery of its job.
    let blocked = secret_job(&mut conn, W_EFFECT, "block-post");
    let release2 = Flag::new();
    let release2_h = Arc::clone(&release2);
    let execute = exec_fn(move |job, ctx| {
        let _ = ctx.mark_effect_started();
        release2_h.wait();
        complete(job)
    });
    let worker = start(&[W_EFFECT], 1, execute, still_required(), no_tweak);
    assert!(wait_for(
        || job(&mut conn, blocked).effect_phase == AsyncEffectPhase::EffectStarted
    ));
    assert_eq!(worker.hard_stop_then(|| release2.set()), 1);

    // Startup with the six AWS names mapped to sentinels: refused without
    // the values.
    let all: HashMap<&'static str, &'static str> = AWS_CREDENTIAL_VARIABLES
        .iter()
        .map(|name| (*name, SENTINEL))
        .collect();
    assert!(check_aws_sources(lookup(all))
        .unwrap_err()
        .iter()
        .all(|problem| !problem.contains(SENTINEL)));

    // An unreachable database whose password is the sentinel.
    let url = format!("postgres://thoth:{SENTINEL}@127.0.0.1:1/thoth");
    let config = config(&[W_EFFECT], 1, &url);
    let timings = fast_timings(&config);
    let worker = TestWorker::start(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, effectful(), still_required()),
        timings,
    );
    assert!(wait_for(|| !logger()
        .containing("DATABASE_UNAVAILABLE")
        .is_empty()));
    assert_eq!(worker.stop(), 0);

    let leaked = logger().containing(SENTINEL);
    let leaked_lower = logger().containing(&SENTINEL.to_lowercase());
    assert!(leaked.is_empty(), "{leaked:?}");
    assert!(leaked_lower.is_empty(), "{leaked_lower:?}");
    assert!(!logger().records().is_empty());
}

#[test]
fn w21_source_guard_never_reads_the_whole_environment() {
    let needle_vars = ["env", "::", "vars"].concat();
    let sources = [
        include_str!("mod.rs"),
        include_str!("config.rs"),
        include_str!("handler.rs"),
        include_str!("runtime.rs"),
        include_str!("runtime/claims.rs"),
        include_str!("runtime/heartbeat.rs"),
        include_str!("runtime/slots.rs"),
        include_str!("health.rs"),
        include_str!("errors.rs"),
        include_str!("test_support.rs"),
        include_str!("tests.rs"),
    ];
    for source in sources {
        // The needle also matches the `_os` variant.
        assert!(!source.contains(&needle_vars));
    }
}

// ---------------------------------------------------------------------------
// W22: busy and unknown outcomes
// ---------------------------------------------------------------------------

#[test]
fn w22a_row_lock_past_the_lock_timeout_and_busy_claims() {
    let (_guard, mut conn) = setup();
    let ready = Flag::new();
    let go = Flag::new();
    let flags = (Arc::clone(&ready), Arc::clone(&go));
    let execute = exec_fn(move |job, _| {
        if mode_of(job) == "locked" {
            flags.0.set();
            flags.1.wait();
        }
        complete(job)
    });
    let hooks = Arc::new(TestHooks::default());
    hooks.push_faults(Op::Claim, vec![Fault::busy(), Fault::busy()]);
    let first = mode_job(&mut conn, W_READ, "plain").async_job_id;
    let worker = start_hooked(
        &[W_READ],
        1,
        execute,
        still_required(),
        |t| {
            t.busy_retries = vec![Duration::from_millis(50); 3];
        },
        Arc::clone(&hooks),
    );
    assert!(wait_status(&mut conn, first, "SUCCEEDED"));
    assert!(worker.shared().counters.busy.load(Ordering::Relaxed) >= 2);

    let locked = mode_job(&mut conn, W_READ, "locked").async_job_id;
    assert!(ready.wait());
    let before = hooks.calls(Op::Complete);
    conn.batch_execute("BEGIN").unwrap();
    exec(
        &mut conn,
        &format!("SELECT 1 FROM public.async_job WHERE async_job_id = '{locked}' FOR UPDATE"),
    );
    go.set();
    // The first try waits out the 5 s lock timeout and answers busy.
    assert!(wait_for(|| hooks.calls(Op::Complete) >= before + 2));
    conn.batch_execute("ROLLBACK").unwrap();
    assert!(wait_status(&mut conn, locked, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
}

/// The T3 + T4 and T5 cases of W22 (b) to (e), one job at a time.
#[test]
fn w22_effect_marks_with_unknown_outcomes() {
    let (_guard, mut conn) = setup();
    let results = notes();
    let results_h = Arc::clone(&results);
    let execute = exec_fn(move |job, ctx| {
        let mode = mode_of(job);
        if let Some(t5) = mode.strip_prefix("t5-") {
            let start = ctx.mark_effect_started();
            let confirm = ctx.mark_effect_confirmed(None);
            note(
                &results_h,
                format!("{mode} {start:?} {confirm:?} {:?}", ctx.tracked_phase()),
            );
            let _ = t5;
            return if confirm.is_ok() {
                complete(job)
            } else {
                fail(FailureClass::Indeterminate, "CONFIRM_UNKNOWN")
            };
        }
        let start = ctx.mark_effect_started();
        note(
            &results_h,
            format!("{mode} {start:?} {:?}", ctx.tracked_phase()),
        );
        match start {
            Ok(()) => {
                let _ = ctx.mark_effect_confirmed(None);
                complete(job)
            }
            Err(_) => fail(FailureClass::Indeterminate, "FENCE_UNKNOWN"),
        }
    });
    let hooks = Arc::new(TestHooks::default());
    let worker = start_hooked(
        &[W_EFFECT],
        1,
        execute,
        still_required(),
        |t| {
            t.busy_retries = vec![Duration::from_millis(20); 3];
        },
        Arc::clone(&hooks),
    );
    let mut run = |mode: &str, op: Op, faults: Vec<Fault>, expected: &str| -> Uuid {
        hooks.push_faults(op, faults);
        let id = mode_job(&mut conn, W_EFFECT, mode).async_job_id;
        assert!(wait_status(&mut conn, id, expected), "{mode}");
        id
    };
    let four = |f: fn() -> Fault| (0..4).map(|_| f()).collect::<Vec<_>>();

    let extra = hooks.calls(Op::ExtraRenew);
    let b = run("b", Op::EffectStart, vec![Fault::busy()], "SUCCEEDED");
    assert_eq!(hooks.calls(Op::ExtraRenew), extra);
    let c = run(
        "c",
        Op::EffectStart,
        vec![Fault::busy_after_commit()],
        "SUCCEEDED",
    );
    assert_eq!(hooks.calls(Op::ExtraRenew), extra + 1, "the extra renewal");
    let d = run(
        "d",
        Op::EffectStart,
        four(Fault::busy_after_commit),
        "RECONCILIATION_REQUIRED",
    );
    let e = run("e", Op::EffectStart, four(Fault::busy), "RETRY_SCHEDULED");
    let t5b = run("t5-b", Op::EffectConfirm, vec![Fault::busy()], "SUCCEEDED");
    let t5c = run(
        "t5-c",
        Op::EffectConfirm,
        vec![Fault::busy_after_commit()],
        "SUCCEEDED",
    );
    assert_eq!(
        hooks.calls(Op::ExtraRenew),
        extra + 1,
        "no extra renewal after T5"
    );
    let t5d = run(
        "t5-d",
        Op::EffectConfirm,
        four(Fault::busy_after_commit),
        "RECONCILIATION_REQUIRED",
    );
    // A retry after an unknown try that answers neither busy nor a proven
    // outcome: the earlier try may have committed.
    let g = run(
        "g",
        Op::EffectStart,
        vec![Fault::busy_after_commit(), Fault::integrity()],
        "RECONCILIATION_REQUIRED",
    );
    // The extra renewal after PHASE leaves less than the window before the
    // runtime deadline.
    let shared = Arc::clone(worker.shared());
    hooks.push_faults(
        Op::ExtraRenew,
        vec![Fault::pass_then(move || {
            shared.set_remaining(Duration::from_secs(10))
        })],
    );
    let h = run(
        "h",
        Op::EffectStart,
        vec![Fault::busy_after_commit()],
        "RECONCILIATION_REQUIRED",
    );
    assert_eq!(worker.stop(), 0);

    assert_eq!(
        noted(&results),
        vec![
            "b Ok(()) EffectStarted",
            "c Ok(()) EffectStarted",
            "d Err(Busy) PossiblyStarted",
            "e Err(Busy) PossiblyStarted",
            "t5-b Ok(()) Ok(()) EffectConfirmed",
            "t5-c Ok(()) Ok(()) EffectConfirmed",
            "t5-d Ok(()) Err(Busy) EffectStarted",
            "g Err(Failed(\"ASYNC_JOB_INTEGRITY_VIOLATION\")) PossiblyStarted",
            "h Err(EffectWindowInsufficient) EffectStarted",
        ]
    );
    for id in [b, c] {
        let t4 = job_transitions(&mut conn, id).unwrap();
        let _ = t4;
        assert_eq!(execution_attempts(&mut conn, id).len(), 1);
        assert!(execution_attempts(&mut conn, id)[0]
            .effect_started_at
            .is_some());
    }
    assert!(
        last_execution(&mut conn, d).effect_started_at.is_some(),
        "T4 committed"
    );
    assert!(
        last_execution(&mut conn, e).effect_started_at.is_none(),
        "no write"
    );
    for id in [t5b, t5c] {
        assert!(last_execution(&mut conn, id).effect_confirmed_at.is_some());
    }
    assert!(last_execution(&mut conn, t5d).effect_confirmed_at.is_some());
    for id in [g, h] {
        assert!(
            last_execution(&mut conn, id).effect_started_at.is_some(),
            "T4 committed on the first try"
        );
    }
}

/// W22 (f): the stop signal during the retries, and during a successful T4.
#[test]
fn w22f_signal_during_effect_mark_never_answers_ok() {
    let (_guard, mut conn) = setup();
    for (case, expected_phase, expected_status) in [
        ("during-retries", "PossiblyStarted", "RETRY_SCHEDULED"),
        ("during-t4", "EffectStarted", "RECONCILIATION_REQUIRED"),
    ] {
        let results = notes();
        let results_h = Arc::clone(&results);
        let execute = exec_fn(move |_job, ctx| {
            let start = ctx.mark_effect_started();
            note(&results_h, format!("{start:?} {:?}", ctx.tracked_phase()));
            fail(FailureClass::Indeterminate, "FENCE_REFUSED")
        });
        let hooks = Arc::new(TestHooks::default());
        let mut worker = start_hooked(
            &[W_EFFECT],
            1,
            execute,
            still_required(),
            no_tweak,
            Arc::clone(&hooks),
        );
        let handle = worker.handle.clone();
        let fault = if case == "during-retries" {
            Fault::busy_then(move || handle.request_shutdown())
        } else {
            Fault::pass_then(move || handle.request_shutdown())
        };
        // The fault is in place before the job exists.
        hooks.push_faults(Op::EffectStart, vec![fault]);
        let id = mode_job(&mut conn, W_EFFECT, case).async_job_id;
        assert_eq!(worker.wait_code(), 0);
        worker.join_threads();
        assert_eq!(
            noted(&results),
            vec![format!("Err(ShutdownInProgress) {expected_phase}")],
            "{case}"
        );
        assert_eq!(status(&mut conn, id), expected_status, "{case}");
    }
}

/// W22: a post-effect `confirm_claim` that returns with less than the
/// window left answers `EffectWindowInsufficient`.
#[test]
fn w22_post_effect_confirm_claim_returning_short_of_the_window() {
    let (_guard, mut conn) = setup();
    let results = notes();
    let results_h = Arc::clone(&results);
    let ready = Flag::new();
    let go = Flag::new();
    let flags = (Arc::clone(&ready), Arc::clone(&go));
    let execute = exec_fn(move |_job, ctx| {
        if ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        flags.0.set();
        flags.1.wait();
        let confirm = ctx.confirm_claim();
        note(&results_h, format!("{confirm:?}"));
        fail(FailureClass::Indeterminate, "WINDOW_INSUFFICIENT")
    });
    let hooks = Arc::new(TestHooks::default());
    let id = mode_job(&mut conn, W_EFFECT, "x").async_job_id;
    let worker = start_hooked(
        &[W_EFFECT],
        1,
        execute,
        still_required(),
        no_tweak,
        Arc::clone(&hooks),
    );
    assert!(ready.wait());
    let shared = Arc::clone(worker.shared());
    hooks.push_faults(
        Op::ConfirmClaim,
        vec![Fault::pass_then(move || {
            shared.set_remaining(Duration::from_secs(10))
        })],
    );
    go.set();
    assert!(wait_status(&mut conn, id, "RECONCILIATION_REQUIRED"));
    assert_eq!(worker.stop(), 0);
    assert_eq!(
        noted(&results),
        vec!["Err(EffectWindowInsufficient)".to_string()]
    );
}

// ---------------------------------------------------------------------------
// W23: pending migrations
// ---------------------------------------------------------------------------

#[test]
fn w23_pending_migrations_comparison() {
    let embedded = embedded_migrations().expect("the embedded migrations");
    assert!(!embedded.is_empty());
    assert!(
        migrations_pending(&embedded, None),
        "a missing table is pending"
    );
    let mut missing_one = embedded.clone();
    missing_one.pop();
    assert!(migrations_pending(&embedded, Some(&missing_one)));
    let mut ahead = embedded.clone();
    ahead.push("99991231".to_string());
    assert!(
        !migrations_pending(&embedded, Some(&ahead)),
        "a newer database is not pending"
    );
}

#[test]
fn w23_pending_migrations_read_only_check() {
    let (_guard, mut conn) = setup();
    let embedded = embedded_migrations().unwrap();
    let applied = applied_migrations(&mut conn)
        .expect("the check")
        .expect("the table exists");
    assert!(!migrations_pending(&embedded, Some(&applied)));
    let refused = in_read_only(&mut conn, |c| {
        sql_query("CREATE TABLE public.wtest_read_only (id integer)").execute(c)
    });
    assert!(refused.is_err(), "DDL fails in the READ ONLY transaction");
    assert_eq!(
        scalar(
            &mut conn,
            "SELECT count(*) AS value FROM pg_tables WHERE tablename = 'wtest_read_only'"
        ),
        0
    );
}

// ---------------------------------------------------------------------------
// W24: liveness
// ---------------------------------------------------------------------------

#[test]
fn w24_health_function() {
    let path = liveness_path();
    let now = super::health::unix_now();
    assert!(!is_healthy(&path, 60, now), "missing");
    write_liveness(&path, now).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), liveness_text(now));
    assert!(is_healthy(&path, 60, now), "fresh");
    assert!(is_healthy(&path, 60, now + 60));
    assert!(!is_healthy(&path, 60, now + 61), "stale");
    assert!(
        !is_healthy(&path, 60, now - 1),
        "a time later than now (the clock stepped back)"
    );
    std::fs::write(&path, "not a number").unwrap();
    assert!(!is_healthy(&path, 60, now), "unparseable");
    std::fs::remove_file(&path).unwrap();
    assert!(parse_health_args("/tmp/x", "15").is_ok());
    assert!(
        parse_health_args("/tmp/x", "14").is_err(),
        "at least 3 x LIVENESS_CADENCE"
    );
    assert!(parse_health_args("/tmp/x", "601").is_err());
    assert!(parse_health_args("relative", "60").is_err());
}

#[test]
fn w24_withhold_rules() {
    let limits = LivenessLimits {
        loop_stall: Duration::from_secs(120),
        heartbeat_stall: Duration::from_secs(30),
        hung_slot_limit: Duration::from_secs(300),
    };
    let healthy = LivenessInputs {
        loop_age: Duration::from_secs(1),
        heartbeat_age: Some(Duration::from_secs(1)),
        in_flight: 1,
        hung: vec![],
        slots: 2,
        panicked: false,
    };
    assert_eq!(withhold_reason(&healthy, &limits), None);
    let case = |f: &dyn Fn(&mut LivenessInputs)| {
        let mut inputs = healthy.clone();
        f(&mut inputs);
        withhold_reason(&inputs, &limits)
    };
    assert_eq!(
        case(&|i| i.loop_age = Duration::from_secs(121)),
        Some("LOOP_STALLED")
    );
    assert_eq!(
        case(&|i| i.heartbeat_age = Some(Duration::from_secs(31))),
        Some("HEARTBEAT_STALLED")
    );
    assert_eq!(
        case(&|i| {
            i.heartbeat_age = Some(Duration::from_secs(31));
            i.in_flight = 0;
        }),
        None,
        "a stalled heartbeat with no claim in flight"
    );
    assert_eq!(
        case(&|i| i.hung = vec![Duration::from_secs(301)]),
        Some("SLOT_HUNG_TOO_LONG")
    );
    assert_eq!(
        case(&|i| i.hung = vec![Duration::from_secs(1), Duration::from_secs(1)]),
        Some("ALL_SLOTS_HUNG")
    );
    assert_eq!(
        case(&|i| i.panicked = true),
        Some("RUNTIME_THREAD_PANICKED")
    );
}

#[test]
fn w24_liveness_while_waiting_for_the_database() {
    logger();
    let config = config(&[W_EFFECT], 1, "postgres://thoth:x@127.0.0.1:1/thoth");
    let timings = fast_timings(&config);
    let worker = TestWorker::start(
        config,
        &REGISTRY,
        handlers_for(&REGISTRY, effectful(), still_required()),
        timings,
    );
    let file = worker.liveness_file();
    let hooks = Arc::clone(&worker.hooks);
    assert!(wait_for(|| hooks.liveness().len() >= 20));
    assert!(is_healthy(&file, 15, super::health::unix_now()));
    assert!(hooks.liveness().iter().all(Option::is_none));
    assert_eq!(worker.stop(), 0);
}

#[test]
fn w24_liveness_during_migration_wait_and_busy_renewals() {
    let (_guard, mut conn) = setup();
    // Migrations pending: nothing claimed, liveness fresh.
    let hooks = Arc::new(TestHooks::default());
    hooks.set_pending_migrations(true);
    let first = create_kind(&mut conn, W_READ).async_job_id;
    let worker = start_hooked(
        &[W_READ],
        1,
        effectful(),
        still_required(),
        no_tweak,
        Arc::clone(&hooks),
    );
    assert!(wait_for(|| hooks.liveness().len() >= 10));
    assert!(is_healthy(
        &worker.liveness_file(),
        15,
        super::health::unix_now()
    ));
    assert!(!logger().containing("MIGRATIONS_PENDING").is_empty());
    assert_eq!(status(&mut conn, first), "PENDING");
    hooks.set_pending_migrations(false);
    assert!(wait_status(&mut conn, first, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);

    // Every renewal busy: renewal health goes stale, claims stop, liveness
    // stays fresh.
    let release = Flag::new();
    let release_h = Arc::clone(&release);
    let execute = exec_fn(move |job, _| {
        if mode_of(job) == "block" {
            release_h.wait();
        }
        complete(job)
    });
    let hooks = Arc::new(TestHooks::default());
    hooks.push_faults(Op::Renew, (0..100_000).map(|_| Fault::busy()).collect());
    let blocked = mode_job(&mut conn, W_READ, "block").async_job_id;
    let worker = start_hooked(
        &[W_READ],
        2,
        execute,
        still_required(),
        |t| {
            t.renew_interval = Some(Duration::from_millis(50));
            t.heartbeat_stall = Duration::from_millis(500);
        },
        Arc::clone(&hooks),
    );
    assert!(wait_for(|| status(&mut conn, blocked) == "RUNNING"));
    let shared = Arc::clone(worker.shared());
    assert!(wait_for(|| shared
        .counters
        .stalled_ticks
        .load(Ordering::Relaxed)
        >= 3));
    let waiting = create_kind(&mut conn, W_READ).async_job_id;
    let ticks = shared.counters.stalled_ticks.load(Ordering::Relaxed);
    assert!(wait_for(|| shared
        .counters
        .stalled_ticks
        .load(Ordering::Relaxed)
        >= ticks + 5));
    assert_eq!(
        status(&mut conn, waiting),
        "PENDING",
        "the claim loop claims nothing"
    );
    assert!(is_healthy(
        &worker.liveness_file(),
        15,
        super::health::unix_now()
    ));
    assert_eq!(hooks.liveness().last(), Some(&None));
    assert!(shared.summary_for_test().contains("renewal health"));
    release.set();
    assert!(wait_status(&mut conn, blocked, "SUCCEEDED"));
    assert!(wait_status(&mut conn, waiting, "SUCCEEDED"));
    assert_eq!(worker.stop(), 0);
}

// ---------------------------------------------------------------------------
// W25: no slot free, and a job re-claimed while its old handler runs
// ---------------------------------------------------------------------------

#[test]
fn w25_recover_when_no_slot_is_free() {
    let (_guard, mut conn) = setup();
    let release = Flag::new();
    let release_h = Arc::clone(&release);
    let execute = exec_fn(move |job, _| {
        release_h.wait();
        complete(job)
    });
    let busy = create_kind(&mut conn, W_READ).async_job_id;
    let worker = start(&[W_READ, W_EFFECT], 1, execute, still_required(), no_tweak);
    assert!(wait_for(|| status(&mut conn, busy) == "RUNNING"));
    assert!(wait_for(|| worker.shared().idle_slot_count() == 0));
    // A peer's job past its lease.
    let peer = create_kind(&mut conn, W_EFFECT).async_job_id;
    engine_claim(&mut conn, W_EFFECT);
    expire_lease(&mut conn, peer);
    let recovers = worker.hooks.calls(Op::Recover);
    assert!(wait_status(&mut conn, peer, "RETRY_SCHEDULED"));
    assert!(worker.hooks.calls(Op::Recover) > recovers);
    assert_eq!(status(&mut conn, busy), "RUNNING");
    assert_eq!(worker.stop_then(|| release.set()), 0);
}

#[test]
fn w25_reclaimed_job_with_its_old_handler_still_running() {
    let (_guard, mut conn) = setup();
    let old_ready = Flag::new();
    let old_release = Flag::new();
    let new_ready = Flag::new();
    let new_release = Flag::new();
    let results = notes();
    let flags = (
        Arc::clone(&old_ready),
        Arc::clone(&old_release),
        Arc::clone(&new_ready),
        Arc::clone(&new_release),
        Arc::clone(&results),
    );
    let execute = exec_fn(move |job, ctx| {
        let (old_ready, old_release, new_ready, new_release, results) = &flags;
        let mode = mode_of(job);
        if job.attempt_number == 1 {
            if mode == "post" && ctx.mark_effect_started().is_err() {
                return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
            }
            old_ready.set();
            old_release.wait();
            let fence = ctx.mark_effect_started();
            note(results, format!("old {mode} {fence:?}"));
            return complete(job);
        }
        new_ready.set();
        new_release.wait();
        run_effect(job, ctx)
    });
    let reconcile_calls = Arc::new(AtomicUsize::new(0));
    let reconcile_calls_h = Arc::clone(&reconcile_calls);
    let reconcile = rec_fn(move |_, _| {
        reconcile_calls_h.fetch_add(1, Ordering::SeqCst);
        ReconciliationAction::still_required()
    });

    // The old claim at PRE_WRITE: the new claim runs.
    let pre = mode_job(&mut conn, W_EFFECT, "pre").async_job_id;
    let worker = start(
        &[W_EFFECT],
        2,
        Arc::clone(&execute),
        Arc::clone(&reconcile),
        |t| {
            t.renew_interval = Some(Duration::from_millis(100));
        },
    );
    assert!(old_ready.wait());
    expire_lease(&mut conn, pre);
    assert!(wait_for(|| status(&mut conn, pre) == "RETRY_SCHEDULED"));
    make_available(&mut conn, pre);
    assert!(new_ready.wait());
    let shared = Arc::clone(worker.shared());
    let tracked: Vec<Arc<ClaimState>> = shared
        .claims_snapshot()
        .into_iter()
        .filter(|state| state.job_id == pre)
        .collect();
    assert_eq!(tracked.len(), 2, "both claims tracked separately");
    assert_eq!(shared.idle_slot_count(), 0, "slot accounting stays exact");
    let heartbeat = last_execution(&mut conn, pre).last_heartbeat_at;
    assert!(
        wait_for(|| last_execution(&mut conn, pre).last_heartbeat_at > heartbeat),
        "the new claim is renewed"
    );
    old_release.set();
    assert!(wait_for(|| noted(&results).len() == 1));
    let old = &noted(&results)[0];
    assert!(
        old == "old pre Err(ClaimLost)" || old == "old pre Err(LeaseExpired)",
        "{old}"
    );
    new_release.set();
    assert!(wait_status(&mut conn, pre, "SUCCEEDED"));
    assert!(wait_for(|| shared.idle_slot_count() == 2));
    assert_eq!(worker.stop(), 0);

    // The old claim after T4: the new reconciliation claim is not run.
    let (old_ready, old_release) = (Flag::new(), Flag::new());
    let results = notes();
    let flags = (
        Arc::clone(&old_ready),
        Arc::clone(&old_release),
        Arc::clone(&results),
    );
    let execute = exec_fn(move |job, ctx| {
        let (old_ready, old_release, results) = &flags;
        if ctx.mark_effect_started().is_err() {
            return fail(FailureClass::Indeterminate, "FENCE_REFUSED");
        }
        old_ready.set();
        old_release.wait();
        note(
            results,
            format!("old {:?}", ctx.claim_lost() || ctx.lease_expired()),
        );
        complete(job)
    });
    let post = mode_job(&mut conn, W_EFFECT, "post").async_job_id;
    let worker = start(&[W_EFFECT], 2, execute, reconcile, |t| {
        t.renew_interval = Some(Duration::from_millis(100));
    });
    assert!(old_ready.wait());
    expire_lease(&mut conn, post);
    assert!(wait_for(|| job(&mut conn, post)
        .reconciliation_attempt_count
        == 1
        && job(&mut conn, post).claimed_by.is_none()));
    let shared = Arc::clone(worker.shared());
    assert!(wait_for(
        || shared.counters.unrun.load(Ordering::Relaxed) == 1
    ));
    assert_eq!(status(&mut conn, post), "RECONCILIATION_REQUIRED");
    assert_eq!(reconcile_calls.load(Ordering::SeqCst), 0);
    assert_eq!(shared.idle_slot_count(), 1);
    old_release.set();
    assert!(wait_for(|| noted(&results).len() == 1));
    assert!(wait_for(|| shared.idle_slot_count() == 2));
    assert_eq!(worker.stop(), 0);
    assert_eq!(job(&mut conn, post).reconciliation_attempt_count, 1);
}

#[allow(dead_code)]
fn unused_helpers(_: &HandlerRegistry, _: Purpose, _: ResumePlan) {}

#[test]
fn resume_plan_and_deadline_guard() {
    let clock = Utc::now();
    let s = chrono::Duration::seconds;
    assert_eq!(
        plan_resume(clock, None, Duration::ZERO, &EFFECT),
        ResumePlan::At(clock + s(15))
    );
    assert_eq!(
        plan_resume(clock, None, Duration::from_secs(100), &EFFECT),
        ResumePlan::At(clock + s(100))
    );
    assert_eq!(
        plan_resume(
            clock,
            Some(clock + s(50)),
            Duration::from_secs(100),
            &EFFECT
        ),
        ResumePlan::At(clock + s(50))
    );
    assert_eq!(
        plan_resume(clock, Some(clock + s(14)), Duration::ZERO, &EFFECT),
        ResumePlan::TooClose
    );
    assert_eq!(
        plan_resume(clock, Some(clock), Duration::ZERO, &EFFECT),
        ResumePlan::DeadlinePassed
    );
    assert!(deadline_guard(clock, None, &EFFECT));
    assert!(deadline_guard(clock, Some(clock + s(16)), &EFFECT));
    assert!(!deadline_guard(clock, Some(clock + s(15)), &EFFECT));
    assert_eq!(effect_lease_request(&EFFECT), 60);
    assert_eq!(effect_lease_request(&LONG), 100);
}
