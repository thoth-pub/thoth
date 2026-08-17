//! `BE-03` protected-surface authorization, capability-exposure, error-shape
//! and query-count evidence.
//!
//! These tests exercise the real production schema, the real resolvers and the
//! real `RequestLoaders` bundle against a disposable database. Coordinator,
//! audit, concurrency, linked-platform and trigger-cascade evidence lives in
//! `crate::model::publisher_service_configuration::tests`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use diesel::{sql_query, RunQueryDsl};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::dataloader::fixture::{BatchStats, SqlProbe};
use super::dataloader::RequestLoaders;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::distribution_job::DistributionJobCreation;
use crate::model::publisher::{Publisher, PublisherCapability, ThothPackage};
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::publisher_service_configuration::crud::replace_publisher_service_configuration;
use crate::model::publisher_service_configuration::{
    PublisherServiceConfigurationSource, ReplacePublisherServiceConfigurationInput,
    ServiceConfigurationWriteContext,
};
use crate::model::tests::db as test_db;
use crate::model::{Crud, Timestamp};
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

/// The single error object of a denied or failed response, with its machine
/// readable `extensions.type`.
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
    assert_eq!(kind, "NO_ACCESS", "expected a fail-closed denial");
    assert_eq!(message, "Unauthorized");
}

/// A user holding one scoped role for several organisations, or several scoped
/// roles for one organisation.
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

fn org_of(publisher: &Publisher) -> String {
    publisher.zitadel_id.clone().expect("publisher zitadel id")
}

fn read_query(publisher_id: Uuid) -> String {
    format!(
        "{{ publisherServiceConfiguration(publisherId: \"{publisher_id}\") \
           {{ publisher {{ publisherId }} subscriptionPackage effectiveCapabilities \
              enabledDistributionPlatforms {{ platform }} updatedAt }} }}"
    )
}

fn mutation(
    publisher_id: Uuid,
    package: &str,
    platforms: &str,
    expected_updated_at: &Timestamp,
) -> String {
    format!(
        "mutation {{ replacePublisherServiceConfiguration(data: {{ \
           publisherId: \"{publisher_id}\", subscriptionPackage: {package}, \
           enabledDistributionPlatforms: [{platforms}], \
           expectedUpdatedAt: \"{}\" }}) \
           {{ subscriptionPackage effectiveCapabilities updatedAt \
              enabledDistributionPlatforms {{ platform }} }} }}",
        expected_updated_at.to_rfc3339()
    )
}

fn token(pool: &PgPool, publisher_id: Uuid) -> Timestamp {
    Publisher::from_id(pool, &publisher_id)
        .expect("publisher")
        .service_configuration_updated_at
}

/// Commit a configuration change through the canonical coordinator, so a
/// GraphQL-level test can establish a fixture without asserting the write path
/// twice.
fn seed_configuration(
    pool: &PgPool,
    publisher_id: Uuid,
    package: ThothPackage,
    platforms: &[DistributionPlatform],
) {
    replace_publisher_service_configuration(
        pool,
        &ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::SuperuserApi,
            actor: "fixture-superuser",
            // Fixtures seed real activations, so they run with creation `ON`;
            // the switch's own fail-closed evidence is separate.
            job_creation: DistributionJobCreation::On,
        },
        &ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: package,
            enabled_distribution_platforms: platforms.to_vec(),
            expected_updated_at: token(pool, publisher_id),
        },
    )
    .expect("seed configuration");
}

/// A superuser context that permits automatic distribution-job creation.
///
/// `BE-03`'s write-path evidence activates `AutomaticPush` destinations, and
/// `BE-04` refuses to **commit** such an activation while creation is `OFF`
/// (fail-closed, `BE-04` specification section 9.4.2). These tests are about the
/// configuration write path rather than about the switch, so they run with
/// creation `ON` and keep asserting exactly what they asserted before. The
/// switch's own evidence — both positions, and the `OFF` rollback — lives in
/// `BE-04`'s own tests.
fn superuser_writer(pool: Arc<PgPool>, user_id: &str) -> Context {
    test_db::test_context_with_job_creation(
        pool,
        Some(test_db::test_superuser(user_id)),
        DistributionJobCreation::On,
    )
}

fn capabilities_of(value: &JsonValue) -> Vec<String> {
    value["effectiveCapabilities"]
        .as_array()
        .expect("capability array")
        .iter()
        .map(|code| code.as_str().expect("capability code").to_string())
        .collect()
}

fn expected_capabilities(package: ThothPackage) -> Vec<String> {
    package
        .capabilities()
        .iter()
        .map(PublisherCapability::to_string)
        .collect()
}

// --------------------------------------------------------------------------
// Read authorization: every row of specification section 11.1
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn anonymous_callers_cannot_read_the_protected_configuration() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    seed_configuration(
        &pool,
        publisher.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    );
    let schema = create_schema();
    let context = test_db::test_context_anonymous(Arc::clone(&pool));

    let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    assert_unauthorized(&response);

    // Rejected before any publisher load: an unknown publisher is equally
    // unauthorized for an anonymous caller, never `EntityNotFound`.
    let unknown = run(&schema, &context, &read_query(Uuid::new_v4())).await;
    assert_unauthorized(&unknown);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_authenticated_caller_with_no_applicable_role_cannot_read_it() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let schema = create_schema();
    let context = test_db::test_context(Arc::clone(&pool), "no-roles");

    assert_unauthorized(&run(&schema, &context, &read_query(publisher.publisher_id)).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_publisher_user_of_the_target_publisher_reads_it_including_its_capabilities() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    seed_configuration(
        &pool,
        publisher.publisher_id,
        ThothPackage::Obelisk,
        &[DistributionPlatform::Oapen],
    );
    let schema = create_schema();
    let context = test_db::test_context_with_user(
        Arc::clone(&pool),
        user_with("owner", &[(Role::PublisherUser, &org_of(&publisher))]),
    );

    let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    let configuration = data(&response, "publisherServiceConfiguration");
    assert_eq!(configuration["subscriptionPackage"], "OBELISK");
    assert_eq!(
        capabilities_of(configuration),
        expected_capabilities(ThothPackage::Obelisk)
    );
    assert_eq!(
        configuration["enabledDistributionPlatforms"]
            .as_array()
            .expect("platforms")
            .len(),
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_publisher_user_of_another_publisher_cannot_read_it() {
    let (_guard, pool) = test_db::setup_test_db();
    let target = test_db::create_publisher(&pool);
    let other = test_db::create_publisher(&pool);
    seed_configuration(
        &pool,
        target.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    );
    let schema = create_schema();
    let context = test_db::test_context_with_user(
        Arc::clone(&pool),
        user_with("outsider", &[(Role::PublisherUser, &org_of(&other))]),
    );

    let response = run(&schema, &context, &read_query(target.publisher_id)).await;
    assert_unauthorized(&response);
    // Specifically: the other publisher's capability codes are not readable.
    assert!(response["data"]["publisherServiceConfiguration"].is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn no_other_scoped_role_implies_publisher_user_for_the_protected_read() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let org = org_of(&publisher);
    let schema = create_schema();

    // PUBLISHER_ADMIN, WORK_LIFECYCLE and CDN_WRITE for the target publisher,
    // each without PUBLISHER_USER, alone and in combination.
    let denied = [
        vec![(Role::PublisherAdmin, org.as_str())],
        vec![(Role::WorkLifecycle, org.as_str())],
        vec![(Role::CdnWrite, org.as_str())],
        vec![
            (Role::PublisherAdmin, org.as_str()),
            (Role::WorkLifecycle, org.as_str()),
            (Role::CdnWrite, org.as_str()),
        ],
    ];
    for roles in denied {
        let context = test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("scoped-but-not-publisher-user", &roles),
        );
        let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
        assert_unauthorized(&response);
    }

    // Adding PUBLISHER_USER, and only that, opens the read.
    let context = test_db::test_context_with_user(
        Arc::clone(&pool),
        user_with(
            "publisher-user",
            &[
                (Role::PublisherAdmin, org.as_str()),
                (Role::PublisherUser, org.as_str()),
            ],
        ),
    );
    let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    assert!(!data(&response, "publisherServiceConfiguration").is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_superuser_reads_any_publishers_configuration_and_capabilities() {
    let (_guard, pool) = test_db::setup_test_db();
    let first = test_db::create_publisher(&pool);
    let second = test_db::create_publisher(&pool);
    seed_configuration(&pool, first.publisher_id, ThothPackage::Pyramid, &[]);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-read"));

    for (publisher_id, package) in [
        (first.publisher_id, ThothPackage::Pyramid),
        (second.publisher_id, ThothPackage::Oasis),
    ] {
        let response = run(&schema, &context, &read_query(publisher_id)).await;
        let configuration = data(&response, "publisherServiceConfiguration");
        assert_eq!(
            capabilities_of(configuration),
            expected_capabilities(package)
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_account_scoped_to_two_publishers_reads_both_and_no_third() {
    let (_guard, pool) = test_db::setup_test_db();
    let first = test_db::create_publisher(&pool);
    let second = test_db::create_publisher(&pool);
    let third = test_db::create_publisher(&pool);
    let schema = create_schema();
    let context = test_db::test_context_with_user(
        Arc::clone(&pool),
        user_with(
            "multi-publisher",
            &[
                (Role::PublisherUser, &org_of(&first)),
                (Role::PublisherUser, &org_of(&second)),
            ],
        ),
    );

    for publisher_id in [first.publisher_id, second.publisher_id] {
        let response = run(&schema, &context, &read_query(publisher_id)).await;
        assert!(!data(&response, "publisherServiceConfiguration").is_null());
    }
    assert_unauthorized(&run(&schema, &context, &read_query(third.publisher_id)).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_publisher_with_a_null_zitadel_id_fails_closed_for_every_non_superuser() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let linked = test_db::create_publisher(&pool);
    {
        let mut connection = pool.get().expect("connection");
        sql_query(format!(
            "UPDATE publisher SET zitadel_id = NULL WHERE publisher_id = '{}'",
            publisher.publisher_id
        ))
        .execute(&mut connection)
        .expect("unlink publisher");
    }
    let schema = create_schema();

    // A caller holding PUBLISHER_USER for a real organisation still cannot read
    // an unlinked publisher, because the publisher resolves to no organisation.
    let context = test_db::test_context_with_user(
        Arc::clone(&pool),
        user_with(
            "linked-elsewhere",
            &[(Role::PublisherUser, &org_of(&linked))],
        ),
    );
    assert_unauthorized(&run(&schema, &context, &read_query(publisher.publisher_id)).await);

    let superuser =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-null"));
    let response = run(&schema, &superuser, &read_query(publisher.publisher_id)).await;
    assert!(!data(&response, "publisherServiceConfiguration").is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_unknown_publisher_is_not_found_for_an_authenticated_caller() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-missing"));

    let response = run(&schema, &context, &read_query(Uuid::new_v4())).await;
    let (message, kind) = only_error(&response);
    assert_eq!(kind, "INTERNAL_ERROR", "EntityNotFound keeps its mapping");
    assert_eq!(message, "No record was found for the given ID.");
}

// --------------------------------------------------------------------------
// Write and report authorization
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn only_a_superuser_may_replace_the_configuration() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let other = test_db::create_publisher(&pool);
    let org = org_of(&publisher);
    let schema = create_schema();
    let current = token(&pool, publisher.publisher_id);
    let query = mutation(publisher.publisher_id, "SPHINX", "ZENODO", &current);

    let denied_contexts = vec![
        test_db::test_context_anonymous(Arc::clone(&pool)),
        test_db::test_context(Arc::clone(&pool), "authenticated-no-roles"),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("owner", &[(Role::PublisherUser, org.as_str())]),
        ),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("admin", &[(Role::PublisherAdmin, org.as_str())]),
        ),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("lifecycle", &[(Role::WorkLifecycle, org.as_str())]),
        ),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("cdn", &[(Role::CdnWrite, org.as_str())]),
        ),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("other-publisher", &[(Role::PublisherUser, &org_of(&other))]),
        ),
    ];
    for context in &denied_contexts {
        assert_unauthorized(&run(&schema, context, &query).await);
    }
    // Every denial was decided before the database was touched.
    assert_eq!(token(&pool, publisher.publisher_id), current);

    let superuser = superuser_writer(Arc::clone(&pool), "su-write");
    let response = run(&schema, &superuser, &query).await;
    let configuration = data(&response, "replacePublisherServiceConfiguration");
    assert_eq!(configuration["subscriptionPackage"], "SPHINX");
    assert!(token(&pool, publisher.publisher_id) > current);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_staff_report_and_its_count_are_superuser_only() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let other = test_db::create_publisher(&pool);
    let schema = create_schema();

    let report = "{ publisherServiceConfigurations { configuration { subscriptionPackage } \
                    lastChange { actor source changedAt } } }";
    let count = "{ publisherServiceConfigurationCount }";

    let denied = vec![
        test_db::test_context_anonymous(Arc::clone(&pool)),
        test_db::test_context(Arc::clone(&pool), "authenticated"),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("owner", &[(Role::PublisherUser, &org_of(&publisher))]),
        ),
        test_db::test_context_with_user(
            Arc::clone(&pool),
            user_with("other", &[(Role::PublisherUser, &org_of(&other))]),
        ),
    ];
    for context in &denied {
        assert_unauthorized(&run(&schema, context, report).await);
        assert_unauthorized(&run(&schema, context, count).await);
    }

    let superuser =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-report"));
    assert_eq!(
        data(
            &run(&schema, &superuser, report).await,
            "publisherServiceConfigurations"
        )
        .as_array()
        .expect("report")
        .len(),
        2
    );
    assert_eq!(
        data(
            &run(&schema, &superuser, count).await,
            "publisherServiceConfigurationCount"
        ),
        2
    );
}

// --------------------------------------------------------------------------
// Effective capabilities
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_package_reports_exactly_its_canonical_capability_sequence() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-caps"));

    for package in [
        ThothPackage::Oasis,
        ThothPackage::Obelisk,
        ThothPackage::Sphinx,
        ThothPackage::Pyramid,
    ] {
        let publisher = test_db::create_publisher(&pool);
        seed_configuration(&pool, publisher.publisher_id, package, &[]);

        let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
        let configuration = data(&response, "publisherServiceConfiguration");

        // Exact sequence, not set equality: a later sort, dedup or reorder
        // fails this.
        assert_eq!(
            capabilities_of(configuration),
            expected_capabilities(package),
            "capabilities for {package}"
        );
        // The package reported in the same response agrees with them, because
        // both are read from the same publisher row.
        assert_eq!(configuration["subscriptionPackage"], package.to_string());

        // Repeated reads return an identical sequence.
        let again = run(&schema, &context, &read_query(publisher.publisher_id)).await;
        assert_eq!(
            capabilities_of(data(&again, "publisherServiceConfiguration")),
            capabilities_of(configuration)
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn oasis_reports_an_empty_capability_list_never_null() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-oasis"));

    let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    let configuration = data(&response, "publisherServiceConfiguration");
    assert_eq!(configuration["subscriptionPackage"], "OASIS");
    assert!(configuration["effectiveCapabilities"].is_array());
    assert!(capabilities_of(configuration).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_package_upgrade_and_downgrade_change_capabilities_with_no_separate_write() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-upgrade"));

    // Upgrade OASIS -> SPHINX.
    let upgrade = run(
        &schema,
        &context,
        &mutation(
            publisher.publisher_id,
            "SPHINX",
            "",
            &token(&pool, publisher.publisher_id),
        ),
    )
    .await;
    let returned = data(&upgrade, "replacePublisherServiceConfiguration");
    assert_eq!(
        capabilities_of(returned),
        expected_capabilities(ThothPackage::Sphinx),
        "the mutation's own returned configuration reflects the new package"
    );
    let queried = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    assert_eq!(
        capabilities_of(data(&queried, "publisherServiceConfiguration")),
        expected_capabilities(ThothPackage::Sphinx)
    );

    // Downgrade SPHINX -> OBELISK.
    let downgrade = run(
        &schema,
        &context,
        &mutation(
            publisher.publisher_id,
            "OBELISK",
            "",
            &token(&pool, publisher.publisher_id),
        ),
    )
    .await;
    assert_eq!(
        capabilities_of(data(&downgrade, "replacePublisherServiceConfiguration")),
        expected_capabilities(ThothPackage::Obelisk)
    );
    let queried = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    assert_eq!(
        capabilities_of(data(&queried, "publisherServiceConfiguration")),
        expected_capabilities(ThothPackage::Obelisk)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_platform_only_change_leaves_effective_capabilities_unchanged() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    seed_configuration(&pool, publisher.publisher_id, ThothPackage::Obelisk, &[]);
    let schema = create_schema();
    let context = superuser_writer(Arc::clone(&pool), "su-platform");

    let response = run(
        &schema,
        &context,
        &mutation(
            publisher.publisher_id,
            "OBELISK",
            "OAPEN",
            &token(&pool, publisher.publisher_id),
        ),
    )
    .await;
    let returned = data(&response, "replacePublisherServiceConfiguration");
    assert_eq!(
        capabilities_of(returned),
        expected_capabilities(ThothPackage::Obelisk)
    );
    assert_eq!(
        returned["enabledDistributionPlatforms"]
            .as_array()
            .expect("platforms")
            .len(),
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn no_anonymous_operation_can_select_a_capability_or_package_value() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    seed_configuration(&pool, publisher.publisher_id, ThothPackage::Pyramid, &[]);
    let schema = create_schema();
    let anonymous = test_db::test_context_anonymous(Arc::clone(&pool));

    // Schema validation rejects the fields outright on the public type.
    for query in [
        "{ publishers(limit: 10) { publisherId effectiveCapabilities } }",
        "{ publishers(limit: 10) { publisherId subscriptionPackage } }",
        "{ publisher(publisherId: \"00000000-0000-0000-0000-000000000001\") { capabilities } }",
    ] {
        let response = run(&schema, &anonymous, query).await;
        assert!(
            response.get("errors").is_some(),
            "public Publisher must not resolve `{query}`"
        );
    }

    // And the protected operation itself is denied.
    assert_unauthorized(&run(&schema, &anonymous, &read_query(publisher.publisher_id)).await);
}

// --------------------------------------------------------------------------
// Error shape
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stale_replacement_returns_the_distinct_machine_readable_error() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stale = token(&pool, publisher.publisher_id);
    seed_configuration(
        &pool,
        publisher.publisher_id,
        ThothPackage::Obelisk,
        &[DistributionPlatform::Zenodo],
    );
    let committed = token(&pool, publisher.publisher_id);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-stale"));

    let response = run(
        &schema,
        &context,
        &mutation(publisher.publisher_id, "SPHINX", "CROSSREF", &stale),
    )
    .await;

    let (message, kind) = only_error(&response);
    assert_eq!(kind, "STALE_SERVICE_CONFIGURATION");
    assert!(message.contains("changed since it was read"));
    // The current token is deliberately not disclosed to a caller that just
    // failed a version check.
    assert!(!message.contains(&committed.to_rfc3339()));
    // No SQL, table name, column name or driver text.
    for leak in [
        "SELECT",
        "UPDATE",
        "service_configuration_updated_at",
        "publisher_service_configuration_history",
        "publisher_distribution_platform",
        "ERROR:",
        "expected_updated_at",
    ] {
        assert!(
            !message.contains(leak),
            "the stale message must not leak `{leak}`: {message}"
        );
    }
    assert_eq!(token(&pool, publisher.publisher_id), committed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_non_assignable_platform_keeps_its_merged_error_mapping() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let schema = create_schema();
    let context =
        test_db::test_context_with_user(Arc::clone(&pool), test_db::test_superuser("su-jisc"));

    let response = run(
        &schema,
        &context,
        &mutation(
            publisher.publisher_id,
            "OASIS",
            "JISC_NBK",
            &token(&pool, publisher.publisher_id),
        ),
    )
    .await;

    let (message, kind) = only_error(&response);
    assert_eq!(kind, "INTERNAL_ERROR", "unchanged BE-02 mapping");
    assert!(message.contains("JISC_NBK"));
}

// --------------------------------------------------------------------------
// Query efficiency (specification sections 12.3 and 18.8)
// --------------------------------------------------------------------------

/// The assignment statements issued by the DataLoader, as opposed to the
/// report's own publisher-page and latest-change SQL.
fn assignment_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| sql.contains("FROM \"publisher_distribution_platform\""))
        .cloned()
        .collect()
}

fn history_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| sql.contains("FROM \"publisher_service_configuration_history\""))
        .cloned()
        .collect()
}

fn publisher_page_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| sql.contains("FROM \"publisher\"") && sql.contains("LIMIT"))
        .cloned()
        .collect()
}

/// The application-issued `UPDATE` statements against the `publisher` table.
///
/// The publisher row carries the shared `AFTER UPDATE` work-freshness trigger,
/// so this count **is** the number of times that trigger's set-based cascade
/// runs over the publisher's whole catalogue. The trigger's own
/// `UPDATE work ... FROM imprint` is server-side and never appears here, which
/// is why the application-level count is the thing worth asserting.
fn publisher_update_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| sql.contains("UPDATE \"publisher\""))
        .cloned()
        .collect()
}

fn seed_publishers_with_assignment(pool: &PgPool, count: usize) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher (publisher_id, publisher_name) \
         SELECT gen_random_uuid(), 'Report Press ' || lpad(i::text, 5, '0') \
         FROM generate_series(1, {count}) AS i"
    ))
    .execute(&mut connection)
    .expect("seed publishers");
    sql_query(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at) \
         SELECT publisher_id, 'OAPEN', true, gen_random_uuid(), now() FROM publisher",
    )
    .execute(&mut connection)
    .expect("seed assignments");
}

async fn measure_report(page_size: usize) -> (Vec<usize>, Vec<String>) {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    seed_publishers_with_assignment(&ordinary_pool, page_size);

    let probe = SqlProbe::install(&test_db::test_db_url());
    let stats = Arc::new(BatchStats::default());
    let mut context = test_db::test_context_with_user(
        Arc::clone(&probe.pool),
        test_db::test_superuser("su-measure"),
    );
    context.loaders =
        RequestLoaders::for_request_observed(Arc::clone(&probe.pool), Arc::clone(&stats));
    let schema = create_schema();

    probe.start();
    let response = run(
        &schema,
        &context,
        &format!(
            "{{ publisherServiceConfigurations(limit: {page_size}) {{ \
                 configuration {{ subscriptionPackage effectiveCapabilities updatedAt \
                                  enabledDistributionPlatforms {{ platform }} }} \
                 lastChange {{ actor source }} }} }}"
        ),
    )
    .await;
    let captured = probe.captured_statements();

    let rows = data(&response, "publisherServiceConfigurations")
        .as_array()
        .expect("report")
        .clone();
    assert_eq!(rows.len(), page_size);
    for row in &rows {
        assert_eq!(
            row["configuration"]["enabledDistributionPlatforms"]
                .as_array()
                .expect("platforms")
                .len(),
            1,
            "every summary loads its own assignments"
        );
        assert!(row["lastChange"].is_null());
    }

    (stats.batch_sizes(), captured)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_report_statement_count_is_bounded_and_does_not_grow_with_the_page() {
    // The report always issues two set-based statements — one publisher page,
    // one latest-change — whatever the page size. The existing BE-02 assignment
    // loader adds `ceil(N / MAX_BATCH_SIZE)` set-based dispatches, because it is
    // configured with a maximum batch size and chunks larger key sets. Nothing
    // is per-publisher in either part.
    //
    // At 1, 25 and 200 the page fits one loader chunk, so the whole request is
    // three statements. 201 is the first page size that does not, and is
    // asserted separately below.
    for page_size in [1, 25, 200] {
        assert!(page_size <= crate::graphql::dataloader::MAX_BATCH_SIZE);
        let (chunks, captured) = measure_report(page_size).await;

        assert_eq!(
            publisher_page_statements(&captured).len(),
            1,
            "one set-based publisher-page statement at page size {page_size}"
        );
        assert_eq!(
            history_statements(&captured).len(),
            1,
            "one set-based latest-change statement at page size {page_size}"
        );
        let history = &history_statements(&captured)[0];
        assert!(history.contains("DISTINCT ON"), "{history}");
        assert!(history.contains("= ANY"), "{history}");

        let assignment_sql = assignment_statements(&captured);
        assert_eq!(
            assignment_sql.len(),
            1,
            "one set-based assignment statement at page size {page_size}: {assignment_sql:?}"
        );
        assert!(assignment_sql[0].contains("= ANY"));
        assert_eq!(chunks, vec![page_size], "one dispatch chunk");
    }
}

/// The first page size that exceeds one loader batch.
///
/// The accurate statement shape is **two** report statements plus
/// `ceil(page publisher count / MAX_BATCH_SIZE)` assignment-loader dispatches —
/// not a count that is flatly independent of N. At 201 that is `[200, 1]` and
/// therefore two assignment statements, still with no per-publisher SQL loop.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_assignment_loader_chunks_a_page_larger_than_the_maximum_batch() {
    let max = crate::graphql::dataloader::MAX_BATCH_SIZE;
    let page_size = max + 1;
    let (chunks, captured) = measure_report(page_size).await;

    // The report's own two statements do not change.
    assert_eq!(publisher_page_statements(&captured).len(), 1);
    assert_eq!(history_statements(&captured).len(), 1);

    assert_eq!(chunks, vec![max, 1], "expected loader chunks [{max}, 1]");

    let assignment_sql = assignment_statements(&captured);
    let expected_dispatches = page_size.div_ceil(max);
    assert_eq!(expected_dispatches, 2);
    assert_eq!(
        assignment_sql.len(),
        expected_dispatches,
        "expected ceil({page_size} / {max}) = {expected_dispatches} assignment statements: \
         {assignment_sql:?}"
    );
    for sql in &assignment_sql {
        assert!(sql.contains("= ANY"), "each dispatch is set-based: {sql}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_single_publisher_query_issues_one_assignment_statement() {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&ordinary_pool);
    seed_configuration(
        &ordinary_pool,
        publisher.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    );

    let probe = SqlProbe::install(&test_db::test_db_url());
    let stats = Arc::new(BatchStats::default());
    let mut context = test_db::test_context_with_user(
        Arc::clone(&probe.pool),
        test_db::test_superuser("su-single"),
    );
    context.loaders =
        RequestLoaders::for_request_observed(Arc::clone(&probe.pool), Arc::clone(&stats));
    let schema = create_schema();

    probe.start();
    let response = run(&schema, &context, &read_query(publisher.publisher_id)).await;
    let captured = probe.captured_statements();

    assert!(!data(&response, "publisherServiceConfiguration").is_null());
    assert_eq!(assignment_statements(&captured).len(), 1);
    assert_eq!(stats.batch_sizes(), vec![1]);
    // The protected read consults no configuration-history statement.
    assert!(history_statements(&captured).is_empty());
}

/// Every committed configuration change costs **exactly one** publisher
/// `UPDATE`, and every uncommitted one costs zero (specification section 7.3
/// steps 8 and 10).
///
/// This is a cascade-amplification regression, not a style assertion. The
/// publisher row carries the shared `AFTER UPDATE` work-freshness trigger, so
/// each extra publisher `UPDATE` re-runs a set-based cascade across that
/// publisher's entire catalogue. A combined package-and-platform change must
/// therefore cost the same single cascade as a platform-only change.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_committed_change_issues_exactly_one_publisher_update() {
    // (case, package, platforms, expected publisher UPDATEs)
    let cases: [(&str, &str, &str, usize); 6] = [
        ("package-only", "SPHINX", "OAPEN, DOAB", 1),
        ("platform-only", "OASIS", "OAPEN, DOAB, ZENODO", 1),
        ("linked repair", "OASIS", "OAPEN, DOAB", 1),
        ("combined package and platform", "PYRAMID", "ZENODO", 1),
        ("true no-op", "OASIS", "OAPEN, DOAB", 0),
        ("stale", "SPHINX", "ZENODO", 0),
    ];

    for (case, package, platforms, expected_updates) in cases {
        let (_guard, ordinary_pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&ordinary_pool);
        // Captured before the fixture commits, so it is a token this publisher
        // genuinely once had and has since superseded.
        let superseded = token(&ordinary_pool, publisher.publisher_id);
        seed_configuration(
            &ordinary_pool,
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
        );

        // The linked-repair case splits the OAPEN/DOAB group behind the
        // coordinator's back, so the request is a membership no-op that the
        // BE-02 primitive still repairs.
        if case == "linked repair" {
            let mut connection = ordinary_pool.get().expect("connection");
            sql_query(format!(
                "UPDATE publisher_distribution_platform SET enabled = false, \
                 disabled_at = now() WHERE publisher_id = '{}' AND platform = 'DOAB'",
                publisher.publisher_id
            ))
            .execute(&mut connection)
            .expect("split the linked group");
        }

        let probe = SqlProbe::install(&test_db::test_db_url());
        let context = superuser_writer(Arc::clone(&probe.pool), "su-update-count");
        let schema = create_schema();

        let supplied = if case == "stale" {
            superseded
        } else {
            token(&probe.pool, publisher.publisher_id)
        };

        probe.start();
        let response = run(
            &schema,
            &context,
            &mutation(publisher.publisher_id, package, platforms, &supplied),
        )
        .await;
        let captured = probe.captured_statements();

        if case == "stale" {
            let (_, kind) = only_error(&response);
            assert_eq!(kind, "STALE_SERVICE_CONFIGURATION", "{case}");
        } else {
            assert!(
                !data(&response, "replacePublisherServiceConfiguration").is_null(),
                "{case}"
            );
        }

        let updates = publisher_update_statements(&captured);
        assert_eq!(
            updates.len(),
            expected_updates,
            "{case}: expected {expected_updates} publisher UPDATE(s), got {}: {updates:?}",
            updates.len()
        );

        // No per-work application loop in any case.
        let work_statements: Vec<&String> = captured
            .iter()
            .filter(|sql| sql.contains("FROM \"work\"") || sql.contains("UPDATE \"work\""))
            .collect();
        assert!(
            work_statements.is_empty(),
            "{case}: no application-level work statement may exist: {work_statements:?}"
        );
    }
}

/// Disposable-environment write-amplification and lock-footprint measurement
/// (specification section 18.4), driven through the real GraphQL mutation.
///
/// This is **empirical evidence about the shape of the cost, not a production
/// SLA**. It is deliberately not extrapolated to production and no "safe"
/// catalogue size is derived from it. The measured numbers are printed so the
/// implementation report can quote them exactly; run with `--nocapture`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn catalogue_scale_write_amplification_is_measured_in_a_disposable_environment() {
    use crate::schema::{imprint, work};
    use diesel::{ExpressionMethods, QueryDsl};
    use std::time::Instant;

    const TARGET_WORKS: usize = 2_000;
    const CONTROL_WORKS: usize = 250;

    let (_guard, ordinary_pool) = test_db::setup_test_db();

    // One target publisher with a materially larger catalogue spread across two
    // imprints, plus an unrelated publisher's catalogue as a control.
    let target = test_db::create_publisher(&ordinary_pool);
    let first_imprint = test_db::create_imprint(&ordinary_pool, &target);
    let second_imprint = test_db::create_imprint(&ordinary_pool, &target);
    let template = test_db::create_work(&ordinary_pool, &first_imprint);

    let control_publisher = test_db::create_publisher(&ordinary_pool);
    let control_imprint = test_db::create_imprint(&ordinary_pool, &control_publisher);
    let control_template = test_db::create_work(&ordinary_pool, &control_imprint);

    let clone_works = |count: usize, imprints: Vec<Uuid>, template_id: Uuid| {
        let mut connection = ordinary_pool.get().expect("connection");
        let cases: Vec<String> = imprints
            .iter()
            .enumerate()
            .map(|(index, imprint_id)| {
                format!(
                    "WHEN i % {} = {index} THEN '{imprint_id}'::uuid",
                    imprints.len()
                )
            })
            .collect();
        sql_query(format!(
            "INSERT INTO work (work_type, work_status, edition, imprint_id) \
             SELECT t.work_type, t.work_status, t.edition, CASE {} END \
             FROM work t CROSS JOIN generate_series(1, {count}) AS i \
             WHERE t.work_id = '{template_id}'",
            cases.join(" ")
        ))
        .execute(&mut connection)
        .expect("clone works");
    };
    clone_works(
        TARGET_WORKS - 1,
        vec![first_imprint.imprint_id, second_imprint.imprint_id],
        template.work_id,
    );
    clone_works(
        CONTROL_WORKS - 1,
        vec![control_imprint.imprint_id],
        control_template.work_id,
    );

    let count_works = |publisher_id: Uuid| -> i64 {
        let mut connection = ordinary_pool.get().expect("connection");
        work::table
            .inner_join(imprint::table)
            .filter(imprint::publisher_id.eq(publisher_id))
            .count()
            .get_result::<i64>(&mut connection)
            .expect("work count")
    };
    let max_freshness = |publisher_id: Uuid| -> Timestamp {
        let mut connection = ordinary_pool.get().expect("connection");
        work::table
            .inner_join(imprint::table)
            .filter(imprint::publisher_id.eq(publisher_id))
            .select(work::updated_at_with_relations)
            .order(work::updated_at_with_relations.desc())
            .first::<Timestamp>(&mut connection)
            .expect("max freshness")
    };
    let moved_since = |publisher_id: Uuid, threshold: Timestamp| -> i64 {
        let mut connection = ordinary_pool.get().expect("connection");
        work::table
            .inner_join(imprint::table)
            .filter(imprint::publisher_id.eq(publisher_id))
            .filter(work::updated_at_with_relations.gt(threshold))
            .count()
            .get_result::<i64>(&mut connection)
            .expect("moved count")
    };

    let target_work_count = count_works(target.publisher_id);
    let control_work_count = count_works(control_publisher.publisher_id);
    assert_eq!(target_work_count, TARGET_WORKS as i64);
    assert_eq!(control_work_count, CONTROL_WORKS as i64);
    let target_before = max_freshness(target.publisher_id);
    let control_before = max_freshness(control_publisher.publisher_id);

    let probe = SqlProbe::install(&test_db::test_db_url());
    let context = superuser_writer(Arc::clone(&probe.pool), "su-catalogue");
    let schema = create_schema();
    let current = token(&probe.pool, target.publisher_id);

    probe.start();
    let started = Instant::now();
    let response = run(
        &schema,
        &context,
        &mutation(target.publisher_id, "SPHINX", "OAPEN, ZENODO", &current),
    )
    .await;
    let elapsed = started.elapsed();
    let captured = probe.captured_statements();
    assert!(!data(&response, "replacePublisherServiceConfiguration").is_null());

    let target_moved = moved_since(target.publisher_id, target_before);
    let control_moved = moved_since(control_publisher.publisher_id, control_before);

    // The publisher trigger issues one set-based `UPDATE work ... FROM imprint`
    // per publisher row `UPDATE`. This request changes both the package and the
    // platforms, and step 10 commits both in a **single** publisher `UPDATE`, so
    // the cascade runs exactly once over the target's catalogue. Every target
    // work is refreshed; no unrelated work is.
    let publisher_updates = publisher_update_statements(&captured);
    assert_eq!(
        publisher_updates.len(),
        1,
        "a combined package-and-platform change must issue exactly one publisher \
         UPDATE, so the work-freshness cascade runs once: {publisher_updates:?}"
    );
    assert_eq!(target_moved, target_work_count);
    assert_eq!(control_moved, 0);
    assert_eq!(
        max_freshness(control_publisher.publisher_id),
        control_before
    );

    // No per-work application loop: the request issues no statement against the
    // `work` table at all. The work rows are changed by the trigger's own
    // set-based statement, which is server-side and never appears here.
    let work_statements: Vec<&String> = captured
        .iter()
        .filter(|sql| sql.contains("FROM \"work\"") || sql.contains("UPDATE \"work\""))
        .collect();
    assert!(
        work_statements.is_empty(),
        "no application-level work statement may exist: {work_statements:?}"
    );
    assert!(
        captured.len() < 32,
        "the statement count must be small and bounded, got {}: {captured:?}",
        captured.len()
    );

    println!(
        "BE-03 catalogue-scale measurement (disposable environment only):\n  \
         target works: {target_work_count}\n  \
         control works (other publisher): {control_work_count}\n  \
         SQL statements issued by the configuration operation: {}\n  \
         work rows changed by the publisher trigger: {target_moved}\n  \
         unrelated publisher work rows changed: {control_moved}\n  \
         request duration in this disposable environment: {elapsed:?}\n  \
         statements:\n{}",
        captured.len(),
        captured
            .iter()
            .enumerate()
            .map(|(index, sql)| format!("    {}. {sql}", index + 1))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_protected_assignment_resolver_is_loader_first_and_uses_try_load_only() {
    let source = include_str!("model.rs");
    let body = source
        .split_once("pub async fn enabled_distribution_platforms(")
        .expect("protected assignment resolver")
        .1
        .split_once("\n    }\n")
        .expect("resolver body")
        .0;

    assert!(
        body.contains("publisher_distribution_platforms"),
        "the protected field must reuse BE-02's existing assignment loader"
    );
    assert!(
        body.contains(".try_load("),
        "`try_load` is the only approved API"
    );
    assert!(
        !body.contains(".load("),
        "`Loader::load` panics on a missing key"
    );
    assert!(
        !body.contains(".await;\n") || body.matches(".await").count() == 1,
        "no unrelated awaited work may precede the loader key registration"
    );
}

#[test]
fn no_second_assignment_loader_was_introduced() {
    let source = include_str!("dataloader.rs");
    // The batcher inventory is named rather than counted, so an authorized
    // addition is visible as an addition and an unauthorized one still fails.
    // `BE-04` adds exactly three, each with a different key, value and
    // statement; `BE-03` still adds none.
    for batcher in [
        "pub(crate) struct PublisherDistributionPlatformBatcher",
        "pub(crate) struct LatestBackCatalogueJobBatcher",
        "pub(crate) struct DistributionJobTargetBatcher",
        "pub(crate) struct DistributionJobAttemptBatcher",
    ] {
        assert_eq!(
            source.matches(batcher).count(),
            1,
            "`{batcher}` exists once"
        );
    }
    let declared: Vec<&str> = source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("pub(crate) struct ") && line.ends_with("Batcher {"))
        .collect();
    assert_eq!(
        declared.len(),
        4,
        "exactly four batcher structs exist, and no fifth was introduced: {declared:?}"
    );
    let bundle = source
        .split_once("pub(crate) struct RequestLoaders {")
        .expect("the request bundle")
        .1
        .split_once("\n}\n")
        .expect("bundle body")
        .0;
    assert_eq!(
        bundle
            .matches("pub(crate) publisher_distribution_platforms:")
            .count(),
        1,
        "exactly one assignment loader field exists on the request bundle"
    );
    assert!(
        !source.contains("ServiceConfigurationLoader"),
        "BE-03 introduces no loader of its own"
    );
}
