//! `MET-WP1-13` protected Metrics source and source-account administration
//! evidence.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the six approved operations, their
//! fail-closed SUPERUSER-only authorization, the exact-code contract as seen
//! through GraphQL, the closed typed configuration representation, replacement
//! (not patch) semantics, sanitized failures at the API boundary, and the
//! strictly additive SDL with its negative-scope guarantees.
//!
//! Coordinator, decoder, audit, atomicity, no-op, locking and concurrency
//! evidence lives with each entity in `crate::model::metric_source`,
//! `crate::model::metric_source_account` and
//! `crate::model::metric_source_registry_history`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use diesel::{sql_query, RunQueryDsl};
use serde_json::{json, Value as JsonValue};
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_platform::tests::setup_registry_db;
use crate::model::tests::db as test_db;
use crate::policy::Role;

/// A fictional hostname used as the CloudFront account's `externalKey`. No
/// real provider value appears anywhere in this module.
const HOST: &str = "cdn.example-press.test";
/// The deterministic fictional publisher every fixture account is pinned to
/// (Specification Amendment 2).
const PUBLISHER: &str = "5f5f0000-0000-4000-8000-000000000002";

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

fn count(pool: &PgPool, query: &str) -> i64 {
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(&mut connection)
        .expect("Failed to run scalar query")
}

fn execute(pool: &PgPool, sql: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(sql)
        .execute(&mut connection)
        .expect("fixture SQL must succeed");
}

/// The fixture publisher (Specification Amendment 2 pin), planted through SQL.
fn insert_publisher(pool: &PgPool) {
    execute(
        pool,
        &format!(
            "INSERT INTO publisher (publisher_id, publisher_name) \
             VALUES ('{PUBLISHER}', 'Fixture Press')"
        ),
    );
}

/// One publisher, one CloudFront driver source, one admin-import source, one
/// platform and one pinned CloudFront account, all planted through SQL so the
/// authorization tests have something to be denied against.
fn fixture(pool: &PgPool) {
    insert_publisher(pool);
    execute(
        pool,
        "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled) VALUES \
             ('cf_source', 'DRIVER', 'cloudfront', TRUE), \
             ('admin_source', 'ADMIN_IMPORT', NULL, TRUE)",
    );
    execute(
        pool,
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ('fixture_platform', 'Fixture', 'EXTERNAL', TRUE)",
    );
    execute(
        pool,
        &format!(
            "INSERT INTO metric_source_account \
                 (code, source_id, platform_id, external_key, configuration, \
                  expected_publisher_id, enabled) \
             SELECT 'existing_account', s.source_id, p.platform_id, '{HOST}', \
                    '{{\"schemaVersion\":\"cloudfront-source-account/1\",\"hostname\":\"{HOST}\",\
                      \"logging\":{{\"mode\":\"LEGACY_S3\",\"bucket\":\"b\",\"prefix\":\"p\"}}}}'::jsonb, \
                    '{PUBLISHER}', TRUE \
               FROM metric_source s, metric_platform p \
              WHERE s.code = 'cf_source' AND p.code = 'fixture_platform'"
        ),
    );
}

// --------------------------------------------------------------------------
// The six approved operations, as GraphQL documents
// --------------------------------------------------------------------------

const SOURCE_FIELDS: &str =
    "sourceId code acquisitionType driverKey enabled defaultLookbackDays defaultFinalizationDelayDays";
const ACCOUNT_FIELDS: &str = "sourceAccountId code sourceId platformId externalKey \
    expectedPublisherId enabled configuration { kind cloudfrontLegacyS3 { hostname bucket prefix } }";

fn create_source_mutation(code: &str, acquisition: &str, driver_key: &str) -> String {
    format!(
        "mutation {{ createMetricSource(data: {{ code: \"{code}\", acquisitionType: {acquisition}, \
           {driver_key} enabled: true, defaultLookbackDays: 30, defaultFinalizationDelayDays: 2 }}) \
         {{ {SOURCE_FIELDS} }} }}"
    )
}

fn update_source_mutation(code: &str, enabled: bool, lookback: &str) -> String {
    format!(
        "mutation {{ updateMetricSource(data: {{ code: \"{code}\", enabled: {enabled}, {lookback} \
           defaultFinalizationDelayDays: 2 }}) {{ {SOURCE_FIELDS} }} }}"
    )
}

fn cloudfront_literal(hostname: &str, bucket: &str, prefix: &str) -> String {
    format!(
        "{{ kind: CLOUDFRONT_LEGACY_S3_V1, cloudfrontLegacyS3: {{ hostname: \"{hostname}\", \
           bucket: \"{bucket}\", prefix: \"{prefix}\" }} }}"
    )
}

fn create_account_mutation(
    code: &str,
    source_code: &str,
    external_key: &str,
    configuration: &str,
) -> String {
    create_account_mutation_with(
        code,
        source_code,
        external_key,
        configuration,
        &format!("expectedPublisherId: \"{PUBLISHER}\","),
    )
}

fn create_account_mutation_with(
    code: &str,
    source_code: &str,
    external_key: &str,
    configuration: &str,
    publisher: &str,
) -> String {
    format!(
        "mutation {{ createMetricSourceAccount(data: {{ code: \"{code}\", sourceCode: \"{source_code}\", \
           platformCode: \"fixture_platform\", externalKey: \"{external_key}\", {publisher} \
           configuration: {configuration}, enabled: true }}) {{ {ACCOUNT_FIELDS} }} }}"
    )
}

fn update_account_mutation(code: &str, configuration: &str, enabled: bool) -> String {
    format!(
        "mutation {{ updateMetricSourceAccount(data: {{ code: \"{code}\", \
           configuration: {configuration}, enabled: {enabled} }}) {{ {ACCOUNT_FIELDS} }} }}"
    )
}

fn source_lookup(code: &str) -> String {
    format!("{{ metricSourceByCode(code: \"{code}\") {{ {SOURCE_FIELDS} }} }}")
}

fn account_lookup(code: &str) -> String {
    format!("{{ metricSourceAccountByCode(code: \"{code}\") {{ {ACCOUNT_FIELDS} }} }}")
}

/// Every one of the six approved operations, with the field each returns.
fn all_six_operations() -> Vec<(&'static str, String)> {
    vec![
        (
            "createMetricSource",
            create_source_mutation("new_source", "DRIVER", "driverKey: \"cloudfront\","),
        ),
        (
            "updateMetricSource",
            update_source_mutation("cf_source", false, "defaultLookbackDays: 1,"),
        ),
        (
            "createMetricSourceAccount",
            create_account_mutation(
                "new_account",
                "cf_source",
                HOST,
                &cloudfront_literal(HOST, "b", "p"),
            ),
        ),
        (
            "updateMetricSourceAccount",
            update_account_mutation(
                "existing_account",
                &cloudfront_literal(HOST, "b2", "p"),
                false,
            ),
        ),
        ("metricSourceByCode", source_lookup("cf_source")),
        (
            "metricSourceAccountByCode",
            account_lookup("existing_account"),
        ),
    ]
}

// --------------------------------------------------------------------------
// Authorization
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_operation_is_denied_to_every_caller_that_is_not_a_superuser() {
    let (_guard, pool) = setup_registry_db();
    fixture(&pool);
    let schema = create_schema();
    let publisher = test_db::create_publisher(&pool);
    let org = publisher.zitadel_id.clone().expect("publisher zitadel id");

    let denied_contexts: Vec<(&str, Context)> = vec![
        (
            "anonymous",
            test_db::test_context_anonymous(Arc::clone(&pool)),
        ),
        (
            "authenticated with no role",
            test_db::test_context(Arc::clone(&pool), "no-roles"),
        ),
        (
            "publisher user",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with("pub-user", &[(Role::PublisherUser, &org)]),
            ),
        ),
        (
            "publisher admin",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with("pub-admin", &[(Role::PublisherAdmin, &org)]),
            ),
        ),
        (
            "work lifecycle",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with("lifecycle", &[(Role::WorkLifecycle, &org)]),
            ),
        ),
        (
            "cdn write",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with("cdn", &[(Role::CdnWrite, &org)]),
            ),
        ),
        (
            "dissemination worker machine role",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with("worker", &[(Role::DisseminationWorker, &org)]),
            ),
        ),
        (
            "every non-superuser role at once",
            test_db::test_context_with_user(
                Arc::clone(&pool),
                user_with(
                    "everything-but-super",
                    &[
                        (Role::PublisherAdmin, &org),
                        (Role::PublisherUser, &org),
                        (Role::WorkLifecycle, &org),
                        (Role::CdnWrite, &org),
                        (Role::DisseminationWorker, &org),
                    ],
                ),
            ),
        ),
    ];

    for (label, context) in &denied_contexts {
        for (operation, document) in all_six_operations() {
            let response = run(&schema, context, &document).await;
            assert_unauthorized(&response);
            assert!(
                response["data"].is_null() || response["data"][operation].is_null(),
                "{label} must receive no data for {operation}: {response}"
            );
        }
    }

    // Not one denied call reached the database: the fixture is exactly as
    // planted and the source audit history is empty.
    assert_eq!(count(&pool, "(SELECT COUNT(*) FROM metric_source)"), 2);
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source WHERE code = 'cf_source' AND enabled AND default_lookback_days IS NULL)"),
        1,
        "a denied update must not change a source"
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        1
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source_account WHERE code = 'existing_account' AND enabled AND configuration->'logging'->>'bucket' = 'b')"),
        1,
        "a denied update must not change an account"
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history)"
        ),
        0,
        "a denied call must write no audit row"
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)"),
        0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authorization_is_decided_before_any_database_access() {
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    for (label, context) in [
        (
            "anonymous",
            test_db::test_context_anonymous(Arc::clone(&unreachable)),
        ),
        (
            "authenticated non-superuser",
            test_db::test_context(Arc::clone(&unreachable), "no-roles"),
        ),
    ] {
        for (operation, document) in all_six_operations() {
            let response = run(&schema, &context, &document).await;
            let (message, kind) = only_error(&response);
            assert_eq!(
                (kind.as_str(), message.as_str()),
                ("NO_ACCESS", "Unauthorized"),
                "{label} must be denied before {operation} reaches the database"
            );
        }
    }
}

// --------------------------------------------------------------------------
// Superuser lifecycle through the API
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_superuser_can_drive_the_whole_administrative_lifecycle() {
    let (_guard, pool) = setup_registry_db();
    execute(
        &pool,
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ('fixture_platform', 'Fixture', 'EXTERNAL', TRUE)",
    );
    insert_publisher(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    // Sources: a DRIVER/cloudfront source and an ADMIN_IMPORT source.
    let created = run(
        &schema,
        &context,
        &create_source_mutation("cf_source", "DRIVER", "driverKey: \"cloudfront\","),
    )
    .await;
    let source = data(&created, "createMetricSource");
    assert_eq!(source["code"], "cf_source");
    assert_eq!(source["acquisitionType"], "DRIVER");
    assert_eq!(source["driverKey"], "cloudfront");
    assert_eq!(source["defaultLookbackDays"], 30);
    let admin = run(
        &schema,
        &context,
        &create_source_mutation("admin_source", "ADMIN_IMPORT", ""),
    )
    .await;
    assert_eq!(
        data(&admin, "createMetricSource")["driverKey"],
        JsonValue::Null
    );

    let looked_up = run(&schema, &context, &source_lookup("cf_source")).await;
    assert_eq!(data(&looked_up, "metricSourceByCode"), source);

    let updated = run(
        &schema,
        &context,
        &update_source_mutation("cf_source", false, "defaultLookbackDays: 7,"),
    )
    .await;
    let updated = data(&updated, "updateMetricSource");
    assert_eq!(updated["enabled"], false);
    assert_eq!(updated["defaultLookbackDays"], 7);
    assert_eq!(updated["driverKey"], "cloudfront", "driverKey is immutable");
    assert_eq!(updated["sourceId"], source["sourceId"]);

    // Accounts: the typed configuration round-trips through the closed types.
    let account = run(
        &schema,
        &context,
        &create_account_mutation(
            "cf_account",
            "cf_source",
            HOST,
            &cloudfront_literal(HOST, "bucket-a", "prefix/a/"),
        ),
    )
    .await;
    let account = data(&account, "createMetricSourceAccount");
    assert_eq!(account["code"], "cf_account");
    assert_eq!(account["sourceId"], source["sourceId"]);
    assert_eq!(account["externalKey"], HOST);
    assert_eq!(
        account["expectedPublisherId"], PUBLISHER,
        "the Amendment 2 pin is stored"
    );
    assert_eq!(
        account["configuration"],
        json!({
            "kind": "CLOUDFRONT_LEGACY_S3_V1",
            "cloudfrontLegacyS3": {"hostname": HOST, "bucket": "bucket-a", "prefix": "prefix/a/"},
        })
    );
    // An EMPTY account needs no pin: expectedPublisherId stays nullable.
    let empty_account = run(
        &schema,
        &context,
        &create_account_mutation_with(
            "admin_account",
            "admin_source",
            "partition-1",
            "{ kind: EMPTY }",
            "",
        ),
    )
    .await;
    assert_eq!(
        data(&empty_account, "createMetricSourceAccount")["expectedPublisherId"],
        JsonValue::Null
    );
    assert_eq!(
        data(&empty_account, "createMetricSourceAccount")["configuration"],
        json!({"kind": "EMPTY", "cloudfrontLegacyS3": null})
    );

    let looked_up = run(&schema, &context, &account_lookup("cf_account")).await;
    assert_eq!(data(&looked_up, "metricSourceAccountByCode"), account);

    let updated = run(
        &schema,
        &context,
        &update_account_mutation(
            "cf_account",
            &cloudfront_literal(HOST, "bucket-b", "prefix/b/"),
            false,
        ),
    )
    .await;
    let updated = data(&updated, "updateMetricSourceAccount");
    assert_eq!(updated["enabled"], false);
    assert_eq!(
        updated["configuration"]["cloudfrontLegacyS3"]["bucket"],
        "bucket-b"
    );
    assert_eq!(updated["sourceAccountId"], account["sourceAccountId"]);
    assert_eq!(updated["externalKey"], HOST, "externalKey is immutable");
    assert_eq!(
        updated["expectedPublisherId"], PUBLISHER,
        "expectedPublisherId is immutable"
    );

    // Every committed write was audited in the source audit, none in WP1-12's.
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history WHERE actor = 'super-1')"
        ),
        6,
        "two source creates, one source update, two account creates, one account update"
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source_registry_history WHERE action = 'UPDATE' AND before_state IS NOT NULL)"),
        2
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)"),
        0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn codes_are_matched_exactly_through_the_api() {
    let (_guard, pool) = setup_registry_db();
    fixture(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    for (label, document) in [
        ("source case variant", source_lookup("CF_SOURCE")),
        ("source whitespace variant", source_lookup("cf_source ")),
        ("account case variant", account_lookup("EXISTING_ACCOUNT")),
        (
            "account update whitespace variant",
            update_account_mutation(
                " existing_account",
                &cloudfront_literal(HOST, "b", "p"),
                true,
            ),
        ),
        (
            "account create with source code variant",
            create_account_mutation("x", "Cf_Source", "k", &cloudfront_literal("k", "b", "p")),
        ),
    ] {
        let response = run(&schema, &context, &document).await;
        let (message, _) = only_error(&response);
        assert_eq!(message, "No record was found for the given ID.", "{label}");
    }
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history)"
        ),
        0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_patch_is_a_replacement_so_an_omitted_nullable_field_stores_null() {
    let (_guard, pool) = setup_registry_db();
    fixture(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));
    execute(
        &pool,
        "UPDATE metric_source SET default_lookback_days = 30 WHERE code = 'cf_source'",
    );

    let response = run(
        &schema,
        &context,
        &update_source_mutation("cf_source", true, ""),
    )
    .await;
    let updated = data(&response, "updateMetricSource");
    assert_eq!(updated["defaultLookbackDays"], JsonValue::Null);
    assert_eq!(updated["defaultFinalizationDelayDays"], 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_no_op_update_through_the_api_changes_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    fixture(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    let response = run(
        &schema,
        &context,
        &update_account_mutation(
            "existing_account",
            &cloudfront_literal(HOST, "b", "p"),
            true,
        ),
    )
    .await;
    assert_eq!(
        data(&response, "updateMetricSourceAccount")["configuration"]["cloudfrontLegacyS3"]
            ["bucket"],
        "b"
    );
    // The source fixture holds NULL day defaults; a replacement that sends
    // exactly the stored values (null lookback, finalization 2 once planted)
    // must be a no-op too.
    execute(
        &pool,
        "UPDATE metric_source SET default_finalization_delay_days = 2 WHERE code = 'admin_source'",
    );
    let response = run(
        &schema,
        &context,
        &update_source_mutation("admin_source", true, ""),
    )
    .await;
    assert_eq!(data(&response, "updateMetricSource")["enabled"], true);
    assert_eq!(
        data(&response, "updateMetricSource")["defaultLookbackDays"],
        JsonValue::Null
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history)"
        ),
        0,
        "no-op updates must not be audited"
    );
}

// --------------------------------------------------------------------------
// Sanitized failure boundary
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn validation_and_constraint_failures_reach_the_client_bounded_and_sanitised() {
    let (_guard, pool) = setup_registry_db();
    fixture(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    // An account whose stored configuration predates the typed surface.
    execute(
        &pool,
        "INSERT INTO metric_source_account \
             (code, source_id, platform_id, external_key, configuration, enabled) \
         SELECT 'legacy_account', s.source_id, p.platform_id, 'legacy-key', \
                '{\"legacy\": true, \"secretName\": \"vault/not-real\"}'::jsonb, TRUE \
           FROM metric_source s, metric_platform p \
          WHERE s.code = 'admin_source' AND p.code = 'fixture_platform'",
    );

    let cases = [
        (
            "duplicate source code",
            create_source_mutation("cf_source", "DRIVER", "driverKey: \"cloudfront\","),
            "A metric source with this code already exists.",
        ),
        (
            "blank source code",
            create_source_mutation("   ", "OPERAS", ""),
            "Metric source code must not be an empty string.",
        ),
        (
            "DRIVER without driver key",
            create_source_mutation("bad_driver", "DRIVER", ""),
            "A DRIVER metric source requires a non-blank driver key, and a non-DRIVER metric source must not carry one.",
        ),
        (
            "non-DRIVER with driver key",
            create_source_mutation("bad_upload", "PUBLISHER_UPLOAD", "driverKey: \"x\","),
            "A DRIVER metric source requires a non-blank driver key, and a non-DRIVER metric source must not carry one.",
        ),
        (
            "duplicate account code",
            create_account_mutation("existing_account", "admin_source", "k", "{ kind: EMPTY }"),
            "A metric source account with this code already exists.",
        ),
        (
            "duplicate (source, externalKey)",
            create_account_mutation("another", "cf_source", HOST, &cloudfront_literal(HOST, "b", "p")),
            "A metric source account with this external key already exists for this metric source.",
        ),
        (
            "EMPTY with a payload",
            create_account_mutation(
                "x",
                "admin_source",
                "k",
                "{ kind: EMPTY, cloudfrontLegacyS3: { hostname: \"k\", bucket: \"b\", prefix: \"p\" } }",
            ),
            "An EMPTY metric source account configuration must not carry a CloudFront legacy S3 payload.",
        ),
        (
            "CloudFront without a payload",
            create_account_mutation("x", "cf_source", "k", "{ kind: CLOUDFRONT_LEGACY_S3_V1 }"),
            "A CLOUDFRONT_LEGACY_S3_V1 metric source account configuration requires its CloudFront legacy S3 payload.",
        ),
        (
            "cloudfront source with EMPTY",
            create_account_mutation("x", "cf_source", "k", "{ kind: EMPTY }"),
            "A metric source account of a DRIVER source with the cloudfront driver key must use the CLOUDFRONT_LEGACY_S3_V1 configuration.",
        ),
        (
            "admin source with CloudFront",
            create_account_mutation("x", "admin_source", "k", &cloudfront_literal("k", "b", "p")),
            "Only a metric source account of a DRIVER source with the cloudfront driver key may use the CLOUDFRONT_LEGACY_S3_V1 configuration; every other source uses EMPTY.",
        ),
        (
            "CloudFront without expectedPublisherId (Amendment 2)",
            create_account_mutation_with("x", "cf_source", "k4", &cloudfront_literal("k4", "b", "p"), ""),
            "A CLOUDFRONT_LEGACY_S3_V1 metric source account must be created with an expectedPublisherId naming an existing publisher.",
        ),
        (
            "CloudFront with an unknown expectedPublisherId",
            create_account_mutation_with(
                "x",
                "cf_source",
                "k5",
                &cloudfront_literal("k5", "b", "p"),
                "expectedPublisherId: \"5f5f0000-0000-4000-8000-0000000000ff\",",
            ),
            "The expected publisher of a metric source account must be an existing publisher.",
        ),
        (
            "hostname differs from externalKey",
            create_account_mutation("x", "cf_source", "k2", &cloudfront_literal("other", "b", "p")),
            "The configured CloudFront hostname must equal the metric source account's external key exactly.",
        ),
        (
            "blank routing value",
            create_account_mutation("x", "cf_source", "k3", &cloudfront_literal("k3", " ", "p")),
            "A CloudFront legacy S3 configuration requires non-blank hostname, bucket and prefix values.",
        ),
        (
            "unsupported stored configuration on lookup",
            account_lookup("legacy_account"),
            "The stored configuration of this metric source account is not supported by this administration surface and requires separately reviewed repair.",
        ),
        (
            "unsupported stored configuration on update",
            update_account_mutation("legacy_account", "{ kind: EMPTY }", false),
            "The stored configuration of this metric source account is not supported by this administration surface and requires separately reviewed repair.",
        ),
    ];

    for (label, document, expected) in cases {
        let response = run(&schema, &context, &document).await;
        let (message, _) = only_error(&response);
        assert_eq!(message, expected, "{label} returned an unexpected message");
        let rendered = response.to_string();
        for leaked in [
            "metric_source_",
            "duplicate key value",
            "violates",
            "INSERT INTO",
            "SELECT ",
            "DETAIL:",
            "CONTEXT:",
            "pg_",
            "postgres://",
            "Database error",
            // the planted pre-existing configuration must never escape
            "legacy\"",
            "secretName",
            "vault/not-real",
        ] {
            assert!(
                !rendered.contains(leaked),
                "{label} leaked `{leaked}`: {rendered}"
            );
        }
    }
    assert_eq!(count(&pool, "(SELECT COUNT(*) FROM metric_source)"), 2);
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        2
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_source_account WHERE code = 'legacy_account' AND enabled AND configuration ? 'secretName')"),
        1,
        "the unsupported stored value must be neither rewritten nor disabled"
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history)"
        ),
        0
    );
}

// --------------------------------------------------------------------------
// Generated schema: strictly additive, exact typed configuration, no
// unauthorized surface
// --------------------------------------------------------------------------

/// Signatures with descriptions removed and whitespace collapsed.
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
fn the_sdl_exposes_exactly_the_six_approved_operations() {
    let sdl = create_schema().as_sdl();
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    let mutation = signatures(sdl_block(&sdl, "type MutationRoot {"));

    for lookup in [
        "metricSourceByCode(code:String!):MetricSource!",
        "metricSourceAccountByCode(code:String!):MetricSourceAccount!",
    ] {
        assert!(query.contains(lookup), "missing `{lookup}` in QueryRoot");
    }
    for operation in [
        "createMetricSource(data:NewMetricSource!):MetricSource!",
        "updateMetricSource(data:PatchMetricSource!):MetricSource!",
        "createMetricSourceAccount(data:NewMetricSourceAccount!):MetricSourceAccount!",
        "updateMetricSourceAccount(data:PatchMetricSourceAccount!):MetricSourceAccount!",
    ] {
        assert!(
            mutation.contains(operation),
            "missing `{operation}` in MutationRoot"
        );
    }
    // Exactly two source-family query fields and four source-family
    // administration mutation fields: no list, search, delete, bulk or
    // checkpoint administration operation. The separately approved `MET-WP2-02`
    // lifecycle mutation `updateMetricSourceCheckpoint` (#908) shares the
    // `updateMetricSource` prefix but is not source administration; it is
    // excluded by exact name here and pinned below.
    assert_eq!(
        sdl_block(&sdl, "type QueryRoot {")
            .lines()
            .filter(|line| line.trim_start().starts_with("metricSource"))
            .count(),
        2
    );
    assert_eq!(
        sdl_block(&sdl, "type MutationRoot {")
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                (line.starts_with("createMetricSource") || line.starts_with("updateMetricSource"))
                    && !line.starts_with("updateMetricSourceCheckpoint(")
            })
            .count(),
        4
    );
    // The MET-WP2-02 lifecycle surface that touches source checkpoints is
    // exactly its two approved machine-service mutations, each declared once,
    // and nothing reaches QueryRoot.
    for lifecycle in [
        "claimMetricSourceUnits(input:ClaimMetricSourceUnitsInput!):[MetricSourceUnitClaim!]!",
        "updateMetricSourceCheckpoint(input:UpdateMetricSourceCheckpointInput!):MetricSourceCheckpoint!",
    ] {
        assert_eq!(
            mutation.matches(lifecycle).count(),
            1,
            "`{lifecycle}` must be declared exactly once in MutationRoot"
        );
    }
    assert!(
        !query.contains("MetricSourceCheckpoint") && !query.contains("claimMetricSource"),
        "no source checkpoint surface may reach QueryRoot"
    );
    for deferred in [
        "metricSources",
        "metricSourceAccounts",
        "metricSourceCheckpoint",
        "metricSourceCheckpoints",
        "createMetricSourceCheckpoint",
        "deleteMetricSourceCheckpoint",
        "input NewMetricSourceCheckpoint",
        "input PatchMetricSourceCheckpoint",
        "deleteMetricSource",
        "claimMetricSourceAccount",
        "MetricSourceRegistryHistory",
        "metricSourceRegistryHistory",
    ] {
        assert!(
            !sdl.contains(deferred),
            "`{deferred}` is not part of MET-WP1-13"
        );
    }
}

#[test]
fn the_typed_configuration_sdl_matches_amendment_1_exactly() {
    let sdl = create_schema().as_sdl();

    let kind = sdl_block(&sdl, "enum MetricSourceAccountConfigurationKind {");
    let values: Vec<&str> = kind
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.rsplit(' ').next().expect("enum value"))
        .collect();
    assert_eq!(values, ["EMPTY", "CLOUDFRONT_LEGACY_S3_V1"]);

    let acquisition = sdl_block(&sdl, "enum MetricSourceAcquisitionType {");
    let values: Vec<&str> = acquisition
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.rsplit(' ').next().expect("enum value"))
        .collect();
    assert_eq!(
        values,
        ["DRIVER", "PUBLISHER_UPLOAD", "OPERAS", "ADMIN_IMPORT"]
    );

    for (declaration, expected) in [
        (
            "input MetricSourceAccountConfigurationInput {",
            "kind:MetricSourceAccountConfigurationKind!cloudfrontLegacyS3:MetricCloudFrontLegacyS3ConfigurationInput",
        ),
        (
            "input MetricCloudFrontLegacyS3ConfigurationInput {",
            "hostname:String!bucket:String!prefix:String!",
        ),
        (
            "type MetricSourceAccountConfiguration {",
            "kind:MetricSourceAccountConfigurationKind!cloudfrontLegacyS3:MetricCloudFrontLegacyS3Configuration",
        ),
        (
            "type MetricCloudFrontLegacyS3Configuration {",
            "hostname:String!bucket:String!prefix:String!",
        ),
        (
            "input NewMetricSource {",
            "code:String!acquisitionType:MetricSourceAcquisitionType!driverKey:Stringenabled:Boolean!defaultLookbackDays:IntdefaultFinalizationDelayDays:Int",
        ),
        (
            "input PatchMetricSource {",
            "code:String!enabled:Boolean!defaultLookbackDays:IntdefaultFinalizationDelayDays:Int",
        ),
        (
            "input NewMetricSourceAccount {",
            "code:String!sourceCode:String!platformCode:String!externalKey:String!expectedPublisherId:Uuidconfiguration:MetricSourceAccountConfigurationInput!enabled:Boolean!",
        ),
        (
            "input PatchMetricSourceAccount {",
            "code:String!configuration:MetricSourceAccountConfigurationInput!enabled:Boolean!",
        ),
        (
            "type MetricSource {",
            "sourceId:Uuid!code:String!acquisitionType:MetricSourceAcquisitionType!driverKey:Stringenabled:Boolean!defaultLookbackDays:IntdefaultFinalizationDelayDays:Int",
        ),
        (
            "type MetricSourceAccount {",
            "sourceAccountId:Uuid!code:String!sourceId:Uuid!platformId:Uuid!externalKey:String!expectedPublisherId:Uuidconfiguration:MetricSourceAccountConfiguration!enabled:Boolean!",
        ),
    ] {
        assert_eq!(
            signatures(sdl_block(&sdl, declaration)),
            expected,
            "`{declaration}` must match the frozen representation exactly"
        );
    }

    // No raw JSON anywhere in the new surface, and no nested relation
    // resolution: identities are plain identifiers.
    for declaration in [
        "input MetricSourceAccountConfigurationInput {",
        "input NewMetricSourceAccount {",
        "input PatchMetricSourceAccount {",
        "type MetricSourceAccount {",
        "type MetricSourceAccountConfiguration {",
    ] {
        let block = sdl_block(&sdl, declaration);
        for forbidden in [
            "Json",
            "JSON",
            "Jsonb",
            "rawConfiguration",
            "credential",
            "secret",
            "source: MetricSource",
            "platform: MetricPlatform",
            "publisher: Publisher",
        ] {
            assert!(
                !block.contains(forbidden),
                "`{declaration}` must not contain `{forbidden}`"
            );
        }
    }
    for absent in ["scalar Json", "scalar JSON", "scalar Jsonb"] {
        assert!(
            !sdl.contains(absent),
            "MET-WP1-13 must not introduce `{absent}`"
        );
    }
}

#[test]
fn no_metrics_source_field_reaches_an_unprotected_type() {
    let sdl = create_schema().as_sdl();
    for public_type in [
        "type Work {",
        "type Publisher {",
        "type Imprint {",
        "type Publication {",
    ] {
        let block = sdl_block(&sdl, public_type);
        assert!(
            !block.contains("MetricSource"),
            "`{public_type}` must not expose a source field: {block}"
        );
    }
}

#[test]
fn the_resolvers_authorize_before_reaching_a_coordinator() {
    let mutation_source = include_str!("mutation.rs");
    let query_source = include_str!("query.rs");
    let guarded: String = mutation_source
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for coordinator in [
        "create_metric_source",
        "update_metric_source",
        "create_metric_source_account",
        "update_metric_source_account",
    ] {
        let guarded_call = format!(
            "authorize_metric_registry_admin(context) .and_then(|actor| {coordinator}(&context.db, actor, &data))"
        );
        assert_eq!(
            guarded.matches(&guarded_call).count(),
            1,
            "`{coordinator}` must have exactly one call site, and it must be guarded"
        );
        assert_eq!(
            guarded
                .matches(&format!("{coordinator}(&context.db"))
                .count(),
            1,
            "`{coordinator}` must not be reachable from a second call site"
        );
    }
    let queries: String = query_source
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for lookup in ["metric_source_by_code", "metric_source_account_by_code"] {
        let guarded_lookup =
            format!("context .require_superuser() .and_then(|_| {lookup}(&context.db, &code))");
        assert_eq!(
            queries.matches(&guarded_lookup).count(),
            1,
            "`{lookup}` must be reached only through the superuser guard"
        );
        assert_eq!(queries.matches(&format!("{lookup}(&context.db")).count(), 1);
    }
}
