//! `MET-WP4-WIDGET-01` public widget read evidence at the API boundary.
//!
//! These tests exercise the real production schema and the real resolver: the
//! additive SDL contract, the uniform refusal for the switch, an unknown work
//! and an unentitled publisher, input-only failures that do not depend on the
//! work, the resolver's indifference to the caller's identity, and DOI
//! spellings. Every execution builds a fresh `Context`.
//!
//! Every assertion that names an engine statement, a captured statement
//! sequence or the statement ceiling lives in
//! `crate::model::metric_widget::tests`, as do the order-of-operations,
//! bounds, pair-mode and several-selection proofs.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest};
use crate::db::PgPool;
use crate::model::metric_dashboard::tests::{
    apply_all, commit, cover, day_n, exec, publisher_sql, setup, works_sql, Fixture,
};
use crate::model::metric_widget::MetricWidgetSwitch;
use crate::model::tests::db as test_db;
use crate::policy::Role;

// --------------------------------------------------------------------------
// Execution helpers
// --------------------------------------------------------------------------

const BOOK_DOI: &str = "https://doi.org/10.1234/abc";
const UNENTITLED_DOI: &str = "https://doi.org/10.1234/obelisk";

const WIDGET: &str = "query Widget($input: MetricWidgetInput!) { metricWidget(input: $input) { \
    work { totals { platformId measureId value } \
           timeline { platformId measureId startDate endDate value } \
           countries { platformId measureId countryCode value } \
           coverage { platformId measureId status dataThrough countryCoverage } \
           dataThrough isPartial } \
    chapters { totals { value } isPartial } \
    excludedChildCount \
    platforms { platformId code displayName publicDescription } \
    measures { measureId code displayName unit definition } \
    startDate endDate asOf rollupWatermark } }";

fn context(
    pool: &Arc<PgPool>,
    user: Option<IntrospectedUser>,
    switch: MetricWidgetSwitch,
) -> Context {
    let context = match user {
        Some(user) => test_db::test_context_with_user(Arc::clone(pool), user),
        None => test_db::test_context_anonymous(Arc::clone(pool)),
    };
    context.with_metric_widget(switch)
}

/// Execute the widget query once, with a fresh context.
async fn widget(
    pool: &Arc<PgPool>,
    user: Option<IntrospectedUser>,
    switch: MetricWidgetSwitch,
    input: JsonValue,
) -> JsonValue {
    let request: GraphQLRequest = serde_json::from_value(json!({
        "query": WIDGET,
        "variables": { "input": input },
    }))
    .expect("a GraphQL request");
    let response = request
        .execute(&create_schema(), &context(pool, user, switch))
        .await;
    serde_json::to_value(response).expect("a serialized response")
}

fn input(doi: &str) -> JsonValue {
    json!({ "doi": doi, "startDate": "2026-03-01", "endDate": "2026-04-01" })
}

fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("an errors array");
    assert_eq!(errors.len(), 1, "exactly one error: {response}");
    assert_eq!(response["data"], JsonValue::Null, "{response}");
    (
        errors[0]["message"]
            .as_str()
            .expect("a message")
            .to_string(),
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("an extensions.type")
            .to_string(),
    )
}

fn served(response: &JsonValue) -> JsonValue {
    assert!(
        response.get("errors").is_none(),
        "unexpected errors: {response}"
    );
    response["data"]["metricWidget"].clone()
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

/// The dashboard fixture with the book at [`BOOK_DOI`], one served pair, and
/// an `OBELISK` publisher's work at [`UNENTITLED_DOI`].
fn fixture() -> (test_db::TestDbGuard, Fixture) {
    let (guard, fx) = setup();
    let obelisk = Uuid::new_v4();
    let obelisk_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}UPDATE work SET doi = '{BOOK_DOI}' WHERE work_id = '{}'; \
             UPDATE work SET doi = '{UNENTITLED_DOI}' WHERE work_id = '{obelisk_work}';",
            publisher_sql(obelisk, "OBELISK"),
            works_sql(obelisk, &[obelisk_work]),
            fx.works[0],
        ),
    );
    cover(
        &fx,
        fx.sessions,
        day_n(1),
        day_n(32),
        "COMPLETE",
        "COMPLETED",
        "'2026-04-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 5);
    apply_all(&fx.pool);
    (guard, fx)
}

const UNAVAILABLE: (&str, &str) = (
    "Metrics are not available for this work.",
    "METRIC_WIDGET_UNAVAILABLE",
);

fn expected(message: &str, kind: &str) -> (String, String) {
    (message.to_string(), kind.to_string())
}

// --------------------------------------------------------------------------
// SDL contract (section 3)
// --------------------------------------------------------------------------

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
fn the_sdl_declares_exactly_the_widget_contract() {
    let sdl = create_schema().as_sdl();
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    assert!(
        query.contains("metricWidget(input:MetricWidgetInput!):MetricWidget!"),
        "{query}"
    );
    for (declaration, expected) in [
        (
            "input MetricWidgetInput {",
            "doi:Doi!chapterDois:[Doi!]startDate:Date!endDate:Date!",
        ),
        (
            "type MetricWidget {",
            "work:MetricWidgetSection!chapters:MetricWidgetSectionexcludedChildCount:Int!\
             platforms:[MetricWidgetPlatform!]!measures:[MetricWidgetMeasure!]!\
             startDate:Date!endDate:Date!asOf:Timestamp!rollupWatermark:Timestamp!",
        ),
        (
            "type MetricWidgetSection {",
            "totals:[MetricTotal!]!timeline:[MetricTimeBucket!]!\
             countries:[MetricCountryTotal!]!coverage:[MetricWidgetPairCoverage!]!\
             dataThrough:DateisPartial:Boolean!",
        ),
        (
            "type MetricWidgetPairCoverage {",
            "platformId:Uuid!measureId:Uuid!status:MetricCoverageStatus!dataThrough:Date\
             countryCoverage:Boolean!",
        ),
        (
            "type MetricWidgetPlatform {",
            "platformId:Uuid!code:String!displayName:String!publicDescription:String",
        ),
        (
            "type MetricWidgetMeasure {",
            "measureId:Uuid!code:String!displayName:String!unit:MetricMeasureUnit!\
             definition:String!",
        ),
    ] {
        assert_eq!(
            signatures(sdl_block(&sdl, declaration)),
            expected,
            "{declaration}"
        );
    }
}

#[test]
fn the_field_description_is_one_line_and_names_no_pinned_surface() {
    let sdl = create_schema().as_sdl();
    let query_root = sdl_block(&sdl, "type QueryRoot {");
    let lines: Vec<&str> = query_root.lines().collect();
    let field = lines
        .iter()
        .position(|line| line.trim_start().starts_with("metricWidget("))
        .expect("the field");
    let description = lines[field - 1].trim();
    assert!(
        description.starts_with('"')
            && description.ends_with('"')
            && !description.starts_with("\"\"\""),
        "a single-line description: {description}"
    );
    let field_text = format!("{description}\n{}", lines[field]);
    for forbidden in [
        "MetricImport",
        "MetricSourceUnitClaim",
        "MetricBatchResult",
        "Rollup",
        "rollupWatermark",
        "metricRollup",
        "workDayRollup",
        "CoverageRun",
        "coverageRun",
        "IdentifierQuarantineReconciliation",
        "MetricSourceCheckpoint",
        "claimMetricSource",
        "distributionJob(",
        "ThothPackage",
        "PublisherCapability",
        "METRICS_READ_SERVICE",
    ] {
        assert!(!field_text.contains(forbidden), "{forbidden}");
    }
    for forbidden in ["cursor", "manifest"] {
        assert!(
            !field_text.to_lowercase().contains(forbidden),
            "{forbidden}"
        );
    }
    // The widget types carry no institution, warning or code-only pair data.
    for declaration in [
        "type MetricWidget {",
        "type MetricWidgetSection {",
        "type MetricWidgetPairCoverage {",
        "type MetricWidgetPlatform {",
        "type MetricWidgetMeasure {",
        "input MetricWidgetInput {",
    ] {
        let block = sdl_block(&sdl, declaration);
        for absent in [
            "institution",
            "Institution",
            "warnings",
            "platformCode:",
            "ThothPackage",
            ": PublisherCapability",
            "timelineGrain",
        ] {
            assert!(!block.contains(absent), "{declaration} {absent}");
        }
    }
}

// --------------------------------------------------------------------------
// The uniform refusal and input failures (sections 4.1 and 4.8)
// --------------------------------------------------------------------------

#[test]
fn a_disabled_switch_refuses_every_request_without_the_database() {
    // A pool that cannot connect: the switch refuses before any checkout.
    let pool = Arc::new(test_db::failing_pool());
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    for doi in [BOOK_DOI, "10.1234/unknown", "not a doi", ""] {
        let response = runtime.block_on(widget(&pool, None, MetricWidgetSwitch::Off, input(doi)));
        assert_eq!(
            only_error(&response),
            expected(UNAVAILABLE.0, UNAVAILABLE.1),
            "{doi}"
        );
    }
    // Even an input failure is unavailable with the switch off.
    let mut bad_dates = input(BOOK_DOI);
    bad_dates["startDate"] = json!("2026-03-02");
    let response = runtime.block_on(widget(&pool, None, MetricWidgetSwitch::Off, bad_dates));
    assert_eq!(
        only_error(&response),
        expected(UNAVAILABLE.0, UNAVAILABLE.1)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_unknown_and_unentitled_are_one_indistinguishable_refusal() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    let off = widget(&pool, None, MetricWidgetSwitch::Off, input(BOOK_DOI)).await;
    let unknown = widget(
        &pool,
        None,
        MetricWidgetSwitch::On,
        input("10.1234/unknown"),
    )
    .await;
    let unentitled = widget(&pool, None, MetricWidgetSwitch::On, input(UNENTITLED_DOI)).await;
    assert_eq!(off, unknown);
    assert_eq!(unknown, unentitled);
    assert_eq!(only_error(&off), expected(UNAVAILABLE.0, UNAVAILABLE.1));
    // The same book is served with the switch on.
    let book = served(&widget(&pool, None, MetricWidgetSwitch::On, input(BOOK_DOI)).await);
    assert_eq!(book["work"]["totals"][0]["value"], json!("5"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn input_failures_are_identical_for_every_work_and_follow_step_4_order() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    let cases: Vec<(JsonValue, (String, String))> = vec![
        (
            json!({"startDate": "2026-03-02", "endDate": "2026-04-01"}),
            expected(
                "startDate and endDate must be first days of months, with startDate before endDate.",
                "METRIC_QUERY_INVALID",
            ),
        ),
        (
            json!({"startDate": "2026-04-01", "endDate": "2026-03-01"}),
            expected(
                "startDate and endDate must be first days of months, with startDate before endDate.",
                "METRIC_QUERY_INVALID",
            ),
        ),
        (
            json!({"startDate": "1999-01-01", "endDate": "1999-02-01"}),
            expected(
                "startDate must be on or after 2000-01-01, and endDate at most 366 days after today (UTC).",
                "METRIC_QUERY_LIMIT_EXCEEDED",
            ),
        ),
        (
            json!({"startDate": "2024-01-01", "endDate": "2025-02-01"}),
            expected(
                "The date range may cover at most 12 months.",
                "METRIC_QUERY_LIMIT_EXCEEDED",
            ),
        ),
        (
            json!({"chapterDois": ["10.1234/a", "10.1234/A"]}),
            expected("chapterDois must not repeat a DOI.", "METRIC_QUERY_INVALID"),
        ),
        (
            json!({"chapterDois": ["10.1234/a", "nonsense"]}),
            expected("doi and chapterDois must be valid DOIs.", "METRIC_QUERY_INVALID"),
        ),
        (
            // Several failures at once: the dates come first.
            json!({"startDate": "2026-03-02", "endDate": "2028-04-01",
                   "chapterDois": ["nonsense", "nonsense"]}),
            expected(
                "startDate and endDate must be first days of months, with startDate before endDate.",
                "METRIC_QUERY_INVALID",
            ),
        ),
    ];
    for (overrides, error) in cases {
        let mut responses = Vec::new();
        for doi in [BOOK_DOI, UNENTITLED_DOI, "10.1234/unknown"] {
            let mut request = input(doi);
            for (key, value) in overrides.as_object().expect("an object") {
                request[key] = value.clone();
            }
            let response = widget(&pool, None, MetricWidgetSwitch::On, request).await;
            assert_eq!(only_error(&response), error, "{doi} {overrides}");
            responses.push(response);
        }
        assert!(responses.windows(2).all(|pair| pair[0] == pair[1]));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_malformed_doi_reaches_the_resolver_and_is_invalid_only_when_on() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    for doi in ["not a doi", "10.1234/abc ", "doi:10.1234/abc", ""] {
        let on = widget(&pool, None, MetricWidgetSwitch::On, input(doi)).await;
        assert_eq!(
            only_error(&on),
            expected(
                "doi and chapterDois must be valid DOIs.",
                "METRIC_QUERY_INVALID"
            ),
            "{doi}"
        );
        let off = widget(&pool, None, MetricWidgetSwitch::Off, input(doi)).await;
        assert_eq!(
            only_error(&off),
            expected(UNAVAILABLE.0, UNAVAILABLE.1),
            "{doi}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_three_doi_spellings_resolve_the_same_work() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    let mut works = Vec::new();
    for doi in [
        "10.1234/abc",
        "https://doi.org/10.1234/abc",
        "HTTP://DX.DOI.ORG/10.1234/ABC",
    ] {
        let mut widget = served(&widget(&pool, None, MetricWidgetSwitch::On, input(doi)).await);
        widget["asOf"] = JsonValue::Null;
        works.push(widget);
    }
    assert_eq!(works[0]["work"]["totals"][0]["value"], json!("5"));
    assert!(works.windows(2).all(|pair| pair[0] == pair[1]));
}

// --------------------------------------------------------------------------
// The caller's identity is never consulted (section 2, ADR-0014 s4.4)
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_caller_gets_the_same_result() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    // `IntrospectedUser` is not `Clone`, so each caller is built afresh.
    let callers: [fn() -> Option<IntrospectedUser>; 5] = [
        || None,
        || Some(user_with("no-role", &[])),
        || {
            Some(user_with(
                "other-publisher-admin",
                &[(Role::PublisherAdmin, "org-of-another-publisher")],
            ))
        },
        || Some(user_with("reader", &[(Role::MetricsReadService, "org-1")])),
        || Some(test_db::test_superuser("superuser")),
    ];
    for doi in [BOOK_DOI, UNENTITLED_DOI, "not a doi"] {
        let mut results = Vec::new();
        for caller in callers {
            let mut response = widget(&pool, caller(), MetricWidgetSwitch::On, input(doi)).await;
            if let Some(widget) = response["data"]["metricWidget"].as_object_mut() {
                widget.remove("asOf");
            }
            results.push(response);
        }
        assert!(
            results.windows(2).all(|pair| pair[0] == pair[1]),
            "{doi}: {results:?}"
        );
    }
}

#[test]
fn the_resolver_reads_no_identity_and_no_role_guard() {
    let source = include_str!("query.rs");
    let start = source
        .find("fn metric_widget(")
        .expect("the widget resolver");
    let body = &source[start..];
    let body = &body[..body.find("\n    }\n").expect("the resolver body")];
    for absent in ["context.user", "require_", "policy", "MetricReadError"] {
        assert!(!body.contains(absent), "{absent}: {body}");
    }
    assert!(body.contains("context.metric_widget_claims"));
}
