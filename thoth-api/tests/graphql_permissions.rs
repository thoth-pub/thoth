#![cfg(feature = "backend")]

mod support;

use std::sync::Arc;

use serde_json::json;
use thoth_api::db::PgPool;
use uuid::Uuid;

struct CreatedImprint {
    imprint_id: Uuid,
    imprint_name: String,
}

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

async fn create_imprint_record(
    pool: Arc<PgPool>,
    publisher_id: Uuid,
    s3_bucket: Option<&str>,
    cdn_domain: Option<&str>,
    cloudfront_dist_id: Option<&str>,
) -> CreatedImprint {
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
            "s3Bucket": s3_bucket,
            "cdnDomain": cdn_domain,
            "cloudfrontDistId": cloudfront_dist_id,
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

    CreatedImprint {
        imprint_id: Uuid::parse_str(id).expect("Invalid imprintId"),
        imprint_name: response
            .pointer("/data/createImprint/imprintName")
            .and_then(|v| v.as_str())
            .expect("Missing imprintName in response")
            .to_string(),
    }
}

async fn create_imprint(pool: Arc<PgPool>, publisher_id: Uuid) -> Uuid {
    create_imprint_record(pool, publisher_id, None, None, None)
        .await
        .imprint_id
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

#[tokio::test(flavor = "current_thread")]
async fn test_create_imprint_storage_fields_requires_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;

    let query = r#"
mutation($data: NewImprint!) {
  createImprint(data: $data) {
    imprintId
  }
}
"#;

    let variables = json!({
        "data": {
            "publisherId": publisher_id,
            "imprintName": format!("Restricted Imprint {}", Uuid::new_v4()),
            "s3Bucket": "bucket-create",
            "cdnDomain": "create.example.org",
            "cloudfrontDistId": "dist-create",
        }
    });
    let response = support::execute_graphql(
        pool,
        Some(support::user_with_roles(
            "user-1",
            &[("PUBLISHER_USER", org_id.as_str())],
        )),
        query,
        Some(variables),
    )
    .await;

    support::assert_no_access(&response);
}

#[tokio::test(flavor = "current_thread")]
async fn test_imprint_storage_fields_superuser_query_returns_nulls_when_unset() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint = create_imprint_record(pool.clone(), publisher_id, None, None, None).await;

    let query = r#"
query($imprintId: Uuid!) {
  imprint(imprintId: $imprintId) {
    imprintId
    s3Bucket
    cdnDomain
    cloudfrontDistId
  }
}
"#;

    let variables = json!({ "imprintId": imprint.imprint_id });
    let response = support::execute_graphql(
        pool,
        Some(support::superuser("superuser-1")),
        query,
        Some(variables),
    )
    .await;
    support::assert_no_errors(&response);

    let imprint_data = response
        .pointer("/data/imprint")
        .expect("Missing imprint in response");
    assert!(imprint_data["s3Bucket"].is_null());
    assert!(imprint_data["cdnDomain"].is_null());
    assert!(imprint_data["cloudfrontDistId"].is_null());
}

#[tokio::test(flavor = "current_thread")]
async fn test_imprint_storage_fields_require_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint = create_imprint_record(
        pool.clone(),
        publisher_id,
        Some("bucket-example"),
        Some("cdn.example.org"),
        Some("dist-example"),
    )
    .await;

    let query = r#"
query($imprintId: Uuid!) {
  imprint(imprintId: $imprintId) {
    imprintId
    s3Bucket
    cdnDomain
    cloudfrontDistId
  }
}
"#;

    let imprint_id = imprint.imprint_id.to_string();
    let variables = json!({ "imprintId": imprint.imprint_id });
    let cases = vec![
        (
            "publisher admin",
            Some(support::user_with_roles(
                "user-admin",
                &[("PUBLISHER_ADMIN", org_id.as_str())],
            )),
        ),
        (
            "publisher user",
            Some(support::user_with_roles(
                "user-basic",
                &[("PUBLISHER_USER", org_id.as_str())],
            )),
        ),
        ("anonymous", None),
    ];

    for (label, user) in cases {
        let response =
            support::execute_graphql(pool.clone(), user, query, Some(variables.clone())).await;
        support::assert_no_access(&response);

        assert_eq!(
            response
                .pointer("/data/imprint/imprintId")
                .and_then(|v| v.as_str()),
            Some(imprint_id.as_str()),
            "Expected imprint data to remain visible for {label}"
        );
        assert!(
            response
                .pointer("/data/imprint/s3Bucket")
                .is_some_and(|v| v.is_null()),
            "Expected null s3Bucket for {label}, got: {response:?}"
        );
        assert!(
            response
                .pointer("/data/imprint/cdnDomain")
                .is_some_and(|v| v.is_null()),
            "Expected null cdnDomain for {label}, got: {response:?}"
        );
        assert!(
            response
                .pointer("/data/imprint/cloudfrontDistId")
                .is_some_and(|v| v.is_null()),
            "Expected null cloudfrontDistId for {label}, got: {response:?}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_imprint_query_without_storage_fields_still_works_for_non_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint = create_imprint_record(pool.clone(), publisher_id, None, None, None).await;

    let query = r#"
query($imprintId: Uuid!) {
  imprint(imprintId: $imprintId) {
    imprintId
    imprintName
  }
}
"#;

    let variables = json!({ "imprintId": imprint.imprint_id });
    let user = support::user_with_roles("user-1", &[("PUBLISHER_USER", org_id.as_str())]);
    let response = support::execute_graphql(pool, Some(user), query, Some(variables)).await;

    support::assert_no_errors(&response);
    let imprint_id = imprint.imprint_id.to_string();
    assert_eq!(
        response
            .pointer("/data/imprint/imprintId")
            .and_then(|v| v.as_str()),
        Some(imprint_id.as_str())
    );
    assert_eq!(
        response
            .pointer("/data/imprint/imprintName")
            .and_then(|v| v.as_str()),
        Some(imprint.imprint_name.as_str())
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_imprint_storage_fields_requires_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint = create_imprint_record(pool.clone(), publisher_id, None, None, None).await;

    let update_query = r#"
mutation($data: PatchImprint!) {
  updateImprint(data: $data) {
    imprintId
  }
}
"#;

    let variables = json!({
        "data": {
            "imprintId": imprint.imprint_id,
            "publisherId": publisher_id,
            "imprintName": imprint.imprint_name,
            "s3Bucket": "bucket-restricted",
            "cdnDomain": "restricted.example.org",
            "cloudfrontDistId": "dist-restricted",
        }
    });
    let response = support::execute_graphql(
        pool,
        Some(support::user_with_roles(
            "user-admin",
            &[("PUBLISHER_ADMIN", org_id.as_str())],
        )),
        update_query,
        Some(variables),
    )
    .await;

    support::assert_no_access(&response);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_imprint_storage_fields_round_trip_for_superuser() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(&pool).expect("Failed to reset DB");

    let org_id = format!("org-{}", Uuid::new_v4());
    let publisher_id = create_publisher(pool.clone(), org_id.as_str()).await;
    let imprint = create_imprint_record(pool.clone(), publisher_id, None, None, None).await;

    let update_query = r#"
mutation($data: PatchImprint!) {
  updateImprint(data: $data) {
    imprintId
  }
}
"#;

    let imprint_name = imprint.imprint_name.clone();
    let variables = json!({
        "data": {
            "imprintId": imprint.imprint_id,
            "publisherId": publisher_id,
            "imprintName": imprint_name,
            "s3Bucket": "bucket-roundtrip",
            "cdnDomain": "roundtrip.example.org",
            "cloudfrontDistId": "dist-roundtrip",
        }
    });
    let update_response = support::execute_graphql(
        pool.clone(),
        Some(support::superuser("superuser-1")),
        update_query,
        Some(variables),
    )
    .await;
    support::assert_no_errors(&update_response);

    let read_query = r#"
query($imprintId: Uuid!) {
  imprint(imprintId: $imprintId) {
    s3Bucket
    cdnDomain
    cloudfrontDistId
  }
}
"#;

    let read_variables = json!({ "imprintId": imprint.imprint_id });
    let read_response = support::execute_graphql(
        pool,
        Some(support::superuser("superuser-1")),
        read_query,
        Some(read_variables),
    )
    .await;
    support::assert_no_errors(&read_response);

    assert_eq!(
        read_response
            .pointer("/data/imprint/s3Bucket")
            .and_then(|v| v.as_str()),
        Some("bucket-roundtrip")
    );
    assert_eq!(
        read_response
            .pointer("/data/imprint/cdnDomain")
            .and_then(|v| v.as_str()),
        Some("roundtrip.example.org")
    );
    assert_eq!(
        read_response
            .pointer("/data/imprint/cloudfrontDistId")
            .and_then(|v| v.as_str()),
        Some("dist-roundtrip")
    );
}
