//! ROR-SYNC-01C (thoth#1024) through GraphQL, built without `cfg(test)`:
//! absent, explicit-null and value inputs for `rorStatus` and `rorSuccessor`,
//! the unprivileged-caller rule, `me.isRorSyncService`, `rorStatuses`, the
//! constraint messages, history rows, and the guard on `Crud::create`.
#![cfg(feature = "backend")]

#[allow(dead_code)]
mod support;

use std::collections::HashMap;
use std::sync::Arc;

use diesel::sql_types::{BigInt, Text};
use diesel::{QueryableByName, RunQueryDsl};
use serde_json::{json, Value};
use thoth_api::db::PgPool;
use thoth_api::model::institution::crud::crud_create_without_history;
use thoth_api::model::institution::NewInstitution;
use thoth_errors::ThothError;
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

const ROR_A: &str = "https://ror.org/0aaaaaa00";
const ROR_B: &str = "https://ror.org/0bbbbbb00";
const ROR_C: &str = "https://ror.org/0cccccc00";

const FIELDS: &str = "institutionId institutionName ror rorStatus rorSuccessor";

fn user_with_unscoped_roles(user_id: &str, roles: &[&str]) -> IntrospectedUser {
    let mut user = support::superuser(user_id);
    let mut project_roles = HashMap::new();
    for role in roles {
        project_roles.insert((*role).to_string(), HashMap::new());
    }
    user.project_roles = Some(project_roles);
    user
}

fn ror_sync(user_id: &str) -> IntrospectedUser {
    user_with_unscoped_roles(user_id, &["ROR_SYNC_SERVICE"])
}

fn no_role(user_id: &str) -> IntrospectedUser {
    let mut user = support::superuser(user_id);
    user.project_roles = None;
    user
}

fn setup() -> (support::TestDbGuard, Arc<PgPool>) {
    let guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");
    (guard, pool)
}

#[derive(QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(QueryableByName)]
struct HistoryRow {
    #[diesel(sql_type = Text)]
    user_id: String,
    #[diesel(sql_type = Text)]
    data: String,
}

fn count(pool: &PgPool, sql: &str) -> i64 {
    let mut connection = pool.get().expect("connection");
    diesel::sql_query(sql)
        .get_result::<Count>(&mut connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .n
}

fn history(pool: &PgPool, institution_id: &str) -> Vec<HistoryRow> {
    let mut connection = pool.get().expect("connection");
    diesel::sql_query(format!(
        "SELECT user_id, data #>> '{{}}' AS data FROM institution_history \
         WHERE institution_id = '{institution_id}' ORDER BY timestamp"
    ))
    .load::<HistoryRow>(&mut connection)
    .expect("history")
}

async fn run(
    pool: &Arc<PgPool>,
    user: Option<IntrospectedUser>,
    query: &str,
    variables: Option<Value>,
) -> Value {
    support::execute_graphql(pool.clone(), user, query, variables).await
}

async fn create(
    pool: &Arc<PgPool>,
    user: IntrospectedUser,
    name: &str,
    ror: Option<&str>,
) -> Value {
    let query = format!(
        "mutation($data: NewInstitution!) {{ createInstitution(data: $data) {{ {FIELDS} }} }}"
    );
    let response = run(
        pool,
        Some(user),
        &query,
        Some(json!({ "data": { "institutionName": name, "ror": ror } })),
    )
    .await;
    support::assert_no_errors(&response);
    response["data"]["createInstitution"].clone()
}

/// `updateInstitution` with `data` written as a GraphQL literal, so that a
/// field the caller leaves out is absent from the request.
async fn update_literal(pool: &Arc<PgPool>, user: IntrospectedUser, data: &str) -> Value {
    let query = format!("mutation {{ updateInstitution(data: {{ {data} }}) {{ {FIELDS} }} }}");
    run(pool, Some(user), &query, None).await
}

/// `updateInstitution` with `data` as a JSON variable.
async fn update_variables(pool: &Arc<PgPool>, user: IntrospectedUser, data: Value) -> Value {
    let query = format!(
        "mutation($data: PatchInstitution!) {{ updateInstitution(data: $data) {{ {FIELDS} }} }}"
    );
    run(pool, Some(user), &query, Some(json!({ "data": data }))).await
}

async fn fetch(pool: &Arc<PgPool>, institution_id: &str) -> Value {
    let query =
        format!("query {{ institution(institutionId: \"{institution_id}\") {{ {FIELDS} }} }}");
    let response = run(pool, None, &query, None).await;
    support::assert_no_errors(&response);
    response["data"]["institution"].clone()
}

fn id_of(institution: &Value) -> String {
    institution["institutionId"]
        .as_str()
        .expect("institutionId")
        .to_string()
}

fn state(institution: &Value) -> (Value, Value) {
    (
        institution["rorStatus"].clone(),
        institution["rorSuccessor"].clone(),
    )
}

/// Create an institution with a ROR ID and give it a status and successor
/// as the sync would.
async fn seeded(pool: &Arc<PgPool>, name: &str, status: &str, successor: Option<&str>) -> String {
    let created = create(pool, ror_sync("seed-sync"), name, Some(ROR_A)).await;
    let id = id_of(&created);
    let successor = successor.map_or("null".to_string(), |value| format!("\"{value}\""));
    let response = update_literal(
        pool,
        ror_sync("seed-sync"),
        &format!(
            "institutionId: \"{id}\", institutionName: \"{name}\", ror: \"{ROR_A}\", \
             rorStatus: {status}, rorSuccessor: {successor}"
        ),
    )
    .await;
    support::assert_no_errors(&response);
    id
}

#[test]
fn crud_create_outside_unit_tests_refuses_and_writes_nothing() {
    let (_guard, pool) = setup();
    let result = crud_create_without_history(
        &pool,
        &NewInstitution {
            institution_name: "Through the trait".to_string(),
            institution_doi: None,
            ror: None,
            country_code: None,
        },
    );
    match result {
        Err(ThothError::InternalError(message)) => {
            assert!(message.contains("create_with_history"), "{message}")
        }
        other => panic!("expected the create_with_history error, got {other:?}"),
    }
    assert_eq!(count(&pool, "SELECT count(*) AS n FROM institution"), 0);
    assert_eq!(
        count(&pool, "SELECT count(*) AS n FROM institution_history"),
        0
    );
}

#[tokio::test(flavor = "current_thread")]
async fn create_institution_is_active_and_writes_one_history_row_under_the_caller() {
    let (_guard, pool) = setup();
    let created = create(&pool, ror_sync("history-sync"), "Created", Some(ROR_A)).await;
    assert_eq!(state(&created), (json!("ACTIVE"), Value::Null));
    let id = id_of(&created);

    let rows = history(&pool, &id);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].user_id, "history-sync");
    let data: Value = serde_json::from_str(&rows[0].data).expect("history data");
    assert_eq!(data["institutionId"], json!(id));
    assert_eq!(data["institutionName"], json!("Created"));
    assert_eq!(data["rorStatus"], json!("ACTIVE"));
    assert_eq!(data["rorSuccessor"], Value::Null);

    // A failed create adds nothing.
    let response = run(
        &pool,
        Some(ror_sync("history-sync")),
        "mutation { createInstitution(data: { institutionName: \"\" }) { institutionId } }",
        None,
    )
    .await;
    assert!(response["errors"].as_array().is_some_and(|e| !e.is_empty()));
    assert_eq!(count(&pool, "SELECT count(*) AS n FROM institution"), 1);
    assert_eq!(
        count(&pool, "SELECT count(*) AS n FROM institution_history"),
        1
    );
}

#[tokio::test(flavor = "current_thread")]
async fn absent_fields_are_unchanged_explicit_null_clears_and_a_value_sets() {
    let (_guard, pool) = setup();
    let id = id_of(&create(&pool, ror_sync("nullable-sync"), "Nullable", Some(ROR_A)).await);
    let base = format!("institutionId: \"{id}\", institutionName: \"Nullable\", ror: \"{ROR_A}\"");

    // A value sets.
    let response = update_literal(
        &pool,
        ror_sync("nullable-sync"),
        &format!("{base}, rorStatus: INACTIVE, rorSuccessor: \"{ROR_B}\""),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&response["data"]["updateInstitution"]),
        (json!("INACTIVE"), json!(ROR_B))
    );

    // Absent, as a literal and as a variable: unchanged.
    let response = update_literal(&pool, ror_sync("nullable-sync"), &base).await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&response["data"]["updateInstitution"]),
        (json!("INACTIVE"), json!(ROR_B))
    );
    let response = update_variables(
        &pool,
        ror_sync("nullable-sync"),
        json!({ "institutionId": id, "institutionName": "Nullable", "ror": ROR_A }),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&fetch(&pool, &id).await),
        (json!("INACTIVE"), json!(ROR_B))
    );

    // An explicit null clears, as a literal.
    let response = update_literal(
        &pool,
        ror_sync("nullable-sync"),
        &format!("{base}, rorSuccessor: null"),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&fetch(&pool, &id).await),
        (json!("INACTIVE"), Value::Null)
    );

    // A value sets, and an explicit null clears, as variables.
    let response = update_variables(
        &pool,
        ror_sync("nullable-sync"),
        json!({ "institutionId": id, "institutionName": "Nullable", "ror": ROR_A,
                "rorStatus": "WITHDRAWN", "rorSuccessor": ROR_C }),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&fetch(&pool, &id).await),
        (json!("WITHDRAWN"), json!(ROR_C))
    );
    let response = update_variables(
        &pool,
        ror_sync("nullable-sync"),
        json!({ "institutionId": id, "institutionName": "Nullable", "ror": ROR_A,
                "rorSuccessor": null }),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&fetch(&pool, &id).await),
        (json!("WITHDRAWN"), Value::Null)
    );

    // Every update wrote one history row under the caller.
    let rows = history(&pool, &id);
    assert_eq!(rows.len(), 1 + 6);
    assert!(rows.iter().all(|row| row.user_id == "nullable-sync"));
}

#[tokio::test(flavor = "current_thread")]
async fn an_unprivileged_caller_is_denied_the_whole_update_for_a_differing_value() {
    let (_guard, pool) = setup();
    let id = seeded(&pool, "Guarded", "INACTIVE", Some(ROR_B)).await;
    let renamed =
        format!("institutionId: \"{id}\", institutionName: \"Renamed\", ror: \"{ROR_A}\"");
    let history_before = history(&pool, &id).len();

    for denied in [
        format!("{renamed}, rorStatus: ACTIVE"),
        format!("{renamed}, rorSuccessor: \"{ROR_C}\""),
        format!("{renamed}, rorSuccessor: null"),
    ] {
        let response = update_literal(&pool, no_role("editor"), &denied).await;
        support::assert_no_access(&response);
        let stored = fetch(&pool, &id).await;
        assert_eq!(stored["institutionName"], json!("Guarded"), "{denied}");
        assert_eq!(state(&stored), (json!("INACTIVE"), json!(ROR_B)));
    }
    assert_eq!(history(&pool, &id).len(), history_before);

    // A client that sends every optional field as null (as `graphql_client`
    // does) is denied while a successor is stored.
    let response = update_variables(
        &pool,
        no_role("editor"),
        json!({ "institutionId": id, "institutionName": "Renamed", "ror": ROR_A,
                "institutionDoi": null, "countryCode": null,
                "rorStatus": null, "rorSuccessor": null }),
    )
    .await;
    support::assert_no_access(&response);

    // Equal values and absent fields pass; the update applies and the state
    // stays as stored.
    for allowed in [
        format!("{renamed}, rorStatus: INACTIVE, rorSuccessor: \"{ROR_B}\""),
        renamed.clone(),
    ] {
        let response = update_literal(&pool, no_role("editor"), &allowed).await;
        support::assert_no_errors(&response);
        let stored = &response["data"]["updateInstitution"];
        assert_eq!(stored["institutionName"], json!("Renamed"));
        assert_eq!(state(stored), (json!("INACTIVE"), json!(ROR_B)));
    }

    // On a row with no successor, the all-null client passes.
    let plain = id_of(&create(&pool, no_role("editor"), "Plain", None).await);
    let response = update_variables(
        &pool,
        no_role("editor"),
        json!({ "institutionId": plain, "institutionName": "Plain renamed", "ror": null,
                "institutionDoi": null, "countryCode": null,
                "rorStatus": null, "rorSuccessor": null }),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&response["data"]["updateInstitution"]),
        (json!("ACTIVE"), Value::Null)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn a_change_of_ror_through_graphql_resets_the_state() {
    let (_guard, pool) = setup();
    let id = seeded(&pool, "Moving", "WITHDRAWN", Some(ROR_B)).await;
    let response = update_literal(
        &pool,
        no_role("editor"),
        &format!("institutionId: \"{id}\", institutionName: \"Moving\", ror: \"{ROR_C}\""),
    )
    .await;
    support::assert_no_errors(&response);
    assert_eq!(
        state(&response["data"]["updateInstitution"]),
        (json!("ACTIVE"), Value::Null)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn me_reports_the_ror_sync_guard_not_superuser() {
    let (_guard, pool) = setup();
    let query = "query { me { isSuperuser isRorSyncService } }";
    for (label, user, superuser, ror_sync_service) in [
        ("superuser", support::superuser("me-superuser"), true, false),
        ("ROR sync", ror_sync("me-sync"), false, true),
        (
            "superuser and ROR sync",
            user_with_unscoped_roles("me-both", &["SUPERUSER", "ROR_SYNC_SERVICE"]),
            true,
            true,
        ),
        ("no role", no_role("me-none"), false, false),
        (
            "other machine roles",
            user_with_unscoped_roles(
                "me-machines",
                &[
                    "DISSEMINATION_WORKER",
                    "METRICS_INGEST_SERVICE",
                    "METRICS_READ_SERVICE",
                ],
            ),
            false,
            false,
        ),
    ] {
        let response = run(&pool, Some(user), query, None).await;
        support::assert_no_errors(&response);
        assert_eq!(
            response["data"]["me"]["isSuperuser"],
            json!(superuser),
            "{label}"
        );
        assert_eq!(
            response["data"]["me"]["isRorSyncService"],
            json!(ror_sync_service),
            "{label}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ror_statuses_filters_institutions_and_institution_count_alike() {
    let (_guard, pool) = setup();
    let active = id_of(&create(&pool, ror_sync("filter-sync"), "Filter active", Some(ROR_A)).await);
    let inactive = seeded(&pool, "Filter inactive", "INACTIVE", None).await;
    let withdrawn = seeded(&pool, "Filter withdrawn", "WITHDRAWN", Some(ROR_B)).await;

    for (argument, expected) in [
        ("", vec![&active, &inactive, &withdrawn]),
        ("rorStatuses: []", vec![&active, &inactive, &withdrawn]),
        ("rorStatuses: [WITHDRAWN]", vec![&withdrawn]),
        ("rorStatuses: [ACTIVE, INACTIVE]", vec![&active, &inactive]),
        (
            "rorStatuses: [INACTIVE], filter: \"Filter\"",
            vec![&inactive],
        ),
        ("rorStatuses: [WITHDRAWN], filter: \"active\"", vec![]),
    ] {
        let separator = if argument.is_empty() { "" } else { ", " };
        let query = format!(
            "query {{ institutions(limit: 10, order: {{field: INSTITUTION_ID, direction: ASC}}\
             {separator}{argument}) {{ institutionId }} \
             institutionCount{} }}",
            if argument.is_empty() {
                String::new()
            } else {
                format!("({argument})")
            }
        );
        let response = run(&pool, None, &query, None).await;
        support::assert_no_errors(&response);
        let mut expected: Vec<String> = expected.into_iter().cloned().collect();
        expected.sort();
        let listed: Vec<String> = response["data"]["institutions"]
            .as_array()
            .expect("institutions")
            .iter()
            .map(id_of)
            .collect();
        assert_eq!(listed, expected, "{argument}");
        assert_eq!(
            response["data"]["institutionCount"],
            json!(expected.len()),
            "{argument}"
        );
    }

    let response = run(
        &pool,
        None,
        "query { institutions(order: {field: ROR_STATUS, direction: DESC}) { rorStatus } }",
        None,
    )
    .await;
    support::assert_no_errors(&response);
    let statuses: Vec<Value> = response["data"]["institutions"]
        .as_array()
        .expect("institutions")
        .iter()
        .map(|row| row["rorStatus"].clone())
        .collect();
    assert_eq!(
        statuses,
        vec![json!("WITHDRAWN"), json!("INACTIVE"), json!("ACTIVE")]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn constraint_violations_return_their_messages_and_write_no_history() {
    let (_guard, pool) = setup();
    let no_ror = id_of(&create(&pool, ror_sync("checks-sync"), "No ror", None).await);
    let active = id_of(&create(&pool, ror_sync("checks-sync"), "Active", Some(ROR_A)).await);
    let history_before = count(&pool, "SELECT count(*) AS n FROM institution_history");

    for (data, message) in [
        (
            format!(
                "institutionId: \"{no_ror}\", institutionName: \"No ror\", rorStatus: INACTIVE"
            ),
            "An institution without a ROR ID must be ACTIVE with no successor.",
        ),
        (
            format!(
                "institutionId: \"{active}\", institutionName: \"Active\", ror: \"{ROR_A}\", \
                 rorSuccessor: \"{ROR_B}\""
            ),
            "An ACTIVE institution cannot have a ROR successor.",
        ),
        (
            format!(
                "institutionId: \"{active}\", institutionName: \"Active\", ror: \"{ROR_A}\", \
                 rorStatus: WITHDRAWN, rorSuccessor: \"{ROR_A}\""
            ),
            "An institution cannot be its own ROR successor.",
        ),
    ] {
        let response = update_literal(&pool, ror_sync("checks-sync"), &data).await;
        assert_eq!(
            support::first_error_message(&response),
            Some(message),
            "{data}"
        );
    }
    assert_eq!(
        count(&pool, "SELECT count(*) AS n FROM institution_history"),
        history_before
    );
    assert_eq!(
        state(&fetch(&pool, &active).await),
        (json!("ACTIVE"), Value::Null)
    );
    let _ = Uuid::parse_str(&active).expect("an institution ID");
}
