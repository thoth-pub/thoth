//! `BE-02` public contract and first-production-DataLoader evidence.
//!
//! These tests exercise the real production schema, the real
//! `Publisher.distributionPlatforms` resolver and the real
//! `RequestLoaders` bundle. Generic scheduler behaviour of the foundation
//! itself is covered by `graphql::dataloader::tests`; everything here is
//! field-specific.

#![cfg(all(test, feature = "backend"))]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use diesel::{sql_query, RunQueryDsl};
use juniper::{graphql_object, DefaultScalarValue, EmptySubscription, FieldResult, RootNode};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::dataloader::fixture::{BatchStats, SqlProbe};
use super::dataloader::RequestLoaders;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::publisher_distribution_platform::{
    DistributionPlatform, PublisherDistributionPlatform, PublisherDistributionPlatformAssignment,
};
use crate::model::tests::db as test_db;

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

/// Anonymous context: every BE-02 read surface is intentionally public.
fn anonymous(pool: Arc<PgPool>) -> Context {
    test_db::test_context_anonymous(pool)
}

fn platforms_of(value: &JsonValue) -> Vec<String> {
    value
        .as_array()
        .expect("assignment array")
        .iter()
        .map(|assignment| {
            assignment["platform"]
                .as_str()
                .expect("platform")
                .to_string()
        })
        .collect()
}

// --------------------------------------------------------------------------
// distributionPlatformOptions
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn options_return_seventeen_descriptors_in_canonical_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let response = run(
        &create_schema(),
        &anonymous(pool),
        "{ distributionPlatformOptions { platform displayLabel linkedGroup \
          backCatalogueBehaviour assignable } }",
    )
    .await;
    let options = data(&response, "distributionPlatformOptions")
        .as_array()
        .expect("options");

    assert_eq!(options.len(), 17);
    let codes: Vec<&str> = options
        .iter()
        .map(|option| option["platform"].as_str().expect("platform"))
        .collect();
    assert_eq!(
        codes,
        vec![
            "INTERNET_ARCHIVE",
            "OAPEN",
            "DOAB",
            "SCIENCE_OPEN",
            "CAMBRIDGE_UNIVERSITY_LIBRARY",
            "CROSSREF",
            "FIGSHARE",
            "ZENODO",
            "PROJECT_MUSE",
            "JSTOR",
            "EBSCO_HOST",
            "PROQUEST_EBOOK_CENTRAL",
            "GOOGLE_PLAY",
            "BKCI",
            "OCLC_KB",
            "EX_LIBRIS_KB",
            "JISC_NBK",
        ]
    );

    assert_eq!(options[1]["displayLabel"], "OAPEN");
    assert_eq!(options[1]["linkedGroup"], "OAPEN_DOAB");
    assert_eq!(options[2]["linkedGroup"], "OAPEN_DOAB");
    assert_eq!(options[0]["backCatalogueBehaviour"], "AUTOMATIC_PUSH");
    assert_eq!(options[3]["backCatalogueBehaviour"], "MANUAL");
    assert_eq!(options[14]["backCatalogueBehaviour"], "PULL_FEED");

    let linked: Vec<&str> = options
        .iter()
        .filter(|option| !option["linkedGroup"].is_null())
        .map(|option| option["platform"].as_str().expect("platform"))
        .collect();
    assert_eq!(linked, vec!["OAPEN", "DOAB"]);

    let non_assignable: Vec<&str> = options
        .iter()
        .filter(|option| option["assignable"] == json!(false))
        .map(|option| option["platform"].as_str().expect("platform"))
        .collect();
    assert_eq!(non_assignable, vec!["JISC_NBK"]);
}

// --------------------------------------------------------------------------
// Publisher.distributionPlatforms
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn publisher_field_returns_enabled_assignments_only_in_canonical_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    for platform in [
        DistributionPlatform::Zenodo,
        DistributionPlatform::InternetArchive,
        DistributionPlatform::Oapen,
        DistributionPlatform::Crossref,
    ] {
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("enable");
    }
    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("disable");

    let response = run(
        &create_schema(),
        &anonymous(Arc::clone(&pool)),
        &format!(
            "{{ publisher(publisherId: \"{}\") {{ distributionPlatforms {{ platform enabledAt }} }} }}",
            publisher.publisher_id
        ),
    )
    .await;

    let assignments = &data(&response, "publisher")["distributionPlatforms"];
    assert_eq!(
        platforms_of(assignments),
        vec!["INTERNET_ARCHIVE", "OAPEN", "DOAB", "ZENODO"],
        "disabled CROSSREF must be excluded and order must be canonical"
    );
    assert!(assignments[0]["enabledAt"].as_str().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn publisher_field_is_an_empty_list_without_assignments() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    let response = run(
        &create_schema(),
        &anonymous(Arc::clone(&pool)),
        &format!(
            "{{ publisher(publisherId: \"{}\") {{ distributionPlatforms {{ platform }} }} }}",
            publisher.publisher_id
        ),
    )
    .await;

    assert_eq!(
        data(&response, "publisher")["distributionPlatforms"],
        json!([])
    );
}

// --------------------------------------------------------------------------
// Reverse lookup and count
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn reverse_lookup_and_count_cover_the_same_enabled_population() {
    let (_guard, pool) = test_db::setup_test_db();
    let enabled_one = test_db::create_publisher(&pool);
    let enabled_two = test_db::create_publisher(&pool);
    let disabled = test_db::create_publisher(&pool);
    let untouched = test_db::create_publisher(&pool);

    for publisher in [&enabled_one, &enabled_two, &disabled] {
        PublisherDistributionPlatform::enable(
            &pool,
            publisher.publisher_id,
            DistributionPlatform::Oapen,
        )
        .expect("enable");
    }
    PublisherDistributionPlatform::disable(
        &pool,
        disabled.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("disable");

    let response = run(
        &create_schema(),
        &anonymous(Arc::clone(&pool)),
        "{ publishersByDistributionPlatform(platform: OAPEN) { publisherId } \
           publisherCountByDistributionPlatform(platform: OAPEN) }",
    )
    .await;

    let ids: Vec<String> = data(&response, "publishersByDistributionPlatform")
        .as_array()
        .expect("publishers")
        .iter()
        .map(|publisher| publisher["publisherId"].as_str().expect("id").to_string())
        .collect();
    let mut expected = vec![
        enabled_one.publisher_id.to_string(),
        enabled_two.publisher_id.to_string(),
    ];
    expected.sort();
    let mut actual = ids.clone();
    actual.sort();
    assert_eq!(actual, expected);
    assert!(!ids.contains(&disabled.publisher_id.to_string()));
    assert!(!ids.contains(&untouched.publisher_id.to_string()));
    assert_eq!(response["data"]["publisherCountByDistributionPlatform"], 2);

    // The linked member reports the same population.
    let doab = run(
        &create_schema(),
        &anonymous(pool),
        "{ publisherCountByDistributionPlatform(platform: DOAB) }",
    )
    .await;
    assert_eq!(doab["data"]["publisherCountByDistributionPlatform"], 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn reverse_lookup_paginates_deterministically_through_ordering_ties() {
    let (_guard, pool) = test_db::setup_test_db();
    // `publisher_uniq_idx` is a unique index on `lower(publisher_name)`, so
    // literally duplicate publisher names cannot exist. The equivalent
    // ordering tie is produced on a nullable sort field: every fixture
    // publisher has a NULL shortname, so the primary sort key is identical
    // across the whole page set and only the mandatory `publisher_id ASC`
    // tie-breaker can make pagination deterministic.
    let mut ids = Vec::new();
    for _ in 0..5 {
        let publisher = test_db::create_publisher(&pool);
        assert!(publisher.publisher_shortname.is_none());
        PublisherDistributionPlatform::enable(
            &pool,
            publisher.publisher_id,
            DistributionPlatform::Jstor,
        )
        .expect("enable");
        ids.push(publisher.publisher_id.to_string());
    }
    ids.sort();

    let schema = create_schema();
    let context = anonymous(pool);
    let mut paged = Vec::new();
    for offset in 0..5 {
        let response = run(
            &schema,
            &context,
            &format!(
                "{{ publishersByDistributionPlatform(platform: JSTOR, limit: 1, offset: {offset}, \
                   order: {{ field: PUBLISHER_SHORTNAME, direction: ASC }}) {{ publisherId }} }}"
            ),
        )
        .await;
        let page = data(&response, "publishersByDistributionPlatform")
            .as_array()
            .expect("page")
            .clone();
        assert_eq!(page.len(), 1, "each page must return exactly one publisher");
        paged.push(page[0]["publisherId"].as_str().expect("id").to_string());
    }

    assert_eq!(
        paged, ids,
        "tied sort keys must fall back to publisher_id ASC, with no duplicate or skipped row"
    );

    // A descending primary sort keeps the same ascending tie-breaker.
    let descending = run(
        &schema,
        &context,
        "{ publishersByDistributionPlatform(platform: JSTOR, \
           order: { field: PUBLISHER_SHORTNAME, direction: DESC }) { publisherId } }",
    )
    .await;
    let descending_ids: Vec<String> = data(&descending, "publishersByDistributionPlatform")
        .as_array()
        .expect("publishers")
        .iter()
        .map(|publisher| publisher["publisherId"].as_str().expect("id").to_string())
        .collect();
    assert_eq!(descending_ids, ids);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn empty_reverse_reads_are_successful_and_never_broaden_scope() {
    let (_guard, pool) = test_db::setup_test_db();
    // Publishers exist, but none has an assignment.
    test_db::create_publisher(&pool);
    test_db::create_publisher(&pool);

    let response = run(
        &create_schema(),
        &anonymous(pool),
        "{ publishersByDistributionPlatform(platform: FIGSHARE) { publisherId } \
           publisherCountByDistributionPlatform(platform: FIGSHARE) \
           jisc: publishersByDistributionPlatform(platform: JISC_NBK) { publisherId } \
           jiscCount: publisherCountByDistributionPlatform(platform: JISC_NBK) }",
    )
    .await;

    assert_eq!(
        data(&response, "publishersByDistributionPlatform"),
        &json!([])
    );
    assert_eq!(response["data"]["publisherCountByDistributionPlatform"], 0);
    assert_eq!(response["data"]["jisc"], json!([]));
    assert_eq!(response["data"]["jiscCount"], 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn all_four_surfaces_answer_anonymously() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("enable");

    let context = anonymous(Arc::clone(&pool));
    assert!(context.user.is_none(), "context must be anonymous");
    let response = run(
        &create_schema(),
        &context,
        "{ distributionPlatformOptions { platform } \
           publishers { publisherId distributionPlatforms { platform } } \
           publishersByDistributionPlatform(platform: CROSSREF) { publisherId } \
           publisherCountByDistributionPlatform(platform: CROSSREF) }",
    )
    .await;

    assert_eq!(
        data(&response, "distributionPlatformOptions")
            .as_array()
            .expect("options")
            .len(),
        17
    );
    assert_eq!(
        platforms_of(&data(&response, "publishers")[0]["distributionPlatforms"]),
        vec!["CROSSREF"]
    );
    assert_eq!(
        data(&response, "publishersByDistributionPlatform")
            .as_array()
            .expect("publishers")
            .len(),
        1
    );
    assert_eq!(response["data"]["publisherCountByDistributionPlatform"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_invalid_platform_value_fails_input_coercion() {
    let (_guard, pool) = test_db::setup_test_db();
    let response = run(
        &create_schema(),
        &anonymous(pool),
        "{ publisherCountByDistributionPlatform(platform: OTHER) }",
    )
    .await;

    assert!(
        response.get("data").is_none() || response["data"].is_null(),
        "invalid enum must fail before the resolver: {response}"
    );
    assert!(!response["errors"].as_array().expect("errors").is_empty());
}

// --------------------------------------------------------------------------
// SDL contract inventory
// --------------------------------------------------------------------------

fn sdl() -> String {
    create_schema().as_sdl()
}

#[test]
fn sdl_adds_exactly_the_approved_public_inventory() {
    let sdl = sdl();

    for field in [
        "distributionPlatformOptions: [DistributionPlatformOption!]!",
        "publisherCountByDistributionPlatform(",
        "publishersByDistributionPlatform(",
        "distributionPlatforms: [PublisherDistributionPlatformAssignment!]!",
    ] {
        assert!(sdl.contains(field), "SDL must contain `{field}`");
    }

    for declaration in [
        "type DistributionPlatformOption {",
        "type PublisherDistributionPlatformAssignment {",
        "enum DistributionPlatform {",
        "enum DistributionPlatformGroup {",
        "enum BackCatalogueBehaviour {",
    ] {
        assert_eq!(
            sdl.matches(declaration).count(),
            1,
            "SDL must declare `{declaration}` exactly once"
        );
    }

    // Exactly 17 enum values, and the linked-group and behaviour vocabularies.
    let platform_enum = sdl
        .split_once("enum DistributionPlatform {")
        .expect("DistributionPlatform enum")
        .1
        .split_once('}')
        .expect("enum body")
        .0;
    for code in [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "DOAB",
        "SCIENCE_OPEN",
        "CAMBRIDGE_UNIVERSITY_LIBRARY",
        "CROSSREF",
        "FIGSHARE",
        "ZENODO",
        "PROJECT_MUSE",
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI",
        "OCLC_KB",
        "EX_LIBRIS_KB",
        "JISC_NBK",
    ] {
        assert!(platform_enum.contains(code), "{code} missing from SDL enum");
    }
    assert!(!platform_enum.contains("OTHER"));
    assert!(!platform_enum.contains("UNKNOWN"));

    let group_enum = sdl
        .split_once("enum DistributionPlatformGroup {")
        .expect("group enum")
        .1
        .split_once('}')
        .expect("enum body")
        .0;
    assert!(group_enum.contains("OAPEN_DOAB"));

    let behaviour_enum = sdl
        .split_once("enum BackCatalogueBehaviour {")
        .expect("behaviour enum")
        .1
        .split_once('}')
        .expect("enum body")
        .0;
    for value in ["AUTOMATIC_PUSH", "PULL_FEED", "MANUAL"] {
        assert!(behaviour_enum.contains(value));
    }
}

#[test]
fn sdl_exposes_no_internal_or_protected_distribution_state() {
    let sdl = sdl();

    // Internal Rust descriptor vocabularies stay out of the public contract.
    for internal in [
        "AssignmentAvailability",
        "MechanismReadiness",
        "DistributionAdapterProfile",
        "OAPEN_DOAB_SWORD",
        "OCLC_KBART_PUBLIC",
        "JISC_NBK_MARC_S3",
        "CROSSREF_DOI_DEPOSIT",
    ] {
        assert!(!sdl.contains(internal), "SDL must not expose `{internal}`");
    }

    // Activation identity, retained history, package/capability state and any
    // endpoint or credential identity stay out of the assignment type.
    let assignment_type = sdl
        .split_once("type PublisherDistributionPlatformAssignment {")
        .expect("assignment type")
        .1
        .split_once('}')
        .expect("type body")
        .0;
    for forbidden in [
        "activationId",
        "disabledAt",
        "enabled:",
        "createdAt",
        "updatedAt",
        "publisherId",
    ] {
        assert!(
            !assignment_type.contains(forbidden),
            "assignment type must not expose `{forbidden}`"
        );
    }
    // Exactly the two approved fields.
    assert!(assignment_type.contains("platform: DistributionPlatform!"));
    assert!(assignment_type.contains("enabledAt: Timestamp!"));

    let option_type = sdl
        .split_once("type DistributionPlatformOption {")
        .expect("option type")
        .1
        .split_once('}')
        .expect("type body")
        .0;
    for forbidden in [
        "adapterProfile",
        "mechanismReadiness",
        "endpoint",
        "host",
        "bucket",
        "account",
        "credential",
        "secret",
    ] {
        assert!(
            !option_type.contains(forbidden),
            "option type must not expose `{forbidden}`"
        );
    }

    // BE-02 exposes no package/capability state and no protected BE-03 surface.
    for forbidden in [
        "subscriptionPackage",
        "PublisherServiceConfiguration",
        "replacePublisherServiceConfiguration",
    ] {
        assert!(
            !sdl.contains(forbidden),
            "SDL must not expose `{forbidden}`"
        );
    }
}

#[test]
fn sdl_adds_no_distribution_mutation_input_scalar_or_interface() {
    let sdl = sdl();
    for forbidden in [
        "input DistributionPlatform",
        "input PublisherDistributionPlatform",
        "scalar DistributionPlatform",
        "interface DistributionPlatform",
        "enableDistributionPlatform",
        "disableDistributionPlatform",
        "setDistributionPlatform",
    ] {
        assert!(
            !sdl.contains(forbidden),
            "SDL must not declare `{forbidden}`"
        );
    }
}

// --------------------------------------------------------------------------
// DataLoader: configuration, loader-first, boundaries
// --------------------------------------------------------------------------

#[test]
fn the_field_resolver_is_loader_first_and_uses_try_load_only() {
    let source = include_str!("model.rs");
    let body = source
        .split_once("pub async fn distribution_platforms(")
        .expect("resolver")
        .1
        .split_once("\n    }\n")
        .expect("resolver body")
        .0;

    assert!(body.contains("try_load("), "the resolver must use try_load");
    assert!(
        !body.contains(".load("),
        "DataLoader::load may panic on a missing key and is not approved"
    );
    let (before_first_await, _) = body.split_once(".await").expect("awaited loader call");
    assert!(
        before_first_await.contains("try_load("),
        "no unrelated awaited work may precede the target try_load"
    );
}

#[test]
fn the_production_bundle_uses_the_explicit_200_10_configuration() {
    use super::dataloader::{LOADER_CONFIG, MAX_BATCH_SIZE, YIELD_COUNT};
    assert_eq!(MAX_BATCH_SIZE, 200);
    assert_eq!(YIELD_COUNT, 10);
    assert_eq!(LOADER_CONFIG.max_batch_size, 200);
    assert_eq!(LOADER_CONFIG.yield_count, 10);

    // The assignment loader is built through `configured_loader`, so the
    // configuration cannot silently fall back to crate defaults.
    let source = include_str!("dataloader.rs");
    let constructor = source
        .split_once("pub(crate) fn for_request(")
        .expect("production constructor")
        .1
        .split_once("\n    }\n")
        .expect("constructor body")
        .0;
    assert!(constructor.contains("configured_loader("));
    assert!(constructor.contains("PublisherDistributionPlatformBatcher"));
}

/// Seed `count` publishers, each with one enabled assignment.
///
/// Bulk SQL keeps the large query-count fixtures fast; the lifecycle itself is
/// proven through the domain path in the model tests.
fn seed_publishers_with_assignment(pool: &PgPool, count: usize, platform: &str) -> Vec<Uuid> {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher (publisher_id, publisher_name) \
         SELECT gen_random_uuid(), 'Seeded ' || lpad(i::text, 5, '0') \
         FROM generate_series(1, {count}) AS i"
    ))
    .execute(&mut connection)
    .expect("seed publishers");
    sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at) \
         SELECT publisher_id, '{platform}', true, gen_random_uuid(), now() FROM publisher"
    ))
    .execute(&mut connection)
    .expect("seed assignments");
    diesel::QueryDsl::select(
        crate::schema::publisher::table,
        crate::schema::publisher::publisher_id,
    )
    .load::<Uuid>(&mut connection)
    .expect("seeded ids")
}

/// The assignment statements issued by the DataLoader, as opposed to the root
/// query's own SQL.
///
/// The loader's statement selects directly `FROM "publisher_distribution_platform"`,
/// while `publishersByDistributionPlatform` reaches the same table through
/// `FROM ("publisher" INNER JOIN ...)`. Classifying on the `FROM` clause keeps
/// the two apart.
fn assignment_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| sql.contains("FROM \"publisher_distribution_platform\""))
        .cloned()
        .collect()
}

fn per_parent_assignment_statements(captured: &[String]) -> Vec<String> {
    captured
        .iter()
        .filter(|sql| {
            sql.contains("FROM \"publisher_distribution_platform\"") && !sql.contains("= ANY")
        })
        .cloned()
        .collect()
}

async fn measure(parent_count: usize, query: &str) -> (Vec<usize>, Vec<String>) {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    let ids = seed_publishers_with_assignment(&ordinary_pool, parent_count, "OAPEN");
    assert_eq!(ids.len(), parent_count);

    let probe = SqlProbe::install(&test_db::test_db_url());
    let stats = Arc::new(BatchStats::default());
    let mut context = anonymous(Arc::clone(&probe.pool));
    context.loaders =
        RequestLoaders::for_request_observed(Arc::clone(&probe.pool), Arc::clone(&stats));
    let schema = create_schema();

    probe.start();
    let response = run(&schema, &context, query).await;
    let captured = probe.captured_statements();

    let parents = response["data"]
        .as_object()
        .expect("data")
        .values()
        .next()
        .expect("parent field")
        .as_array()
        .expect("parents")
        .clone();
    assert_eq!(parents.len(), parent_count, "unexpected parent count");
    for parent in &parents {
        assert_eq!(
            platforms_of(&parent["distributionPlatforms"]),
            vec!["OAPEN"],
            "every parent must load its own enabled assignments"
        );
    }

    (stats.batch_sizes(), captured)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn publishers_shape_250_parents_use_two_set_based_assignment_statements() {
    let (sizes, captured) = measure(
        250,
        "{ publishers(limit: 250, order: { field: PUBLISHER_NAME, direction: ASC }) \
           { publisherId distributionPlatforms { platform enabledAt } } }",
    )
    .await;

    assert_eq!(sizes, vec![200, 50], "loader chunks");
    let assignment_sql = assignment_statements(&captured);
    assert_eq!(
        assignment_sql.len(),
        2,
        "expected exactly two set-based assignment statements, got: {assignment_sql:?}"
    );
    assert!(assignment_sql.iter().all(|sql| sql.contains("= ANY")));
    assert!(assignment_sql
        .iter()
        .all(|sql| sql.contains("\"publisher_distribution_platform\".\"enabled\"")));
    assert!(assignment_sql.iter().all(|sql| sql.contains("ORDER BY")));
    assert!(
        per_parent_assignment_statements(&captured).is_empty(),
        "no per-parent assignment statement may be issued"
    );
    // The parent root query's own SQL is measured separately.
    let root_sql: Vec<&String> = captured
        .iter()
        .filter(|sql| sql.contains("FROM \"publisher\""))
        .collect();
    assert_eq!(root_sql.len(), 1, "one root publisher statement");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn reverse_lookup_shape_250_parents_use_two_set_based_assignment_statements() {
    let (sizes, captured) = measure(
        250,
        "{ publishersByDistributionPlatform(platform: OAPEN, limit: 250) \
           { publisherId distributionPlatforms { platform enabledAt } } }",
    )
    .await;

    assert_eq!(sizes, vec![200, 50], "loader chunks");
    let assignment_sql = assignment_statements(&captured);
    assert_eq!(
        assignment_sql.len(),
        2,
        "expected exactly two set-based assignment statements, got: {assignment_sql:?}"
    );
    assert!(assignment_sql.iter().all(|sql| sql.contains("= ANY")));
    assert!(
        per_parent_assignment_statements(&captured).is_empty(),
        "no per-parent assignment statement may be issued"
    );
    // The reverse root query is one set-based join, not a loop.
    let root_sql: Vec<&String> = captured
        .iter()
        .filter(|sql| sql.contains("INNER JOIN \"publisher_distribution_platform\""))
        .collect();
    assert_eq!(root_sql.len(), 1, "one set-based reverse-lookup statement");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn assignment_sql_count_tracks_configured_chunks_not_parent_count() {
    for (parents, expected_sizes) in [
        (1usize, vec![1usize]),
        (100, vec![100]),
        (200, vec![200]),
        (201, vec![200, 1]),
    ] {
        let (sizes, captured) = measure(
            parents,
            &format!(
                "{{ publishers(limit: {parents}, order: {{ field: PUBLISHER_NAME, direction: ASC }}) \
                   {{ publisherId distributionPlatforms {{ platform }} }} }}"
            ),
        )
        .await;
        assert_eq!(sizes, expected_sizes, "N={parents} loader chunks");
        assert_eq!(
            assignment_statements(&captured).len(),
            expected_sizes.len(),
            "N={parents} assignment statements"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn five_hundred_ready_keys_dispatch_in_three_configured_chunks() {
    let (sizes, captured) = measure(
        500,
        "{ publishers(limit: 500, order: { field: PUBLISHER_NAME, direction: ASC }) \
           { publisherId distributionPlatforms { platform } } }",
    )
    .await;
    assert_eq!(sizes, vec![200, 200, 100]);
    assert_eq!(assignment_statements(&captured).len(), 3);
}

#[tokio::test]
async fn batch_boundaries_hold_on_a_current_thread_runtime() {
    let (sizes, captured) = measure(
        201,
        "{ publishers(limit: 201, order: { field: PUBLISHER_NAME, direction: ASC }) \
           { publisherId distributionPlatforms { platform } } }",
    )
    .await;
    assert_eq!(sizes, vec![200, 1]);
    assert_eq!(assignment_statements(&captured).len(), 2);
}

// --------------------------------------------------------------------------
// DataLoader: totality, request scope, freshness
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_batch_is_total_for_a_publisher_with_no_enabled_assignments() {
    let (_guard, pool) = test_db::setup_test_db();
    let with_none = test_db::create_publisher(&pool);
    let with_disabled = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        with_disabled.publisher_id,
        DistributionPlatform::Zenodo,
    )
    .expect("enable");
    PublisherDistributionPlatform::disable(
        &pool,
        with_disabled.publisher_id,
        DistributionPlatform::Zenodo,
    )
    .expect("disable");

    let context = anonymous(Arc::clone(&pool));
    let loader = &context.loaders.publisher_distribution_platforms;
    for publisher_id in [with_none.publisher_id, with_disabled.publisher_id] {
        let value = loader
            .try_load(publisher_id)
            .await
            .expect("key must be present in the batch result")
            .expect("successful empty relationship");
        assert!(value.is_empty());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_request_contexts_share_no_assignment_loader_state() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    let first = anonymous(Arc::clone(&pool));
    let before = first
        .loaders
        .publisher_distribution_platforms
        .try_load(publisher.publisher_id)
        .await
        .expect("key")
        .expect("value");
    assert!(before.is_empty());

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("enable");

    let second = anonymous(Arc::clone(&pool));
    let after = second
        .loaders
        .publisher_distribution_platforms
        .try_load(publisher.publisher_id)
        .await
        .expect("key")
        .expect("value");
    assert_eq!(
        after
            .iter()
            .map(|assignment| assignment.platform)
            .collect::<Vec<_>>(),
        vec![DistributionPlatform::Crossref],
        "a new request must observe current state, never a replayed value"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_completed_load_is_not_cached_within_one_request() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stats = Arc::new(BatchStats::default());
    let mut context = anonymous(Arc::clone(&pool));
    context.loaders = RequestLoaders::for_request_observed(Arc::clone(&pool), Arc::clone(&stats));

    let first = context
        .loaders
        .publisher_distribution_platforms
        .try_load(publisher.publisher_id)
        .await
        .expect("key")
        .expect("value");
    assert!(first.is_empty());

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Bkci,
    )
    .expect("enable");

    let second = context
        .loaders
        .publisher_distribution_platforms
        .try_load(publisher.publisher_id)
        .await
        .expect("key")
        .expect("value");
    assert_eq!(second.len(), 1, "completed values must not be replayed");
    assert_eq!(stats.dispatch_count(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pending_duplicate_keys_coalesce_into_one_dispatch() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stats = Arc::new(BatchStats::default());
    let mut context = anonymous(Arc::clone(&pool));
    context.loaders = RequestLoaders::for_request_observed(Arc::clone(&pool), Arc::clone(&stats));
    let loader = &context.loaders.publisher_distribution_platforms;

    let (a, b) = tokio::join!(
        loader.try_load(publisher.publisher_id),
        loader.try_load(publisher.publisher_id)
    );
    assert!(a.expect("a key").is_ok());
    assert!(b.expect("b key").is_ok());
    assert_eq!(stats.dispatch_count(), 1);
    assert_eq!(stats.batch_sizes(), vec![1]);
}

// --------------------------------------------------------------------------
// DataLoader: failure semantics and direct-baseline equivalence
// --------------------------------------------------------------------------

/// A test-only schema whose `publishers` root field resolves
/// `distributionPlatforms` through the **direct synchronous** query, so the
/// loader-backed response can be compared against its baseline at an identical
/// GraphQL path.
struct DirectQuery;
struct DirectPublisher {
    publisher_id: Uuid,
}

type DirectSchema = RootNode<'static, DirectQuery, EmptyMutation, EmptySubscription<Context>>;
struct EmptyMutation;

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "BE02DirectQuery")]
impl DirectQuery {
    fn publishers(publisher_ids: Vec<Uuid>) -> Vec<DirectPublisher> {
        publisher_ids
            .into_iter()
            .map(|publisher_id| DirectPublisher { publisher_id })
            .collect()
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "BE02DirectMutation")]
impl EmptyMutation {
    fn noop() -> bool {
        true
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "BE02DirectPublisher")]
impl DirectPublisher {
    fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    fn distribution_platforms(
        &self,
        context: &Context,
    ) -> FieldResult<Vec<PublisherDistributionPlatformAssignment>> {
        PublisherDistributionPlatform::enabled_assignments(&context.db, self.publisher_id)
            .map_err(Into::into)
    }
}

fn failing_pool() -> Arc<PgPool> {
    Arc::new(test_db::failing_pool())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_backend_failure_errors_every_key_with_no_retry_and_no_fallback_sql() {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    let ids = seed_publishers_with_assignment(&ordinary_pool, 3, "OAPEN");

    // Loader level: every requested key in the failed chunk receives an error
    // value rather than a fabricated empty success or a missing entry, from
    // exactly one dispatch.
    let key_stats = Arc::new(BatchStats::default());
    let mut key_context = anonymous(Arc::clone(&ordinary_pool));
    key_context.loaders =
        RequestLoaders::for_request_observed(failing_pool(), Arc::clone(&key_stats));
    let loader = &key_context.loaders.publisher_distribution_platforms;
    let (a, b, c) = tokio::join!(
        loader.try_load(ids[0]),
        loader.try_load(ids[1]),
        loader.try_load(ids[2])
    );
    for outcome in [a, b, c] {
        assert!(
            outcome.expect("key present in failed batch").is_err(),
            "a failed batch must not fabricate empty success"
        );
    }
    assert_eq!(
        key_stats.dispatch_count(),
        1,
        "one failed dispatch, no retry"
    );

    // GraphQL level: the failure propagates through the non-null list and no
    // per-parent fallback SQL runs against the working backend.
    let probe = SqlProbe::install(&test_db::test_db_url());
    let stats = Arc::new(BatchStats::default());
    let mut context = anonymous(Arc::clone(&probe.pool));
    // The parent query still works; only the loader's backend fails.
    context.loaders = RequestLoaders::for_request_observed(failing_pool(), Arc::clone(&stats));
    let schema = create_schema();

    probe.start();
    let response = run(
        &schema,
        &context,
        "{ publishers(limit: 3) { publisherId distributionPlatforms { platform } } }",
    )
    .await;
    let captured = probe.captured_statements();

    assert!(
        response["data"].is_null(),
        "a non-null list must propagate the failure rather than fabricate empty data: {response}"
    );
    assert!(!response["errors"].as_array().expect("errors").is_empty());
    assert_eq!(stats.dispatch_count(), 1, "one failed dispatch, no retry");
    assert!(
        assignment_statements(&captured).is_empty(),
        "no direct per-parent fallback SQL may run after a failed batch"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn loader_failure_matches_the_direct_baseline_error_shape() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    // Loader-backed path: real schema, working parent query, failing loader.
    let mut loader_context = anonymous(Arc::clone(&pool));
    loader_context.loaders = RequestLoaders::for_request(failing_pool());
    let loader_response = run(
        &create_schema(),
        &loader_context,
        "{ publishers(limit: 1) { publisherId distributionPlatforms { platform } } }",
    )
    .await;

    // Direct baseline: the same field name at the same path, resolved
    // synchronously against the same failing backend.
    let direct_context = anonymous(failing_pool());
    let direct_schema = DirectSchema::new(DirectQuery, EmptyMutation, EmptySubscription::new());
    let direct_response = serde_json::to_value(
        request(&format!(
            "{{ publishers(publisherIds: [\"{}\"]) {{ publisherId distributionPlatforms {{ platform }} }} }}",
            publisher.publisher_id
        ))
        .execute(&direct_schema, &direct_context)
        .await,
    )
    .expect("serialize direct response");

    let loader_error = &loader_response["errors"][0];
    let direct_error = &direct_response["errors"][0];

    assert!(loader_response["data"].is_null());
    assert!(direct_response["data"].is_null());
    assert_eq!(
        loader_error["message"], direct_error["message"],
        "loader-backed message must equal the direct baseline"
    );
    assert_eq!(
        loader_error["path"],
        json!(["publishers", "distributionPlatforms"]),
        "the failure must surface at the owning child field path"
    );
    assert_eq!(loader_error["path"], direct_error["path"]);
    assert_eq!(
        loader_error.get("extensions"),
        direct_error.get("extensions"),
        "the field's conventional message-only error shape must be preserved"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn loader_and_direct_paths_agree_on_successful_membership_and_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    for platform in [
        DistributionPlatform::OclcKb,
        DistributionPlatform::Oapen,
        DistributionPlatform::InternetArchive,
    ] {
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("enable");
    }

    let loader_response = run(
        &create_schema(),
        &anonymous(Arc::clone(&pool)),
        &format!(
            "{{ publisher(publisherId: \"{}\") {{ distributionPlatforms {{ platform enabledAt }} }} }}",
            publisher.publisher_id
        ),
    )
    .await;
    let loader_assignments = &data(&loader_response, "publisher")["distributionPlatforms"];

    let direct_schema = DirectSchema::new(DirectQuery, EmptyMutation, EmptySubscription::new());
    let direct_response = serde_json::to_value(
        request(&format!(
            "{{ publishers(publisherIds: [\"{}\"]) {{ distributionPlatforms {{ platform enabledAt }} }} }}",
            publisher.publisher_id
        ))
        .execute(&direct_schema, &anonymous(pool))
        .await,
    )
    .expect("serialize direct response");
    let direct_assignments = &direct_response["data"]["publishers"][0]["distributionPlatforms"];

    assert_eq!(loader_assignments, direct_assignments);
    assert_eq!(
        platforms_of(loader_assignments),
        vec!["INTERNET_ARCHIVE", "OAPEN", "DOAB", "OCLC_KB"]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sibling_publishers_receive_their_own_assignments() {
    let (_guard, pool) = test_db::setup_test_db();
    let alpha = test_db::create_publisher(&pool);
    let beta = test_db::create_publisher(&pool);
    let gamma = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        alpha.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("enable");
    PublisherDistributionPlatform::enable(&pool, beta.publisher_id, DistributionPlatform::Jstor)
        .expect("enable");

    let response = run(
        &create_schema(),
        &anonymous(pool),
        "{ publishers(limit: 100) { publisherId distributionPlatforms { platform } } }",
    )
    .await;

    let mut seen = 0;
    for publisher in data(&response, "publishers")
        .as_array()
        .expect("publishers")
    {
        let id: Uuid = publisher["publisherId"]
            .as_str()
            .expect("id")
            .parse()
            .expect("uuid");
        let platforms = platforms_of(&publisher["distributionPlatforms"]);
        if id == alpha.publisher_id {
            assert_eq!(platforms, vec!["CROSSREF"]);
            seen += 1;
        } else if id == beta.publisher_id {
            assert_eq!(platforms, vec!["JSTOR"]);
            seen += 1;
        } else if id == gamma.publisher_id {
            assert!(platforms.is_empty());
            seen += 1;
        }
    }
    assert_eq!(seen, 3);
}

/// The resolver must call the loader exactly once per parent occurrence and
/// never aggregate keys by hand.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_parent_occurrence_registers_exactly_one_loader_key() {
    let (_guard, pool) = test_db::setup_test_db();
    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..5 {
        let publisher = test_db::create_publisher(&pool);
        PublisherDistributionPlatform::enable(
            &pool,
            publisher.publisher_id,
            DistributionPlatform::Zenodo,
        )
        .expect("enable");
        counter.fetch_add(1, Ordering::SeqCst);
    }
    let stats = Arc::new(BatchStats::default());
    let mut context = anonymous(Arc::clone(&pool));
    context.loaders = RequestLoaders::for_request_observed(Arc::clone(&pool), Arc::clone(&stats));

    let response = run(
        &create_schema(),
        &context,
        "{ publishers(limit: 100) { publisherId distributionPlatforms { platform } } }",
    )
    .await;

    assert_eq!(
        data(&response, "publishers")
            .as_array()
            .expect("publishers")
            .len(),
        5
    );
    assert_eq!(stats.batch_sizes(), vec![5]);
}
