//! `BE-04` authorization-matrix, error-contract, SDL, transaction-order and
//! statement-count evidence.
//!
//! These tests exercise the real production schema, the real resolvers and the
//! real `RequestLoaders` bundle against a disposable database. Database
//! contract, creation, state-machine, concurrency and cancellation evidence
//! lives in `crate::model::distribution_job::tests`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::dataloader::fixture::SqlProbe;
use super::dataloader::{ObservedLoaderStats, RequestLoaders};
use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::distribution_job::crud::claim_distribution_jobs;
use crate::model::distribution_job::{
    ClaimedDistributionJob, DistributionJob, DistributionJobCreation,
};
use crate::model::publisher::{Publisher, ThothPackage};
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::publisher_service_configuration::crud::replace_publisher_service_configuration;
use crate::model::publisher_service_configuration::{
    PublisherServiceConfigurationSource, ReplacePublisherServiceConfigurationInput,
    ServiceConfigurationWriteContext,
};
use crate::model::tests::db as test_db;
use crate::model::{Crud, Timestamp};
use crate::policy::Role;

const WORKER_ID: &str = "zitadel-dissemination-worker-1";

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

fn user_with(user_id: &str, roles: &[Role]) -> IntrospectedUser {
    user_with_scopes(
        user_id,
        &roles
            .iter()
            .map(|role| (*role, "org-1"))
            .collect::<Vec<_>>(),
    )
}

fn user_with_scopes(user_id: &str, roles: &[(Role, &str)]) -> IntrospectedUser {
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
    test_db::test_context_with_job_creation(Arc::clone(pool), user, DistributionJobCreation::On)
}

fn token(pool: &PgPool, publisher_id: Uuid) -> Timestamp {
    Publisher::from_id(pool, &publisher_id)
        .expect("publisher")
        .service_configuration_updated_at
}

/// Commit an activation through the canonical coordinator, with creation `ON`.
fn activate(pool: &PgPool, publisher_id: Uuid, platforms: &[DistributionPlatform]) {
    replace_publisher_service_configuration(
        pool,
        &ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::SuperuserApi,
            actor: "fixture-superuser",
            job_creation: DistributionJobCreation::On,
        },
        &ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: ThothPackage::Sphinx,
            enabled_distribution_platforms: platforms.to_vec(),
            expected_updated_at: token(pool, publisher_id),
        },
    )
    .expect("activation");
}

fn only_job(pool: &PgPool, publisher_id: Uuid) -> DistributionJob {
    let mut connection = pool.get().expect("connection");
    let mut jobs = crate::schema::distribution_job::table
        .filter(crate::schema::distribution_job::publisher_id.eq(publisher_id))
        .load::<DistributionJob>(&mut connection)
        .expect("jobs");
    assert_eq!(jobs.len(), 1);
    jobs.remove(0)
}

/// A publisher with one `PENDING` `ZENODO` job.
fn seeded_job(pool: &PgPool) -> (Publisher, DistributionJob) {
    let publisher = test_db::create_publisher(pool);
    activate(
        pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let job = only_job(pool, publisher.publisher_id);
    (publisher, job)
}

fn claim_directly(pool: &PgPool) -> ClaimedDistributionJob {
    claim_distribution_jobs(pool, WORKER_ID, 1, 900, &[])
        .expect("claim")
        .remove(0)
}

// --------------------------------------------------------------------------
// Operation strings
// --------------------------------------------------------------------------

const CLAIM: &str = "mutation { claimDistributionJobs(data: { limit: 5 }) \
                       { claimToken attemptNumber leaseExpiresAt \
                         job { distributionJobId status attemptCount \
                               targets { platform } attempts { attemptNumber result } } } }";

fn complete(job_id: Uuid, claim_token: Uuid) -> String {
    format!(
        "mutation {{ completeDistributionJob(data: {{ distributionJobId: \"{job_id}\", \
           claimToken: \"{claim_token}\" }}) {{ distributionJobId status completedAt }} }}"
    )
}

fn fail(job_id: Uuid, claim_token: Uuid, error_code: &str, retryable: bool) -> String {
    format!(
        "mutation {{ failDistributionJob(data: {{ distributionJobId: \"{job_id}\", \
           claimToken: \"{claim_token}\", errorCode: \"{error_code}\", \
           errorDetail: \"a bounded description\", retryable: {retryable} }}) \
           {{ distributionJobId status attemptCount lastErrorCode lastErrorDetail }} }}"
    )
}

fn cancel(job_id: Uuid) -> String {
    format!(
        "mutation {{ cancelDistributionJob(data: {{ distributionJobId: \"{job_id}\" }}) \
           {{ distributionJobId status cancellationReason }} }}"
    )
}

const REPORT: &str = "{ publisherServiceConfigurations { \
                         configuration { publisher { publisherId } } \
                         latestBackCatalogueJob { distributionJobId status } } }";

// ==========================================================================
// 25.11  The complete section 15.2 authorization matrix
// ==========================================================================

/// Every caller of the section 15.2 matrix, in matrix order.
///
/// The tuple is `(label, user, may_claim_complete_fail, may_cancel, may_report)`.
fn matrix(org: &str) -> Vec<(&'static str, Option<IntrospectedUser>, bool, bool, bool)> {
    vec![
        ("anonymous", None, false, false, false),
        (
            "authenticated, no applicable role",
            Some(user_with("no-roles", &[])),
            false,
            false,
            false,
        ),
        (
            "PUBLISHER_USER for the target publisher",
            Some(user_with_scopes("owner", &[(Role::PublisherUser, org)])),
            false,
            false,
            false,
        ),
        (
            "PUBLISHER_USER for another publisher",
            Some(user_with_scopes(
                "other-owner",
                &[(Role::PublisherUser, "org-elsewhere")],
            )),
            false,
            false,
            false,
        ),
        (
            "PUBLISHER_ADMIN without PUBLISHER_USER",
            Some(user_with_scopes("admin", &[(Role::PublisherAdmin, org)])),
            false,
            false,
            false,
        ),
        (
            "WORK_LIFECYCLE without PUBLISHER_USER",
            Some(user_with_scopes("lifecycle", &[(Role::WorkLifecycle, org)])),
            false,
            false,
            false,
        ),
        (
            "CDN_WRITE without PUBLISHER_USER",
            Some(user_with_scopes("cdn", &[(Role::CdnWrite, org)])),
            false,
            false,
            false,
        ),
        (
            "SUPERUSER without DISSEMINATION_WORKER",
            Some(user_with("superuser", &[Role::Superuser])),
            false,
            true,
            true,
        ),
        (
            "DISSEMINATION_WORKER only",
            Some(user_with("worker", &[Role::DisseminationWorker])),
            true,
            false,
            false,
        ),
        (
            "SUPERUSER + DISSEMINATION_WORKER",
            Some(user_with(
                "superworker",
                &[Role::Superuser, Role::DisseminationWorker],
            )),
            true,
            true,
            true,
        ),
        (
            "invalid, expired or unintrospectable token",
            None,
            false,
            false,
            false,
        ),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_claim() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();

    for (label, user, may_work, _, _) in matrix("org-1") {
        test_db::reset_db(&pool).expect("reset");
        let publisher = test_db::create_publisher(&pool);
        let org = publisher.zitadel_id.clone().expect("org");
        // Re-scope the publisher-scoped rows onto this publisher's real org, so
        // "for the target publisher" is genuinely that.
        let user = user.map(|user| rescope(user, &org));
        activate(
            &pool,
            publisher.publisher_id,
            &[DistributionPlatform::Zenodo],
        );

        let context = context_for(&pool, user);
        let response = run(&schema, &context, CLAIM).await;

        if may_work {
            let claimed = data(&response, "claimDistributionJobs");
            assert_eq!(claimed.as_array().expect("array").len(), 1, "{label}");
        } else {
            assert_unauthorized(&response);
            // Denied before the database was touched: nothing was claimed.
            assert_eq!(
                only_job(&pool, publisher.publisher_id).status.to_string(),
                "PENDING",
                "{label} must be denied before any state change"
            );
        }
    }
}

/// Move every scoped role of `user` onto `org`.
fn rescope(user: IntrospectedUser, org: &str) -> IntrospectedUser {
    let project_roles = user.project_roles.map(|roles| {
        roles
            .into_iter()
            .map(|(key, scoped)| {
                let rescoped = scoped
                    .into_iter()
                    .map(|(existing, label)| {
                        let key = if existing == "org-elsewhere" {
                            existing
                        } else {
                            org.to_string()
                        };
                        (key, label)
                    })
                    .collect();
                (key, rescoped)
            })
            .collect()
    });
    IntrospectedUser {
        project_roles,
        ..user
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_complete_and_fail() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();

    // The matrix is rebuilt per operation because `IntrospectedUser` is not
    // `Clone`.
    for operation in ["complete", "fail"] {
        for (label, user, may_work, _, _) in matrix("org-1") {
            test_db::reset_db(&pool).expect("reset");
            let publisher = test_db::create_publisher(&pool);
            let org = publisher.zitadel_id.clone().expect("org");
            let user = user.map(|user| rescope(user, &org));
            activate(
                &pool,
                publisher.publisher_id,
                &[DistributionPlatform::Zenodo],
            );
            let job = only_job(&pool, publisher.publisher_id);
            let claim = claim_directly(&pool);

            let query = if operation == "complete" {
                complete(job.distribution_job_id, claim.claim_token)
            } else {
                fail(
                    job.distribution_job_id,
                    claim.claim_token,
                    "TRANSIENT",
                    true,
                )
            };
            let context = context_for(&pool, user);
            let response = run(&schema, &context, &query).await;

            if may_work {
                let field = if operation == "complete" {
                    "completeDistributionJob"
                } else {
                    "failDistributionJob"
                };
                assert!(!data(&response, field).is_null(), "{label} / {operation}");
            } else {
                assert_unauthorized(&response);
                assert_eq!(
                    only_job(&pool, publisher.publisher_id).status.to_string(),
                    "RUNNING",
                    "{label} / {operation} must be denied before any state change"
                );
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_cancel() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();

    for (label, user, _, may_cancel, _) in matrix("org-1") {
        test_db::reset_db(&pool).expect("reset");
        let publisher = test_db::create_publisher(&pool);
        let org = publisher.zitadel_id.clone().expect("org");
        let user = user.map(|user| rescope(user, &org));
        activate(
            &pool,
            publisher.publisher_id,
            &[DistributionPlatform::Zenodo],
        );
        let job = only_job(&pool, publisher.publisher_id);

        let context = context_for(&pool, user);
        let response = run(&schema, &context, &cancel(job.distribution_job_id)).await;

        if may_cancel {
            assert_eq!(
                data(&response, "cancelDistributionJob")["status"],
                "CANCELLED",
                "{label}"
            );
        } else {
            assert_unauthorized(&response);
            assert_eq!(
                only_job(&pool, publisher.publisher_id).status.to_string(),
                "PENDING",
                "{label} must be denied before any state change"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_authorization_matrix_holds_for_the_job_bearing_report() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let org = publisher.zitadel_id.clone().expect("org");
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let schema = create_schema();

    for (label, user, _, _, may_report) in matrix(&org) {
        let user = user.map(|user| rescope(user, &org));
        let context = context_for(&pool, user);
        let response = run(&schema, &context, REPORT).await;

        if may_report {
            let rows = data(&response, "publisherServiceConfigurations");
            assert_eq!(rows.as_array().expect("array").len(), 1, "{label}");
            assert!(!rows[0]["latestBackCatalogueJob"].is_null(), "{label}");
        } else {
            assert_unauthorized(&response);
        }
    }

    // The count query is protected identically, including its new filters.
    let count = "{ publisherServiceConfigurationCount(jobStatuses: [PENDING], \
                   withoutBackCatalogueJob: false) }";
    for (label, user, _, _, may_report) in matrix(&org) {
        let user = user.map(|user| rescope(user, &org));
        let context = context_for(&pool, user);
        let response = run(&schema, &context, count).await;
        if may_report {
            assert_eq!(
                data(&response, "publisherServiceConfigurationCount"),
                1,
                "{label}"
            );
        } else {
            assert_unauthorized(&response);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_worker_role_confers_no_publisher_scope_and_no_configuration_access() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let org = publisher.zitadel_id.clone().expect("org");
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let schema = create_schema();

    // A worker-only account, including one that carries an organisation key
    // under the unscoped role.
    let worker = user_with_scopes("worker", &[(Role::DisseminationWorker, &org)]);
    {
        use crate::policy::UserAccess;
        assert!(
            worker.publisher_org_ids().is_empty(),
            "a worker account must not appear to hold publisher organisations"
        );
        assert_eq!(
            worker.permissions_for_org(&org),
            crate::policy::PublisherPermissions::default(),
            "PublisherPermissions is untouched: publisher_admin, work_lifecycle \
             and cdn_write all remain false"
        );
        assert!(!worker.is_superuser());
    }

    let context = context_for(&pool, Some(worker));
    for query in [
        format!(
            "{{ publisherServiceConfiguration(publisherId: \"{}\") \
               {{ subscriptionPackage }} }}",
            publisher.publisher_id
        ),
        REPORT.to_string(),
        format!(
            "mutation {{ replacePublisherServiceConfiguration(data: {{ publisherId: \"{}\", \
               subscriptionPackage: SPHINX, enabledDistributionPlatforms: [], \
               expectedUpdatedAt: \"{}\" }}) {{ subscriptionPackage }} }}",
            publisher.publisher_id,
            token(&pool, publisher.publisher_id).to_rfc3339()
        ),
        cancel(only_job(&pool, publisher.publisher_id).distribution_job_id),
    ] {
        assert_unauthorized(&run(&schema, &context, &query).await);
    }
}

/// The body of a Rust block declaration, brace-balanced.
fn sdl_free_block<'a>(source: &'a str, declaration: &str) -> &'a str {
    let body = source
        .split_once(declaration)
        .unwrap_or_else(|| panic!("source must declare `{declaration}`"))
        .1;
    let bytes = body.as_bytes();
    let mut depth = 1usize;
    for (index, byte) in bytes.iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &body[..index];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced braces after `{declaration}`")
}

#[test]
fn the_worker_guard_is_one_explicit_predicate_with_no_inheritance_or_composition() {
    let source = include_str!("../policy.rs");

    assert!(source.contains("fn is_dissemination_worker(&self) -> bool"));
    assert!(source.contains("fn require_dissemination_worker(&self)"));
    // The guard is not satisfied by `SUPERUSER`, and no role-composition rule is
    // introduced anywhere in the policy layer.
    assert!(
        source.contains("if user.is_dissemination_worker()"),
        "the guard must test exactly the one role"
    );
    // Declarations, not prose: the module's own documentation names these to
    // record that they are deliberately absent.
    for forbidden in [
        "struct ServiceRole",
        "enum ServiceRole",
        "trait ServiceRole",
        "struct RoleRegistry",
        "enum MachineIdentity",
        "SERVICE_ACCOUNT,",
        "Machine,",
        "ServiceAccount,",
    ] {
        assert!(
            !source.contains(forbidden),
            "no generic service-role API, role registry or machine-identity model \
             may be introduced: found `{forbidden}`"
        );
    }
    // The role enum gains exactly one variant, and it is domain-specific.
    let role_enum = sdl_free_block(source, "pub(crate) enum Role {");
    assert!(role_enum.contains("DisseminationWorker,"));
    let variants: Vec<&str> = role_enum
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(',') && !line.starts_with("//") && !line.starts_with("#["))
        .collect();
    for generic in ["Service,", "Machine,", "Worker,", "ServiceAccount,"] {
        assert!(
            !variants.contains(&generic),
            "no catch-all machine role may be introduced: `{generic}`"
        );
    }
    assert!(variants.contains(&"DisseminationWorker,"));
    // `has_unscoped_role` stays private to the module.
    assert!(
        source.contains("trait UnscopedRoleAccess {"),
        "the shared key-presence check is a module-private trait"
    );
    assert!(
        !source.contains("pub(crate) trait UnscopedRoleAccess"),
        "it must not be exposed as a general service-role API"
    );
}

// ==========================================================================
// 25.10  Error contracts and non-exposure
// ==========================================================================

fn worker_context(pool: &Arc<PgPool>) -> Context {
    context_for(
        pool,
        Some(user_with(WORKER_ID, &[Role::DisseminationWorker])),
    )
}

fn superuser_context(pool: &Arc<PgPool>) -> Context {
    context_for(pool, Some(user_with("su", &[Role::Superuser])))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_four_new_errors_map_to_exactly_their_specified_graphql_types() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();

    // STALE_DISTRIBUTION_JOB_CLAIM.
    let (_publisher, job) = seeded_job(&pool);
    let context = worker_context(&pool);
    let response = run(
        &schema,
        &context,
        &complete(job.distribution_job_id, Uuid::new_v4()),
    )
    .await;
    let (message, kind) = only_error(&response);
    assert_eq!(kind, "STALE_DISTRIBUTION_JOB_CLAIM");
    assert_eq!(message, "The distribution job claim is no longer valid.");
    // It discloses neither the current token, nor the holder, nor whether one
    // exists.
    assert!(!message.to_lowercase().contains("token"));
    assert!(!message.contains(WORKER_ID));

    // DISTRIBUTION_JOB_TERMINAL, with the current status code as its payload.
    let claim = claim_directly(&pool);
    run(
        &schema,
        &context,
        &complete(job.distribution_job_id, claim.claim_token),
    )
    .await;
    let response = run(
        &schema,
        &context,
        &complete(job.distribution_job_id, claim.claim_token),
    )
    .await;
    let (message, kind) = only_error(&response);
    assert_eq!(kind, "DISTRIBUTION_JOB_TERMINAL");
    assert_eq!(
        message,
        "The distribution job is already in the terminal state SUCCEEDED."
    );

    // INVALID_DISTRIBUTION_JOB_ERROR_CODE — and explicitly **not**
    // INTERNAL_ERROR.
    test_db::reset_db(&pool).expect("reset");
    let (_publisher, job) = seeded_job(&pool);
    let claim = claim_directly(&pool);
    for invalid in ["not a code", &"A".repeat(65)] {
        let response = run(
            &schema,
            &context,
            &fail(job.distribution_job_id, claim.claim_token, invalid, true),
        )
        .await;
        let (message, kind) = only_error(&response);
        assert_eq!(
            kind, "INVALID_DISTRIBUTION_JOB_ERROR_CODE",
            "a deliberately specified validation contract must not be routed \
             through the server-fault family"
        );
        assert_ne!(kind, "INTERNAL_ERROR");
        assert_eq!(
            message,
            "The supplied distribution job error code is not a valid classification code."
        );
        assert!(
            !message.contains(invalid),
            "the rejected value is never reflected"
        );
        assert!(!message.contains(&invalid.chars().count().to_string()));
        assert!(!message.contains("A-Z"));
    }
    // The claim token survives, proven by a conforming resubmission.
    let response = run(
        &schema,
        &context,
        &fail(
            job.distribution_job_id,
            claim.claim_token,
            "TRANSPORT_FAILURE",
            true,
        ),
    )
    .await;
    assert_eq!(
        data(&response, "failDistributionJob")["lastErrorCode"],
        "TRANSPORT_FAILURE"
    );

    // DISTRIBUTION_JOB_CREATION_DISABLED, on the configuration path.
    test_db::reset_db(&pool).expect("reset");
    let publisher = test_db::create_publisher(&pool);
    let off = test_db::test_context_with_job_creation(
        Arc::clone(&pool),
        Some(user_with("su", &[Role::Superuser])),
        DistributionJobCreation::Off,
    );
    let query = format!(
        "mutation {{ replacePublisherServiceConfiguration(data: {{ publisherId: \"{}\", \
           subscriptionPackage: SPHINX, enabledDistributionPlatforms: [OAPEN], \
           expectedUpdatedAt: \"{}\" }}) {{ subscriptionPackage }} }}",
        publisher.publisher_id,
        token(&pool, publisher.publisher_id).to_rfc3339()
    );
    let response = run(&schema, &off, &query).await;
    let (message, kind) = only_error(&response);
    assert_eq!(kind, "DISTRIBUTION_JOB_CREATION_DISABLED");
    assert_eq!(
        message,
        "Automatic distribution job creation is disabled, so this platform \
         activation cannot be saved."
    );
    for forbidden in [
        "SELECT",
        "INSERT",
        "UPDATE",
        "distribution_job",
        "publisher_distribution_platform",
        "THOTH_DISTRIBUTION_JOB_CREATION",
        "OFF",
        "diesel",
        "postgres",
    ] {
        assert!(
            !message.contains(forbidden),
            "the message must disclose no SQL, table name, column name, driver \
             text, environment-variable name or value: found `{forbidden}`"
        );
    }
}

#[test]
fn exactly_four_new_error_variants_and_four_new_arms_exist() {
    let source = include_str!("../../../thoth-errors/src/lib.rs");
    for (variant, extension) in [
        ("StaleDistributionJobClaim", "STALE_DISTRIBUTION_JOB_CLAIM"),
        (
            "DistributionJobAlreadyTerminal",
            "DISTRIBUTION_JOB_TERMINAL",
        ),
        (
            "DistributionJobCreationDisabled",
            "DISTRIBUTION_JOB_CREATION_DISABLED",
        ),
        (
            "InvalidDistributionJobErrorCode",
            "INVALID_DISTRIBUTION_JOB_ERROR_CODE",
        ),
    ] {
        assert!(source.contains(variant), "missing variant `{variant}`");
        assert_eq!(
            source.matches(&format!("\"{extension}\"")).count(),
            1,
            "`{extension}` must have exactly one into_field_error arm"
        );
    }
    // No fifth variant: every `DistributionJob`-named variant is one of the
    // four above.
    let job_variants = source.matches("DistributionJob").count();
    assert!(job_variants > 0);
    for unexpected in [
        "DistributionJobNotClaimable",
        "DistributionJobRetryFailed",
        "DistributionJobLeaseExpired",
    ] {
        assert!(
            !source.contains(unexpected),
            "no fifth variant: `{unexpected}`"
        );
    }

    // No existing mapping changed.
    assert!(source.contains("\"STALE_SERVICE_CONFIGURATION\""));
    assert!(source.contains("\"INVALID_SUBJECT_CODE\""));
    assert!(source.contains("\"NO_ACCESS\""));
    assert_eq!(source.matches("\"INTERNAL_ERROR\"").count(), 1);
}

#[test]
fn the_generated_sdl_never_exposes_a_claim_token_or_an_operational_identity() {
    let sdl = create_schema().as_sdl();

    let job = sdl_block(&sdl, "type DistributionJob {");
    for forbidden in [
        "claimToken",
        "claimedBy",
        "deduplicationKey",
        "activationId",
    ] {
        assert!(
            !job.contains(forbidden),
            "`DistributionJob` must not expose `{forbidden}`: exposing the claim \
             token would let any caller who can read a job steal the live claim"
        );
    }
    // It is returned only on the claim payload.
    let claimed = sdl_block(&sdl, "type ClaimedDistributionJob {");
    assert!(claimed.contains("claimToken: Uuid!"));

    // `claimedBy` appears on attempt history and nowhere else on the job.
    let attempt = sdl_block(&sdl, "type DistributionJobAttempt {");
    assert!(attempt.contains("claimedBy: String!"));
    assert!(
        !attempt.contains("claimToken"),
        "an attempt's own claim token is stored for audit but never exposed"
    );

    // A success reports no error, so the completion input gains no `errorCode`.
    let complete_input = sdl_block(&sdl, "input CompleteDistributionJobInput {");
    assert!(!complete_input.contains("errorCode"));
    assert!(!complete_input.contains("errorDetail"));
    let fail_input = sdl_block(&sdl, "input FailDistributionJobInput {");
    assert!(fail_input.contains("errorCode: String!"));

    // No type exposes an adapter profile, endpoint, bucket, host or credential.
    for block in [
        "type DistributionJob {",
        "type DistributionJobTarget {",
        "type DistributionJobAttempt {",
        "type ClaimedDistributionJob {",
    ] {
        let body = sdl_block(&sdl, block);
        for forbidden in [
            "adapter",
            "Adapter",
            "endpoint",
            "Endpoint",
            "bucket",
            "Bucket",
            "host",
            "Host",
            "credential",
            "Credential",
            "secret",
            "Secret",
            "profile",
            "Profile",
        ] {
            assert!(
                !body.contains(forbidden),
                "{block} must not expose `{forbidden}`"
            );
        }
    }
}

#[test]
fn the_new_list_argument_follows_the_merged_option_vec_plus_default_convention() {
    let sdl = create_schema().as_sdl();
    let query = sdl_block(&sdl, "type QueryRoot {");

    // Quoted verbatim and compared against the merged siblings it must match.
    assert!(
        query.contains("jobStatuses: [DistributionJobStatus!] = []"),
        "jobStatuses must render as a nullable list with an empty default, \
         exactly as its merged siblings do"
    );
    assert!(query.contains("enabledPlatforms: [DistributionPlatform!] = []"));
    assert!(query.contains("packages: [ThothPackage!] = []"));
    assert!(
        !query.contains("jobStatuses: [DistributionJobStatus!]! ="),
        "a stricter non-null list would be wrong"
    );
    assert!(query.contains("withoutBackCatalogueJob: Boolean"));
    assert!(
        !query.contains("withoutBackCatalogueJob: Boolean!"),
        "absent and null must both mean `no filter`, so it is nullable with no default"
    );

    // The claim input's list argument follows the same convention.
    let claim_input = sdl_block(&sdl, "input ClaimDistributionJobsInput {");
    assert!(claim_input.contains("kinds: [DistributionJobKind!] = []"));
    assert!(claim_input.contains("limit: Int = 10"));
    assert!(claim_input.contains("leaseSeconds: Int = 900"));
}

#[test]
fn the_additive_sdl_inventory_is_exactly_section_20_1() {
    let sdl = create_schema().as_sdl();

    for declaration in [
        "enum DistributionJobKind {",
        "enum DistributionJobStatus {",
        "enum DistributionJobAttemptResult {",
        "enum DistributionJobCancellationReason {",
        "type DistributionJob {",
        "type DistributionJobTarget {",
        "type DistributionJobAttempt {",
        "type ClaimedDistributionJob {",
        "input ClaimDistributionJobsInput {",
        "input CompleteDistributionJobInput {",
        "input FailDistributionJobInput {",
        "input CancelDistributionJobInput {",
    ] {
        assert!(
            sdl.contains(declaration),
            "SDL must declare `{declaration}`"
        );
    }

    let mutation = sdl_block(&sdl, "type MutationRoot {");
    for operation in [
        "claimDistributionJobs(",
        "completeDistributionJob(",
        "failDistributionJob(",
        "cancelDistributionJob(",
    ] {
        assert!(
            mutation.contains(operation),
            "MutationRoot must expose `{operation}`"
        );
    }

    // No new top-level query: durable job state is reachable only through the
    // staff report and through the mutations' own payloads.
    let query = sdl_block(&sdl, "type QueryRoot {");
    for absent in [
        "distributionJob(",
        "distributionJobs(",
        "claimableDistributionJobs(",
    ] {
        assert!(
            !query.contains(absent),
            "no top-level job query may be added: `{absent}`"
        );
    }

    // One new field on one existing type, and BE-03's configuration type gains
    // none.
    let summary = sdl_block(&sdl, "type PublisherServiceConfigurationSummary {");
    assert!(summary.contains("latestBackCatalogueJob: DistributionJob"));
    assert!(summary.contains("configuration: PublisherServiceConfiguration!"));
    assert!(summary.contains("lastChange: PublisherServiceConfigurationChange"));
    let configuration = sdl_block(&sdl, "type PublisherServiceConfiguration {");
    for forbidden in ["Job", "job"] {
        assert!(
            !configuration.contains(forbidden),
            "BE-03's configuration-only type gains no job field"
        );
    }
}

/// The rendered SDL description attached to one `MutationRoot` field.
///
/// Juniper renders a field description as a quoted line immediately preceding
/// the field declaration, so the description is the last line before it. The
/// surrounding quotes are stripped; a field rendered without a description
/// fails loudly rather than returning a declaration line that would silently
/// satisfy an absence assertion.
fn mutation_field_description<'a>(sdl: &'a str, field_declaration: &str) -> &'a str {
    let mutation = sdl_block(sdl, "type MutationRoot {");
    let declaration = format!("\n  {field_declaration}");
    let line = mutation
        .split_once(declaration.as_str())
        .unwrap_or_else(|| panic!("`MutationRoot` must declare `{field_declaration}`"))
        .0
        .lines()
        .next_back()
        .expect("a declared field is preceded by at least one line")
        .trim();

    line.strip_prefix('"')
        .and_then(|body| body.strip_suffix('"'))
        .unwrap_or_else(|| {
            panic!("`{field_declaration}` must carry a rendered description, found `{line}`")
        })
}

#[test]
fn the_replacement_mutation_description_states_its_conditional_job_creation() {
    let sdl = create_schema().as_sdl();
    let description = mutation_field_description(&sdl, "replacePublisherServiceConfiguration(");

    // The defect this guards. BE-03's description survived into BE-04 stating
    // that the mutation creates no distribution job, which stopped being true
    // the moment BE-04 made a qualifying activation create one inside this very
    // transaction. A caller reading only introspection would have been told the
    // opposite of the mutation's most consequential side effect.
    assert!(
        !description.contains("creates no distribution job"),
        "the replacement mutation must not describe itself as creating no \
         distribution job: {description}"
    );

    // Conditional, atomic durable-job creation: which activation creates a job,
    // that it is durable, that it is committed with the configuration rather
    // than after it, and that it happens only while creation is enabled.
    for required in [
        "AUTOMATIC_PUSH",
        "durable distribution job",
        "atomically in the same transaction",
        "while automatic distribution job creation is enabled",
    ] {
        assert!(
            description.contains(required),
            "the description must state `{required}`: {description}"
        );
    }

    // Fail-closed: while creation is disabled the qualifying replacement is
    // refused whole. Nothing here may read as "commits, minus the job".
    for required in ["while it is disabled", "fails and rolls back in full"] {
        assert!(
            description.contains(required),
            "the description must state the fail-closed behaviour `{required}`: {description}"
        );
    }

    // The two statements the correction must not lose: creation is conditional,
    // not universal, and BE-04 still disseminates nothing.
    assert!(
        description.contains("No other change creates a job"),
        "the description must not imply every configuration change creates a job: {description}"
    );
    assert!(
        description.contains("performs no dissemination"),
        "the accurate no-dissemination statement must survive the correction: {description}"
    );
}

// ==========================================================================
// 25.12  Report semantics, filters and statement counts
// ==========================================================================

/// Seed `count` publishers, each with one back-catalogue job carrying `targets`
/// target rows and one closed attempt, using set-based SQL.
///
/// The attempt matters: a fixture whose nested collections are all empty would
/// prove a low statement count only because no loader had anything to dispatch
/// for.
fn seed_publishers_with_jobs(pool: &PgPool, count: usize) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher (publisher_id, publisher_name, zitadel_id) \
         SELECT gen_random_uuid(), 'Job Press ' || lpad(i::text, 5, '0'), \
                'org-' || lpad(i::text, 5, '0') \
         FROM generate_series(1, {count}) AS i"
    ))
    .execute(&mut connection)
    .expect("seed publishers");

    sql_query(
        "INSERT INTO publisher_distribution_platform \
           (publisher_id, platform, enabled, activation_id, enabled_at) \
         SELECT p.publisher_id, 'ZENODO', true, gen_random_uuid(), now() FROM publisher p",
    )
    .execute(&mut connection)
    .expect("seed assignments");

    sql_query(
        "INSERT INTO distribution_job \
           (kind, publisher_id, activation_id, deduplication_key) \
         SELECT 'PUBLISHER_BACK_CATALOGUE', p.publisher_id, a.activation_id, \
                'PUBLISHER_BACK_CATALOGUE:' || p.publisher_id::text || ':' \
                  || a.activation_id::text \
         FROM publisher p \
         JOIN publisher_distribution_platform a ON a.publisher_id = p.publisher_id",
    )
    .execute(&mut connection)
    .expect("seed jobs");

    sql_query(
        "INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT j.distribution_job_id, 'ZENODO' FROM distribution_job j",
    )
    .execute(&mut connection)
    .expect("seed targets");

    sql_query(
        "INSERT INTO distribution_job_attempt \
           (distribution_job_id, attempt_number, claim_token, claimed_by, \
            finished_at, result, error_code, error_detail) \
         SELECT j.distribution_job_id, 1, gen_random_uuid(), 'seed-worker', \
                now(), 'FAILED', 'TRANSPORT_FAILURE', 'a bounded description' \
         FROM distribution_job j",
    )
    .execute(&mut connection)
    .expect("seed attempts");

    // The job rows above were written with `attempt_count = 0`; make them
    // consistent with the attempt row they carry.
    sql_query("UPDATE distribution_job SET attempt_count = 1")
        .execute(&mut connection)
        .expect("align attempt counts");
}

/// The no-job fixture: publishers with enabled `AutomaticPush` assignments and
/// **no** durable job, which is the state of every publisher whose assignments
/// predate this feature.
///
/// The assignments matter: they keep `enabledDistributionPlatforms` non-empty,
/// so the full report selection still exercises `BE-02`'s loader and the
/// measured difference between the two fixtures is the job path alone.
fn seed_publishers_without_jobs(pool: &PgPool, count: usize) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher (publisher_id, publisher_name, zitadel_id) \
         SELECT gen_random_uuid(), 'Jobless Press ' || lpad(i::text, 5, '0'), \
                'org-' || lpad(i::text, 5, '0') \
         FROM generate_series(1, {count}) AS i"
    ))
    .execute(&mut connection)
    .expect("seed publishers");

    sql_query(
        "INSERT INTO publisher_distribution_platform \
           (publisher_id, platform, enabled, activation_id, enabled_at) \
         SELECT p.publisher_id, 'ZENODO', true, gen_random_uuid(), now() FROM publisher p",
    )
    .execute(&mut connection)
    .expect("seed assignments");

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    assert_eq!(
        sql_query("SELECT count(*) AS count FROM distribution_job")
            .get_result::<CountRow>(&mut connection)
            .expect("count jobs")
            .count,
        0,
        "the no-job fixture must contain no job at all"
    );
}

/// The job-only selection of section 17.4 A: every `BE-04` field, and no field
/// that invokes `BE-02`'s assignment loader.
fn job_only_selection(limit: usize) -> String {
    format!(
        "{{ publisherServiceConfigurations(limit: {limit}) {{ \
             configuration {{ publisher {{ publisherId }} }} \
             latestBackCatalogueJob {{ \
               distributionJobId status \
               targets {{ platform }} \
               attempts {{ attemptNumber result }} }} }} }}"
    )
}

/// The full report selection of section 17.4 B: the job-only selection **plus**
/// `enabledDistributionPlatforms`.
fn full_report_selection(limit: usize) -> String {
    format!(
        "{{ publisherServiceConfigurations(limit: {limit}) {{ \
             configuration {{ publisher {{ publisherId }} \
                              enabledDistributionPlatforms {{ platform }} }} \
             latestBackCatalogueJob {{ \
               distributionJobId status \
               targets {{ platform }} \
               attempts {{ attemptNumber result }} }} }} }}"
    )
}

/// The measured report statement count, for **both** section 17.4.3 selections,
/// at page sizes 1, 25 and 200, on a page that contains at least one job **and**
/// on a page that contains none.
///
/// The expectation is **derived** from the measured per-chunk classification
/// rather than hard-coded, exactly as section 25.12 requires:
///
/// ```text
/// statements = 2 + 3 * C_job_nonempty + 1 * C_job_empty + 1 * C_assign
/// ```
///
/// so the four named outcomes — five and six on a page with a job, three and
/// four on a page without one — fall out of the arithmetic instead of being
/// asserted as magic numbers. `C_job_empty` costs one statement rather than
/// three because the composite loader skips L2 and L3 entirely for a chunk whose
/// L1 returned no job, which is why the arithmetic is stated per dispatch chunk
/// and never collapsed to a page-global "does this page have any job" flag.
///
/// What is BE-04's own, provable by construction and independent of any
/// scheduler, is the per-chunk statement shape and the **zero** dispatches of
/// the second-level target and attempt loaders on the report path — the
/// assertion that proves the report carries no dependent-arrival cohort. The
/// dispatch-chunk count itself is the shared `ADR-0007` foundation's property:
/// section 4.6 fixes `ceil(N / 200)`, which is one for every page size measured
/// here, and section 10.2 records it measured. This test consumes that property
/// and neither restates it as a BE-04 guarantee nor weakens it.
///
/// An unexpected `C_job > 1` at a page size at or below 200 is `BLOCKED` under
/// stop condition 23. It is not absorbed by relaxing this expectation.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_report_statement_count_equals_the_derived_per_chunk_arithmetic() {
    for (fixture_label, seed, page_has_job) in [
        (
            "page containing at least one job",
            seed_publishers_with_jobs as fn(&PgPool, usize),
            true,
        ),
        (
            "page whose publishers have no job",
            seed_publishers_without_jobs as fn(&PgPool, usize),
            false,
        ),
    ] {
        for (label, selection, selects_assignments) in [
            (
                "job-only selection",
                job_only_selection as fn(usize) -> String,
                false,
            ),
            ("full report selection", full_report_selection, true),
        ] {
            for page_size in [1usize, 25, 200] {
                let (_guard, ordinary_pool) = test_db::setup_test_db();
                seed(&ordinary_pool, page_size);

                let probe = SqlProbe::install(&test_db::test_db_url());
                let stats = ObservedLoaderStats::default();
                let observed = (
                    Arc::clone(&stats.publisher_distribution_platforms),
                    Arc::clone(&stats.latest_back_catalogue_jobs),
                    Arc::clone(&stats.distribution_job_targets),
                    Arc::clone(&stats.distribution_job_attempts),
                );
                let mut context = superuser_context(&probe.pool);
                context.loaders =
                    RequestLoaders::for_request_observed_all(Arc::clone(&probe.pool), stats);
                let schema = create_schema();

                probe.start();
                let response = run(&schema, &context, &selection(page_size)).await;
                let captured = probe.captured_statements();
                let statements = domain_statements(&captured);
                let (assignments, composite, targets, attempts) = observed;

                let case = format!("{label} | {fixture_label} | page {page_size}");
                let rows = data(&response, "publisherServiceConfigurations");
                assert_eq!(rows.as_array().expect("array").len(), page_size, "{case}");

                // The fixture must actually be the fixture, or a low count would
                // prove nothing.
                if page_has_job {
                    assert!(
                        rows[0]["latestBackCatalogueJob"]["distributionJobId"].is_string(),
                        "{case}: the fixture must carry a non-null job"
                    );
                    assert_eq!(
                        rows[0]["latestBackCatalogueJob"]["targets"]
                            .as_array()
                            .expect("targets")
                            .len(),
                        1,
                        "{case}: and a target, so the count is not low merely \
                         because every nested collection was empty"
                    );
                    assert_eq!(
                        rows[0]["latestBackCatalogueJob"]["attempts"]
                            .as_array()
                            .expect("attempts")
                            .len(),
                        1,
                        "{case}: and an attempt, for the same reason"
                    );
                } else {
                    assert!(
                        rows.as_array()
                            .expect("array")
                            .iter()
                            .all(|row| row["latestBackCatalogueJob"].is_null()),
                        "{case}: no publisher on this page may have a job"
                    );
                }

                // --------------------------------------------------------
                // Measured chunk classification.
                // --------------------------------------------------------
                let classifications = composite.classifications();
                assert!(
                    classifications.iter().all(Option::is_some),
                    "{case}: every composite chunk must be classified; a `None` \
                     means a chunk failed. Observed {classifications:?}"
                );
                let c_job_nonempty = classifications
                    .iter()
                    .filter(|outcome| **outcome == Some(true))
                    .count();
                let c_job_empty = classifications
                    .iter()
                    .filter(|outcome| **outcome == Some(false))
                    .count();
                let c_job = composite.dispatch_count();
                let c_assign = assignments.dispatch_count();

                let derived = 2 + 3 * c_job_nonempty + c_job_empty + c_assign;

                println!(
                    "BE-04 report measurement | {case} | statements {observed_total} \
                     (derived {derived}) | composite chunks {composite_chunks:?} \
                     classified {classifications:?} | C_job_nonempty {c_job_nonempty} \
                     | C_job_empty {c_job_empty} | C_assign {c_assign} \
                     | target dispatches {target_dispatches} \
                     | attempt dispatches {attempt_dispatches} \
                     | assignment chunks {assignment_chunks:?} \
                     | driver metadata lookups {metadata}",
                    observed_total = statements.len(),
                    composite_chunks = composite.batch_sizes(),
                    target_dispatches = targets.dispatch_count(),
                    attempt_dispatches = attempts.dispatch_count(),
                    assignment_chunks = assignments.batch_sizes(),
                    metadata = metadata_lookups(&captured).len(),
                );

                // 1. The derived total, and the observed total, are equal.
                assert_eq!(
                    statements.len(),
                    derived,
                    "{case}: the observed statement count must equal the section \
                     17.4.3 arithmetic evaluated with the measured chunk \
                     classification: 2 + 3*{c_job_nonempty} + {c_job_empty} + \
                     {c_assign}. Observed statements:\n{}",
                    statements
                        .iter()
                        .map(|sql| sql.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                // 2. Exactly two root statements, at every page size.
                assert_eq!(
                    statements_matching(&captured, "FROM \"publisher\" ").len(),
                    1,
                    "{case}: one filtered, ordered, paginated page query"
                );
                assert_eq!(
                    statements_matching(
                        &captured,
                        "FROM \"publisher_service_configuration_history\""
                    )
                    .len(),
                    1,
                    "{case}: one latest-change query"
                );

                // 3. Per-loader dispatch expectations, stated per loader rather
                //    than as a blanket "every loader dispatches once".
                assert_eq!(
                    c_job,
                    1,
                    "{case}: the composite loader is keyed by publisher_id, which \
                     is available at resolver entry, so a page of N <= 200 is one \
                     loader-first cohort and one chunk (ADR-0007 section 4.6). \
                     C_job > 1 here is BLOCKED under stop condition 23, not a \
                     count to relax. Observed chunks {:?}",
                    composite.batch_sizes()
                );
                assert_eq!(
                    c_assign,
                    usize::from(selects_assignments),
                    "{case}: BE-02's assignment loader dispatches when and only \
                     when enabledDistributionPlatforms is selected"
                );

                // 4. The scheduler-independent assertion: the second-level
                //    loaders are not on the report path at all.
                assert_eq!(
                    targets.dispatch_count(),
                    0,
                    "{case}: DistributionJob.targets must read the preloaded value \
                     on the report path and issue no second-level loader call"
                );
                assert_eq!(
                    attempts.dispatch_count(),
                    0,
                    "{case}: DistributionJob.attempts must read the preloaded value \
                     on the report path and issue no second-level loader call"
                );

                // 5. Both directions of the per-chunk branch, each measured
                //    rather than inferred.
                if page_has_job {
                    assert_eq!(
                        (c_job_nonempty, c_job_empty),
                        (1, 0),
                        "{case}: a chunk whose L1 returns a job costs three \
                         statements"
                    );
                    assert_eq!(
                        statements_matching(&captured, "FROM \"distribution_job_target\"").len(),
                        1,
                        "{case}: L2 runs exactly once for a non-empty chunk"
                    );
                    assert_eq!(
                        statements_matching(&captured, "FROM \"distribution_job_attempt\"").len(),
                        1,
                        "{case}: L3 runs exactly once for a non-empty chunk"
                    );
                } else {
                    assert_eq!(
                        (c_job_nonempty, c_job_empty),
                        (0, 1),
                        "{case}: a chunk whose L1 returns no job costs one statement"
                    );
                    assert!(
                        statements_matching(&captured, "FROM \"distribution_job_target\"")
                            .is_empty(),
                        "{case}: L2 must not be issued at all for an empty chunk"
                    );
                    assert!(
                        statements_matching(&captured, "FROM \"distribution_job_attempt\"")
                            .is_empty(),
                        "{case}: L3 must not be issued at all for an empty chunk"
                    );
                }
                assert_eq!(
                    statements_matching(&captured, "FROM \"distribution_job\" ").len(),
                    1,
                    "{case}: L1 is issued once per composite chunk, always"
                );

                // 6. Every dispatch chunk partitions the requested key set.
                for (name, loader) in [("composite", &composite), ("assignment", &assignments)] {
                    let chunks = loader.batch_sizes();
                    if chunks.is_empty() {
                        continue;
                    }
                    assert_eq!(
                        chunks.iter().sum::<usize>(),
                        page_size,
                        "{case}: the {name} loader's chunks must partition the \
                         requested keys — no key loaded twice, none missed. \
                         Observed {chunks:?}"
                    );
                }

                // 7. Every statement is set-based.
                assert!(
                    statements
                        .iter()
                        .all(|sql| sql.contains("= ANY(") || sql.contains("LIMIT")),
                    "{case}: every statement must be set-based:\n{}",
                    statements
                        .iter()
                        .map(|sql| sql.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                assert!(
                    metadata_lookups(&captured).len() <= 4,
                    "{case}: Diesel resolves a custom type's OID once per \
                     connection per type and caches it, so these are bounded by \
                     the four new enum types and never by the key count"
                );
            }
        }
    }
}

/// The other half of exact selection dependence: a selection that does **not**
/// reach `latestBackCatalogueJob` dispatches the composite loader **zero**
/// times, so the loader costs nothing when the field is not asked for.
///
/// The matrix above proves "dispatches when selected"; this proves "and only
/// when".
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_unselected_composite_loader_dispatches_nothing() {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    seed_publishers_with_jobs(&ordinary_pool, 25);

    let probe = SqlProbe::install(&test_db::test_db_url());
    let stats = ObservedLoaderStats::default();
    let observed = (
        Arc::clone(&stats.publisher_distribution_platforms),
        Arc::clone(&stats.latest_back_catalogue_jobs),
        Arc::clone(&stats.distribution_job_targets),
        Arc::clone(&stats.distribution_job_attempts),
    );
    let mut context = superuser_context(&probe.pool);
    context.loaders = RequestLoaders::for_request_observed_all(Arc::clone(&probe.pool), stats);
    let schema = create_schema();

    probe.start();
    let response = run(
        &schema,
        &context,
        "{ publisherServiceConfigurations(limit: 25) { \
             configuration { publisher { publisherId } \
                             enabledDistributionPlatforms { platform } } } }",
    )
    .await;
    let captured = probe.captured_statements();
    let statements = domain_statements(&captured);
    let (assignments, composite, targets, attempts) = observed;

    assert_eq!(
        data(&response, "publisherServiceConfigurations")
            .as_array()
            .expect("array")
            .len(),
        25
    );
    assert_eq!(composite.dispatch_count(), 0, "C_job must be 0");
    assert_eq!(targets.dispatch_count(), 0);
    assert_eq!(attempts.dispatch_count(), 0);
    assert_eq!(assignments.dispatch_count(), 1, "C_assign must be 1");
    assert!(
        statements_matching(&captured, "FROM \"distribution_job\"").is_empty(),
        "an unselected loader issues no statement of any kind"
    );
    // 2 + 3*0 + 0 + 1.
    assert_eq!(statements.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_publisher_with_no_durable_job_is_reported_as_null_and_nothing_else() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();

    // A publisher with enabled AutomaticPush assignments and no job: the state
    // of every publisher whose assignments predate this feature.
    let publisher = test_db::create_publisher(&pool);
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at) \
         VALUES ('{}', 'ZENODO', true, gen_random_uuid(), now())",
        publisher.publisher_id
    ))
    .execute(&mut connection)
    .expect("seed a pre-existing assignment");
    drop(connection);

    let context = superuser_context(&pool);
    let response = run(&schema, &context, &job_only_selection(10)).await;
    let rows = data(&response, "publisherServiceConfigurations");
    assert_eq!(rows.as_array().expect("array").len(), 1);
    assert!(
        rows[0]["latestBackCatalogueJob"].is_null(),
        "no job is represented by null, and by nothing else"
    );

    // Nothing anywhere in the payload fabricates a status or a delivery marker.
    let rendered = serde_json::to_string(&response).expect("serialize");
    for fabricated in [
        "NOT_STARTED",
        "UNKNOWN",
        "NOT_APPLICABLE",
        "NONE",
        "delivered",
        "submitted",
        "adapterActive",
        "SUCCEEDED",
        "FAILED",
    ] {
        assert!(
            !rendered.contains(fabricated),
            "the response must not fabricate `{fabricated}` for a publisher with no job"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_repaired_group_with_no_job_stays_no_job_and_implies_no_delivery() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let publisher = test_db::create_publisher(&pool);

    // A pre-existing enabled linked pair with a split activation, and no job.
    let mut connection = pool.get().expect("connection");
    for platform in ["OAPEN", "DOAB"] {
        sql_query(format!(
            "INSERT INTO publisher_distribution_platform \
             (publisher_id, platform, enabled, activation_id, enabled_at) \
             VALUES ('{}', '{platform}', true, gen_random_uuid(), now())",
            publisher.publisher_id
        ))
        .execute(&mut connection)
        .expect("seed split pair");
    }
    drop(connection);

    // Repair it through the real coordinator.
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );

    let context = superuser_context(&pool);
    let response = run(&schema, &context, &job_only_selection(10)).await;
    let rows = data(&response, "publisherServiceConfigurations");
    assert!(
        rows[0]["latestBackCatalogueJob"].is_null(),
        "a repaired group with no job remains no job. The null means Thoth holds \
         no durable job, and nothing else: not that delivery happened, not that \
         it did not, not that an adapter ran"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_report_filters_by_job_status_and_by_job_presence() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let context = superuser_context(&pool);

    // Three publishers: one PENDING job, one SUCCEEDED job, one with no job.
    let pending = test_db::create_publisher(&pool);
    activate(&pool, pending.publisher_id, &[DistributionPlatform::Zenodo]);

    let succeeded = test_db::create_publisher(&pool);
    activate(
        &pool,
        succeeded.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let succeeded_job = only_job(&pool, succeeded.publisher_id);
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE distribution_job SET status = 'SUCCEEDED', completed_at = now() \
         WHERE distribution_job_id = '{}'",
        succeeded_job.distribution_job_id
    ))
    .execute(&mut connection)
    .expect("terminalize");
    drop(connection);

    let jobless = test_db::create_publisher(&pool);

    let ids = |response: &JsonValue| -> Vec<String> {
        data(response, "publisherServiceConfigurations")
            .as_array()
            .expect("array")
            .iter()
            .map(|row| {
                row["configuration"]["publisher"]["publisherId"]
                    .as_str()
                    .expect("publisher id")
                    .to_string()
            })
            .collect()
    };
    let query = |filter: &str| {
        format!(
            "{{ publisherServiceConfigurations({filter}) {{ \
               configuration {{ publisher {{ publisherId }} }} }} }}"
        )
    };
    let count = |filter: &str| format!("{{ publisherServiceConfigurationCount({filter}) }}");

    // Each status individually.
    let response = run(&schema, &context, &query("jobStatuses: [PENDING]")).await;
    assert_eq!(ids(&response), vec![pending.publisher_id.to_string()]);
    let response = run(&schema, &context, &query("jobStatuses: [SUCCEEDED]")).await;
    assert_eq!(ids(&response), vec![succeeded.publisher_id.to_string()]);

    // Several statuses widen: OR within the list.
    let response = run(
        &schema,
        &context,
        &query("jobStatuses: [PENDING, SUCCEEDED]"),
    )
    .await;
    assert_eq!(ids(&response).len(), 2);

    // Empty means no filter.
    let response = run(&schema, &context, &query("jobStatuses: []")).await;
    assert_eq!(ids(&response).len(), 3);

    // The count query applies exactly the same predicates.
    for (filter, expected) in [
        ("jobStatuses: [PENDING]", 1),
        ("jobStatuses: [PENDING, SUCCEEDED]", 2),
        ("jobStatuses: []", 3),
        ("withoutBackCatalogueJob: true", 1),
        ("withoutBackCatalogueJob: false", 2),
        // The documented contradiction matches zero publishers, deterministically
        // and without error.
        ("withoutBackCatalogueJob: true, jobStatuses: [PENDING]", 0),
    ] {
        let response = run(&schema, &context, &count(filter)).await;
        assert_eq!(
            data(&response, "publisherServiceConfigurationCount"),
            expected,
            "count with `{filter}`"
        );
        let listed = run(&schema, &context, &query(filter)).await;
        assert_eq!(
            ids(&listed).len(),
            expected as usize,
            "the list and the count cannot diverge: `{filter}`"
        );
    }

    // `withoutBackCatalogueJob: true` selects exactly the jobless publisher.
    let response = run(&schema, &context, &query("withoutBackCatalogueJob: true")).await;
    assert_eq!(ids(&response), vec![jobless.publisher_id.to_string()]);

    // Filters combine conjunctively with the existing ones, and apply before
    // pagination.
    let response = run(
        &schema,
        &context,
        &query("jobStatuses: [PENDING, SUCCEEDED], packages: [SPHINX], limit: 1"),
    )
    .await;
    assert_eq!(ids(&response).len(), 1);
    let response = run(
        &schema,
        &context,
        &query("jobStatuses: [PENDING], packages: [OBELISK]"),
    )
    .await;
    assert!(ids(&response).is_empty(), "filters are conjunctive");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_report_paginates_deterministically_with_the_new_filters() {
    let (_guard, pool) = test_db::setup_test_db();
    seed_publishers_with_jobs(&pool, 12);
    let schema = create_schema();
    let context = superuser_context(&pool);

    let mut paged: Vec<String> = Vec::new();
    for offset in [0, 5, 10] {
        let response = run(
            &schema,
            &context,
            &format!(
                "{{ publisherServiceConfigurations(limit: 5, offset: {offset}, \
                     jobStatuses: [PENDING]) {{ \
                     configuration {{ publisher {{ publisherId }} }} }} }}"
            ),
        )
        .await;
        paged.extend(
            data(&response, "publisherServiceConfigurations")
                .as_array()
                .expect("array")
                .iter()
                .map(|row| {
                    row["configuration"]["publisher"]["publisherId"]
                        .as_str()
                        .expect("id")
                        .to_string()
                }),
        );
    }
    let unique: std::collections::HashSet<&String> = paged.iter().collect();
    assert_eq!(unique.len(), paged.len(), "offset pagination is stable");
    assert_eq!(paged.len(), 12);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_batch_wide_loader_failure_fails_closed_for_every_key() {
    let (_guard, pool) = test_db::setup_test_db();
    seed_publishers_with_jobs(&pool, 3);
    let schema = create_schema();

    // A context whose loaders point at an unreachable database, while the page
    // query itself succeeds against the real one.
    let mut context = superuser_context(&pool);
    context.loaders = RequestLoaders::for_request(Arc::new(test_db::failing_pool()));

    let response = run(&schema, &context, &job_only_selection(10)).await;
    let errors = response["errors"].as_array().expect("errors array");
    assert!(
        !errors.is_empty(),
        "a failed batch must not become successful empty data"
    );
    for error in errors {
        assert!(
            error["path"]
                .as_array()
                .expect("path")
                .iter()
                .any(|segment| segment == "latestBackCatalogueJob"),
            "the failure surfaces at the owning child field path"
        );
    }
    // No per-key fallback: the response carries no partially populated job.
    let rendered = serde_json::to_string(&response).expect("serialize");
    assert!(!rendered.contains("\"status\":\"PENDING\""));
}

// ==========================================================================
// 25.7  Transaction integration and 25.8 claim statement counts
// ==========================================================================

fn statements_matching<'a>(captured: &'a [String], needle: &str) -> Vec<&'a String> {
    captured.iter().filter(|sql| sql.contains(needle)).collect()
}

/// The captured statements that are actually domain work.
///
/// The probe also records traffic that belongs to the pool and the driver
/// rather than to the resolver's query shape, and counting it would make the
/// bound depend on how many connections happened to be checked out rather than
/// on the selection:
///
/// - r2d2's `SELECT 1` liveness check;
/// - the transaction control statements Diesel issues around
///   `connection.transaction(...)`;
/// - Diesel's `pg_type` OID resolution for a custom enum type, which each
///   connection performs **once per type** and then caches. It is bounded by
///   the number of distinct custom types, never by the number of keys, and
///   [`metadata_lookups`] asserts that separately.
fn domain_statements(captured: &[String]) -> Vec<&String> {
    captured
        .iter()
        .filter(|sql| {
            let statement = sql.trim();
            !(statement.starts_with("SELECT 1")
                || statement.starts_with("BEGIN")
                || statement.starts_with("COMMIT")
                || statement.starts_with("ROLLBACK")
                || statement.starts_with("SET ")
                || statement.contains("FROM \"pg_type\""))
        })
        .collect()
}

/// Diesel's per-connection custom-type OID lookups.
fn metadata_lookups(captured: &[String]) -> Vec<&String> {
    captured
        .iter()
        .filter(|sql| sql.contains("FROM \"pg_type\""))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_job_creating_change_keeps_the_specified_statement_order_and_one_publisher_update() {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&ordinary_pool);
    let imprint = test_db::create_imprint(&ordinary_pool, &publisher);
    test_db::create_work(&ordinary_pool, &imprint);
    let current = token(&ordinary_pool, publisher.publisher_id);

    let probe = SqlProbe::install(&test_db::test_db_url());
    let context = test_db::test_context_with_job_creation(
        Arc::clone(&probe.pool),
        Some(user_with("su-order", &[Role::Superuser])),
        DistributionJobCreation::On,
    );
    let schema = create_schema();

    probe.start();
    let response = run(
        &schema,
        &context,
        &format!(
            "mutation {{ replacePublisherServiceConfiguration(data: {{ publisherId: \"{}\", \
               subscriptionPackage: SPHINX, enabledDistributionPlatforms: [OAPEN], \
               expectedUpdatedAt: \"{}\" }}) {{ subscriptionPackage }} }}",
            publisher.publisher_id,
            current.to_rfc3339()
        ),
    )
    .await;
    let captured = probe.captured_statements();
    assert_eq!(
        data(&response, "replacePublisherServiceConfiguration")["subscriptionPackage"],
        "SPHINX"
    );

    let position = |needle: &str| {
        captured
            .iter()
            .position(|sql| sql.contains(needle))
            .unwrap_or_else(|| {
                panic!(
                    "expected a statement containing `{needle}`:\n{}",
                    captured.join("\n")
                )
            })
    };

    let lock = position("FOR UPDATE");
    let lifecycle = position("INSERT INTO \"publisher_distribution_platform\"");
    let job_insert = position("INSERT INTO \"distribution_job\"");
    let target_insert = position("INSERT INTO \"distribution_job_target\"");
    let publisher_update = position("UPDATE \"publisher\"");
    let audit = position("INSERT INTO \"publisher_service_configuration_history\"");

    assert!(
        lock < lifecycle,
        "the publisher row lock is the first statement"
    );
    assert!(
        lifecycle < job_insert,
        "lifecycle writes precede job writes"
    );
    assert!(job_insert < target_insert, "the job precedes its targets");
    assert!(
        target_insert < publisher_update,
        "job writes precede the publisher UPDATE, which fires the work-freshness \
         trigger and holds row locks on every one of the publisher's works until \
         commit"
    );
    assert!(publisher_update < audit, "the audit row is written last");

    assert_eq!(
        statements_matching(&captured, "UPDATE \"publisher\" ").len(),
        1,
        "exactly one publisher UPDATE, so the work-freshness cascade runs once"
    );
    // Application lock order: nothing locks a distribution_job row before the
    // publisher row, and the configuration path takes no job row lock at all.
    assert!(
        !captured
            .iter()
            .any(|sql| sql.contains("distribution_job") && sql.contains("FOR UPDATE")),
        "the configuration path takes no distribution_job row lock"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_job_creating_change_writes_one_audit_row_with_the_unwidened_key_set() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );

    let mut connection = pool.get().expect("connection");
    #[derive(diesel::QueryableByName)]
    struct AuditRow {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        before_state: JsonValue,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        after_state: JsonValue,
    }
    let rows = sql_query(format!(
        "SELECT before_state, after_state FROM publisher_service_configuration_history \
         WHERE publisher_id = '{}'",
        publisher.publisher_id
    ))
    .load::<AuditRow>(&mut connection)
    .expect("audit rows");

    assert_eq!(
        rows.len(),
        1,
        "exactly one audit row for the whole committed change"
    );
    for state in [&rows[0].before_state, &rows[0].after_state] {
        let mut keys: Vec<&String> = state.as_object().expect("object").keys().collect();
        keys.sort();
        assert_eq!(
            keys,
            vec![
                "configurationVersion",
                "enabledDistributionPlatforms",
                "subscriptionPackage"
            ],
            "the audit key set is not widened: job creation is recorded in \
             distribution_job, which is the durable, queryable, typed record of it"
        );
    }
    // The token moved, so a job never exists without a token bump.
    assert!(token(&pool, publisher.publisher_id) > Timestamp::default());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_claim_payload_costs_a_constant_number_of_statements_at_every_batch_size() {
    for batch in [1usize, 10, 50] {
        let (_guard, ordinary_pool) = test_db::setup_test_db();
        for _ in 0..batch {
            let publisher = test_db::create_publisher(&ordinary_pool);
            activate(
                &ordinary_pool,
                publisher.publisher_id,
                &[DistributionPlatform::Oapen],
            );
        }

        let probe = SqlProbe::install(&test_db::test_db_url());
        let context = context_for(
            &probe.pool,
            Some(user_with(WORKER_ID, &[Role::DisseminationWorker])),
        );
        let schema = create_schema();
        let query = format!(
            "mutation {{ claimDistributionJobs(data: {{ limit: {batch} }}) \
               {{ claimToken attemptNumber \
                  job {{ distributionJobId status attemptCount \
                         targets {{ platform }} attempts {{ attemptNumber result }} }} }} }}"
        );

        probe.start();
        let response = run(&schema, &context, &query).await;
        let captured = probe.captured_statements();

        let claimed = data(&response, "claimDistributionJobs")
            .as_array()
            .expect("array")
            .clone();
        assert_eq!(claimed.len(), batch);
        assert_eq!(
            claimed[0]["job"]["targets"]
                .as_array()
                .expect("targets")
                .len(),
            2,
            "the payload really carries its targets, so the count is not low \
             merely because nothing was resolved"
        );
        assert_eq!(
            claimed[0]["job"]["attempts"]
                .as_array()
                .expect("attempts")
                .len(),
            1
        );

        // Statement 1 recovery, statement 2 the atomic claim, statements 3 and 4
        // the two set-based payload reads.
        let statements = domain_statements(&captured);
        assert_eq!(
            statements.len(),
            4,
            "claim of {batch}: expected a constant 4 statements, observed {}:\n{}",
            statements.len(),
            statements
                .iter()
                .map(|sql| sql.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            metadata_lookups(&captured).len() <= 4,
            "claim of {batch}: custom-type OID lookups are per connection per \
             type, never per key"
        );
        assert_eq!(
            statements_matching(&captured, "FOR UPDATE OF j SKIP LOCKED").len(),
            1,
            "exactly one claim statement, and no second claim query or read-back \
             of recently claimed rows"
        );
        assert_eq!(
            statements_matching(&captured, "lease_expires_at <= CURRENT_TIMESTAMP").len(),
            1,
            "one bounded lease-recovery statement"
        );
        assert_eq!(
            statements_matching(&captured, "FROM \"distribution_job_target\"").len(),
            1
        );
        assert_eq!(
            statements_matching(&captured, "FROM \"distribution_job_attempt\"").len(),
            1
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_zero_claim_returns_an_empty_list_and_issues_no_payload_statements() {
    let (_guard, _ordinary_pool) = test_db::setup_test_db();
    let probe = SqlProbe::install(&test_db::test_db_url());
    let context = context_for(
        &probe.pool,
        Some(user_with(WORKER_ID, &[Role::DisseminationWorker])),
    );
    let schema = create_schema();

    probe.start();
    let response = run(&schema, &context, CLAIM).await;
    let captured = probe.captured_statements();

    assert!(data(&response, "claimDistributionJobs")
        .as_array()
        .expect("array")
        .is_empty());
    assert!(
        statements_matching(&captured, "FROM \"distribution_job_target\"").is_empty(),
        "zero claims issues no payload statement at all"
    );
    assert!(statements_matching(&captured, "FROM \"distribution_job_attempt\"").is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_worker_claim_path_does_not_use_the_request_local_loaders() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );

    let stats = ObservedLoaderStats::default();
    let observed = (
        Arc::clone(&stats.distribution_job_targets),
        Arc::clone(&stats.distribution_job_attempts),
        Arc::clone(&stats.latest_back_catalogue_jobs),
    );
    let mut context = context_for(
        &pool,
        Some(user_with(WORKER_ID, &[Role::DisseminationWorker])),
    );
    context.loaders = RequestLoaders::for_request_observed_all(Arc::clone(&pool), stats);
    let schema = create_schema();

    let response = run(&schema, &context, CLAIM).await;
    assert_eq!(
        data(&response, "claimDistributionJobs")
            .as_array()
            .expect("array")
            .len(),
        1
    );

    let (targets, attempts, latest) = observed;
    assert_eq!(
        targets.dispatch_count(),
        0,
        "the claim path resolves its own targets"
    );
    assert_eq!(attempts.dispatch_count(), 0, "and its own attempts");
    assert_eq!(latest.dispatch_count(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_failed_transaction_leaves_neither_a_job_nor_a_configuration_change() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let schema = create_schema();

    // The `OFF` fail-closed path is the induced failure after step 9b's
    // qualifying determination: both halves must be absent afterwards.
    let before = token(&pool, publisher.publisher_id);
    let off = test_db::test_context_with_job_creation(
        Arc::clone(&pool),
        Some(user_with("su", &[Role::Superuser])),
        DistributionJobCreation::Off,
    );
    let response = run(
        &schema,
        &off,
        &format!(
            "mutation {{ replacePublisherServiceConfiguration(data: {{ publisherId: \"{}\", \
               subscriptionPackage: SPHINX, enabledDistributionPlatforms: [OAPEN], \
               expectedUpdatedAt: \"{}\" }}) {{ subscriptionPackage }} }}",
            publisher.publisher_id,
            before.to_rfc3339()
        ),
    )
    .await;
    let (_, kind) = only_error(&response);
    assert_eq!(kind, "DISTRIBUTION_JOB_CREATION_DISABLED");
    assert!(
        response["data"]["replacePublisherServiceConfiguration"].is_null(),
        "there is no silent success: the mutation must not return a \
         configuration describing an activation it did not commit"
    );

    let mut connection = pool.get().expect("connection");
    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    for table in [
        "distribution_job",
        "distribution_job_target",
        "publisher_service_configuration_history",
        "publisher_distribution_platform",
    ] {
        assert_eq!(
            sql_query(format!("SELECT count(*) AS count FROM {table}"))
                .get_result::<Count>(&mut connection)
                .expect("count")
                .count,
            0,
            "{table} must carry no committed row"
        );
    }
    assert_eq!(token(&pool, publisher.publisher_id), before);
}

// ==========================================================================
// ADR-0008 boundary
// ==========================================================================

#[test]
fn no_generic_framework_universal_queue_or_shared_worker_convention_is_introduced() {
    let sources: Vec<(&str, &str)> = vec![
        (
            "model/distribution_job/mod.rs",
            include_str!("../model/distribution_job/mod.rs"),
        ),
        (
            "model/distribution_job/crud.rs",
            include_str!("../model/distribution_job/crud.rs"),
        ),
        ("policy.rs", include_str!("../policy.rs")),
        ("graphql/dataloader.rs", include_str!("dataloader.rs")),
    ];

    for (name, source) in &sources {
        for forbidden in [
            "trait Job",
            "trait Queue",
            "trait Lease",
            "trait Worker",
            "trait ServiceRole",
            "struct GenericJob",
            "struct JobQueue",
            "enum JobKind ",
            "MetricsJob",
            "SERVICE_ACCOUNT",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must introduce no generic job/queue/lease/worker abstraction: \
                 found `{forbidden}`"
            );
        }
    }

    // Every table, type, enum, constant and role code BE-04 adds is named for
    // distribution jobs and is unusable as a generic facility.
    let model = sources[0].1;
    for name in [
        "DistributionJobKind",
        "DistributionJobStatus",
        "DistributionJobAttemptResult",
        "DistributionJobCancellationReason",
        "DISTRIBUTION_JOB_MAX_ATTEMPTS",
    ] {
        assert!(model.contains(name));
    }
    assert!(model.contains("DISSEMINATION") || sources[2].1.contains("DisseminationWorker"));

    // No Metrics surface is reached, and no cross-programme reuse is implied.
    for (name, source) in &sources {
        assert!(
            !source.contains("MetricPlatform"),
            "{name} must not reach a Metrics surface"
        );
    }
}
