//! `MET-WP1-12` protected Metrics registry administration evidence.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the nine approved operations, their
//! fail-closed SUPERUSER-only authorization, the exact-code contract as seen
//! through GraphQL, the replacement (not patch) null semantics, sanitized
//! constraint failures at the API boundary, and the strictly additive SDL.
//!
//! Coordinator, audit, atomicity, no-op, locking and concurrency evidence lives
//! with each registry in `crate::model::metric_platform`,
//! `crate::model::metric_measure` and `crate::model::metric_platform_measure`.

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

fn insert_platform(pool: &PgPool, code: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ($1, $2, 'EXTERNAL', TRUE)",
    )
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(format!("Platform {code}"))
    .execute(&mut connection)
    .expect("Failed to insert platform fixture");
}

// --------------------------------------------------------------------------
// The nine approved operations, as GraphQL documents
// --------------------------------------------------------------------------

fn create_platform_mutation(code: &str) -> String {
    format!(
        "mutation {{ createMetricPlatform(data: {{ code: \"{code}\", \
           displayName: \"Platform\", ownershipClass: EXTERNAL, enabled: true, \
           publicDescription: \"Described\" }}) \
         {{ platformId code displayName ownershipClass enabled publicDescription \
            createdAt updatedAt }} }}"
    )
}

fn update_platform_mutation(code: &str, display_name: &str, description: &str) -> String {
    format!(
        "mutation {{ updateMetricPlatform(data: {{ code: \"{code}\", \
           displayName: \"{display_name}\", enabled: true, {description} }}) \
         {{ platformId code displayName ownershipClass enabled publicDescription }} }}"
    )
}

fn create_measure_mutation(code: &str) -> String {
    format!(
        "mutation {{ createMetricMeasure(data: {{ code: \"{code}\", displayName: \"Measure\", \
           category: USAGE, unit: COUNT, allowNegative: false, publicVisibility: true, \
           additiveAcrossTime: true, additiveAcrossWorks: true, \
           definition: \"What it counts.\", methodologyVersion: \"m/1\", enabled: true }}) \
         {{ measureId code displayName category unit allowNegative publicVisibility \
            additiveAcrossTime additiveAcrossWorks definition methodologyVersion enabled }} }}"
    )
}

fn update_measure_mutation(code: &str, display_name: &str, methodology: &str) -> String {
    format!(
        "mutation {{ updateMetricMeasure(data: {{ code: \"{code}\", \
           displayName: \"{display_name}\", publicVisibility: true, \
           definition: \"What it counts.\", {methodology} enabled: true }}) \
         {{ measureId code displayName category unit allowNegative methodologyVersion enabled }} }}"
    )
}

fn create_mapping_mutation(platform_code: &str, measure_code: &str) -> String {
    format!(
        "mutation {{ createMetricPlatformMeasure(data: {{ platformCode: \"{platform_code}\", \
           measureCode: \"{measure_code}\", supportedGrains: [DAY, MONTH], \
           supportsCountry: true, supportsInstitution: false, supportsPublication: true, \
           directCollection: false, enabled: true }}) \
         {{ platformMeasureId platformId measureId supportedGrains supportsCountry \
            supportsInstitution supportsPublication directCollection enabled }} }}"
    )
}

fn update_mapping_mutation(platform_code: &str, measure_code: &str, grains: &str) -> String {
    format!(
        "mutation {{ updateMetricPlatformMeasure(data: {{ platformCode: \"{platform_code}\", \
           measureCode: \"{measure_code}\", supportedGrains: {grains}, \
           supportsCountry: true, supportsInstitution: false, supportsPublication: true, \
           directCollection: true, enabled: true }}) \
         {{ platformMeasureId supportedGrains directCollection }} }}"
    )
}

fn platform_lookup(code: &str) -> String {
    format!("{{ metricPlatformByCode(code: \"{code}\") {{ platformId code displayName }} }}")
}

fn measure_lookup(code: &str) -> String {
    format!("{{ metricMeasureByCode(code: \"{code}\") {{ measureId code displayName }} }}")
}

fn mapping_lookup(platform_code: &str, measure_code: &str) -> String {
    format!(
        "{{ metricPlatformMeasureByCodes(platformCode: \"{platform_code}\", \
           measureCode: \"{measure_code}\") {{ platformMeasureId enabled }} }}"
    )
}

/// Every one of the nine approved operations, with the field each returns.
///
/// Naming them in one place is what makes the authorization matrix below
/// exhaustive: a tenth operation added without review would not appear here,
/// and the SDL guard further down counts the Metrics surface independently.
fn all_nine_operations(platform_code: &str, measure_code: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "createMetricPlatform",
            create_platform_mutation("new_platform"),
        ),
        (
            "updateMetricPlatform",
            update_platform_mutation(platform_code, "Renamed", "publicDescription: \"d\""),
        ),
        (
            "createMetricMeasure",
            create_measure_mutation("new_measure"),
        ),
        (
            "updateMetricMeasure",
            update_measure_mutation(measure_code, "Renamed", "methodologyVersion: \"m/2\","),
        ),
        (
            "createMetricPlatformMeasure",
            create_mapping_mutation(platform_code, measure_code),
        ),
        (
            "updateMetricPlatformMeasure",
            update_mapping_mutation(platform_code, measure_code, "[DAY]"),
        ),
        ("metricPlatformByCode", platform_lookup(platform_code)),
        ("metricMeasureByCode", measure_lookup(measure_code)),
        (
            "metricPlatformMeasureByCodes",
            mapping_lookup(platform_code, measure_code),
        ),
    ]
}

// --------------------------------------------------------------------------
// Authorization
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_operation_is_denied_to_every_caller_that_is_not_a_superuser() {
    let (_guard, pool) = setup_registry_db();
    insert_platform(&pool, "existing");
    let schema = create_schema();

    // A publisher exists so a publisher-scoped role is a real, well-formed
    // principal rather than a role with no organisation.
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
    ];

    for (label, context) in &denied_contexts {
        for (operation, document) in all_nine_operations("existing", "net_units") {
            let response = run(&schema, context, &document).await;
            assert_unauthorized(&response);
            assert!(
                response["data"].is_null() || response["data"][operation].is_null(),
                "{label} must receive no data for {operation}: {response}"
            );
        }
    }

    // Not one denied call reached the database: the registry is exactly as the
    // fixture left it and the audit history is empty.
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_platform)"),
        1,
        "a denied mutation must not create a platform"
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_platform WHERE display_name = 'Platform existing')"
        ),
        1,
        "a denied update must not change a platform"
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_platform_measure)"),
        0
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)"),
        0,
        "a denied call must write no audit row"
    );
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_measure)"),
        2,
        "a denied create must not add a measure to the two seeds"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authorization_is_decided_before_any_database_access() {
    // The context's pool points at an unreachable database. If any operation
    // touched the database before deciding authorization, the response would be
    // an internal/connection error rather than the fail-closed denial.
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
        for (operation, document) in all_nine_operations("any", "any") {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_superuser_can_drive_the_whole_administrative_lifecycle() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    // Create a platform.
    let created = run(&schema, &context, &create_platform_mutation("cf_demo")).await;
    let platform = data(&created, "createMetricPlatform");
    assert_eq!(platform["code"], "cf_demo");
    assert_eq!(platform["ownershipClass"], "EXTERNAL");
    assert_eq!(platform["publicDescription"], "Described");
    assert!(platform["platformId"].is_string());

    // Create a measure, and address the seeded ones by code.
    let measure = run(&schema, &context, &create_measure_mutation("downloads")).await;
    assert_eq!(data(&measure, "createMetricMeasure")["code"], "downloads");
    for seeded in ["title_sessions", "net_units"] {
        let looked_up = run(&schema, &context, &measure_lookup(seeded)).await;
        assert_eq!(data(&looked_up, "metricMeasureByCode")["code"], seeded);
    }

    // Map the platform to a seeded measure, then administer the mapping.
    let mapping = run(
        &schema,
        &context,
        &create_mapping_mutation("cf_demo", "title_sessions"),
    )
    .await;
    let mapping = data(&mapping, "createMetricPlatformMeasure");
    assert_eq!(mapping["supportedGrains"], json!(["DAY", "MONTH"]));
    assert_eq!(mapping["directCollection"], json!(false));

    let updated = run(
        &schema,
        &context,
        &update_mapping_mutation("cf_demo", "title_sessions", "[REPORTING_PERIOD]"),
    )
    .await;
    let updated = data(&updated, "updateMetricPlatformMeasure");
    assert_eq!(updated["supportedGrains"], json!(["REPORTING_PERIOD"]));
    assert_eq!(updated["directCollection"], json!(true));
    assert_eq!(updated["platformMeasureId"], mapping["platformMeasureId"]);

    // The lookups resolve what was written.
    let found = run(&schema, &context, &platform_lookup("cf_demo")).await;
    assert_eq!(
        data(&found, "metricPlatformByCode")["platformId"],
        platform["platformId"]
    );
    let found = run(
        &schema,
        &context,
        &mapping_lookup("cf_demo", "title_sessions"),
    )
    .await;
    assert_eq!(
        data(&found, "metricPlatformMeasureByCodes")["enabled"],
        true
    );

    // Four committed changes, four audit rows, all attributed to the caller.
    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)"),
        4
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT COUNT(*) FROM metric_registry_history WHERE actor = 'super-1')"
        ),
        4,
        "the audit actor is the authenticated principal, not a request value"
    );
}

// --------------------------------------------------------------------------
// Exact-code and replacement semantics through the API
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn codes_are_matched_exactly_through_the_api() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    run(&schema, &context, &create_platform_mutation("Exact_Code")).await;

    let found = run(&schema, &context, &platform_lookup("Exact_Code")).await;
    assert_eq!(data(&found, "metricPlatformByCode")["code"], "Exact_Code");

    for variant in ["exact_code", "EXACT_CODE", " Exact_Code", "Exact_Code "] {
        let response = run(&schema, &context, &platform_lookup(variant)).await;
        let (message, kind) = only_error(&response);
        // The existing repository taxonomy is unchanged: an absent code is the
        // ordinary `EntityNotFound`, which carries no PostgreSQL detail.
        assert_eq!(
            (message.as_str(), kind.as_str()),
            ("No record was found for the given ID.", "INTERNAL_ERROR"),
            "`{variant}` must not fold onto the stored code"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_patch_is_a_replacement_so_an_omitted_nullable_field_stores_null() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    run(&schema, &context, &create_platform_mutation("nullable")).await;
    run(&schema, &context, &create_measure_mutation("nullable")).await;

    // publicDescription omitted entirely.
    let updated = run(
        &schema,
        &context,
        &update_platform_mutation("nullable", "Renamed", ""),
    )
    .await;
    assert_eq!(
        data(&updated, "updateMetricPlatform")["publicDescription"],
        JsonValue::Null,
        "an omitted nullable field must store SQL NULL, not retain the old value"
    );

    // methodologyVersion sent explicitly as null.
    let updated = run(
        &schema,
        &context,
        &update_measure_mutation("nullable", "Renamed", "methodologyVersion: null,"),
    )
    .await;
    assert_eq!(
        data(&updated, "updateMetricMeasure")["methodologyVersion"],
        JsonValue::Null,
        "an explicit null must store SQL NULL"
    );

    assert_eq!(
        count(
            &pool,
            "((SELECT COUNT(*) FROM metric_platform WHERE public_description IS NULL) \
              + (SELECT COUNT(*) FROM metric_measure \
                 WHERE code = 'nullable' AND methodology_version IS NULL))"
        ),
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_no_op_update_through_the_api_changes_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    run(&schema, &context, &create_platform_mutation("noop")).await;
    let audit_after_create = count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)");
    let timestamp_before = count(
        &pool,
        "(SELECT (EXTRACT(EPOCH FROM updated_at) * 1000000)::bigint FROM metric_platform \
          WHERE code = 'noop')",
    );

    let response = run(
        &schema,
        &context,
        &update_platform_mutation("noop", "Platform", "publicDescription: \"Described\""),
    )
    .await;
    let returned = data(&response, "updateMetricPlatform");
    assert_eq!(returned["displayName"], "Platform");
    assert_eq!(returned["publicDescription"], "Described");

    assert_eq!(
        count(&pool, "(SELECT COUNT(*) FROM metric_registry_history)"),
        audit_after_create,
        "a no-op update must write no audit row"
    );
    assert_eq!(
        count(
            &pool,
            "(SELECT (EXTRACT(EPOCH FROM updated_at) * 1000000)::bigint FROM metric_platform \
              WHERE code = 'noop')"
        ),
        timestamp_before,
        "a no-op update must not move the canonical timestamp"
    );
}

// --------------------------------------------------------------------------
// Sanitized failure boundary
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn constraint_failures_reach_the_client_bounded_and_sanitised() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("super-1"));

    run(&schema, &context, &create_platform_mutation("dup")).await;
    run(
        &schema,
        &context,
        &create_mapping_mutation("dup", "net_units"),
    )
    .await;

    let blank_code = "mutation { createMetricPlatform(data: { code: \"   \", \
        displayName: \"Platform\", ownershipClass: EXTERNAL, enabled: true }) { platformId } }"
        .to_string();
    let empty_grains = "mutation { createMetricPlatformMeasure(data: { platformCode: \"dup\", \
        measureCode: \"title_sessions\", supportedGrains: [], supportsCountry: true, \
        supportsInstitution: false, supportsPublication: true, directCollection: false, \
        enabled: true }) { platformMeasureId } }"
        .to_string();

    let cases = [
        (
            "duplicate platform code",
            create_platform_mutation("dup"),
            "A metric platform with this code already exists.",
        ),
        (
            "duplicate measure code",
            create_measure_mutation("net_units"),
            "A metric measure with this code already exists.",
        ),
        (
            "duplicate mapping pair",
            create_mapping_mutation("dup", "net_units"),
            "A mapping between this metric platform and this metric measure already exists.",
        ),
        (
            "blank platform code",
            blank_code,
            "Metric platform code must not be an empty string.",
        ),
        (
            "empty supported grains",
            empty_grains,
            "Supported grains must list at least one reporting grain, with no duplicates.",
        ),
    ];

    for (label, document, expected) in cases {
        let response = run(&schema, &context, &document).await;
        let (message, _) = only_error(&response);
        assert_eq!(message, expected, "{label} returned an unexpected message");

        // Nothing internal escapes: no constraint name, no SQL, no driver or
        // connection diagnostics, no table or column names from the driver.
        let rendered = response.to_string();
        for leaked in [
            "metric_platform_code_key",
            "metric_measure_code_key",
            "metric_platform_measure_platform_id_measure_id_key",
            "metric_platform_code_check",
            "metric_platform_measure_supported_grains_check",
            "duplicate key value",
            "violates",
            "INSERT INTO",
            "SELECT ",
            "DETAIL:",
            "CONTEXT:",
            "pg_",
            "postgres://",
            "cardinality(",
            "Database error",
        ] {
            assert!(
                !rendered.contains(leaked),
                "{label} leaked `{leaked}`: {rendered}"
            );
        }
    }
}

// --------------------------------------------------------------------------
// Generated schema: strictly additive, and no unauthorized surface
// --------------------------------------------------------------------------

#[test]
fn the_sdl_exposes_exactly_the_nine_approved_operations() {
    let sdl = create_schema().as_sdl();
    let query_root = sdl_block(&sdl, "type QueryRoot {");
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");

    // Juniper writes each argument's description inline, before the argument
    // itself, so the signature is compared with descriptions removed and
    // whitespace collapsed. Names, argument types, return types and nullability
    // are still matched exactly.
    let signatures = |block: &str| -> String {
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
    };
    let query_signatures = signatures(query_root);
    let mutation_signatures = signatures(mutation_root);

    for lookup in [
        "metricPlatformByCode(code:String!):MetricPlatform!",
        "metricMeasureByCode(code:String!):MetricMeasure!",
        "metricPlatformMeasureByCodes(platformCode:String!,measureCode:String!):MetricPlatformMeasure!",
    ] {
        assert!(
            query_signatures.contains(lookup),
            "the approved lookup `{lookup}` is missing from QueryRoot: {query_signatures}"
        );
    }
    for mutation in [
        "createMetricPlatform(data:NewMetricPlatform!):MetricPlatform!",
        "updateMetricPlatform(data:PatchMetricPlatform!):MetricPlatform!",
        "createMetricMeasure(data:NewMetricMeasure!):MetricMeasure!",
        "updateMetricMeasure(data:PatchMetricMeasure!):MetricMeasure!",
        "createMetricPlatformMeasure(data:NewMetricPlatformMeasure!):MetricPlatformMeasure!",
        "updateMetricPlatformMeasure(data:PatchMetricPlatformMeasure!):MetricPlatformMeasure!",
    ] {
        assert!(
            mutation_signatures.contains(mutation),
            "the approved mutation `{mutation}` is missing from MutationRoot"
        );
    }

    // Exactly nine registry Metrics fields across both roots, plus the six
    // `MET-WP1-13` source/source-account operations approved under #904 and
    // proven exactly in `metric_source_registry_tests`, plus the three
    // `MET-WP4-02` read-service operations approved under #910 and proven
    // exactly in `metric_dashboard_tests`, plus the one `MET-WP2-02` lifecycle
    // mutation approved under #908 whose name also begins `updateMetric`
    // (proven exactly, with its four sibling lifecycle operations, in
    // `metric_ingestion_lifecycle_tests`). No other Metrics operation exists.
    // Each group is compared as an exact set of field names, so a renamed,
    // missing or additional operation fails here rather than hiding behind a
    // matching count.
    let wp1_13_query_fields = ["metricSourceByCode", "metricSourceAccountByCode"];
    let wp1_13_mutation_fields = [
        "createMetricSource",
        "updateMetricSource",
        "createMetricSourceAccount",
        "updateMetricSourceAccount",
    ];
    let wp4_02_query_fields = ["metricDashboard", "metricMeasures", "metricPlatforms"];
    let wp2_02_mutation_fields = ["updateMetricSourceCheckpoint"];
    let field_name = |line: &str| -> String {
        line.trim_start()
            .split(['(', ':'])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let sorted = |mut names: Vec<String>| {
        names.sort();
        names
    };
    let owned = |names: &[&str]| sorted(names.iter().map(|name| name.to_string()).collect());
    let all_query_fields: Vec<String> = query_root
        .lines()
        .filter(|line| line.trim_start().starts_with("metric"))
        .map(field_name)
        .collect();
    let all_mutation_fields: Vec<String> = mutation_root
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.starts_with("createMetric") || line.starts_with("updateMetric")
        })
        .map(field_name)
        .collect();
    let in_group = |fields: &[String], group: &[&str]| {
        sorted(
            fields
                .iter()
                .filter(|name| group.contains(&name.as_str()))
                .cloned()
                .collect(),
        )
    };
    let metric_query_fields: Vec<String> = sorted(
        all_query_fields
            .iter()
            .filter(|name| {
                !wp1_13_query_fields.contains(&name.as_str())
                    && !wp4_02_query_fields.contains(&name.as_str())
            })
            .cloned()
            .collect(),
    );
    let metric_mutation_fields: Vec<String> = sorted(
        all_mutation_fields
            .iter()
            .filter(|name| {
                !wp1_13_mutation_fields.contains(&name.as_str())
                    && !wp2_02_mutation_fields.contains(&name.as_str())
            })
            .cloned()
            .collect(),
    );
    assert_eq!(
        metric_query_fields,
        owned(&[
            "metricPlatformByCode",
            "metricMeasureByCode",
            "metricPlatformMeasureByCodes",
        ]),
        "QueryRoot: {query_root}"
    );
    assert_eq!(
        metric_mutation_fields,
        owned(&[
            "createMetricPlatform",
            "updateMetricPlatform",
            "createMetricMeasure",
            "updateMetricMeasure",
            "createMetricPlatformMeasure",
            "updateMetricPlatformMeasure",
        ]),
        "MutationRoot: {mutation_root}"
    );
    assert_eq!(
        in_group(&all_query_fields, &wp1_13_query_fields),
        owned(&wp1_13_query_fields),
        "the MET-WP1-13 lookups must each appear exactly once"
    );
    assert_eq!(
        in_group(&all_mutation_fields, &wp1_13_mutation_fields),
        owned(&wp1_13_mutation_fields),
        "the MET-WP1-13 mutations must each appear exactly once"
    );
    assert_eq!(
        in_group(&all_query_fields, &wp4_02_query_fields),
        owned(&wp4_02_query_fields),
        "the MET-WP4-02 read operations must each appear exactly once"
    );
    assert_eq!(
        in_group(&all_mutation_fields, &wp2_02_mutation_fields),
        owned(&wp2_02_mutation_fields),
        "the MET-WP2-02 checkpoint lifecycle mutation must appear exactly once"
    );

    for deferred in [
        "metricPlatformMeasures",
        "deleteMetric",
        "metricRegistryHistory",
        "metricPublisherPlatformApproval",
        "metricSources",
        "metricSourceAccounts",
        "metricSourceCheckpoint",
    ] {
        assert!(
            !sdl.contains(deferred),
            "`{deferred}` belongs to a later, separately reviewed slice"
        );
    }
}

#[test]
fn the_added_types_expose_only_registry_state_and_no_audit_surface() {
    let sdl = create_schema().as_sdl();

    // The audit table is persistence-only: no object, input or enum for it.
    for absent in [
        "type MetricRegistryHistory",
        "MetricRegistryHistoryEntity",
        "MetricRegistryHistoryAction",
        "beforeState",
        "afterState",
    ] {
        assert!(
            !sdl.contains(absent),
            "`{absent}` must not reach the public schema in MET-WP1-12"
        );
    }

    // The mapping object exposes identifiers, not resolved relations: a nested
    // platform/measure field would be an unauthorized second read path.
    let mapping = sdl_block(&sdl, "type MetricPlatformMeasure {");
    assert!(mapping.contains("platformId: Uuid!"));
    assert!(mapping.contains("measureId: Uuid!"));
    for forbidden in ["platform: MetricPlatform", "measure: MetricMeasure"] {
        assert!(
            !mapping.contains(forbidden),
            "the mapping type must not resolve `{forbidden}`"
        );
    }

    // The immutable fields are absent from the patch inputs, so an attempt to
    // change one is a schema error rather than a silently ignored field.
    let platform_patch = sdl_block(&sdl, "input PatchMetricPlatform {");
    assert!(!platform_patch.contains("ownershipClass"));
    let measure_patch = sdl_block(&sdl, "input PatchMetricMeasure {");
    for immutable in [
        "category",
        "unit",
        "allowNegative",
        "additiveAcrossTime",
        "additiveAcrossWorks",
    ] {
        assert!(
            !measure_patch.contains(immutable),
            "`{immutable}` must not be changeable through PatchMetricMeasure"
        );
    }
    let mapping_patch = sdl_block(&sdl, "input PatchMetricPlatformMeasure {");
    assert!(!mapping_patch.contains("platformId"));
    assert!(!mapping_patch.contains("measureId"));
}

#[test]
fn the_exposed_enums_carry_exactly_their_existing_database_values() {
    let sdl = create_schema().as_sdl();

    for (declaration, values) in [
        (
            "enum MetricPlatformOwnershipClass {",
            vec!["THOTH_MANAGED", "PUBLISHER_CONTROLLED", "EXTERNAL"],
        ),
        ("enum MetricMeasureCategory {", vec!["USAGE", "SALES"]),
        ("enum MetricMeasureUnit {", vec!["COUNT"]),
        (
            "enum MetricReportingGrain {",
            vec!["DAY", "MONTH", "REPORTING_PERIOD"],
        ),
    ] {
        let block = sdl_block(&sdl, declaration);
        // Juniper emits each value as `"description" VALUE` on one line, so the
        // value is the final whitespace-separated token.
        let declared: Vec<&str> = block
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.rsplit(' ').next().expect("enum value token"))
            .collect();
        assert_eq!(
            declared, values,
            "`{declaration}` must expose exactly its existing database values"
        );
    }
}

#[test]
fn no_metrics_administration_field_reaches_an_unprotected_type() {
    let sdl = create_schema().as_sdl();

    // The Metrics admin surface hangs off QueryRoot and MutationRoot only. If a
    // Metrics field were added to a publicly reachable type, an anonymous
    // caller could navigate into the registry without passing a guard.
    for public_type in [
        "type Work {",
        "type Publisher {",
        "type Imprint {",
        "type Publication {",
    ] {
        let block = sdl_block(&sdl, public_type);
        assert!(
            !block.contains("Metric"),
            "`{public_type}` must not expose a Metrics field: {block}"
        );
    }
}

#[test]
fn the_resolvers_authorize_before_reaching_a_coordinator() {
    // Source-level containment: every Metrics administration resolver goes
    // through the one guard helper, and no resolver calls a coordinator without
    // it. This complements the runtime evidence above by proving there is no
    // second, unguarded call site.
    let mutation_source = include_str!("mutation.rs");
    let query_source = include_str!("query.rs");

    // Every coordinator call site is reached only through the guard: the
    // resolvers all use the same `guard(...).and_then(|actor| coordinator(...))`
    // shape, so an unguarded call would not match.
    let guarded: String = mutation_source
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for coordinator in [
        "create_metric_platform",
        "update_metric_platform",
        "create_metric_measure",
        "update_metric_measure",
        "create_metric_platform_measure",
        "update_metric_platform_measure",
    ] {
        let guarded_call = format!(
            "authorize_metric_registry_admin(context) .and_then(|actor| {coordinator}(&context.db, actor, &data))"
        );
        assert_eq!(
            guarded.matches(&guarded_call).count(),
            1,
            "`{coordinator}` must have exactly one call site, and it must be guarded"
        );
        // No second call site of any shape: the coordinator name appears once in
        // the import list, once as the resolver's own name, and once in the
        // guarded call above.
        assert_eq!(
            guarded
                .matches(&format!("{coordinator}(&context.db"))
                .count(),
            1,
            "`{coordinator}` must not be reachable from a second call site"
        );
    }
    // The six registry mutations plus the four `MET-WP1-13` source/source-account
    // mutations share the one guard; the WP1-13 call sites are proven
    // individually in `metric_source_registry_tests`.
    assert_eq!(
        mutation_source
            .matches("authorize_metric_registry_admin(context)")
            .count(),
        10,
        "each of the ten Metrics administration mutations must call the guard exactly once"
    );
    assert_eq!(
        mutation_source
            .matches("fn authorize_metric_registry_admin")
            .count(),
        1,
        "there must be exactly one Metrics administration guard"
    );
    assert_eq!(
        query_source.matches("require_superuser()").count(),
        7,
        "the three registry lookups and the two MET-WP1-13 source lookups join the two \
         pre-existing superuser queries"
    );
}
