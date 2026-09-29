//! `MET-WP4-01` protected rollup application evidence at the API boundary.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the two approved operations, their
//! fail-closed `METRICS_INGEST_SERVICE`-only authorization, the complete
//! negative matrix, the fact that authorization precedes every
//! rollup-specific read, lock and write, the bounded error contract as seen
//! through GraphQL, and the strictly additive SDL.
//!
//! `MET-WP4-03A` adds no operation, type, field or argument: the derived
//! monthly projections are maintained inside the existing completion and
//! never cross the API. The evidence here is that the resolver path still
//! maintains them.
//!
//! `MET-WP4-03A-OPS-02` adds exactly two further protected operations,
//! `verifyMetricRollupMonths` and `rebuildMetricRollupMonths`, under the same
//! `METRICS_INGEST_SERVICE`-only boundary. The evidence here is their
//! complete authorization matrix, that a denied request issues no
//! rollup-specific statement at all, the exact bounded result and input
//! shapes, the fixed rejections as seen through GraphQL, and that the
//! generated schema gains exactly those two operations and three result
//! types, one input type, and no query-side surface.
//!
//! `MET-WP4-03C-A` adds no operation either: the same two maintenance
//! operations become authoritative over six derived projection families
//! (four monthly, two yearly), `MetricRollupMonthVerification` gains exactly
//! the two additive fields `countryYear` and `institutionYear`, and the
//! descriptions of the operations and result types state the six-family
//! contract. The evidence here is the unchanged operation set and
//! signatures, the exact additive shape, the corrected descriptions, and a
//! yearly-only corruption diagnosed and repaired through the resolvers.
//!
//! Frontier, lease, atomicity, arithmetic, watermark, monthly and yearly
//! resolution, verification, rebuild, rollback, concurrency and migration
//! evidence lives with the durable state itself, in
//! `crate::model::metric_rollup_delta::tests`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use diesel::RunQueryDsl;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_platform::tests::setup_registry_db;
use crate::model::metric_record_revision::tests::fixture_record;
use crate::model::metric_rollup_delta::tests::{
    commit_work_day_delta, country_year_rows, day, deltas, logging_pool, month_rows, projection,
    state, DayDimensions, DAY_ONE, DAY_TWO,
};
use crate::model::tests::db as test_db;
use crate::policy::Role;

/// The authenticated machine principal the authorized rows act as.
const INGEST_USER: &str = "metrics-ingest-service-1";

// --------------------------------------------------------------------------
// Execution helpers
// --------------------------------------------------------------------------

fn request(query: &str) -> GraphQLRequest {
    serde_json::from_value(json!({ "query": query })).expect("build GraphQL request")
}

async fn run(schema: &Schema, context: &Context, query: &str) -> JsonValue {
    serde_json::to_value(request(query).execute(schema, context).await)
        .expect("serialize GraphQL response")
}

fn data<'a>(response: &'a JsonValue, field: &str) -> &'a JsonValue {
    assert!(
        response.get("errors").is_none()
            || response["errors"].as_array().is_some_and(Vec::is_empty),
        "unexpected GraphQL errors: {response}"
    );
    &response["data"][field]
}

fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1, "expected exactly one error: {response}");
    (
        errors[0]["message"].as_str().expect("message").to_string(),
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("extensions.type")
            .to_string(),
    )
}

fn assert_unauthorized(response: &JsonValue) {
    let (message, kind) = only_error(response);
    assert_eq!(
        kind, "NO_ACCESS",
        "expected a fail-closed denial: {response}"
    );
    assert_eq!(message, "Unauthorized");
}

fn user_with(user_id: &str, roles: &[(Role, &str)]) -> IntrospectedUser {
    let mut project_roles: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (role, org_id) in roles {
        project_roles
            .entry(role.as_ref().to_string())
            .or_default()
            .insert((*org_id).to_string(), "role".to_string());
    }
    IntrospectedUser {
        user_id: user_id.to_string(),
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

fn context_for(pool: &Arc<PgPool>, user: Option<IntrospectedUser>) -> Context {
    match user {
        Some(user) => test_db::test_context_with_user(Arc::clone(pool), user),
        None => test_db::test_context_anonymous(Arc::clone(pool)),
    }
}

// --------------------------------------------------------------------------
// Operation documents
// --------------------------------------------------------------------------

const CLAIM: &str = "mutation { claimMetricRollupDeltas(limit: 10) \
                       { deltaId sequence claimToken leaseExpiresAt } }";

fn complete(claim_token: Uuid) -> String {
    format!(
        "mutation {{ completeMetricRollupDeltas(input: {{ claimToken: \"{claim_token}\" }}) \
           {{ appliedThroughSequence watermarkAt }} }}"
    )
}

/// Every field of the verification result, once.
const VERIFICATION_FIELDS: &str = "appliedThroughSequence nextSequence watermarkAt \
     maxWorkDayWatermark workDayRowCount representedMonthKeyCount \
     total { expectedRows actualRows missingRows extraRows mismatchedRows } \
     country { expectedRows actualRows missingRows extraRows mismatchedRows } \
     institution { expectedRows actualRows missingRows extraRows mismatchedRows } \
     ambiguity { expectedRows actualRows missingRows extraRows mismatchedRows } \
     countryYear { expectedRows actualRows missingRows extraRows mismatchedRows } \
     institutionYear { expectedRows actualRows missingRows extraRows mismatchedRows } \
     matches";

fn verify_document() -> String {
    format!("mutation {{ verifyMetricRollupMonths {{ {VERIFICATION_FIELDS} }} }}")
}

fn rebuild_document(expected: &str) -> String {
    format!(
        "mutation {{ rebuildMetricRollupMonths(input: {{ expectedAppliedThroughSequence: \
           \"{expected}\" }}) {{ rebuilt verification {{ {VERIFICATION_FIELDS} }} }} }}"
    )
}

/// The captured statements that touch rollup state at all, without the
/// bind suffix Diesel's instrumentation appends to a query's text.
fn rollup_statements(log: &std::sync::Mutex<Vec<String>>) -> Vec<String> {
    log.lock()
        .expect("statement log")
        .iter()
        .map(|statement| {
            statement
                .split_once(" -- binds:")
                .map_or(statement.as_str(), |(text, _)| text)
                .to_string()
        })
        .filter(|statement| {
            statement.contains("metric_rollup") || statement.to_uppercase().starts_with("BEGIN")
        })
        .collect()
}

fn ingest_context(pool: &Arc<PgPool>) -> Context {
    context_for(
        pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    )
}

/// One publisher fixture with two committed work-day deltas.
fn seeded(pool: &Arc<PgPool>) {
    let (fixture, _record_id) = fixture_record(pool, "identity-base");
    commit_work_day_delta(
        pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    commit_work_day_delta(
        pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );
}

/// Every caller of the approved negative matrix, in matrix order.
///
/// The tuple is `(label, user, may_apply_rollups)`.
fn matrix(org: &str) -> Vec<(&'static str, Option<IntrospectedUser>, bool)> {
    vec![
        ("anonymous", None, false),
        (
            "authenticated, no applicable role",
            Some(user_with("no-roles", &[])),
            false,
        ),
        (
            "PUBLISHER_USER",
            Some(user_with("owner", &[(Role::PublisherUser, org)])),
            false,
        ),
        (
            "PUBLISHER_ADMIN",
            Some(user_with("admin", &[(Role::PublisherAdmin, org)])),
            false,
        ),
        (
            "WORK_LIFECYCLE",
            Some(user_with("lifecycle", &[(Role::WorkLifecycle, org)])),
            false,
        ),
        (
            "CDN_WRITE",
            Some(user_with("cdn", &[(Role::CdnWrite, org)])),
            false,
        ),
        (
            "SUPERUSER",
            Some(user_with("root", &[(Role::Superuser, org)])),
            false,
        ),
        (
            "DISSEMINATION_WORKER",
            Some(user_with("worker", &[(Role::DisseminationWorker, org)])),
            false,
        ),
        (
            "METRICS_READ_SERVICE",
            Some(user_with("reader", &[(Role::MetricsReadService, org)])),
            false,
        ),
        (
            "METRICS_INGEST_SERVICE",
            Some(user_with(INGEST_USER, &[(Role::MetricsIngestService, org)])),
            true,
        ),
        (
            "METRICS_INGEST_SERVICE alongside unrelated roles",
            Some(user_with(
                INGEST_USER,
                &[
                    (Role::MetricsIngestService, org),
                    (Role::PublisherUser, "org-elsewhere"),
                ],
            )),
            true,
        ),
    ]
}

// ==========================================================================
// The complete authorization matrix
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_claim() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();

    for (label, user, allowed) in matrix("org-1") {
        let context = context_for(&pool, user);
        let response = run(&schema, &context, CLAIM).await;
        if allowed {
            let claims = data(&response, "claimMetricRollupDeltas")
                .as_array()
                .unwrap_or_else(|| panic!("{label}: expected a claim list: {response}"))
                .clone();
            assert_eq!(claims.len(), 2, "{label}: {response}");
            // Release the frontier again so the next authorized row of the
            // matrix is tested against the same starting state.
            let token: Uuid = claims[0]["claimToken"]
                .as_str()
                .expect("claimToken")
                .parse()
                .expect("a UUID claim token");
            let mut connection = pool.get().expect("Failed to get DB connection");
            diesel::sql_query(format!(
                "UPDATE metric_rollup_delta SET status = 'PENDING', claim_token = NULL, \
                     claimed_by = NULL, claimed_at = NULL, lease_expires_at = NULL \
                 WHERE claim_token = '{token}'"
            ))
            .execute(&mut connection)
            .expect("release the fixture claim");
        } else {
            assert_unauthorized(&response);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_completion() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();

    // One live claim, held by the authorized principal. Every denied row is
    // therefore denied on authority alone, not because it had nothing to
    // complete.
    let ingest = context_for(
        &pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );
    let claims = data(
        &run(&schema, &ingest, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    let token: Uuid = claims[0]["claimToken"]
        .as_str()
        .expect("claimToken")
        .parse()
        .expect("a UUID claim token");
    let document = complete(token);

    for (label, user, allowed) in matrix("org-1") {
        if allowed {
            continue;
        }
        let context = context_for(&pool, user);
        let response = run(&schema, &context, &document).await;
        assert_unauthorized(&response);
        assert!(
            projection(&pool).is_empty(),
            "{label}: a denied completion must apply nothing"
        );
        assert_eq!(
            state(&pool).applied_through_sequence,
            0,
            "{label}: a denied completion must not move the watermark"
        );
        assert_eq!(
            deltas(&pool)
                .iter()
                .map(|row| row.status.as_str())
                .collect::<Vec<_>>(),
            vec!["CLAIMED", "CLAIMED"],
            "{label}: a denied completion must not change delta state"
        );
    }

    // And the authorized principal still applies the same batch afterwards.
    let watermark = data(
        &run(&schema, &ingest, &document).await,
        "completeMetricRollupDeltas",
    )
    .clone();
    assert_eq!(watermark["appliedThroughSequence"], json!("2"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authorization_precedes_every_rollup_read_lock_and_write() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();

    // A denied claim is denied before the frontier is even read, so a request
    // that would otherwise have been a perfectly valid claim leaves the
    // deltas PENDING, the counter untouched and nothing to roll back.
    let before = state(&pool);
    for user in [
        None,
        Some(user_with("root", &[(Role::Superuser, "org-1")])),
        Some(user_with("reader", &[(Role::MetricsReadService, "org-1")])),
    ] {
        let context = context_for(&pool, user);
        assert_unauthorized(&run(&schema, &context, CLAIM).await);
    }
    let after = state(&pool);
    assert_eq!(after.next_sequence, before.next_sequence);
    assert_eq!(
        after.applied_through_sequence,
        before.applied_through_sequence
    );
    assert_eq!(after.updated_at, before.updated_at);
    assert!(deltas(&pool)
        .iter()
        .all(|row| row.status == "PENDING" && row.claim_token.is_none()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_ingest_role_confers_no_other_authority() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context = context_for(
        &pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    // Holding the rollup authority is not holding registry administration
    // authority, and it supplies no publisher scope: `ADR-0008` gives a
    // machine role exactly its own operations and nothing adjacent.
    let response = run(
        &schema,
        &context,
        "mutation { createMetricPlatform(data: { code: \"x\", displayName: \"X\", \
           ownershipClass: EXTERNAL, enabled: true }) { platformId } }",
    )
    .await;
    assert_unauthorized(&response);
    let response = run(
        &schema,
        &context,
        "{ metricPlatformByCode(code: \"x\") { platformId } }",
    )
    .await;
    assert_unauthorized(&response);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_verification() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let (logged, log) = logging_pool();
    // Warm the captured connection so a checkout issues nothing later.
    let _ = run(&schema, &ingest_context(&logged), &verify_document()).await;

    for (label, user, allowed) in matrix("org-1") {
        log.lock().expect("statement log").clear();
        let context = context_for(&logged, user);
        let response = run(&schema, &context, &verify_document()).await;
        if allowed {
            let verification = data(&response, "verifyMetricRollupMonths").clone();
            assert_eq!(verification["matches"], json!(true), "{label}: {response}");
            assert_eq!(verification["appliedThroughSequence"], json!("0"));
            assert_eq!(verification["nextSequence"], json!("3"), "{label}");
            assert!(
                rollup_statements(&log)
                    .iter()
                    .any(|s| s.contains("metric_rollup_work_day")),
                "{label}: the authorized verification reads rollup state"
            );
        } else {
            assert_unauthorized(&response);
            assert_eq!(
                rollup_statements(&log),
                Vec::<String>::new(),
                "{label}: a denied verification must open no transaction and read no rollup state"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_rebuild() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let (logged, log) = logging_pool();
    let _ = run(&schema, &ingest_context(&logged), &verify_document()).await;
    let before = state(&pool);

    for (label, user, allowed) in matrix("org-1") {
        log.lock().expect("statement log").clear();
        let context = context_for(&logged, user);
        let response = run(&schema, &context, &rebuild_document("0")).await;
        if allowed {
            let result = data(&response, "rebuildMetricRollupMonths").clone();
            // Nothing is applied yet, so the empty monthly state is exact
            // for the empty work-day projection: a healthy no-op.
            assert_eq!(result["rebuilt"], json!(false), "{label}: {response}");
            assert_eq!(result["verification"]["matches"], json!(true));
            assert!(rollup_statements(&log)
                .iter()
                .any(|s| s.contains("metric_rollup_work_day_state") && s.ends_with("FOR UPDATE")));
        } else {
            assert_unauthorized(&response);
            assert_eq!(
                rollup_statements(&log),
                Vec::<String>::new(),
                "{label}: a denied rebuild must open no transaction and touch no rollup state"
            );
        }
        assert_eq!(
            state(&pool),
            before,
            "{label}: the frontier row is untouched"
        );
        assert!(month_rows(&pool).is_empty(), "{label}");
        assert_eq!(deltas(&pool).len(), 2, "{label}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_invalid_expected_frontier_is_rejected_by_the_resolver_without_any_rollup_statement() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let (logged, log) = logging_pool();
    let context = ingest_context(&logged);
    let _ = run(&schema, &context, &verify_document()).await;

    for invalid in ["", "abc", "-1", "+1", "1.0", "9223372036854775808"] {
        log.lock().expect("statement log").clear();
        let response = run(&schema, &context, &rebuild_document(invalid)).await;
        let (message, kind) = only_error(&response);
        assert_eq!(kind, "INTERNAL_ERROR");
        assert_eq!(
            message,
            "The expected applied-through sequence must be a decimal string holding a \
             non-negative 64-bit integer. Nothing was rebuilt.",
            "{invalid:?}"
        );
        assert_eq!(
            log.lock().expect("statement log").clone(),
            Vec::<String>::new(),
            "{invalid:?}: an invalid frontier reaches no statement"
        );
    }
}

// ==========================================================================
// The operations as seen through GraphQL
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_claim_and_completion_round_trip_through_the_resolvers() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let context = context_for(
        &pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    let claims = data(
        &run(&schema, &context, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    let claims = claims.as_array().expect("a claim list");
    assert_eq!(claims.len(), 2);
    // Durable positions cross the API as decimal strings, never as the 32-bit
    // GraphQL `Int` that would eventually truncate them.
    assert_eq!(claims[0]["sequence"], json!("1"));
    assert_eq!(claims[1]["sequence"], json!("2"));
    // One token and one lease identify the whole batch.
    assert_eq!(claims[0]["claimToken"], claims[1]["claimToken"]);
    assert_eq!(claims[0]["leaseExpiresAt"], claims[1]["leaseExpiresAt"]);

    let token: Uuid = claims[0]["claimToken"]
        .as_str()
        .expect("claimToken")
        .parse()
        .expect("a UUID claim token");
    let watermark = data(
        &run(&schema, &context, &complete(token)).await,
        "completeMetricRollupDeltas",
    )
    .clone();
    assert_eq!(watermark["appliedThroughSequence"], json!("2"));
    assert!(watermark["watermarkAt"].is_string());

    let rows = projection(&pool);
    assert_eq!(
        rows.iter()
            .map(|row| (row.day, row.value))
            .collect::<Vec<_>>(),
        vec![(day(DAY_ONE), 10), (day(DAY_TWO), 20)]
    );

    // An exhausted frontier is an empty list, not an error: there is nothing
    // for a worker to retry or alert on.
    let empty = data(
        &run(&schema, &context, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    assert_eq!(empty, json!([]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_completion_resolver_maintains_the_monthly_projections() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let context = context_for(
        &pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    let claims = data(
        &run(&schema, &context, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    let token: Uuid = claims[0]["claimToken"]
        .as_str()
        .expect("claimToken")
        .parse()
        .expect("a UUID claim token");
    let watermark = data(
        &run(&schema, &context, &complete(token)).await,
        "completeMetricRollupDeltas",
    )
    .clone();
    assert_eq!(watermark["appliedThroughSequence"], json!("2"));

    // The same completion that applied the two March day rows resolved and
    // summed them into one monthly total, beneath the same lock and in the
    // same transaction, with no new operation and nothing returned about it.
    let months = month_rows(&pool);
    assert_eq!(months.len(), 1, "one resolved monthly total: {months:?}");
    assert_eq!(months[0].month_start, day((2026, 3, 1)));
    assert_eq!(months[0].publication_id, None);
    assert_eq!(months[0].value, 30);
    assert!(!months[0].requires_country_coverage);
    assert!(!months[0].requires_institution_coverage);
    assert_eq!(months[0].watermark, 2);
    assert_eq!(projection(&pool).len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_verification_and_rebuild_round_trip_through_the_resolvers() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let context = ingest_context(&pool);

    let claims = data(
        &run(&schema, &context, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    let token: Uuid = claims[0]["claimToken"]
        .as_str()
        .expect("claimToken")
        .parse()
        .expect("a UUID claim token");
    run(&schema, &context, &complete(token)).await;
    assert_eq!(month_rows(&pool).len(), 1);

    // Healthy: every 64-bit fact is a decimal string, every family is
    // exact, and the frontier facts are the completion's.
    let verification = data(
        &run(&schema, &context, &verify_document()).await,
        "verifyMetricRollupMonths",
    )
    .clone();
    let exact_one = json!({
        "expectedRows": "1", "actualRows": "1",
        "missingRows": "0", "extraRows": "0", "mismatchedRows": "0"
    });
    let exact_none = json!({
        "expectedRows": "0", "actualRows": "0",
        "missingRows": "0", "extraRows": "0", "mismatchedRows": "0"
    });
    assert_eq!(verification["appliedThroughSequence"], json!("2"));
    assert_eq!(verification["nextSequence"], json!("3"));
    assert!(verification["watermarkAt"].is_string());
    assert_eq!(verification["maxWorkDayWatermark"], json!("2"));
    assert_eq!(verification["workDayRowCount"], json!("2"));
    assert_eq!(verification["representedMonthKeyCount"], json!("1"));
    assert_eq!(verification["total"], exact_one);
    assert_eq!(verification["country"], exact_none);
    assert_eq!(verification["institution"], exact_none);
    assert_eq!(verification["ambiguity"], exact_none);
    assert_eq!(verification["countryYear"], exact_none);
    assert_eq!(verification["institutionYear"], exact_none);
    assert_eq!(verification["matches"], json!(true));

    // A healthy rebuild at the pinned frontier is a no-op with the same
    // verification, and the surrogate id proves no row was rewritten.
    let surrogate = month_rows(&pool)[0].rollup_work_month_id;
    let result = data(
        &run(&schema, &context, &rebuild_document("2")).await,
        "rebuildMetricRollupMonths",
    )
    .clone();
    assert_eq!(result["rebuilt"], json!(false));
    assert_eq!(result["verification"], verification);
    assert_eq!(month_rows(&pool)[0].rollup_work_month_id, surrogate);

    // Empty monthly state: verification reports the missing row without
    // its contents, and the rebuild at the pinned frontier repopulates it
    // and returns an exact verification.
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::sql_query("TRUNCATE TABLE metric_rollup_work_month")
        .execute(&mut connection)
        .expect("truncate");
    drop(connection);
    let verification = data(
        &run(&schema, &context, &verify_document()).await,
        "verifyMetricRollupMonths",
    )
    .clone();
    assert_eq!(
        verification["total"],
        json!({
            "expectedRows": "1", "actualRows": "0",
            "missingRows": "1", "extraRows": "0", "mismatchedRows": "0"
        })
    );
    assert_eq!(verification["matches"], json!(false));
    assert!(month_rows(&pool).is_empty(), "verification wrote nothing");

    let result = data(
        &run(&schema, &context, &rebuild_document("2")).await,
        "rebuildMetricRollupMonths",
    )
    .clone();
    assert_eq!(result["rebuilt"], json!(true));
    assert_eq!(result["verification"]["matches"], json!(true));
    assert_eq!(result["verification"]["total"], exact_one);
    assert_eq!(result["verification"]["appliedThroughSequence"], json!("2"));
    let rows = month_rows(&pool);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, 30);
    assert_ne!(rows[0].rollup_work_month_id, surrogate);
    assert_eq!(
        state(&pool).applied_through_sequence,
        2,
        "the frontier never moves"
    );
    assert_eq!(projection(&pool).len(), 2);
    assert!(deltas(&pool).iter().all(|row| row.status == "APPLIED"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_bounded_rejections_reach_the_caller_without_leaking_state() {
    let (_guard, pool) = setup_registry_db();
    seeded(&pool);
    let schema = create_schema();
    let context = context_for(
        &pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    let response = run(
        &schema,
        &context,
        "mutation { claimMetricRollupDeltas(limit: 51) { deltaId } }",
    )
    .await;
    let (message, _) = only_error(&response);
    assert_eq!(
        message,
        "A rollup delta claim limit must be between 1 and 50 inclusive."
    );

    let unknown = Uuid::new_v4();
    let response = run(&schema, &context, &complete(unknown)).await;
    let (message, _) = only_error(&response);
    assert!(
        message.starts_with("This rollup claim token names no claimed delta."),
        "unexpected message: {message}"
    );
    // The rejection says what the caller may act on and nothing else: no
    // delta id, canonical value, principal, frontier position or database
    // detail crosses the boundary.
    for leaked in [
        "metric_rollup",
        "sequence",
        "SELECT",
        "postgres",
        INGEST_USER,
    ] {
        assert!(
            !message.contains(leaked),
            "the rejection leaked `{leaked}`: {message}"
        );
    }
    assert!(
        deltas(&pool).iter().all(|row| row.status == "PENDING"),
        "a rejected request must change no durable state"
    );

    // A stale pinned frontier is rejected with the fixed message and, like
    // every rollup rejection, discloses no frontier value, row, SQL,
    // principal or database detail.
    let response = run(&schema, &context, &rebuild_document("7")).await;
    let (message, kind) = only_error(&response);
    assert_eq!(kind, "INTERNAL_ERROR");
    assert_eq!(
        message,
        "The expected applied-through sequence does not match the current durable rollup \
         frontier. Nothing was rebuilt; verify again and pin the current frontier."
    );
    for leaked in ["metric_rollup", "SELECT", "postgres", INGEST_USER, "0", "7"] {
        assert!(
            !message.contains(leaked),
            "the rejection leaked `{leaked}`: {message}"
        );
    }
    assert!(month_rows(&pool).is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 0);
}

// ==========================================================================
// Generated schema: strictly additive, and no unauthorized surface
// ==========================================================================

/// One SDL block with descriptions removed and whitespace collapsed, so
/// names, argument types, return types and nullability are compared exactly
/// while prose is not.
fn signatures(block: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in block.chars() {
        match character {
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            _ if in_string => {}
            _ if character.is_whitespace() => {}
            _ => out.push(character),
        }
    }
    out
}

#[test]
fn the_sdl_declares_exactly_the_four_approved_operations() {
    let sdl = create_schema().as_sdl();
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    let mutation_signatures = signatures(mutation_root);

    for operation in [
        "claimMetricRollupDeltas(limit:Int!):[MetricRollupDeltaClaim!]!",
        "completeMetricRollupDeltas(input:CompleteMetricRollupDeltasInput!):MetricRollupWatermark!",
        "verifyMetricRollupMonths:MetricRollupMonthVerification!",
        "rebuildMetricRollupMonths(input:RebuildMetricRollupMonthsInput!):MetricRollupMonthRebuildResult!",
    ] {
        assert_eq!(
            mutation_signatures.matches(operation).count(),
            1,
            "the approved operation `{operation}` must be declared exactly once in \
             MutationRoot: {mutation_signatures}"
        );
    }

    // Exactly four — the two `MET-WP4-01` application operations and the two
    // `MET-WP4-03A-OPS-02` maintenance operations — and no fifth rollup
    // operation by analogy. A retry, a release, an unclaim, a repair, a
    // reconciliation ledger write or a scheduled rebuild each need their own
    // authorization matrix and rollback semantics.
    let rollup_fields: Vec<&str> = mutation_root
        .lines()
        .map(str::trim_start)
        .filter(|line| {
            line.starts_with("claimMetricRollup")
                || line.starts_with("completeMetricRollup")
                || line.starts_with("verifyMetricRollup")
                || line.starts_with("rebuildMetricRollup")
                || line.contains("Rollup(")
                || line.contains("Rollup:")
                || line.starts_with("rebuildMetric")
                || line.starts_with("verifyMetric")
                || line.starts_with("reconcileMetricRollup")
                || line.starts_with("repairMetric")
        })
        .collect();
    assert_eq!(rollup_fields.len(), 4, "MutationRoot: {rollup_fields:?}");
    // And none of them reaches QueryRoot.
    let query_root = sdl_block(&sdl, "type QueryRoot {");
    for absent in [
        "verifyMetricRollup",
        "rebuildMetricRollup",
        "metricRollup",
        "MetricRollupMonth",
        "Rollup",
    ] {
        assert!(
            !query_root.contains(absent),
            "no rollup maintenance or read surface may reach QueryRoot: `{absent}`"
        );
    }
}

#[test]
fn the_verification_and_rebuild_types_are_bounded_and_exactly_the_approved_shape() {
    let sdl = create_schema().as_sdl();

    let family = sdl_block(&sdl, "type MetricRollupMonthProjectionVerification {");
    assert_eq!(
        signatures(family),
        "expectedRows:String!actualRows:String!missingRows:String!extraRows:String!\
         mismatchedRows:String!",
        "the per-family verification carries exactly five bounded counts"
    );

    let verification = sdl_block(&sdl, "type MetricRollupMonthVerification {");
    assert_eq!(
        signatures(verification),
        "appliedThroughSequence:String!nextSequence:String!watermarkAt:Timestamp!\
         maxWorkDayWatermark:StringworkDayRowCount:String!representedMonthKeyCount:String!\
         total:MetricRollupMonthProjectionVerification!\
         country:MetricRollupMonthProjectionVerification!\
         institution:MetricRollupMonthProjectionVerification!\
         ambiguity:MetricRollupMonthProjectionVerification!\
         countryYear:MetricRollupMonthProjectionVerification!\
         institutionYear:MetricRollupMonthProjectionVerification!matches:Boolean!",
        "the verification carries exactly the approved fields (the four monthly \
         families, the two additive yearly families of MET-WP4-03C-A and matches), \
         with only maxWorkDayWatermark nullable"
    );

    let result = sdl_block(&sdl, "type MetricRollupMonthRebuildResult {");
    assert_eq!(
        signatures(result),
        "rebuilt:Boolean!verification:MetricRollupMonthVerification!"
    );

    // The rebuild takes the pinned frontier and nothing else: no key, id,
    // value, dimension, flag, watermark, SQL or repair instruction.
    let input = sdl_block(&sdl, "input RebuildMetricRollupMonthsInput {");
    assert_eq!(signatures(input), "expectedAppliedThroughSequence:String!");

    // No unbounded or row-level surface: nothing in the three result types
    // names a row, an identity, a value, a list or a mismatch detail.
    for block in [family, verification, result] {
        let block_signatures = signatures(block);
        for forbidden in ["[", "Uuid", "value", "rowId", "mismatches", "rows:", "Int!"] {
            assert!(
                !block_signatures.contains(forbidden),
                "a bounded verification type must not expose `{forbidden}`: {block_signatures}"
            );
        }
    }
    // Exactly these four new declarations, each once, and no sibling type
    // by analogy.
    for declaration in [
        "type MetricRollupMonthProjectionVerification {",
        "type MetricRollupMonthVerification {",
        "type MetricRollupMonthRebuildResult {",
        "input RebuildMetricRollupMonthsInput {",
    ] {
        assert_eq!(sdl.matches(declaration).count(), 1, "{declaration}");
    }
    let mut month_declarations: Vec<&str> = sdl
        .lines()
        .filter_map(|line| {
            line.strip_prefix("type ")
                .or_else(|| line.strip_prefix("input "))
                .and_then(|rest| rest.strip_suffix(" {"))
        })
        .filter(|name| name.contains("MetricRollupMonth"))
        .collect();
    month_declarations.sort_unstable();
    assert_eq!(
        month_declarations,
        [
            "MetricRollupMonthProjectionVerification",
            "MetricRollupMonthRebuildResult",
            "MetricRollupMonthVerification",
            "RebuildMetricRollupMonthsInput",
        ],
        "no other MetricRollupMonth* type may exist"
    );
    for absent in [
        "input VerifyMetricRollupMonthsInput",
        "type MetricRollupMonthMismatch",
        "type MetricRollupMonthRow",
        "reconcileMetricRollupMonths",
        "recordMetricReconciliation",
    ] {
        assert!(!sdl.contains(absent), "`{absent}` must not be declared");
    }
}

#[test]
fn the_added_types_expose_progress_state_and_no_read_surface() {
    let sdl = create_schema().as_sdl();

    let claim = sdl_block(&sdl, "type MetricRollupDeltaClaim {");
    let claim_fields = signatures(claim);
    assert_eq!(
        claim_fields, "deltaId:Uuid!sequence:String!claimToken:Uuid!leaseExpiresAt:Timestamp!",
        "MetricRollupDeltaClaim must expose exactly the four approved fields"
    );

    let watermark = sdl_block(&sdl, "type MetricRollupWatermark {");
    assert_eq!(
        signatures(watermark),
        "appliedThroughSequence:String!watermarkAt:Timestamp!",
        "MetricRollupWatermark must expose exactly the two approved fields"
    );

    // Completion takes the batch token and nothing else. A second input field
    // would be a value, dimension or watermark the caller could choose, and
    // the whole point of the contract is that it cannot.
    let input = sdl_block(&sdl, "input CompleteMetricRollupDeltasInput {");
    assert_eq!(
        signatures(input),
        "claimToken:Uuid!",
        "CompleteMetricRollupDeltasInput must carry only the batch token"
    );

    // No rollup read surface: the coverage-aware Metrics read contract
    // (`metricDashboard`) is owned and guarded by `MET-WP4-02`, and nothing
    // here may let a caller read projected rows, delta state or the watermark
    // directly.
    let query = sdl_block(&sdl, "type QueryRoot {");
    for absent in ["metricRollup", "rollupWatermark", "workDayRollup"] {
        assert!(
            !query.contains(absent),
            "no rollup read surface may be added: `{absent}`"
        );
    }
    for absent in [
        "type MetricRollupDelta {",
        "type MetricRollupWorkDay ",
        "type MetricRollupWorkDayState",
        "input ClaimMetricRollupDeltasInput",
    ] {
        assert!(
            !sdl.contains(absent),
            "`{absent}` must not reach the public schema in MET-WP4-01"
        );
    }
    // MET-WP4-03A introduces derived monthly state and no reader of it: no
    // monthly row type, selector or ambiguity surface may appear beyond the
    // separately specified MET-WP4-03B read contract and the bounded
    // MET-WP4-03A-OPS-02 verification/rebuild counts proven below.
    for absent in [
        "MetricRollupWorkMonth",
        "MetricRollupWorkCountryMonth",
        "MetricRollupWorkInstitutionMonth",
        "MetricRollupWorkMonthAmbiguity",
        "monthAmbiguity",
        "rollupWorkMonth",
    ] {
        assert!(
            !sdl.contains(absent),
            "`{absent}` must not reach the public schema in MET-WP4-03A"
        );
    }

    // Durable 64-bit progress positions are strings. GraphQL's `Int` is
    // 32-bit, so exposing a position as one would eventually truncate an
    // ordering identity that must never be approximated.
    for forbidden in [
        "sequence: Int",
        "appliedThroughSequence: Int",
        "watermark: Int",
        "nextSequence: Int",
        "maxWorkDayWatermark: Int",
        "workDayRowCount: Int",
        "representedMonthKeyCount: Int",
        "Rows: Int",
    ] {
        assert!(
            !sdl.contains(forbidden),
            "a durable progress position must not be a 32-bit Int: `{forbidden}`"
        );
    }
}

// ==========================================================================
// MET-WP4-03C-A: six-family descriptions and the yearly families through the
// resolvers
// ==========================================================================

/// The description text of one SDL block, whitespace-collapsed: the prose
/// the generated schema publishes about a type or field.
fn descriptions(block: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in block.chars() {
        match character {
            _ if escaped => {
                escaped = false;
                out.push(character);
            }
            '\\' if in_string => escaped = true,
            '"' => {
                in_string = !in_string;
                out.push(' ');
            }
            _ if in_string => out.push(character),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_maintenance_descriptions_state_the_six_family_contract() {
    let sdl = create_schema().as_sdl();

    // The two maintenance operations keep their names, arguments and return
    // types (asserted above) and now describe six families, not four.
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    let verify_index = mutation_root
        .find("verifyMetricRollupMonths")
        .expect("verifyMetricRollupMonths");
    let rebuild_index = mutation_root
        .find("rebuildMetricRollupMonths")
        .expect("rebuildMetricRollupMonths");
    let verify_prose = descriptions(&mutation_root[..verify_index]);
    let verify_prose = verify_prose
        .rsplit_once("Verify the six")
        .map(|(_, tail)| format!("Verify the six{tail}"))
        .expect("the verification description");
    for phrase in [
        "six derived rollup projection families",
        "monthly total",
        "monthly country",
        "monthly institution",
        "monthly ambiguity",
        "yearly country",
        "yearly institution",
        "metric_rollup_work_day",
        "REPEATABLE READ",
        "METRICS_INGEST_SERVICE",
        "Writes nothing",
        "not projection corruption",
        "never triggers a rebuild",
    ] {
        assert!(
            verify_prose.contains(phrase),
            "verifyMetricRollupMonths must state `{phrase}`: {verify_prose}"
        );
    }
    let rebuild_prose = descriptions(&mutation_root[verify_index..rebuild_index]);
    for phrase in [
        "all six derived rollup projection families",
        "caller-pinned durable frontier",
        "more than 100000 represented month keys is rejected before any projection row is touched",
        "returns rebuilt: false without projection writes",
        "exactly the four monthly and the two yearly tables are truncated",
        "chunks of at most 50 month keys",
        "yearly country and institution tables are then derived set-wise from the rebuilt monthly state",
        "all six families are independently verified again before commit",
        "roll the whole rebuild back",
        "120 seconds",
        "The frontier, the work-day projection, rollup deltas and canonical records are never modified",
        "not a scheduled operation",
    ] {
        assert!(
            rebuild_prose.contains(phrase),
            "rebuildMetricRollupMonths must state `{phrase}`: {rebuild_prose}"
        );
    }
    for stale in [
        "four derived monthly projections",
        "the four tables are truncated",
    ] {
        assert!(
            !descriptions(mutation_root).contains(stale),
            "the retired four-family wording `{stale}` must not survive"
        );
    }

    // The shared per-family type is described generically, the verification
    // object names the six families with the monthly and yearly ones told
    // apart, `matches` covers all six, and the rebuild receipt no longer
    // speaks of four monthly datasets.
    let family = descriptions(sdl_block(
        &sdl,
        "type MetricRollupMonthProjectionVerification {",
    ));
    let family_type_prose = descriptions(
        sdl.split("type MetricRollupMonthProjectionVerification {")
            .next()
            .expect("prefix")
            .rsplit("type ")
            .next()
            .unwrap_or(""),
    );
    assert!(
        family_type_prose.contains("one derived rollup projection family, monthly or yearly"),
        "{family_type_prose}"
    );
    assert!(!family_type_prose.contains("one monthly projection family"));
    assert!(
        !family.contains("monthly"),
        "per-family counts stay generic: {family}"
    );

    let verification_block = sdl_block(&sdl, "type MetricRollupMonthVerification {");
    let verification_type_prose = descriptions(
        sdl.split("type MetricRollupMonthVerification {")
            .next()
            .expect("prefix")
            .rsplit("type ")
            .next()
            .unwrap_or(""),
    );
    assert!(
        verification_type_prose.contains("six derived rollup projection families"),
        "{verification_type_prose}"
    );
    let verification_prose = descriptions(verification_block);
    for phrase in [
        "monthly total projection (monthly family)",
        "monthly per-country projection (monthly family)",
        "monthly per-institution projection (monthly family)",
        "monthly ambiguity state (monthly family)",
        "yearly per-country projection (yearly family)",
        "yearly per-institution projection (yearly family)",
        "True only when all six projection families have zero missing, extra and mismatched rows",
        "no derived monthly or yearly row is watermarked above the frontier",
    ] {
        assert!(
            verification_prose.contains(phrase),
            "MetricRollupMonthVerification must state `{phrase}`: {verification_prose}"
        );
    }

    let result_type_prose = descriptions(
        sdl.split("type MetricRollupMonthRebuildResult {")
            .next()
            .expect("prefix")
            .rsplit("type ")
            .next()
            .unwrap_or(""),
    );
    let result_prose = descriptions(sdl_block(&sdl, "type MetricRollupMonthRebuildResult {"));
    assert!(
        result_type_prose.contains("six-family verification"),
        "{result_type_prose}"
    );
    for phrase in [
        "all six derived projection families were already exact",
        "no projection replacement occurred",
        "four monthly and two yearly derived projection tables were replaced",
        "six-family verification was exact before commit",
    ] {
        assert!(
            result_prose.contains(phrase),
            "MetricRollupMonthRebuildResult must state `{phrase}`: {result_prose}"
        );
    }
    assert!(!result_prose.contains("four monthly datasets"));
    assert!(!result_type_prose.contains("one monthly rebuild request"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_yearly_only_corruption_is_diagnosed_and_repaired_through_the_resolvers() {
    let (_guard, pool) = setup_registry_db();
    // One publisher fixture whose two March day rows carry a country, so the
    // completion derives a monthly country row and a yearly country row.
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for (hash, on, value) in [
        ("identity-day-1", DAY_ONE, 10),
        ("identity-day-2", DAY_TWO, 20),
    ] {
        commit_work_day_delta(
            &pool,
            &fixture,
            hash,
            on,
            DayDimensions {
                country_code: Some("GB"),
                ..DayDimensions::default()
            },
            value,
        );
    }
    let schema = create_schema();
    let context = ingest_context(&pool);
    let claims = data(
        &run(&schema, &context, CLAIM).await,
        "claimMetricRollupDeltas",
    )
    .clone();
    let token: Uuid = claims[0]["claimToken"]
        .as_str()
        .expect("claimToken")
        .parse()
        .expect("a UUID claim token");
    run(&schema, &context, &complete(token)).await;

    let exact_one = json!({
        "expectedRows": "1", "actualRows": "1",
        "missingRows": "0", "extraRows": "0", "mismatchedRows": "0"
    });
    let exact_none = json!({
        "expectedRows": "0", "actualRows": "0",
        "missingRows": "0", "extraRows": "0", "mismatchedRows": "0"
    });
    let verification = data(
        &run(&schema, &context, &verify_document()).await,
        "verifyMetricRollupMonths",
    )
    .clone();
    assert_eq!(verification["total"], exact_one);
    assert_eq!(verification["country"], exact_one);
    assert_eq!(verification["institution"], exact_none);
    assert_eq!(verification["ambiguity"], exact_none);
    assert_eq!(verification["countryYear"], exact_one);
    assert_eq!(verification["institutionYear"], exact_none);
    assert_eq!(verification["matches"], json!(true));
    let yearly = country_year_rows(&pool);
    assert_eq!(yearly.len(), 1);
    assert_eq!(yearly[0].year_start, day((2026, 1, 1)));
    assert_eq!(yearly[0].value, 30);
    assert_eq!(yearly[0].watermark, 2);
    let surrogate = yearly[0].rollup_work_country_year_id;

    // Yearly-only corruption: the yearly country row vanishes while every
    // monthly family stays exact. The verification names the yearly family
    // alone, exposes no row content, and the monthly families stay exact.
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::sql_query("DELETE FROM metric_rollup_work_country_year")
        .execute(&mut connection)
        .expect("delete the yearly row");
    drop(connection);
    let verification = data(
        &run(&schema, &context, &verify_document()).await,
        "verifyMetricRollupMonths",
    )
    .clone();
    assert_eq!(verification["total"], exact_one);
    assert_eq!(verification["country"], exact_one);
    assert_eq!(
        verification["countryYear"],
        json!({
            "expectedRows": "1", "actualRows": "0",
            "missingRows": "1", "extraRows": "0", "mismatchedRows": "0"
        })
    );
    assert_eq!(verification["institutionYear"], exact_none);
    assert_eq!(verification["matches"], json!(false));
    assert!(
        country_year_rows(&pool).is_empty(),
        "verification wrote nothing"
    );
    assert_eq!(
        verification.as_object().expect("an object").len(),
        13,
        "bounded facts and six families only: {verification}"
    );

    // The rebuild at the pinned frontier repairs the yearly family and
    // returns an exact six-family verification; the monthly row is
    // replaced too (all six tables are rebuilt), the frontier never moves.
    let month_surrogate = month_rows(&pool)[0].rollup_work_month_id;
    let result = data(
        &run(&schema, &context, &rebuild_document("2")).await,
        "rebuildMetricRollupMonths",
    )
    .clone();
    assert_eq!(result["rebuilt"], json!(true));
    assert_eq!(result["verification"]["matches"], json!(true));
    assert_eq!(result["verification"]["countryYear"], exact_one);
    assert_eq!(result["verification"]["country"], exact_one);
    let yearly = country_year_rows(&pool);
    assert_eq!(yearly.len(), 1);
    assert_eq!(yearly[0].value, 30);
    assert_eq!(yearly[0].watermark, 2);
    assert_ne!(yearly[0].rollup_work_country_year_id, surrogate);
    assert_ne!(month_rows(&pool)[0].rollup_work_month_id, month_surrogate);
    assert_eq!(
        state(&pool).applied_through_sequence,
        2,
        "the frontier never moves"
    );
    assert!(deltas(&pool).iter().all(|row| row.status == "APPLIED"));
}
