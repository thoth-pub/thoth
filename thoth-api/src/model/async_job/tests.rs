//! The E1 tests of spec section 19.2 (THOTH-ASYNC-01-ENGINE-01A).
//!
//! Real PostgreSQL through the existing test-database pattern, deterministic
//! and never sleeping: concurrency tests use barriers, transactions held open
//! by a controlling thread and bounded `pg_locks` polling, and every one of
//! them asserts an unchanged `pg_stat_database.deadlocks` counter. Time moves
//! through fixtures that rewrite a job's lease, availability or deadline.
//! Test names start with their 19.2 group: `schema_`, `creation_`, `claims_`,
//! `leases_`, `effect_`, `targets_`, `commands_` (cancellation, retry and
//! reconciliation), `redaction_`, `recovery_`, `signals_`, `backstop_`,
//! `time_`, `bounds_`, `phase_`, `regression_` (accepted regression), `locks_`
//! (commands and locks), `s9_` (section 9 additions), `mapping_` (error
//! mapping, which with the deadlock assertions covers deadlock freedom), plus
//! `registry_` and `carry_` (carry-forward part A).

use std::sync::mpsc;
use std::thread;
use std::time::{Duration as StdDuration, Instant};

use chrono::Duration;
use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde_json::json;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::test_support::*;
use super::*;
use crate::model::tests::db::{setup_test_db, TestDbGuard};

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

fn setup() -> (TestDbGuard, PgConnection) {
    let (guard, _pool) = setup_test_db();
    (guard, connect())
}

const NO_EFFECT_EVIDENCE: &str = r#"{"valid": true, "providerRef": "absent-1"}"#;
const NOT_APPLIED_EVIDENCE: &str = r#"{"notApplied": true}"#;

fn fail_req(class: FailureClass) -> FailRequest {
    FailRequest {
        class,
        error_code: "TEST_FAILURE".to_string(),
        error_detail: None,
        no_effect_proven: false,
        evidence: None,
    }
}

fn no_effect_req(class: FailureClass, evidence: Option<&str>) -> FailRequest {
    FailRequest {
        no_effect_proven: true,
        evidence: evidence.map(str::to_string),
        ..fail_req(class)
    }
}

fn cs_entry(target: &str, fingerprint: &str) -> EvidenceEntry {
    EvidenceEntry {
        target_key: target.to_string(),
        order: None,
        fingerprint: Some(fingerprint.to_string()),
        source_ref: None,
    }
}

fn rb_entry(target: &str, order: i64, fingerprint: &str) -> EvidenceEntry {
    EvidenceEntry {
        target_key: target.to_string(),
        order: Some(i128::from(order)),
        fingerprint: Some(fingerprint.to_string()),
        source_ref: None,
    }
}

fn completion(outcome: CompletionOutcome, entries: Vec<EvidenceEntry>) -> CompleteRequest {
    CompleteRequest {
        outcome,
        entries,
        evidence: None,
    }
}

fn applied_evidence(order: i64) -> String {
    json!({ "valid": true, "order": order }).to_string()
}

fn resolution(outcome: ReconcilerOutcome) -> ResolveReconciliation {
    ResolveReconciliation {
        outcome,
        entries: None,
        evidence: None,
        checkpoint: None,
        resume_at: None,
    }
}

fn staff_res(outcome: StaffOutcome) -> StaffResolution {
    StaffResolution {
        outcome,
        entries: None,
        evidence: None,
        checkpoint: None,
        resume_at: None,
    }
}

fn do_renew(conn: &mut PgConnection, claim: &WorkerClaim) -> ThothResult<LeaseRenewal> {
    tx(conn, |c| renew_on(c, &REGISTRY, claim, None))
}

fn do_start(conn: &mut PgConnection, claim: &WorkerClaim) -> ThothResult<AsyncJob> {
    tx(conn, |c| mark_effect_started_on(c, &REGISTRY, claim))
}

fn do_confirm(conn: &mut PgConnection, claim: &WorkerClaim) -> ThothResult<AsyncJob> {
    tx(conn, |c| {
        mark_effect_confirmed_on(c, &REGISTRY, claim, Some("provider-1"))
    })
}

fn do_complete(
    conn: &mut PgConnection,
    claim: &WorkerClaim,
    request: CompleteRequest,
) -> ThothResult<CompleteOutcome> {
    tx(conn, |c| complete_on(c, &REGISTRY, claim, request))
}

fn do_fail(
    conn: &mut PgConnection,
    claim: &WorkerClaim,
    request: FailRequest,
) -> ThothResult<AsyncJob> {
    tx(conn, |c| fail_on(c, &REGISTRY, claim, request))
}

fn do_wait(
    conn: &mut PgConnection,
    claim: &WorkerClaim,
    checkpoint: &str,
    resume_at: chrono::DateTime<chrono::Utc>,
) -> ThothResult<WaitOutcome> {
    let request = WaitRequest {
        checkpoint: checkpoint.to_string(),
        resume_at,
        provider_ref: None,
    };
    tx(conn, |c| wait_on(c, &REGISTRY, claim, request))
}

fn do_release(conn: &mut PgConnection, claim: &WorkerClaim) -> ThothResult<AsyncJob> {
    tx(conn, |c| release_on(c, &REGISTRY, claim))
}

fn do_read_state(conn: &mut PgConnection, claim: &WorkerClaim) -> ThothResult<Vec<TargetState>> {
    tx(conn, |c| read_target_state_on(c, &REGISTRY, claim))
}

fn do_resolve(
    conn: &mut PgConnection,
    claim: &WorkerClaim,
    request: ResolveReconciliation,
) -> ThothResult<AsyncJob> {
    tx(conn, |c| {
        resolve_reconciliation_on(c, &REGISTRY, claim, request)
    })
}

fn do_recover(conn: &mut PgConnection) -> RecoveryReport {
    tx(conn, |c| recover_on(c, &REGISTRY, None, 1000)).expect("recover")
}

fn do_cancel(conn: &mut PgConnection, id: Uuid) -> CommandResult {
    tx(conn, |c| {
        cancel_job_command_on(c, &REGISTRY, &staff("cancel"), id)
    })
    .expect("cancel")
}

fn do_retry(conn: &mut PgConnection, id: Uuid) -> CommandResult {
    tx(conn, |c| {
        retry_job_command_on(c, &REGISTRY, &staff("retry"), id)
    })
    .expect("retry")
}

fn do_staff(conn: &mut PgConnection, id: Uuid, resolution: StaffResolution) -> CommandResult {
    tx(conn, |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, &staff("resolve"), id, resolution)
    })
    .expect("resolve command")
}

fn do_redact(conn: &mut PgConnection, id: Uuid) -> CommandResult {
    tx(conn, |c| {
        redact_job_diagnostics_command_on(c, &staff("redact"), id)
    })
    .expect("redact")
}

#[track_caller]
fn assert_applied(result: &CommandResult, code: &str) {
    assert_eq!(
        result.outcome,
        AsyncControlCommandOutcome::Applied,
        "{result:?}"
    );
    assert_eq!(result.outcome_code, code, "{result:?}");
}

#[track_caller]
fn assert_command_refused(result: &CommandResult, code: &str) {
    assert_eq!(
        result.outcome,
        AsyncControlCommandOutcome::Refused,
        "{result:?}"
    );
    assert_eq!(result.outcome_code, code, "{result:?}");
}

fn status(conn: &mut PgConnection, id: Uuid) -> AsyncJobStatus {
    job(conn, id).status
}

/// The claim is live: a renewal with its token succeeds.
#[track_caller]
fn assert_live(conn: &mut PgConnection, claim: &WorkerClaim) {
    do_renew(conn, claim).expect("the claim must still be live");
}

/// A job at EFFECT_STARTED in RECONCILIATION_REQUIRED (T9), unclaimed.
fn rr_started(conn: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    let (job, claim) = started(conn, input);
    let failed = do_fail(conn, &claim, fail_req(FailureClass::Indeterminate)).expect("T9");
    assert_eq!(failed.status, AsyncJobStatus::ReconciliationRequired);
    job
}

/// A job at EFFECT_CONFIRMED in RECONCILIATION_REQUIRED (T9), unclaimed.
fn rr_confirmed(conn: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    let (job, claim) = confirmed(conn, input);
    let failed = do_fail(conn, &claim, fail_req(FailureClass::Indeterminate)).expect("T9");
    assert_eq!(failed.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(failed.effect_phase, AsyncEffectPhase::EffectConfirmed);
    job
}

/// A target whose applied state is `order`/`fingerprint`, written by a
/// SUCCEEDED `TEST_REVISION` job. Call it before creating other
/// `TEST_REVISION` jobs: its claim takes every ready job of the kind.
fn seed_rb_state(conn: &mut PgConnection, target: &str, order: i64, fingerprint: &str) -> Uuid {
    let id = format!("seed-{}", Uuid::new_v4());
    let (job, claim) = started(conn, revision_job(&id, target, order, fingerprint));
    let outcome = do_complete(
        conn,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(target, order, fingerprint)],
        ),
    )
    .expect("seed completion");
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    job.async_job_id
}

/// Spawn an engine call in its own engine-owned transaction on a fresh
/// connection.
fn spawn_tx<T: Send + 'static>(
    op: impl FnOnce(&mut PgConnection) -> ThothResult<T> + Send + 'static,
) -> (i32, thread::JoinHandle<ThothResult<T>>) {
    spawn_with_connection(move |conn| tx(conn, op))
}

/// The deadlock counter must not move during a concurrency test.
#[track_caller]
fn assert_no_deadlock(conn: &mut PgConnection, before: i64) {
    assert_eq!(deadlocks(conn), before, "a concurrency test met a deadlock");
}

fn attempt_token(conn: &mut PgConnection, attempt_id: Uuid) -> Option<String> {
    text(
        conn,
        &format!(
            "SELECT claim_token::text AS value FROM public.async_job_attempt \
              WHERE async_job_attempt_id = '{attempt_id}'"
        ),
    )
}

fn count(conn: &mut PgConnection, sql_from_where: &str) -> i64 {
    scalar(conn, &format!("SELECT count(*) AS value {sql_from_where}"))
}

// ---------------------------------------------------------------------------
// Registry (section 8): validation on every test registry, production empty
// ---------------------------------------------------------------------------

#[test]
fn registry_production_is_empty_and_valid() {
    assert!(PRODUCTION_REGISTRY.is_empty());
    assert_eq!(PRODUCTION_REGISTRY.validate(), Ok(()));
}

#[test]
fn registry_every_test_registry_validates() {
    for registry in test_registries() {
        assert_eq!(registry.validate(), Ok(()), "{registry:?}");
    }
}

static BAD_MULTI_TARGET_RB: AsyncJobRegistry = AsyncJobRegistry::new(&[AsyncJobKindPolicy {
    kind: "TEST_BAD_RB",
    max_targets: 2,
    ..REVISION_POLICY
}]);

static BAD_STAFF_OUTCOMES: AsyncJobRegistry = AsyncJobRegistry::new(&[
    AsyncJobKindPolicy {
        kind: "TEST_BAD_NO_FAILED",
        staff_reconciliation_outcomes: &[StaffOutcome::Succeeded],
        ..EFFECT_POLICY
    },
    AsyncJobKindPolicy {
        kind: "TEST_BAD_NO_ACCEPT",
        staff_reconciliation_outcomes: &[StaffOutcome::Succeeded, StaffOutcome::Failed],
        ..REVISION_POLICY
    },
    AsyncJobKindPolicy {
        kind: "TEST_BAD_ACCEPT",
        staff_reconciliation_outcomes: &[
            StaffOutcome::Succeeded,
            StaffOutcome::Failed,
            StaffOutcome::AcceptRegression,
        ],
        ..EFFECT_POLICY
    },
]);

static BAD_CROSS_KIND: AsyncJobRegistry = AsyncJobRegistry::new(&[
    EFFECT_POLICY,
    AsyncJobKindPolicy {
        kind: "TEST_OTHER_DOMAIN",
        owning_domain: "other_domain",
        ..EFFECT_POLICY
    },
    AsyncJobKindPolicy {
        kind: "TEST_OTHER_MODE",
        revision_mode: RevisionMode::RevisionBound,
        max_targets: 1,
        staff_reconciliation_outcomes: &[
            StaffOutcome::Succeeded,
            StaffOutcome::Failed,
            StaffOutcome::AcceptRegression,
        ],
        ..EFFECT_POLICY
    },
    EFFECT_POLICY,
]);

static BAD_BOUNDS: AsyncJobRegistry = AsyncJobRegistry::new(&[AsyncJobKindPolicy {
    kind: "bad",
    retry_budget: 21,
    max_execution_claims: 101,
    max_reconciliation_attempts: 51,
    backoff_base_seconds: 0,
    backoff_max_seconds: 86_401,
    poll_interval_min_seconds: 4,
    lease_min_seconds: 14,
    lease_default_seconds: 10,
    max_attempt_runtime_seconds: 86_401,
    effect_window_seconds: None,
    waiting_horizon_seconds: 30 * 86_400 + 1,
    concurrency_limit: 0,
    reconciliation_concurrency_limit: 0,
    max_targets: 33,
    creatable_versions: &[3],
    executable_versions: &[1],
    ..EFFECT_POLICY
}]);

#[test]
fn registry_refuses_a_multi_target_revision_bound_kind() {
    let problems = BAD_MULTI_TARGET_RB.validate().unwrap_err();
    assert!(
        problems
            .iter()
            .any(|p| p.contains("REVISION_BOUND kind has max_targets <= 1")),
        "{problems:?}"
    );
}

#[test]
fn registry_refuses_an_effectful_kind_without_the_required_staff_outcomes() {
    let problems = BAD_STAFF_OUTCOMES.validate().unwrap_err();
    let has = |kind: &str, text: &str| {
        problems
            .iter()
            .any(|p| p.starts_with(kind) && p.contains(text))
    };
    assert!(
        has("TEST_BAD_NO_FAILED", "must list SUCCEEDED and FAILED"),
        "{problems:?}"
    );
    assert!(
        has("TEST_BAD_NO_ACCEPT", "must list ACCEPT_REGRESSION"),
        "{problems:?}"
    );
    assert!(
        has("TEST_BAD_ACCEPT", "only an EFFECTFUL REVISION_BOUND kind"),
        "{problems:?}"
    );
}

#[test]
fn registry_refuses_cross_kind_namespace_and_duplicate_kind_conflicts() {
    let problems = BAD_CROSS_KIND.validate().unwrap_err();
    assert!(
        problems.iter().any(|p| p.contains("registered twice")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("TEST_OTHER_DOMAIN") && p.contains("is owned by")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("TEST_OTHER_MODE") && p.contains("has revision mode")),
        "{problems:?}"
    );
}

#[test]
fn registry_refuses_every_bound_of_section_8() {
    let problems = BAD_BOUNDS.validate().unwrap_err();
    for text in [
        "kind does not match",
        "retry_budget is outside",
        "max_execution_claims is outside",
        "max_reconciliation_attempts is outside",
        "backoff_base_seconds is below 1",
        "backoff_max_seconds is outside",
        "poll_interval_min_seconds is below 5",
        "lease_min_seconds is below 15",
        "lease_default_seconds is outside",
        "max_attempt_runtime_seconds exceeds 86400",
        "an EFFECTFUL kind needs effect_window_seconds",
        "waiting_horizon_seconds is outside",
        "concurrency_limit is below 1",
        "reconciliation_concurrency_limit is below 1",
        "max_targets exceeds 32",
        "not a superset of creatable_versions",
    ] {
        assert!(
            problems.iter().any(|p| p.contains(text)),
            "missing {text}: {problems:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Schema and triggers (E1)
// ---------------------------------------------------------------------------

/// Run `sql` in a transaction that is always rolled back. With `triggers`
/// false, ordinary triggers (and so foreign keys) are disabled and only
/// check, not-null and unique constraints apply. `Ok` when accepted,
/// `Err(constraint)` when refused.
fn probe(conn: &mut PgConnection, sql: &str, triggers: bool) -> Result<(), String> {
    let result = conn.transaction::<(), DieselError, _>(|c| {
        if !triggers {
            c.batch_execute("SET LOCAL session_replication_role = replica")?;
        }
        c.batch_execute(sql)?;
        Err(DieselError::RollbackTransaction)
    });
    match result {
        Err(DieselError::RollbackTransaction) => Ok(()),
        Err(DieselError::DatabaseError(_, info)) => Err(info
            .constraint_name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("<{}>", info.message()))),
        Err(other) => panic!("{sql}: {other}"),
        Ok(()) => unreachable!("the probe always rolls back"),
    }
}

/// An INSERT of `table` from `base` with `overrides` replacing or adding
/// columns (values are SQL expressions).
fn insert_sql<'a>(
    table: &str,
    base: &[(&'a str, &'a str)],
    overrides: &[(&'a str, &'a str)],
) -> String {
    let mut columns: Vec<(&str, &str)> = base.to_vec();
    for (column, value) in overrides {
        match columns.iter_mut().find(|entry| entry.0 == *column) {
            Some(entry) => entry.1 = *value,
            None => columns.push((*column, *value)),
        }
    }
    let names: Vec<&str> = columns.iter().map(|(c, _)| *c).collect();
    let values: Vec<&str> = columns.iter().map(|(_, v)| *v).collect();
    format!(
        "INSERT INTO public.{table} ({}) VALUES ({})",
        names.join(", "),
        values.join(", ")
    )
}

fn with<'a>(
    first: &[(&'a str, &'a str)],
    second: &[(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    first.iter().chain(second.iter()).copied().collect()
}

#[track_caller]
fn check_bounds(
    conn: &mut PgConnection,
    table: &str,
    base: &[(&str, &str)],
    constraint: &str,
    accepted: &[Vec<(&str, &str)>],
    refused: &[Vec<(&str, &str)>],
) {
    for overrides in accepted {
        let sql = insert_sql(table, base, overrides);
        assert_eq!(
            probe(conn, &sql, false),
            Ok(()),
            "{constraint} must accept {overrides:?}"
        );
    }
    for overrides in refused {
        let sql = insert_sql(table, base, overrides);
        assert_eq!(
            probe(conn, &sql, false),
            Err(constraint.to_string()),
            "{constraint} must refuse {overrides:?}"
        );
    }
}

const JOB_BASE: &[(&str, &str)] = &[
    ("kind", "'TEST_EFFECT'"),
    ("payload_version", "1"),
    ("payload", "'{\"id\": \"x\"}'"),
    ("idempotency_key", "'TEST_EFFECT:x'"),
    ("effect_class", "'EFFECTFUL'"),
    ("revision_mode", "'CURRENT_STATE'"),
    ("retry_budget", "3"),
    ("max_execution_claims", "4"),
    ("max_reconciliation_attempts", "2"),
    ("backoff_base_seconds", "10"),
    ("backoff_max_seconds", "300"),
    ("created_by", "'test'"),
];

const RUNNING_COLUMNS: &[(&str, &str)] = &[
    ("status", "'RUNNING'"),
    ("claim_token", "uuid_generate_v4()"),
    ("claimed_by", "'worker'"),
    ("executor_principal", "'principal'"),
    ("claimed_at", "now()"),
    ("lease_expires_at", "now()"),
    ("attempt_runtime_deadline", "now()"),
    ("attempt_count", "1"),
    ("execution_claim_count", "1"),
    ("first_claimed_at", "now()"),
];

const CLAIMED_ONCE: &[(&str, &str)] = &[
    ("attempt_count", "1"),
    ("execution_claim_count", "1"),
    ("first_claimed_at", "now()"),
];

#[test]
fn schema_async_job_checks_accept_their_boundary_and_refuse_one_past_it() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let t = "async_job";
    let b = JOB_BASE;
    check_bounds(
        c,
        t,
        b,
        "async_job_kind_check",
        &[
            vec![("kind", "'ABC'"), ("idempotency_key", "'ABC:x'")],
            vec![
                ("kind", "'A' || repeat('B', 63)"),
                ("idempotency_key", "'A' || repeat('B', 63) || ':x'"),
            ],
        ],
        &[
            vec![("kind", "'AB'"), ("idempotency_key", "'AB:x'")],
            vec![
                ("kind", "'A' || repeat('B', 64)"),
                ("idempotency_key", "'A' || repeat('B', 64) || ':x'"),
            ],
            vec![("kind", "'abc'"), ("idempotency_key", "'abc:x'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_payload_version_check",
        &[vec![("payload_version", "1")]],
        &[vec![("payload_version", "0")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_payload_check",
        &[vec![(
            "payload",
            "jsonb_build_object('a', repeat('x', 16375))",
        )]],
        &[
            vec![("payload", "jsonb_build_object('a', repeat('x', 16376))")],
            vec![("payload", "'[]'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_idempotency_key_check",
        &[vec![(
            "idempotency_key",
            "'TEST_EFFECT:' || repeat('x', 500)",
        )]],
        &[
            vec![("idempotency_key", "'TEST_EFFECT:' || repeat('x', 501)")],
            vec![("idempotency_key", "'OTHER:x'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_retry_budget_check",
        &[
            vec![("retry_budget", "1")],
            vec![("retry_budget", "20"), ("max_execution_claims", "20")],
        ],
        &[
            vec![("retry_budget", "0")],
            vec![("retry_budget", "21"), ("max_execution_claims", "21")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_max_execution_claims_check",
        &[
            vec![("max_execution_claims", "3")],
            vec![("max_execution_claims", "100")],
        ],
        &[
            vec![("max_execution_claims", "2")],
            vec![("max_execution_claims", "101")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_max_reconciliation_attempts_check",
        &[
            vec![("max_reconciliation_attempts", "1")],
            vec![("max_reconciliation_attempts", "50")],
        ],
        &[
            vec![("max_reconciliation_attempts", "0")],
            vec![("max_reconciliation_attempts", "51")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_backoff_base_seconds_check",
        &[vec![("backoff_base_seconds", "1")]],
        &[vec![("backoff_base_seconds", "0")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_backoff_max_seconds_check",
        &[
            vec![("backoff_max_seconds", "10")],
            vec![("backoff_max_seconds", "86400")],
        ],
        &[
            vec![("backoff_max_seconds", "9")],
            vec![("backoff_max_seconds", "86401")],
        ],
    );
    let rb: &[(&str, &str)] = &[("revision_mode", "'REVISION_BOUND'"), ("source_order", "1")];
    check_bounds(
        c,
        t,
        b,
        "async_job_source_fingerprint_check",
        &[
            with(rb, &[("source_fingerprint", "repeat('f', 256)")]),
            with(rb, &[("source_fingerprint", "'!~'")]),
        ],
        &[
            with(rb, &[("source_fingerprint", "repeat('f', 257)")]),
            with(rb, &[("source_fingerprint", "'a b'")]),
            with(rb, &[("source_fingerprint", "''")]),
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_source_ref_check",
        &[vec![("source_ref", "repeat('r', 256)")]],
        &[
            vec![("source_ref", "repeat('r', 257)")],
            vec![("source_ref", "'a b'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_priority_check",
        &[vec![("priority", "-100")], vec![("priority", "100")]],
        &[vec![("priority", "-101")], vec![("priority", "101")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_attempt_count_check",
        &[vec![("attempt_count", "0")]],
        &[vec![("attempt_count", "-1")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_execution_claim_count_check",
        &[
            vec![("execution_claim_count", "0")],
            vec![
                ("execution_claim_count", "4"),
                ("attempt_count", "4"),
                ("first_claimed_at", "now()"),
            ],
        ],
        &[
            vec![("execution_claim_count", "-1")],
            vec![
                ("execution_claim_count", "5"),
                ("attempt_count", "5"),
                ("first_claimed_at", "now()"),
            ],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_prewrite_failure_count_check",
        &[
            vec![("prewrite_failure_count", "0")],
            vec![("prewrite_failure_count", "3")],
        ],
        &[
            vec![("prewrite_failure_count", "-1")],
            vec![("prewrite_failure_count", "4")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_recon_attempt_count_check",
        &[vec![
            ("reconciliation_attempt_count", "2"),
            ("attempt_count", "2"),
            ("first_claimed_at", "now()"),
        ]],
        &[
            vec![("reconciliation_attempt_count", "-1")],
            vec![
                ("reconciliation_attempt_count", "3"),
                ("attempt_count", "3"),
                ("first_claimed_at", "now()"),
            ],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_claimed_by_check",
        &[with(RUNNING_COLUMNS, &[("claimed_by", "repeat('c', 128)")])],
        &[
            with(RUNNING_COLUMNS, &[("claimed_by", "repeat('c', 129)")]),
            with(RUNNING_COLUMNS, &[("claimed_by", "'   '")]),
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_executor_principal_check",
        &[with(
            RUNNING_COLUMNS,
            &[("executor_principal", "repeat('p', 256)")],
        )],
        &[with(
            RUNNING_COLUMNS,
            &[("executor_principal", "repeat('p', 257)")],
        )],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_first_claimed_at_check",
        &[vec![], CLAIMED_ONCE.to_vec()],
        &[
            vec![("attempt_count", "1"), ("execution_claim_count", "1")],
            vec![("first_claimed_at", "now()")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_checkpoint_check",
        &[vec![(
            "checkpoint",
            "jsonb_build_object('a', repeat('x', 65527))",
        )]],
        &[
            vec![("checkpoint", "jsonb_build_object('a', repeat('x', 65528))")],
            vec![("checkpoint", "'[]'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_provider_ref_check",
        &[vec![("provider_ref", "repeat('p', 512)")]],
        &[vec![("provider_ref", "repeat('p', 513)")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_last_outcome_code_check",
        &[
            vec![("last_outcome_code", "'A' || repeat('B', 63)")],
            vec![("last_outcome_code", "'A1_B'")],
        ],
        &[
            vec![("last_outcome_code", "'A' || repeat('B', 64)")],
            vec![("last_outcome_code", "'aB'")],
            vec![("last_outcome_code", "'1A'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_causation_ref_check",
        &[vec![("causation_ref", "repeat('c', 256)")]],
        &[
            vec![("causation_ref", "repeat('c', 257)")],
            vec![("causation_ref", "'a b'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_created_by_check",
        &[vec![("created_by", "repeat('c', 256)")]],
        &[
            vec![("created_by", "repeat('c', 257)")],
            vec![("created_by", "'  '")],
            vec![("created_by", "''")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_claim_state_check",
        &[
            RUNNING_COLUMNS.to_vec(),
            with(
                CLAIMED_ONCE,
                &[
                    ("status", "'RECONCILIATION_REQUIRED'"),
                    ("effect_phase", "'EFFECT_STARTED'"),
                ],
            ),
            with(
                RUNNING_COLUMNS,
                &[
                    ("status", "'RECONCILIATION_REQUIRED'"),
                    ("effect_phase", "'EFFECT_STARTED'"),
                ],
            ),
        ],
        &[
            with(RUNNING_COLUMNS, &[("lease_expires_at", "NULL")]),
            with(
                CLAIMED_ONCE,
                &[
                    ("status", "'RECONCILIATION_REQUIRED'"),
                    ("effect_phase", "'EFFECT_STARTED'"),
                    ("claim_token", "uuid_generate_v4()"),
                ],
            ),
            vec![("claimed_by", "'worker'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_terminal_check",
        &[with(
            CLAIMED_ONCE,
            &[("status", "'SUCCEEDED'"), ("terminal_at", "now()")],
        )],
        &[
            with(CLAIMED_ONCE, &[("status", "'SUCCEEDED'")]),
            vec![("terminal_at", "now()")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_redacted_at_check",
        &[with(
            CLAIMED_ONCE,
            &[
                ("status", "'FAILED'"),
                ("terminal_at", "now()"),
                ("redacted_at", "now()"),
            ],
        )],
        &[vec![("redacted_at", "now()")]],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_waiting_check",
        &[with(
            CLAIMED_ONCE,
            &[
                ("status", "'WAITING'"),
                ("checkpoint", "'{}'"),
                ("waiting_deadline", "now()"),
            ],
        )],
        &[
            with(
                CLAIMED_ONCE,
                &[("status", "'WAITING'"), ("waiting_deadline", "now()")],
            ),
            with(
                CLAIMED_ONCE,
                &[("status", "'WAITING'"), ("checkpoint", "'{}'")],
            ),
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_pre_write_states_check",
        &[with(
            RUNNING_COLUMNS,
            &[("effect_phase", "'EFFECT_STARTED'")],
        )],
        &[
            vec![("effect_phase", "'EFFECT_STARTED'")],
            with(
                CLAIMED_ONCE,
                &[
                    ("status", "'RETRY_SCHEDULED'"),
                    ("effect_phase", "'EFFECT_CONFIRMED'"),
                ],
            ),
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_effect_class_check",
        &[vec![("effect_class", "'READ_ONLY'")]],
        &[with(
            RUNNING_COLUMNS,
            &[
                ("effect_class", "'READ_ONLY'"),
                ("effect_phase", "'EFFECT_STARTED'"),
            ],
        )],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_revision_check",
        &[vec![
            ("revision_mode", "'REVISION_BOUND'"),
            ("source_order", "1"),
            ("source_fingerprint", "'f'"),
        ]],
        &[
            vec![
                ("revision_mode", "'REVISION_BOUND'"),
                ("source_fingerprint", "'f'"),
            ],
            vec![("source_order", "1")],
            vec![("source_fingerprint", "'f'")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_lineage_check",
        &[vec![
            ("lineage", "'MANUAL_RETRY'"),
            ("parent_job_id", "uuid_generate_v4()"),
        ]],
        &[
            vec![("lineage", "'MANUAL_RETRY'")],
            vec![("parent_job_id", "uuid_generate_v4()")],
        ],
    );
    check_bounds(
        c,
        t,
        b,
        "async_job_counters_check",
        &[with(
            CLAIMED_ONCE,
            &[
                ("attempt_count", "2"),
                ("reconciliation_attempt_count", "1"),
            ],
        )],
        &[with(CLAIMED_ONCE, &[("reconciliation_attempt_count", "1")])],
    );
}

#[test]
fn schema_other_table_checks_accept_their_boundary_and_refuse_one_past_it() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    let target: &[(&str, &str)] = &[
        ("async_job_id", "uuid_generate_v4()"),
        ("target_key", "'ab:x'"),
    ];
    check_bounds(
        c,
        "async_job_target",
        target,
        "async_job_target_target_key_check",
        &[
            vec![],
            vec![("target_key", "'a' || repeat('b', 31) || ':x'")],
            vec![("target_key", "'ns:' || repeat('v', 400)")],
            vec![("target_key", "'n1_:!~'")],
        ],
        &[
            vec![("target_key", "'a:x'")],
            vec![("target_key", "'a' || repeat('b', 32) || ':x'")],
            vec![("target_key", "'ns:' || repeat('v', 401)")],
            vec![("target_key", "'Ab:x'")],
            vec![("target_key", "'ab:x y'")],
            vec![("target_key", "'ab:'")],
            vec![("target_key", "'abx'")],
        ],
    );

    let attempt: &[(&str, &str)] = &[
        ("async_job_id", "uuid_generate_v4()"),
        ("attempt_number", "1"),
        ("purpose", "'EXECUTION'"),
        ("claim_token", "uuid_generate_v4()"),
        ("claimed_by", "'worker'"),
        ("executor_principal", "'principal'"),
        ("started_at", "now()"),
        ("last_heartbeat_at", "now()"),
        ("phase_at_start", "'PRE_WRITE'"),
    ];
    let a = "async_job_attempt";
    let finished: &[(&str, &str)] = &[
        ("finished_at", "now()"),
        ("result", "'SUCCEEDED'"),
        ("job_status_after", "'SUCCEEDED'"),
    ];
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_number_check",
        &[vec![("attempt_number", "1")]],
        &[vec![("attempt_number", "0")]],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_command_check",
        &[vec![
            ("purpose", "'STAFF_RECONCILIATION'"),
            ("control_command_id", "uuid_generate_v4()"),
        ]],
        &[
            vec![("purpose", "'STAFF_RECONCILIATION'")],
            vec![("control_command_id", "uuid_generate_v4()")],
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_finished_check",
        &[finished.to_vec()],
        &[
            vec![
                ("finished_at", "now()"),
                ("job_status_after", "'SUCCEEDED'"),
            ],
            vec![("result", "'SUCCEEDED'")],
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_job_status_after_check",
        &[finished.to_vec()],
        &[vec![("finished_at", "now()"), ("result", "'SUCCEEDED'")]],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_outcome_code_check",
        &[with(
            finished,
            &[("outcome_code", "'A' || repeat('B', 63)")],
        )],
        &[
            with(finished, &[("outcome_code", "'A' || repeat('B', 64)")]),
            with(finished, &[("outcome_code", "'bad'")]),
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_error_code_check",
        &[vec![("error_code", "'E' || repeat('B', 63)")]],
        &[
            vec![("error_code", "'E' || repeat('B', 64)")],
            vec![("error_code", "'e'")],
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_error_detail_check",
        &[vec![
            ("error_code", "'E'"),
            ("error_detail", "repeat('d', 2048)"),
        ]],
        &[
            vec![("error_code", "'E'"), ("error_detail", "repeat('d', 2049)")],
            vec![("error_detail", "'d'")],
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_provider_ref_check",
        &[vec![("provider_ref", "repeat('p', 512)")]],
        &[vec![("provider_ref", "repeat('p', 513)")]],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_evidence_check",
        &[vec![(
            "evidence",
            "jsonb_build_object('a', repeat('x', 16375))",
        )]],
        &[
            vec![("evidence", "jsonb_build_object('a', repeat('x', 16376))")],
            vec![("evidence", "'[]'")],
        ],
    );
    check_bounds(
        c,
        a,
        attempt,
        "async_job_attempt_redaction_check",
        &[vec![
            ("redacted_at", "now()"),
            ("redaction_command_id", "uuid_generate_v4()"),
        ]],
        &[
            vec![("redacted_at", "now()")],
            vec![("redaction_command_id", "uuid_generate_v4()")],
        ],
    );

    let command: &[(&str, &str)] = &[
        ("command_kind", "'CANCEL_JOB'"),
        ("request_id", "uuid_generate_v4()"),
        ("actor_principal", "'staff'"),
        ("reason", "'r'"),
        ("async_job_id", "uuid_generate_v4()"),
        ("request", "'{}'"),
    ];
    let k = "async_control_command";
    let decided: &[(&str, &str)] = &[
        ("outcome", "'APPLIED'"),
        ("outcome_code", "'CANCELLED'"),
        ("decided_at", "now()"),
    ];
    check_bounds(
        c,
        k,
        command,
        "async_control_command_actor_principal_check",
        &[vec![("actor_principal", "repeat('a', 256)")]],
        &[vec![("actor_principal", "repeat('a', 257)")]],
    );
    check_bounds(
        c,
        k,
        command,
        "async_control_command_reason_check",
        &[vec![("reason", "repeat('r', 1000)")]],
        &[
            vec![("reason", "repeat('r', 1001)")],
            vec![("reason", "' '")],
            vec![("reason", "''")],
        ],
    );
    check_bounds(
        c,
        k,
        command,
        "async_control_command_request_check",
        &[vec![(
            "request",
            "jsonb_build_object('a', repeat('x', 4087))",
        )]],
        &[
            vec![("request", "jsonb_build_object('a', repeat('x', 4088))")],
            vec![("request", "'[]'")],
        ],
    );
    check_bounds(
        c,
        k,
        command,
        "async_control_command_decision_check",
        &[decided.to_vec()],
        &[
            vec![("outcome", "'APPLIED'")],
            vec![("decided_at", "now()")],
            vec![("outcome_code", "'X'")],
        ],
    );
    check_bounds(
        c,
        k,
        command,
        "async_control_command_result_job_id_check",
        &[with(
            decided,
            &[
                ("command_kind", "'RETRY_JOB'"),
                ("result_job_id", "uuid_generate_v4()"),
            ],
        )],
        &[
            with(decided, &[("result_job_id", "uuid_generate_v4()")]),
            with(
                decided,
                &[
                    ("command_kind", "'RETRY_JOB'"),
                    ("outcome", "'REFUSED'"),
                    ("result_job_id", "uuid_generate_v4()"),
                ],
            ),
        ],
    );

    let state: &[(&str, &str)] = &[
        ("target_key", "'ns:k'"),
        ("applied_fingerprint", "'fp'"),
        ("last_job_id", "uuid_generate_v4()"),
        ("last_attempt_id", "uuid_generate_v4()"),
        ("updated_at", "now()"),
    ];
    let s = "async_effect_target_state";
    check_bounds(
        c,
        s,
        state,
        "async_effect_target_state_applied_fingerprint_check",
        &[vec![("applied_fingerprint", "repeat('f', 256)")]],
        &[
            vec![("applied_fingerprint", "repeat('f', 257)")],
            vec![("applied_fingerprint", "'a b'")],
            vec![("applied_fingerprint", "''")],
        ],
    );
    check_bounds(
        c,
        s,
        state,
        "async_effect_target_state_applied_source_ref_check",
        &[vec![("applied_source_ref", "repeat('s', 256)")]],
        &[
            vec![("applied_source_ref", "repeat('s', 257)")],
            vec![("applied_source_ref", "'a b'")],
        ],
    );
    check_bounds(
        c,
        s,
        state,
        "async_effect_target_state_regressed_check",
        &[vec![
            ("regressed_at", "now()"),
            ("regressed_from_order", "1"),
            ("regressed_from_fingerprint", "'f'"),
        ]],
        &[
            vec![("regressed_at", "now()")],
            vec![
                ("regressed_from_order", "1"),
                ("regressed_from_fingerprint", "'f'"),
            ],
        ],
    );

    let failure: &[(&str, &str)] = &[
        ("async_job_id", "uuid_generate_v4()"),
        ("first_failed_at", "now()"),
        ("last_failed_at", "now()"),
        ("failure_count", "1"),
        ("error_code", "'ASYNC_JOB_BUSY'"),
    ];
    check_bounds(
        c,
        "async_recovery_failure",
        failure,
        "async_recovery_failure_failure_count_check",
        &[vec![("failure_count", "1")]],
        &[vec![("failure_count", "0")]],
    );

    let poll: &[(&str, &str)] = &[
        ("executor_name", "'e'"),
        ("executor_principal", "'p'"),
        ("kind", "'TEST_EFFECT'"),
        ("purpose", "'EXECUTION'"),
        ("payload_versions", "'{1}'"),
        ("last_poll_at", "now()"),
    ];
    check_bounds(
        c,
        "async_executor_poll",
        poll,
        "async_executor_poll_purpose_check",
        &[
            vec![("purpose", "'EXECUTION'")],
            vec![("purpose", "'RECONCILIATION'")],
        ],
        &[vec![("purpose", "'STAFF_RECONCILIATION'")]],
    );
}

const ALL_STATUSES: [&str; 8] = [
    "PENDING",
    "RUNNING",
    "WAITING",
    "RETRY_SCHEDULED",
    "SUCCEEDED",
    "FAILED",
    "CANCELLED",
    "RECONCILIATION_REQUIRED",
];

fn legal_pair(from: &str, to: &str) -> bool {
    let terminal = |s: &str| matches!(s, "SUCCEEDED" | "FAILED" | "CANCELLED");
    if from == to {
        return !terminal(from);
    }
    match from {
        "PENDING" | "RETRY_SCHEDULED" => matches!(to, "RUNNING" | "CANCELLED"),
        "WAITING" => matches!(
            to,
            "RUNNING" | "CANCELLED" | "FAILED" | "RECONCILIATION_REQUIRED"
        ),
        "RUNNING" => matches!(
            to,
            "SUCCEEDED"
                | "FAILED"
                | "RETRY_SCHEDULED"
                | "WAITING"
                | "PENDING"
                | "CANCELLED"
                | "RECONCILIATION_REQUIRED"
        ),
        "RECONCILIATION_REQUIRED" => {
            matches!(to, "SUCCEEDED" | "FAILED" | "CANCELLED" | "WAITING")
        }
        _ => false,
    }
}

/// A row in `status` at `phase`, inserted with triggers disabled, satisfying
/// every check constraint.
fn state_row_sql(id: Uuid, status: &str, phase: &str) -> String {
    let claimed = status == "RUNNING";
    let terminal = matches!(status, "SUCCEEDED" | "FAILED" | "CANCELLED");
    let claim = |value: &str| {
        if claimed {
            value.to_string()
        } else {
            "NULL".to_string()
        }
    };
    format!(
        "INSERT INTO public.async_job (async_job_id, kind, payload_version, payload, \
             idempotency_key, effect_class, revision_mode, retry_budget, max_execution_claims, \
             max_reconciliation_attempts, backoff_base_seconds, backoff_max_seconds, created_by, \
             status, effect_phase, attempt_count, execution_claim_count, first_claimed_at, \
             checkpoint, waiting_deadline, claim_token, claimed_by, executor_principal, \
             claimed_at, lease_expires_at, attempt_runtime_deadline, terminal_at) \
         VALUES ('{id}', 'TEST_EFFECT', 1, '{{}}', 'TEST_EFFECT:{id}', 'EFFECTFUL', \
             'CURRENT_STATE', 3, 4, 2, 10, 300, 'test', '{status}', '{phase}', 1, 1, now(), \
             '{{}}', now() + interval '1 day', {}, {}, {}, {}, {}, {}, {})",
        claim("uuid_generate_v4()"),
        claim("'worker'"),
        claim("'principal'"),
        claim("now()"),
        claim("now()"),
        claim("now()"),
        if terminal { "now()" } else { "NULL" },
    )
}

/// The status change to `to`, with the claim and terminal columns its check
/// constraints need.
fn status_update_sql(id: Uuid, to: &str) -> String {
    let claimed = to == "RUNNING";
    let terminal = matches!(to, "SUCCEEDED" | "FAILED" | "CANCELLED");
    let claim = |value: &str| {
        if claimed {
            value.to_string()
        } else {
            "NULL".to_string()
        }
    };
    format!(
        "UPDATE public.async_job SET status = '{to}', claim_token = {}, claimed_by = {}, \
             executor_principal = {}, claimed_at = {}, lease_expires_at = {}, \
             attempt_runtime_deadline = {}, terminal_at = {} \
          WHERE async_job_id = '{id}'",
        claim("uuid_generate_v4()"),
        claim("'worker'"),
        claim("'principal'"),
        claim("now()"),
        claim("now()"),
        claim("now()"),
        if terminal { "now()" } else { "NULL" },
    )
}

fn transition_probe(
    conn: &mut PgConnection,
    from: &str,
    phase: &str,
    to: &str,
) -> Result<(), String> {
    let id = Uuid::new_v4();
    let sql = format!(
        "SET LOCAL session_replication_role = replica; {}; \
         SET LOCAL session_replication_role = origin; {}",
        state_row_sql(id, from, phase),
        status_update_sql(id, to)
    );
    probe(conn, &sql, true)
}

#[test]
fn schema_every_illegal_status_pair_is_refused_exhaustive_matrix() {
    let (_guard, mut conn) = setup();
    for from in ALL_STATUSES {
        for to in ALL_STATUSES {
            let result = transition_probe(&mut conn, from, "PRE_WRITE", to);
            if legal_pair(from, to) {
                assert_eq!(result, Ok(()), "{from} -> {to} is legal");
            } else {
                assert_eq!(
                    result,
                    Err("async_job_update_guard".to_string()),
                    "{from} -> {to} is illegal"
                );
            }
        }
    }
}

#[test]
fn schema_pending_and_retry_scheduled_require_pre_write() {
    let (_guard, mut conn) = setup();
    for to in ["PENDING", "RETRY_SCHEDULED"] {
        for phase in ["EFFECT_STARTED", "EFFECT_CONFIRMED"] {
            assert_eq!(
                transition_probe(&mut conn, "RUNNING", phase, to),
                Err("async_job_pre_write_states_check".to_string()),
                "RUNNING at {phase} -> {to}"
            );
        }
        assert_eq!(
            transition_probe(&mut conn, "RUNNING", "PRE_WRITE", to),
            Ok(())
        );
    }
}

/// A job row with triggers disabled, committed, for direct-SQL guard tests.
fn insert_row(conn: &mut PgConnection, status: &str, phase: &str) -> Uuid {
    let id = Uuid::new_v4();
    bypass(conn, &state_row_sql(id, status, phase));
    id
}

#[test]
fn schema_terminal_rows_are_frozen_except_one_redaction() {
    let (_guard, mut conn) = setup();
    for status in ["SUCCEEDED", "FAILED", "CANCELLED"] {
        let id = insert_row(&mut conn, status, "PRE_WRITE");
        bypass(
            &mut conn,
            &format!("UPDATE public.async_job SET provider_ref = 'p' WHERE async_job_id = '{id}'"),
        );
        let guard = "async_job_update_guard";
        for change in [
            "provider_ref = 'q'",
            "priority = 5",
            "available_at = now() + interval '1 hour'",
            "last_outcome_code = 'OTHER'",
            "provider_ref = NULL, checkpoint = NULL, redacted_at = now(), priority = 1",
            "checkpoint = NULL, redacted_at = now()",
            "provider_ref = NULL, redacted_at = now()",
            "provider_ref = NULL, checkpoint = NULL",
        ] {
            assert_eq!(
                refused_by(
                    &mut conn,
                    &format!("UPDATE public.async_job SET {change} WHERE async_job_id = '{id}'")
                ),
                guard,
                "{status}: {change}"
            );
        }
        exec(
            &mut conn,
            &format!(
                "UPDATE public.async_job SET provider_ref = NULL, checkpoint = NULL, \
                 redacted_at = now() WHERE async_job_id = '{id}'"
            ),
        );
        assert_eq!(
            refused_by(
                &mut conn,
                &format!(
                    "UPDATE public.async_job SET provider_ref = NULL, checkpoint = NULL, \
                     redacted_at = now() WHERE async_job_id = '{id}'"
                )
            ),
            guard,
            "a second redaction"
        );
    }
}

#[test]
fn schema_identity_and_snapshot_columns_are_frozen() {
    let (_guard, mut conn) = setup();
    let id = insert_row(&mut conn, "RUNNING", "PRE_WRITE");
    for change in [
        "kind = 'TEST_PEER', idempotency_key = 'TEST_PEER:x'",
        "payload_version = 2",
        "payload = '{\"other\": 1}'",
        "idempotency_key = 'TEST_EFFECT:other'",
        "effect_class = 'READ_ONLY'",
        "revision_mode = 'REVISION_BOUND', source_order = 1, source_fingerprint = 'f'",
        "retry_budget = 4",
        "max_execution_claims = 5",
        "max_reconciliation_attempts = 3",
        "backoff_base_seconds = 11",
        "backoff_max_seconds = 301",
        "source_ref = 'ref'",
        "correlation_id = uuid_generate_v4()",
        "causation_ref = 'cause'",
        "created_by = 'someone'",
        "created_at = now() - interval '1 day'",
        "async_job_id = uuid_generate_v4()",
    ] {
        assert_eq!(
            refused_by(
                &mut conn,
                &format!("UPDATE public.async_job SET {change} WHERE async_job_id = '{id}'")
            ),
            "async_job_update_guard",
            "{change}"
        );
    }
    // Lineage and parent: an ORIGINAL row cannot become a retry.
    let other = insert_row(&mut conn, "FAILED", "PRE_WRITE");
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "UPDATE public.async_job SET lineage = 'MANUAL_RETRY', parent_job_id = '{other}' \
                 WHERE async_job_id = '{id}'"
            )
        ),
        "async_job_update_guard"
    );
}

#[test]
fn schema_effect_phase_counters_and_write_once_columns_never_go_back() {
    let (_guard, mut conn) = setup();
    let id = insert_row(&mut conn, "RUNNING", "EFFECT_STARTED");
    bypass(
        &mut conn,
        &format!(
            "UPDATE public.async_job SET effect_phase = 'EFFECT_CONFIRMED', attempt_count = 2, \
             checkpoint_version = 2, prewrite_failure_count = 1 WHERE async_job_id = '{id}'"
        ),
    );
    for change in [
        "effect_phase = 'EFFECT_STARTED'",
        "effect_phase = 'PRE_WRITE'",
        "attempt_count = 1",
        "execution_claim_count = 0",
        "prewrite_failure_count = 0",
        "checkpoint_version = 1",
        "first_claimed_at = now() - interval '1 hour'",
        "waiting_deadline = now() + interval '2 days'",
        // Rule 7 (a BEFORE trigger) fires before the check constraints.
        "first_claimed_at = NULL",
        "waiting_deadline = NULL",
    ] {
        assert_eq!(
            refused_by(
                &mut conn,
                &format!("UPDATE public.async_job SET {change} WHERE async_job_id = '{id}'")
            ),
            "async_job_update_guard",
            "{change}"
        );
    }
    let rr = insert_row(&mut conn, "RECONCILIATION_REQUIRED", "EFFECT_STARTED");
    bypass(
        &mut conn,
        &format!(
            "UPDATE public.async_job SET reconciliation_attempt_count = 1, attempt_count = 2 \
             WHERE async_job_id = '{rr}'"
        ),
    );
    assert_eq!(
        refused_by(
            &mut conn,
            &format!("UPDATE public.async_job SET reconciliation_attempt_count = 0 WHERE async_job_id = '{rr}'")
        ),
        "async_job_update_guard"
    );
}

#[test]
fn schema_insert_guard_requires_a_fresh_pending_row() {
    let (_guard, mut conn) = setup();
    let base = insert_sql("async_job", JOB_BASE, &[]);
    assert_eq!(probe(&mut conn, &base, true), Ok(()));
    for overrides in [
        with(
            CLAIMED_ONCE,
            &[("status", "'SUCCEEDED'"), ("terminal_at", "now()")],
        ),
        RUNNING_COLUMNS.to_vec(),
        CLAIMED_ONCE.to_vec(),
        vec![("checkpoint_version", "1")],
        vec![("prewrite_failure_count", "1")],
        vec![("provider_ref", "'p'")],
        vec![("last_outcome_code", "'X'")],
        vec![("checkpoint", "'{}'")],
        vec![("waiting_deadline", "now()")],
        vec![
            ("effect_phase", "'EFFECT_STARTED'"),
            ("status", "'RECONCILIATION_REQUIRED'"),
        ],
    ] {
        let sql = insert_sql("async_job", JOB_BASE, &overrides);
        assert_eq!(
            probe(&mut conn, &sql, true),
            Err("async_job_insert_guard".to_string()),
            "{overrides:?}"
        );
    }
}

#[test]
fn schema_attempt_immutability_and_redaction_rules() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let (job_row, claim) = running(&mut conn, new_job(TEST_EFFECT, "a", &[&target]));
    let open = attempts(&mut conn, job_row.async_job_id)[0].async_job_attempt_id;
    let attempt_sql = |change: &str| {
        format!(
            "UPDATE public.async_job_attempt SET {change} WHERE async_job_attempt_id = '{open}'"
        )
    };
    let guard = "async_job_attempt_guard";
    // An open attempt: the heartbeat and provider ref change freely, effect
    // marks are write-once, identity columns never change, and the closing
    // columns are set only together with finished_at.
    exec(
        &mut conn,
        &attempt_sql("last_heartbeat_at = now(), provider_ref = 'p1'"),
    );
    exec(&mut conn, &attempt_sql("provider_ref = 'p2'"));
    do_start(&mut conn, &claim).expect("effect started");
    for change in [
        "effect_started_at = now() + interval '1 second'",
        "effect_started_at = NULL",
        "claim_token = uuid_generate_v4()",
        "claimed_by = 'other'",
        "executor_principal = 'other'",
        "purpose = 'RECONCILIATION'",
        "attempt_number = 9",
        "started_at = now() - interval '1 hour'",
        "phase_at_start = 'EFFECT_STARTED'",
        "outcome_code = 'X'",
        "error_code = 'E'",
        "error_detail = 'd'",
        "evidence = '{}'",
        "result = 'SUCCEEDED'",
        "redacted_at = now()",
    ] {
        assert_eq!(
            refused_by(&mut conn, &attempt_sql(change)),
            guard,
            "{change}"
        );
    }
    assert_eq!(
        refused_by(
            &mut conn,
            &format!("DELETE FROM public.async_job_attempt WHERE async_job_attempt_id = '{open}'")
        ),
        guard
    );
    // Close it through the engine; then only the one redaction may change it.
    let outcome = do_complete(
        &mut conn,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&target, "fp")],
        ),
    )
    .expect("complete");
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    for change in [
        "outcome_code = 'OTHER'",
        "provider_ref = 'p3'",
        "last_heartbeat_at = now()",
        "error_detail = NULL",
        "finished_at = NULL, result = NULL, job_status_after = NULL",
        "effect_confirmed_at = now()",
    ] {
        assert_eq!(
            refused_by(&mut conn, &attempt_sql(change)),
            guard,
            "{change}"
        );
    }
    assert_eq!(
        refused_by(
            &mut conn,
            &format!("DELETE FROM public.async_job_attempt WHERE async_job_attempt_id = '{open}'")
        ),
        guard
    );
    // The redaction needs a decided command row; build one in the statement.
    let redaction = |extra: &str| {
        format!(
            "WITH c AS (INSERT INTO public.async_control_command \
                 (command_kind, request_id, actor_principal, reason, async_job_id, request, \
                  outcome, outcome_code, decided_at) \
               VALUES ('REDACT_JOB_DIAGNOSTICS', uuid_generate_v4(), 'staff', 'r', '{}', '{{}}', \
                  'APPLIED', 'REDACTED', now()) RETURNING async_control_command_id) \
             UPDATE public.async_job_attempt SET redacted_at = now(), \
                 redaction_command_id = (SELECT async_control_command_id FROM c), \
                 error_detail = NULL, provider_ref = NULL, evidence = NULL{extra} \
              WHERE async_job_attempt_id = '{open}'",
            job_row.async_job_id
        )
    };
    assert_eq!(
        refused_by(&mut conn, &redaction(", outcome_code = 'X'")),
        guard
    );
    assert_eq!(probe(&mut conn, &redaction(""), true), Ok(()));
    // Applied once (committed), a second redaction is refused.
    exec(&mut conn, &redaction(""));
    assert_eq!(refused_by(&mut conn, &redaction("")), guard);
}

// ---------------------------------------------------------------------------
// Sessions: one connection per thread inside one READ COMMITTED transaction,
// driven step by step, so a test orders work across connections explicitly
// ---------------------------------------------------------------------------

type BoxedStep = Box<dyn FnOnce(&mut PgConnection) -> Box<dyn std::any::Any + Send> + Send>;

struct Session {
    pid: i32,
    steps: mpsc::Sender<Option<BoxedStep>>,
    results: mpsc::Receiver<Box<dyn std::any::Any + Send>>,
    handle: thread::JoinHandle<Result<(), DieselError>>,
}

impl Session {
    /// A session with a 30-second lock timeout, so an unexpected wait fails
    /// the test instead of hanging it.
    fn open() -> Session {
        Session::open_with("SET LOCAL lock_timeout = '30s'")
    }

    /// A session whose transaction first runs `setup`.
    fn open_with(setup: &str) -> Session {
        let (steps, step_rx) = mpsc::channel::<Option<BoxedStep>>();
        let (result_tx, results) = mpsc::channel();
        let setup = setup.to_string();
        let (pid, handle) = spawn_with_connection(move |conn| {
            conn.build_transaction()
                .read_committed()
                .run(|c| -> Result<(), DieselError> {
                    if !setup.is_empty() {
                        c.batch_execute(&setup)?;
                    }
                    while let Ok(Some(step)) = step_rx.recv() {
                        if result_tx.send(step(c)).is_err() {
                            break;
                        }
                    }
                    Ok(())
                })
        });
        Session {
            pid,
            steps,
            results,
            handle,
        }
    }

    /// Run a step and wait for its result.
    fn run<T: Send + 'static>(
        &self,
        step: impl FnOnce(&mut PgConnection) -> T + Send + 'static,
    ) -> T {
        self.send(step);
        self.result()
    }

    /// Start a step without waiting for it (it may block on a lock).
    fn send<T: Send + 'static>(&self, step: impl FnOnce(&mut PgConnection) -> T + Send + 'static) {
        let step: BoxedStep = Box::new(move |c| Box::new(step(c)));
        self.steps.send(Some(step)).expect("send a step");
    }

    /// The result of the step started last.
    fn result<T: 'static>(&self) -> T {
        *self
            .results
            .recv()
            .expect("the session step ran")
            .downcast::<T>()
            .expect("the step result has the expected type")
    }

    /// Commit the session's transaction (PostgreSQL turns the commit of an
    /// aborted transaction into a rollback).
    fn commit(self) {
        self.end().expect("commit the session");
    }

    /// End the session and return its commit result, for a session whose
    /// connection the test terminated on purpose.
    fn end(self) -> Result<(), DieselError> {
        self.steps.send(None).expect("signal the commit");
        self.handle.join().expect("the session thread panicked")
    }
}

// ---------------------------------------------------------------------------
// Schema and triggers (E1), continued
// ---------------------------------------------------------------------------

#[test]
fn schema_command_rows_are_append_only_and_decided_before_commit() {
    let (_guard, mut conn) = setup();
    let id = create(&mut conn, new_job(TEST_EFFECT, "cmd", &[&key(NS_CS)])).async_job_id;
    let insert = |command_id: Uuid| {
        format!(
            "INSERT INTO public.async_control_command (async_control_command_id, command_kind, \
                 request_id, actor_principal, reason, async_job_id, request) \
             VALUES ('{command_id}', 'CANCEL_JOB', '{command_id}', 'staff', 'r', '{id}', '{{}}')"
        )
    };
    let decide = |command_id: Uuid| {
        format!(
            "UPDATE public.async_control_command SET outcome = 'REFUSED', \
                 outcome_code = 'KIND_POLICY', decided_at = now() \
              WHERE async_control_command_id = '{command_id}'"
        )
    };
    // An undecided command never commits; one decided in its transaction does.
    assert_eq!(
        refused_by(&mut conn, &insert(Uuid::new_v4())),
        "async_control_command_decided"
    );
    let decided = Uuid::new_v4();
    exec(
        &mut conn,
        &format!("BEGIN; {}; {}; COMMIT", insert(decided), decide(decided)),
    );
    assert_eq!(command_rows(&mut conn), 1);
    // Decided once, the row never changes again and is never deleted.
    for change in [
        "outcome = 'APPLIED'",
        "outcome_code = 'OTHER'",
        "decided_at = now()",
        "reason = 'other'",
        "request = '{\"a\": 1}'",
        "actor_principal = 'other'",
        "result_job_id = NULL",
    ] {
        assert_eq!(
            refused_by(
                &mut conn,
                &format!(
                    "UPDATE public.async_control_command SET {change} \
                      WHERE async_control_command_id = '{decided}'"
                )
            ),
            "async_control_command_guard",
            "{change}"
        );
    }
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "DELETE FROM public.async_control_command \
                  WHERE async_control_command_id = '{decided}'"
            )
        ),
        "async_control_command_guard"
    );
    // A second decision in the deciding transaction, and a partial decision.
    let twice = Uuid::new_v4();
    assert_eq!(
        refused_by(
            &mut conn,
            &format!("{}; {}; {}", insert(twice), decide(twice), decide(twice))
        ),
        "async_control_command_guard"
    );
    let partial = Uuid::new_v4();
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "{}; UPDATE public.async_control_command SET outcome = 'APPLIED' \
                  WHERE async_control_command_id = '{partial}'",
                insert(partial)
            )
        ),
        "async_control_command_guard"
    );
    assert_eq!(command_rows(&mut conn), 1);
}

#[test]
fn schema_s5_deferred_hold_check_refuses_direct_sql() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let pending = create(&mut conn, new_job(TEST_EFFECT, "s5-pending", &[&target])).async_job_id;
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "INSERT INTO public.async_target_hold (target_key, async_job_id) \
                 VALUES ('{target}', '{pending}')"
            )
        ),
        "async_job_hold_consistency"
    );
    let other = key(NS_CS);
    let (running_job, _claim) = running(&mut conn, new_job(TEST_EFFECT, "s5-running", &[&other]));
    let id = running_job.async_job_id;
    assert_eq!(
        refused_by(
            &mut conn,
            &format!("DELETE FROM public.async_target_hold WHERE async_job_id = '{id}'")
        ),
        "async_job_hold_consistency"
    );
    // A status change that leaves a terminal job holding its target (its
    // attempt closed, so only S5 is broken).
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "UPDATE public.async_job_attempt SET finished_at = now(), result = 'CANCELLED', \
                     job_status_after = 'CANCELLED', outcome_code = 'CANCELLED' \
                  WHERE async_job_id = '{id}' AND finished_at IS NULL; \
                 UPDATE public.async_job SET status = 'CANCELLED', terminal_at = now(), \
                     claim_token = NULL, claimed_by = NULL, executor_principal = NULL, \
                     claimed_at = NULL, lease_expires_at = NULL, attempt_runtime_deadline = NULL \
                  WHERE async_job_id = '{id}'"
            )
        ),
        "async_job_hold_consistency"
    );
    // The check judges the transaction's final state: a delete and re-insert
    // of the same hold commits.
    exec(
        &mut conn,
        &format!(
            "BEGIN; DELETE FROM public.async_target_hold WHERE async_job_id = '{id}'; \
             INSERT INTO public.async_target_hold (target_key, async_job_id) \
             VALUES ('{other}', '{id}'); COMMIT"
        ),
    );
    assert_eq!(holds(&mut conn, id), vec![other]);
}

#[test]
fn schema_transition_log_names_every_engine_cause() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let causes_of = |c: &mut PgConnection, id: Uuid| causes(c, id);

    // T1, T2, T6; renewals and effect marks are same-status updates.
    let target = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "t-a", &[&target]));
    do_renew(c, &claim).unwrap();
    do_start(c, &claim).unwrap();
    do_confirm(c, &claim).unwrap();
    do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&target, "fp")],
        ),
    )
    .unwrap();
    assert_eq!(
        causes_of(c, a.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T6_COMPLETE"]
    );
    let log = transitions(c, a.async_job_id);
    assert_eq!(log[0].from_status, None);
    assert!(log[1].async_job_attempt_id.is_some());
    assert!(log.iter().all(|t| t.async_control_command_id.is_none()));

    // T7 then T8.
    let (b, claim) = running(c, new_job(TEST_EFFECT, "t-b", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    make_available(c, b.async_job_id);
    let claim = claim_job(c, &b);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_eq!(
        causes_of(c, b.async_job_id),
        [
            "T1_CREATE",
            "T2_CLAIM",
            "T7_FAIL_RETRYABLE",
            "T2_CLAIM",
            "T8_FAIL"
        ]
    );

    // T9; T15, STILL_REQUIRED and T17 leave no row; T16.
    let d = rr_started(c, new_job(TEST_EFFECT, "t-d", &[&key(NS_CS)]));
    let recon = claim_recon(c, &d);
    do_resolve(c, &recon, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    make_available(c, d.async_job_id);
    let recon = claim_recon(c, &d);
    expire_lease(c, d.async_job_id);
    do_recover(c);
    assert_eq!(
        status(c, d.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(
        causes_of(c, d.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T9_FAIL_AFTER_EFFECT_START"]
    );
    let _ = recon;
    // T18 (staff) on the same job.
    let result = do_staff(
        c,
        d.async_job_id,
        StaffResolution {
            evidence: Some(NO_EFFECT_EVIDENCE.to_string()),
            ..staff_res(StaffOutcome::Failed)
        },
    );
    assert_applied(&result, "NO_EFFECT_PROVEN");
    let log = transitions(c, d.async_job_id);
    assert_eq!(log.last().unwrap().cause, "T18_STAFF_RESOLVE");
    assert_eq!(
        log.last().unwrap().async_control_command_id,
        Some(result.async_control_command_id)
    );

    // T16.
    let e = rr_started(c, new_job(TEST_EFFECT, "t-e", &[&key(NS_CS)]));
    let recon = claim_recon(c, &e);
    do_resolve(
        c,
        &recon,
        ResolveReconciliation {
            evidence: Some(NO_EFFECT_EVIDENCE.to_string()),
            ..resolution(ReconcilerOutcome::Failed)
        },
    )
    .unwrap();
    assert_eq!(
        causes_of(c, e.async_job_id),
        [
            "T1_CREATE",
            "T2_CLAIM",
            "T9_FAIL_AFTER_EFFECT_START",
            "T16_RESOLVE"
        ]
    );

    // T6x.
    let (f, claim) = started(c, new_job(TEST_EFFECT, "t-f", &[&key(NS_CS)]));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&key(NS_CS), "fp")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        causes_of(c, f.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T6X_EVIDENCE_FAULT"]
    );

    // T9n.
    let (g, claim) = started(c, new_job(TEST_EFFECT, "t-g", &[&key(NS_CS)]));
    do_fail(
        c,
        &claim,
        no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
    )
    .unwrap();
    assert_eq!(
        causes_of(c, g.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T9N_FAIL_NO_EFFECT_PROVEN"]
    );

    // T10, a resume (T2) and T10d.
    let (h, claim) = started(c, new_job(TEST_EFFECT, "t-h", &[&key(NS_CS)]));
    let at = resume_in(c, 60);
    do_wait(c, &claim, "{\"step\": 1}", at).unwrap();
    make_available(c, h.async_job_id);
    let claim = claim_job(c, &h);
    expire_waiting_deadline(c, h.async_job_id, 1);
    let at = resume_in(c, 60);
    assert!(matches!(
        do_wait(c, &claim, "{\"step\": 2}", at).unwrap(),
        WaitOutcome::DeadlineExceeded(_)
    ));
    assert_eq!(
        causes_of(c, h.async_job_id),
        [
            "T1_CREATE",
            "T2_CLAIM",
            "T10_WAIT",
            "T2_CLAIM",
            "T10D_WAIT_DEADLINE_EXCEEDED"
        ]
    );

    // T11.
    let (i, claim) = running(c, new_job(TEST_EFFECT, "t-i", &[&key(NS_CS)]));
    let at = resume_in(c, 60);
    do_wait(c, &claim, "{}", at).unwrap();
    expire_waiting_deadline(c, i.async_job_id, 1);
    do_recover(c);
    assert_eq!(
        causes_of(c, i.async_job_id),
        [
            "T1_CREATE",
            "T2_CLAIM",
            "T10_WAIT",
            "T11_WAITING_DEADLINE_EXCEEDED"
        ]
    );
    assert!(transitions(c, i.async_job_id)[3]
        .async_job_attempt_id
        .is_none());

    // T12.
    let (j, _claim) = running(c, new_job(TEST_EFFECT, "t-j", &[&key(NS_CS)]));
    expire_lease(c, j.async_job_id);
    do_recover(c);
    assert_eq!(
        causes_of(c, j.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T12_LEASE_EXPIRED"]
    );

    // T13.
    let (k, claim) = running(c, new_job(TEST_EFFECT, "t-k", &[&key(NS_CS)]));
    do_release(c, &claim).unwrap();
    assert_eq!(
        causes_of(c, k.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T13_RELEASE"]
    );

    // T14, T14w, T14r, T14e.
    let l = create(c, new_job(TEST_EFFECT, "t-l", &[&key(NS_CS)]));
    let cancel = do_cancel(c, l.async_job_id);
    assert_eq!(causes_of(c, l.async_job_id), ["T1_CREATE", "T14_CANCEL"]);
    assert_eq!(
        transitions(c, l.async_job_id)[1].async_control_command_id,
        Some(cancel.async_control_command_id)
    );
    let (m, claim) = running(c, new_job(TEST_EFFECT, "t-m", &[&key(NS_CS)]));
    let at = resume_in(c, 60);
    do_wait(c, &claim, "{}", at).unwrap();
    do_cancel(c, m.async_job_id);
    assert_eq!(
        causes_of(c, m.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T10_WAIT", "T14W_CANCEL_WAITING"]
    );
    let (n, _claim) = running(c, new_job(TEST_EFFECT, "t-n", &[&key(NS_CS)]));
    do_cancel(c, n.async_job_id);
    assert_eq!(
        causes_of(c, n.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T14R_CANCEL_RUNNING"]
    );
    let (o, _claim) = started(c, new_job(TEST_EFFECT, "t-o", &[&key(NS_CS)]));
    do_cancel(c, o.async_job_id);
    assert_eq!(
        causes_of(c, o.async_job_id),
        ["T1_CREATE", "T2_CLAIM", "T14E_OPERATOR_STOP"]
    );

    // RETRY_JOB creation.
    let retry = do_retry(c, b.async_job_id);
    assert_applied(&retry, "RETRY_CREATED");
    let child = retry.result_job_id.unwrap();
    assert_eq!(causes_of(c, child), ["RETRY_JOB_CREATE"]);
    assert_eq!(
        transitions(c, child)[0].async_control_command_id,
        Some(retry.async_control_command_id)
    );

    assert_no_unspecified(c);
}

#[test]
fn schema_one_open_attempt_per_job() {
    let (_guard, mut conn) = setup();
    let (job_row, _claim) = running(&mut conn, new_job(TEST_EFFECT, "open", &[&key(NS_CS)]));
    let id = job_row.async_job_id;
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "INSERT INTO public.async_job_attempt (async_job_id, attempt_number, purpose, \
                     claim_token, claimed_by, executor_principal, started_at, last_heartbeat_at, \
                     phase_at_start) \
                 VALUES ('{id}', 2, 'EXECUTION', uuid_generate_v4(), 'w', 'p', now(), now(), \
                     'PRE_WRITE')"
            )
        ),
        "async_job_attempt_open_key"
    );
}

#[test]
fn schema_target_and_hold_foreign_keys() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let job_row = create(&mut conn, new_job(TEST_EFFECT, "fk", &[&target]));
    let id = job_row.async_job_id;
    let missing = Uuid::new_v4();
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "INSERT INTO public.async_job_target (async_job_id, target_key) \
                 VALUES ('{missing}', '{target}')"
            )
        ),
        "async_job_target_async_job_id_fkey"
    );
    // A hold must name one of the holder's own targets.
    let not_a_target = key(NS_CS);
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "INSERT INTO public.async_target_hold (target_key, async_job_id) \
                 VALUES ('{not_a_target}', '{id}')"
            )
        ),
        "async_target_hold_target_fkey"
    );
    // Targets are written only at creation: never updated or deleted.
    let (held, _claim) = running(&mut conn, new_job(TEST_EFFECT, "fk-held", &[&key(NS_CS)]));
    let held_id = held.async_job_id;
    for sql in [
        format!("DELETE FROM public.async_job_target WHERE async_job_id = '{held_id}'"),
        format!(
            "UPDATE public.async_job_target SET target_key = 'test_cs:moved' \
              WHERE async_job_id = '{held_id}'"
        ),
    ] {
        assert_eq!(
            refused_by(&mut conn, &sql),
            "async_job_target_guard",
            "{sql}"
        );
    }
}

#[test]
fn schema_one_retry_child_per_job() {
    let (_guard, mut conn) = setup();
    let parent = create(&mut conn, new_job(TEST_EFFECT, "parent", &[&key(NS_CS)])).async_job_id;
    let parent_value = format!("'{parent}'");
    let child = |n: u32| {
        let key_value = format!("'TEST_EFFECT:child-{n}'");
        insert_sql(
            "async_job",
            JOB_BASE,
            &[
                ("idempotency_key", key_value.as_str()),
                ("lineage", "'MANUAL_RETRY'"),
                ("parent_job_id", parent_value.as_str()),
            ],
        )
    };
    exec(&mut conn, &child(1));
    assert_eq!(
        refused_by(&mut conn, &child(2)),
        "async_job_parent_job_id_key"
    );
}

// ---------------------------------------------------------------------------
// Creation (E1)
// ---------------------------------------------------------------------------

#[test]
fn creation_idempotent_repeat_returns_existing_in_any_state() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let input = new_job(TEST_EFFECT, "idem", &[&target]);
    let first = create(&mut conn, input.clone());
    let again = tx(&mut conn, |c| create_job_on(c, &REGISTRY, input.clone())).unwrap();
    assert_eq!(again, CreateOutcome::Existing(first.clone()));
    // Metadata of the first request is not compared.
    let metadata = NewAsyncJob {
        correlation_id: Some(Uuid::new_v4()),
        causation_ref: Some("another-cause".to_string()),
        created_by: "someone-else".to_string(),
        ..input.clone()
    };
    assert!(matches!(
        tx(&mut conn, |c| create_job_on(c, &REGISTRY, metadata)).unwrap(),
        CreateOutcome::Existing(job) if job.async_job_id == first.async_job_id
    ));
    // Whatever its state, terminal included.
    let claim = claim_job(&mut conn, &first);
    do_start(&mut conn, &claim).unwrap();
    do_complete(
        &mut conn,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&target, "fp")],
        ),
    )
    .unwrap();
    match tx(&mut conn, |c| create_job_on(c, &REGISTRY, input.clone())).unwrap() {
        CreateOutcome::Existing(job) => {
            assert_eq!(job.async_job_id, first.async_job_id);
            assert_eq!(job.status, AsyncJobStatus::Succeeded);
        }
        other => panic!("expected Existing, got {other:?}"),
    }
    assert_eq!(count(&mut conn, "FROM public.async_job"), 1);
}

#[test]
fn creation_conflict_aborts_the_callers_transaction() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    create(&mut conn, new_job(TEST_EFFECT, "conflict", &[&target]));
    let conflicting = new_job(TEST_EFFECT, "conflict", &[&target, &key(NS_CS)]);
    let result = conn.build_transaction().read_committed().run(|c| {
        // The caller's own work, then the conflicting creation.
        create_job_on(
            c,
            &REGISTRY,
            new_job(TEST_EFFECT, "caller-work", &[&key(NS_CS)]),
        )?;
        create_job_on(c, &REGISTRY, conflicting.clone())
    });
    assert_code(result, "ASYNC_JOB_IDEMPOTENCY_CONFLICT");
    assert_eq!(
        count(&mut conn, "FROM public.async_job"),
        1,
        "the caller's work rolled back"
    );
    assert_eq!(
        count(
            &mut conn,
            "FROM public.async_job WHERE payload->>'id' = 'caller-work'"
        ),
        0
    );

    // Fail closed: a caller that ignores the error cannot commit its own
    // work, because the engine left the transaction aborted.
    let ignored: Result<(), DieselError> = conn.build_transaction().read_committed().run(|c| {
        create_job_on(
            c,
            &REGISTRY,
            new_job(TEST_EFFECT, "ignored-work", &[&key(NS_CS)]),
        )
        .expect("the caller's own job");
        let error = create_job_on(c, &REGISTRY, conflicting.clone()).unwrap_err();
        assert_eq!(code(&error), "ASYNC_JOB_IDEMPOTENCY_CONFLICT");
        assert!(
            c.batch_execute("SELECT 1").is_err(),
            "the caller's transaction is aborted"
        );
        Ok(())
    });
    let _ = ignored; // PostgreSQL turns the COMMIT of an aborted transaction into a rollback.
    assert_eq!(
        count(
            &mut conn,
            "FROM public.async_job WHERE payload->>'id' = 'ignored-work'"
        ),
        0,
        "nothing of the caller's transaction committed"
    );
    assert_eq!(count(&mut conn, "FROM public.async_job"), 1);
    assert_eq!(count(&mut conn, "FROM public.async_job_target"), 1);
    assert_no_unspecified(&mut conn);
}

#[test]
fn creation_payload_equality_is_by_canonical_form() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let with_payload = |payload: String| NewAsyncJob {
        payload,
        ..new_job(TEST_EFFECT, "canonical", &[&target])
    };
    let first = create(
        &mut conn,
        with_payload(format!(
            r#"{{"id":"canonical","n":1,"targets":["{target}"]}}"#
        )),
    );
    // Other key order, spacing and a duplicate key resolve to the same
    // canonical form: the same request.
    let same = format!(r#"{{ "targets": ["{target}"], "n": 0, "n": 1, "id": "canonical" }}"#);
    assert_eq!(
        tx(&mut conn, |c| create_job_on(
            c,
            &REGISTRY,
            with_payload(same)
        ))
        .unwrap(),
        CreateOutcome::Existing(first.clone())
    );
    // Number scale is significant: `1.0` is another request.
    let scaled = format!(r#"{{"id":"canonical","n":1.0,"targets":["{target}"]}}"#);
    assert_code(
        tx(&mut conn, |c| {
            create_job_on(c, &REGISTRY, with_payload(scaled))
        }),
        "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
    );
    assert_eq!(job(&mut conn, first.async_job_id).payload, first.payload);
}

#[test]
fn creation_key_collision_is_never_success_for_a_newer_revision_or_slot() {
    let (_guard, mut conn) = setup();
    let target = key(NS_RB);
    let same_identity = |order: i64, fingerprint: &str| NewAsyncJob {
        payload: json!({ "id": "x", "identity": "same", "targets": [target] }).to_string(),
        source_order: Some(order),
        source_fingerprint: Some(fingerprint.to_string()),
        ..new_job(TEST_REVISION, "x", &[&target])
    };
    create(&mut conn, same_identity(1, "f1"));
    assert_code(
        tx(&mut conn, |c| {
            create_job_on(c, &REGISTRY, same_identity(2, "f2"))
        }),
        "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
    );
    assert_code(
        tx(&mut conn, |c| {
            create_job_on(c, &REGISTRY, same_identity(1, "f1-other"))
        }),
        "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
    );
    let other_ref = NewAsyncJob {
        source_ref: Some("ref-other".to_string()),
        ..same_identity(1, "f1")
    };
    assert_code(
        tx(&mut conn, |c| create_job_on(c, &REGISTRY, other_ref)),
        "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
    );
    // A schedule slot whose identity collides with an earlier slot.
    let slot = |at: &str| NewAsyncJob {
        payload: json!({ "id": "s", "identity": "slot", "slot": at, "targets": [key(NS_CS)] })
            .to_string(),
        ..new_job(TEST_EFFECT, "s", &[])
    };
    create(&mut conn, slot("2026-11-13T00:00Z"));
    assert_code(
        tx(&mut conn, |c| {
            create_job_on(c, &REGISTRY, slot("2026-11-13T01:00Z"))
        }),
        "ASYNC_JOB_IDEMPOTENCY_CONFLICT",
    );
    // A newer revision with its own identity is a new job.
    let rb = key(NS_RB);
    let one = create(&mut conn, revision_job("doc", &rb, 1, "f1"));
    let two = create(&mut conn, revision_job("doc", &rb, 2, "f2"));
    assert_ne!(one.async_job_id, two.async_job_id);
}

fn creation_cases(target: &str) -> Vec<(&'static str, NewAsyncJob, &'static str)> {
    let cs = |payload: serde_json::Value| NewAsyncJob {
        payload: payload.to_string(),
        ..new_job(TEST_EFFECT, "c", &[])
    };
    let rb = key(NS_RB);
    vec![
        (
            "unknown kind",
            new_job("TEST_NOPE", "c", &[target]),
            "ASYNC_JOB_KIND_UNKNOWN",
        ),
        (
            "closed admission",
            new_job(TEST_RETIRED, "c", &[target]),
            "ASYNC_JOB_KIND_ADMISSION_CLOSED",
        ),
        (
            "non-creatable version",
            NewAsyncJob {
                payload_version: 2,
                ..new_job(TEST_VERSIONED, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED",
        ),
        (
            "payload without id",
            cs(json!({ "targets": [target] })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "payload refused by the kind",
            cs(json!({ "id": "c", "invalid": true, "targets": [target] })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "payload not JSON",
            NewAsyncJob {
                payload: "not json".to_string(),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "payload not an object",
            NewAsyncJob {
                payload: "[]".to_string(),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "credential key",
            cs(json!({ "id": "c", "targets": [target], "nested": { "apiKey": "v" } })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "credential key in an array",
            cs(json!({ "id": "c", "targets": [target], "list": [{ "Password": "v" }] })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "broad credential key",
            cs(json!({ "id": "c", "targets": [target], "token_count": 1 })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "undeclared namespace",
            new_job(TEST_EFFECT, "c", &["other:x"]),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "malformed target",
            new_job(TEST_EFFECT, "c", &["NoColon"]),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "duplicate target",
            new_job(TEST_EFFECT, "c", &[target, target]),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "oversized payload",
            cs(json!({ "id": "c", "targets": [target], "pad": "x".repeat(16384) })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "blank created_by",
            NewAsyncJob {
                created_by: "   ".to_string(),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "over-long created_by",
            NewAsyncJob {
                created_by: "c".repeat(257),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "priority above 100",
            NewAsyncJob {
                priority: 101,
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "priority below -100",
            NewAsyncJob {
                priority: -101,
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "source_order on CURRENT_STATE",
            NewAsyncJob {
                source_order: Some(1),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "source_fingerprint on CURRENT_STATE",
            NewAsyncJob {
                source_fingerprint: Some("f".to_string()),
                ..new_job(TEST_EFFECT, "c", &[target])
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "too many targets",
            new_job(
                TEST_EFFECT,
                "c",
                &[
                    &key(NS_CS),
                    &key(NS_CS),
                    &key(NS_CS),
                    &key(NS_CS),
                    &key(NS_CS),
                ],
            ),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "EFFECTFUL without target",
            new_job(TEST_EFFECT, "c", &[]),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "EFFECTFUL REVISION_BOUND without target",
            NewAsyncJob {
                payload: json!({ "id": "c", "rev": 1, "targets": [] }).to_string(),
                ..revision_job("c", &rb, 1, "f")
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "EFFECTFUL REVISION_BOUND with two targets",
            NewAsyncJob {
                payload: json!({ "id": "c", "rev": 1, "targets": [rb, key(NS_RB)] }).to_string(),
                ..revision_job("c", &rb, 1, "f")
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "REVISION_BOUND without source_order",
            NewAsyncJob {
                source_order: None,
                ..revision_job("c", &rb, 1, "f")
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "REVISION_BOUND without source_fingerprint",
            NewAsyncJob {
                source_fingerprint: None,
                ..revision_job("c", &rb, 1, "f")
            },
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
        (
            "over-long effect identity",
            cs(json!({ "id": "c", "identity": "i".repeat(401), "targets": [target] })),
            "ASYNC_JOB_PAYLOAD_INVALID",
        ),
    ]
}

#[test]
fn creation_refuses_every_invalid_input_and_writes_nothing() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    for (case, input, expected) in creation_cases(&target) {
        let result = tx(&mut conn, |c| create_job_on(c, &REGISTRY, input));
        match result {
            Err(error) => assert_eq!(code(&error), expected, "{case}: {error:?}"),
            Ok(outcome) => panic!("{case}: expected {expected}, got {outcome:?}"),
        }
    }
    assert_eq!(count(&mut conn, "FROM public.async_job"), 0);
    assert_eq!(count(&mut conn, "FROM public.async_job_target"), 0);
    assert_eq!(count(&mut conn, "FROM public.async_job_transition"), 0);
}

#[test]
fn creation_inside_a_caller_transaction_that_rolls_back_leaves_nothing() {
    let (_guard, mut conn) = setup();
    let result = conn
        .build_transaction()
        .read_committed()
        .run(|c| -> Result<(), DieselError> {
            let created = create_job_on(c, &REGISTRY, new_job(TEST_EFFECT, "rb", &[&key(NS_CS)]));
            assert!(matches!(created, Ok(CreateOutcome::Created(_))));
            Err(DieselError::RollbackTransaction)
        });
    assert!(matches!(result, Err(DieselError::RollbackTransaction)));
    for table in ["async_job", "async_job_target", "async_job_transition"] {
        assert_eq!(
            count(&mut conn, &format!("FROM public.{table}")),
            0,
            "{table}"
        );
    }
}

#[test]
fn creation_refuses_callers_outside_read_committed() {
    let (_guard, mut conn) = setup();
    let input = || new_job(TEST_EFFECT, "iso", &[&key(NS_CS)]);
    assert_code(
        conn.build_transaction()
            .repeatable_read()
            .run(|c| create_job_on(c, &REGISTRY, input())),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    assert_code(
        conn.build_transaction()
            .serializable()
            .run(|c| create_job_on(c, &REGISTRY, input())),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    // A bare connection: each statement would commit alone.
    assert_code(
        create_job_on(&mut conn, &REGISTRY, input()),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    // Every other engine function checks the same.
    assert_code(
        conn.build_transaction()
            .repeatable_read()
            .run(|c| claim_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 1))),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    assert_code(
        claim_on(
            &mut conn,
            &REGISTRY,
            &dissemination(),
            request(&[TEST_EFFECT], 1),
        ),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    assert_code(
        conn.build_transaction()
            .serializable()
            .run(|c| cancel_job_command_on(c, &REGISTRY, &staff("iso"), Uuid::new_v4())),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    assert_code(
        conn.build_transaction()
            .repeatable_read()
            .run(|c| get_job(c, Uuid::new_v4())),
        "ASYNC_JOB_ISOLATION_INVALID",
    );
    assert_eq!(count(&mut conn, "FROM public.async_job"), 0);
    assert_eq!(command_rows(&mut conn), 0);
}

#[test]
fn creation_identifier_format_faults_are_payload_invalid() {
    let (_guard, mut conn) = setup();
    let rb = key(NS_RB);
    let cases = vec![
        NewAsyncJob {
            source_fingerprint: Some("a b".to_string()),
            ..revision_job("i", &rb, 1, "f")
        },
        NewAsyncJob {
            source_fingerprint: Some("f".repeat(257)),
            ..revision_job("i", &rb, 1, "f")
        },
        NewAsyncJob {
            source_fingerprint: Some(String::new()),
            ..revision_job("i", &rb, 1, "f")
        },
        NewAsyncJob {
            source_fingerprint: Some("caf\u{e9}".to_string()),
            ..revision_job("i", &rb, 1, "f")
        },
        NewAsyncJob {
            source_ref: Some("a b".to_string()),
            ..revision_job("i", &rb, 1, "f")
        },
        NewAsyncJob {
            source_ref: Some("r".repeat(257)),
            ..new_job(TEST_EFFECT, "i", &[&key(NS_CS)])
        },
        NewAsyncJob {
            causation_ref: Some("a\tb".to_string()),
            ..new_job(TEST_EFFECT, "i", &[&key(NS_CS)])
        },
        NewAsyncJob {
            causation_ref: Some("c".repeat(257)),
            ..new_job(TEST_EFFECT, "i", &[&key(NS_CS)])
        },
    ];
    for input in cases {
        assert_code(
            tx(&mut conn, |c| create_job_on(c, &REGISTRY, input.clone())),
            "ASYNC_JOB_PAYLOAD_INVALID",
        );
    }
    // The boundary itself is accepted.
    create(
        &mut conn,
        NewAsyncJob {
            source_fingerprint: Some("f".repeat(256)),
            source_ref: Some("!~".repeat(128)),
            causation_ref: Some("c".repeat(256)),
            ..revision_job("i", &rb, 1, "f")
        },
    );
}

// ---------------------------------------------------------------------------
// Claims (E1)
// ---------------------------------------------------------------------------

const STALE: &str = "ASYNC_JOB_STALE_CLAIM";

fn signals_of(conn: &mut PgConnection, kind: &str) -> KindSignals {
    signals_with(conn, &REGISTRY, kind)
}

fn signals_with(conn: &mut PgConnection, registry: &AsyncJobRegistry, kind: &str) -> KindSignals {
    kind_signals(conn, registry)
        .expect("signals")
        .into_iter()
        .find(|signal| signal.kind == kind)
        .unwrap_or_else(|| panic!("no signals for {kind}"))
}

fn claim_ids(result: ThothResult<ClaimResult>) -> Vec<Uuid> {
    result
        .expect("claim")
        .jobs
        .into_iter()
        .map(|job| job.async_job_id)
        .collect()
}

fn claim_max(conn: &mut PgConnection, kinds: &[&str], max_jobs: i32) -> Vec<Uuid> {
    claim_ids(tx(conn, |c| {
        claim_on(c, &REGISTRY, &dissemination(), request(kinds, max_jobs))
    }))
}

#[test]
fn claims_four_connections_with_overlapping_kinds_never_claim_twice_and_keep_limits() {
    let (_guard, mut conn) = setup();
    for i in 0..6 {
        for kind in [TEST_LIMITED, TEST_EFFECT, TEST_PEER] {
            create(
                &mut conn,
                new_job(kind, &format!("{kind}-{i}"), &[&key(NS_CS)]),
            );
        }
    }
    let before = deadlocks(&mut conn);
    let requests: [&[&'static str]; 4] = [
        &[TEST_LIMITED, TEST_EFFECT],
        &[TEST_EFFECT, TEST_PEER],
        &[TEST_PEER, TEST_LIMITED],
        &[TEST_LIMITED, TEST_EFFECT, TEST_PEER],
    ];
    type ClaimTask = Box<dyn FnOnce(&mut PgConnection) -> ThothResult<ClaimResult> + Send>;
    let tasks: Vec<ClaimTask> = requests
        .iter()
        .enumerate()
        .map(|(n, kinds)| {
            let kinds: Vec<&'static str> = kinds.to_vec();
            let task: ClaimTask = Box::new(move |c: &mut PgConnection| {
                tx(c, |c| {
                    claim_on(
                        c,
                        &REGISTRY,
                        &executor_named(&format!("claimer-{n}")),
                        request(&kinds, 50),
                    )
                })
            });
            task
        })
        .collect();
    let mut claimed = Vec::new();
    let mut per_kind = std::collections::BTreeMap::<String, usize>::new();
    for result in run_together(tasks) {
        for job in result.expect("claim").jobs {
            claimed.push(job.async_job_id);
            *per_kind.entry(job.kind).or_default() += 1;
        }
    }
    let unique: std::collections::HashSet<Uuid> = claimed.iter().copied().collect();
    assert_eq!(unique.len(), claimed.len(), "a job was claimed twice");
    assert_eq!(
        per_kind.get(TEST_LIMITED),
        Some(&3),
        "the per-kind limit is exact"
    );
    assert_eq!(per_kind.get(TEST_EFFECT), Some(&6));
    assert_eq!(per_kind.get(TEST_PEER), Some(&6));
    assert_eq!(
        count(
            &mut conn,
            "FROM public.async_job WHERE kind = 'TEST_LIMITED' AND status = 'RUNNING'"
        ),
        3
    );
    assert_eq!(count(&mut conn, "FROM public.async_job_attempt"), 15);
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn claims_order_by_priority_within_a_kind() {
    let (_guard, mut conn) = setup();
    let mut names = std::collections::HashMap::new();
    for (name, priority) in [("a", 0), ("b", 5), ("c", -3), ("d", 5)] {
        let created = create(
            &mut conn,
            NewAsyncJob {
                priority,
                ..new_job(TEST_EFFECT, name, &[&key(NS_CS)])
            },
        );
        names.insert(created.async_job_id, name);
    }
    let order: Vec<&str> = (0..4)
        .map(|_| {
            let ids = claim_max(&mut conn, &[TEST_EFFECT], 1);
            assert_eq!(ids.len(), 1);
            names[&ids[0]]
        })
        .collect();
    assert_eq!(order, ["b", "d", "a", "c"]);
}

#[test]
fn claims_respect_available_at() {
    let (_guard, mut conn) = setup();
    let later = create(&mut conn, new_job(TEST_EFFECT, "later", &[&key(NS_CS)]));
    exec(
        &mut conn,
        &format!(
            "UPDATE public.async_job SET available_at = clock_timestamp() + interval '1 hour' \
              WHERE async_job_id = '{}'",
            later.async_job_id
        ),
    );
    assert!(claim_max(&mut conn, &[TEST_EFFECT], 50).is_empty());
    assert_eq!(job(&mut conn, later.async_job_id).attempt_count, 0);
    make_available(&mut conn, later.async_job_id);
    assert_eq!(
        claim_max(&mut conn, &[TEST_EFFECT], 50),
        vec![later.async_job_id]
    );
}

#[test]
fn claims_cross_kind_fairness_does_not_starve_an_older_job() {
    let (_guard, mut conn) = setup();
    let old = create(&mut conn, new_job(TEST_PEER, "old", &[&key(NS_CS)]));
    let mut flood = Vec::new();
    for i in 0..20 {
        flood.push(
            create(
                &mut conn,
                NewAsyncJob {
                    priority: 100,
                    ..new_job(TEST_EFFECT, &format!("flood-{i}"), &[&key(NS_CS)])
                },
            )
            .async_job_id,
        );
    }
    assert_eq!(
        claim_max(&mut conn, &[TEST_EFFECT, TEST_PEER], 1),
        vec![old.async_job_id]
    );
    // The flood follows in its own order.
    assert_eq!(
        claim_max(&mut conn, &[TEST_EFFECT, TEST_PEER], 2),
        flood[..2].to_vec()
    );
}

#[test]
fn claims_skip_unsupported_and_non_executable_versions_and_leave_them_claimable() {
    let (_guard, mut conn) = setup();
    let versioned = |version: i16, id: &str| NewAsyncJob {
        payload_version: version,
        ..new_job(TEST_VERSIONED, id, &[&key(NS_CS)])
    };
    let v2 = create_earlier(&mut conn, versioned(2, "v2")).async_job_id;
    let v3 = create_earlier(&mut conn, versioned(3, "v3")).async_job_id;
    let versions = |list: &[i16]| ClaimRequest {
        kinds: vec![(TEST_VERSIONED.to_string(), list.to_vec())],
        max_jobs: 50,
        lease_seconds: None,
    };
    // The executor does not support version 2.
    let only_v1 = versions(&[1]);
    assert!(claim_ids(tx(&mut conn, |c| claim_on(
        c,
        &REGISTRY,
        &dissemination(),
        only_v1
    )))
    .is_empty());
    // Version 3 is not executable by this binary.
    let all = versions(&[1, 2, 3]);
    assert_eq!(
        claim_ids(tx(&mut conn, |c| claim_on(
            c,
            &REGISTRY,
            &dissemination(),
            all
        ))),
        vec![v2]
    );
    let skipped = job(&mut conn, v3);
    assert_eq!(
        (skipped.status, skipped.attempt_count),
        (AsyncJobStatus::Pending, 0)
    );
    // It stays claimable for a binary that executes it.
    let all = versions(&[1, 2, 3]);
    assert_eq!(
        claim_ids(tx(&mut conn, |c| claim_on(
            c,
            &EARLIER_REGISTRY,
            &dissemination(),
            all
        ))),
        vec![v3]
    );
}

#[test]
fn claims_report_unknown_kinds_and_still_claim_known_ones() {
    let (_guard, mut conn) = setup();
    let known = create(&mut conn, new_job(TEST_EFFECT, "known", &[&key(NS_CS)]));
    let result = tx(&mut conn, |c| {
        claim_on(
            c,
            &REGISTRY,
            &dissemination(),
            request(&[TEST_EFFECT, "TEST_NOPE", "TEST_NOPE"], 50),
        )
    })
    .unwrap();
    assert_eq!(result.skipped_kinds, ["TEST_NOPE"]);
    assert_eq!(result.jobs.len(), 1);
    assert_eq!(result.jobs[0].async_job_id, known.async_job_id);
    // A claim naming only unknown kinds recovers nothing and writes nothing.
    expire_lease(&mut conn, known.async_job_id);
    let polls = count(&mut conn, "FROM public.async_executor_poll");
    let result = tx(&mut conn, |c| {
        claim_on(
            c,
            &REGISTRY,
            &executor_named("other"),
            request(&["TEST_NOPE"], 50),
        )
    })
    .unwrap();
    assert!(result.jobs.is_empty());
    assert_eq!(result.skipped_kinds, ["TEST_NOPE"]);
    assert_eq!(
        status(&mut conn, known.async_job_id),
        AsyncJobStatus::Running
    );
    assert_eq!(count(&mut conn, "FROM public.async_executor_poll"), polls);
}

#[test]
fn claims_filter_by_risk_capability_revision_mode_and_domain() {
    let (_guard, mut conn) = setup();
    let effect = create(
        &mut conn,
        new_job(TEST_EFFECT, "auth-effect", &[&key(NS_CS)]),
    );
    let untrusted = create(
        &mut conn,
        new_job(TEST_UNTRUSTED, "auth-untrusted", &[&key(NS_CS)]),
    );
    let capability = create(
        &mut conn,
        new_job(TEST_CAPABILITY, "auth-cap", &[&key(NS_CS)]),
    );
    let revision = create(&mut conn, revision_job("auth-rev", &key(NS_RB), 1, "f"));
    let read = create(&mut conn, new_job(TEST_READ, "auth-read", &[]));
    let refuse = |conn: &mut PgConnection, executor: Executor, kinds: &[&str]| {
        assert_code(
            tx(conn, |c| {
                claim_on(c, &REGISTRY, &executor, request(kinds, 50))
            }),
            "ASYNC_JOB_KIND_NOT_AUTHORIZED",
        );
    };
    refuse(&mut conn, dissemination(), &[TEST_UNTRUSTED]);
    refuse(
        &mut conn,
        Executor {
            capabilities: Vec::new(),
            ..dissemination()
        },
        &[TEST_CAPABILITY],
    );
    refuse(
        &mut conn,
        Executor {
            revision_modes: vec![RevisionMode::CurrentState],
            ..dissemination()
        },
        &[TEST_REVISION],
    );
    refuse(&mut conn, thoth_worker(), &[TEST_EFFECT]);
    refuse(&mut conn, dissemination(), &[TEST_READ]);
    // No partial authorization: nothing of the authorized kind is claimed.
    refuse(&mut conn, dissemination(), &[TEST_EFFECT, TEST_UNTRUSTED]);
    assert_eq!(
        status(&mut conn, effect.async_job_id),
        AsyncJobStatus::Pending
    );
    assert_eq!(count(&mut conn, "FROM public.async_executor_poll"), 0);
    // Matching executors claim.
    let untrusted_executor = Executor {
        risk_class: RiskClass::UntrustedContent,
        ..dissemination()
    };
    assert_eq!(
        claim_ids(tx(&mut conn, |c| {
            claim_on(
                c,
                &REGISTRY,
                &untrusted_executor,
                request(&[TEST_UNTRUSTED], 50),
            )
        })),
        vec![untrusted.async_job_id]
    );
    assert_eq!(
        claim_max(
            &mut conn,
            &[TEST_EFFECT, TEST_CAPABILITY, TEST_REVISION],
            50
        )
        .len(),
        3
    );
    assert_eq!(
        status(&mut conn, capability.async_job_id),
        AsyncJobStatus::Running
    );
    assert_eq!(
        status(&mut conn, revision.async_job_id),
        AsyncJobStatus::Running
    );
    assert_eq!(
        claim_ids(tx(&mut conn, |c| {
            claim_on(c, &REGISTRY, &thoth_worker(), request(&[TEST_READ], 50))
        })),
        vec![read.async_job_id]
    );
}

#[test]
fn claims_whose_candidates_cross_targets_finish_without_waiting_or_deadlock() {
    let (_guard, mut conn) = setup();
    let (x, y) = (key(NS_CS), key(NS_CS));
    let a1 = create(&mut conn, new_job(TEST_EFFECT, "a1", &[&x])).async_job_id;
    let a2 = create(&mut conn, new_job(TEST_EFFECT, "a2", &[&y])).async_job_id;
    let b1 = create(&mut conn, new_job(TEST_PEER, "b1", &[&y])).async_job_id;
    let b2 = create(&mut conn, new_job(TEST_PEER, "b2", &[&x])).async_job_id;
    let before = deadlocks(&mut conn);
    // Any lock wait fails a step after 100 ms instead of passing silently.
    let a = Session::open_with("SET LOCAL lock_timeout = '100ms'");
    let b = Session::open_with("SET LOCAL lock_timeout = '100ms'");
    let claim_kind = |kind: &'static str, name: &'static str| {
        move |c: &mut PgConnection| {
            claim_ids(claim_on(
                c,
                &REGISTRY,
                &executor_named(name),
                request(&[kind], 1),
            ))
        }
    };
    assert_eq!(a.run(claim_kind(TEST_EFFECT, "a")), vec![a1]); // A takes x
    assert_eq!(b.run(claim_kind(TEST_PEER, "b")), vec![b1]); // B takes y
    assert!(a.run(claim_kind(TEST_EFFECT, "a")).is_empty()); // A wants y: skipped
    assert!(b.run(claim_kind(TEST_PEER, "b")).is_empty()); // B wants x: skipped
    a.commit();
    b.commit();
    assert_eq!(holds(&mut conn, a1), vec![x.clone()]);
    assert_eq!(holds(&mut conn, b1), vec![y.clone()]);
    assert_eq!(status(&mut conn, a2), AsyncJobStatus::Pending);
    assert_eq!(status(&mut conn, b2), AsyncJobStatus::Pending);

    // The same crossing released together: both finish, each key has one
    // holder.
    let (p, q) = (key(NS_CS), key(NS_CS));
    for (kind, name, first, second) in [
        (TEST_EFFECT, "c1", &p, &q),
        (TEST_EFFECT, "c2", &q, &p),
        (TEST_PEER, "d1", &q, &p),
        (TEST_PEER, "d2", &p, &q),
    ] {
        create(
            &mut conn,
            new_job(kind, name, &[first.as_str(), second.as_str()]),
        );
    }
    let tasks: Vec<Task<Vec<Uuid>>> = [TEST_EFFECT, TEST_PEER]
        .into_iter()
        .map(|kind| {
            let task: Task<Vec<Uuid>> = Box::new(move |c: &mut PgConnection| {
                claim_ids(tx(c, |c| {
                    claim_on(c, &REGISTRY, &executor_named(kind), request(&[kind], 50))
                }))
            });
            task
        })
        .collect();
    let claimed: usize = run_together(tasks).into_iter().map(|ids| ids.len()).sum();
    assert_eq!(
        claimed, 1,
        "the two crossing jobs share both keys, so one runs"
    );
    assert_eq!(
        count(
            &mut conn,
            &format!("FROM public.async_target_hold WHERE target_key IN ('{p}', '{q}')")
        ),
        2
    );
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn claims_recovering_an_expired_holder_while_another_wants_its_key_finish_without_deadlock() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let (holder, _claim) = running(&mut conn, new_job(TEST_EFFECT, "holder", &[&x]));
    let wanting = create(&mut conn, new_job(TEST_PEER, "wanting", &[&x])).async_job_id;
    expire_lease(&mut conn, holder.async_job_id);
    let before = deadlocks(&mut conn);
    let a = Session::open_with("SET LOCAL lock_timeout = '100ms'");
    let b = Session::open_with("SET LOCAL lock_timeout = '100ms'");
    // A's claim recovers the holder (T12 releases x, uncommitted).
    let a_claimed = a.run(|c| {
        claim_ids(claim_on(
            c,
            &REGISTRY,
            &executor_named("a"),
            request(&[TEST_EFFECT], 50),
        ))
    });
    assert!(
        a_claimed.is_empty(),
        "the recovered holder waits for its backoff"
    );
    // B wants x: it still sees the committed hold, so it skips at once.
    let b_claim = |c: &mut PgConnection| {
        claim_ids(claim_on(
            c,
            &REGISTRY,
            &executor_named("b"),
            request(&[TEST_PEER], 50),
        ))
    };
    assert!(b.run(b_claim).is_empty());
    a.commit();
    // Once A commits, B takes the released key.
    assert_eq!(b.run(b_claim), vec![wanting]);
    b.commit();
    assert_eq!(
        status(&mut conn, holder.async_job_id),
        AsyncJobStatus::RetryScheduled
    );
    assert_eq!(holds(&mut conn, wanting), vec![x]);

    // Released together by a barrier, in a fresh pair.
    let z = key(NS_CS);
    let (other_holder, _claim) = running(&mut conn, new_job(TEST_EFFECT, "holder-2", &[&z]));
    let other_wanting = create(&mut conn, new_job(TEST_PEER, "wanting-2", &[&z])).async_job_id;
    expire_lease(&mut conn, other_holder.async_job_id);
    let tasks: Vec<Task<ThothResult<ClaimResult>>> = vec![
        Box::new(|c: &mut PgConnection| {
            tx(c, |c| {
                claim_on(
                    c,
                    &REGISTRY,
                    &executor_named("a"),
                    request(&[TEST_EFFECT], 50),
                )
            })
        }),
        Box::new(|c: &mut PgConnection| {
            tx(c, |c| {
                claim_on(
                    c,
                    &REGISTRY,
                    &executor_named("b"),
                    request(&[TEST_PEER], 50),
                )
            })
        }),
    ];
    for result in run_together(tasks) {
        result.expect("both claims finish");
    }
    assert_eq!(
        status(&mut conn, other_holder.async_job_id),
        AsyncJobStatus::RetryScheduled
    );
    // The wanting job either took z or is still claimable.
    let wanting_status = status(&mut conn, other_wanting);
    assert!(matches!(
        wanting_status,
        AsyncJobStatus::Running | AsyncJobStatus::Pending
    ));
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn claims_skip_a_key_whose_hold_lock_is_taken_without_entering_a_lock_wait() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let skipped = create(&mut conn, new_job(TEST_EFFECT, "skipped", &[&x])).async_job_id;
    let before = deadlocks(&mut conn);
    let holder = Session::open();
    let lock_sql = format!("SELECT pg_advisory_xact_lock({NS_HOLD}, hashtext('{x}'))");
    holder.run(move |c| exec(c, &lock_sql));
    // A short statement timeout: a lock wait would fail the claim.
    let claimer = Session::open_with("SET LOCAL statement_timeout = '500ms'");
    let result =
        claimer.run(|c| claim_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50)));
    assert!(result.expect("the claim never waits").jobs.is_empty());
    // pg_locks: the claimer waits for nothing and took no hold lock.
    assert!(!is_blocked(&mut conn, claimer.pid));
    assert_eq!(
        count(
            &mut conn,
            &format!(
                "FROM pg_locks WHERE pid = {} AND locktype = 'advisory' \
                   AND classid = {NS_HOLD}::oid",
                claimer.pid
            )
        ),
        0
    );
    claimer.commit();
    let untouched = job(&mut conn, skipped);
    assert_eq!(
        (untouched.status, untouched.attempt_count),
        (AsyncJobStatus::Pending, 0)
    );
    holder.commit();
    assert_eq!(claim_max(&mut conn, &[TEST_EFFECT], 50), vec![skipped]);
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn claims_report_a_blocked_job_with_its_holder() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let (holder, claim_h) = running(&mut conn, new_job(TEST_EFFECT, "holder", &[&x]));
    let blocked = create(&mut conn, new_job(TEST_PEER, "blocked", &[&x])).async_job_id;
    assert!(claim_max(&mut conn, &[TEST_PEER], 50).is_empty());
    let peer = signals_of(&mut conn, TEST_PEER);
    assert_eq!(peer.blocked_count, 1);
    assert_eq!(
        peer.blocked,
        vec![BlockedJob {
            async_job_id: blocked,
            target_key: x.clone(),
            holder_job_id: holder.async_job_id,
            holder_kind: TEST_EFFECT.to_string(),
        }]
    );
    do_release(&mut conn, &claim_h).unwrap();
    assert_eq!(signals_of(&mut conn, TEST_PEER).blocked_count, 0);
    assert_eq!(claim_max(&mut conn, &[TEST_PEER], 50), vec![blocked]);
}

// ---------------------------------------------------------------------------
// Leases and tokens (E1)
// ---------------------------------------------------------------------------

#[test]
fn leases_renew_needs_the_current_token_and_a_live_lease_and_stops_at_the_runtime_deadline() {
    let (_guard, mut conn) = setup();
    let (job_row, claim) = running(&mut conn, new_job(TEST_EFFECT, "renew", &[&key(NS_CS)]));
    let id = job_row.async_job_id;
    let renewed_for = |conn: &mut PgConnection, requested: Option<i32>| -> i64 {
        let renewal = tx(conn, |c| renew_on(c, &REGISTRY, &claim, requested)).unwrap();
        let heartbeat = attempts(conn, id)[0].last_heartbeat_at;
        (renewal.lease_expires_at - heartbeat).num_seconds()
    };
    assert_eq!(renewed_for(&mut conn, None), 60);
    assert_eq!(renewed_for(&mut conn, Some(1000)), 300);
    assert_eq!(renewed_for(&mut conn, Some(1)), 15);
    assert_eq!(renewed_for(&mut conn, Some(100)), 100);
    // Never past the runtime deadline.
    exec(
        &mut conn,
        &format!(
            "UPDATE public.async_job \
                SET attempt_runtime_deadline = clock_timestamp() + interval '20 seconds' \
              WHERE async_job_id = '{id}'"
        ),
    );
    let renewal = tx(&mut conn, |c| renew_on(c, &REGISTRY, &claim, Some(300))).unwrap();
    assert_eq!(renewal.lease_expires_at, renewal.attempt_runtime_deadline);
    assert_eq!(
        job(&mut conn, id).lease_expires_at,
        Some(renewal.attempt_runtime_deadline)
    );
    // Only with the current token and an unexpired lease.
    let stale = WorkerClaim {
        claim_token: Uuid::new_v4(),
        ..claim
    };
    assert_code(do_renew(&mut conn, &stale), STALE);
    expire_lease(&mut conn, id);
    assert_code(do_renew(&mut conn, &claim), "ASYNC_JOB_LEASE_EXPIRED");
}

#[test]
fn leases_the_lease_rule_of_each_operation() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // Refused with the lease expired: renew and mark effect started.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "rule-a", &[&key(NS_CS)]));
    expire_lease(c, a.async_job_id);
    assert_code(do_renew(c, &claim), "ASYNC_JOB_LEASE_EXPIRED");
    assert_code(do_start(c, &claim), "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT");
    // Accepted, the token being the fence: read target state and release.
    assert_eq!(do_read_state(c, &claim).unwrap().len(), 1);
    assert_eq!(
        do_release(c, &claim).unwrap().status,
        AsyncJobStatus::Pending
    );
    // Mark effect confirmed and complete.
    let target = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "rule-b", &[&target]));
    expire_lease(c, b.async_job_id);
    do_confirm(c, &claim).unwrap();
    assert!(matches!(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::EffectApplied,
                vec![cs_entry(&target, "fp")]
            )
        ),
        Ok(CompleteOutcome::Succeeded(_))
    ));
    // Fail.
    let (d, claim) = running(c, new_job(TEST_EFFECT, "rule-d", &[&key(NS_CS)]));
    expire_lease(c, d.async_job_id);
    assert_eq!(
        do_fail(c, &claim, fail_req(FailureClass::Permanent))
            .unwrap()
            .status,
        AsyncJobStatus::Failed
    );
    // Wait.
    let (e, claim) = running(c, new_job(TEST_EFFECT, "rule-e", &[&key(NS_CS)]));
    expire_lease(c, e.async_job_id);
    let at = resume_in(c, 60);
    assert!(matches!(
        do_wait(c, &claim, "{}", at),
        Ok(WaitOutcome::Waiting(_))
    ));
    // Resolve reconciliation.
    let f = rr_started(c, new_job(TEST_EFFECT, "rule-f", &[&key(NS_CS)]));
    let recon = claim_recon(c, &f);
    expire_lease(c, f.async_job_id);
    assert_eq!(
        do_resolve(c, &recon, resolution(ReconcilerOutcome::StillRequired))
            .unwrap()
            .status,
        AsyncJobStatus::ReconciliationRequired
    );
}

#[test]
fn leases_stale_tokens_are_refused_by_every_token_fenced_write() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let target = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "stale", &[&target]));
    let stale = WorkerClaim {
        claim_token: Uuid::new_v4(),
        ..claim
    };
    let before = job(c, a.async_job_id);
    let at = resume_in(c, 60);
    assert_code(do_renew(c, &stale), STALE);
    assert_code(do_start(c, &stale), STALE);
    assert_code(do_confirm(c, &stale), STALE);
    assert_code(do_wait(c, &stale, "{}", at), STALE);
    assert_code(
        do_complete(
            c,
            &stale,
            completion(CompletionOutcome::NoOp, vec![cs_entry(&target, "fp")]),
        ),
        STALE,
    );
    assert_code(do_fail(c, &stale, fail_req(FailureClass::Permanent)), STALE);
    assert_code(do_release(c, &stale), STALE);
    assert_code(do_read_state(c, &stale), STALE);
    let missing = WorkerClaim {
        async_job_id: Uuid::new_v4(),
        ..claim
    };
    assert_code(do_renew(c, &missing), STALE);
    assert_eq!(
        job(c, a.async_job_id),
        before,
        "a stale write changes nothing"
    );
    assert_live(c, &claim);
    let r = rr_started(c, new_job(TEST_EFFECT, "stale-recon", &[&key(NS_CS)]));
    let recon = claim_recon(c, &r);
    let stale_recon = WorkerClaim {
        claim_token: Uuid::new_v4(),
        ..recon
    };
    assert_code(
        do_resolve(
            c,
            &stale_recon,
            resolution(ReconcilerOutcome::StillRequired),
        ),
        STALE,
    );
    assert_live(c, &recon);
}

#[test]
fn leases_a_superseded_worker_cannot_write_after_recovery_or_cancellation() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let target = key(NS_CS);
    let (a, old) = running(c, new_job(TEST_EFFECT, "superseded", &[&target]));
    expire_lease(c, a.async_job_id);
    do_recover(c);
    make_available(c, a.async_job_id);
    let current = claim_job(c, &a);
    let at = resume_in(c, 60);
    assert_code(do_renew(c, &old), STALE);
    assert_code(do_start(c, &old), STALE);
    assert_code(do_confirm(c, &old), STALE);
    assert_code(do_wait(c, &old, "{}", at), STALE);
    assert_code(
        do_complete(
            c,
            &old,
            completion(CompletionOutcome::NoOp, vec![cs_entry(&target, "fp")]),
        ),
        STALE,
    );
    assert_code(do_fail(c, &old, fail_req(FailureClass::Permanent)), STALE);
    assert_code(do_release(c, &old), STALE);
    assert_code(do_read_state(c, &old), STALE);
    assert_live(c, &current);
    // Cancellation invalidates the token.
    let (b, claim) = running(c, new_job(TEST_EFFECT, "cancelled", &[&key(NS_CS)]));
    assert_applied(&do_cancel(c, b.async_job_id), "CANCELLED");
    assert_code(do_renew(c, &claim), STALE);
    assert_code(do_start(c, &claim), STALE);
    assert_code(do_fail(c, &claim, fail_req(FailureClass::Permanent)), STALE);
}

#[test]
fn leases_recovery_racing_a_late_completion_exactly_one_wins() {
    let (_guard, mut conn) = setup();
    let before = deadlocks(&mut conn);
    // Recovery first: the late completion waits for the row, then is stale.
    let x = key(NS_CS);
    let (a, claim) = started(&mut conn, new_job(TEST_EFFECT, "race-a", &[&x]));
    expire_lease(&mut conn, a.async_job_id);
    let recovery = Session::open();
    let report = recovery
        .run(|c| recover_on(c, &REGISTRY, None, 1000))
        .unwrap();
    assert_eq!(report.recovered, vec![a.async_job_id]);
    let entries = vec![cs_entry(&x, "fp")];
    let (pid, late) = spawn_tx(move |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(CompletionOutcome::EffectApplied, entries),
        )
    });
    wait_until_blocked(&mut conn, pid);
    recovery.commit();
    assert_code(late.join().unwrap(), STALE);
    assert_eq!(
        status(&mut conn, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert!(state(&mut conn, &x).is_none());

    // Completion first: recovery skips the locked row; the completion wins.
    let y = key(NS_CS);
    let (b, claim) = started(&mut conn, new_job(TEST_EFFECT, "race-b", &[&y]));
    expire_lease(&mut conn, b.async_job_id);
    let worker = Session::open();
    let entries = vec![cs_entry(&y, "fp")];
    let outcome = worker.run(move |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(CompletionOutcome::EffectApplied, entries),
        )
    });
    assert!(matches!(outcome, Ok(CompleteOutcome::Succeeded(_))));
    let report = do_recover(&mut conn);
    assert!(report.recovered.is_empty() && report.failed.is_empty());
    worker.commit();
    assert_eq!(status(&mut conn, b.async_job_id), AsyncJobStatus::Succeeded);
    assert!(do_recover(&mut conn).recovered.is_empty());
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn leases_one_token_one_attempt() {
    let (_guard, mut conn) = setup();
    let (a, first) = running(&mut conn, new_job(TEST_EFFECT, "token", &[&key(NS_CS)]));
    let id = a.async_job_id;
    do_release(&mut conn, &first).unwrap();
    let second = claim_job(&mut conn, &a);
    assert_ne!(first.claim_token, second.claim_token);
    let rows = attempts(&mut conn, id);
    assert_eq!(rows.len(), 2);
    assert_eq!(
        attempt_token(&mut conn, rows[0].async_job_attempt_id),
        Some(first.claim_token.to_string())
    );
    assert_eq!(
        attempt_token(&mut conn, rows[1].async_job_attempt_id),
        Some(second.claim_token.to_string())
    );
    assert_eq!(
        text(
            &mut conn,
            &format!("SELECT claim_token::text AS value FROM public.async_job WHERE async_job_id = '{id}'")
        ),
        Some(second.claim_token.to_string())
    );
    // A token belongs to one attempt for ever.
    assert_eq!(
        refused_by(
            &mut conn,
            &format!(
                "INSERT INTO public.async_job_attempt (async_job_id, attempt_number, purpose, \
                     claim_token, claimed_by, executor_principal, started_at, last_heartbeat_at, \
                     phase_at_start, finished_at, result, job_status_after, outcome_code) \
                 VALUES ('{id}', 9, 'EXECUTION', '{}', 'w', 'p', now(), now(), 'PRE_WRITE', \
                     now(), 'RELEASED', 'PENDING', 'RELEASED')",
                first.claim_token
            )
        ),
        "async_job_attempt_claim_token_key"
    );
}

#[test]
fn leases_no_read_returns_the_token() {
    let (_guard, mut conn) = setup();
    let target = key(NS_CS);
    let other = create(&mut conn, new_job(TEST_PEER, "secret-2", &[&key(NS_CS)]));
    let claimed = claim(&mut conn, &dissemination(), &[TEST_PEER]);
    assert_eq!(claimed[0].async_job_id, other.async_job_id);
    let (a, claim) = running(&mut conn, new_job(TEST_EFFECT, "secret", &[&target]));
    let tokens = [
        claim.claim_token.to_string(),
        claimed[0].claim_token.to_string(),
    ];
    let id = a.async_job_id;
    let reads = [
        serde_json::to_string(&get_job(&mut conn, id).unwrap()).unwrap(),
        format!("{:?}", get_job(&mut conn, id).unwrap()),
        serde_json::to_string(&job_attempts(&mut conn, id).unwrap()).unwrap(),
        serde_json::to_string(&job_transitions(&mut conn, id).unwrap()).unwrap(),
        serde_json::to_string(
            &list_jobs(&mut conn, &AsyncJobFilter::default(), None, 100).unwrap(),
        )
        .unwrap(),
        serde_json::to_string(&kind_signals(&mut conn, &REGISTRY).unwrap()).unwrap(),
        serde_json::to_string(&executor_activity(&mut conn).unwrap()).unwrap(),
        serde_json::to_string(&do_read_state(&mut conn, &claim).unwrap()).unwrap(),
        format!("{:?}", do_renew(&mut conn, &claim).unwrap()),
        format!("{claim:?}"),
        format!("{:?}", claimed[0]),
        format!("{:?}", do_cancel(&mut conn, id)),
    ];
    for read in &reads {
        for token in &tokens {
            assert!(
                !read.contains(token.as_str()),
                "a read returned a token: {read}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Effect phases and outcomes (section 9.3)
// ---------------------------------------------------------------------------

/// Claim every ready job of `kinds` as the dissemination executor (a name
/// that a local `claim` binding cannot shadow).
fn dissemination_claim(conn: &mut PgConnection, kinds: &[&str]) -> Vec<ClaimedJob> {
    claim(conn, &dissemination(), kinds)
}

/// Put a running job's lease and runtime deadline `lease` and `runtime`
/// seconds after the database clock (a same-status update the triggers
/// allow).
fn set_windows(conn: &mut PgConnection, id: Uuid, lease: i64, runtime: i64) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job \
                SET lease_expires_at = clock_timestamp() + interval '{lease} seconds', \
                    attempt_runtime_deadline = clock_timestamp() + interval '{runtime} seconds' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Wait once (T10), make the job due and claim it again: RUNNING with its
/// waiting deadline and checkpoint stored.
fn resumed(conn: &mut PgConnection, job: &AsyncJob, claim: &WorkerClaim) -> WorkerClaim {
    let resume_at = resume_in(conn, 10);
    let outcome = do_wait(conn, claim, r#"{"step": 1}"#, resume_at).expect("T10");
    assert!(matches!(outcome, WaitOutcome::Waiting(_)));
    make_available(conn, job.async_job_id);
    claim_job(conn, job)
}

/// The last attempt of a job.
fn last_attempt(conn: &mut PgConnection, id: Uuid) -> AsyncJobAttempt {
    attempts(conn, id).pop().expect("an attempt")
}

/// Section 9.3's backoff for counter `n`, computed independently of the
/// engine: `min(max, base * 2^(n - 1))` saturating, plus
/// `floor(delay * j / 100)` with the spec's jitter over the job's
/// `attempt_count`.
fn expected_backoff(conn: &mut PgConnection, job: &AsyncJob, n: i32) -> i64 {
    let (base, max) = (
        i128::from(job.backoff_base_seconds),
        i128::from(job.backoff_max_seconds),
    );
    let n = n.max(1);
    let delay = if n >= 32 {
        max
    } else {
        (base * (1_i128 << (n - 1))).min(max)
    };
    let j = scalar(
        conn,
        &format!(
            "SELECT (((hashtextextended('{}' || ':' || {}, 0) % 21) + 21) % 21) AS value",
            job.async_job_id, job.attempt_count
        ),
    );
    assert!((0..=20).contains(&j));
    i64::try_from(delay + delay * i128::from(j) / 100).unwrap()
}

#[test]
fn effect_window_is_refused_on_the_lease_and_on_the_runtime_deadline() {
    let (_guard, mut conn) = setup();
    let (a, claim) = running(&mut conn, new_job(TEST_EFFECT, "window", &[&key(NS_CS)]));
    let id = a.async_job_id;
    // The kind's effect window is 30 s.
    set_windows(&mut conn, id, 20, 600);
    assert_code(
        do_start(&mut conn, &claim),
        "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
    );
    set_windows(&mut conn, id, 120, 20);
    assert_code(
        do_start(&mut conn, &claim),
        "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
    );
    assert_eq!(job(&mut conn, id).effect_phase, AsyncEffectPhase::PreWrite);
    assert!(last_attempt(&mut conn, id).effect_started_at.is_none());
    assert_live(&mut conn, &claim);
    set_windows(&mut conn, id, 120, 120);
    let row = do_start(&mut conn, &claim).unwrap();
    assert_eq!(row.effect_phase, AsyncEffectPhase::EffectStarted);
    assert!(last_attempt(&mut conn, id).effect_started_at.is_some());
}

#[test]
fn effect_crash_before_effect_started_counts_as_a_pre_write_failure_then_retries_or_fails() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let a = create(&mut conn, new_job(TEST_EFFECT, "crash-pre", &[&x]));
    let id = a.async_job_id;
    // retry_budget 3: two lease losses schedule a retry, the third fails.
    for n in 1..=3_i16 {
        claim_job(&mut conn, &a);
        assert_eq!(holds(&mut conn, id), vec![x.clone()]);
        expire_lease(&mut conn, id);
        assert_eq!(do_recover(&mut conn).recovered, vec![id]);
        let row = job(&mut conn, id);
        let last = last_attempt(&mut conn, id);
        assert_eq!(row.prewrite_failure_count, n);
        assert!(row.claimed_by.is_none());
        assert!(holds(&mut conn, id).is_empty());
        assert_eq!(last.result, Some(AsyncAttemptResult::LeaseExpired));
        if n < 3 {
            assert_eq!(row.status, AsyncJobStatus::RetryScheduled);
            assert_eq!(row.last_outcome_code.as_deref(), Some("LEASE_EXPIRED"));
            make_available(&mut conn, id);
        } else {
            assert_eq!(row.status, AsyncJobStatus::Failed);
            assert_eq!(
                row.last_outcome_code.as_deref(),
                Some("RETRY_BUDGET_EXHAUSTED")
            );
            assert_eq!(last.outcome_code.as_deref(), Some("RETRY_BUDGET_EXHAUSTED"));
            assert_eq!(last.job_status_after, Some(AsyncJobStatus::Failed));
        }
    }
    assert_eq!(
        causes(&mut conn, id),
        [
            "T1_CREATE",
            "T2_CLAIM",
            "T12_LEASE_EXPIRED",
            "T2_CLAIM",
            "T12_LEASE_EXPIRED",
            "T2_CLAIM",
            "T12_LEASE_EXPIRED"
        ]
    );
}

#[test]
fn effect_crash_after_effect_started_goes_to_reconciliation() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let (a, claim) = started(&mut conn, new_job(TEST_EFFECT, "crash-post", &[&x]));
    let id = a.async_job_id;
    expire_lease(&mut conn, id);
    assert_eq!(do_recover(&mut conn).recovered, vec![id]);
    let row = job(&mut conn, id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(row.effect_phase, AsyncEffectPhase::EffectStarted);
    assert_eq!(row.prewrite_failure_count, 0);
    assert!(row.claimed_by.is_none() && row.lease_expires_at.is_none());
    assert!(row.available_at <= db_now(&mut conn));
    assert_eq!(holds(&mut conn, id), vec![x]);
    let last = last_attempt(&mut conn, id);
    assert_eq!(last.result, Some(AsyncAttemptResult::LeaseExpired));
    assert_eq!(
        last.job_status_after,
        Some(AsyncJobStatus::ReconciliationRequired)
    );
    assert_eq!(last.outcome_code.as_deref(), Some("LEASE_EXPIRED"));
    assert_code(do_renew(&mut conn, &claim), STALE);
}

#[test]
fn effect_confirmed_is_resumable_with_a_checkpoint_and_otherwise_goes_to_reconciliation() {
    let (_guard, mut conn) = setup();

    // Resumable kind with a checkpoint: WAITING, due after the poll interval.
    let p = key(NS_POLL);
    let a = create(&mut conn, new_job(TEST_POLL, "resumable", &[&p]));
    let claim = claim_job(&mut conn, &a);
    do_start(&mut conn, &claim).unwrap();
    do_confirm(&mut conn, &claim).unwrap();
    let claim = resumed(&mut conn, &a, &claim);
    let before = job(&mut conn, a.async_job_id);
    assert_eq!(before.effect_phase, AsyncEffectPhase::EffectConfirmed);
    assert!(before.checkpoint.is_some());
    expire_lease(&mut conn, a.async_job_id);
    assert_eq!(do_recover(&mut conn).recovered, vec![a.async_job_id]);
    let row = job(&mut conn, a.async_job_id);
    let last = last_attempt(&mut conn, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::Waiting);
    assert_eq!(row.waiting_deadline, before.waiting_deadline);
    assert_eq!(
        Some(row.available_at),
        last.finished_at.map(|at| at + Duration::seconds(5))
    );
    assert_eq!(last.result, Some(AsyncAttemptResult::LeaseExpired));
    assert_eq!(last.job_status_after, Some(AsyncJobStatus::Waiting));
    assert_eq!(holds(&mut conn, a.async_job_id), vec![p]);
    assert_code(do_renew(&mut conn, &claim), STALE);

    // Resumable kind without a checkpoint: RR.
    let b = create(
        &mut conn,
        new_job(TEST_POLL, "no-checkpoint", &[&key(NS_POLL)]),
    );
    let claim = claim_job(&mut conn, &b);
    do_start(&mut conn, &claim).unwrap();
    do_confirm(&mut conn, &claim).unwrap();
    expire_lease(&mut conn, b.async_job_id);
    assert_eq!(do_recover(&mut conn).recovered, vec![b.async_job_id]);
    assert_eq!(
        status(&mut conn, b.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );

    // Resumable kind with a checkpoint whose waiting deadline has passed: RR.
    let c = create(
        &mut conn,
        new_job(TEST_POLL, "deadline-passed", &[&key(NS_POLL)]),
    );
    let claim = claim_job(&mut conn, &c);
    do_start(&mut conn, &claim).unwrap();
    do_confirm(&mut conn, &claim).unwrap();
    resumed(&mut conn, &c, &claim);
    expire_waiting_deadline(&mut conn, c.async_job_id, 1);
    expire_lease(&mut conn, c.async_job_id);
    assert_eq!(do_recover(&mut conn).recovered, vec![c.async_job_id]);
    assert_eq!(
        status(&mut conn, c.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );

    // A kind that is not resumable, even with a checkpoint: RR.
    let x = key(NS_CS);
    let (d, claim) = started(&mut conn, new_job(TEST_EFFECT, "not-resumable", &[&x]));
    do_confirm(&mut conn, &claim).unwrap();
    resumed(&mut conn, &d, &claim);
    assert!(job(&mut conn, d.async_job_id).checkpoint.is_some());
    expire_lease(&mut conn, d.async_job_id);
    assert_eq!(do_recover(&mut conn).recovered, vec![d.async_job_id]);
    let row = job(&mut conn, d.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(holds(&mut conn, d.async_job_id), vec![x]);
}

#[test]
fn effect_waiting_atomically_checkpoints_and_releases_the_claim() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let (a, claim) = running(&mut conn, new_job(TEST_EFFECT, "wait", &[&x]));
    let id = a.async_job_id;
    let resume_at = resume_in(&mut conn, 30);
    let request = WaitRequest {
        checkpoint: r#"{"page": 2}"#.to_string(),
        resume_at,
        provider_ref: Some("provider-wait".to_string()),
    };
    let outcome = tx(&mut conn, |c| wait_on(c, &REGISTRY, &claim, request)).unwrap();
    let WaitOutcome::Waiting(row) = outcome else {
        panic!("expected WAITING: {outcome:?}");
    };
    assert_eq!(row.status, AsyncJobStatus::Waiting);
    assert_eq!(row.checkpoint.as_deref(), Some(r#"{"page": 2}"#));
    assert_eq!(row.checkpoint_version, 1);
    assert_eq!(row.provider_ref.as_deref(), Some("provider-wait"));
    assert_eq!(row.available_at, resume_at);
    assert!(row.claimed_by.is_none() && row.lease_expires_at.is_none());
    assert_eq!(
        count(
            &mut conn,
            &format!("FROM public.async_job WHERE async_job_id = '{id}' AND claim_token IS NULL")
        ),
        1
    );
    let last = last_attempt(&mut conn, id);
    assert_eq!(last.result, Some(AsyncAttemptResult::Waiting));
    assert_eq!(last.job_status_after, Some(AsyncJobStatus::Waiting));
    assert_eq!(last.provider_ref.as_deref(), Some("provider-wait"));
    assert_eq!(
        row.waiting_deadline,
        last.finished_at.map(|at| at + Duration::seconds(3600))
    );
    assert_eq!(holds(&mut conn, id), vec![x]);
    assert_code(do_renew(&mut conn, &claim), STALE);
    assert_eq!(causes(&mut conn, id), ["T1_CREATE", "T2_CLAIM", "T10_WAIT"]);
}

#[test]
fn effect_resume_is_a_new_attempt_that_sees_the_checkpoint_and_consumes_no_budget_or_cap() {
    let (_guard, mut conn) = setup();
    let (a, mut claim) = running(&mut conn, new_job(TEST_EFFECT, "resume", &[&key(NS_CS)]));
    let id = a.async_job_id;
    // Six resumes, more than max_execution_claims (4).
    for n in 1..=6_i32 {
        let checkpoint = format!(r#"{{"n": {n}}}"#);
        let resume_at = resume_in(&mut conn, 10);
        do_wait(&mut conn, &claim, &checkpoint, resume_at).unwrap();
        make_available(&mut conn, id);
        let claimed = dissemination_claim(&mut conn, &[TEST_EFFECT]);
        assert_eq!(claimed.len(), 1);
        let resumed = &claimed[0];
        assert_eq!(resumed.async_job_id, id);
        assert_eq!(resumed.checkpoint.as_deref(), Some(checkpoint.as_str()));
        assert_eq!(resumed.checkpoint_version, n);
        assert_eq!(resumed.attempt_number, n + 1);
        let row = job(&mut conn, id);
        assert_eq!(row.execution_claim_count, 1);
        assert_eq!(row.prewrite_failure_count, 0);
        assert_eq!(row.attempt_count, n + 1);
        claim = resumed.worker_claim(ExecutorDomain::Dissemination);
    }
    assert_eq!(attempts(&mut conn, id).len(), 7);
    let outcome = do_complete(
        &mut conn,
        &claim,
        completion(
            CompletionOutcome::NoOp,
            vec![cs_entry(&a.target_keys[0], "fp")],
        ),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
}

#[test]
fn effect_wait_needs_the_minimum_resume_delay() {
    let (_guard, mut conn) = setup();
    let (a, claim) = running(&mut conn, new_job(TEST_EFFECT, "delay", &[&key(NS_CS)]));
    // poll_interval_min_seconds 5.
    let too_soon = resume_in(&mut conn, 4);
    assert_code(
        do_wait(&mut conn, &claim, "{}", too_soon),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_live(&mut conn, &claim);
    assert_eq!(status(&mut conn, a.async_job_id), AsyncJobStatus::Running);
    let resume_at = resume_in(&mut conn, 7);
    let outcome = do_wait(&mut conn, &claim, "{}", resume_at).unwrap();
    assert!(matches!(outcome, WaitOutcome::Waiting(ref row) if row.available_at == resume_at));
    // Not claimable before its resume time.
    assert!(dissemination_claim(&mut conn, &[TEST_EFFECT]).is_empty());
}

#[test]
fn effect_deadline_exhaustion_goes_to_rr_after_start_and_to_failed_before_it() {
    let (_guard, mut conn) = setup();

    // T11 at PRE_WRITE: FAILED, holds released, no new attempt.
    let x = key(NS_CS);
    let (a, claim) = running(&mut conn, new_job(TEST_EFFECT, "t11-pre", &[&x]));
    let resume_at = resume_in(&mut conn, 10);
    do_wait(&mut conn, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(&mut conn, a.async_job_id, 1);
    assert_eq!(do_recover(&mut conn).recovered, vec![a.async_job_id]);
    let row = job(&mut conn, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::Failed);
    assert_eq!(row.last_outcome_code.as_deref(), Some("DEADLINE_EXCEEDED"));
    assert!(holds(&mut conn, a.async_job_id).is_empty());
    assert_eq!(attempts(&mut conn, a.async_job_id).len(), 1);

    // T11 after effect start: RR, holds kept.
    let y = key(NS_CS);
    let (b, claim) = started(&mut conn, new_job(TEST_EFFECT, "t11-post", &[&y]));
    let resume_at = resume_in(&mut conn, 10);
    do_wait(&mut conn, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(&mut conn, b.async_job_id, 1);
    assert_eq!(do_recover(&mut conn).recovered, vec![b.async_job_id]);
    let row = job(&mut conn, b.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(row.last_outcome_code.as_deref(), Some("DEADLINE_EXCEEDED"));
    assert_eq!(holds(&mut conn, b.async_job_id), vec![y]);

    // T10d at PRE_WRITE: `wait` after the stored deadline fails the job.
    let z = key(NS_CS);
    let (c, claim) = running(&mut conn, new_job(TEST_EFFECT, "t10d-pre", &[&z]));
    let claim = resumed(&mut conn, &c, &claim);
    expire_waiting_deadline(&mut conn, c.async_job_id, 1);
    let resume_at = resume_in(&mut conn, 10);
    let outcome = do_wait(&mut conn, &claim, "{}", resume_at).unwrap();
    let WaitOutcome::DeadlineExceeded(row) = outcome else {
        panic!("expected T10d: {outcome:?}");
    };
    assert_eq!(row.status, AsyncJobStatus::Failed);
    assert!(holds(&mut conn, c.async_job_id).is_empty());
    let last = last_attempt(&mut conn, c.async_job_id);
    assert_eq!(last.result, Some(AsyncAttemptResult::PermanentFailure));
    assert_eq!(last.outcome_code.as_deref(), Some("DEADLINE_EXCEEDED"));

    // T10d after effect start: RR, holds kept.
    let w = key(NS_CS);
    let (d, claim) = started(&mut conn, new_job(TEST_EFFECT, "t10d-post", &[&w]));
    let claim = resumed(&mut conn, &d, &claim);
    expire_waiting_deadline(&mut conn, d.async_job_id, 1);
    let resume_at = resume_in(&mut conn, 10);
    let outcome = do_wait(&mut conn, &claim, "{}", resume_at).unwrap();
    let WaitOutcome::DeadlineExceeded(row) = outcome else {
        panic!("expected T10d: {outcome:?}");
    };
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(holds(&mut conn, d.async_job_id), vec![w]);
    let last = last_attempt(&mut conn, d.async_job_id);
    assert_eq!(last.result, Some(AsyncAttemptResult::Indeterminate));
    assert_eq!(last.outcome_code.as_deref(), Some("DEADLINE_EXCEEDED"));
    assert_code(do_renew(&mut conn, &claim), STALE);
}

#[test]
fn effect_failure_after_start_goes_to_reconciliation_whatever_the_class() {
    let (_guard, mut conn) = setup();
    for (n, class, result) in [
        (
            1,
            FailureClass::Indeterminate,
            AsyncAttemptResult::Indeterminate,
        ),
        (
            2,
            FailureClass::Permanent,
            AsyncAttemptResult::PermanentFailure,
        ),
        (
            3,
            FailureClass::RetryablePreWrite,
            AsyncAttemptResult::RetryableFailure,
        ),
    ] {
        let x = key(NS_CS);
        let (a, claim) = started(&mut conn, new_job(TEST_EFFECT, &format!("t9-{n}"), &[&x]));
        let row = do_fail(&mut conn, &claim, fail_req(class)).unwrap();
        assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
        assert_eq!(
            row.last_outcome_code.as_deref(),
            Some("RECONCILIATION_REQUIRED")
        );
        assert_eq!(holds(&mut conn, a.async_job_id), vec![x]);
        let last = last_attempt(&mut conn, a.async_job_id);
        assert_eq!(last.result, Some(result));
        assert_eq!(last.error_code.as_deref(), Some("TEST_FAILURE"));
    }
}

#[test]
fn effect_proven_no_effect_after_start_fails_only_where_the_kind_allows_with_valid_evidence() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let (a, claim) = started(&mut conn, new_job(TEST_EFFECT, "no-effect", &[&x]));
    let id = a.async_job_id;
    for evidence in [
        None,
        Some(r#"{"valid": false}"#),
        Some(r#"{"valid": true}"#),
    ] {
        assert_code(
            do_fail(
                &mut conn,
                &claim,
                no_effect_req(FailureClass::Permanent, evidence),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_live(&mut conn, &claim);
    }
    let row = do_fail(
        &mut conn,
        &claim,
        no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
    )
    .unwrap();
    assert_eq!(row.status, AsyncJobStatus::Failed);
    assert_eq!(row.last_outcome_code.as_deref(), Some("NO_EFFECT_PROVEN"));
    assert!(holds(&mut conn, id).is_empty());
    let last = last_attempt(&mut conn, id);
    assert_eq!(last.result, Some(AsyncAttemptResult::PermanentFailure));
    assert_eq!(last.outcome_code.as_deref(), Some("NO_EFFECT_PROVEN"));
    assert_eq!(
        last.evidence.as_deref(),
        Some(r#"{"valid": true, "providerRef": "absent-1"}"#)
    );
    assert_eq!(
        causes(&mut conn, id).last().unwrap(),
        "T9N_FAIL_NO_EFFECT_PROVEN"
    );

    // A kind without `no_effect_after_start_allowed`: refused, claim live.
    let (b, claim) = started(&mut conn, new_job(TEST_STRICT, "strict", &[&key(NS_CS)]));
    assert_refused(
        do_fail(
            &mut conn,
            &claim,
            no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
        ),
        "KIND_POLICY",
    );
    assert_live(&mut conn, &claim);
    let row = do_fail(&mut conn, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(b.async_job_id, row.async_job_id);
}

#[test]
fn effect_release_loop_is_bounded_by_the_execution_claim_cap() {
    let (_guard, mut conn) = setup();
    let a = create(&mut conn, new_job(TEST_EFFECT, "release", &[&key(NS_CS)]));
    let id = a.async_job_id;
    // max_execution_claims 4.
    for n in 1..=4_i16 {
        let claim = claim_job(&mut conn, &a);
        let row = do_release(&mut conn, &claim).unwrap();
        assert_eq!(row.execution_claim_count, n);
        assert_eq!(row.prewrite_failure_count, 0);
        assert!(holds(&mut conn, id).is_empty());
        let last = last_attempt(&mut conn, id);
        if n < 4 {
            assert_eq!(row.status, AsyncJobStatus::Pending);
            assert!(row.available_at <= db_now(&mut conn));
            assert_eq!(last.result, Some(AsyncAttemptResult::Released));
            assert_eq!(last.outcome_code.as_deref(), Some("RELEASED"));
        } else {
            assert_eq!(row.status, AsyncJobStatus::Failed);
            assert_eq!(row.last_outcome_code.as_deref(), Some("ATTEMPT_CAP"));
            assert_eq!(last.result, Some(AsyncAttemptResult::Released));
            assert_eq!(last.outcome_code.as_deref(), Some("ATTEMPT_CAP"));
        }
    }
    assert!(dissemination_claim(&mut conn, &[TEST_EFFECT]).is_empty());
}

#[test]
fn effect_reconciliation_loop_is_bounded_by_its_cap_then_only_staff_resolves() {
    let (_guard, mut conn) = setup();
    let a = rr_started(&mut conn, new_job(TEST_EFFECT, "recon-cap", &[&key(NS_CS)]));
    let id = a.async_job_id;
    // max_reconciliation_attempts 2.
    for n in 1..=2_i16 {
        let claim = claim_recon(&mut conn, &a);
        let row = do_resolve(
            &mut conn,
            &claim,
            resolution(ReconcilerOutcome::StillRequired),
        )
        .unwrap();
        assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
        assert_eq!(row.reconciliation_attempt_count, n);
        assert!(row.claimed_by.is_none());
        make_available(&mut conn, id);
    }
    let none = tx(&mut conn, |c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50))
    })
    .unwrap();
    assert!(none.jobs.is_empty());
    assert_eq!(
        signals_of(&mut conn, TEST_EFFECT).reconciliation_exhausted_count,
        1
    );
    let result = do_staff(
        &mut conn,
        id,
        StaffResolution {
            evidence: Some(NO_EFFECT_EVIDENCE.to_string()),
            ..staff_res(StaffOutcome::Failed)
        },
    );
    assert_applied(&result, "NO_EFFECT_PROVEN");
    assert_eq!(result.job.status, AsyncJobStatus::Failed);
    assert_eq!(
        signals_of(&mut conn, TEST_EFFECT).reconciliation_exhausted_count,
        0
    );
}

#[test]
fn effect_backoff_values_match_the_formula_for_both_counters() {
    let (_guard, mut conn) = setup();

    // Pre-write failure count: T7 (n = 1), then T12 (n = 2).
    let a = create(
        &mut conn,
        new_job(TEST_EFFECT, "backoff-pre", &[&key(NS_CS)]),
    );
    let id = a.async_job_id;
    let claim = claim_job(&mut conn, &a);
    let row = do_fail(&mut conn, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    let finished = last_attempt(&mut conn, id).finished_at.unwrap();
    let delay = expected_backoff(&mut conn, &row, 1);
    assert_eq!(row.available_at, finished + Duration::seconds(delay));
    make_available(&mut conn, id);
    claim_job(&mut conn, &a);
    expire_lease(&mut conn, id);
    do_recover(&mut conn);
    let row = job(&mut conn, id);
    assert_eq!(row.status, AsyncJobStatus::RetryScheduled);
    let finished = last_attempt(&mut conn, id).finished_at.unwrap();
    let delay = expected_backoff(&mut conn, &row, 2);
    assert_eq!(row.available_at, finished + Duration::seconds(delay));

    // Reconciliation attempt count: T16 STILL_REQUIRED (n = 1), then T17
    // (n = 2).
    let b = rr_started(
        &mut conn,
        new_job(TEST_EFFECT, "backoff-recon", &[&key(NS_CS)]),
    );
    let id = b.async_job_id;
    let claim = claim_recon(&mut conn, &b);
    let row = do_resolve(
        &mut conn,
        &claim,
        resolution(ReconcilerOutcome::StillRequired),
    )
    .unwrap();
    let finished = last_attempt(&mut conn, id).finished_at.unwrap();
    let delay = expected_backoff(&mut conn, &row, 1);
    assert_eq!(row.available_at, finished + Duration::seconds(delay));
    make_available(&mut conn, id);
    claim_recon(&mut conn, &b);
    expire_lease(&mut conn, id);
    do_recover(&mut conn);
    let row = job(&mut conn, id);
    assert_eq!(row.reconciliation_attempt_count, 2);
    let finished = last_attempt(&mut conn, id).finished_at.unwrap();
    let delay = expected_backoff(&mut conn, &row, 2);
    assert_eq!(row.available_at, finished + Duration::seconds(delay));
}

// ---------------------------------------------------------------------------
// Targets and evidence (section 13)
// ---------------------------------------------------------------------------

/// A staff resolution with entries.
fn staff_entries(outcome: StaffOutcome, entries: Vec<EvidenceEntry>) -> StaffResolution {
    StaffResolution {
        entries: Some(entries),
        ..staff_res(outcome)
    }
}

/// A staff resolution with an evidence object.
fn staff_evidence(outcome: StaffOutcome, evidence: &str) -> StaffResolution {
    StaffResolution {
        evidence: Some(evidence.to_string()),
        ..staff_res(outcome)
    }
}

/// An executor resolution with entries.
fn resolution_entries(entries: Vec<EvidenceEntry>) -> ResolveReconciliation {
    ResolveReconciliation {
        entries: Some(entries),
        ..resolution(ReconcilerOutcome::Succeeded)
    }
}

/// An executor resolution with an evidence object.
fn resolution_evidence(outcome: ReconcilerOutcome, evidence: &str) -> ResolveReconciliation {
    ResolveReconciliation {
        evidence: Some(evidence.to_string()),
        ..resolution(outcome)
    }
}

/// The job and attempt that last wrote a key's state.
fn state_writer(conn: &mut PgConnection, key: &str) -> (Option<String>, Option<String>) {
    (
        text(
            conn,
            &format!(
                "SELECT last_job_id::text AS value FROM public.async_effect_target_state \
                  WHERE target_key = '{key}'"
            ),
        ),
        text(
            conn,
            &format!(
                "SELECT last_attempt_id::text AS value FROM public.async_effect_target_state \
                  WHERE target_key = '{key}'"
            ),
        ),
    )
}

fn rb_state(order: i64, fingerprint: &str) -> (Option<i64>, String) {
    (Some(order), fingerprint.to_string())
}

fn applied(conn: &mut PgConnection, key: &str) -> Option<(Option<i64>, String)> {
    state(conn, key).map(|s| (s.applied_order, s.applied_fingerprint))
}

#[test]
fn targets_two_kinds_sharing_a_target_never_overlap_in_running_waiting_or_rr() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "share-a", &[&x]));
    let b = create(c, new_job(TEST_PEER, "share-b", &[&x]));
    // A RUNNING.
    assert!(dissemination_claim(c, &[TEST_PEER]).is_empty());
    // A WAITING after effect start.
    do_start(c, &claim).unwrap();
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert!(dissemination_claim(c, &[TEST_PEER]).is_empty());
    // A RECONCILIATION_REQUIRED.
    make_available(c, a.async_job_id);
    let claim = claim_job(c, &a);
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    assert!(dissemination_claim(c, &[TEST_PEER]).is_empty());
    let blocked = signals_of(c, TEST_PEER).blocked;
    assert_eq!(blocked.len(), 1);
    assert_eq!(
        (blocked[0].async_job_id, blocked[0].holder_job_id),
        (b.async_job_id, a.async_job_id)
    );
    // A resolved: its hold is released and B runs.
    let result = do_staff(
        c,
        a.async_job_id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp-a")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    let claimed = dissemination_claim(c, &[TEST_PEER]);
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].async_job_id, b.async_job_id);
    assert_eq!(holds(c, b.async_job_id), vec![x]);
}

#[test]
fn targets_holds_are_released_at_each_listed_point() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let held = |c: &mut PgConnection, id: Uuid| !holds(c, id).is_empty();

    // T6.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t6", &[&key(NS_CS)]));
    assert!(held(c, a.async_job_id));
    let entries = vec![cs_entry(&a.target_keys[0], "fp")];
    do_complete(c, &claim, completion(CompletionOutcome::NoOp, entries)).unwrap();
    assert!(!held(c, a.async_job_id));
    // T7, then T12 to RETRY_SCHEDULED, then T12 to FAILED.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t7", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    assert!(!held(c, a.async_job_id));
    for expected in [AsyncJobStatus::RetryScheduled, AsyncJobStatus::Failed] {
        make_available(c, a.async_job_id);
        claim_job(c, &a);
        assert!(held(c, a.async_job_id));
        expire_lease(c, a.async_job_id);
        do_recover(c);
        assert_eq!(status(c, a.async_job_id), expected);
        assert!(!held(c, a.async_job_id));
    }
    // T8.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t8", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert!(!held(c, a.async_job_id));
    // T9n.
    let (a, claim) = started(c, new_job(TEST_EFFECT, "r-t9n", &[&key(NS_CS)]));
    do_fail(
        c,
        &claim,
        no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
    )
    .unwrap();
    assert!(!held(c, a.async_job_id));
    // T10d to FAILED.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t10d", &[&key(NS_CS)]));
    let claim = resumed(c, &a, &claim);
    expire_waiting_deadline(c, a.async_job_id, 1);
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::Failed);
    assert!(!held(c, a.async_job_id));
    // T11 to FAILED.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t11", &[&key(NS_CS)]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert!(held(c, a.async_job_id));
    expire_waiting_deadline(c, a.async_job_id, 1);
    do_recover(c);
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::Failed);
    assert!(!held(c, a.async_job_id));
    // T13.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t13", &[&key(NS_CS)]));
    do_release(c, &claim).unwrap();
    assert!(!held(c, a.async_job_id));
    // T14w.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "r-t14w", &[&key(NS_CS)]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert_applied(&do_cancel(c, a.async_job_id), "CANCELLED");
    assert!(!held(c, a.async_job_id));
    // T14r.
    let (a, _claim) = running(c, new_job(TEST_EFFECT, "r-t14r", &[&key(NS_CS)]));
    assert_applied(&do_cancel(c, a.async_job_id), "CANCELLED");
    assert!(!held(c, a.async_job_id));
    // T16 SUCCEEDED, FAILED and CANCELLED.
    for (n, request) in [
        resolution_entries(vec![]),
        resolution_evidence(ReconcilerOutcome::Failed, NO_EFFECT_EVIDENCE),
        resolution_evidence(ReconcilerOutcome::Cancelled, NO_EFFECT_EVIDENCE),
    ]
    .into_iter()
    .enumerate()
    {
        let x = key(NS_CS);
        let a = rr_started(c, new_job(TEST_EFFECT, &format!("r-t16-{n}"), &[&x]));
        let claim = claim_recon(c, &a);
        let request = if request.outcome == ReconcilerOutcome::Succeeded {
            resolution_entries(vec![cs_entry(&x, "fp")])
        } else {
            request
        };
        let row = do_resolve(c, &claim, request).unwrap();
        assert!(row.status.is_terminal());
        assert!(!held(c, a.async_job_id));
    }
    // T18 SUCCEEDED, FAILED and CANCELLED at EFFECT_STARTED, FAILED at
    // EFFECT_CONFIRMED.
    for n in 0..3 {
        let x = key(NS_CS);
        let a = rr_started(c, new_job(TEST_EFFECT, &format!("r-t18-{n}"), &[&x]));
        let resolution = match n {
            0 => staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
            1 => staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
            _ => staff_evidence(StaffOutcome::Cancelled, NO_EFFECT_EVIDENCE),
        };
        assert!(do_staff(c, a.async_job_id, resolution).is_applied());
        assert!(!held(c, a.async_job_id));
    }
    let a = rr_confirmed(c, new_job(TEST_EFFECT, "r-t18-confirmed", &[&key(NS_CS)]));
    let result = do_staff(
        c,
        a.async_job_id,
        staff_evidence(StaffOutcome::Failed, NOT_APPLIED_EVIDENCE),
    );
    assert_applied(&result, "CONFIRMED_EFFECT_NOT_APPLIED");
    assert!(!held(c, a.async_job_id));
}

#[test]
fn targets_holds_are_kept_through_waiting_and_reconciliation() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let held = |c: &mut PgConnection, id: Uuid, x: &str| holds(c, id) == vec![x.to_string()];

    // T10, then T14e from WAITING after start.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "k-t10", &[&x]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert!(held(c, a.async_job_id, &x));
    assert_applied(&do_cancel(c, a.async_job_id), "OPERATOR_STOP");
    assert_eq!(
        status(c, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert!(held(c, a.async_job_id, &x));
    // T14e from RUNNING after start.
    let x = key(NS_CS);
    let (a, _claim) = started(c, new_job(TEST_EFFECT, "k-t14e", &[&x]));
    assert_applied(&do_cancel(c, a.async_job_id), "OPERATOR_STOP");
    assert!(held(c, a.async_job_id, &x));
    // T9, T15, T16 STILL_REQUIRED, T15, T17, T15, T16 WAITING.
    let x = key(NS_CS);
    let a = rr_started(c, new_job(TEST_EFFECT, "k-t9", &[&x]));
    assert!(held(c, a.async_job_id, &x));
    let claim = claim_recon(c, &a);
    assert!(held(c, a.async_job_id, &x));
    do_resolve(c, &claim, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    assert!(held(c, a.async_job_id, &x));
    make_available(c, a.async_job_id);
    claim_recon(c, &a);
    expire_lease(c, a.async_job_id);
    do_recover(c);
    assert!(held(c, a.async_job_id, &x));
    let resume_at = resume_in(c, 10);
    let row = do_staff(
        c,
        a.async_job_id,
        StaffResolution {
            checkpoint: Some("{}".to_string()),
            resume_at: Some(resume_at),
            ..staff_res(StaffOutcome::Waiting)
        },
    );
    assert_applied(&row, "WAITING");
    assert!(held(c, a.async_job_id, &x));
    // T6x.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "k-t6x", &[&x]));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&key(NS_CS), "fp")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert!(held(c, a.async_job_id, &x));
    // T16 WAITING.
    let claim = claim_recon(c, &a);
    let resume_at = resume_in(c, 10);
    let row = do_resolve(
        c,
        &claim,
        ResolveReconciliation {
            checkpoint: Some("{}".to_string()),
            resume_at: Some(resume_at),
            ..resolution(ReconcilerOutcome::Waiting)
        },
    )
    .unwrap();
    assert_eq!(row.status, AsyncJobStatus::Waiting);
    assert!(held(c, a.async_job_id, &x));
    // T10d and T11 to RR.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "k-t10d", &[&x]));
    let claim = resumed(c, &a, &claim);
    expire_waiting_deadline(c, a.async_job_id, 1);
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert!(held(c, a.async_job_id, &x));
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "k-t11", &[&x]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(c, a.async_job_id, 1);
    do_recover(c);
    assert_eq!(
        status(c, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert!(held(c, a.async_job_id, &x));
    // T12 to RR and to WAITING.
    let x = key(NS_CS);
    let (a, _claim) = started(c, new_job(TEST_EFFECT, "k-t12", &[&x]));
    expire_lease(c, a.async_job_id);
    do_recover(c);
    assert!(held(c, a.async_job_id, &x));
    let p = key(NS_POLL);
    let a = create(c, new_job(TEST_POLL, "k-t12w", &[&p]));
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    do_confirm(c, &claim).unwrap();
    resumed(c, &a, &claim);
    expire_lease(c, a.async_job_id);
    do_recover(c);
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::Waiting);
    assert!(held(c, a.async_job_id, &p));
}

#[test]
fn targets_current_state_rows_of_the_evidence_table() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // EFFECT_APPLIED after start: writes the fingerprint and the sourceRef.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "ea", &[&x]));
    let entry = EvidenceEntry {
        source_ref: Some("ref-x".to_string()),
        ..cs_entry(&x, "fp-1")
    };
    let outcome = do_complete(
        c,
        &claim,
        completion(CompletionOutcome::EffectApplied, vec![entry]),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    let row = state(c, &x).unwrap();
    assert_eq!(
        (
            row.applied_order,
            row.applied_fingerprint.as_str(),
            row.applied_source_ref.as_deref()
        ),
        (None, "fp-1", Some("ref-x"))
    );
    let last = last_attempt(c, a.async_job_id);
    assert_eq!(
        state_writer(c, &x),
        (
            Some(a.async_job_id.to_string()),
            Some(last.async_job_attempt_id.to_string())
        )
    );
    assert_eq!(last.outcome_code.as_deref(), Some("EFFECT_APPLIED"));

    // EFFECT_APPLIED at PRE_WRITE: PHASE, claim live.
    let (_b, claim) = running(c, new_job(TEST_EFFECT, "ea-pre", &[&key(NS_CS)]));
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::EffectApplied, vec![cs_entry(&x, "fp")]),
        ),
        "PHASE",
    );
    assert_live(c, &claim);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();

    // NO_OP at PRE_WRITE: written when absent, nothing when equal, written
    // when different.
    let y = key(NS_CS);
    let (n1, claim) = running(c, new_job(TEST_EFFECT, "noop-1", &[&y]));
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&y, "fp-y")]),
    )
    .unwrap();
    assert_eq!(state_writer(c, &y).0, Some(n1.async_job_id.to_string()));
    let (_n2, claim) = running(c, new_job(TEST_EFFECT, "noop-2", &[&y]));
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&y, "fp-y")]),
    )
    .unwrap();
    assert_eq!(state_writer(c, &y).0, Some(n1.async_job_id.to_string()));
    let (n3, claim) = running(c, new_job(TEST_EFFECT, "noop-3", &[&y]));
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&y, "fp-z")]),
    )
    .unwrap();
    assert_eq!(state_writer(c, &y).0, Some(n3.async_job_id.to_string()));
    assert_eq!(applied(c, &y), Some((None, "fp-z".to_string())));

    // NO_OP after start: PHASE, claim live.
    let (_d, claim) = started(c, new_job(TEST_EFFECT, "noop-post", &[&key(NS_CS)]));
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::NoOp, vec![cs_entry(&y, "fp")]),
        ),
        "PHASE",
    );
    assert_live(c, &claim);

    // SUPERSEDED and COMPLETED on a CURRENT_STATE EFFECTFUL kind: KIND_POLICY.
    let z = key(NS_CS);
    let (_e, claim) = running(c, new_job(TEST_EFFECT, "kind-policy", &[&z]));
    for outcome in [CompletionOutcome::Superseded, CompletionOutcome::Completed] {
        assert_refused(
            do_complete(
                c,
                &claim,
                completion(outcome, vec![EvidenceEntry::key_only(&z)]),
            ),
            "KIND_POLICY",
        );
        assert_live(c, &claim);
    }

    // The entry set must equal the job's targets: at PRE_WRITE nothing is
    // written; after start it is T6x.
    let (p, q) = (key(NS_CS), key(NS_CS));
    let (_f, claim) = running(c, new_job(TEST_EFFECT, "set-pre", &[&p, &q]));
    for entries in [
        vec![cs_entry(&p, "fp")],
        vec![
            cs_entry(&p, "fp"),
            cs_entry(&q, "fp"),
            cs_entry(&key(NS_CS), "fp"),
        ],
    ] {
        assert_code(
            do_complete(c, &claim, completion(CompletionOutcome::NoOp, entries)),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_live(c, &claim);
    }
    assert!(state(c, &p).is_none());
    let (g, claim) = started(
        c,
        new_job(TEST_EFFECT, "set-post", &[&key(NS_CS), &key(NS_CS)]),
    );
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&g.target_keys[0], "fp")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        last_attempt(c, g.async_job_id).outcome_code.as_deref(),
        Some("EVIDENCE_INVALID")
    );
}

#[test]
fn targets_revision_bound_rows_of_the_evidence_table() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // EFFECT_APPLIED forward after start: written.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 5, "h5");
    let (_a, claim) = started(c, revision_job("ea-forward", &k, 6, "h6"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 6, "h6")],
        ),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    assert_eq!(applied(c, &k), Some(rb_state(6, "h6")));

    // EFFECT_APPLIED below the stored order: T6x EVIDENCE_REGRESSION.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 7, "h7");
    let (a, claim) = started(c, revision_job("ea-regression", &k, 6, "h6"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 6, "h6")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        last_attempt(c, a.async_job_id).outcome_code.as_deref(),
        Some("EVIDENCE_REGRESSION")
    );
    assert_eq!(applied(c, &k), Some(rb_state(7, "h7")));

    // EFFECT_APPLIED at the stored order with another fingerprint: T6x
    // EVIDENCE_INVALID.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 6, "h6");
    let (a, claim) = started(c, revision_job("ea-conflict", &k, 6, "h6b"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 6, "h6b")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        last_attempt(c, a.async_job_id).outcome_code.as_deref(),
        Some("EVIDENCE_INVALID")
    );
    assert_eq!(applied(c, &k), Some(rb_state(6, "h6")));

    // An entry other than the job's own revision after start: T6x.
    let k = key(NS_RB);
    let (a, claim) = started(c, revision_job("ea-other", &k, 6, "h6"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 8, "h8")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert!(state(c, &k).is_none());
    assert_eq!(
        status(c, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );

    // NO_OP at PRE_WRITE: written when absent; nothing when equal.
    let k = key(NS_RB);
    let (a, claim) = running(c, revision_job("noop-absent", &k, 3, "h3"));
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![rb_entry(&k, 3, "h3")]),
    )
    .unwrap();
    assert_eq!(applied(c, &k), Some(rb_state(3, "h3")));
    assert_eq!(state_writer(c, &k).0, Some(a.async_job_id.to_string()));
    let (_b, claim) = running(c, revision_job("noop-equal", &k, 3, "h3"));
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![rb_entry(&k, 3, "h3")]),
    )
    .unwrap();
    assert_eq!(state_writer(c, &k).0, Some(a.async_job_id.to_string()));

    // NO_OP below the stored order: invalid (report SUPERSEDED); SUPERSEDED
    // then succeeds and writes nothing.
    let k = key(NS_RB);
    let seed = seed_rb_state(c, &k, 7, "h7");
    let (a, claim) = running(c, revision_job("noop-older", &k, 6, "h6"));
    assert_code(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::NoOp, vec![rb_entry(&k, 6, "h6")]),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_live(c, &claim);
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::Superseded,
            vec![EvidenceEntry::key_only(&k)],
        ),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    assert_eq!(applied(c, &k), Some(rb_state(7, "h7")));
    assert_eq!(state_writer(c, &k).0, Some(seed.to_string()));
    assert_eq!(
        last_attempt(c, a.async_job_id).outcome_code.as_deref(),
        Some("SUPERSEDED")
    );

    // SUPERSEDED without a stored state, and with a lower stored order:
    // invalid at PRE_WRITE, claim live.
    let k = key(NS_RB);
    let (_a, claim) = running(c, revision_job("sup-absent", &k, 3, "h3"));
    assert_code(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::Superseded,
                vec![EvidenceEntry::key_only(&k)],
            ),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_live(c, &claim);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    let k = key(NS_RB);
    seed_rb_state(c, &k, 2, "h2");
    let (_a, claim) = running(c, revision_job("sup-lower", &k, 3, "h3"));
    assert_code(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::Superseded,
                vec![EvidenceEntry::key_only(&k)],
            ),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_live(c, &claim);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();

    // SUPERSEDED after start: PHASE.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 9, "h9");
    let (_a, claim) = started(c, revision_job("sup-post", &k, 3, "h3"));
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::Superseded,
                vec![EvidenceEntry::key_only(&k)],
            ),
        ),
        "PHASE",
    );
    assert_live(c, &claim);
}

#[test]
fn targets_key_only_rows_and_read_only_completion() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // COMPLETED on a READ_ONLY kind: SUCCEEDED, nothing written.
    let r = key(NS_READ);
    let (a, claim) = running(c, new_job(TEST_READ, "read", &[&r]));
    let outcome = tx(c, |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(
                CompletionOutcome::Completed,
                vec![EvidenceEntry::key_only(&r)],
            ),
        )
    })
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    assert!(state(c, &r).is_none());
    assert!(holds(c, a.async_job_id).is_empty());
    // EFFECT_APPLIED needs a started effect, which a READ_ONLY job never
    // has (PHASE comes first); SUPERSEDED needs a REVISION_BOUND kind.
    let r2 = key(NS_READ);
    let (b, claim) = running(c, new_job(TEST_READ, "read-2", &[&r2]));
    for (outcome, entry, reason) in [
        (
            CompletionOutcome::EffectApplied,
            cs_entry(&r2, "fp"),
            "PHASE",
        ),
        (
            CompletionOutcome::Superseded,
            EvidenceEntry::key_only(&r2),
            "KIND_POLICY",
        ),
    ] {
        assert_refused(
            tx(c, |c| {
                complete_on(c, &REGISTRY, &claim, completion(outcome, vec![entry]))
            }),
            reason,
        );
        assert_live(c, &claim);
    }
    // NO_OP names only the phase in section 13.3, so a READ_ONLY job may
    // record the fingerprint it observed.
    let outcome = tx(c, |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(CompletionOutcome::NoOp, vec![cs_entry(&r2, "seen")]),
        )
    })
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    assert_eq!(applied(c, &r2), Some((None, "seen".to_string())));
    assert_eq!(
        last_attempt(c, b.async_job_id).outcome_code.as_deref(),
        Some("NO_OP")
    );
    assert!(holds(c, b.async_job_id).is_empty());
}

/// A READ_ONLY kind with a REVISION_BOUND target (section 13.3 names only
/// the phase and the revision mode for `SUPERSEDED`).
static READ_REVISION_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[AsyncJobKindPolicy {
    kind: "TEST_READ_REVISION",
    effect_class: EffectClass::ReadOnly,
    effect_window_seconds: None,
    staff_reconciliation_outcomes: &[],
    ..REVISION_POLICY
}]);

#[test]
fn targets_a_read_only_revision_bound_job_completes_superseded_or_completed_with_key_only_entries()
{
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let registry = &READ_REVISION_REGISTRY;
    assert_eq!(registry.validate(), Ok(()));
    let run = |c: &mut PgConnection, id: &str, target: &str, order: i64| {
        let input = NewAsyncJob {
            kind: "TEST_READ_REVISION".to_string(),
            ..revision_job(id, target, order, &format!("h{order}"))
        };
        let created = match tx(c, |c| create_job_on(c, registry, input)).expect("create") {
            CreateOutcome::Created(job) => job,
            other => panic!("expected a new job, got {other:?}"),
        };
        let executor = dissemination();
        let claimed = tx(c, |c| {
            claim_on(c, registry, &executor, request(&["TEST_READ_REVISION"], 1))
        })
        .expect("claim")
        .jobs;
        assert_eq!(claimed.len(), 1);
        (created, claimed[0].worker_claim(executor.domain))
    };
    let k = key(NS_RB);
    seed_rb_state(c, &k, 9, "h9");
    // An older revision is SUPERSEDED with its key-only entry.
    let (old, claim) = run(c, "read-old", &k, 3);
    let complete = |c: &mut PgConnection, claim: &WorkerClaim, outcome| {
        tx(c, |c| {
            complete_on(
                c,
                registry,
                claim,
                completion(outcome, vec![EvidenceEntry::key_only(&k)]),
            )
        })
    };
    assert!(matches!(
        complete(c, &claim, CompletionOutcome::Superseded).unwrap(),
        CompleteOutcome::Succeeded(_)
    ));
    assert_eq!(status(c, old.async_job_id), AsyncJobStatus::Succeeded);
    assert_eq!(
        last_attempt(c, old.async_job_id).outcome_code.as_deref(),
        Some("SUPERSEDED")
    );
    // A newer revision cannot claim SUPERSEDED, and COMPLETED changes
    // nothing.
    let (new, claim) = run(c, "read-new", &k, 12);
    assert_code(
        complete(c, &claim, CompletionOutcome::Superseded),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    tx(c, |c| renew_on(c, registry, &claim, None)).expect("the claim must still be live");
    assert!(matches!(
        complete(c, &claim, CompletionOutcome::Completed).unwrap(),
        CompleteOutcome::Succeeded(_)
    ));
    assert_eq!(status(c, new.async_job_id), AsyncJobStatus::Succeeded);
    assert_eq!(applied(c, &k), Some(rb_state(9, "h9")));
    assert!(holds(c, new.async_job_id).is_empty());
}

#[test]
fn targets_an_older_retried_revision_cannot_overwrite_a_newer_one() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let r = key(NS_RB);
    // Revision 3 fails before any write; revision 5 then succeeds.
    let (old, claim) = running(c, revision_job("doc", &r, 3, "h3"));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    let (_new, claim) = started(c, revision_job("doc", &r, 5, "h5"));
    do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&r, 5, "h5")],
        ),
    )
    .unwrap();
    // Staff retries revision 3.
    let retry = do_retry(c, old.async_job_id);
    assert_applied(&retry, "RETRY_CREATED");
    let child = job(c, retry.result_job_id.unwrap());
    assert_eq!(child.lineage, AsyncJobLineage::ManualRetry);
    assert_eq!(child.parent_job_id, Some(old.async_job_id));
    assert_eq!(child.source_order, Some(3));
    let claim = claim_job(c, &child);
    // At PRE_WRITE the older revision cannot be recorded as a no-op...
    assert_code(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::NoOp, vec![rb_entry(&r, 3, "h3")]),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    // ... and after effect start its write is a regression (T6x).
    do_start(c, &claim).unwrap();
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&r, 3, "h3")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        last_attempt(c, child.async_job_id).outcome_code.as_deref(),
        Some("EVIDENCE_REGRESSION")
    );
    assert_eq!(applied(c, &r), Some(rb_state(5, "h5")));
    // An executor reconciler cannot record it either.
    let claim = claim_recon(c, &child);
    assert_code(
        do_resolve(c, &claim, resolution_entries(vec![rb_entry(&r, 3, "h3")])),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_live(c, &claim);
    assert_eq!(applied(c, &r), Some(rb_state(5, "h5")));
}

#[test]
fn targets_evidence_advances_only_with_the_current_token_under_the_hold() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, first) = running(c, new_job(TEST_EFFECT, "fenced", &[&x]));
    do_release(c, &first).unwrap();
    let second = claim_job(c, &a);
    assert_code(
        do_complete(
            c,
            &first,
            completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp-old")]),
        ),
        STALE,
    );
    assert!(state(c, &x).is_none());
    do_complete(
        c,
        &second,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp-new")]),
    )
    .unwrap();
    let writer = last_attempt(c, a.async_job_id).async_job_attempt_id;
    assert_eq!(
        state_writer(c, &x),
        (Some(a.async_job_id.to_string()), Some(writer.to_string()))
    );
    // Direct SQL by a job that does not hold the key is refused.
    let (b, _claim) = running(c, new_job(TEST_PEER, "intruder", &[&key(NS_CS)]));
    let intruder = last_attempt(c, b.async_job_id).async_job_attempt_id;
    assert_eq!(
        refused_by(
            c,
            &format!(
                "UPDATE public.async_effect_target_state \
                    SET applied_fingerprint = 'fp-intruder', last_job_id = '{}', \
                        last_attempt_id = '{intruder}' \
                  WHERE target_key = '{x}'",
                b.async_job_id
            )
        ),
        "async_effect_target_state_guard"
    );
    assert_eq!(applied(c, &x), Some((None, "fp-new".to_string())));
}

#[test]
fn targets_a_provider_without_revision_ids_still_blocks_regression_through_local_order() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let r = key(NS_RB);
    let no_ref = |id: &str, order: i64, fingerprint: &str| NewAsyncJob {
        source_ref: None,
        ..revision_job(id, &r, order, fingerprint)
    };
    let (_newer, claim) = started(c, no_ref("plain", 9, "c9"));
    do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&r, 9, "c9")],
        ),
    )
    .unwrap();
    let row = state(c, &r).unwrap();
    assert_eq!(row.applied_source_ref, None);
    let (older, claim) = started(c, no_ref("plain", 4, "c4"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&r, 4, "c4")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_eq!(
        last_attempt(c, older.async_job_id).outcome_code.as_deref(),
        Some("EVIDENCE_REGRESSION")
    );
    assert_eq!(applied(c, &r), Some(rb_state(9, "c9")));
}

#[test]
fn targets_identifier_format_faults_in_entries_are_evidence_invalid_at_every_phase() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let long = "f".repeat(257);
    let bad_fingerprints = ["has space", long.as_str()];
    let bad_refs = ["bad ref", long.as_str()];
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "ident", &[&x]));
    // PRE_WRITE (NO_OP lists no sourceRef, so only the fingerprint).
    for fingerprint in bad_fingerprints {
        assert_code(
            do_complete(
                c,
                &claim,
                completion(CompletionOutcome::NoOp, vec![cs_entry(&x, fingerprint)]),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_live(c, &claim);
    }
    // EFFECT_STARTED and EFFECT_CONFIRMED.
    for mark in [do_start, do_confirm] {
        mark(c, &claim).unwrap();
        for fingerprint in bad_fingerprints {
            assert_code(
                do_complete(
                    c,
                    &claim,
                    completion(
                        CompletionOutcome::EffectApplied,
                        vec![cs_entry(&x, fingerprint)],
                    ),
                ),
                "ASYNC_JOB_EVIDENCE_INVALID",
            );
            assert_live(c, &claim);
        }
        for source_ref in bad_refs {
            let entry = EvidenceEntry {
                source_ref: Some(source_ref.to_string()),
                ..cs_entry(&x, "fp")
            };
            assert_code(
                do_complete(
                    c,
                    &claim,
                    completion(CompletionOutcome::EffectApplied, vec![entry]),
                ),
                "ASYNC_JOB_EVIDENCE_INVALID",
            );
            assert_live(c, &claim);
        }
    }
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::Running);
    // T16: the job stays RR with its claim live.
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    let claim = claim_recon(c, &a);
    for fingerprint in bad_fingerprints {
        assert_code(
            do_resolve(
                c,
                &claim,
                resolution_entries(vec![cs_entry(&x, fingerprint)]),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_live(c, &claim);
    }
    // T18 (after the reconciliation claim expires): a committed REFUSED.
    expire_lease(c, a.async_job_id);
    for fingerprint in bad_fingerprints {
        let result = do_staff(
            c,
            a.async_job_id,
            staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, fingerprint)]),
        );
        assert_command_refused(&result, "EVIDENCE_INVALID");
    }
    assert_eq!(
        status(c, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert!(state(c, &x).is_none());
}

#[test]
fn targets_normal_completion_racing_a_reconciler_one_wins_and_the_loser_is_stale() {
    let (_guard, mut conn) = setup();
    let before = deadlocks(&mut conn);

    // The reconciler first: its claim recovers the expired lease (T12 to RR)
    // and claims the job (T15); the late completion is stale.
    let x = key(NS_CS);
    let (a, claim) = started(&mut conn, new_job(TEST_EFFECT, "race-1", &[&x]));
    expire_lease(&mut conn, a.async_job_id);
    let reconciler = Session::open();
    let claimed = reconciler.run(|c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50))
    });
    let claimed = claimed.unwrap().jobs;
    assert_eq!(claimed.len(), 1);
    let recon_claim = claimed[0].worker_claim(ExecutorDomain::Dissemination);
    let entries = vec![cs_entry(&x, "fp")];
    let (pid, late) = spawn_tx(move |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(CompletionOutcome::EffectApplied, entries),
        )
    });
    wait_until_blocked(&mut conn, pid);
    reconciler.commit();
    assert_code(late.join().unwrap(), STALE);
    let row = do_resolve(
        &mut conn,
        &recon_claim,
        resolution_entries(vec![cs_entry(&x, "fp-r")]),
    )
    .unwrap();
    assert_eq!(row.status, AsyncJobStatus::Succeeded);

    // The worker first: the reconciliation claim skips the locked row and
    // claims nothing; the completion wins.
    let y = key(NS_CS);
    let (b, claim) = started(&mut conn, new_job(TEST_EFFECT, "race-2", &[&y]));
    expire_lease(&mut conn, b.async_job_id);
    let worker = Session::open();
    let entries = vec![cs_entry(&y, "fp")];
    let outcome = worker.run(move |c| {
        complete_on(
            c,
            &REGISTRY,
            &claim,
            completion(CompletionOutcome::EffectApplied, entries),
        )
    });
    assert!(matches!(outcome, Ok(CompleteOutcome::Succeeded(_))));
    let none = tx(&mut conn, |c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50))
    })
    .unwrap();
    assert!(none.jobs.is_empty());
    worker.commit();
    assert_eq!(status(&mut conn, b.async_job_id), AsyncJobStatus::Succeeded);

    // T16 against T17, both orders: an expired reconciliation claim.
    let z = key(NS_CS);
    let c3 = rr_started(&mut conn, new_job(TEST_EFFECT, "race-3", &[&z]));
    let recon_claim = claim_recon(&mut conn, &c3);
    expire_lease(&mut conn, c3.async_job_id);
    let recovery = Session::open();
    let report = recovery
        .run(|c| recover_on(c, &REGISTRY, None, 1000))
        .unwrap();
    assert_eq!(report.recovered, vec![c3.async_job_id]);
    let entries = vec![cs_entry(&z, "fp")];
    let (pid, late) = spawn_tx(move |c| {
        resolve_reconciliation_on(c, &REGISTRY, &recon_claim, resolution_entries(entries))
    });
    wait_until_blocked(&mut conn, pid);
    recovery.commit();
    assert_code(late.join().unwrap(), STALE);
    let row = job(&mut conn, c3.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert!(row.claimed_by.is_none());

    let w = key(NS_CS);
    let d = rr_started(&mut conn, new_job(TEST_EFFECT, "race-4", &[&w]));
    let recon_claim = claim_recon(&mut conn, &d);
    expire_lease(&mut conn, d.async_job_id);
    let resolver = Session::open();
    let entries = vec![cs_entry(&w, "fp")];
    let row = resolver
        .run(move |c| {
            resolve_reconciliation_on(c, &REGISTRY, &recon_claim, resolution_entries(entries))
        })
        .unwrap();
    assert_eq!(row.status, AsyncJobStatus::Succeeded);
    let report = do_recover(&mut conn);
    assert!(!report.recovered.contains(&d.async_job_id));
    resolver.commit();
    assert_eq!(status(&mut conn, d.async_job_id), AsyncJobStatus::Succeeded);
    assert_no_deadlock(&mut conn, before);
}

// ---------------------------------------------------------------------------
// Cancellation, retry and reconciliation (sections 9.3 and 10.5)
// ---------------------------------------------------------------------------

/// A staff actor with a fixed request id.
fn staff_with(request_id: Uuid) -> StaffActor {
    StaffActor {
        request_id: Some(request_id),
        ..staff("fixed request")
    }
}

#[test]
fn commands_cancel_from_each_state_per_the_table() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // T14 from PENDING and from RETRY_SCHEDULED.
    let a = create(c, new_job(TEST_EFFECT, "c-pending", &[&key(NS_CS)]));
    let result = do_cancel(c, a.async_job_id);
    assert_applied(&result, "CANCELLED");
    assert_eq!(result.job.status, AsyncJobStatus::Cancelled);
    let log = transitions(c, a.async_job_id);
    assert_eq!(log.last().unwrap().cause, "T14_CANCEL");
    assert_eq!(
        log.last().unwrap().async_control_command_id,
        Some(result.async_control_command_id)
    );
    let (b, claim) = running(c, new_job(TEST_EFFECT, "c-retry", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    assert_applied(&do_cancel(c, b.async_job_id), "CANCELLED");
    assert_eq!(causes(c, b.async_job_id).last().unwrap(), "T14_CANCEL");

    // T14w from WAITING at PRE_WRITE.
    let (d, claim) = running(c, new_job(TEST_EFFECT, "c-waiting", &[&key(NS_CS)]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert_applied(&do_cancel(c, d.async_job_id), "CANCELLED");
    assert_eq!(
        causes(c, d.async_job_id).last().unwrap(),
        "T14W_CANCEL_WAITING"
    );
    assert!(holds(c, d.async_job_id).is_empty());

    // T14r from RUNNING at PRE_WRITE: attempt CANCELLED, token invalidated.
    let (e, claim) = running(c, new_job(TEST_EFFECT, "c-running", &[&key(NS_CS)]));
    assert_applied(&do_cancel(c, e.async_job_id), "CANCELLED");
    assert_eq!(
        causes(c, e.async_job_id).last().unwrap(),
        "T14R_CANCEL_RUNNING"
    );
    let last = last_attempt(c, e.async_job_id);
    assert_eq!(last.result, Some(AsyncAttemptResult::Cancelled));
    assert_eq!(last.job_status_after, Some(AsyncJobStatus::Cancelled));
    assert!(holds(c, e.async_job_id).is_empty());
    assert_code(do_renew(c, &claim), STALE);

    // T14e from RUNNING at EFFECT_STARTED and EFFECT_CONFIRMED, and from
    // WAITING after start: RR, holds kept.
    for confirm in [false, true] {
        let x = key(NS_CS);
        let (f, claim) = started(c, new_job(TEST_EFFECT, &format!("c-stop-{confirm}"), &[&x]));
        if confirm {
            do_confirm(c, &claim).unwrap();
        }
        let result = do_cancel(c, f.async_job_id);
        assert_applied(&result, "OPERATOR_STOP");
        assert_eq!(result.job.status, AsyncJobStatus::ReconciliationRequired);
        assert_eq!(
            causes(c, f.async_job_id).last().unwrap(),
            "T14E_OPERATOR_STOP"
        );
        let last = last_attempt(c, f.async_job_id);
        assert_eq!(last.result, Some(AsyncAttemptResult::Cancelled));
        assert_eq!(
            last.job_status_after,
            Some(AsyncJobStatus::ReconciliationRequired)
        );
        assert_eq!(holds(c, f.async_job_id), vec![x]);
        assert_code(do_renew(c, &claim), STALE);
    }
    let x = key(NS_CS);
    let (g, claim) = started(c, new_job(TEST_EFFECT, "c-stop-waiting", &[&x]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    let attempts_before = attempts(c, g.async_job_id).len();
    assert_applied(&do_cancel(c, g.async_job_id), "OPERATOR_STOP");
    assert_eq!(attempts(c, g.async_job_id).len(), attempts_before);
    assert_eq!(holds(c, g.async_job_id), vec![x]);

    // RR and terminal jobs: NOT_CANCELLABLE.
    for id in [g.async_job_id, a.async_job_id, e.async_job_id] {
        assert_command_refused(&do_cancel(c, id), "NOT_CANCELLABLE");
    }
    let (h, claim) = running(c, new_job(TEST_EFFECT, "c-succeeded", &[&key(NS_CS)]));
    let entries = vec![cs_entry(&h.target_keys[0], "fp")];
    do_complete(c, &claim, completion(CompletionOutcome::NoOp, entries)).unwrap();
    assert_command_refused(&do_cancel(c, h.async_job_id), "NOT_CANCELLABLE");
    let (i, claim) = running(c, new_job(TEST_EFFECT, "c-failed", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_command_refused(&do_cancel(c, i.async_job_id), "NOT_CANCELLABLE");

    // A kind whose policy allows no cancel: KIND_POLICY before effect start,
    // T14e after it whatever the policy.
    let j = create(c, new_job(TEST_STRICT, "c-strict", &[&key(NS_CS)]));
    assert_command_refused(&do_cancel(c, j.async_job_id), "KIND_POLICY");
    let claim = claim_job(c, &j);
    assert_command_refused(&do_cancel(c, j.async_job_id), "KIND_POLICY");
    assert_live(c, &claim);
    do_start(c, &claim).unwrap();
    assert_applied(&do_cancel(c, j.async_job_id), "OPERATOR_STOP");
    assert_no_unspecified(c);
}

#[test]
fn commands_cancel_racing_effect_start_in_both_orders() {
    let (_guard, mut conn) = setup();
    let before = deadlocks(&mut conn);

    // Cancel first: the effect start waits for the row, then is stale.
    let (a, claim) = running(
        &mut conn,
        new_job(TEST_EFFECT, "race-cancel", &[&key(NS_CS)]),
    );
    let id = a.async_job_id;
    let operator = Session::open();
    let result = operator
        .run(move |c| cancel_job_command_on(c, &REGISTRY, &staff("race"), id))
        .unwrap();
    assert_applied(&result, "CANCELLED");
    let (pid, start) = spawn_tx(move |c| mark_effect_started_on(c, &REGISTRY, &claim));
    wait_until_blocked(&mut conn, pid);
    operator.commit();
    assert_code(start.join().unwrap(), STALE);
    let row = job(&mut conn, id);
    assert_eq!(row.status, AsyncJobStatus::Cancelled);
    assert_eq!(row.effect_phase, AsyncEffectPhase::PreWrite);

    // Effect start first: the cancel waits, then stops the job (T14e).
    let (b, claim) = running(
        &mut conn,
        new_job(TEST_EFFECT, "race-start", &[&key(NS_CS)]),
    );
    let id = b.async_job_id;
    let worker = Session::open();
    let row = worker
        .run(move |c| mark_effect_started_on(c, &REGISTRY, &claim))
        .unwrap();
    assert_eq!(row.effect_phase, AsyncEffectPhase::EffectStarted);
    let (pid, cancel) = spawn_tx(move |c| cancel_job_command_on(c, &REGISTRY, &staff("race"), id));
    wait_until_blocked(&mut conn, pid);
    worker.commit();
    let result = cancel.join().unwrap().unwrap();
    assert_applied(&result, "OPERATOR_STOP");
    assert_eq!(result.job.status, AsyncJobStatus::ReconciliationRequired);
    assert_no_deadlock(&mut conn, before);
}

#[test]
fn commands_rr_cannot_be_cancelled_or_retried() {
    let (_guard, mut conn) = setup();
    let a = rr_started(&mut conn, new_job(TEST_EFFECT, "rr", &[&key(NS_CS)]));
    assert_command_refused(&do_cancel(&mut conn, a.async_job_id), "NOT_CANCELLABLE");
    assert_command_refused(&do_retry(&mut conn, a.async_job_id), "RETRY_NOT_ALLOWED");
    assert_eq!(
        status(&mut conn, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(command_rows(&mut conn), 2);
}

#[test]
fn commands_manual_retry_rules() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // FAILED at PRE_WRITE: one retry child, which runs normally.
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "retry-a", &[&x]));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    let result = do_retry(c, a.async_job_id);
    assert_applied(&result, "RETRY_CREATED");
    let child = job(c, result.result_job_id.unwrap());
    assert_eq!(child.lineage, AsyncJobLineage::ManualRetry);
    assert_eq!(child.parent_job_id, Some(a.async_job_id));
    assert_eq!(
        child.idempotency_key,
        format!("TEST_EFFECT:retry-a#retry:{}", a.async_job_id)
    );
    assert_eq!(child.created_by, "staff:operator@example.org");
    assert_eq!(
        (child.kind.as_str(), child.payload.as_str()),
        (a.kind.as_str(), a.payload.as_str())
    );
    assert_eq!(child.target_keys, a.target_keys);
    assert_eq!(child.status, AsyncJobStatus::Pending);
    assert_eq!(
        (
            child.attempt_count,
            child.execution_claim_count,
            child.prewrite_failure_count
        ),
        (0, 0, 0)
    );
    assert_eq!(causes(c, child.async_job_id), ["RETRY_JOB_CREATE"]);
    assert_command_refused(&do_retry(c, a.async_job_id), "RETRY_NOT_ALLOWED");
    assert_command_refused(&do_retry(c, child.async_job_id), "RETRY_NOT_ALLOWED");
    let claim = claim_job(c, &child);
    let outcome = do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp")]),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    // SUCCEEDED: not retryable.
    assert_command_refused(&do_retry(c, child.async_job_id), "RETRY_NOT_ALLOWED");

    // CANCELLED at PRE_WRITE.
    let b = create(c, new_job(TEST_EFFECT, "retry-b", &[&key(NS_CS)]));
    do_cancel(c, b.async_job_id);
    assert_applied(&do_retry(c, b.async_job_id), "RETRY_CREATED");

    // FAILED after a proven no-effect (T9n).
    let (d, claim) = started(c, new_job(TEST_EFFECT, "retry-d", &[&key(NS_CS)]));
    do_fail(
        c,
        &claim,
        no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
    )
    .unwrap();
    assert_applied(&do_retry(c, d.async_job_id), "RETRY_CREATED");

    // CANCELLED by a reconciler, and FAILED by staff, at EFFECT_STARTED with
    // no-effect evidence.
    let e = rr_started(c, new_job(TEST_EFFECT, "retry-e", &[&key(NS_CS)]));
    let claim = claim_recon(c, &e);
    do_resolve(
        c,
        &claim,
        resolution_evidence(ReconcilerOutcome::Cancelled, NO_EFFECT_EVIDENCE),
    )
    .unwrap();
    assert_applied(&do_retry(c, e.async_job_id), "RETRY_CREATED");
    let f = rr_started(c, new_job(TEST_EFFECT, "retry-f", &[&key(NS_CS)]));
    do_staff(
        c,
        f.async_job_id,
        staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
    );
    assert_applied(&do_retry(c, f.async_job_id), "RETRY_CREATED");

    // Not yet finished: refused.
    let g = create(c, new_job(TEST_EFFECT, "retry-g", &[&key(NS_CS)]));
    assert_command_refused(&do_retry(c, g.async_job_id), "RETRY_NOT_ALLOWED");
    let claim = claim_job(c, &g);
    assert_command_refused(&do_retry(c, g.async_job_id), "RETRY_NOT_ALLOWED");
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    assert_command_refused(&do_retry(c, g.async_job_id), "RETRY_NOT_ALLOWED");

    // Kinds that do not allow it: no manual retry, admission closed, and a
    // version that is no longer creatable.
    let (h, claim) = running(c, new_job(TEST_STRICT, "retry-h", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_command_refused(&do_retry(c, h.async_job_id), "KIND_POLICY");
    let i = create_earlier(c, new_job(TEST_RETIRED, "retry-i", &[&key(NS_CS)]));
    let claim = claim_job(c, &i);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_command_refused(&do_retry(c, i.async_job_id), "KIND_POLICY");
    let j = create_earlier(
        c,
        NewAsyncJob {
            payload_version: 2,
            ..new_job(TEST_VERSIONED, "retry-j", &[&key(NS_CS)])
        },
    );
    let claim = claim_job(c, &j);
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_command_refused(&do_retry(c, j.async_job_id), "KIND_POLICY");
    assert_no_unspecified(c);
}

#[test]
fn commands_reconciliation_claim_and_resolution_are_token_fenced_and_recorded_as_attempts() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, execution) = started(c, new_job(TEST_EFFECT, "recon", &[&x]));
    do_fail(c, &execution, fail_req(FailureClass::Indeterminate)).unwrap();
    let claim = claim_recon(c, &a);
    let row = job(c, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(
        (row.attempt_count, row.reconciliation_attempt_count),
        (2, 1)
    );
    assert!(row.claimed_by.is_some());
    let open = last_attempt(c, a.async_job_id);
    assert_eq!(open.purpose, AsyncAttemptPurpose::Reconciliation);
    assert_eq!(open.phase_at_start, AsyncEffectPhase::EffectStarted);
    assert!(open.finished_at.is_none());
    // T3 renews a reconciliation claim.
    do_renew(c, &claim).unwrap();
    // An execution write with the reconciliation claim is refused.
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::EffectApplied, vec![cs_entry(&x, "fp")]),
        ),
        "RECONCILIATION_IN_PROGRESS",
    );
    // Stale and forged tokens.
    let forged = WorkerClaim {
        claim_token: Uuid::new_v4(),
        ..claim
    };
    for stale in [execution, forged] {
        assert_code(
            do_resolve(c, &stale, resolution_entries(vec![cs_entry(&x, "fp")])),
            STALE,
        );
    }
    let row = do_resolve(c, &claim, resolution_entries(vec![cs_entry(&x, "fp")])).unwrap();
    assert_eq!(row.status, AsyncJobStatus::Succeeded);
    let closed = last_attempt(c, a.async_job_id);
    assert_eq!(closed.async_job_attempt_id, open.async_job_attempt_id);
    assert_eq!(closed.result, Some(AsyncAttemptResult::Succeeded));
    assert_eq!(closed.job_status_after, Some(AsyncJobStatus::Succeeded));
    let log = transitions(c, a.async_job_id);
    let last = log.last().unwrap();
    assert_eq!(last.cause, "T16_RESOLVE");
    assert_eq!(last.async_job_attempt_id, Some(open.async_job_attempt_id));
    assert_eq!(
        state_writer(c, &x).1,
        Some(open.async_job_attempt_id.to_string())
    );
}

#[test]
fn commands_reconciliation_lease_expiry_and_backoff() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let a = rr_started(c, new_job(TEST_EFFECT, "t17", &[&x]));
    let claim = claim_recon(c, &a);
    let causes_before = causes(c, a.async_job_id);
    expire_lease(c, a.async_job_id);
    assert_eq!(do_recover(c).recovered, vec![a.async_job_id]);
    let row = job(c, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert!(row.claimed_by.is_none());
    assert_eq!(row.last_outcome_code.as_deref(), Some("LEASE_EXPIRED"));
    let last = last_attempt(c, a.async_job_id);
    assert_eq!(last.purpose, AsyncAttemptPurpose::Reconciliation);
    assert_eq!(last.result, Some(AsyncAttemptResult::LeaseExpired));
    assert_eq!(
        last.job_status_after,
        Some(AsyncJobStatus::ReconciliationRequired)
    );
    let delay = expected_backoff(c, &row, 1);
    assert_eq!(
        Some(row.available_at),
        last.finished_at.map(|at| at + Duration::seconds(delay))
    );
    // Status preserving: no transition row.
    assert_eq!(causes(c, a.async_job_id), causes_before);
    assert_eq!(holds(c, a.async_job_id), vec![x]);
    assert_code(do_renew(c, &claim), STALE);
    // Not claimable for reconciliation before its backoff.
    let none = tx(c, |c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50))
    })
    .unwrap();
    assert!(none.jobs.is_empty());
}

#[test]
fn commands_staff_resolution_of_unclaimed_and_expired_claims_and_refusal_while_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // Unclaimed RR: SUCCEEDED with entries.
    let x = key(NS_CS);
    let a = rr_started(c, new_job(TEST_EFFECT, "s-unclaimed", &[&x]));
    let result = do_staff(
        c,
        a.async_job_id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    let row = result.job;
    assert_eq!(row.status, AsyncJobStatus::Succeeded);
    assert_eq!(
        (row.attempt_count, row.reconciliation_attempt_count),
        (2, 0)
    );
    let last = last_attempt(c, a.async_job_id);
    assert_eq!(last.purpose, AsyncAttemptPurpose::StaffReconciliation);
    assert_eq!(
        last.control_command_id,
        Some(result.async_control_command_id)
    );
    assert_eq!(last.result, Some(AsyncAttemptResult::Succeeded));

    // A live reconciliation claim: a committed REFUSED, nothing else.
    let b = rr_started(c, new_job(TEST_EFFECT, "s-live", &[&key(NS_CS)]));
    let claim = claim_recon(c, &b);
    let before = job(c, b.async_job_id);
    let rows = command_rows(c);
    let result = do_staff(
        c,
        b.async_job_id,
        staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
    );
    assert_command_refused(&result, "RECONCILIATION_IN_PROGRESS");
    assert_eq!(command_rows(c), rows + 1);
    assert_eq!(job(c, b.async_job_id), before);
    assert_live(c, &claim);

    // An expired reconciliation claim at EFFECT_CONFIRMED: T17, then T18
    // FAILED with not-applied evidence; RETRY_JOB judges the staff attempt.
    let d = rr_confirmed(c, new_job(TEST_EFFECT, "s-confirmed", &[&key(NS_CS)]));
    let claim = claim_recon(c, &d);
    expire_lease(c, d.async_job_id);
    let result = do_staff(
        c,
        d.async_job_id,
        staff_evidence(StaffOutcome::Failed, NOT_APPLIED_EVIDENCE),
    );
    assert_applied(&result, "CONFIRMED_EFFECT_NOT_APPLIED");
    assert_eq!(result.job.status, AsyncJobStatus::Failed);
    let rows = attempts(c, d.async_job_id);
    let recon = &rows[rows.len() - 2];
    let staff_attempt = &rows[rows.len() - 1];
    assert_eq!(recon.result, Some(AsyncAttemptResult::LeaseExpired));
    assert_eq!(
        staff_attempt.result,
        Some(AsyncAttemptResult::PermanentFailure)
    );
    assert_eq!(
        staff_attempt.outcome_code.as_deref(),
        Some("CONFIRMED_EFFECT_NOT_APPLIED")
    );
    assert!(staff_attempt.attempt_number > recon.attempt_number);
    assert_command_refused(&do_retry(c, d.async_job_id), "RETRY_NOT_ALLOWED");
    assert_code(do_renew(c, &claim), STALE);

    // An expired reconciliation claim at EFFECT_STARTED: T17, then T18
    // CANCELLED with no-effect evidence; RETRY_JOB is then allowed.
    let e = rr_started(c, new_job(TEST_EFFECT, "s-started", &[&key(NS_CS)]));
    claim_recon(c, &e);
    expire_lease(c, e.async_job_id);
    let result = do_staff(
        c,
        e.async_job_id,
        staff_evidence(StaffOutcome::Cancelled, NO_EFFECT_EVIDENCE),
    );
    assert_applied(&result, "NO_EFFECT_PROVEN");
    assert_eq!(result.job.status, AsyncJobStatus::Cancelled);
    let rows = attempts(c, e.async_job_id);
    assert_eq!(
        rows[rows.len() - 2].result,
        Some(AsyncAttemptResult::LeaseExpired)
    );
    assert_eq!(
        rows[rows.len() - 1].result,
        Some(AsyncAttemptResult::Cancelled)
    );
    assert_applied(&do_retry(c, e.async_job_id), "RETRY_CREATED");

    // T17 commits with the command even when the command is refused.
    let f = rr_started(c, new_job(TEST_EFFECT, "s-refused", &[&key(NS_CS)]));
    claim_recon(c, &f);
    expire_lease(c, f.async_job_id);
    let result = do_staff(
        c,
        f.async_job_id,
        staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(1)),
    );
    assert_command_refused(&result, "OUTCOME_NOT_ALLOWED");
    assert!(result.job.claimed_by.is_none());
    assert_eq!(
        last_attempt(c, f.async_job_id).result,
        Some(AsyncAttemptResult::LeaseExpired)
    );
}

#[test]
fn commands_staff_resolution_is_atomic_with_its_command_row() {
    let (_guard, mut conn) = setup();
    let x = key(NS_CS);
    let a = rr_started(&mut conn, new_job(TEST_EFFECT, "atomic", &[&x]));
    let id = a.async_job_id;
    let attempts_before = attempts(&mut conn, id);
    // The caller's transaction rolls back: neither the command nor the job
    // change survives.
    let rolled_back =
        conn.build_transaction()
            .read_committed()
            .run(|c| -> Result<(), DieselError> {
                let result = resolve_reconciliation_command_on(
                    c,
                    &REGISTRY,
                    &staff("atomic"),
                    id,
                    staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
                )
                .expect("resolve");
                assert!(result.is_applied());
                Err(DieselError::RollbackTransaction)
            });
    assert!(rolled_back.is_err());
    assert_eq!(command_rows(&mut conn), 0);
    assert_eq!(
        status(&mut conn, id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(attempts(&mut conn, id), attempts_before);
    assert!(state(&mut conn, &x).is_none());
    // Committed: one command row, its id on the attempt and the transition.
    let result = do_staff(
        &mut conn,
        id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    let command = result.async_control_command_id;
    assert_eq!(
        count(
            &mut conn,
            &format!(
                "FROM public.async_control_command WHERE async_control_command_id = '{command}' \
                   AND outcome = 'APPLIED' AND outcome_code = 'EFFECT_APPLIED' \
                   AND decided_at IS NOT NULL AND async_job_id = '{id}'"
            )
        ),
        1
    );
    assert_eq!(
        last_attempt(&mut conn, id).control_command_id,
        Some(command)
    );
    let log = transitions(&mut conn, id);
    let last = log.last().unwrap();
    assert_eq!(last.cause, "T18_STAFF_RESOLVE");
    assert_eq!(last.async_control_command_id, Some(command));
}

#[test]
fn commands_idempotency_on_repeated_and_conflicting_request_ids() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let a = create(c, new_job(TEST_EFFECT, "idem-a", &[&key(NS_CS)]));
    let b = create(c, new_job(TEST_EFFECT, "idem-b", &[&key(NS_CS)]));
    let actor = staff_with(Uuid::new_v4());
    let first = tx(c, |c| {
        cancel_job_command_on(c, &REGISTRY, &actor, a.async_job_id)
    })
    .unwrap();
    assert_applied(&first, "CANCELLED");
    assert!(!first.replayed);
    // The same request id and inputs: the stored decision, nothing new.
    let again = tx(c, |c| {
        cancel_job_command_on(c, &REGISTRY, &actor, a.async_job_id)
    })
    .unwrap();
    assert!(again.replayed);
    assert_eq!(
        again.async_control_command_id,
        first.async_control_command_id
    );
    assert_eq!(
        (again.outcome, again.outcome_code.as_str()),
        (first.outcome, "CANCELLED")
    );
    assert_eq!(command_rows(c), 1);
    // The same request id for another job: a conflict, nothing written.
    assert_code(
        tx(c, |c| {
            cancel_job_command_on(c, &REGISTRY, &actor, b.async_job_id)
        }),
        "ASYNC_COMMAND_REQUEST_CONFLICT",
    );
    assert_eq!(command_rows(c), 1);
    assert_eq!(status(c, b.async_job_id), AsyncJobStatus::Pending);

    // A staff resolution: the same request id with other inputs conflicts;
    // a stored REFUSED is replayed even after its cause has gone.
    let d = rr_started(c, new_job(TEST_EFFECT, "idem-d", &[&key(NS_CS)]));
    let claim = claim_recon(c, &d);
    let actor = staff_with(Uuid::new_v4());
    let failed = || staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE);
    let refused = tx(c, |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, &actor, d.async_job_id, failed())
    })
    .unwrap();
    assert_command_refused(&refused, "RECONCILIATION_IN_PROGRESS");
    do_resolve(c, &claim, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    let replay = tx(c, |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, &actor, d.async_job_id, failed())
    })
    .unwrap();
    assert!(replay.replayed);
    assert_command_refused(&replay, "RECONCILIATION_IN_PROGRESS");
    let other_inputs = staff_evidence(StaffOutcome::Cancelled, NO_EFFECT_EVIDENCE);
    assert_code(
        tx(c, |c| {
            resolve_reconciliation_command_on(c, &REGISTRY, &actor, d.async_job_id, other_inputs)
        }),
        "ASYNC_COMMAND_REQUEST_CONFLICT",
    );
    // A new request id applies.
    assert_applied(&do_staff(c, d.async_job_id, failed()), "NO_EFFECT_PROVEN");
    // Request ids are per command kind.
    let redact = tx(c, |c| {
        redact_job_diagnostics_command_on(c, &actor, d.async_job_id)
    })
    .unwrap();
    assert!(redact.is_applied() && !redact.replayed);
}

#[test]
fn commands_against_a_missing_job_write_nothing() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let missing = Uuid::new_v4();
    assert_code(
        tx(c, |c| {
            cancel_job_command_on(c, &REGISTRY, &staff("x"), missing)
        }),
        "ASYNC_JOB_NOT_FOUND",
    );
    assert_code(
        tx(c, |c| {
            retry_job_command_on(c, &REGISTRY, &staff("x"), missing)
        }),
        "ASYNC_JOB_NOT_FOUND",
    );
    assert_code(
        tx(c, |c| {
            resolve_reconciliation_command_on(
                c,
                &REGISTRY,
                &staff("x"),
                missing,
                staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
            )
        }),
        "ASYNC_JOB_NOT_FOUND",
    );
    assert_code(
        tx(c, |c| {
            redact_job_diagnostics_command_on(c, &staff("x"), missing)
        }),
        "ASYNC_JOB_NOT_FOUND",
    );
    // A malformed input on an unknown job is PAYLOAD_INVALID.
    let blank = StaffActor {
        reason: "  ".to_string(),
        ..staff("x")
    };
    assert_code(
        tx(c, |c| cancel_job_command_on(c, &REGISTRY, &blank, missing)),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_eq!(command_rows(c), 0);
    assert_eq!(count(c, "FROM public.async_job"), 0);
}

// ---------------------------------------------------------------------------
// Redaction (section 10.5)
// ---------------------------------------------------------------------------

/// A TEST_POLL job driven through every diagnostics column: confirmed with a
/// provider ref, waited with a checkpoint, resumed and failed with an error
/// detail (RR at EFFECT_CONFIRMED).
fn diagnostics_job(c: &mut PgConnection, id: &str) -> AsyncJob {
    let a = create(c, new_job(TEST_POLL, id, &[&key(NS_POLL)]));
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    tx(c, |c| {
        mark_effect_confirmed_on(c, &REGISTRY, &claim, Some("PROVIDER-SECRET-A"))
    })
    .unwrap();
    let request = WaitRequest {
        checkpoint: r#"{"cursor": "CHECKPOINT-SECRET"}"#.to_string(),
        resume_at: resume_in(c, 10),
        provider_ref: None,
    };
    tx(c, |c| wait_on(c, &REGISTRY, &claim, request)).unwrap();
    make_available(c, a.async_job_id);
    let claim = claim_job(c, &a);
    let request = FailRequest {
        error_detail: Some("DETAIL-SECRET".to_string()),
        ..fail_req(FailureClass::Indeterminate)
    };
    do_fail(c, &claim, request).unwrap();
    a
}

const SECRET_EVIDENCE: &str = r#"{"notApplied": true, "note": "EVIDENCE-SECRET"}"#;

#[test]
fn redaction_erases_finished_attempts_and_terminal_diagnostics_and_leaves_an_open_attempt() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let p = key(NS_POLL);
    let a = create(c, new_job(TEST_POLL, "redact", &[&p]));
    let id = a.async_job_id;
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    tx(c, |c| {
        mark_effect_confirmed_on(c, &REGISTRY, &claim, Some("PROVIDER-SECRET-A"))
    })
    .unwrap();
    let request = WaitRequest {
        checkpoint: r#"{"cursor": "CHECKPOINT-SECRET"}"#.to_string(),
        resume_at: resume_in(c, 10),
        provider_ref: Some("PROVIDER-SECRET-B".to_string()),
    };
    tx(c, |c| wait_on(c, &REGISTRY, &claim, request)).unwrap();
    make_available(c, id);
    let claim = claim_job(c, &a);

    // An open attempt is left unchanged and reported.
    let first = do_redact(c, id);
    assert_applied(&first, "OPEN_ATTEMPT_UNCHANGED");
    let rows = attempts(c, id);
    assert_eq!(
        rows[0].redaction_command_id,
        Some(first.async_control_command_id)
    );
    assert!(rows[0].redacted_at.is_some());
    assert_eq!(
        (
            &rows[0].provider_ref,
            &rows[0].error_detail,
            &rows[0].evidence
        ),
        (&None, &None, &None)
    );
    assert!(rows[1].finished_at.is_none() && rows[1].redacted_at.is_none());
    // A non-terminal job keeps its operational provider ref and checkpoint.
    let row = job(c, id);
    assert!(row.redacted_at.is_none());
    assert_eq!(row.provider_ref.as_deref(), Some("PROVIDER-SECRET-B"));
    assert_eq!(
        row.checkpoint.as_deref(),
        Some(r#"{"cursor": "CHECKPOINT-SECRET"}"#)
    );

    // Terminal: every finished attempt and the job's diagnostics are erased.
    let request = FailRequest {
        error_detail: Some("DETAIL-SECRET".to_string()),
        ..fail_req(FailureClass::Indeterminate)
    };
    do_fail(c, &claim, request).unwrap();
    let staff_result = do_staff(c, id, staff_evidence(StaffOutcome::Failed, SECRET_EVIDENCE));
    assert_applied(&staff_result, "CONFIRMED_EFFECT_NOT_APPLIED");
    let second = do_redact(c, id);
    assert_applied(&second, "REDACTED");
    let rows = attempts(c, id);
    assert_eq!(rows.len(), 3);
    for attempt in &rows {
        assert!(attempt.redacted_at.is_some());
        assert_eq!(
            (
                &attempt.provider_ref,
                &attempt.error_detail,
                &attempt.evidence
            ),
            (&None, &None, &None)
        );
    }
    assert_eq!(
        rows[0].redaction_command_id,
        Some(first.async_control_command_id)
    );
    assert_eq!(
        rows[2].redaction_command_id,
        Some(second.async_control_command_id)
    );
    // Outcome codes remain.
    assert_eq!(
        rows[2].outcome_code.as_deref(),
        Some("CONFIRMED_EFFECT_NOT_APPLIED")
    );
    let row = job(c, id);
    assert!(row.redacted_at.is_some());
    assert_eq!((row.provider_ref, row.checkpoint), (None, None));
    assert_eq!(
        row.last_outcome_code.as_deref(),
        Some("CONFIRMED_EFFECT_NOT_APPLIED")
    );
}

#[test]
fn redaction_repeated_is_applied_and_changes_no_row() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let a = diagnostics_job(c, "repeat");
    let id = a.async_job_id;
    do_staff(c, id, staff_evidence(StaffOutcome::Failed, SECRET_EVIDENCE));
    assert_applied(&do_redact(c, id), "REDACTED");
    let job_before = job(c, id);
    let attempts_before = attempts(c, id);
    let rows = command_rows(c);
    let again = do_redact(c, id);
    assert_applied(&again, "NOTHING_TO_REDACT");
    assert_eq!(job(c, id), job_before);
    assert_eq!(attempts(c, id), attempts_before);
    assert_eq!(command_rows(c), rows + 1);
}

#[test]
fn redaction_command_rows_contain_digests_only() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let request_of = |c: &mut PgConnection, command: Uuid| {
        text(
            c,
            &format!(
                "SELECT request::text AS value FROM public.async_control_command \
                  WHERE async_control_command_id = '{command}'"
            ),
        )
        .unwrap()
    };
    let digest_of = |c: &mut PgConnection, json: &str| {
        text(
            c,
            &format!(
                "SELECT encode(sha256(convert_to('{}'::jsonb::text, 'UTF8')), 'hex') AS value",
                json.replace('\'', "''")
            ),
        )
        .unwrap()
    };

    // Evidence: the digest of its canonical form, never its content; an
    // equal object with other key order and spacing replays.
    let a = rr_confirmed(c, new_job(TEST_EFFECT, "digest-evidence", &[&key(NS_CS)]));
    let actor = staff_with(Uuid::new_v4());
    let result = tx(c, |c| {
        resolve_reconciliation_command_on(
            c,
            &REGISTRY,
            &actor,
            a.async_job_id,
            staff_evidence(StaffOutcome::Failed, SECRET_EVIDENCE),
        )
    })
    .unwrap();
    assert_applied(&result, "CONFIRMED_EFFECT_NOT_APPLIED");
    let request = request_of(c, result.async_control_command_id);
    let stored: serde_json::Value = serde_json::from_str(&request).unwrap();
    assert_eq!(
        stored["evidenceSha256"],
        json!(digest_of(c, SECRET_EVIDENCE))
    );
    assert_eq!(stored["outcome"], json!("FAILED"));
    assert!(!request.contains("EVIDENCE-SECRET") && !request.contains("notApplied"));
    let reordered = r#"{ "note":"EVIDENCE-SECRET",   "notApplied":true }"#;
    let replay = tx(c, |c| {
        resolve_reconciliation_command_on(
            c,
            &REGISTRY,
            &actor,
            a.async_job_id,
            staff_evidence(StaffOutcome::Failed, reordered),
        )
    })
    .unwrap();
    assert!(replay.replayed);

    // Entries: the digest of the sorted canonical list.
    let x = key(NS_CS);
    let b = rr_started(c, new_job(TEST_EFFECT, "digest-entries", &[&x]));
    let result = do_staff(
        c,
        b.async_job_id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "FP-SECRET")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    let request = request_of(c, result.async_control_command_id);
    let stored: serde_json::Value = serde_json::from_str(&request).unwrap();
    let canonical =
        json!([{"targetKey": x, "order": null, "fingerprint": "FP-SECRET", "sourceRef": null}])
            .to_string();
    assert_eq!(stored["entriesSha256"], json!(digest_of(c, &canonical)));
    assert!(!request.contains("FP-SECRET") && !request.contains(&x));

    // Checkpoint: its digest only.
    let d = rr_started(c, new_job(TEST_EFFECT, "digest-checkpoint", &[&key(NS_CS)]));
    let checkpoint = r#"{"cursor": "CHECKPOINT-SECRET"}"#;
    let resume_at = resume_in(c, 10);
    let result = do_staff(
        c,
        d.async_job_id,
        StaffResolution {
            checkpoint: Some(checkpoint.to_string()),
            resume_at: Some(resume_at),
            ..staff_res(StaffOutcome::Waiting)
        },
    );
    assert_applied(&result, "WAITING");
    let request = request_of(c, result.async_control_command_id);
    let stored: serde_json::Value = serde_json::from_str(&request).unwrap();
    assert_eq!(stored["checkpointSha256"], json!(digest_of(c, checkpoint)));
    assert!(!request.contains("CHECKPOINT-SECRET"));
    // Every stored digest is 64 hex characters.
    assert_eq!(
        count(
            c,
            "FROM public.async_control_command \
              WHERE command_kind = 'RESOLVE_RECONCILIATION' \
                AND NOT (coalesce(request->>'evidenceSha256', request->>'entriesSha256', \
                                  request->>'checkpointSha256') ~ '^[0-9a-f]{64}$')"
        ),
        0
    );
}

#[test]
fn redaction_leaves_no_redacted_value_in_any_stored_diagnostics_column() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let a = diagnostics_job(c, "sweep");
    let id = a.async_job_id;
    do_staff(c, id, staff_evidence(StaffOutcome::Failed, SECRET_EVIDENCE));
    let dump = |c: &mut PgConnection| {
        [
            format!("SELECT row_to_json(j)::text AS value FROM public.async_job j WHERE async_job_id = '{id}'"),
            format!(
                "SELECT string_agg(row_to_json(a)::text, ',') AS value \
                   FROM public.async_job_attempt a WHERE async_job_id = '{id}'"
            ),
            format!(
                "SELECT string_agg(row_to_json(k)::text, ',') AS value \
                   FROM public.async_control_command k WHERE async_job_id = '{id}'"
            ),
            format!(
                "SELECT string_agg(row_to_json(t)::text, ',') AS value \
                   FROM public.async_job_transition t WHERE async_job_id = '{id}'"
            ),
        ]
        .iter()
        .map(|sql| text(c, sql).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
    };
    let secrets = [
        "PROVIDER-SECRET-A",
        "CHECKPOINT-SECRET",
        "DETAIL-SECRET",
        "EVIDENCE-SECRET",
    ];
    let before = dump(c);
    for secret in secrets {
        assert!(before.contains(secret), "the fixture must store {secret}");
    }
    assert_applied(&do_redact(c, id), "REDACTED");
    let after = dump(c);
    for secret in secrets {
        assert!(
            !after.contains(secret),
            "{secret} survived redaction: {after}"
        );
    }
}

#[test]
fn redaction_identifier_columns_outside_redaction_refuse_spaces_and_257_characters() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let rb: &[(&str, &str)] = &[
        ("revision_mode", "'REVISION_BOUND'"),
        ("source_order", "1"),
        ("source_fingerprint", "'f'"),
    ];
    for (column, constraint, base) in [
        ("source_ref", "async_job_source_ref_check", None),
        ("causation_ref", "async_job_causation_ref_check", None),
        (
            "source_fingerprint",
            "async_job_source_fingerprint_check",
            Some(rb),
        ),
    ] {
        let at = |value: &'static str| -> Vec<(&str, &str)> {
            let mut overrides = base.map(<[_]>::to_vec).unwrap_or_default();
            overrides.push((column, value));
            overrides
        };
        check_bounds(
            c,
            "async_job",
            JOB_BASE,
            constraint,
            &[at("repeat('x', 256)")],
            &[at("'a b'"), at("repeat('x', 257)")],
        );
    }
    let state_base: &[(&str, &str)] = &[
        ("target_key", "'ns:k'"),
        ("applied_fingerprint", "'fp'"),
        ("last_job_id", "uuid_generate_v4()"),
        ("last_attempt_id", "uuid_generate_v4()"),
        ("updated_at", "now()"),
    ];
    for (column, constraint) in [
        (
            "applied_fingerprint",
            "async_effect_target_state_applied_fingerprint_check",
        ),
        (
            "applied_source_ref",
            "async_effect_target_state_applied_source_ref_check",
        ),
    ] {
        check_bounds(
            c,
            "async_effect_target_state",
            state_base,
            constraint,
            &[vec![(column, "repeat('x', 256)")]],
            &[vec![(column, "'a b'")], vec![(column, "repeat('x', 257)")]],
        );
    }
    // Redaction leaves them in place.
    let r = key(NS_RB);
    let input = NewAsyncJob {
        causation_ref: Some("cause-1".to_string()),
        ..revision_job("ident", &r, 2, "h2")
    };
    let (a, claim) = started(c, input);
    let entry = EvidenceEntry {
        source_ref: Some("applied-ref".to_string()),
        ..rb_entry(&r, 2, "h2")
    };
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::EffectApplied, vec![entry]),
    )
    .unwrap();
    assert_applied(&do_redact(c, a.async_job_id), "REDACTED");
    let row = job(c, a.async_job_id);
    assert_eq!(
        (
            row.source_ref.as_deref(),
            row.source_fingerprint.as_deref(),
            row.causation_ref.as_deref()
        ),
        (Some("ref-2"), Some("h2"), Some("cause-1"))
    );
    let s = state(c, &r).unwrap();
    assert_eq!(
        (
            s.applied_fingerprint.as_str(),
            s.applied_source_ref.as_deref()
        ),
        ("h2", Some("applied-ref"))
    );
}

// ---------------------------------------------------------------------------
// Recovery and retirement (section 10.6)
// ---------------------------------------------------------------------------

fn sorted(mut ids: Vec<Uuid>) -> Vec<Uuid> {
    ids.sort();
    ids
}

#[test]
fn recovery_expired_leases_and_deadlines_are_recovered_by_claims_and_by_recover_on() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // By a claim of the kind: an expired lease and a passed waiting deadline.
    let (a, _claim) = running(c, new_job(TEST_EFFECT, "by-claim-lease", &[&key(NS_CS)]));
    let (b, claim) = running(c, new_job(TEST_EFFECT, "by-claim-deadline", &[&key(NS_CS)]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(c, b.async_job_id, 1);
    expire_lease(c, a.async_job_id);
    // A due row of another kind is not this claim's to recover.
    let (p, claim) = running(c, new_job(TEST_PEER, "by-recover-lease", &[&key(NS_CS)]));
    expire_lease(c, p.async_job_id);
    assert!(dissemination_claim(c, &[TEST_EFFECT]).is_empty());
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::RetryScheduled);
    assert_eq!(status(c, b.async_job_id), AsyncJobStatus::Failed);
    assert_eq!(status(c, p.async_job_id), AsyncJobStatus::Running);
    // By `recover_on`: a lease and a waiting deadline.
    let (q, wait_claim) = running(
        c,
        new_job(TEST_STRICT, "by-recover-deadline", &[&key(NS_CS)]),
    );
    let resume_at = resume_in(c, 10);
    do_wait(c, &wait_claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(c, q.async_job_id, 1);
    let report = do_recover(c);
    assert_eq!(
        sorted(report.recovered),
        sorted(vec![p.async_job_id, q.async_job_id])
    );
    assert!(report.failed.is_empty() && report.deferred.is_empty());
    assert_eq!(status(c, p.async_job_id), AsyncJobStatus::RetryScheduled);
    assert_eq!(status(c, q.async_job_id), AsyncJobStatus::Failed);
    assert_code(do_renew(c, &claim), STALE);
    // Nothing left to recover.
    assert_eq!(do_recover(c), RecoveryReport::default());
}

#[test]
fn recovery_a_row_whose_recovery_fails_is_skipped_and_counted_without_aborting_the_claim() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let x = key(NS_CS);
    let (f, _claim) = started(c, new_job(TEST_EFFECT, "broken", &[&x]));
    let fid = f.async_job_id;
    expire_lease(c, fid);
    // Break the row behind the triggers: without its hold, T12 to RR fails
    // the forced hold check.
    bypass(
        c,
        &format!("DELETE FROM public.async_target_hold WHERE async_job_id = '{fid}'"),
    );
    let g = create(c, new_job(TEST_EFFECT, "healthy", &[&key(NS_CS)]));
    let claimed = dissemination_claim(c, &[TEST_EFFECT]);
    assert_eq!(
        claimed.iter().map(|j| j.async_job_id).collect::<Vec<_>>(),
        vec![g.async_job_id]
    );
    assert_eq!(status(c, fid), AsyncJobStatus::Running);
    let entry = |c: &mut PgConnection, n: i64| {
        count(
            c,
            &format!(
                "FROM public.async_recovery_failure WHERE async_job_id = '{fid}' \
                   AND failure_count = {n} AND error_code = 'ASYNC_JOB_INTEGRITY_VIOLATION'"
            ),
        )
    };
    assert_eq!(entry(c, 1), 1);
    let report = do_recover(c);
    assert_eq!(report.failed, vec![fid]);
    assert!(report.recovered.is_empty());
    assert_eq!(entry(c, 2), 1);
    let signals = signals_of(c, TEST_EFFECT);
    assert_eq!(signals.recovery_failure_count, 1);
    assert_eq!(
        signals.recovery_failure_codes,
        vec!["ASYNC_JOB_INTEGRITY_VIOLATION".to_string()]
    );
    assert!(signals.oldest_recovery_failure_at.is_some());
    // Repaired, the row recovers and its entry is deleted.
    bypass(
        c,
        &format!(
            "INSERT INTO public.async_target_hold (target_key, async_job_id) VALUES ('{x}', '{fid}')"
        ),
    );
    assert_eq!(do_recover(c).recovered, vec![fid]);
    assert_eq!(status(c, fid), AsyncJobStatus::ReconciliationRequired);
    assert_eq!(count(c, "FROM public.async_recovery_failure"), 0);
    let signals = signals_of(c, TEST_EFFECT);
    assert_eq!(signals.recovery_failure_count, 0);
    assert!(signals.recovery_failure_codes.is_empty());
    assert_no_deadlock(c, before);
}

#[test]
fn recovery_rows_of_a_kind_unknown_to_the_registry_recover_from_their_snapshot() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // T12 at PRE_WRITE: stored budget and backoff.
    let (a, _claim) = running(c, new_job(TEST_EFFECT, "u-pre", &[&key(NS_CS)]));
    // T12 after effect start.
    let (b, _claim) = started(c, new_job(TEST_PEER, "u-post", &[&key(NS_CS)]));
    // T12 at EFFECT_CONFIRMED of a resumable kind with a checkpoint: an
    // unknown kind is never resumable.
    let p = create(c, new_job(TEST_POLL, "u-poll", &[&key(NS_POLL)]));
    let claim = claim_job(c, &p);
    do_start(c, &claim).unwrap();
    do_confirm(c, &claim).unwrap();
    resumed(c, &p, &claim);
    // T11 at PRE_WRITE.
    let (w, claim) = running(c, new_job(TEST_STRICT, "u-wait", &[&key(NS_CS)]));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(c, w.async_job_id, 1);
    // T17.
    let r = rr_started(c, new_job(TEST_CAPABILITY, "u-recon", &[&key(NS_CS)]));
    claim_recon(c, &r);
    for id in [
        a.async_job_id,
        b.async_job_id,
        p.async_job_id,
        r.async_job_id,
    ] {
        expire_lease(c, id);
    }
    let report = tx(c, |c| recover_on(c, &NARROW_REGISTRY, None, 1000)).unwrap();
    assert_eq!(
        sorted(report.recovered),
        sorted(vec![
            a.async_job_id,
            b.async_job_id,
            p.async_job_id,
            w.async_job_id,
            r.async_job_id
        ])
    );
    let row = job(c, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::RetryScheduled);
    let finished = last_attempt(c, a.async_job_id).finished_at.unwrap();
    let delay = expected_backoff(c, &row, 1);
    assert_eq!(row.available_at, finished + Duration::seconds(delay));
    assert_eq!(
        status(c, b.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(
        status(c, p.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(status(c, w.async_job_id), AsyncJobStatus::Failed);
    let row = job(c, r.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert!(row.claimed_by.is_none());
    assert_eq!(
        last_attempt(c, r.async_job_id).result,
        Some(AsyncAttemptResult::LeaseExpired)
    );
    // At its stored budget an unknown kind's job fails.
    for _ in 0..2 {
        make_available(c, a.async_job_id);
        claim_job(c, &a);
        expire_lease(c, a.async_job_id);
        tx(c, |c| recover_on(c, &NARROW_REGISTRY, None, 1000)).unwrap();
    }
    let row = job(c, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::Failed);
    assert_eq!(
        row.last_outcome_code.as_deref(),
        Some("RETRY_BUDGET_EXHAUSTED")
    );
}

#[test]
fn recovery_an_admission_closed_kind_still_drains() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    assert_code(
        tx(c, |c| {
            create_job_on(
                c,
                &REGISTRY,
                new_job(TEST_RETIRED, "closed", &[&key(NS_CS)]),
            )
        }),
        "ASYNC_JOB_KIND_ADMISSION_CLOSED",
    );
    let x = key(NS_CS);
    let a = create_earlier(c, new_job(TEST_RETIRED, "drain", &[&x]));
    // Claims.
    claim_job(c, &a);
    // Recovery.
    expire_lease(c, a.async_job_id);
    assert_eq!(do_recover(c).recovered, vec![a.async_job_id]);
    assert_eq!(status(c, a.async_job_id), AsyncJobStatus::RetryScheduled);
    make_available(c, a.async_job_id);
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    // Reconciliation.
    let claim = claim_recon(c, &a);
    let row = do_resolve(c, &claim, resolution_entries(vec![cs_entry(&x, "fp")])).unwrap();
    assert_eq!(row.status, AsyncJobStatus::Succeeded);
}

#[test]
fn recovery_a_version_removed_from_creatable_versions_but_still_executable_is_claimed() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let v2 = || NewAsyncJob {
        payload_version: 2,
        ..new_job(TEST_VERSIONED, "v2", &[&key(NS_CS)])
    };
    assert_code(
        tx(c, |c| create_job_on(c, &REGISTRY, v2())),
        "ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED",
    );
    let a = create_earlier(c, v2());
    let claimed = dissemination_claim(c, &[TEST_VERSIONED]);
    assert_eq!(claimed.len(), 1);
    assert_eq!(
        (claimed[0].async_job_id, claimed[0].payload_version),
        (a.async_job_id, 2)
    );
}

#[test]
fn recovery_unknown_kind_and_non_executable_version_signals_count_premature_removals() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // A kind removed from the registry while it has non-terminal jobs.
    let a = create(c, new_job(TEST_EFFECT, "removed-kind", &[&key(NS_CS)]));
    let (_b, _claim) = running(c, new_job(TEST_EFFECT, "removed-kind-2", &[&key(NS_CS)]));
    let s = signals_with(c, &NARROW_REGISTRY, TEST_EFFECT);
    assert!(!s.known);
    assert_eq!(s.non_terminal_unknown_kind_count, 2);
    assert_eq!(
        signals_of(c, TEST_EFFECT).non_terminal_unknown_kind_count,
        0
    );
    do_cancel(c, a.async_job_id);
    do_cancel(c, _b.async_job_id);
    assert_eq!(
        signals_with(c, &NARROW_REGISTRY, TEST_EFFECT).non_terminal_unknown_kind_count,
        0
    );
    // A version removed from `executable_versions` while it has jobs.
    let v3 = create_earlier(
        c,
        NewAsyncJob {
            payload_version: 3,
            ..new_job(TEST_VERSIONED, "v3", &[&key(NS_CS)])
        },
    );
    let s = signals_of(c, TEST_VERSIONED);
    assert!(s.known);
    assert_eq!(s.non_terminal_version_not_executable_count, 1);
    assert_eq!(
        signals_with(c, &EARLIER_REGISTRY, TEST_VERSIONED)
            .non_terminal_version_not_executable_count,
        0
    );
    assert_applied(&do_cancel(c, v3.async_job_id), "CANCELLED");
    assert_eq!(
        signals_of(c, TEST_VERSIONED).non_terminal_version_not_executable_count,
        0
    );
}

// ---------------------------------------------------------------------------
// Signals (section 15): each signal set by one fixture and cleared by another
// ---------------------------------------------------------------------------

#[test]
fn signals_job_state_counts_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let s = |c: &mut PgConnection| signals_of(c, TEST_EFFECT);
    let x = key(NS_CS);
    let v = s(c);
    assert_eq!(
        (
            v.ready_count,
            v.running_count,
            v.retry_scheduled_count,
            v.waiting_count
        ),
        (0, 0, 0, 0)
    );
    assert_eq!((v.reconciliation_required_count, v.overdue_count), (0, 0));

    // Ready.
    let a = create(c, new_job(TEST_EFFECT, "states", &[&x]));
    let id = a.async_job_id;
    let v = s(c);
    assert_eq!(v.ready_count, 1);
    assert_eq!(v.oldest_ready_at, Some(a.available_at));
    assert!(v.oldest_ready_age_seconds.is_some());
    // Running; ready cleared.
    claim_job(c, &a);
    let v = s(c);
    assert_eq!(
        (v.ready_count, v.running_count, v.running_past_lease_count),
        (0, 1, 0)
    );
    assert_eq!(
        (v.oldest_ready_at, v.oldest_ready_age_seconds),
        (None, None)
    );
    // Running past its lease, then overdue.
    expire_lease(c, id);
    let v = s(c);
    assert_eq!((v.running_past_lease_count, v.overdue_count), (1, 0));
    expire_lease_ago(c, id, 11);
    assert_eq!(s(c).overdue_count, 1);
    // Retry scheduled; running, past lease and overdue cleared.
    do_recover(c);
    let v = s(c);
    assert_eq!(
        (
            v.running_count,
            v.running_past_lease_count,
            v.overdue_count,
            v.retry_scheduled_count
        ),
        (0, 0, 0, 1)
    );
    assert_eq!(v.ready_count, 0);
    make_available(c, id);
    assert_eq!(s(c).ready_count, 1);
    // Waiting; retry scheduled cleared.
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    let resume_at = resume_in(c, 10);
    let WaitOutcome::Waiting(row) = do_wait(c, &claim, "{}", resume_at).unwrap() else {
        panic!("expected WAITING");
    };
    let v = s(c);
    assert_eq!(
        (v.retry_scheduled_count, v.waiting_count, v.ready_count),
        (0, 1, 0)
    );
    assert_eq!(v.oldest_waiting_deadline, row.waiting_deadline);
    // Waiting past its deadline by more than 10 minutes: overdue.
    expire_waiting_deadline(c, id, 660);
    assert_eq!(s(c).overdue_count, 1);
    // RR (T11 after start); waiting and overdue cleared.
    do_recover(c);
    let v = s(c);
    assert_eq!(
        (
            v.waiting_count,
            v.overdue_count,
            v.reconciliation_required_count
        ),
        (0, 0, 1)
    );
    assert_eq!(v.oldest_waiting_deadline, None);
    assert!(v.oldest_reconciliation_required_age_seconds.is_some());
    // RR cleared by its resolution.
    do_staff(
        c,
        id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
    );
    let v = s(c);
    assert_eq!(v.reconciliation_required_count, 0);
    assert_eq!(v.oldest_reconciliation_required_age_seconds, None);
}

#[test]
fn signals_blocked_jobs_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "holder", &[&x]));
    let b = create(c, new_job(TEST_PEER, "blocked", &[&x]));
    let v = signals_of(c, TEST_PEER);
    assert_eq!(v.blocked_count, 1);
    assert_eq!(
        v.blocked,
        vec![BlockedJob {
            async_job_id: b.async_job_id,
            target_key: x.clone(),
            holder_job_id: a.async_job_id,
            holder_kind: TEST_EFFECT.to_string(),
        }]
    );
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp")]),
    )
    .unwrap();
    let v = signals_of(c, TEST_PEER);
    assert_eq!(v.blocked_count, 0);
    assert!(v.blocked.is_empty());
}

#[test]
fn signals_ready_jobs_whose_version_no_executor_polled_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // A peer job holds the target, so polls below claim nothing.
    let x = key(NS_CS);
    let (_holder, _claim) = running(c, new_job(TEST_PEER, "holder", &[&x]));
    create(c, new_job(TEST_EFFECT, "unpolled", &[&x]));
    let v = signals_of(c, TEST_EFFECT);
    assert_eq!(
        (v.ready_unpolled_count, v.unpolled_versions.clone()),
        (1, vec![1])
    );
    // A poll for another version leaves it set.
    let only_v2 = ClaimRequest {
        kinds: vec![(TEST_EFFECT.to_string(), vec![2])],
        max_jobs: 1,
        lease_seconds: None,
    };
    tx(c, |c| claim_on(c, &REGISTRY, &dissemination(), only_v2)).unwrap();
    assert_eq!(signals_of(c, TEST_EFFECT).ready_unpolled_count, 1);
    // A poll for its version clears it.
    assert!(dissemination_claim(c, &[TEST_EFFECT]).is_empty());
    let v = signals_of(c, TEST_EFFECT);
    assert_eq!((v.ready_unpolled_count, v.ready_count), (0, 1));
    assert!(v.unpolled_versions.is_empty());
    // A poll older than an hour sets it again.
    exec(
        c,
        "UPDATE public.async_executor_poll SET last_poll_at = now() - interval '2 hours'",
    );
    assert_eq!(signals_of(c, TEST_EFFECT).ready_unpolled_count, 1);
}

/// A REVISION_BOUND target whose stored revision is `stored`, and an RR job
/// for the older revision `older` that wrote it (T6x with a regression).
fn regressed_job(c: &mut PgConnection, stored: i64, older: i64) -> (String, AsyncJob) {
    let k = key(NS_RB);
    seed_rb_state(c, &k, stored, &format!("h{stored}"));
    let fingerprint = format!("h{older}");
    let (a, claim) = started(
        c,
        revision_job(&format!("older-{older}"), &k, older, &fingerprint),
    );
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, older, &fingerprint)],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    (k, a)
}

#[test]
fn signals_accepted_regressions_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (k, a) = regressed_job(c, 9, 4);
    assert!(signals_of(c, TEST_REVISION).regressed_targets.is_empty());
    let result = do_staff(
        c,
        a.async_job_id,
        staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(4)),
    );
    assert_applied(&result, "REGRESSION_ACCEPTED");
    let v = signals_of(c, TEST_REVISION);
    assert_eq!(v.regressed_targets.len(), 1);
    let target = &v.regressed_targets[0];
    assert_eq!(target.target_key, k);
    assert_eq!(
        (
            target.applied_order,
            target.regressed_from_order,
            target.regressed_from_fingerprint.as_deref()
        ),
        (Some(4), Some(9), Some("h9"))
    );
    // An ordinary write above the pre-acceptance revision clears it.
    let (_b, claim) = started(c, revision_job("newer", &k, 10, "h10"));
    do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 10, "h10")],
        ),
    )
    .unwrap();
    assert!(signals_of(c, TEST_REVISION).regressed_targets.is_empty());
}

#[test]
fn signals_terminal_counts_and_attempt_durations_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let v = signals_of(c, TEST_EFFECT);
    assert_eq!(
        (
            v.succeeded_last_24h,
            v.failed_last_24h,
            v.cancelled_last_24h
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (
            v.attempt_duration_p50_seconds,
            v.attempt_duration_p95_seconds
        ),
        (None, None)
    );
    let (a, claim) = running(c, new_job(TEST_EFFECT, "done", &[&key(NS_CS)]));
    let entries = vec![cs_entry(&a.target_keys[0], "fp")];
    do_complete(c, &claim, completion(CompletionOutcome::NoOp, entries)).unwrap();
    let (_b, claim) = running(c, new_job(TEST_EFFECT, "failed", &[&key(NS_CS)]));
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    let d = create(c, new_job(TEST_EFFECT, "cancelled", &[&key(NS_CS)]));
    do_cancel(c, d.async_job_id);
    let v = signals_of(c, TEST_EFFECT);
    assert_eq!(
        (
            v.succeeded_last_24h,
            v.failed_last_24h,
            v.cancelled_last_24h
        ),
        (1, 1, 1)
    );
    assert!(v.attempt_duration_p50_seconds.is_some() && v.attempt_duration_p95_seconds.is_some());
    // Outside the 24-hour window (the clock moved behind the triggers).
    bypass(
        c,
        "UPDATE public.async_job SET terminal_at = terminal_at - interval '25 hours', \
                updated_at = updated_at - interval '25 hours'; \
         UPDATE public.async_job_attempt SET started_at = started_at - interval '25 hours', \
                last_heartbeat_at = last_heartbeat_at - interval '25 hours', \
                finished_at = finished_at - interval '25 hours'",
    );
    let v = signals_of(c, TEST_EFFECT);
    assert_eq!(
        (
            v.succeeded_last_24h,
            v.failed_last_24h,
            v.cancelled_last_24h
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (
            v.attempt_duration_p50_seconds,
            v.attempt_duration_p95_seconds
        ),
        (None, None)
    );
}

#[test]
fn signals_executor_activity_set_and_clear() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    assert!(executor_activity(c).unwrap().is_empty());
    let (a, claim) = running(c, new_job(TEST_EFFECT, "activity", &[&key(NS_CS)]));
    let find = |c: &mut PgConnection| {
        executor_activity(c)
            .unwrap()
            .into_iter()
            .find(|row| row.kind == TEST_EFFECT && row.executor_name == "dissemination-1")
    };
    let row = find(c).unwrap();
    assert_eq!(row.executor_principal, "machine:dissemination");
    assert!(row.last_execution_poll_at.is_some() && row.last_execution_claim_at.is_some());
    assert!(row.last_reconciliation_poll_at.is_none());
    assert_eq!(
        row.latest_open_heartbeat_at,
        Some(last_attempt(c, a.async_job_id).last_heartbeat_at)
    );
    // A renewal moves the heartbeat.
    do_renew(c, &claim).unwrap();
    let heartbeat = last_attempt(c, a.async_job_id).last_heartbeat_at;
    assert_eq!(find(c).unwrap().latest_open_heartbeat_at, Some(heartbeat));
    // A reconciliation poll.
    tx(c, |c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 1))
    })
    .unwrap();
    assert!(find(c).unwrap().last_reconciliation_poll_at.is_some());
    // Closing the attempt clears the open heartbeat.
    do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_eq!(find(c).unwrap().latest_open_heartbeat_at, None);
    // Without poll rows or open attempts the identity is gone.
    exec(c, "DELETE FROM public.async_executor_poll");
    assert!(executor_activity(c).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Mapping (E1) and carry-forward part A: real lock and statement timeouts, a
// real deadlock, a cancel and a terminated backend are ASYNC_JOB_BUSY; any
// other database error is ASYNC_JOB_INTEGRITY_VIOLATION with its fixed
// message; no PostgreSQL text reaches the caller
// ---------------------------------------------------------------------------

/// A RUNNING job on one fresh target and a session holding its row lock.
fn row_locked_job(c: &mut PgConnection, id: &str) -> (AsyncJob, WorkerClaim, Session) {
    let (created, claim) = running(c, new_job(TEST_EFFECT, id, &[&key(NS_CS)]));
    let job_id = created.async_job_id;
    let current = job(c, job_id);
    let holder = Session::open();
    holder.run(move |c| exec(c, &lock_row_sql(job_id)));
    (current, claim, holder)
}

fn lock_row_sql(id: Uuid) -> String {
    format!("SELECT 1 FROM public.async_job WHERE async_job_id = '{id}' FOR UPDATE")
}

/// A renewal of `claim` inside the session's transaction, as its outcome.
fn renew_step(claim: WorkerClaim) -> impl FnOnce(&mut PgConnection) -> ThothResult<()> {
    move |c: &mut PgConnection| renew_on(c, &REGISTRY, &claim, None).map(|_| ())
}

/// The error is `ASYNC_JOB_BUSY` and its client text carries no server text.
#[track_caller]
fn assert_busy(result: ThothResult<()>) {
    match result {
        Err(error) => {
            assert_eq!(code(&error), "ASYNC_JOB_BUSY", "{error:?}");
            assert_eq!(
                error.to_string(),
                "The async job engine is busy. Retry later."
            );
        }
        Ok(()) => panic!("expected ASYNC_JOB_BUSY"),
    }
}

/// Wait, by bounded polling, until the deadlock counter reaches `expected`
/// (a backend reports its statistics when it goes idle or exits).
fn wait_for_deadlocks(conn: &mut PgConnection, expected: i64) {
    let deadline = Instant::now() + StdDuration::from_secs(60);
    loop {
        let now = deadlocks(conn);
        if now == expected {
            return;
        }
        assert!(now < expected, "more deadlocks than the one forced: {now}");
        assert!(
            Instant::now() < deadline,
            "the forced deadlock was never counted"
        );
        thread::yield_now();
    }
}

/// The lease is unchanged and nothing was written for the job.
#[track_caller]
fn assert_untouched(conn: &mut PgConnection, before: &AsyncJob) {
    let after = job(conn, before.async_job_id);
    assert_eq!(after.status, before.status);
    assert_eq!(after.lease_expires_at, before.lease_expires_at);
    assert_eq!(after.updated_at, before.updated_at);
}

#[test]
fn mapping_a_real_lock_timeout_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let (job_row, claim, holder) = row_locked_job(c, "lock-timeout");
    let caller = Session::open_with("SET LOCAL lock_timeout = '100ms'");
    assert_busy(caller.run(renew_step(claim)));
    caller.commit();
    holder.commit();
    assert_untouched(c, &job_row);
    assert_no_deadlock(c, before);
}

#[test]
fn mapping_a_real_statement_timeout_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let (job_row, claim, holder) = row_locked_job(c, "statement-timeout");
    let caller = Session::open_with("SET LOCAL statement_timeout = '100ms'");
    assert_busy(caller.run(renew_step(claim)));
    caller.commit();
    holder.commit();
    assert_untouched(c, &job_row);
    assert_no_deadlock(c, before);
}

#[test]
fn mapping_a_query_cancel_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let (job_row, claim, holder) = row_locked_job(c, "cancel");
    let caller = Session::open();
    caller.send(renew_step(claim));
    wait_until_blocked(c, caller.pid);
    exec(c, &format!("SELECT pg_cancel_backend({})", caller.pid));
    assert_busy(caller.result());
    caller.commit();
    holder.commit();
    assert_untouched(c, &job_row);
    assert_no_deadlock(c, before);
}

/// Carry-forward A3: the FATAL "terminating connection due to administrator
/// command" of a terminated backend is a closed connection, so BUSY.
#[test]
fn mapping_a_terminated_backend_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let (job_row, claim, holder) = row_locked_job(c, "terminate");
    let caller = Session::open();
    caller.send(renew_step(claim));
    wait_until_blocked(c, caller.pid);
    exec(
        c,
        &format!("SELECT pg_terminate_backend({}, 10000)", caller.pid),
    );
    assert_busy(caller.result());
    assert!(
        caller.end().is_err(),
        "the terminated session cannot commit"
    );
    holder.commit();
    assert_untouched(c, &job_row);
    assert_no_deadlock(c, before);
}

#[test]
fn mapping_a_real_deadlock_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (created, claim) = running(c, new_job(TEST_EFFECT, "deadlock", &[&x]));
    let id = created.async_job_id;
    let job_row = job(c, id);
    let before = deadlocks(c);
    // The engine's caller has the short deadlock timeout, so it is the one
    // that detects the cycle; the other side would wait a minute.
    let engine = Session::open_with("SET LOCAL deadlock_timeout = '100ms'");
    let other = Session::open_with("SET LOCAL deadlock_timeout = '60s'");
    // The engine's transaction already holds the job row; the other holds
    // the hold lock of x and waits for the row.
    engine.run(move |c| exec(c, &lock_row_sql(id)));
    let lock_x = format!("SELECT pg_advisory_xact_lock({NS_HOLD}, hashtext('{x}'))");
    other.run(move |c| exec(c, &lock_x));
    other.send(move |c| c.batch_execute(&lock_row_sql(id)).is_ok());
    wait_until_blocked(c, other.pid);
    // The release waits for x's hold lock (rule H): a cycle, detected by the
    // engine's side.
    assert_busy(engine.run(move |c| release_on(c, &REGISTRY, &claim).map(|_| ())));
    engine.commit();
    assert!(other.result::<bool>(), "the other side gets the row");
    other.commit();
    wait_for_deadlocks(c, before + 1);
    assert_untouched(c, &job_row);
    assert_eq!(holds(c, id), vec![x]);
}

#[test]
fn mapping_another_database_error_is_an_integrity_violation_with_its_fixed_message() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (created, claim) = running(c, new_job(TEST_EFFECT, "read-only", &[&key(NS_CS)]));
    let job_row = job(c, created.async_job_id);
    // A read-only caller transaction: the row lock fails with an error no
    // async constraint or trigger raises.
    let caller = Session::open_with("SET LOCAL transaction_read_only = on");
    let error = caller
        .run(renew_step(claim))
        .expect_err("a read-only transaction cannot lock the row");
    caller.commit();
    assert_eq!(error, ThothError::AsyncJobIntegrityViolation);
    assert_eq!(
        error.to_string(),
        "The async job operation failed an integrity check."
    );
    assert_untouched(c, &job_row);
}

/// Carry-forward A1 and A3 at the wrapper's commit: a failed send, a closed
/// connection and an administrator termination are BUSY whatever kind Diesel
/// reports; every other commit failure is an integrity violation.
#[test]
fn carry_a_failed_send_or_closed_connection_at_commit_is_busy() {
    let error = |kind: DatabaseErrorKind, message: &str| {
        DieselError::DatabaseError(kind, Box::new(message.to_string()))
    };
    for (kind, message) in [
        (
            DatabaseErrorKind::UnableToSendCommand,
            "could not send data to server",
        ),
        (
            DatabaseErrorKind::ClosedConnection,
            "server closed the connection unexpectedly",
        ),
        (
            DatabaseErrorKind::Unknown,
            "terminating connection due to administrator command",
        ),
        (
            DatabaseErrorKind::Unknown,
            "canceling statement due to lock timeout",
        ),
        (
            DatabaseErrorKind::Unknown,
            "canceling statement due to statement timeout",
        ),
        (DatabaseErrorKind::Unknown, "deadlock detected"),
        (
            DatabaseErrorKind::Unknown,
            "canceling statement due to user request",
        ),
    ] {
        assert_eq!(
            map_commit_error(error(kind, message)),
            ThothError::AsyncJobBusy,
            "{message}"
        );
    }
    let rollback_after_send = DieselError::RollbackErrorOnCommit {
        rollback_error: Box::new(error(
            DatabaseErrorKind::ClosedConnection,
            "server closed the connection unexpectedly",
        )),
        commit_error: Box::new(error(
            DatabaseErrorKind::UnableToSendCommand,
            "could not send data to server",
        )),
    };
    assert_eq!(
        map_commit_error(rollback_after_send),
        ThothError::AsyncJobBusy
    );
    assert_eq!(
        map_commit_error(error(
            DatabaseErrorKind::CheckViolation,
            "new row violates check constraint"
        )),
        ThothError::AsyncJobIntegrityViolation
    );
}

/// Carry-forward A4: the backend is terminated between the engine call and
/// the wrapper's commit. The commit fails, the wrapper answers BUSY, and the
/// re-read shows what committed (here nothing).
#[test]
fn carry_a_backend_terminated_before_the_wrappers_commit_is_busy() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let (created, claim) = running(c, new_job(TEST_EFFECT, "commit-lost", &[&key(NS_CS)]));
    let job_row = job(c, created.async_job_id);
    let (called, called_rx) = mpsc::channel();
    let (go, go_rx) = mpsc::channel::<()>();
    let (pid, handle) = spawn_with_connection(move |conn| {
        with_engine_transaction_on(conn, |c| {
            let renewed = renew_on(c, &REGISTRY, &claim, Some(120)).map(|_| ());
            called
                .send(renewed.is_ok())
                .expect("announce the engine call");
            go_rx.recv().expect("wait for the termination");
            renewed
        })
    });
    assert!(
        called_rx.recv().expect("the engine call ran"),
        "the engine call itself succeeded"
    );
    exec(c, &format!("SELECT pg_terminate_backend({pid}, 10000)"));
    go.send(()).expect("let the wrapper commit");
    assert_busy(handle.join().expect("the wrapper thread panicked"));
    // The outcome was unknown to the caller; the re-read shows that the
    // renewal never committed.
    assert_untouched(c, &job_row);
    assert_no_deadlock(c, before);
}

/// Carry-forward A2: after a BUSY with an unknown outcome, a retried
/// token-fenced worker call answers STALE_CLAIM if the first commit
/// succeeded; a staff command replays by request id; a lost claim is
/// recovered at lease expiry.
#[test]
fn carry_after_an_unknown_outcome_each_caller_retries_safely() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // A completion whose commit succeeded, retried with the same token.
    let x = key(NS_CS);
    let (_a, claim) = running(c, new_job(TEST_EFFECT, "retried-worker", &[&x]));
    let request = completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp")]);
    do_complete(c, &claim, request.clone()).unwrap();
    assert_code(do_complete(c, &claim, request), "ASYNC_JOB_STALE_CLAIM");
    assert_code(do_renew(c, &claim), "ASYNC_JOB_STALE_CLAIM");
    assert_code(
        do_fail(c, &claim, fail_req(FailureClass::Permanent)),
        "ASYNC_JOB_STALE_CLAIM",
    );
    // A staff command repeated with its request id replays its outcome.
    let b = create(c, new_job(TEST_EFFECT, "retried-command", &[&key(NS_CS)]));
    let actor = staff("cancel, retried after BUSY");
    let first = tx(c, |c| {
        cancel_job_command_on(c, &REGISTRY, &actor, b.async_job_id)
    })
    .unwrap();
    let rows = command_rows(c);
    let again = tx(c, |c| {
        cancel_job_command_on(c, &REGISTRY, &actor, b.async_job_id)
    })
    .unwrap();
    assert!(!first.replayed && again.replayed);
    assert_eq!(
        (
            again.async_control_command_id,
            again.outcome,
            again.outcome_code.clone()
        ),
        (
            first.async_control_command_id,
            first.outcome,
            first.outcome_code.clone()
        )
    );
    assert_eq!(command_rows(c), rows);
    // A claim whose result was lost is recovered at lease expiry.
    let d = create(c, new_job(TEST_EFFECT, "lost-claim", &[&key(NS_CS)]));
    let _lost = claim_job(c, &d);
    expire_lease(c, d.async_job_id);
    assert_eq!(do_recover(c).recovered, vec![d.async_job_id]);
    let recovered = job(c, d.async_job_id);
    assert_eq!(
        (recovered.status, recovered.prewrite_failure_count),
        (AsyncJobStatus::RetryScheduled, 1)
    );
    assert!(holds(c, d.async_job_id).is_empty());
    assert_no_unspecified(c);
}

/// Carry-forward A5: T16 and T18 SUCCEEDED entries follow the EFFECT_APPLIED
/// row, so they may carry a `sourceRef`.
#[test]
fn carry_t16_and_t18_succeeded_entries_may_carry_a_source_ref() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let with_ref = |target: &str, fingerprint: &str, source_ref: &str| EvidenceEntry {
        source_ref: Some(source_ref.to_string()),
        ..cs_entry(target, fingerprint)
    };
    let x = key(NS_CS);
    let a = rr_started(c, new_job(TEST_EFFECT, "t16-ref", &[&x]));
    let claim = claim_recon(c, &a);
    let done = do_resolve(
        c,
        &claim,
        resolution_entries(vec![with_ref(&x, "f16", "ref-16")]),
    )
    .unwrap();
    assert_eq!(done.status, AsyncJobStatus::Succeeded);
    assert_eq!(
        state(c, &x).unwrap().applied_source_ref.as_deref(),
        Some("ref-16")
    );
    let y = key(NS_CS);
    let b = rr_started(c, new_job(TEST_EFFECT, "t18-ref", &[&y]));
    let result = do_staff(
        c,
        b.async_job_id,
        staff_entries(StaffOutcome::Succeeded, vec![with_ref(&y, "f18", "ref-18")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    assert_eq!(
        state(c, &y).unwrap().applied_source_ref.as_deref(),
        Some("ref-18")
    );
    // A REVISION_BOUND reconciler entry with a sourceRef too.
    let z = key(NS_RB);
    let (r, claim) = started(c, revision_job("t16-rb-ref", &z, 3, "h3"));
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    let claim = claim_recon(c, &r);
    let entry = EvidenceEntry {
        source_ref: Some("ref-3".to_string()),
        ..rb_entry(&z, 3, "h3")
    };
    let done = do_resolve(c, &claim, resolution_entries(vec![entry])).unwrap();
    assert_eq!(done.status, AsyncJobStatus::Succeeded);
    assert_eq!(state(c, &z).unwrap().applied_order, Some(3));
    assert_no_unspecified(c);
}

// ---------------------------------------------------------------------------
// Time (E1): `now` is clock_timestamp() read after the row lock
// ---------------------------------------------------------------------------

/// Move the job's lease, from a session that holds its row lock, to
/// `extra` plus the midpoint between the waiting backend's transaction start
/// (its `now()`) and the clock: live by `now()`, but past by the time the
/// waiter gets the lock.
fn lease_between_start_and_lock_sql(id: Uuid, waiter: i32, extra_seconds: i64) -> String {
    format!(
        "UPDATE public.async_job j \
            SET lease_expires_at = s.xact_start + interval '{extra_seconds} seconds' \
                                 + (clock_timestamp() - s.xact_start) / 2 \
           FROM pg_stat_activity s \
          WHERE s.pid = {waiter} AND j.async_job_id = '{id}'"
    )
}

#[test]
fn time_a_lease_that_expires_while_a_call_waits_for_the_row_lock_is_expired() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    // Renewal: the lease is live at the caller's transaction start.
    let (job_row, claim, holder) = row_locked_job(c, "time-renew");
    let id = job_row.async_job_id;
    let caller = Session::open();
    caller.send(renew_step(claim));
    wait_until_blocked(c, caller.pid);
    let update = lease_between_start_and_lock_sql(id, caller.pid, 0);
    holder.run(move |c| exec(c, &update));
    holder.commit();
    assert_code(
        caller.result::<ThothResult<()>>(),
        "ASYNC_JOB_LEASE_EXPIRED",
    );
    caller.commit();
    // The effect-window check uses the same instant: 30 s of window plus
    // half the wait were left at the transaction start, less than 30 s at
    // the lock.
    let (job_row, claim, holder) = row_locked_job(c, "time-window");
    let id = job_row.async_job_id;
    let caller = Session::open();
    caller.send(move |c| mark_effect_started_on(c, &REGISTRY, &claim).map(|_| ()));
    wait_until_blocked(c, caller.pid);
    let update = lease_between_start_and_lock_sql(id, caller.pid, 30);
    holder.run(move |c| exec(c, &update));
    holder.commit();
    assert_code(
        caller.result::<ThothResult<()>>(),
        "ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT",
    );
    caller.commit();
    assert_eq!(job(c, id).effect_phase, AsyncEffectPhase::PreWrite);
    assert_no_deadlock(c, before);
}

// ---------------------------------------------------------------------------
// Phase rules (E1)
// ---------------------------------------------------------------------------

#[test]
fn phase_fail_indeterminate_at_pre_write_is_a_retryable_pre_write_failure() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (a, claim) = running(c, new_job(TEST_EFFECT, "indeterminate-pre", &[&key(NS_CS)]));
    let failed = do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    assert_eq!(
        (failed.status, failed.prewrite_failure_count),
        (AsyncJobStatus::RetryScheduled, 1)
    );
    let last = last_attempt(c, a.async_job_id);
    assert_eq!(last.result, Some(AsyncAttemptResult::RetryableFailure));
    assert!(holds(c, a.async_job_id).is_empty());
    // Without room it ends FAILED with the budget exhausted (T8).
    for _ in 1..3 {
        make_available(c, a.async_job_id);
        let claim = claim_job(c, &a);
        do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    }
    let done = job(c, a.async_job_id);
    assert_eq!(
        (done.status, done.prewrite_failure_count),
        (AsyncJobStatus::Failed, 3)
    );
    assert_eq!(
        last_attempt(c, a.async_job_id).outcome_code.as_deref(),
        Some("RETRY_BUDGET_EXHAUSTED")
    );
    assert_eq!(
        causes(c, a.async_job_id),
        vec![
            "T1_CREATE",
            "T2_CLAIM",
            "T7_FAIL_RETRYABLE",
            "T2_CLAIM",
            "T7_FAIL_RETRYABLE",
            "T2_CLAIM",
            "T8_FAIL"
        ]
    );
}

#[test]
fn phase_t9n_is_refused_at_effect_confirmed() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (a, claim) = confirmed(c, new_job(TEST_EFFECT, "t9n-confirmed", &[&key(NS_CS)]));
    assert_refused(
        do_fail(
            c,
            &claim,
            no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
        ),
        "EFFECT_CONFIRMED",
    );
    assert_live(c, &claim);
    // Without the flag it is T9.
    let rr = do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_eq!(
        (rr.status, rr.effect_phase),
        (
            AsyncJobStatus::ReconciliationRequired,
            AsyncEffectPhase::EffectConfirmed
        )
    );
    assert_eq!(holds(c, a.async_job_id), a.target_keys);
}

#[test]
fn phase_wait_with_resume_at_after_the_waiting_deadline_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (a, claim) = started(c, new_job(TEST_EFFECT, "wait-late", &[&key(NS_CS)]));
    // No stored deadline: the bound is now + horizon (3600 s).
    let too_late = resume_in(c, 3700);
    assert_code(
        do_wait(c, &claim, "{}", too_late),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_live(c, &claim);
    let resume_at = resume_in(c, 10);
    let WaitOutcome::Waiting(waiting) = do_wait(c, &claim, "{}", resume_at).unwrap() else {
        panic!("expected WAITING");
    };
    let deadline = waiting.waiting_deadline.unwrap();
    // A resumed attempt is bound by the stored deadline.
    make_available(c, a.async_job_id);
    let claim = claim_job(c, &a);
    let past_deadline = deadline + Duration::seconds(1);
    assert_code(
        do_wait(c, &claim, "{}", past_deadline),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_live(c, &claim);
    assert_eq!(job(c, a.async_job_id).status, AsyncJobStatus::Running);
}

// ---------------------------------------------------------------------------
// Input bounds (19.2)
// ---------------------------------------------------------------------------

/// Everything an engine call could write for one job: the row, its attempts,
/// transitions and holds, its targets' state rows, and the command rows.
#[derive(Debug, Clone, PartialEq)]
struct Footprint {
    job: AsyncJob,
    attempts: Vec<AsyncJobAttempt>,
    transitions: Vec<AsyncJobTransition>,
    holds: Vec<String>,
    states: Vec<Option<StateView>>,
    commands: i64,
}

fn footprint(conn: &mut PgConnection, id: Uuid) -> Footprint {
    let row = job(conn, id);
    let states = row.target_keys.iter().map(|k| state(conn, k)).collect();
    Footprint {
        attempts: attempts(conn, id),
        transitions: transitions(conn, id),
        holds: holds(conn, id),
        states,
        commands: command_rows(conn),
        job: row,
    }
}

/// Nothing was written for the job, and the claim is still live.
#[track_caller]
fn assert_nothing_written_and_live(
    conn: &mut PgConnection,
    before: &Footprint,
    claim: &WorkerClaim,
) {
    assert_eq!(&footprint(conn, before.job.async_job_id), before);
    assert_live(conn, claim);
}

/// Run `op` in a caller-owned READ COMMITTED transaction and assert it is
/// refused with `expected`; the same transaction must then still run a
/// statement and commit a job created after the refusal.
fn refused_in_a_usable_transaction<T: std::fmt::Debug>(
    conn: &mut PgConnection,
    expected: &str,
    op: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
) {
    let marker = format!("usable-{}", Uuid::new_v4());
    // A kind no test here claims, so the marker never takes a claim slot.
    let input = new_job(TEST_CAPABILITY, &marker, &[&key(NS_CS)]);
    conn.build_transaction()
        .read_committed()
        .run::<_, DieselError, _>(|c| {
            assert_code(op(c), expected);
            c.batch_execute("SELECT 1")?;
            let created = create_job_on(c, &REGISTRY, input).expect("a create after the refusal");
            assert!(matches!(created, CreateOutcome::Created(_)));
            Ok(())
        })
        .expect("the caller's transaction must still commit");
    assert_eq!(
        count(
            conn,
            &format!("FROM public.async_job WHERE payload->>'id' = '{marker}'")
        ),
        1
    );
}

/// Two minutes from now: inside every test kind's wait bounds.
fn in_two_minutes() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + Duration::seconds(120)
}

fn wait_req(checkpoint: &str) -> WaitRequest {
    WaitRequest {
        checkpoint: checkpoint.to_string(),
        resume_at: in_two_minutes(),
        provider_ref: None,
    }
}

fn payload_with(id: &str, target: &str, extra: &str) -> String {
    format!(r#"{{"id": "{id}", "targets": ["{target}"], {extra}}}"#)
}

fn commands_for(conn: &mut PgConnection, id: Uuid, outcome: &str, code: &str) -> i64 {
    count(
        conn,
        &format!(
            "FROM public.async_control_command WHERE async_job_id = '{id}' \
               AND outcome = '{outcome}' AND outcome_code = '{code}'"
        ),
    )
}

/// A staff resolution command with `actor`.
fn staff_resolve(
    conn: &mut PgConnection,
    actor: &StaffActor,
    id: Uuid,
    resolution: StaffResolution,
) -> ThothResult<CommandResult> {
    tx(conn, |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, actor, id, resolution)
    })
}

#[test]
fn bounds_u0000_in_every_input_is_refused_with_its_code_and_leaves_the_callers_transaction_usable()
{
    let (_guard, mut conn) = setup();
    let c = &mut conn;

    // A payload, in a string and in a key: ASYNC_JOB_PAYLOAD_INVALID.
    let x = key(NS_CS);
    for (id, extra) in [
        ("nul-value", r#""note": "a\u0000b""#),
        ("nul-key", r#""a\u0000b": 1"#),
    ] {
        let input = NewAsyncJob {
            payload: payload_with(id, &x, extra),
            ..new_job(TEST_EFFECT, id, &[&x])
        };
        refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
            create_job_on(c, &REGISTRY, input)
        });
        assert_eq!(
            count(
                c,
                &format!("FROM public.async_job WHERE payload->>'id' = '{id}'")
            ),
            0
        );
    }

    // A worker's checkpoint: ASYNC_JOB_PAYLOAD_INVALID, the claim live.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "nul-wait", &[&key(NS_CS)]));
    let before = footprint(c, a.async_job_id);
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        wait_on(c, &REGISTRY, &claim, wait_req(r#"{"step": "a\u0000b"}"#))
    });
    assert_nothing_written_and_live(c, &before, &claim);

    // A worker's evidence object, in `complete` after effect start and in
    // `fail` with the flag: a shape fault, the claim live (never T6x).
    let y = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "nul-evidence", &[&y]));
    let before = footprint(c, b.async_job_id);
    let call = CompleteRequest {
        evidence: Some(r#"{"valid": true, "note": "a\u0000b"}"#.to_string()),
        ..completion(CompletionOutcome::EffectApplied, vec![cs_entry(&y, "fp")])
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        complete_on(c, &REGISTRY, &claim, call)
    });
    assert_nothing_written_and_live(c, &before, &claim);
    let before = footprint(c, b.async_job_id);
    let call = no_effect_req(
        FailureClass::Permanent,
        Some(r#"{"valid": true, "providerRef": "p\u0000"}"#),
    );
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        fail_on(c, &REGISTRY, &claim, call)
    });
    assert_nothing_written_and_live(c, &before, &claim);
    // An entry's string is a shape fault too.
    let before = footprint(c, b.async_job_id);
    let call = completion(CompletionOutcome::EffectApplied, vec![cs_entry(&y, "f\0p")]);
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        complete_on(c, &REGISTRY, &claim, call)
    });
    assert_nothing_written_and_live(c, &before, &claim);

    // A command's evidence object: ASYNC_JOB_PAYLOAD_INVALID, no command row.
    let z = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_EFFECT, "nul-command", &[&z]));
    let before = footprint(c, rr.async_job_id);
    let resolution = StaffResolution {
        entries: Some(vec![cs_entry(&z, "fp")]),
        evidence: Some(r#"{"valid": true, "note": "a\u0000b"}"#.to_string()),
        ..staff_res(StaffOutcome::Succeeded)
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        resolve_reconciliation_command_on(
            c,
            &REGISTRY,
            &staff("resolve"),
            rr.async_job_id,
            resolution,
        )
    });
    assert_eq!(footprint(c, rr.async_job_id), before);

    // A `reason`: ASYNC_JOB_PAYLOAD_INVALID before any statement.
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        cancel_job_command_on(c, &REGISTRY, &staff("stop\0now"), rr.async_job_id)
    });
    assert_eq!(footprint(c, rr.async_job_id), before);

    // `claimedBy`: ASYNC_JOB_PAYLOAD_INVALID, nothing claimed.
    let pending = create(c, new_job(TEST_PEER, "nul-claimer", &[&key(NS_CS)]));
    let executor = Executor {
        name: "worker\0one".to_string(),
        ..dissemination()
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        claim_on(c, &REGISTRY, &executor, request(&[TEST_PEER], 5))
    });
    assert_eq!(status(c, pending.async_job_id), AsyncJobStatus::Pending);
    assert_no_unspecified(c);
}

#[test]
fn bounds_a_value_jsonb_refuses_is_refused_with_its_code_and_leaves_the_callers_transaction_usable()
{
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // serde accepts `1e-20000`; `jsonb` refuses it (it overflows numeric).
    assert!(serde_json::from_str::<serde_json::Value>(r#"{"n": 1e-20000}"#).is_ok());

    // A payload: ASYNC_JOB_PAYLOAD_INVALID.
    let x = key(NS_CS);
    let input = NewAsyncJob {
        payload: payload_with("tiny", &x, r#""n": 1e-20000"#),
        ..new_job(TEST_EFFECT, "tiny", &[&x])
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        create_job_on(c, &REGISTRY, input)
    });
    assert_eq!(
        count(c, "FROM public.async_job WHERE payload->>'id' = 'tiny'"),
        0
    );

    // A worker's checkpoint: ASYNC_JOB_PAYLOAD_INVALID, the claim live.
    let (a, claim) = running(c, new_job(TEST_EFFECT, "tiny-wait", &[&key(NS_CS)]));
    let before = footprint(c, a.async_job_id);
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        wait_on(c, &REGISTRY, &claim, wait_req(r#"{"n": 1e-20000}"#))
    });
    assert_nothing_written_and_live(c, &before, &claim);

    // A worker's evidence object: a shape fault, the claim live (never T6x).
    let y = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "tiny-evidence", &[&y]));
    let before = footprint(c, b.async_job_id);
    let request = CompleteRequest {
        evidence: Some(r#"{"valid": true, "n": 1e-20000}"#.to_string()),
        ..completion(CompletionOutcome::EffectApplied, vec![cs_entry(&y, "fp")])
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        complete_on(c, &REGISTRY, &claim, request)
    });
    assert_nothing_written_and_live(c, &before, &claim);
    let before = footprint(c, b.async_job_id);
    let request = no_effect_req(
        FailureClass::Permanent,
        Some(r#"{"valid": true, "providerRef": "p", "n": 1e-20000}"#),
    );
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        fail_on(c, &REGISTRY, &claim, request)
    });
    assert_nothing_written_and_live(c, &before, &claim);

    // An executor reconciler's evidence and checkpoint (T16).
    let z = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_PEER, "tiny-t16", &[&z]));
    let recon = claim_recon(c, &rr);
    let before = footprint(c, rr.async_job_id);
    let request = ResolveReconciliation {
        entries: Some(vec![cs_entry(&z, "fp")]),
        ..resolution_evidence(
            ReconcilerOutcome::Succeeded,
            r#"{"valid": true, "n": 1e-20000}"#,
        )
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_EVIDENCE_INVALID", |c| {
        resolve_reconciliation_on(c, &REGISTRY, &recon, request)
    });
    assert_nothing_written_and_live(c, &before, &recon);
    let before = footprint(c, rr.async_job_id);
    let request = ResolveReconciliation {
        checkpoint: Some(r#"{"n": 1e-20000}"#.to_string()),
        resume_at: Some(in_two_minutes()),
        ..resolution(ReconcilerOutcome::Waiting)
    };
    refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
        resolve_reconciliation_on(c, &REGISTRY, &recon, request)
    });
    assert_nothing_written_and_live(c, &before, &recon);

    // A command's evidence or checkpoint: ASYNC_JOB_PAYLOAD_INVALID before
    // any write, so no command row.
    let w = key(NS_CS);
    let staffed = rr_started(c, new_job(TEST_EFFECT, "tiny-t18", &[&w]));
    let before = footprint(c, staffed.async_job_id);
    for resolution in [
        StaffResolution {
            entries: Some(vec![cs_entry(&w, "fp")]),
            evidence: Some(r#"{"valid": true, "n": 1e-20000}"#.to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
        StaffResolution {
            checkpoint: Some(r#"{"n": 1e-20000}"#.to_string()),
            resume_at: Some(in_two_minutes()),
            ..staff_res(StaffOutcome::Waiting)
        },
    ] {
        let id = staffed.async_job_id;
        refused_in_a_usable_transaction(c, "ASYNC_JOB_PAYLOAD_INVALID", |c| {
            resolve_reconciliation_command_on(c, &REGISTRY, &staff("resolve"), id, resolution)
        });
        assert_eq!(footprint(c, staffed.async_job_id), before);
    }
    assert_no_unspecified(c);
}

#[test]
fn bounds_an_oversized_checkpoint_and_a_malformed_error_code_write_nothing_and_leave_the_claim_live(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // `{"blob": "<n characters>"}` renders as n + 12 bytes of `jsonb` text.
    let checkpoint = |n: usize| format!(r#"{{"blob":"{}"}}"#, "x".repeat(n));
    assert_eq!(
        scalar(
            c,
            &format!(
                "SELECT octet_length('{}'::jsonb::text)::bigint AS value",
                checkpoint(65_524)
            )
        ),
        65_536
    );
    let (a, claim) = running(c, new_job(TEST_EFFECT, "big-wait", &[&key(NS_CS)]));
    let before = footprint(c, a.async_job_id);
    assert_code(
        tx(c, |c| {
            wait_on(c, &REGISTRY, &claim, wait_req(&checkpoint(65_525)))
        }),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    // The boundary itself is taken, with the same claim.
    let waited = tx(c, |c| {
        wait_on(c, &REGISTRY, &claim, wait_req(&checkpoint(65_524)))
    })
    .unwrap();
    assert!(matches!(waited, WaitOutcome::Waiting(_)));

    let (b, claim) = running(c, new_job(TEST_EFFECT, "bad-code", &[&key(NS_CS)]));
    for error_code in [
        "",
        "lower_case",
        "BAD CODE",
        "1LEADING_DIGIT",
        "TRAILING-DASH",
        &"A".repeat(65),
    ] {
        let before = footprint(c, b.async_job_id);
        let request = FailRequest {
            error_code: error_code.to_string(),
            ..fail_req(FailureClass::Permanent)
        };
        assert_code(do_fail(c, &claim, request), "ASYNC_JOB_PAYLOAD_INVALID");
        assert_nothing_written_and_live(c, &before, &claim);
    }
    // Sixty-four characters are taken.
    let request = FailRequest {
        error_code: "A".repeat(64),
        ..fail_req(FailureClass::Permanent)
    };
    let failed = do_fail(c, &claim, request).unwrap();
    assert_eq!(failed.status, AsyncJobStatus::Failed);
    assert_eq!(
        last_attempt(c, b.async_job_id).error_code,
        Some("A".repeat(64))
    );
}

#[test]
fn bounds_every_cell_of_the_evidence_fault_table() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let other = key(NS_CS);

    // `complete` at PRE_WRITE: both faults are ASYNC_JOB_EVIDENCE_INVALID,
    // nothing written, the claim live.
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "cell-pre", &[&x]));
    let shape = EvidenceEntry {
        source_ref: Some("ref".to_string()),
        ..cs_entry(&x, "fp")
    };
    for entry in [shape, cs_entry(&other, "fp")] {
        let before = footprint(c, a.async_job_id);
        assert_code(
            do_complete(c, &claim, completion(CompletionOutcome::NoOp, vec![entry])),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_nothing_written_and_live(c, &before, &claim);
    }

    // `complete` after effect start: a shape fault keeps the claim live; a
    // semantic fault is T6x.
    let y = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "cell-post", &[&y]));
    let before = footprint(c, b.async_job_id);
    assert_code(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::EffectApplied,
                vec![cs_entry(&y, "fp"), cs_entry(&y, "fp")],
            ),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![cs_entry(&other, "fp")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    let after = job(c, b.async_job_id);
    assert_eq!(after.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(after.last_outcome_code.as_deref(), Some("EVIDENCE_INVALID"));
    let closed = last_attempt(c, b.async_job_id);
    assert_eq!(closed.result, Some(AsyncAttemptResult::Indeterminate));
    assert_eq!(closed.outcome_code.as_deref(), Some("EVIDENCE_INVALID"));
    assert_eq!(holds(c, b.async_job_id), vec![y.clone()]);
    assert_eq!(state(c, &y), None);
    assert_eq!(state(c, &other), None);

    // `fail(PERMANENT, no_effect_proven)`: both faults are
    // ASYNC_JOB_EVIDENCE_INVALID, nothing written, the claim live.
    let (d, claim) = started(c, new_job(TEST_EFFECT, "cell-fail", &[&key(NS_CS)]));
    for evidence in [r#"[1, 2]"#, r#"{"valid": false, "providerRef": "p"}"#] {
        let before = footprint(c, d.async_job_id);
        assert_code(
            do_fail(
                c,
                &claim,
                no_effect_req(FailureClass::Permanent, Some(evidence)),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_nothing_written_and_live(c, &before, &claim);
    }

    // T16: both faults are ASYNC_JOB_EVIDENCE_INVALID, the job stays RR and
    // the claim live.
    let z = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_PEER, "cell-t16", &[&z]));
    let recon = claim_recon(c, &rr);
    for entry in [cs_entry(&z, "has space"), cs_entry(&other, "fp")] {
        let before = footprint(c, rr.async_job_id);
        assert_code(
            do_resolve(c, &recon, resolution_entries(vec![entry])),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_eq!(
            status(c, rr.async_job_id),
            AsyncJobStatus::ReconciliationRequired
        );
        assert_nothing_written_and_live(c, &before, &recon);
    }

    // T18 (the T6x job above, unclaimed): both faults are a committed
    // REFUSED with EVIDENCE_INVALID, and nothing else changes.
    for resolution in [
        StaffResolution {
            entries: Some(vec![cs_entry(&y, "fp")]),
            evidence: Some(r#"{"valid": true, "token": "t"}"#.to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
        StaffResolution {
            entries: Some(vec![cs_entry(&other, "fp")]),
            ..staff_res(StaffOutcome::Succeeded)
        },
    ] {
        let before = footprint(c, b.async_job_id);
        let result = do_staff(c, b.async_job_id, resolution);
        assert_command_refused(&result, "EVIDENCE_INVALID");
        assert_eq!(
            footprint(c, b.async_job_id),
            Footprint {
                commands: before.commands + 1,
                ..before
            }
        );
    }
    assert_eq!(
        commands_for(c, b.async_job_id, "REFUSED", "EVIDENCE_INVALID"),
        2
    );
    assert_no_unspecified(c);
}

#[test]
fn bounds_fail_with_no_effect_proven_is_refused_with_its_reason_and_the_claim_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let check = |c: &mut PgConnection, claim: &WorkerClaim, id: Uuid, class, reason: &str| {
        let before = footprint(c, id);
        assert_refused(
            do_fail(c, claim, no_effect_req(class, Some(NO_EFFECT_EVIDENCE))),
            reason,
        );
        assert_nothing_written_and_live(c, &before, claim);
    };
    // At EFFECT_CONFIRMED: refused, and the worker can then fail without
    // the flag (T9).
    let (a, claim) = confirmed(c, new_job(TEST_EFFECT, "ne-confirmed", &[&key(NS_CS)]));
    check(
        c,
        &claim,
        a.async_job_id,
        FailureClass::Permanent,
        "EFFECT_CONFIRMED",
    );
    let failed = do_fail(c, &claim, fail_req(FailureClass::Permanent)).unwrap();
    assert_eq!(failed.status, AsyncJobStatus::ReconciliationRequired);
    // At PRE_WRITE.
    let (b, claim) = running(c, new_job(TEST_EFFECT, "ne-pre", &[&key(NS_CS)]));
    check(c, &claim, b.async_job_id, FailureClass::Permanent, "PHASE");
    // With a class other than PERMANENT.
    let (d, claim) = started(c, new_job(TEST_EFFECT, "ne-class", &[&key(NS_CS)]));
    for class in [FailureClass::RetryablePreWrite, FailureClass::Indeterminate] {
        check(c, &claim, d.async_job_id, class, "CLASS");
    }
    // For a kind without `no_effect_after_start_allowed`.
    let (e, claim) = started(c, new_job(TEST_STRICT, "ne-kind", &[&key(NS_CS)]));
    check(
        c,
        &claim,
        e.async_job_id,
        FailureClass::Permanent,
        "KIND_POLICY",
    );
}

#[test]
fn bounds_complete_with_an_outcome_its_phase_or_kind_does_not_allow_is_refused_with_the_claim_live()
{
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let check = |c: &mut PgConnection, claim: &WorkerClaim, id: Uuid, request, reason: &str| {
        let before = footprint(c, id);
        assert_refused(do_complete(c, claim, request), reason);
        assert_nothing_written_and_live(c, &before, claim);
    };
    let r = key(NS_RB);
    seed_rb_state(c, &r, 9, "h9");
    // PHASE: NO_OP and SUPERSEDED after effect start, EFFECT_APPLIED at
    // PRE_WRITE.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "phase-noop", &[&x]));
    check(
        c,
        &claim,
        a.async_job_id,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp")]),
        "PHASE",
    );
    let (b, claim) = started(c, revision_job("phase-sup", &r, 3, "h3"));
    check(
        c,
        &claim,
        b.async_job_id,
        completion(
            CompletionOutcome::Superseded,
            vec![EvidenceEntry::key_only(&r)],
        ),
        "PHASE",
    );
    let y = key(NS_CS);
    let (d, claim) = running(c, new_job(TEST_EFFECT, "phase-applied", &[&y]));
    check(
        c,
        &claim,
        d.async_job_id,
        completion(CompletionOutcome::EffectApplied, vec![cs_entry(&y, "fp")]),
        "PHASE",
    );
    // KIND_POLICY: COMPLETED on an EFFECTFUL kind, SUPERSEDED on a
    // CURRENT_STATE kind.
    for outcome in [CompletionOutcome::Completed, CompletionOutcome::Superseded] {
        check(
            c,
            &claim,
            d.async_job_id,
            completion(outcome, vec![EvidenceEntry::key_only(&y)]),
            "KIND_POLICY",
        );
    }
}

#[test]
fn bounds_a_call_with_several_refusal_reasons_reports_the_first_in_the_stated_order() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // TEST_STRICT has no `no_effect_after_start_allowed`, so KIND_POLICY
    // applies to every flagged `fail` below, and INDETERMINATE adds CLASS.
    let flagged = |c: &mut PgConnection, claim: &WorkerClaim, reason: &str| {
        assert_refused(
            do_fail(
                c,
                claim,
                no_effect_req(FailureClass::Indeterminate, Some(NO_EFFECT_EVIDENCE)),
            ),
            reason,
        );
        assert_live(c, claim);
    };
    // PHASE, KIND_POLICY and CLASS: PHASE.
    let (_a, claim) = running(c, new_job(TEST_STRICT, "multi-pre", &[&key(NS_CS)]));
    flagged(c, &claim, "PHASE");
    // EFFECT_CONFIRMED, KIND_POLICY and CLASS: EFFECT_CONFIRMED.
    let (_b, claim) = confirmed(c, new_job(TEST_STRICT, "multi-confirmed", &[&key(NS_CS)]));
    flagged(c, &claim, "EFFECT_CONFIRMED");
    // KIND_POLICY and CLASS: KIND_POLICY.
    let (_d, claim) = started(c, new_job(TEST_STRICT, "multi-started", &[&key(NS_CS)]));
    flagged(c, &claim, "KIND_POLICY");
    // `complete`: SUPERSEDED after effect start on a CURRENT_STATE kind is
    // PHASE before KIND_POLICY.
    let x = key(NS_CS);
    let (_e, claim) = started(c, new_job(TEST_EFFECT, "multi-complete", &[&x]));
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::Superseded,
                vec![EvidenceEntry::key_only(&x)],
            ),
        ),
        "PHASE",
    );
    assert_live(c, &claim);
    // A refusal comes before a shape fault in the same call.
    assert_refused(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::Completed,
                vec![cs_entry(&x, "has space")],
            ),
        ),
        "KIND_POLICY",
    );
    assert_live(c, &claim);
}

#[test]
fn bounds_an_entry_missing_or_carrying_a_field_is_a_shape_fault_with_the_claim_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let check = |c: &mut PgConnection, claim: &WorkerClaim, id: Uuid, request| {
        let before = footprint(c, id);
        assert_code(do_complete(c, claim, request), "ASYNC_JOB_EVIDENCE_INVALID");
        assert_nothing_written_and_live(c, &before, claim);
    };
    let r = key(NS_RB);
    seed_rb_state(c, &r, 2, "h2");
    // After effect start (a semantic fault would be T6x).
    let (a, claim) = started(c, revision_job("fields-rb", &r, 3, "h3"));
    let rb = rb_entry(&r, 3, "h3");
    for entry in [
        EvidenceEntry {
            order: None,
            ..rb.clone()
        },
        EvidenceEntry {
            fingerprint: None,
            ..rb.clone()
        },
    ] {
        check(
            c,
            &claim,
            a.async_job_id,
            completion(CompletionOutcome::EffectApplied, vec![entry]),
        );
    }
    let x = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "fields-cs", &[&x]));
    for entry in [
        EvidenceEntry {
            fingerprint: None,
            ..cs_entry(&x, "fp")
        },
        EvidenceEntry {
            order: Some(1),
            ..cs_entry(&x, "fp")
        },
    ] {
        check(
            c,
            &claim,
            b.async_job_id,
            completion(CompletionOutcome::EffectApplied, vec![entry]),
        );
    }
    // At PRE_WRITE: a NO_OP entry with a sourceRef, in both modes.
    let y = key(NS_CS);
    let (d, claim) = running(c, new_job(TEST_EFFECT, "fields-noop", &[&y]));
    check(
        c,
        &claim,
        d.async_job_id,
        completion(
            CompletionOutcome::NoOp,
            vec![EvidenceEntry {
                source_ref: Some("ref".to_string()),
                ..cs_entry(&y, "fp")
            }],
        ),
    );
    let q = key(NS_RB);
    let (e, claim) = running(c, revision_job("fields-rb-noop", &q, 4, "h4"));
    for (outcome, entry) in [
        (
            CompletionOutcome::NoOp,
            EvidenceEntry {
                source_ref: Some("ref".to_string()),
                ..rb_entry(&q, 4, "h4")
            },
        ),
        // A key-only entry with a fingerprint.
        (
            CompletionOutcome::Superseded,
            EvidenceEntry {
                fingerprint: Some("h4".to_string()),
                ..EvidenceEntry::key_only(&q)
            },
        ),
    ] {
        check(c, &claim, e.async_job_id, completion(outcome, vec![entry]));
    }
    // COMPLETED with a fingerprint on a READ_ONLY kind.
    let z = key(NS_READ);
    let (g, claim) = running(c, new_job(TEST_READ, "fields-read", &[&z]));
    check(
        c,
        &claim,
        g.async_job_id,
        completion(CompletionOutcome::Completed, vec![cs_entry(&z, "fp")]),
    );
}

#[test]
fn bounds_an_input_in_a_none_cell_of_the_evidence_argument_table_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let r = key(NS_RB);
    seed_rb_state(c, &r, 9, "h9");
    let q = key(NS_RB);
    seed_rb_state(c, &q, 7, "h7");

    // `complete`: an evidence object with SUPERSEDED or COMPLETED.
    let (a, claim) = running(c, revision_job("none-sup", &r, 3, "h3"));
    let before = footprint(c, a.async_job_id);
    let request = CompleteRequest {
        evidence: Some(applied_evidence(3)),
        ..completion(
            CompletionOutcome::Superseded,
            vec![EvidenceEntry::key_only(&r)],
        )
    };
    assert_code(
        do_complete(c, &claim, request),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    let z = key(NS_READ);
    let (b, claim) = running(c, new_job(TEST_READ, "none-read", &[&z]));
    let before = footprint(c, b.async_job_id);
    let request = CompleteRequest {
        evidence: Some(r#"{"valid": true}"#.to_string()),
        ..completion(
            CompletionOutcome::Completed,
            vec![EvidenceEntry::key_only(&z)],
        )
    };
    assert_code(
        do_complete(c, &claim, request),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    // `fail` without the flag takes no evidence object.
    let (d, claim) = started(c, new_job(TEST_EFFECT, "none-fail", &[&key(NS_CS)]));
    let before = footprint(c, d.async_job_id);
    let request = FailRequest {
        evidence: Some(NO_EFFECT_EVIDENCE.to_string()),
        ..fail_req(FailureClass::Permanent)
    };
    assert_code(do_fail(c, &claim, request), "ASYNC_JOB_EVIDENCE_INVALID");
    assert_nothing_written_and_live(c, &before, &claim);

    // T16 FAILED or CANCELLED at EFFECT_STARTED: entries are refused, the
    // job stays RR and the claim live; so are entries or evidence with
    // STILL_REQUIRED, which the table does not list.
    let x = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_PEER, "none-t16", &[&x]));
    let recon = claim_recon(c, &rr);
    for outcome in [ReconcilerOutcome::Failed, ReconcilerOutcome::Cancelled] {
        let before = footprint(c, rr.async_job_id);
        let request = ResolveReconciliation {
            entries: Some(vec![cs_entry(&x, "fp")]),
            ..resolution_evidence(outcome, NO_EFFECT_EVIDENCE)
        };
        assert_code(do_resolve(c, &recon, request), "ASYNC_JOB_EVIDENCE_INVALID");
        assert_nothing_written_and_live(c, &before, &recon);
    }
    for request in [
        ResolveReconciliation {
            entries: Some(vec![cs_entry(&x, "fp")]),
            ..resolution(ReconcilerOutcome::StillRequired)
        },
        resolution_evidence(ReconcilerOutcome::StillRequired, NO_EFFECT_EVIDENCE),
    ] {
        let before = footprint(c, rr.async_job_id);
        assert_code(do_resolve(c, &recon, request), "ASYNC_JOB_EVIDENCE_INVALID");
        assert_nothing_written_and_live(c, &before, &recon);
    }

    // T18: entries with FAILED at EFFECT_STARTED, with ACCEPT_REGRESSION
    // and with FAILED at EFFECT_CONFIRMED are committed REFUSED with
    // EVIDENCE_INVALID.
    let y = key(NS_CS);
    let started_rr = rr_started(c, new_job(TEST_EFFECT, "none-t18", &[&y]));
    let confirmed_rr = rr_confirmed(c, new_job(TEST_EFFECT, "none-t18c", &[&key(NS_CS)]));
    let (regressed, claim) = started(c, revision_job("none-accept", &q, 5, "h5"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&q, 5, "h5")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    for (id, resolution) in [
        (
            started_rr.async_job_id,
            StaffResolution {
                entries: Some(vec![cs_entry(&y, "fp")]),
                evidence: Some(NO_EFFECT_EVIDENCE.to_string()),
                ..staff_res(StaffOutcome::Failed)
            },
        ),
        (
            regressed.async_job_id,
            StaffResolution {
                entries: Some(vec![rb_entry(&q, 5, "h5")]),
                evidence: Some(applied_evidence(5)),
                ..staff_res(StaffOutcome::AcceptRegression)
            },
        ),
        (
            confirmed_rr.async_job_id,
            StaffResolution {
                entries: Some(vec![EvidenceEntry::key_only(&y)]),
                evidence: Some(NOT_APPLIED_EVIDENCE.to_string()),
                ..staff_res(StaffOutcome::Failed)
            },
        ),
    ] {
        let before = footprint(c, id);
        let result = do_staff(c, id, resolution);
        assert_command_refused(&result, "EVIDENCE_INVALID");
        assert_eq!(
            footprint(c, id),
            Footprint {
                commands: before.commands + 1,
                ..before
            }
        );
    }
    assert_no_unspecified(c);
}

#[test]
fn bounds_complete_after_effect_start_with_a_shape_and_a_semantic_fault_keeps_the_claim_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let other = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "both-cs", &[&x]));
    let before = footprint(c, a.async_job_id);
    // A duplicate key (shape) on a key the job does not have (semantic).
    assert_code(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::EffectApplied,
                vec![cs_entry(&other, "fp"), cs_entry(&other, "fp")],
            ),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    // A regression (semantic) carrying a fingerprint outside the identifier
    // format (shape).
    let r = key(NS_RB);
    seed_rb_state(c, &r, 9, "h9");
    let (b, claim) = started(c, revision_job("both-rb", &r, 3, "h3"));
    let before = footprint(c, b.async_job_id);
    assert_code(
        do_complete(
            c,
            &claim,
            completion(
                CompletionOutcome::EffectApplied,
                vec![rb_entry(&r, 3, "has space")],
            ),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    assert_eq!(status(c, b.async_job_id), AsyncJobStatus::Running);
}

#[test]
fn bounds_an_omitted_required_evidence_object_after_effect_start_keeps_the_claim_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    // TEST_STRICT's validator refuses JSON null for applied evidence.
    let (a, claim) = started(c, new_job(TEST_STRICT, "omitted", &[&x]));
    let before = footprint(c, a.async_job_id);
    assert_code(
        do_complete(
            c,
            &claim,
            completion(CompletionOutcome::EffectApplied, vec![cs_entry(&x, "fp")]),
        ),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    // Corrected, it completes.
    let request = CompleteRequest {
        evidence: Some(r#"{"valid": true}"#.to_string()),
        ..completion(CompletionOutcome::EffectApplied, vec![cs_entry(&x, "fp")])
    };
    assert!(matches!(
        do_complete(c, &claim, request).unwrap(),
        CompleteOutcome::Succeeded(_)
    ));
    // The same for an executor reconciler's SUCCEEDED (T16).
    let y = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_STRICT, "omitted-t16", &[&y]));
    let recon = claim_recon(c, &rr);
    let before = footprint(c, rr.async_job_id);
    assert_code(
        do_resolve(c, &recon, resolution_entries(vec![cs_entry(&y, "fp")])),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &recon);
}

#[test]
fn bounds_an_unparseable_or_jsonb_refused_value_in_a_staff_command_is_payload_invalid_with_no_command_row(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let rr = rr_started(c, new_job(TEST_EFFECT, "staff-json", &[&x]));
    let id = rr.async_job_id;
    let before = footprint(c, id);
    let entries = || Some(vec![cs_entry(&x, "fp")]);
    for resolution in [
        StaffResolution {
            entries: entries(),
            evidence: Some("not json".to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
        StaffResolution {
            entries: entries(),
            evidence: Some(r#"{"valid": true, "n": 1e-20000}"#.to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
        StaffResolution {
            evidence: Some(r#"{"valid": true, "providerRef": "a\u0000"}"#.to_string()),
            ..staff_res(StaffOutcome::Failed)
        },
        StaffResolution {
            checkpoint: Some("{".to_string()),
            resume_at: Some(in_two_minutes()),
            ..staff_res(StaffOutcome::Waiting)
        },
        StaffResolution {
            checkpoint: Some(r#"{"n": 1e-20000}"#.to_string()),
            resume_at: Some(in_two_minutes()),
            ..staff_res(StaffOutcome::Waiting)
        },
        StaffResolution {
            entries: Some(vec![cs_entry(&x, "f\0p")]),
            ..staff_res(StaffOutcome::Succeeded)
        },
    ] {
        assert_code(
            staff_resolve(c, &staff("resolve"), id, resolution),
            "ASYNC_JOB_PAYLOAD_INVALID",
        );
        assert_eq!(footprint(c, id), before);
    }
    // A value that parses and that `jsonb` takes has a canonical form, so its
    // shape fault is a committed REFUSED instead.
    let result = do_staff(
        c,
        id,
        StaffResolution {
            entries: entries(),
            evidence: Some("[1, 2]".to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
    );
    assert_command_refused(&result, "EVIDENCE_INVALID");
    assert_eq!(command_rows(c), before.commands + 1);
}

#[test]
fn bounds_an_out_of_range_order_is_a_shape_fault() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let beyond = [i128::from(i64::MAX) + 1, i128::from(i64::MIN) - 1];
    let entry = |target: &str, order: i128| EvidenceEntry {
        order: Some(order),
        ..rb_entry(target, 0, "h3")
    };
    // `complete` after effect start: the claim live, never T6x.
    let r = key(NS_RB);
    seed_rb_state(c, &r, 9, "h9");
    let (a, claim) = started(c, revision_job("order-worker", &r, 3, "h3"));
    for order in beyond {
        let before = footprint(c, a.async_job_id);
        assert_code(
            do_complete(
                c,
                &claim,
                completion(CompletionOutcome::EffectApplied, vec![entry(&r, order)]),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_nothing_written_and_live(c, &before, &claim);
    }
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    // T16: the job stays RR and the claim live.
    let recon = claim_recon(c, &a);
    for order in beyond {
        let before = footprint(c, a.async_job_id);
        assert_code(
            do_resolve(c, &recon, resolution_entries(vec![entry(&r, order)])),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_nothing_written_and_live(c, &before, &recon);
    }
    do_resolve(c, &recon, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    // T18 (the job is unclaimed again): a committed REFUSED with
    // EVIDENCE_INVALID.
    for order in beyond {
        let result = do_staff(
            c,
            a.async_job_id,
            StaffResolution {
                entries: Some(vec![entry(&r, order)]),
                evidence: Some(applied_evidence(3)),
                ..staff_res(StaffOutcome::Succeeded)
            },
        );
        assert_command_refused(&result, "EVIDENCE_INVALID");
    }
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
}

#[test]
fn bounds_a_credential_named_key_in_a_commands_checkpoint_is_a_committed_wait_invalid() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let rr = rr_started(c, new_job(TEST_EFFECT, "staff-wait", &[&key(NS_CS)]));
    let id = rr.async_job_id;
    for checkpoint in [r#"{"apiKey": "k"}"#, r#"{"step": {"Password": "p"}}"#] {
        let before = footprint(c, id);
        let result = do_staff(
            c,
            id,
            StaffResolution {
                checkpoint: Some(checkpoint.to_string()),
                resume_at: Some(in_two_minutes()),
                ..staff_res(StaffOutcome::Waiting)
            },
        );
        assert_command_refused(&result, "WAIT_INVALID");
        assert_eq!(
            footprint(c, id),
            Footprint {
                commands: before.commands + 1,
                ..before
            }
        );
    }
    assert_eq!(commands_for(c, id, "REFUSED", "WAIT_INVALID"), 2);
    // Without the key, the same command applies.
    let result = do_staff(
        c,
        id,
        StaffResolution {
            checkpoint: Some(r#"{"step": 1}"#.to_string()),
            resume_at: Some(in_two_minutes()),
            ..staff_res(StaffOutcome::Waiting)
        },
    );
    assert_applied(&result, "WAITING");
}

#[test]
fn bounds_a_blank_reason_is_payload_invalid_before_any_write() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let rr = rr_started(c, new_job(TEST_EFFECT, "blank-reason", &[&key(NS_CS)]));
    let id = rr.async_job_id;
    let before = footprint(c, id);
    for reason in ["", " ", "\t\n", "\u{3000}"] {
        let actor = staff(reason);
        let results = [
            tx(c, |c| cancel_job_command_on(c, &REGISTRY, &actor, id)),
            tx(c, |c| retry_job_command_on(c, &REGISTRY, &actor, id)),
            tx(c, |c| {
                resolve_reconciliation_command_on(
                    c,
                    &REGISTRY,
                    &actor,
                    id,
                    staff_res(StaffOutcome::Waiting),
                )
            }),
            tx(c, |c| redact_job_diagnostics_command_on(c, &actor, id)),
        ];
        for result in results {
            assert_code(result, "ASYNC_JOB_PAYLOAD_INVALID");
        }
        assert_eq!(footprint(c, id), before);
    }
}

#[test]
fn bounds_no_engine_path_surfaces_a_column_check_as_an_integrity_violation() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let long = |n: usize| "a".repeat(n);
    // Claims: the executor's name and principal.
    for executor in [
        Executor {
            name: long(129),
            ..dissemination()
        },
        Executor {
            name: " ".to_string(),
            ..dissemination()
        },
        Executor {
            principal: long(257),
            ..dissemination()
        },
    ] {
        assert_code(
            tx(c, |c| {
                claim_on(c, &REGISTRY, &executor, request(&[TEST_EFFECT], 1))
            }),
            "ASYNC_JOB_PAYLOAD_INVALID",
        );
    }
    // Free text is truncated, never refused: a provider reference and an
    // error detail far over their columns' lengths are stored truncated.
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "long-text", &[&x]));
    let confirmed_job = tx(c, |c| {
        mark_effect_confirmed_on(c, &REGISTRY, &claim, Some(&long(10_000)))
    })
    .unwrap();
    let stored_ref = confirmed_job.provider_ref.expect("a provider reference");
    assert!(stored_ref.chars().count() < 10_000);
    let request = FailRequest {
        error_detail: Some(long(100_000)),
        ..fail_req(FailureClass::Indeterminate)
    };
    let failed = do_fail(c, &claim, request).unwrap();
    assert_eq!(failed.status, AsyncJobStatus::ReconciliationRequired);
    let detail = last_attempt(c, a.async_job_id)
        .error_detail
        .expect("an error detail");
    assert!(detail.chars().count() < 100_000);
    // Entries: an identifier one past its bound (257 characters) is
    // ASYNC_JOB_EVIDENCE_INVALID, at PRE_WRITE and after effect start.
    let y = key(NS_CS);
    let (b, claim) = started(c, new_job(TEST_EFFECT, "long-entry", &[&y]));
    for entry in [
        cs_entry(&y, &long(257)),
        EvidenceEntry {
            source_ref: Some(long(257)),
            ..cs_entry(&y, "fp")
        },
    ] {
        let before = footprint(c, b.async_job_id);
        assert_code(
            do_complete(
                c,
                &claim,
                completion(CompletionOutcome::EffectApplied, vec![entry]),
            ),
            "ASYNC_JOB_EVIDENCE_INVALID",
        );
        assert_nothing_written_and_live(c, &before, &claim);
    }
    // An evidence object one byte over 16384 as `jsonb`.
    // `{"pad": "<n>", "valid": true}` renders as n + 26 bytes.
    let evidence = format!(r#"{{"valid":true,"pad":"{}"}}"#, long(16_359));
    assert_eq!(
        scalar(
            c,
            &format!("SELECT octet_length('{evidence}'::jsonb::text)::bigint AS value")
        ),
        16_385
    );
    let before = footprint(c, b.async_job_id);
    let request = CompleteRequest {
        evidence: Some(evidence.clone()),
        ..completion(CompletionOutcome::EffectApplied, vec![cs_entry(&y, "fp")])
    };
    assert_code(
        do_complete(c, &claim, request),
        "ASYNC_JOB_EVIDENCE_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    // Commands: the actor's principal and reason one past their bounds, and
    // a missing request id, are ASYNC_JOB_PAYLOAD_INVALID with no row.
    let rr = rr_started(c, new_job(TEST_EFFECT, "long-command", &[&key(NS_CS)]));
    let id = rr.async_job_id;
    let rows = command_rows(c);
    for actor in [
        StaffActor {
            reason: long(1001),
            ..staff("x")
        },
        StaffActor {
            principal: long(257),
            ..staff("x")
        },
        StaffActor {
            request_id: None,
            ..staff("x")
        },
    ] {
        assert_code(
            tx(c, |c| cancel_job_command_on(c, &REGISTRY, &actor, id)),
            "ASYNC_JOB_PAYLOAD_INVALID",
        );
    }
    assert_eq!(command_rows(c), rows);
    // A command's evidence or checkpoint over its byte limit is a committed
    // REFUSED, never an integrity violation.
    let result = do_staff(
        c,
        id,
        StaffResolution {
            entries: Some(vec![cs_entry(&rr.target_keys[0], "fp")]),
            evidence: Some(evidence),
            ..staff_res(StaffOutcome::Succeeded)
        },
    );
    assert_command_refused(&result, "EVIDENCE_INVALID");
    let checkpoint = format!(r#"{{"blob":"{}"}}"#, long(65_525));
    let result = do_staff(
        c,
        id,
        StaffResolution {
            checkpoint: Some(checkpoint),
            resume_at: Some(in_two_minutes()),
            ..staff_res(StaffOutcome::Waiting)
        },
    );
    assert_command_refused(&result, "WAIT_INVALID");
    // The reason at its bound is taken.
    let actor = StaffActor {
        reason: long(1000),
        ..staff("x")
    };
    let result = tx(c, |c| cancel_job_command_on(c, &REGISTRY, &actor, id)).unwrap();
    assert_command_refused(&result, "NOT_CANCELLABLE");
}

// ---------------------------------------------------------------------------
// Database backstops (19.2)
// ---------------------------------------------------------------------------

/// The open attempt of a job.
fn open_attempt_of(conn: &mut PgConnection, id: Uuid) -> Uuid {
    attempts(conn, id)
        .into_iter()
        .find(|attempt| attempt.finished_at.is_none())
        .expect("an open attempt")
        .async_job_attempt_id
}

/// `regressed_at|regressed_from_order|regressed_from_fingerprint` of a key,
/// each empty when null.
fn regressed_text(conn: &mut PgConnection, key: &str) -> String {
    text(
        conn,
        &format!(
            "SELECT format('%s|%s|%s', regressed_at, regressed_from_order, \
                    regressed_from_fingerprint) AS value \
               FROM public.async_effect_target_state WHERE target_key = '{key}'"
        ),
    )
    .expect("a state row")
}

/// A direct-SQL write of a key's state as `job`'s attempt `attempt`, with
/// `extra` further assignments.
fn state_write_sql(
    key: &str,
    order: Option<i64>,
    fingerprint: &str,
    job: Uuid,
    attempt: Uuid,
    extra: &str,
) -> String {
    let order = order.map_or("NULL".to_string(), |o| o.to_string());
    format!(
        "UPDATE public.async_effect_target_state \
            SET applied_order = {order}, applied_fingerprint = '{fingerprint}', \
                last_job_id = '{job}', last_attempt_id = '{attempt}'{extra} \
          WHERE target_key = '{key}'"
    )
}

/// Close a job's open attempt and move the job to the terminal `status`,
/// releasing its holds, in direct SQL.
fn terminate_sql(id: Uuid, status: &str, result: &str, outcome: &str) -> String {
    format!(
        "UPDATE public.async_job_attempt SET finished_at = now(), result = '{result}', \
             job_status_after = '{status}', outcome_code = '{outcome}' \
          WHERE async_job_id = '{id}' AND finished_at IS NULL; \
         DELETE FROM public.async_target_hold WHERE async_job_id = '{id}'; \
         UPDATE public.async_job SET status = '{status}', terminal_at = now(), \
             claim_token = NULL, claimed_by = NULL, executor_principal = NULL, \
             claimed_at = NULL, lease_expires_at = NULL, attempt_runtime_deadline = NULL \
          WHERE async_job_id = '{id}'"
    )
}

/// A TEST_REVISION job for `order` of `key`, taken to RR by T9 and resolved
/// by a staff SUCCEEDED with its own revision (T18).
fn staff_succeeded(conn: &mut PgConnection, key: &str, order: i64) -> CommandResult {
    let id = format!("t18-{}", Uuid::new_v4());
    let fingerprint = format!("h{order}");
    let rr = rr_started(conn, revision_job(&id, key, order, &fingerprint));
    let result = do_staff(
        conn,
        rr.async_job_id,
        StaffResolution {
            entries: Some(vec![rb_entry(key, order, &fingerprint)]),
            evidence: Some(applied_evidence(order)),
            ..staff_res(StaffOutcome::Succeeded)
        },
    );
    assert_applied(&result, "EFFECT_APPLIED");
    result
}

#[test]
fn backstop_a_direct_sql_insert_of_a_non_pending_or_claimed_row_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let refused = Err("async_job_insert_guard".to_string());
    for status in ALL_STATUSES.iter().filter(|status| **status != "PENDING") {
        let value = format!("'{status}'");
        let sql = insert_sql("async_job", JOB_BASE, &[("status", value.as_str())]);
        assert_eq!(probe(c, &sql, true), refused, "{status}");
    }
    // A PENDING row that carries a claim.
    let mut claimed = RUNNING_COLUMNS.to_vec();
    claimed.retain(|(column, _)| *column != "status");
    let sql = insert_sql("async_job", JOB_BASE, &claimed);
    assert_eq!(probe(c, &sql, true), refused);
    assert_eq!(
        probe(c, &insert_sql("async_job", JOB_BASE, &[]), true),
        Ok(())
    );
}

#[test]
fn backstop_each_phase_rule_of_rule_4_refuses_its_direct_sql_status_change() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let refused = || Err("async_job_update_guard".to_string());
    for (from, phase, to, expected) in [
        // CANCELLED is refused at EFFECT_CONFIRMED.
        ("RUNNING", "EFFECT_CONFIRMED", "CANCELLED", refused()),
        ("WAITING", "EFFECT_CONFIRMED", "CANCELLED", refused()),
        (
            "RECONCILIATION_REQUIRED",
            "EFFECT_CONFIRMED",
            "CANCELLED",
            refused(),
        ),
        // FAILED at EFFECT_CONFIRMED only from RR (the staff close).
        ("RUNNING", "EFFECT_CONFIRMED", "FAILED", refused()),
        ("WAITING", "EFFECT_CONFIRMED", "FAILED", refused()),
        (
            "RECONCILIATION_REQUIRED",
            "EFFECT_CONFIRMED",
            "FAILED",
            Ok(()),
        ),
        // RUNNING and WAITING are cancelled only at PRE_WRITE.
        ("RUNNING", "EFFECT_STARTED", "CANCELLED", refused()),
        ("WAITING", "EFFECT_STARTED", "CANCELLED", refused()),
        ("RUNNING", "PRE_WRITE", "CANCELLED", Ok(())),
        ("WAITING", "PRE_WRITE", "CANCELLED", Ok(())),
        // WAITING fails directly only at PRE_WRITE.
        ("WAITING", "EFFECT_STARTED", "FAILED", refused()),
        ("WAITING", "PRE_WRITE", "FAILED", Ok(())),
        // Not phase-restricted at EFFECT_STARTED: T16 and T18 from RR, T9n.
        (
            "RECONCILIATION_REQUIRED",
            "EFFECT_STARTED",
            "CANCELLED",
            Ok(()),
        ),
        (
            "RECONCILIATION_REQUIRED",
            "EFFECT_STARTED",
            "FAILED",
            Ok(()),
        ),
        ("RUNNING", "EFFECT_STARTED", "FAILED", Ok(())),
    ] {
        assert_eq!(
            transition_probe(c, from, phase, to),
            expected,
            "{from} at {phase} -> {to}"
        );
    }
}

#[test]
fn backstop_an_update_of_a_hold_or_target_row_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, _claim) = running(c, new_job(TEST_EFFECT, "held", &[&x]));
    let other = create(c, new_job(TEST_EFFECT, "other", &[&key(NS_CS)])).async_job_id;
    let id = a.async_job_id;
    for (sql, guard) in [
        (
            format!(
                "UPDATE public.async_target_hold SET acquired_at = now() WHERE target_key = '{x}'"
            ),
            "async_target_hold_guard",
        ),
        (
            format!(
                "UPDATE public.async_target_hold SET async_job_id = '{other}' \
                  WHERE target_key = '{x}'"
            ),
            "async_target_hold_guard",
        ),
        (
            format!(
                "UPDATE public.async_job_target SET target_key = target_key \
                  WHERE async_job_id = '{id}'"
            ),
            "async_job_target_guard",
        ),
    ] {
        assert_eq!(refused_by(c, &sql), guard, "{sql}");
    }
    assert_eq!(holds(c, id), vec![x]);
}

#[test]
fn backstop_a_commit_leaving_a_failed_or_cancelled_job_without_its_closing_attempt_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let attempt_check = "async_job_attempt_consistency";

    // FAILED at EFFECT_STARTED needs a NO_EFFECT_PROVEN close.
    let (a, _claim) = started(c, new_job(TEST_EFFECT, "bs-failed", &[&key(NS_CS)]));
    let id = a.async_job_id;
    assert_eq!(
        refused_by(
            c,
            &terminate_sql(id, "FAILED", "PERMANENT_FAILURE", "PERMANENT")
        ),
        attempt_check
    );
    exec(
        c,
        &format!(
            "BEGIN; {}; COMMIT",
            terminate_sql(id, "FAILED", "PERMANENT_FAILURE", "NO_EFFECT_PROVEN")
        ),
    );
    assert_eq!(status(c, id), AsyncJobStatus::Failed);

    // CANCELLED at EFFECT_STARTED (from RR, under a reconciliation claim)
    // needs one too.
    let rr = rr_started(c, new_job(TEST_EFFECT, "bs-cancelled", &[&key(NS_CS)]));
    let id = rr.async_job_id;
    claim_recon(c, &rr);
    assert_eq!(
        refused_by(
            c,
            &terminate_sql(id, "CANCELLED", "CANCELLED", "OPERATOR_STOP")
        ),
        attempt_check
    );
    exec(
        c,
        &format!(
            "BEGIN; {}; COMMIT",
            terminate_sql(id, "CANCELLED", "CANCELLED", "NO_EFFECT_PROVEN")
        ),
    );
    assert_eq!(status(c, id), AsyncJobStatus::Cancelled);

    // At EFFECT_CONFIRMED: CANCELLED is refused at once (rule 4), and FAILED
    // needs a staff close with CONFIRMED_EFFECT_NOT_APPLIED; an executor
    // reconciliation attempt with that outcome, or no new attempt, is not
    // enough.
    let rr = rr_confirmed(c, new_job(TEST_EFFECT, "bs-confirmed", &[&key(NS_CS)]));
    let id = rr.async_job_id;
    assert_eq!(
        refused_by(
            c,
            &format!(
                "DELETE FROM public.async_target_hold WHERE async_job_id = '{id}'; \
                 UPDATE public.async_job SET status = 'CANCELLED', terminal_at = now() \
                  WHERE async_job_id = '{id}'"
            )
        ),
        "async_job_update_guard"
    );
    assert_eq!(
        refused_by(
            c,
            &format!(
                "DELETE FROM public.async_target_hold WHERE async_job_id = '{id}'; \
                 UPDATE public.async_job SET status = 'FAILED', terminal_at = now() \
                  WHERE async_job_id = '{id}'"
            )
        ),
        attempt_check
    );
    claim_recon(c, &rr);
    assert_eq!(
        refused_by(
            c,
            &terminate_sql(
                id,
                "FAILED",
                "PERMANENT_FAILURE",
                "CONFIRMED_EFFECT_NOT_APPLIED"
            )
        ),
        attempt_check
    );
    assert_eq!(status(c, id), AsyncJobStatus::ReconciliationRequired);
}

#[test]
fn backstop_the_deferred_s5_and_attempt_checks_refuse_a_commit_that_breaks_them() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (a, claim) = running(c, new_job(TEST_EFFECT, "bs-open", &[&key(NS_CS)]));
    let id = a.async_job_id;
    for sql in [
        // A RUNNING job without its open attempt.
        format!(
            "UPDATE public.async_job_attempt SET finished_at = now(), result = 'RELEASED', \
                 job_status_after = 'PENDING', outcome_code = 'RELEASED' \
              WHERE async_job_id = '{id}' AND finished_at IS NULL"
        ),
        // A RUNNING job whose token is not its open attempt's.
        format!(
            "UPDATE public.async_job SET claim_token = uuid_generate_v4() \
              WHERE async_job_id = '{id}'"
        ),
    ] {
        assert_eq!(
            refused_by(c, &sql),
            "async_job_attempt_consistency",
            "{sql}"
        );
    }
    // A PENDING job holding its target (S5).
    let pending = create(c, new_job(TEST_EFFECT, "bs-s5", &[&key(NS_CS)]));
    let target = &pending.target_keys[0];
    assert_eq!(
        refused_by(
            c,
            &format!(
                "INSERT INTO public.async_target_hold (target_key, async_job_id) \
                 VALUES ('{target}', '{}')",
                pending.async_job_id
            )
        ),
        "async_job_hold_consistency"
    );
    // A caller's `SET CONSTRAINTS ALL IMMEDIATE` makes the same check fire
    // at the statement instead of the commit.
    assert_eq!(
        refused_by(
            c,
            &format!(
                "SET CONSTRAINTS ALL IMMEDIATE; \
                 UPDATE public.async_job SET claim_token = uuid_generate_v4() \
                  WHERE async_job_id = '{id}'"
            )
        ),
        "async_job_attempt_consistency"
    );
    assert_live(c, &claim);
}

#[test]
fn backstop_the_state_trigger_refuses_a_non_holder_a_null_order_and_a_backwards_order() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let guard = "async_effect_target_state_guard";
    let k = key(NS_RB);
    seed_rb_state(c, &k, 5, "h5");
    let (holder, _claim) = running(c, revision_job("bs-holder", &k, 7, "h7"));
    let h = holder.async_job_id;
    let ha = open_attempt_of(c, h);
    let (intruder, _other) = running(c, new_job(TEST_PEER, "bs-intruder", &[&key(NS_CS)]));
    let i = intruder.async_job_id;
    let ia = open_attempt_of(c, i);
    let seed_job = state_writer_job(c, &k);
    let seed_attempt = attempts(c, seed_job)[0].async_job_attempt_id;
    for sql in [
        // A job that does not hold the key.
        state_write_sql(&k, Some(7), "h7", i, ia, ""),
        // The holder with a finished attempt of another job.
        state_write_sql(&k, Some(7), "h7", h, seed_attempt, ""),
        // A null order over a stored order.
        state_write_sql(&k, None, "h7", h, ha, ""),
        // A backwards order, and the same order with another fingerprint,
        // from an execution attempt.
        state_write_sql(&k, Some(4), "h4", h, ha, ""),
        state_write_sql(&k, Some(5), "hx", h, ha, ""),
    ] {
        assert_eq!(refused_by(c, &sql), guard, "{sql}");
    }
    assert_eq!(applied(c, &k), Some(rb_state(5, "h5")));
    // A delete is refused too.
    assert_eq!(
        refused_by(
            c,
            &format!("DELETE FROM public.async_effect_target_state WHERE target_key = '{k}'")
        ),
        guard
    );

    // From an open STAFF_RECONCILIATION attempt of a REVISION_BOUND job, a
    // backwards write is admitted only to that job's own source values.
    let (r, a) = regressed_job(c, 9, 4);
    let id = a.async_job_id;
    let staff_attempt = Uuid::new_v4();
    let command = Uuid::new_v4();
    let open_staff = format!(
        "INSERT INTO public.async_control_command (async_control_command_id, command_kind, \
             request_id, actor_principal, reason, async_job_id, request) \
         VALUES ('{command}', 'RESOLVE_RECONCILIATION', uuid_generate_v4(), 'staff:x', 'test', \
             '{id}', '{{}}'); \
         INSERT INTO public.async_job_attempt (async_job_attempt_id, async_job_id, \
             attempt_number, purpose, claim_token, claimed_by, executor_principal, \
             control_command_id, started_at, last_heartbeat_at, phase_at_start) \
         SELECT '{staff_attempt}', j.async_job_id, j.attempt_count + 1, 'STAFF_RECONCILIATION', \
             uuid_generate_v4(), 'staff', 'staff:x', '{command}', now(), now(), j.effect_phase \
           FROM public.async_job j WHERE j.async_job_id = '{id}'"
    );
    let staff_write = |order: i64, fingerprint: &str| {
        format!(
            "{open_staff}; {}",
            state_write_sql(&r, Some(order), fingerprint, id, staff_attempt, "")
        )
    };
    assert_eq!(probe(c, &staff_write(4, "h4"), true), Ok(()));
    for (order, fingerprint) in [(3, "h3"), (4, "hx"), (9, "hx")] {
        assert_eq!(
            probe(c, &staff_write(order, fingerprint), true),
            Err(guard.to_string()),
            "{order}/{fingerprint}"
        );
    }
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
    // The same kind of write, to the job's own source values, from an open
    // execution attempt is refused.
    let q = key(NS_RB);
    seed_rb_state(c, &q, 9, "h9");
    let (older, _claim) = running(c, revision_job("bs-older", &q, 4, "h4"));
    let oa = open_attempt_of(c, older.async_job_id);
    assert_eq!(
        refused_by(
            c,
            &state_write_sql(&q, Some(4), "h4", older.async_job_id, oa, "")
        ),
        guard
    );
    assert_eq!(applied(c, &q), Some(rb_state(9, "h9")));
}

/// The job that last wrote a key's state.
fn state_writer_job(conn: &mut PgConnection, key: &str) -> Uuid {
    state_writer(conn, key)
        .0
        .expect("a writer")
        .parse()
        .expect("a uuid")
}

#[test]
fn backstop_the_state_trigger_ignores_writer_supplied_regressed_values() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let supplied = ", regressed_at = now(), regressed_from_order = 1, \
                    regressed_from_fingerprint = 'f', updated_at = '2000-01-01'";
    // An ordinary forward write that supplies regressed values: ignored.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 5, "h5");
    let (holder, _claim) = running(c, revision_job("bs-supplied", &k, 7, "h7"));
    let h = holder.async_job_id;
    let ha = open_attempt_of(c, h);
    exec(
        c,
        &format!(
            "BEGIN; {}; COMMIT",
            state_write_sql(&k, Some(7), "h7", h, ha, supplied)
        ),
    );
    assert_eq!(applied(c, &k), Some(rb_state(7, "h7")));
    assert_eq!(regressed_text(c, &k), "||");
    assert_eq!(
        scalar(
            c,
            &format!(
                "SELECT (updated_at > now() - interval '1 hour')::int::bigint AS value \
                   FROM public.async_effect_target_state WHERE target_key = '{k}'"
            )
        ),
        1
    );
    // An insert that supplies them: ignored.
    let x = key(NS_CS);
    let (j, _claim) = running(c, new_job(TEST_EFFECT, "bs-insert", &[&x]));
    let ja = open_attempt_of(c, j.async_job_id);
    exec(
        c,
        &format!(
            "INSERT INTO public.async_effect_target_state (target_key, applied_order, \
                 applied_fingerprint, last_job_id, last_attempt_id, regressed_at, \
                 regressed_from_order, regressed_from_fingerprint, updated_at) \
             VALUES ('{x}', NULL, 'fp', '{}', '{ja}', now(), 1, 'f', '2000-01-01')",
            j.async_job_id
        ),
    );
    assert_eq!(regressed_text(c, &x), "||");
    // A write that stays below the pre-acceptance order and supplies nulls
    // keeps the recorded regression.
    let (r, a) = regressed_job(c, 9, 4);
    let accepted = do_staff(
        c,
        a.async_job_id,
        staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(4)),
    );
    assert_applied(&accepted, "REGRESSION_ACCEPTED");
    let regressed = regressed_text(c, &r);
    assert!(regressed.ends_with("|9|h9"), "{regressed}");
    let (next, _claim) = running(c, revision_job("bs-below", &r, 6, "h6"));
    let na = open_attempt_of(c, next.async_job_id);
    exec(
        c,
        &format!(
            "BEGIN; {}; COMMIT",
            state_write_sql(
                &r,
                Some(6),
                "h6",
                next.async_job_id,
                na,
                ", regressed_at = NULL, regressed_from_order = NULL, \
                 regressed_from_fingerprint = NULL"
            )
        ),
    );
    assert_eq!(applied(c, &r), Some(rb_state(6, "h6")));
    assert_eq!(regressed_text(c, &r), regressed);
}

#[test]
fn backstop_t18_succeeded_clears_the_regressed_columns_only_above_the_pre_acceptance_order() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // With `regressed_*` null, a forward T18 SUCCEEDED and one that rewrites
    // equal values leave them null.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 5, "h5");
    staff_succeeded(c, &k, 7);
    assert_eq!(applied(c, &k), Some(rb_state(7, "h7")));
    assert_eq!(regressed_text(c, &k), "||");
    staff_succeeded(c, &k, 7);
    assert_eq!(regressed_text(c, &k), "||");

    // From a regressed state (9 accepted back to 4), a forward T18 SUCCEEDED
    // that stays below 9, and an equal rewrite, leave all three unchanged.
    let (r, a) = regressed_job(c, 9, 4);
    let accepted = do_staff(
        c,
        a.async_job_id,
        staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(4)),
    );
    assert_applied(&accepted, "REGRESSION_ACCEPTED");
    let regressed = regressed_text(c, &r);
    assert!(regressed.ends_with("|9|h9"), "{regressed}");
    staff_succeeded(c, &r, 6);
    assert_eq!(applied(c, &r), Some(rb_state(6, "h6")));
    assert_eq!(regressed_text(c, &r), regressed);
    staff_succeeded(c, &r, 6);
    assert_eq!(regressed_text(c, &r), regressed);
    // One above 9 clears all three.
    staff_succeeded(c, &r, 12);
    assert_eq!(applied(c, &r), Some(rb_state(12, "h12")));
    assert_eq!(regressed_text(c, &r), "||");
}

// ---------------------------------------------------------------------------
// Accepted regression (19.2)
// ---------------------------------------------------------------------------

/// Accept the regression of `job` with its own applied evidence.
fn accept(conn: &mut PgConnection, job: &AsyncJob) -> CommandResult {
    let order = job.source_order.expect("a REVISION_BOUND job");
    do_staff(
        conn,
        job.async_job_id,
        staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(order)),
    )
}

/// A worker's forward EFFECT_APPLIED write of `order` to `key`.
fn forward_write(conn: &mut PgConnection, key: &str, order: i64, fingerprint: &str) {
    let id = format!("forward-{}", Uuid::new_v4());
    let (_job, claim) = started(conn, revision_job(&id, key, order, fingerprint));
    let outcome = do_complete(
        conn,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(key, order, fingerprint)],
        ),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
}

#[test]
fn regression_a_t6x_job_accepted_with_valid_evidence_succeeds_and_records_the_regression() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (r, a) = regressed_job(c, 9, 4);
    let id = a.async_job_id;
    assert_eq!(holds(c, id), vec![r.clone()]);
    let current = job(c, id);
    let result = accept(c, &current);
    assert_applied(&result, "REGRESSION_ACCEPTED");
    let after = job(c, id);
    assert_eq!(after.status, AsyncJobStatus::Succeeded);
    assert_eq!(
        after.last_outcome_code.as_deref(),
        Some("REGRESSION_ACCEPTED")
    );
    let row = state(c, &r).unwrap();
    assert_eq!(
        (
            row.applied_order,
            row.applied_fingerprint.as_str(),
            row.regressed,
            row.regressed_from_order,
            row.regressed_from_fingerprint.as_deref()
        ),
        (Some(4), "h4", true, Some(9), Some("h9"))
    );
    assert_eq!(state_writer_job(c, &r), id);
    let staff_attempt = last_attempt(c, id);
    assert_eq!(
        staff_attempt.purpose,
        AsyncAttemptPurpose::StaffReconciliation
    );
    assert_eq!(staff_attempt.result, Some(AsyncAttemptResult::Succeeded));
    assert_eq!(
        staff_attempt.outcome_code.as_deref(),
        Some("REGRESSION_ACCEPTED")
    );
    assert_eq!(
        staff_attempt.control_command_id,
        Some(result.async_control_command_id)
    );
    assert!(holds(c, id).is_empty());
}

#[test]
fn regression_a_forward_write_below_the_pre_acceptance_order_keeps_it_and_one_above_or_equal_clears_it(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // Below 9: kept, with the same `regressed_at`; equal order with another
    // fingerprint: kept; equal with the pre-acceptance fingerprint: cleared.
    let (r, a) = regressed_job(c, 9, 4);
    assert_applied(&accept(c, &a), "REGRESSION_ACCEPTED");
    let regressed = regressed_text(c, &r);
    forward_write(c, &r, 6, "h6");
    assert_eq!(applied(c, &r), Some(rb_state(6, "h6")));
    assert_eq!(regressed_text(c, &r), regressed);
    forward_write(c, &r, 9, "hx");
    assert_eq!(regressed_text(c, &r), regressed);
    // (A forward write of 9/h9 is now a same-order conflict, so the equal
    // case is shown on a second target.)
    let (q, b) = regressed_job(c, 8, 3);
    assert_applied(&accept(c, &b), "REGRESSION_ACCEPTED");
    forward_write(c, &q, 8, "h8");
    assert_eq!(applied(c, &q), Some(rb_state(8, "h8")));
    assert_eq!(regressed_text(c, &q), "||");
    // Above it: cleared.
    forward_write(c, &r, 10, "h10");
    assert_eq!(applied(c, &r), Some(rb_state(10, "h10")));
    assert_eq!(regressed_text(c, &r), "||");
}

#[test]
fn regression_an_executor_reconcilers_regressing_succeeded_is_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let (r, a) = regressed_job(c, 9, 4);
    let recon = claim_recon(c, &a);
    let before = footprint(c, a.async_job_id);
    let request = ResolveReconciliation {
        entries: Some(vec![rb_entry(&r, 4, "h4")]),
        ..resolution_evidence(ReconcilerOutcome::Succeeded, &applied_evidence(4))
    };
    assert_code(do_resolve(c, &recon, request), "ASYNC_JOB_EVIDENCE_INVALID");
    assert_nothing_written_and_live(c, &before, &recon);
    assert_eq!(
        status(c, a.async_job_id),
        AsyncJobStatus::ReconciliationRequired
    );
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
}

// ---------------------------------------------------------------------------
// Commands and locks (19.2)
// ---------------------------------------------------------------------------

type CommandStep = Box<dyn FnOnce(&mut PgConnection) -> ThothResult<CommandResult> + Send>;

fn cancel_step(id: Uuid) -> CommandStep {
    Box::new(move |c| cancel_job_command_on(c, &REGISTRY, &staff("cancel"), id))
}

fn retry_step(id: Uuid) -> CommandStep {
    Box::new(move |c| retry_job_command_on(c, &REGISTRY, &staff("retry"), id))
}

fn redact_step(id: Uuid) -> CommandStep {
    Box::new(move |c| redact_job_diagnostics_command_on(c, &staff("redact"), id))
}

fn resolve_step(id: Uuid, resolution: StaffResolution) -> CommandStep {
    Box::new(move |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, &staff("resolve"), id, resolution)
    })
}

/// Run `first` in one session and keep its transaction open; start `second`
/// in another, which must wait for the job row; then commit the first and
/// the second. Both results, in call order.
fn overlapped(
    observer: &mut PgConnection,
    first: CommandStep,
    second: CommandStep,
) -> (CommandResult, CommandResult) {
    let a = Session::open();
    let b = Session::open();
    let first_result = a.run(first);
    b.send(second);
    wait_until_blocked(observer, b.pid);
    a.commit();
    let second_result: ThothResult<CommandResult> = b.result();
    b.commit();
    (
        first_result.expect("the first command"),
        second_result.expect("the second command"),
    )
}

/// A job that failed permanently at PRE_WRITE, so it can be retried.
fn failed_job(conn: &mut PgConnection, id: &str) -> Uuid {
    let (job_row, claim) = running(conn, new_job(TEST_EFFECT, id, &[&key(NS_CS)]));
    do_fail(conn, &claim, fail_req(FailureClass::Permanent)).expect("T8");
    job_row.async_job_id
}

fn retry_children(conn: &mut PgConnection, id: Uuid) -> i64 {
    count(
        conn,
        &format!("FROM public.async_job WHERE parent_job_id = '{id}'"),
    )
}

#[test]
fn locks_two_different_commands_on_one_job_run_concurrently_in_both_orders() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    // RESOLVE_RECONCILIATION and REDACT_JOB_DIAGNOSTICS on an RR job.
    for resolve_first in [true, false] {
        let x = key(NS_CS);
        let rr = rr_started(
            c,
            new_job(TEST_EFFECT, &format!("pair-resolve-{resolve_first}"), &[&x]),
        );
        let id = rr.async_job_id;
        let resolve = resolve_step(
            id,
            StaffResolution {
                entries: Some(vec![cs_entry(&x, "fp")]),
                ..staff_res(StaffOutcome::Succeeded)
            },
        );
        let (resolved, redacted) = if resolve_first {
            overlapped(c, resolve, redact_step(id))
        } else {
            let (redacted, resolved) = overlapped(c, redact_step(id), resolve);
            (resolved, redacted)
        };
        assert_applied(&resolved, "EFFECT_APPLIED");
        assert_applied(&redacted, "REDACTED");
        assert_eq!(status(c, id), AsyncJobStatus::Succeeded);
        assert_eq!(applied(c, &x), Some((None, "fp".to_string())));
    }
    // CANCEL_JOB and RETRY_JOB on a FAILED job.
    for cancel_first in [true, false] {
        let id = failed_job(c, &format!("pair-retry-{cancel_first}"));
        let (cancelled, retried) = if cancel_first {
            overlapped(c, cancel_step(id), retry_step(id))
        } else {
            let (retried, cancelled) = overlapped(c, retry_step(id), cancel_step(id));
            (cancelled, retried)
        };
        assert_command_refused(&cancelled, "NOT_CANCELLABLE");
        assert_applied(&retried, "RETRY_CREATED");
        assert_eq!(retry_children(c, id), 1);
    }
    assert_no_deadlock(c, before);
}

#[test]
fn locks_two_retries_with_different_request_ids_run_concurrently_in_both_orders() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let retry_with = |id: Uuid, request_id: Uuid| -> CommandStep {
        Box::new(move |c| retry_job_command_on(c, &REGISTRY, &staff_with(request_id), id))
    };
    // Explicit order: the second waits for the first, then sees its child.
    for n in 0..2 {
        let id = failed_job(c, &format!("retry-order-{n}"));
        let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
        let (applied_retry, refused_retry) =
            overlapped(c, retry_with(id, first), retry_with(id, second));
        assert_applied(&applied_retry, "RETRY_CREATED");
        assert_command_refused(&refused_retry, "RETRY_NOT_ALLOWED");
        assert_eq!(retry_children(c, id), 1);
        // Each request id keeps its own decision.
        let replay = tx(c, |c| {
            retry_job_command_on(c, &REGISTRY, &staff_with(second), id)
        })
        .unwrap();
        assert!(replay.replayed);
        assert_command_refused(&replay, "RETRY_NOT_ALLOWED");
    }
    // Released together by a barrier: exactly one is applied.
    let id = failed_job(c, "retry-barrier");
    let task = |id: Uuid| -> Task<ThothResult<CommandResult>> {
        Box::new(move |c| {
            tx(c, |c| {
                retry_job_command_on(c, &REGISTRY, &staff("together"), id)
            })
        })
    };
    let results: Vec<CommandResult> = run_together(vec![task(id), task(id)])
        .into_iter()
        .map(|result| result.expect("a retry"))
        .collect();
    let applied_count = results.iter().filter(|r| r.is_applied()).count();
    assert_eq!(applied_count, 1, "{results:?}");
    for result in &results {
        if !result.is_applied() {
            assert_command_refused(result, "RETRY_NOT_ALLOWED");
        }
    }
    assert_eq!(retry_children(c, id), 1);
    assert_no_deadlock(c, before);
}

#[test]
fn locks_a_claim_and_a_reconciliation_claim_by_one_executor_identity_run_concurrently_in_both_orders(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let kinds = [TEST_EFFECT, TEST_PEER];
    let executor = dissemination();
    for execution_first in [true, false] {
        // RR jobs first: `rr_started` claims every ready job of its kind.
        let mut rr = Vec::new();
        let mut pending = Vec::new();
        let mut expired = Vec::new();
        for kind in kinds {
            let tag = format!("{kind}-{execution_first}");
            rr.push(
                rr_started(c, new_job(kind, &format!("rr-{tag}"), &[&key(NS_CS)])).async_job_id,
            );
            let (lapsed, _claim) =
                running(c, new_job(kind, &format!("lapsed-{tag}"), &[&key(NS_CS)]));
            expire_lease(c, lapsed.async_job_id);
            expired.push(lapsed.async_job_id);
            pending.push(
                create(c, new_job(kind, &format!("pending-{tag}"), &[&key(NS_CS)])).async_job_id,
            );
        }
        let execution = {
            let executor = executor.clone();
            move |c: &mut PgConnection| claim_on(c, &REGISTRY, &executor, request(&kinds, 10))
        };
        let reconciliation = {
            let executor = executor.clone();
            move |c: &mut PgConnection| {
                claim_reconciliation_on(c, &REGISTRY, &executor, request(&kinds, 10))
            }
        };
        // The second claim runs to its end while the first transaction is
        // still open: it never waits for the first.
        let a = Session::open();
        let b = Session::open();
        let (claimed, reconciling) = if execution_first {
            let first = a.run(execution);
            let second = b.run(reconciliation);
            (first, second)
        } else {
            let first = a.run(reconciliation);
            let second = b.run(execution);
            (second, first)
        };
        b.commit();
        a.commit();
        let ids = |result: ThothResult<ClaimResult>| {
            sorted(
                result
                    .expect("a claim")
                    .jobs
                    .into_iter()
                    .map(|job| job.async_job_id)
                    .collect(),
            )
        };
        assert_eq!(ids(claimed), sorted(pending.clone()));
        assert_eq!(ids(reconciling), sorted(rr.clone()));
        for id in expired {
            assert_eq!(status(c, id), AsyncJobStatus::RetryScheduled);
        }
    }
    assert_no_deadlock(c, before);
}

/// Expire a RUNNING job's lease `minutes_ago` and delete its hold behind the
/// triggers: at EFFECT_STARTED, T12 to RR then fails the forced hold check,
/// so its recovery fails every time.
fn break_for_recovery(conn: &mut PgConnection, job_id: Uuid, minutes_ago: i64) {
    expire_lease_ago(conn, job_id, minutes_ago);
    bypass(
        conn,
        &format!("DELETE FROM public.async_target_hold WHERE async_job_id = '{job_id}'"),
    );
}

#[test]
fn locks_recovery_with_limit_plus_one_failing_rows_still_recovers_a_newly_expired_lease() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    let limit = 3;
    let recover = |c: &mut PgConnection| tx(c, |c| recover_on(c, &REGISTRY, None, limit)).unwrap();
    // limit + 1 failing rows, due in this order. They are all started
    // before any is broken, so no claim recovers one early.
    let failing: Vec<Uuid> = (0..=limit)
        .map(|n| {
            let input = new_job(TEST_EFFECT, &format!("failing-{n}"), &[&key(NS_CS)]);
            started(c, input).0.async_job_id
        })
        .collect();
    for (minutes_ago, id) in (7..=10).rev().zip(&failing) {
        break_for_recovery(c, *id, minutes_ago);
    }
    let first = recover(c);
    assert_eq!(first.failed, failing[..3].to_vec());
    assert!(first.recovered.is_empty());
    // A lease that expires now is recovered at the next pass, ahead of the
    // rows that just failed, which move behind it.
    let (fresh, _claim) = running(c, new_job(TEST_PEER, "fresh", &[&key(NS_CS)]));
    expire_lease(c, fresh.async_job_id);
    let second = recover(c);
    assert_eq!(second.recovered, vec![fresh.async_job_id]);
    assert_eq!(second.failed, vec![failing[3], failing[0]]);
    assert_eq!(
        status(c, fresh.async_job_id),
        AsyncJobStatus::RetryScheduled
    );
    // The rows that failed longest ago come next.
    let third = recover(c);
    assert_eq!(third.failed, vec![failing[1], failing[2], failing[0]]);
    assert_eq!(
        count(c, "FROM public.async_recovery_failure"),
        i64::try_from(failing.len()).unwrap()
    );
    assert_no_deadlock(c, before);
}

#[test]
fn locks_a_row_made_inconsistent_behind_the_triggers_is_skipped_by_recovery_and_claims() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let before = deadlocks(c);
    // A WAITING row that still has its open attempt, made behind the
    // triggers, with its waiting deadline passed: T11 at PRE_WRITE fails it
    // and the forced attempt check then refuses the row.
    let (broken, _claim) = running(c, new_job(TEST_EFFECT, "ghost", &[&key(NS_CS)]));
    let id = broken.async_job_id;
    bypass(
        c,
        &format!(
            "UPDATE public.async_job SET status = 'WAITING', claim_token = NULL, \
                 claimed_by = NULL, executor_principal = NULL, claimed_at = NULL, \
                 lease_expires_at = NULL, attempt_runtime_deadline = NULL, checkpoint = '{{}}', \
                 waiting_deadline = clock_timestamp() - interval '1 second', \
                 available_at = clock_timestamp() + interval '1 hour' \
              WHERE async_job_id = '{id}'"
        ),
    );
    let healthy = create(c, new_job(TEST_EFFECT, "healthy", &[&key(NS_CS)]));
    // The claim skips the row, records its failure and still claims.
    let claimed = dissemination_claim(c, &[TEST_EFFECT]);
    assert_eq!(
        claimed.iter().map(|j| j.async_job_id).collect::<Vec<_>>(),
        vec![healthy.async_job_id]
    );
    assert_eq!(status(c, id), AsyncJobStatus::Waiting);
    let failures = |c: &mut PgConnection| {
        scalar(
            c,
            &format!(
                "SELECT COALESCE(max(failure_count), 0)::bigint AS value \
                   FROM public.async_recovery_failure \
                  WHERE async_job_id = '{id}' AND error_code = 'ASYNC_JOB_INTEGRITY_VIOLATION'"
            ),
        )
    };
    assert_eq!(failures(c), 1);
    // So does `recover_on`.
    let report = do_recover(c);
    assert_eq!(report.failed, vec![id]);
    assert!(report.recovered.is_empty());
    assert_eq!(failures(c), 2);
    assert_eq!(status(c, id), AsyncJobStatus::Waiting);
    assert_no_deadlock(c, before);
}

// ---------------------------------------------------------------------------
// Section 9 additions (19.2)
// ---------------------------------------------------------------------------

/// Run a command in its own engine-owned transaction and assert a committed
/// `REFUSED` with `code`: one command row and nothing else for the job.
#[track_caller]
fn assert_command_only(
    conn: &mut PgConnection,
    id: Uuid,
    code: &str,
    command: impl FnOnce(&mut PgConnection) -> ThothResult<CommandResult>,
) {
    let before = footprint(conn, id);
    let result = tx(conn, command).expect("a command");
    assert_command_refused(&result, code);
    assert_eq!(
        footprint(conn, id),
        Footprint {
            commands: before.commands + 1,
            ..before
        }
    );
}

/// A staff resolution refused with `code`, writing only its command row.
#[track_caller]
fn assert_staff_refused(
    conn: &mut PgConnection,
    id: Uuid,
    resolution: StaffResolution,
    code: &str,
) {
    assert_command_only(conn, id, code, |c| {
        resolve_reconciliation_command_on(c, &REGISTRY, &staff("resolve"), id, resolution)
    });
}

#[test]
fn s9_t3_renews_a_reconciliation_claim() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let a = rr_started(c, new_job(TEST_EFFECT, "s9-t3", &[&key(NS_CS)]));
    let id = a.async_job_id;
    let claim = claim_recon(c, &a);
    let before = job(c, id);
    let opened = last_attempt(c, id);
    let causes_before = causes(c, id);
    let renewal = tx(c, |c| renew_on(c, &REGISTRY, &claim, Some(300))).unwrap();
    let after = job(c, id);
    assert_eq!(after.status, AsyncJobStatus::ReconciliationRequired);
    assert_eq!(
        after.reconciliation_attempt_count,
        before.reconciliation_attempt_count
    );
    assert_eq!(after.lease_expires_at, Some(renewal.lease_expires_at));
    assert!(renewal.lease_expires_at > before.lease_expires_at.unwrap());
    assert!(renewal.lease_expires_at <= renewal.attempt_runtime_deadline);
    assert_eq!(
        after.attempt_runtime_deadline,
        Some(renewal.attempt_runtime_deadline)
    );
    assert_eq!(
        after.attempt_runtime_deadline,
        before.attempt_runtime_deadline
    );
    // The heartbeat lands on the open reconciliation attempt.
    let renewed = last_attempt(c, id);
    assert_eq!(renewed.async_job_attempt_id, opened.async_job_attempt_id);
    assert_eq!(renewed.purpose, AsyncAttemptPurpose::Reconciliation);
    assert!(renewed.finished_at.is_none());
    assert!(renewed.last_heartbeat_at > opened.last_heartbeat_at);
    // Status preserving: no transition row.
    assert_eq!(causes(c, id), causes_before);
    // The renewed claim still resolves the job, and then no longer renews.
    let row = do_resolve(c, &claim, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    assert_eq!(
        (row.status, row.claimed_by),
        (AsyncJobStatus::ReconciliationRequired, None)
    );
    assert_code(do_renew(c, &claim), STALE);

    // An expired reconciliation lease is not renewed.
    let b = rr_started(c, new_job(TEST_PEER, "s9-t3-expired", &[&key(NS_CS)]));
    let claim = claim_recon(c, &b);
    expire_lease(c, b.async_job_id);
    let before = footprint(c, b.async_job_id);
    assert_code(do_renew(c, &claim), "ASYNC_JOB_LEASE_EXPIRED");
    assert_eq!(footprint(c, b.async_job_id), before);
}

#[test]
fn s9_t16_refuses_failed_and_cancelled_at_effect_confirmed_with_the_job_rr_and_the_claim_live() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let a = rr_confirmed(c, new_job(TEST_EFFECT, "s9-t16-confirmed", &[&key(NS_CS)]));
    let id = a.async_job_id;
    let claim = claim_recon(c, &a);
    for request in [
        resolution_evidence(ReconcilerOutcome::Failed, NO_EFFECT_EVIDENCE),
        resolution_evidence(ReconcilerOutcome::Cancelled, NO_EFFECT_EVIDENCE),
        resolution_evidence(ReconcilerOutcome::Failed, NOT_APPLIED_EVIDENCE),
        // Without evidence the phase refusal still comes first.
        resolution(ReconcilerOutcome::Cancelled),
    ] {
        let before = footprint(c, id);
        assert_eq!(before.job.status, AsyncJobStatus::ReconciliationRequired);
        assert_refused(do_resolve(c, &claim, request), "EFFECT_CONFIRMED");
        assert_nothing_written_and_live(c, &before, &claim);
    }
    // The live claim then records another outcome.
    let row = do_resolve(c, &claim, resolution(ReconcilerOutcome::StillRequired)).unwrap();
    assert_eq!(
        (row.status, row.effect_phase),
        (
            AsyncJobStatus::ReconciliationRequired,
            AsyncEffectPhase::EffectConfirmed
        )
    );
}

#[test]
fn s9_t18_at_effect_confirmed_refuses_cancelled_and_closes_failed_only_with_valid_not_applied_evidence(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let a = rr_confirmed(c, new_job(TEST_EFFECT, "s9-t18-confirmed", &[&x]));
    let id = a.async_job_id;
    // CANCELLED there: a committed REFUSED, whatever its evidence.
    for evidence in [NO_EFFECT_EVIDENCE, NOT_APPLIED_EVIDENCE] {
        assert_staff_refused(
            c,
            id,
            staff_evidence(StaffOutcome::Cancelled, evidence),
            "EFFECT_CONFIRMED",
        );
    }
    // FAILED there without valid not-applied evidence: a committed REFUSED
    // with EVIDENCE_INVALID (missing, refused by the kind, no-effect
    // evidence instead, and entries in a "none" cell).
    for resolution in [
        staff_res(StaffOutcome::Failed),
        staff_evidence(StaffOutcome::Failed, r#"{"notApplied": false}"#),
        staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
        StaffResolution {
            entries: Some(vec![cs_entry(&x, "fp")]),
            ..staff_evidence(StaffOutcome::Failed, NOT_APPLIED_EVIDENCE)
        },
    ] {
        assert_staff_refused(c, id, resolution, "EVIDENCE_INVALID");
    }
    assert_eq!(commands_for(c, id, "REFUSED", "EFFECT_CONFIRMED"), 2);
    assert_eq!(commands_for(c, id, "REFUSED", "EVIDENCE_INVALID"), 4);

    // With valid not-applied evidence: FAILED, no state write, the hold
    // released.
    let result = do_staff(
        c,
        id,
        staff_evidence(StaffOutcome::Failed, NOT_APPLIED_EVIDENCE),
    );
    assert_applied(&result, "CONFIRMED_EFFECT_NOT_APPLIED");
    let row = job(c, id);
    assert_eq!(
        (row.status, row.effect_phase),
        (AsyncJobStatus::Failed, AsyncEffectPhase::EffectConfirmed)
    );
    assert_eq!(
        row.last_outcome_code.as_deref(),
        Some("CONFIRMED_EFFECT_NOT_APPLIED")
    );
    let closed = last_attempt(c, id);
    assert_eq!(closed.purpose, AsyncAttemptPurpose::StaffReconciliation);
    assert_eq!(closed.result, Some(AsyncAttemptResult::PermanentFailure));
    assert_eq!(
        closed.outcome_code.as_deref(),
        Some("CONFIRMED_EFFECT_NOT_APPLIED")
    );
    assert_eq!(
        closed.control_command_id,
        Some(result.async_control_command_id)
    );
    assert!(state(c, &x).is_none());
    assert!(holds(c, id).is_empty());
    // RETRY_JOB is then refused: the engine never replays a confirmed effect.
    assert_command_only(c, id, "RETRY_NOT_ALLOWED", |c| {
        retry_job_command_on(c, &REGISTRY, &staff("retry"), id)
    });
}

#[test]
fn s9_a_polling_kinds_confirmed_submission_rejected_later_is_closed_by_a_staff_failed() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let p = key(NS_POLL);
    // Submitted and acknowledged (EFFECT_CONFIRMED), then polled from
    // WAITING by a resumed attempt.
    let a = create(c, new_job(TEST_POLL, "s9-poll-rejected", &[&p]));
    let id = a.async_job_id;
    let claim = claim_job(c, &a);
    do_start(c, &claim).unwrap();
    do_confirm(c, &claim).unwrap();
    let claim = resumed(c, &a, &claim);
    // The provider then rejects the submission: T9 at EFFECT_CONFIRMED.
    let rejected = FailRequest {
        error_code: "SUBMISSION_REJECTED".to_string(),
        ..fail_req(FailureClass::Permanent)
    };
    let rr = do_fail(c, &claim, rejected).unwrap();
    assert_eq!(
        (rr.status, rr.effect_phase),
        (
            AsyncJobStatus::ReconciliationRequired,
            AsyncEffectPhase::EffectConfirmed
        )
    );
    assert_eq!(holds(c, id), vec![p.clone()]);
    let failed_attempt = last_attempt(c, id);
    assert_eq!(
        failed_attempt.error_code.as_deref(),
        Some("SUBMISSION_REJECTED")
    );
    // Neither SUCCEEDED nor a CANCELLED is truthful; the staff FAILED with
    // not-applied evidence closes it.
    assert_staff_refused(
        c,
        id,
        staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
        "EVIDENCE_INVALID",
    );
    let result = do_staff(
        c,
        id,
        staff_evidence(StaffOutcome::Failed, NOT_APPLIED_EVIDENCE),
    );
    assert_applied(&result, "CONFIRMED_EFFECT_NOT_APPLIED");
    assert_eq!(result.job.status, AsyncJobStatus::Failed);
    assert!(holds(c, id).is_empty());
    assert!(state(c, &p).is_none());
    assert_command_only(c, id, "RETRY_NOT_ALLOWED", |c| {
        retry_job_command_on(c, &REGISTRY, &staff("retry"), id)
    });
    // The target is free for the next job its owning domain creates.
    let (next, claim) = running(c, new_job(TEST_POLL, "s9-poll-next", &[&p]));
    assert_eq!(holds(c, next.async_job_id), vec![p.clone()]);
    do_complete(
        c,
        &claim,
        completion(CompletionOutcome::NoOp, vec![cs_entry(&p, "fp")]),
    )
    .unwrap();
    assert_eq!(status(c, next.async_job_id), AsyncJobStatus::Succeeded);
}

/// An unclaimed RR job at EFFECT_STARTED with its waiting deadline stored:
/// T10, a resumed attempt, then T9.
fn rr_with_deadline(conn: &mut PgConnection, kind: &str, id: &str) -> AsyncJob {
    let (a, claim) = started(conn, new_job(kind, id, &[&key(NS_CS)]));
    let claim = resumed(conn, &a, &claim);
    do_fail(conn, &claim, fail_req(FailureClass::Indeterminate)).expect("T9");
    let row = job(conn, a.async_job_id);
    assert_eq!(row.status, AsyncJobStatus::ReconciliationRequired);
    assert!(row.waiting_deadline.is_some());
    row
}

#[test]
fn s9_t16_and_t18_waiting_with_resume_at_after_the_waiting_deadline_are_refused() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let checkpoint = r#"{"step": 2}"#;
    let t16 = |resume_at| ResolveReconciliation {
        checkpoint: Some(checkpoint.to_string()),
        resume_at: Some(resume_at),
        ..resolution(ReconcilerOutcome::Waiting)
    };
    let t18 = |resume_at| StaffResolution {
        checkpoint: Some(checkpoint.to_string()),
        resume_at: Some(resume_at),
        ..staff_res(StaffOutcome::Waiting)
    };

    // T16 with no stored deadline: the bound is now + horizon (3600 s).
    let a = rr_started(c, new_job(TEST_EFFECT, "s9-t16-wait", &[&key(NS_CS)]));
    assert!(job(c, a.async_job_id).waiting_deadline.is_none());
    let claim = claim_recon(c, &a);
    let before = footprint(c, a.async_job_id);
    let too_late = resume_in(c, 3_700);
    assert_code(
        do_resolve(c, &claim, t16(too_late)),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    let in_bounds = resume_in(c, 60);
    let row = do_resolve(c, &claim, t16(in_bounds)).unwrap();
    assert_eq!(row.status, AsyncJobStatus::Waiting);

    // T16 with a stored deadline: one second after it is refused, the
    // deadline itself is accepted.
    let d = rr_with_deadline(c, TEST_EFFECT, "s9-t16-deadline");
    let deadline = d.waiting_deadline.unwrap();
    let claim = claim_recon(c, &d);
    let before = footprint(c, d.async_job_id);
    assert_code(
        do_resolve(c, &claim, t16(deadline + Duration::seconds(1))),
        "ASYNC_JOB_PAYLOAD_INVALID",
    );
    assert_nothing_written_and_live(c, &before, &claim);
    let row = do_resolve(c, &claim, t16(deadline)).unwrap();
    assert_eq!(
        (row.status, row.available_at, row.waiting_deadline),
        (AsyncJobStatus::Waiting, deadline, Some(deadline))
    );

    // T18, with no stored deadline and with one: a committed REFUSED with
    // WAIT_INVALID, then the deadline itself is accepted.
    let b = rr_started(c, new_job(TEST_PEER, "s9-t18-wait", &[&key(NS_CS)]));
    let e = rr_with_deadline(c, TEST_PEER, "s9-t18-deadline");
    let deadline = e.waiting_deadline.unwrap();
    let too_late = resume_in(c, 3_700);
    assert_staff_refused(c, b.async_job_id, t18(too_late), "WAIT_INVALID");
    assert_staff_refused(
        c,
        e.async_job_id,
        t18(deadline + Duration::seconds(1)),
        "WAIT_INVALID",
    );
    let result = do_staff(c, e.async_job_id, t18(deadline));
    assert_applied(&result, "WAITING");
    assert_eq!(
        (result.job.status, result.job.available_at),
        (AsyncJobStatus::Waiting, deadline)
    );
}

#[test]
fn s9_t18_succeeded_with_invalid_evidence_is_a_committed_refused_evidence_invalid() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    // CURRENT_STATE: evidence the kind refuses, no entries, an entry for
    // another target, and an empty entry list.
    let x = key(NS_CS);
    let a = rr_started(c, new_job(TEST_EFFECT, "s9-t18-invalid", &[&x]));
    let id = a.async_job_id;
    for resolution in [
        StaffResolution {
            entries: Some(vec![cs_entry(&x, "fp")]),
            evidence: Some(r#"{"valid": false}"#.to_string()),
            ..staff_res(StaffOutcome::Succeeded)
        },
        staff_res(StaffOutcome::Succeeded),
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&key(NS_CS), "fp")]),
        staff_entries(StaffOutcome::Succeeded, vec![]),
    ] {
        assert_staff_refused(c, id, resolution, "EVIDENCE_INVALID");
    }
    assert!(state(c, &x).is_none());
    // REVISION_BOUND (stored 9, the job's own revision 4): the regressing
    // write, which only ACCEPT_REGRESSION records, and an entry other than
    // the job's revision.
    let (r, b) = regressed_job(c, 9, 4);
    for resolution in [
        StaffResolution {
            entries: Some(vec![rb_entry(&r, 4, "h4")]),
            evidence: Some(applied_evidence(4)),
            ..staff_res(StaffOutcome::Succeeded)
        },
        staff_entries(StaffOutcome::Succeeded, vec![rb_entry(&r, 9, "h9")]),
    ] {
        assert_staff_refused(c, b.async_job_id, resolution, "EVIDENCE_INVALID");
    }
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
    assert_eq!(regressed_text(c, &r), "||");
    // A forward revision whose evidence names another revision.
    let q = key(NS_RB);
    let f = rr_started(c, revision_job("s9-t18-forward", &q, 7, "h7"));
    assert_staff_refused(
        c,
        f.async_job_id,
        StaffResolution {
            entries: Some(vec![rb_entry(&q, 7, "h7")]),
            evidence: Some(applied_evidence(8)),
            ..staff_res(StaffOutcome::Succeeded)
        },
        "EVIDENCE_INVALID",
    );
    assert!(state(c, &q).is_none());
    // A valid SUCCEEDED still applies afterwards.
    let result = do_staff(
        c,
        id,
        staff_entries(StaffOutcome::Succeeded, vec![cs_entry(&x, "fp")]),
    );
    assert_applied(&result, "EFFECT_APPLIED");
    assert_eq!(applied(c, &x), Some((None, "fp".to_string())));
}

#[test]
fn s9_worker_writes_and_commands_on_a_kind_unknown_to_the_registry_are_refused_and_redaction_still_applies(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let narrow = &NARROW_REGISTRY;
    let unknown = "ASYNC_JOB_KIND_UNKNOWN";
    assert!(narrow.get(TEST_EFFECT).is_none() && narrow.get(TEST_PEER).is_none());

    // Every worker write on an execution claim: refused, nothing written.
    let x = key(NS_CS);
    let (a, claim) = running(c, new_job(TEST_EFFECT, "s9-unknown-run", &[&x]));
    let before = footprint(c, a.async_job_id);
    assert_code(tx(c, |c| renew_on(c, narrow, &claim, None)), unknown);
    assert_code(
        tx(c, |c| mark_effect_started_on(c, narrow, &claim)),
        unknown,
    );
    assert_code(
        tx(c, |c| {
            mark_effect_confirmed_on(c, narrow, &claim, Some("provider-1"))
        }),
        unknown,
    );
    assert_code(
        tx(c, |c| {
            complete_on(
                c,
                narrow,
                &claim,
                completion(CompletionOutcome::NoOp, vec![cs_entry(&x, "fp")]),
            )
        }),
        unknown,
    );
    assert_code(
        tx(c, |c| {
            fail_on(c, narrow, &claim, fail_req(FailureClass::Permanent))
        }),
        unknown,
    );
    assert_code(
        tx(c, |c| wait_on(c, narrow, &claim, wait_req("{}"))),
        unknown,
    );
    assert_code(tx(c, |c| release_on(c, narrow, &claim)), unknown);
    assert_code(tx(c, |c| read_target_state_on(c, narrow, &claim)), unknown);
    assert_nothing_written_and_live(c, &before, &claim);

    // And on a reconciliation claim.
    let r = rr_started(c, new_job(TEST_PEER, "s9-unknown-recon", &[&key(NS_CS)]));
    let recon = claim_recon(c, &r);
    let before = footprint(c, r.async_job_id);
    assert_code(tx(c, |c| renew_on(c, narrow, &recon, None)), unknown);
    assert_code(
        tx(c, |c| {
            resolve_reconciliation_on(
                c,
                narrow,
                &recon,
                resolution(ReconcilerOutcome::StillRequired),
            )
        }),
        unknown,
    );
    assert_nothing_written_and_live(c, &before, &recon);

    // Commands that the full registry would apply: a committed REFUSED with
    // KIND_UNKNOWN, and nothing else.
    let id = a.async_job_id;
    assert_command_only(c, id, "KIND_UNKNOWN", |c| {
        cancel_job_command_on(c, narrow, &staff("cancel"), id)
    });
    let failed = failed_job(c, "s9-unknown-failed");
    assert_command_only(c, failed, "KIND_UNKNOWN", |c| {
        retry_job_command_on(c, narrow, &staff("retry"), failed)
    });
    let u = rr_started(c, new_job(TEST_PEER, "s9-unknown-rr", &[&key(NS_CS)]));
    let rr = u.async_job_id;
    assert_command_only(c, rr, "KIND_UNKNOWN", |c| {
        resolve_reconciliation_command_on(
            c,
            narrow,
            &staff("resolve"),
            rr,
            staff_evidence(StaffOutcome::Failed, NO_EFFECT_EVIDENCE),
        )
    });
    assert_eq!(status(c, rr), AsyncJobStatus::ReconciliationRequired);

    // Redaction needs no kind policy: it still applies to the failed job.
    let result = do_redact(c, failed);
    assert_applied(&result, "REDACTED");
    assert!(job(c, failed).redacted_at.is_some());
}

/// `TEST_EFFECT` with each cap at its section 8 maximum, and a backoff whose
/// doubling passes `backoff_max` at n = 3.
static CAPPED_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[AsyncJobKindPolicy {
    retry_budget: 20,
    max_execution_claims: 100,
    max_reconciliation_attempts: 50,
    backoff_base_seconds: 40_000,
    backoff_max_seconds: 86_400,
    ..EFFECT_POLICY
}]);

#[test]
fn s9_backoff_saturates_at_each_caps_maximum_counter_and_the_jitter_is_in_range_for_the_minimum_bigint_hash(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    assert_eq!(CAPPED_REGISTRY.validate(), Ok(()));
    let capped = |c: &mut PgConnection, id: &str| -> AsyncJob {
        let input = new_job(TEST_EFFECT, id, &[&key(NS_CS)]);
        tx(c, |c| create_job_on(c, &CAPPED_REGISTRY, input))
            .expect("create")
            .into_job()
    };

    // The pre-write failure count: T7 up to n = retry_budget - 1 = 19, each
    // available_at matching the independent formula (saturated from n = 3);
    // the 20th failure ends the job.
    let a = capped(c, "s9-backoff-pre");
    let id = a.async_job_id;
    assert_eq!(
        (
            a.retry_budget,
            a.max_execution_claims,
            a.max_reconciliation_attempts
        ),
        (20, 100, 50)
    );
    for n in 1..=19_i16 {
        make_available(c, id);
        let claim = claim_job(c, &a);
        let row = do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
        assert_eq!(
            (row.status, row.prewrite_failure_count),
            (AsyncJobStatus::RetryScheduled, n)
        );
        let finished = last_attempt(c, id).finished_at.unwrap();
        let delay = expected_backoff(c, &row, i32::from(n));
        assert_eq!(row.available_at, finished + Duration::seconds(delay));
        let base = backoff_delay_seconds(40_000, 86_400, i32::from(n));
        assert_eq!(base, if n < 3 { 40_000 << (n - 1) } else { 86_400 });
    }
    make_available(c, id);
    let claim = claim_job(c, &a);
    let row = do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    assert_eq!(
        (row.status, row.prewrite_failure_count),
        (AsyncJobStatus::Failed, 20)
    );
    assert_eq!(
        row.last_outcome_code.as_deref(),
        Some("RETRY_BUDGET_EXHAUSTED")
    );

    // The reconciliation attempt count: T16 STILL_REQUIRED up to n = 49,
    // then T17 at n = max_reconciliation_attempts = 50.
    let b = capped(c, "s9-backoff-recon");
    let rid = b.async_job_id;
    let claim = claim_job(c, &b);
    do_start(c, &claim).unwrap();
    do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
    for n in 1..=50_i16 {
        let claim = claim_recon(c, &b);
        if n < 50 {
            do_resolve(c, &claim, resolution(ReconcilerOutcome::StillRequired)).unwrap();
        } else {
            expire_lease(c, rid);
            assert_eq!(do_recover(c).recovered, vec![rid]);
        }
        let row = job(c, rid);
        assert_eq!(
            (row.status, row.reconciliation_attempt_count),
            (AsyncJobStatus::ReconciliationRequired, n)
        );
        let finished = last_attempt(c, rid).finished_at.unwrap();
        let delay = expected_backoff(c, &row, i32::from(n));
        assert_eq!(row.available_at, finished + Duration::seconds(delay));
        make_available(c, rid);
    }
    // At the cap no executor claims it again.
    let none = tx(c, |c| {
        claim_reconciliation_on(c, &REGISTRY, &dissemination(), request(&[TEST_EFFECT], 50))
    })
    .unwrap();
    assert!(none.jobs.is_empty());

    // The formula itself at the largest counters the caps admit and beyond,
    // for every base and maximum: saturated, never overflowing.
    for (base, max) in [
        (1, 86_400),
        (40_000, 86_400),
        (86_400, 86_400),
        (i32::MAX, i32::MAX),
    ] {
        for n in [20, 31, 32, 50, 100, i32::MAX] {
            assert_eq!(
                backoff_delay_seconds(base, max, n),
                i64::from(max),
                "{base}/{max}/{n}"
            );
        }
    }
    assert_eq!(backoff_delay_seconds(1, 86_400, 17), 65_536);
    assert_eq!(backoff_delay_seconds(1, 86_400, 18), 86_400);
    assert_eq!(backoff_delay_seconds(1, i32::MAX, 31), 1 << 30);
    assert_eq!(backoff_delay_seconds(10, 300, 0), 10);
    assert_eq!(jittered_delay_seconds(86_400, 20), 103_680);
    assert_eq!(
        jittered_delay_seconds(i64::from(i32::MAX), 20),
        i64::from(i32::MAX) + i64::from(i32::MAX) / 5
    );

    // The jitter over the minimum `bigint`, where `abs` overflows, and other
    // edges: always in 0..=20.
    assert!(c
        .batch_execute("SELECT abs((-9223372036854775808)::bigint)")
        .is_err());
    for (hash, expected) in [
        ("(-9223372036854775808)::bigint", 13),
        ("9223372036854775807::bigint", 7),
        ("(-1)::bigint", 20),
        ("0::bigint", 0),
        ("(-21)::bigint", 0),
        ("41::bigint", 20),
    ] {
        let j = scalar(c, &format!("SELECT {} AS value", jitter_expression(hash)));
        assert_eq!(j, expected, "{hash}");
    }
}

#[test]
fn s9_retry_job_after_a_proven_no_effect_failed_job_takes_its_policy_snapshot_from_the_current_registry(
) {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let x = key(NS_CS);
    let (a, claim) = started(c, new_job(TEST_EFFECT, "s9-retry-t9n", &[&x]));
    let failed = do_fail(
        c,
        &claim,
        no_effect_req(FailureClass::Permanent, Some(NO_EFFECT_EVIDENCE)),
    )
    .unwrap();
    assert_eq!(
        (failed.status, failed.effect_phase),
        (AsyncJobStatus::Failed, AsyncEffectPhase::EffectStarted)
    );
    assert_eq!(
        failed.last_outcome_code.as_deref(),
        Some("NO_EFFECT_PROVEN")
    );
    assert_eq!(
        (
            failed.retry_budget,
            failed.max_execution_claims,
            failed.backoff_base_seconds,
            failed.backoff_max_seconds
        ),
        (3, 4, 10, 300)
    );
    // A later binary retunes the kind: the retry takes its snapshot from it.
    let result = tx(c, |c| {
        retry_job_command_on(c, &RETUNED_REGISTRY, &staff("retry"), a.async_job_id)
    })
    .unwrap();
    assert_applied(&result, "RETRY_CREATED");
    let child_id = result.result_job_id.expect("the new job");
    let child = job(c, child_id);
    assert_eq!(child.parent_job_id, Some(a.async_job_id));
    assert_eq!(child.lineage, AsyncJobLineage::ManualRetry);
    assert_eq!(
        (child.status, child.effect_phase, child.attempt_count),
        (AsyncJobStatus::Pending, AsyncEffectPhase::PreWrite, 0)
    );
    assert_eq!(
        (
            child.retry_budget,
            child.max_execution_claims,
            child.max_reconciliation_attempts,
            child.backoff_base_seconds,
            child.backoff_max_seconds
        ),
        (5, 6, 2, 20, 600)
    );
    assert_eq!(child.target_keys, vec![x.clone()]);
    // The new job runs normally under that snapshot: a pre-write failure
    // takes the retuned backoff, then the job succeeds.
    let claim = claim_job(c, &child);
    let row = do_fail(c, &claim, fail_req(FailureClass::RetryablePreWrite)).unwrap();
    let finished = last_attempt(c, child_id).finished_at.unwrap();
    let delay = expected_backoff(c, &row, 1);
    assert!((20..=24).contains(&delay), "{delay}");
    assert_eq!(row.available_at, finished + Duration::seconds(delay));
    make_available(c, child_id);
    let claim = claim_job(c, &child);
    do_start(c, &claim).unwrap();
    let outcome = do_complete(
        c,
        &claim,
        completion(CompletionOutcome::EffectApplied, vec![cs_entry(&x, "fp")]),
    )
    .unwrap();
    assert!(matches!(outcome, CompleteOutcome::Succeeded(_)));
    assert_eq!(applied(c, &x), Some((None, "fp".to_string())));
}

/// Accept the regression or conflict of the RR job `rr` over the stored
/// revision `stored` of its one target, and check what it records.
#[track_caller]
fn assert_regression_accepted(conn: &mut PgConnection, rr: &AsyncJob, stored: (i64, &str)) {
    let id = rr.async_job_id;
    let k = rr.target_keys[0].clone();
    assert_eq!(status(conn, id), AsyncJobStatus::ReconciliationRequired);
    assert_eq!(holds(conn, id), vec![k.clone()]);
    assert_applied(&accept(conn, rr), "REGRESSION_ACCEPTED");
    let after = job(conn, id);
    assert_eq!(
        (after.status, after.last_outcome_code.as_deref()),
        (AsyncJobStatus::Succeeded, Some("REGRESSION_ACCEPTED"))
    );
    let row = state(conn, &k).expect("a state row");
    assert_eq!(
        (
            row.applied_order,
            row.applied_fingerprint.as_str(),
            row.applied_source_ref.as_deref(),
            row.regressed,
            row.regressed_from_order,
            row.regressed_from_fingerprint.as_deref()
        ),
        (
            rr.source_order,
            rr.source_fingerprint.as_deref().unwrap(),
            rr.source_ref.as_deref(),
            true,
            Some(stored.0),
            Some(stored.1)
        )
    );
    assert_eq!(state_writer_job(conn, &k), id);
    assert!(holds(conn, id).is_empty());
}

#[test]
fn s9_accept_regression_is_accepted_after_every_route_into_reconciliation() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let confirmed_phase = |c: &mut PgConnection, id: Uuid| job(c, id).effect_phase;

    // After T6x.
    let (_r, a) = regressed_job(c, 9, 4);
    assert_regression_accepted(c, &a, (9, "h9"));

    // After T12, T9 and T14e at EFFECT_CONFIRMED, with a newer stored order.
    for route in ["t12", "t9", "t14e"] {
        let k = key(NS_RB);
        seed_rb_state(c, &k, 9, "h9");
        let (a, claim) = confirmed(c, revision_job(&format!("s9-{route}"), &k, 4, "h4"));
        let id = a.async_job_id;
        match route {
            "t12" => {
                expire_lease(c, id);
                assert_eq!(do_recover(c).recovered, vec![id]);
            }
            "t9" => {
                do_fail(c, &claim, fail_req(FailureClass::Indeterminate)).unwrap();
            }
            _ => assert_applied(&do_cancel(c, id), "OPERATOR_STOP"),
        }
        assert_eq!(confirmed_phase(c, id), AsyncEffectPhase::EffectConfirmed);
        assert_regression_accepted(c, &a, (9, "h9"));
    }

    // After T10 then T11 at EFFECT_CONFIRMED.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 9, "h9");
    let (a, claim) = confirmed(c, revision_job("s9-t10-t11", &k, 4, "h4"));
    let resume_at = resume_in(c, 10);
    do_wait(c, &claim, "{}", resume_at).unwrap();
    expire_waiting_deadline(c, a.async_job_id, 1);
    assert_eq!(do_recover(c).recovered, vec![a.async_job_id]);
    assert_eq!(
        confirmed_phase(c, a.async_job_id),
        AsyncEffectPhase::EffectConfirmed
    );
    assert_regression_accepted(c, &a, (9, "h9"));

    // After T6x, then WAITING (T16), then T12 of the resumed attempt (its
    // own revision 3, since an effect identity is used once).
    let (_r, a) = regressed_job(c, 9, 3);
    let id = a.async_job_id;
    let recon = claim_recon(c, &a);
    let resume_at = resume_in(c, 10);
    let waiting = do_resolve(
        c,
        &recon,
        ResolveReconciliation {
            checkpoint: Some(r#"{"step": 1}"#.to_string()),
            resume_at: Some(resume_at),
            ..resolution(ReconcilerOutcome::Waiting)
        },
    )
    .unwrap();
    assert_eq!(waiting.status, AsyncJobStatus::Waiting);
    make_available(c, id);
    claim_job(c, &a);
    expire_lease(c, id);
    assert_eq!(do_recover(c).recovered, vec![id]);
    assert_regression_accepted(c, &a, (9, "h9"));

    // An equal order with another fingerprint (a conflict, by T6x).
    let k = key(NS_RB);
    seed_rb_state(c, &k, 6, "h6");
    let (a, claim) = started(c, revision_job("s9-conflict", &k, 6, "h6b"));
    let outcome = do_complete(
        c,
        &claim,
        completion(
            CompletionOutcome::EffectApplied,
            vec![rb_entry(&k, 6, "h6b")],
        ),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CompleteOutcome::ReconciliationRequired(_)
    ));
    assert_regression_accepted(c, &a, (6, "h6"));
    assert_no_unspecified(c);
}

#[test]
fn s9_accept_regression_is_refused_without_a_regression_or_valid_applied_evidence() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let accept_with =
        |order: i64| staff_evidence(StaffOutcome::AcceptRegression, &applied_evidence(order));

    // NOT_A_REGRESSION, even with valid evidence: a lower stored order, an
    // equal order with the same fingerprint, and no stored state.
    let k = key(NS_RB);
    seed_rb_state(c, &k, 3, "h3");
    let lower = rr_started(c, revision_job("s9-nar-lower", &k, 5, "h5"));
    assert_staff_refused(c, lower.async_job_id, accept_with(5), "NOT_A_REGRESSION");
    assert_eq!(applied(c, &k), Some(rb_state(3, "h3")));
    let k = key(NS_RB);
    seed_rb_state(c, &k, 5, "h5");
    let equal = rr_started(c, revision_job("s9-nar-equal", &k, 5, "h5"));
    assert_staff_refused(c, equal.async_job_id, accept_with(5), "NOT_A_REGRESSION");
    assert_eq!(applied(c, &k), Some(rb_state(5, "h5")));
    let k = key(NS_RB);
    let absent = rr_started(c, revision_job("s9-nar-absent", &k, 5, "h5"));
    assert_staff_refused(c, absent.async_job_id, accept_with(5), "NOT_A_REGRESSION");
    assert!(state(c, &k).is_none());

    // EVIDENCE_INVALID on a real regression: no evidence, evidence the kind
    // refuses, evidence naming another revision, and entries in a "none"
    // cell.
    let (r, a) = regressed_job(c, 9, 4);
    let id = a.async_job_id;
    for resolution in [
        staff_res(StaffOutcome::AcceptRegression),
        staff_evidence(
            StaffOutcome::AcceptRegression,
            r#"{"valid": false, "order": 4}"#,
        ),
        accept_with(9),
        StaffResolution {
            entries: Some(vec![rb_entry(&r, 4, "h4")]),
            ..accept_with(4)
        },
    ] {
        assert_staff_refused(c, id, resolution, "EVIDENCE_INVALID");
    }
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
    assert_eq!(regressed_text(c, &r), "||");
    // With valid evidence the same job is accepted.
    assert_regression_accepted(c, &a, (9, "h9"));
}

#[test]
fn s9_the_state_trigger_refuses_a_backwards_or_same_order_write_by_a_reconciliation_attempt() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let guard = "async_effect_target_state_guard";
    // A regression job (stored 9, its own revision 4) in RR, with an open
    // executor reconciliation attempt.
    let (r, a) = regressed_job(c, 9, 4);
    let id = a.async_job_id;
    claim_recon(c, &a);
    let ra = open_attempt_of(c, id);
    assert_eq!(
        last_attempt(c, id).purpose,
        AsyncAttemptPurpose::Reconciliation
    );
    // The job's own source values, another older revision, and the same
    // order with another fingerprint: each refused.
    for (order, fingerprint) in [(4, "h4"), (3, "h3"), (9, "hx")] {
        assert_eq!(
            refused_by(
                c,
                &state_write_sql(&r, Some(order), fingerprint, id, ra, "")
            ),
            guard,
            "{order}/{fingerprint}"
        );
    }
    // A forward write by the same attempt passes the trigger (rolled back),
    // so the refusals above are the backwards rule's.
    assert_eq!(
        probe(c, &state_write_sql(&r, Some(10), "h10", id, ra, ""), true),
        Ok(())
    );
    assert_eq!(applied(c, &r), Some(rb_state(9, "h9")));
    assert_eq!(regressed_text(c, &r), "||");
}

#[test]
fn s9_a_pre_write_job_facing_an_equal_order_with_another_fingerprint_can_only_fail_permanent() {
    let (_guard, mut conn) = setup();
    let c = &mut conn;
    let k = key(NS_RB);
    seed_rb_state(c, &k, 6, "h6");
    let (a, claim) = running(c, revision_job("s9-pre-conflict", &k, 6, "h6b"));
    let id = a.async_job_id;
    // No completion is valid at PRE_WRITE: each is refused, nothing written,
    // the claim live.
    for (request, reason) in [
        (
            completion(CompletionOutcome::NoOp, vec![rb_entry(&k, 6, "h6b")]),
            None,
        ),
        (
            completion(CompletionOutcome::NoOp, vec![rb_entry(&k, 6, "h6")]),
            None,
        ),
        (
            completion(
                CompletionOutcome::Superseded,
                vec![EvidenceEntry::key_only(&k)],
            ),
            None,
        ),
        (
            completion(
                CompletionOutcome::EffectApplied,
                vec![rb_entry(&k, 6, "h6b")],
            ),
            Some("PHASE"),
        ),
        (
            completion(
                CompletionOutcome::Completed,
                vec![EvidenceEntry::key_only(&k)],
            ),
            Some("KIND_POLICY"),
        ),
    ] {
        let before = footprint(c, id);
        let result = do_complete(c, &claim, request);
        match reason {
            None => assert_code(result, "ASYNC_JOB_EVIDENCE_INVALID"),
            Some(reason) => assert_refused(result, reason),
        }
        assert_nothing_written_and_live(c, &before, &claim);
    }
    // fail(PERMANENT), its error code naming the conflict: FAILED, nothing
    // written, the hold released.
    let conflict = FailRequest {
        error_code: "SOURCE_REVISION_CONFLICT".to_string(),
        ..fail_req(FailureClass::Permanent)
    };
    let failed = do_fail(c, &claim, conflict).unwrap();
    assert_eq!(
        (failed.status, failed.effect_phase),
        (AsyncJobStatus::Failed, AsyncEffectPhase::PreWrite)
    );
    let closed = last_attempt(c, id);
    assert_eq!(closed.result, Some(AsyncAttemptResult::PermanentFailure));
    assert_eq!(
        closed.error_code.as_deref(),
        Some("SOURCE_REVISION_CONFLICT")
    );
    assert_eq!(applied(c, &k), Some(rb_state(6, "h6")));
    assert!(holds(c, id).is_empty());
}
