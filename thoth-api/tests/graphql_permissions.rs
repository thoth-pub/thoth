#![cfg(feature = "backend")]

mod support;

use std::sync::Arc;

use serde_json::json;
use thoth_api::db::PgPool;
use uuid::Uuid;

async fn create_publisher(pool: Arc<PgPool>, org_id: &str) -> Uuid {
    let name = format!("Test Publisher {}", Uuid::new_v4());
    let query = r#"
mutation($data: NewPublisher!) {
  createPublisher(data: $data) {
    publisherId
    publisherName
    zitadelId
  }
}
"#;

    let variables = json!({
        "data": {
            "publisherName": name,
            "zitadelId": org_id,
        }
    });

    let response = support::execute_graphql(
        pool,
        Some(support::superuser("superuser-1")),
        query,
        Some(variables),
    )
    .await;
    support::assert_no_errors(&response);

    let id = response
        .pointer("/data/createPublisher/publisherId")
        .and_then(|v| v.as_str())
        .expect("Missing publisherId in response");
    Uuid::parse_str(id).expect("Invalid publisherId")
}

async fn create_imprint(pool: Arc<PgPool>, publisher_id: Uuid) -> Uuid {
    let name = format!("Test Imprint {}", Uuid::new_v4());
    let query = r#"
mutation($data: NewImprint!) {
  createImprint(data: $data) {
    imprintId
    imprintName
  }
}
"#;

    let variables = json!({
        "data": {
            "publisherId": publisher_id,
            "imprintName": name,
        }
    });

    let response = support::execute_graphql(
        pool,
        Some(support::superuser("superuser-1")),
        query,
        Some(variables),
    )
    .await;
    support::assert_no_errors(&response);

    let id = response
        .pointer("/data/createImprint/imprintId")
        .and_then(|v| v.as_str())
        .expect("Missing imprintId in response");
    Uuid::parse_str(id).expect("Invalid imprintId")
}

async fn create_work(pool: Arc<PgPool>, imprint_id: Uuid) -> Uuid {
    let query = r#"
mutation($data: NewWork!) {
  createWork(data: $data) {
    workId
    workStatus
  }
}
"#;

    let variables = json!({
        "data": {
            "workType": "MONOGRAPH",
            "workStatus": "FORTHCOMING",
            "edition": 1,
            "imprintId": imprint_id,
        }
    });

    let response = support::execute_graphql(
        pool,
        Some(support::superuser("superuser-1")),
        query,
        Some(variables),
    )
    .await;
    support::assert_no_errors(&response);

    let id = response
        .pointer("/data/createWork/workId")
        .and_then(|v| v.as_str())
        .expect("Missing workId in response");
    Uuid::parse_str(id).expect("Invalid workId")
}

#[tokio::test(flavor = "current_thread")]
async fn test_me_requires_auth() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let query = "query { me { userId } }";
    let response = support::execute_graphql(pool, None, query, None).await;

    support::assert_no_access(&response);
}

#[tokio::test(flavor = "current_thread")]
async fn test_me_publisher_contexts() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let _publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;

    let user = support::user_with_roles(
        "user-1",
        &[
            ("PUBLISHER_ADMIN", org_id.as_str()),
            ("WORK_LIFECYCLE", org_id.as_str()),
        ],
    );

    let query = r#"
query {
  me {
    userId
    isSuperuser
    publisherContexts {
      publisher { publisherId publisherName zitadelId }
      permissions { publisherAdmin workLifecycle cdnWrite }
    }
  }
}
"#;

    let response = support::execute_graphql(pool, Some(user), query, None).await;
    support::assert_no_errors(&response);

    let contexts = response
        .pointer("/data/me/publisherContexts")
        .and_then(|v| v.as_array())
        .expect("Missing publisherContexts");
    assert_eq!(contexts.len(), 1);

    let permissions = &contexts[0]["permissions"];
    assert_eq!(permissions["publisherAdmin"].as_bool(), Some(true));
    assert_eq!(permissions["workLifecycle"].as_bool(), Some(true));
    assert_eq!(permissions["cdnWrite"].as_bool(), Some(false));

    let publisher = &contexts[0]["publisher"];
    let zitadel_id = publisher["zitadelId"].as_str().expect("Missing zitadelId");
    assert_eq!(zitadel_id, org_id.as_str());
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_publisher_requires_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let query = r#"
mutation($data: NewPublisher!) {
  createPublisher(data: $data) {
    publisherId
  }
}
"#;

    let variables = json!({
        "data": {
            "publisherName": "Nope Publisher",
            "zitadelId": "org-1",
        }
    });

    let user = support::user_with_roles("user-1", &[]);
    let response = support::execute_graphql(pool, Some(user), query, Some(variables)).await;

    support::assert_no_access(&response);
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_work_allows_publisher_user() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint_id = create_imprint(pool.clone(), publisher_id).await;

    let query = r#"
mutation($data: NewWork!) {
  createWork(data: $data) {
    workId
    workStatus
    imprintId
  }
}
"#;

    let variables = json!({
        "data": {
            "workType": "MONOGRAPH",
            "workStatus": "FORTHCOMING",
            "edition": 1,
            "imprintId": imprint_id,
        }
    });

    let user = support::user_with_roles("user-1", &[("PUBLISHER_USER", org_id.as_str())]);
    let response = support::execute_graphql(pool, Some(user), query, Some(variables)).await;
    support::assert_no_errors(&response);

    let imprint = response
        .pointer("/data/createWork/imprintId")
        .and_then(|v| v.as_str())
        .expect("Missing imprintId");
    assert_eq!(imprint, imprint_id.to_string());

    let status = response
        .pointer("/data/createWork/workStatus")
        .and_then(|v| v.as_str())
        .expect("Missing workStatus");
    assert_eq!(status, "FORTHCOMING");
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_work_requires_work_lifecycle() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint_id = create_imprint(pool.clone(), publisher_id).await;
    let work_id = create_work(pool.clone(), imprint_id).await;

    let query = r#"
mutation($data: PatchWork!) {
  updateWork(data: $data) {
    workId
    workStatus
    publicationDate
  }
}
"#;

    let variables = json!({
        "data": {
            "workId": work_id,
            "workType": "MONOGRAPH",
            "workStatus": "ACTIVE",
            "edition": 1,
            "imprintId": imprint_id,
            "publicationDate": "2020-01-01"
        }
    });

    let user = support::user_with_roles("user-1", &[("PUBLISHER_USER", org_id.as_str())]);
    let response = support::execute_graphql(pool, Some(user), query, Some(variables)).await;

    support::assert_no_access(&response);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_work_allows_work_lifecycle() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint_id = create_imprint(pool.clone(), publisher_id).await;
    let work_id = create_work(pool.clone(), imprint_id).await;

    let query = r#"
mutation($data: PatchWork!) {
  updateWork(data: $data) {
    workId
    workStatus
    publicationDate
  }
}
"#;

    let variables = json!({
        "data": {
            "workId": work_id,
            "workType": "MONOGRAPH",
            "workStatus": "ACTIVE",
            "edition": 1,
            "imprintId": imprint_id,
            "publicationDate": "2020-01-01"
        }
    });

    let user = support::user_with_roles(
        "user-1",
        &[
            ("PUBLISHER_USER", org_id.as_str()),
            ("WORK_LIFECYCLE", org_id.as_str()),
        ],
    );
    let response = support::execute_graphql(pool, Some(user), query, Some(variables)).await;
    support::assert_no_errors(&response);

    let status = response
        .pointer("/data/updateWork/workStatus")
        .and_then(|v| v.as_str())
        .expect("Missing workStatus");
    assert_eq!(status, "ACTIVE");

    let publication_date = response
        .pointer("/data/updateWork/publicationDate")
        .and_then(|v| v.as_str())
        .expect("Missing publicationDate");
    assert_eq!(publication_date, "2020-01-01");
}
