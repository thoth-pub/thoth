//! `MET-WP4-03C-B1` protected coverage-run maintenance evidence at the API
//! boundary.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the two approved operations
//! `verifyMetricCoverageRuns` and `rebuildMetricCoverageRuns`, their
//! fail-closed `METRICS_INGEST_SERVICE`-only authorization, the complete
//! negative matrix, the fact that authorization precedes every coverage-run
//! read, snapshot, lock and write, the bounded error contract as seen
//! through GraphQL, a verify / corrupt / rebuild / verify round trip, and
//! the strictly additive SDL: exactly two operations, three result types,
//! `Int!` counts, no input type, no query-side surface and no other H1
//! vocabulary.
//!
//! Producer, trigger, provenance, winner, coalescing, isolation, locking,
//! rebuild, verifier, migration and identity-manifest evidence lives with the
//! durable state itself, in `crate::model::metric_coverage_run::tests`.

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
use crate::model::metric_coverage::MetricCoverageStatus::{Complete, Partial};
use crate::model::metric_coverage_run::tests::{d, setup, Fixture};
use crate::model::metric_rollup_delta::tests::logging_pool;
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
    let response = request(query).execute(schema, context).await;
    serde_json::to_value(response).expect("serialize GraphQL response")
}

fn data<'a>(response: &'a JsonValue, field: &str) -> &'a JsonValue {
    assert!(
        response.get("errors").is_none(),
        "expected no errors: {response}"
    );
    &response["data"][field]
}

/// The one error's message and `extensions.type`.
fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("an errors array");
    assert_eq!(errors.len(), 1, "exactly one error: {response}");
    assert!(response["data"].is_null(), "no partial data: {response}");
    (
        errors[0]["message"].as_str().expect("message").to_string(),
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("extensions.type")
            .to_string(),
    )
}

fn assert_unauthorized(response: &JsonValue) {
    let (message, code) = only_error(response);
    assert_eq!(code, "NO_ACCESS", "{response}");
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

fn ingest_context(pool: &Arc<PgPool>) -> Context {
    context_for(
        pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    )
}

/// Every caller of the approved negative matrix, in matrix order.
///
/// The tuple is `(label, user, may_maintain_coverage_runs)`.
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
                    (Role::Superuser, org),
                ],
            )),
            true,
        ),
    ]
}

// --------------------------------------------------------------------------
// Operation documents
// --------------------------------------------------------------------------

const ACCOUNT_FIELDS: &str = "sourceAccountId expectedRows actualRows missingRows extraRows \
     mismatchedRows structuralViolations exact";

fn verify_document(arguments: &str) -> String {
    format!(
        "mutation {{ verifyMetricCoverageRuns{arguments} {{ accounts {{ {ACCOUNT_FIELDS} }} \
           nextAfterSourceAccountId domainAccountCount domainFingerprint }} }}"
    )
}

fn rebuild_document(source_account_id: Uuid) -> String {
    format!(
        "mutation {{ rebuildMetricCoverageRuns(sourceAccountId: \"{source_account_id}\") \
           {{ sourceAccountId rebuilt verification {{ {ACCOUNT_FIELDS} }} }} }}"
    )
}

/// The captured statements that touch coverage-run or account state at all,
/// or open a transaction, without Diesel's bind suffix.
fn maintenance_statements(log: &std::sync::Mutex<Vec<String>>) -> Vec<String> {
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
            statement.contains("metric_coverage")
                || statement.contains("metric_source_account")
                || statement.contains("metric_import")
                || statement.to_uppercase().starts_with("BEGIN")
                || statement.starts_with("SET LOCAL")
        })
        .collect()
}

/// One terminal import of the fixture account covering 1-10 March.
fn seeded(f: &Fixture) -> Uuid {
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    import
}

fn exact_json(source_account_id: Uuid, expected: i32) -> JsonValue {
    json!({
        "sourceAccountId": source_account_id.to_string(),
        "expectedRows": expected, "actualRows": expected,
        "missingRows": 0, "extraRows": 0, "mismatchedRows": 0,
        "structuralViolations": 0, "exact": true
    })
}

// ==========================================================================
// The complete authorization matrix
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_verification() {
    let (_guard, f) = setup();
    seeded(&f);
    let schema = create_schema();
    let (logging, log) = logging_pool();

    for (label, user, allowed) in matrix("org-1") {
        log.lock().unwrap().clear();
        let context = context_for(&logging, user);
        let response = run(&schema, &context, &verify_document("")).await;
        if allowed {
            let page = data(&response, "verifyMetricCoverageRuns").clone();
            assert_eq!(page["domainAccountCount"], json!(2), "{label}: {response}");
            assert_eq!(page["accounts"].as_array().unwrap().len(), 2, "{label}");
            assert!(
                page["accounts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|a| a["exact"] == json!(true)),
                "{label}: {page}"
            );
            assert_eq!(page["nextAfterSourceAccountId"], JsonValue::Null, "{label}");
            assert!(
                !maintenance_statements(&log).is_empty(),
                "{label}: the authorized call reads"
            );
        } else {
            assert_unauthorized(&response);
            assert!(
                maintenance_statements(&log).is_empty(),
                "{label}: a denied verification must open no transaction and read nothing: {:?}",
                maintenance_statements(&log)
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_rebuild() {
    let (_guard, f) = setup();
    seeded(&f);
    let schema = create_schema();
    let (logging, log) = logging_pool();
    let versions_before = {
        let mut c = f.pool.get().unwrap();
        diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
            "(SELECT string_agg(xmin::text, ',' ORDER BY run_start) FROM metric_coverage_run)",
        ))
        .get_result::<String>(&mut c)
        .unwrap()
    };

    for (label, user, allowed) in matrix("org-1") {
        log.lock().unwrap().clear();
        let context = context_for(&logging, user);
        let response = run(&schema, &context, &rebuild_document(f.account)).await;
        if allowed {
            let result = data(&response, "rebuildMetricCoverageRuns").clone();
            assert_eq!(
                result["sourceAccountId"],
                json!(f.account.to_string()),
                "{label}"
            );
            assert_eq!(
                result["rebuilt"],
                json!(false),
                "{label}: an exact account is not rebuilt"
            );
            assert_eq!(result["verification"], exact_json(f.account, 1), "{label}");
            let statements = maintenance_statements(&log);
            assert!(
                statements.iter().any(|s| s.contains("FOR UPDATE")),
                "{label}: the account row is locked"
            );
            assert!(
                !statements
                    .iter()
                    .any(|s| s.starts_with("DELETE") || s.starts_with("INSERT")),
                "{label}: nothing is written"
            );
        } else {
            assert_unauthorized(&response);
            assert!(
                maintenance_statements(&log).is_empty(),
                "{label}: a denied rebuild must open no transaction, lock nothing and write nothing"
            );
        }
    }
    let versions_after = {
        let mut c = f.pool.get().unwrap();
        diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
            "(SELECT string_agg(xmin::text, ',' ORDER BY run_start) FROM metric_coverage_run)",
        ))
        .get_result::<String>(&mut c)
        .unwrap()
    };
    assert_eq!(
        versions_before, versions_after,
        "no caller of the matrix rewrote a run"
    );
}

// ==========================================================================
// Bounded rejections and the error contract
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_bounded_rejections_reach_the_caller_without_leaking_state() {
    let (_guard, f) = setup();
    seeded(&f);
    let schema = create_schema();
    let (logging, log) = logging_pool();
    let context = ingest_context(&logging);

    // Limits 0 and 11 are rejected before any connection is used; 1 and 10
    // succeed; an omitted or null limit means the default of 10.
    for bad in ["(limit: 0)", "(limit: 11)", "(limit: -5)"] {
        log.lock().unwrap().clear();
        let response = run(&schema, &context, &verify_document(bad)).await;
        let (message, code) = only_error(&response);
        assert_eq!(code, "METRIC_QUERY_LIMIT_EXCEEDED", "{bad}: {response}");
        assert_eq!(
            message,
            "verifyMetricCoverageRuns limit must be between 1 and 10 inclusive."
        );
        assert!(
            maintenance_statements(&log).is_empty(),
            "{bad}: a rejected limit opens no transaction"
        );
    }
    for (good, accounts) in [
        ("(limit: 1)", 1),
        ("(limit: 10)", 2),
        ("(limit: null)", 2),
        ("", 2),
    ] {
        let response = run(&schema, &context, &verify_document(good)).await;
        let page = data(&response, "verifyMetricCoverageRuns");
        assert_eq!(
            page["accounts"].as_array().unwrap().len(),
            accounts,
            "{good}: {response}"
        );
    }
    // A boundary that names no account is accepted as an exclusive boundary.
    let response = run(
        &schema,
        &context,
        &verify_document(&format!(
            "(afterSourceAccountId: \"{}\", limit: 10)",
            Uuid::nil()
        )),
    )
    .await;
    assert_eq!(
        data(&response, "verifyMetricCoverageRuns")["domainAccountCount"],
        json!(2)
    );

    // An unknown source account is the invalid-query classification, with
    // the fixed message and nothing written.
    log.lock().unwrap().clear();
    let response = run(&schema, &context, &rebuild_document(Uuid::new_v4())).await;
    let (message, code) = only_error(&response);
    assert_eq!(code, "METRIC_QUERY_INVALID", "{response}");
    assert_eq!(message, "The metric source account was not found.");
    let statements = maintenance_statements(&log);
    assert!(statements.iter().any(|s| s.contains("FOR UPDATE")));
    assert!(!statements.iter().any(|s| s.starts_with("DELETE")
        || s.starts_with("INSERT")
        || s.contains("WITH accounts AS")));
    // A malformed identifier never reaches the resolver.
    let response = run(
        &schema,
        &context,
        "mutation { rebuildMetricCoverageRuns(sourceAccountId: \"not-a-uuid\") { rebuilt } }",
    )
    .await;
    assert!(response.get("errors").is_some() && response["data"].is_null());

    // No error message carries SQL, a constraint name or a connection detail.
    for response in [
        run(&schema, &context, &verify_document("(limit: 0)")).await,
        run(&schema, &context, &rebuild_document(Uuid::new_v4())).await,
    ] {
        let text = response.to_string().to_lowercase();
        for leaked in [
            "select ",
            "insert ",
            "constraint",
            "postgres",
            "fkey",
            "pkey",
            "metric_coverage_run",
        ] {
            assert!(
                !text.contains(leaked),
                "`{leaked}` must not leak: {response}"
            );
        }
    }
}

// ==========================================================================
// Round trip through the resolvers
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_verification_and_rebuild_round_trip_through_the_resolvers() {
    let (_guard, f) = setup();
    let base = seeded(&f);
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);
    f.complete(partial, 2);
    let schema = create_schema();
    let context = ingest_context(&f.pool);
    let mut ordered = [f.account, f.account_b];
    ordered.sort();

    // Exact: three runs for the account, none for the other, both exact.
    let page = data(
        &run(&schema, &context, &verify_document("")).await,
        "verifyMetricCoverageRuns",
    )
    .clone();
    let expected_accounts: Vec<JsonValue> = ordered
        .iter()
        .map(|id| exact_json(*id, if *id == f.account { 3 } else { 0 }))
        .collect();
    assert_eq!(page["accounts"], json!(expected_accounts));
    assert_eq!(page["nextAfterSourceAccountId"], JsonValue::Null);
    assert_eq!(page["domainAccountCount"], json!(2));
    let fingerprint = page["domainFingerprint"].as_str().unwrap().to_string();
    assert_eq!(fingerprint.len(), 64);
    assert_eq!(
        page.as_object().unwrap().len(),
        4,
        "exactly the four approved page fields: {page}"
    );
    assert_eq!(
        page["accounts"][0].as_object().unwrap().len(),
        8,
        "exactly the eight approved account fields"
    );
    assert!(
        page["accounts"][0]["expectedRows"].is_i64(),
        "counts are GraphQL Int, not strings"
    );

    // A one-account page with a boundary: the second account only.
    let page = data(
        &run(
            &schema,
            &context,
            &verify_document(&format!(
                "(afterSourceAccountId: \"{}\", limit: 1)",
                ordered[0]
            )),
        )
        .await,
        "verifyMetricCoverageRuns",
    )
    .clone();
    assert_eq!(page["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(
        page["accounts"][0]["sourceAccountId"],
        json!(ordered[1].to_string())
    );
    assert_eq!(page["nextAfterSourceAccountId"], JsonValue::Null);
    assert_eq!(page["domainFingerprint"], json!(fingerprint));
    // A one-account page from the start names the second as the next
    // boundary.
    let page = data(
        &run(&schema, &context, &verify_document("(limit: 1)")).await,
        "verifyMetricCoverageRuns",
    )
    .clone();
    assert_eq!(
        page["accounts"][0]["sourceAccountId"],
        json!(ordered[0].to_string())
    );
    assert_eq!(
        page["nextAfterSourceAccountId"],
        json!(ordered[0].to_string())
    );

    // Corrupt: a missing run, an extra run and a mismatched run. The
    // verification names the counts and nothing else; a verification writes
    // nothing.
    f.sql(&format!("DELETE FROM metric_coverage_run WHERE run_start = '2026-03-05' AND source_account_id = '{}'", f.account));
    f.sql(&format!("UPDATE metric_coverage_run SET run_end = '2026-03-04' WHERE run_start = '2026-03-01' AND source_account_id = '{}'", f.account));
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-08-01', '2026-08-03', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_b, f.measure));
    let page = data(
        &run(&schema, &context, &verify_document("")).await,
        "verifyMetricCoverageRuns",
    )
    .clone();
    let account = page["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["sourceAccountId"] == json!(f.account.to_string()))
        .unwrap()
        .clone();
    assert_eq!(
        account,
        json!({
            "sourceAccountId": f.account.to_string(),
            "expectedRows": 3, "actualRows": 3,
            "missingRows": 1, "extraRows": 1, "mismatchedRows": 1,
            "structuralViolations": 0, "exact": false
        })
    );
    assert_eq!(
        page["domainFingerprint"],
        json!(fingerprint),
        "the domain is unchanged by corruption"
    );
    assert!(!f.verify(f.account).exact, "verification repaired nothing");

    // Rebuild: replaced and exact in the same committed transaction.
    let result = data(
        &run(&schema, &context, &rebuild_document(f.account)).await,
        "rebuildMetricCoverageRuns",
    )
    .clone();
    assert_eq!(result["sourceAccountId"], json!(f.account.to_string()));
    assert_eq!(result["rebuilt"], json!(true));
    assert_eq!(result["verification"], exact_json(f.account, 3));
    assert_eq!(
        result.as_object().unwrap().len(),
        3,
        "exactly the three approved result fields"
    );
    assert_eq!(f.runs().len(), 3);
    assert!(f.runs().iter().all(|run| run.publisher_id == f.publisher_a));

    // Verified exact again through the resolver, and a repeated rebuild is a
    // no-op receipt with the same verification.
    let page = data(
        &run(&schema, &context, &verify_document("")).await,
        "verifyMetricCoverageRuns",
    )
    .clone();
    assert_eq!(page["accounts"], json!(expected_accounts));
    let again = data(
        &run(&schema, &context, &rebuild_document(f.account)).await,
        "rebuildMetricCoverageRuns",
    )
    .clone();
    assert_eq!(again["rebuilt"], json!(false));
    assert_eq!(again["verification"], result["verification"]);
    // The raw evidence of the base import is untouched by any of it.
    let mut c = f.pool.get().unwrap();
    let count: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
        "(SELECT COUNT(*) FROM metric_coverage WHERE import_id = '{base}')"
    )))
    .get_result(&mut c)
    .unwrap();
    assert_eq!(count, 1);
}

// ==========================================================================
// Generated schema: strictly additive, exact, and no unauthorized surface
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

/// The description text of one SDL block, whitespace-collapsed.
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
fn the_sdl_declares_exactly_the_two_approved_operations_and_three_result_types() {
    let sdl = create_schema().as_sdl();
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    let mutation_signatures = signatures(mutation_root);

    for operation in [
        "verifyMetricCoverageRuns(afterSourceAccountId:Uuid,limit:Int=10):MetricCoverageRunVerificationPage!",
        "rebuildMetricCoverageRuns(sourceAccountId:Uuid!):MetricCoverageRunRebuildResult!",
    ] {
        assert_eq!(
            mutation_signatures.matches(operation).count(),
            1,
            "`{operation}` must be declared exactly once in MutationRoot: {mutation_signatures}"
        );
    }
    // Exactly two coverage-run operations and no third by analogy.
    let coverage_run_fields: Vec<&str> = mutation_root
        .lines()
        .map(str::trim_start)
        .filter(|line| line.contains("CoverageRun") || line.contains("coverageRun"))
        .collect();
    assert_eq!(coverage_run_fields.len(), 2, "{coverage_run_fields:?}");
    // No query-side or other public surface mentions coverage runs.
    let query_root = sdl_block(&sdl, "type QueryRoot {");
    assert!(!query_root.contains("CoverageRun") && !query_root.contains("coverageRun"));
    let mut declarations: Vec<&str> = sdl
        .lines()
        .filter_map(|line| {
            line.strip_prefix("type ")
                .or_else(|| line.strip_prefix("input "))
                .or_else(|| line.strip_prefix("enum "))
                .or_else(|| line.strip_prefix("scalar "))
                .and_then(|rest| rest.strip_suffix(" {").or(Some(rest)))
        })
        .filter(|name| name.contains("CoverageRun"))
        .collect();
    declarations.sort_unstable();
    assert_eq!(
        declarations,
        [
            "MetricCoverageRunAccountVerification",
            "MetricCoverageRunRebuildResult",
            "MetricCoverageRunVerificationPage",
        ],
        "exactly the three approved result types and no input, enum or scalar"
    );
    for declaration in [
        "type MetricCoverageRunAccountVerification {",
        "type MetricCoverageRunVerificationPage {",
        "type MetricCoverageRunRebuildResult {",
    ] {
        assert_eq!(sdl.matches(declaration).count(), 1, "{declaration}");
    }

    // Exact field shapes and nullability: every count is a non-null Int,
    // only the next boundary is nullable.
    assert_eq!(
        signatures(sdl_block(
            &sdl,
            "type MetricCoverageRunAccountVerification {"
        )),
        "sourceAccountId:Uuid!expectedRows:Int!actualRows:Int!missingRows:Int!extraRows:Int!\
         mismatchedRows:Int!structuralViolations:Int!exact:Boolean!"
    );
    assert_eq!(
        signatures(sdl_block(&sdl, "type MetricCoverageRunVerificationPage {")),
        "accounts:[MetricCoverageRunAccountVerification!]!nextAfterSourceAccountId:Uuid\
         domainAccountCount:Int!domainFingerprint:String!"
    );
    assert_eq!(
        signatures(sdl_block(&sdl, "type MetricCoverageRunRebuildResult {")),
        "sourceAccountId:Uuid!rebuilt:Boolean!verification:MetricCoverageRunAccountVerification!"
    );
    // The result types are reachable only through the two operations.
    let reaching: Vec<&str> = sdl
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains(": MetricCoverageRunVerificationPage")
                || line.contains(": MetricCoverageRunRebuildResult")
        })
        .collect();
    assert_eq!(reaching.len(), 2, "{reaching:?}");
    let reaching_account: Vec<&str> = sdl
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains("MetricCoverageRunAccountVerification") && !line.starts_with("type ")
        })
        .collect();
    assert_eq!(
        reaching_account.len(),
        2,
        "accounts and verification only: {reaching_account:?}"
    );

    // Existing repository-banned vocabulary is neither introduced nor
    // repurposed, and the operation descriptions avoid the words the
    // existing MutationRoot prose guard bans.
    for absent in [
        "MetricReconciliationRun",
        "recordMetricReconciliation",
        "scalar UUID",
        "scalar DateTime",
        "type MetricCoverageRun {",
        "input VerifyMetricCoverageRunsInput",
        "input RebuildMetricCoverageRunsInput",
        "coverageRuns(",
        "metricCoverageRun",
    ] {
        assert!(!sdl.contains(absent), "`{absent}` must not be declared");
    }
    let prose = descriptions(mutation_root).to_lowercase();
    let verify_index = prose
        .find("verify one strictly ascending page")
        .expect("the verify description");
    let rebuild_index = prose
        .find("verify and, only if not already exact, rebuild")
        .expect("the rebuild description");
    for window in [
        &prose[verify_index..rebuild_index],
        &prose[rebuild_index..rebuild_index + 900],
    ] {
        for banned in ["cursor", "manifest"] {
            assert!(
                !window.contains(banned),
                "the coverage-run descriptions avoid `{banned}`: {window}"
            );
        }
        assert!(window.contains("metrics_ingest_service"), "{window}");
    }
    for phrase in [
        "read-only repeatable-read snapshot",
        "exclusive ordering boundary",
        "between 1 and 10 inclusive and is never clamped",
        "domain's count and fingerprint",
        "a mismatch never triggers a rebuild",
    ] {
        assert!(
            prose.contains(phrase),
            "verifyMetricCoverageRuns must state `{phrase}`"
        );
    }
    for phrase in [
        "read-committed transaction beneath the source account row lock",
        "an unknown source account is rejected before anything is read",
        "returns rebuilt: false",
        "independently verified again",
        "roll the whole rebuild back",
        "not a scheduled operation",
    ] {
        assert!(
            prose.contains(phrase),
            "rebuildMetricCoverageRuns must state `{phrase}`"
        );
    }
}
