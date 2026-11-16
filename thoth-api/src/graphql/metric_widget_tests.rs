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

/// Execute the widget query once with `literal` as an inline input object
/// and no variables, with a fresh context.
async fn widget_inline(pool: &Arc<PgPool>, switch: MetricWidgetSwitch, literal: &str) -> JsonValue {
    let query = WIDGET.replacen(
        "query Widget($input: MetricWidgetInput!) { metricWidget(input: $input)",
        &format!("query Widget {{ metricWidget(input: {literal})"),
        1,
    );
    assert_ne!(query, WIDGET, "the inline document differs");
    let request: GraphQLRequest =
        serde_json::from_value(json!({ "query": query })).expect("a GraphQL request");
    let response = request
        .execute(&create_schema(), &context(pool, None, switch))
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
            "doi:Doi!chapterDois:[Doi!]startDate:DateendDate:Date!timelineGrain:MetricWidgetGrain=",
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
            // The input declares the grain; the output types never do.
            if absent == "timelineGrain" && declaration == "input MetricWidgetInput {" {
                continue;
            }
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

/// The two new input shapes: an all-history `YEAR` read and a `YEAR` read
/// with a start.
fn year_inputs(doi: &str) -> [JsonValue; 2] {
    [
        json!({ "doi": doi, "endDate": "2026-04-01", "timelineGrain": "YEAR" }),
        json!({
            "doi": doi,
            "startDate": "2025-01-01",
            "endDate": "2026-04-01",
            "timelineGrain": "YEAR",
        }),
    ]
}

#[test]
fn a_disabled_switch_refuses_year_requests_without_the_database() {
    // A pool that cannot connect: the switch refuses before any checkout.
    let pool = Arc::new(test_db::failing_pool());
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    for doi in [BOOK_DOI, UNENTITLED_DOI, "10.1234/unknown", "not a doi", ""] {
        for request in year_inputs(doi) {
            let response = runtime.block_on(widget(
                &pool,
                None,
                MetricWidgetSwitch::Off,
                request.clone(),
            ));
            assert_eq!(
                only_error(&response),
                expected(UNAVAILABLE.0, UNAVAILABLE.1),
                "{request}"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_unknown_and_unentitled_year_requests_are_the_month_refusal() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    let text = |response: &JsonValue| serde_json::to_string(response).expect("a JSON text");
    // R11's MONTH refusal, as the existing test produces it.
    let month = text(&widget(&pool, None, MetricWidgetSwitch::Off, input(BOOK_DOI)).await);
    for doi in ["10.1234/unknown", UNENTITLED_DOI] {
        let response = widget(&pool, None, MetricWidgetSwitch::On, input(doi)).await;
        assert_eq!(text(&response), month, "{doi}");
    }
    assert_eq!(
        only_error(&serde_json::from_str(&month).expect("a JSON value")),
        expected(UNAVAILABLE.0, UNAVAILABLE.1)
    );
    for (switch, doi) in [
        (MetricWidgetSwitch::Off, BOOK_DOI),
        (MetricWidgetSwitch::On, "10.1234/unknown"),
        (MetricWidgetSwitch::On, UNENTITLED_DOI),
    ] {
        for request in year_inputs(doi) {
            let response = widget(&pool, None, switch, request.clone()).await;
            assert_eq!(text(&response), month, "{switch:?} {request}");
        }
    }
    // The same book is served with the switch on, for both shapes.
    for request in year_inputs(BOOK_DOI) {
        let book = served(&widget(&pool, None, MetricWidgetSwitch::On, request).await);
        assert_eq!(book["work"]["totals"][0]["value"], json!("5"));
    }
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
                "A MONTH timeline may cover at most 12 months.",
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

// --------------------------------------------------------------------------
// MET-WP4-WIDGET-ALLTIME: the grain and the optional start (section 3)
// --------------------------------------------------------------------------

/// The rendered description of `declaration` (`member` `None`), or of one
/// field, input field or enum value of it. Input fields and enum values
/// carry theirs inline, before the name; object fields on the line before.
fn description_of(sdl: &str, declaration: &str, member: Option<&str>) -> String {
    let unquote = |line: &str| {
        let line = line.trim();
        line.strip_prefix('"')
            .and_then(|body| body.strip_suffix('"'))
            .filter(|body| !body.starts_with('"'))
            .unwrap_or_else(|| panic!("a single-line description, found `{line}`"))
            .to_string()
    };
    let Some(member) = member else {
        let previous = sdl
            .split_once(declaration)
            .unwrap_or_else(|| panic!("SDL must declare `{declaration}`"))
            .0
            .lines()
            .next_back()
            .expect("a preceding line");
        return unquote(previous);
    };
    let lines: Vec<&str> = sdl_block(sdl, declaration).lines().collect();
    for (at, line) in lines.iter().enumerate() {
        let line = line.trim();
        if let Some((text, rest)) = line
            .strip_prefix('"')
            .and_then(|body| body.split_once("\" "))
        {
            if rest == member || rest.starts_with(&format!("{member}:")) {
                return text.to_string();
            }
        }
        if line == member
            || line.starts_with(&format!("{member}:"))
            || line.starts_with(&format!("{member}("))
        {
            return unquote(lines[at - 1]);
        }
    }
    panic!("`{declaration}` must declare `{member}`: {lines:#?}")
}

/// Every description this stage writes, as section 3 states it.
fn alltime_descriptions() -> Vec<(&'static str, Option<&'static str>, &'static str)> {
    vec![
        (
            "input MetricWidgetInput {",
            None,
            "A public widget request: one work by DOI, optionally some of its child works, a range of whole calendar months, and a monthly or yearly timeline",
        ),
        (
            "input MetricWidgetInput {",
            Some("startDate"),
            "First day of the range, inclusive: the first day of a month, on or after 2000-01-01. Required for a MONTH timeline; for a YEAR timeline it may be omitted to serve all history",
        ),
        (
            "input MetricWidgetInput {",
            Some("endDate"),
            "First day of the month after the range, exclusive, later than startDate and at most 366 days after today (UTC). A MONTH timeline may cover at most 12 months; a YEAR timeline has no maximum. When startDate is omitted, endDate must be after 2000-01-01",
        ),
        (
            "input MetricWidgetInput {",
            Some("timelineGrain"),
            "The size of each timeline bucket. Omitted or null means MONTH",
        ),
        (
            "enum MetricWidgetGrain {",
            None,
            "The size of each timeline bucket in a widget response",
        ),
        (
            "enum MetricWidgetGrain {",
            Some("MONTH"),
            "One bucket per calendar month of the served range",
        ),
        (
            "enum MetricWidgetGrain {",
            Some("YEAR"),
            "One bucket per calendar year, clipped to the served range. Each bucket is the exact sum of the daily values it contains",
        ),
        (
            "type MetricWidget {",
            None,
            "Public metrics for one work and its child works, read from one consistent database snapshot",
        ),
        (
            "type MetricWidget {",
            Some("startDate"),
            "First day served, inclusive: startDate when given, otherwise the first day of the earliest month with a served value in either section, or of the month before endDate when there is none",
        ),
        (
            "type MetricWidgetSection {",
            None,
            "Values and coverage of one widget section, for each platform and measure it serves",
        ),
        (
            "type MetricWidgetSection {",
            Some("timeline"),
            "One bucket per calendar month, or per calendar year clipped to the served range, for each served platform and measure, ordered by platform ID, measure ID and bucket start",
        ),
        (
            "type MetricWidgetSection {",
            Some("totals"),
            "One total per served platform and measure, ordered by platform ID and measure ID",
        ),
        (
            "type MetricWidgetSection {",
            Some("countries"),
            "Known per-country totals over the served range for the served platforms and measures, ordered by platform ID, measure ID and country code. A country without a row is not a zero",
        ),
        (
            "type MetricWidgetSection {",
            Some("dataThrough"),
            "The earliest per-pair dataThrough date, or null unless every served pair has one",
        ),
        (
            "type MetricWidgetSection {",
            Some("isPartial"),
            "True when coverage is not complete for some served platform, measure and day, when some values may still change or be incomplete, or when nothing is served",
        ),
        (
            "type MetricWidgetPairCoverage {",
            Some("status"),
            "COMPLETE only when every day of the served range is completely covered, including by country; UNKNOWN when any day has no coverage; otherwise PARTIAL",
        ),
        (
            "type MetricWidgetPairCoverage {",
            Some("dataThrough"),
            "The last day of the unbroken run from the served startDate in which every day is completely covered and its values settled, or null when the first day is not",
        ),
        (
            "type MetricWidgetPairCoverage {",
            Some("countryCoverage"),
            "True only when every day of the served range has coverage that includes the country dimension",
        ),
    ]
}

#[test]
fn the_sdl_declares_the_widget_grain_and_its_default() {
    let sdl = create_schema().as_sdl();
    let input = sdl_block(&sdl, "input MetricWidgetInput {");
    assert!(
        input.contains("timelineGrain: MetricWidgetGrain = \"MONTH\""),
        "{input}"
    );
    assert!(!input.contains("MetricTimelineGrain"), "{input}");
    assert_eq!(
        signatures(sdl_block(&sdl, "enum MetricWidgetGrain {")),
        "MONTHYEAR"
    );
    assert_eq!(sdl.matches("enum MetricWidgetGrain {").count(), 1);
}

#[test]
fn every_alltime_description_is_the_specified_text_and_names_no_pinned_surface() {
    let sdl = create_schema().as_sdl();
    let field = "Public, read-only Metrics for one work and, optionally, its child works, for the embeddable Metrics widget. No login is needed. Served only while the server enables it and only for works whose publisher is entitled to the Metrics widget. Serves monthly timelines of at most 12 months, or yearly timelines over all history, and only one metricWidget field may be requested at a time";
    assert_eq!(
        description_of(&sdl, "type QueryRoot {", Some("metricWidget")),
        field
    );
    let argument = "The work, optional child works, the date range and the timeline grain";
    assert!(sdl_block(&sdl, "type QueryRoot {").contains(&format!(
        "metricWidget(\"{argument}\" input: MetricWidgetInput!): MetricWidget!"
    )));
    let mut texts = vec![field, argument];
    for (declaration, member, text) in alltime_descriptions() {
        assert_eq!(
            description_of(&sdl, declaration, member),
            text,
            "{declaration} {member:?}"
        );
        texts.push(text);
    }
    for text in texts {
        for pinned in [
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
            "metricPlatformMeasures",
            "deleteMetric",
            "metricRegistryHistory",
            "metricPublisherPlatformApproval",
            "metricSources",
            "metricSourceAccounts",
            "metricSourceCheckpoint",
            "beforeState",
            "afterState",
            "MetricRegistryHistory",
            "monthAmbiguity",
            "rollupWorkMonth",
            "MetricRollupWorkMonth",
            "MetricRollupWorkDayState",
            "metricCoverageRun",
            "coverageRuns(",
            "MetricReconciliationRun",
            "metricReconciliationRun",
            "metricReconciliationIssue",
            "recordMetricReconciliation",
            "platformCode:",
            "require_",
            "metric_dashboard(",
            "institution",
            "Institution",
            "warnings",
            "timelineGrain",
        ] {
            assert!(!text.contains(pinned), "{pinned}: {text}");
        }
        for pinned in ["cursor", "manifest", "rollup"] {
            assert!(!text.to_lowercase().contains(pinned), "{pinned}: {text}");
        }
    }
}

fn without_as_of(mut response: JsonValue) -> JsonValue {
    if let Some(widget) = response["data"]["metricWidget"].as_object_mut() {
        widget.remove("asOf");
    }
    response
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_omitted_or_null_grain_is_a_month_read_and_needs_a_start() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    // Four months across one 1 January: four MONTH buckets per pair, two
    // YEAR buckets.
    let window = |grain: Option<JsonValue>| {
        let mut request =
            json!({ "doi": BOOK_DOI, "startDate": "2025-12-01", "endDate": "2026-04-01" });
        if let Some(grain) = grain {
            request["timelineGrain"] = grain;
        }
        request
    };
    let timeline_len = |response: &JsonValue| {
        let widget = served(response);
        let pairs = widget["work"]["totals"].as_array().map_or(0, Vec::len);
        assert_eq!(pairs, 1, "{response}");
        widget["work"]["timeline"].as_array().map_or(0, Vec::len) / pairs
    };
    let explicit = widget(
        &pool,
        None,
        MetricWidgetSwitch::On,
        window(Some(json!("MONTH"))),
    )
    .await;
    assert_eq!(timeline_len(&explicit), 4);
    for other in [window(None), window(Some(JsonValue::Null))] {
        let response = widget(&pool, None, MetricWidgetSwitch::On, other).await;
        assert_eq!(timeline_len(&response), 4);
        assert_eq!(without_as_of(response), without_as_of(explicit.clone()));
    }
    let year = widget(
        &pool,
        None,
        MetricWidgetSwitch::On,
        window(Some(json!("YEAR"))),
    )
    .await;
    assert_eq!(timeline_len(&year), 2);
    assert_eq!(
        served(&year)["work"]["timeline"],
        json!([
            {
                "platformId": fx.platform_id,
                "measureId": fx.sessions,
                "startDate": "2025-12-01",
                "endDate": "2026-01-01",
                "value": null,
            },
            {
                "platformId": fx.platform_id,
                "measureId": fx.sessions,
                "startDate": "2026-01-01",
                "endDate": "2026-04-01",
                "value": "5",
            },
        ])
    );
    let required = expected(
        "startDate is required for a MONTH timeline.",
        "METRIC_QUERY_INVALID",
    );
    for grain in [None, Some(JsonValue::Null), Some(json!("MONTH"))] {
        for doi in [BOOK_DOI, UNENTITLED_DOI, "10.1234/unknown"] {
            let mut request = json!({ "doi": doi, "endDate": "2026-04-01" });
            if let Some(grain) = &grain {
                request["timelineGrain"] = grain.clone();
            }
            let response = widget(&pool, None, MetricWidgetSwitch::On, request).await;
            assert_eq!(only_error(&response), required, "{grain:?} {doi}");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_null_start_on_a_year_read_is_an_omitted_start() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    let omitted = json!({ "doi": BOOK_DOI, "endDate": "2026-04-01", "timelineGrain": "YEAR" });
    let mut null = omitted.clone();
    null["startDate"] = JsonValue::Null;
    let first = widget(&pool, None, MetricWidgetSwitch::On, omitted).await;
    let second = widget(&pool, None, MetricWidgetSwitch::On, null).await;
    assert_eq!(without_as_of(first.clone()), without_as_of(second));
    let history = served(&first);
    assert_eq!(history["startDate"], json!("2026-03-01"));
    assert_eq!(history["work"]["totals"][0]["value"], json!("5"));
    assert_eq!(
        history["work"]["timeline"],
        json!([{
            "platformId": fx.platform_id,
            "measureId": fx.sessions,
            "startDate": "2026-03-01",
            "endDate": "2026-04-01",
            "value": "5",
        }])
    );
    // An omitted start needs an endDate after 2000-01-01.
    let early = json!({ "doi": BOOK_DOI, "endDate": "2000-01-01", "timelineGrain": "YEAR" });
    assert_eq!(
        only_error(&widget(&pool, None, MetricWidgetSwitch::On, early).await),
        expected(
            "When startDate is omitted, endDate must be after 2000-01-01 and at most 366 days after today (UTC).",
            "METRIC_QUERY_LIMIT_EXCEEDED",
        )
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn inline_null_grain_and_start_literals_equal_their_variable_cases() {
    let (_guard, fx) = fixture();
    let pool = Arc::clone(&fx.pool);
    // An inline `timelineGrain: null` with a start is the variable case.
    let mut variable = input(BOOK_DOI);
    variable["timelineGrain"] = JsonValue::Null;
    let by_variable = widget(&pool, None, MetricWidgetSwitch::On, variable).await;
    let inline = widget_inline(
        &pool,
        MetricWidgetSwitch::On,
        &format!(
            "{{doi: \"{BOOK_DOI}\", startDate: \"2026-03-01\", endDate: \"2026-04-01\", \
             timelineGrain: null}}"
        ),
    )
    .await;
    served(&inline);
    assert_eq!(without_as_of(inline), without_as_of(by_variable));
    // Without a start, it is the variable case's refusal.
    let by_variable = widget(
        &pool,
        None,
        MetricWidgetSwitch::On,
        json!({ "doi": BOOK_DOI, "endDate": "2026-04-01", "timelineGrain": null }),
    )
    .await;
    let inline = widget_inline(
        &pool,
        MetricWidgetSwitch::On,
        &format!("{{doi: \"{BOOK_DOI}\", endDate: \"2026-04-01\", timelineGrain: null}}"),
    )
    .await;
    assert_eq!(only_error(&inline), only_error(&by_variable));
    assert_eq!(
        only_error(&inline),
        expected(
            "startDate is required for a MONTH timeline.",
            "METRIC_QUERY_INVALID",
        )
    );
    // An inline `startDate: null` on a YEAR read is the variable case, and
    // so an omitted start.
    let by_variable = widget(
        &pool,
        None,
        MetricWidgetSwitch::On,
        json!({
            "doi": BOOK_DOI,
            "startDate": null,
            "endDate": "2026-04-01",
            "timelineGrain": "YEAR",
        }),
    )
    .await;
    let inline = widget_inline(
        &pool,
        MetricWidgetSwitch::On,
        &format!(
            "{{doi: \"{BOOK_DOI}\", startDate: null, endDate: \"2026-04-01\", \
             timelineGrain: YEAR}}"
        ),
    )
    .await;
    assert_eq!(served(&inline)["startDate"], json!("2026-03-01"));
    assert_eq!(without_as_of(inline), without_as_of(by_variable));
}
