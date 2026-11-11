//! `MET-WP10-FILTER-API-01` evidence for the public `workFilterOptions` read
//! at the API boundary and against a real database (spec section 11).
//!
//! Every test that reads through the schema holds the database test lock
//! (`setup_test_db`), and only T11 holds the process-wide slots across calls,
//! so no schema read can meet T11's held slots. Tests that lower a control
//! (T6, T9, T13, T14) call the builder directly with their own slot counter.

#![cfg(all(test, feature = "backend"))]

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use diesel::connection::SimpleConnection;
use diesel::sql_types::{Array, BigInt as SqlBigInt, Nullable, Text, Uuid as SqlUuid};
use diesel::RunQueryDsl;
use juniper::IntoFieldError;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest};
use crate::db::PgPool;
use crate::model::metric_dashboard::tests::capturing_pool;
use crate::model::metric_dashboard::WORKS_SQL;
use crate::model::tests::db::{
    failing_pool, setup_test_db, test_context_anonymous, test_context_with_user, test_superuser,
    TestDbGuard,
};
use crate::model::work_filter_options::{
    try_take_slot, work_filter_options_with, WorkFilterOptions, WorkFilterOptionsError,
    WorkFilterReadControls, WORK_FILTER_OPTIONS_SLOTS,
};

// --------------------------------------------------------------------------
// Fixture
// --------------------------------------------------------------------------

/// Publishers P1 and P2 with works, P3 with none, and P4, never selected,
/// whose works carry values of their own.
///
/// - P1: `w1a` (monograph, issued in S1, `eng`, funded by F1, a contributor
///   affiliated with A1, and `is-child-of` P4's `w4b`, which is issued in S5:
///   not a chapter, so S5 is not reached); `w1b` (edited book, `fre`, funded
///   by FA); `w1c` (book chapter, `is-child-of` P4's `w4a`, issued in S4: S4
///   is reached).
/// - P2: `w2a` (textbook, issued in S2, `eng`, funded by F1, a contributor
///   affiliated with FA).
/// - P4: `w4a` (monograph, issued in S4, `ger`, funded by F4, affiliated with
///   A4), `w4b` (journal issue, issued in S5), `w4c` (book set, issued in S6).
struct Fixture {
    pool: Arc<PgPool>,
    p1: Uuid,
    p2: Uuid,
    p3: Uuid,
    p4: Uuid,
    s1: Uuid,
    s2: Uuid,
    s4: Uuid,
    s5: Uuid,
    s6: Uuid,
    f1: Uuid,
    fa: Uuid,
    f4: Uuid,
    a1: Uuid,
    a4: Uuid,
}

/// Builds fixture SQL with unique imprint names and contribution ordinals.
#[derive(Default)]
struct Sql {
    text: String,
    ordinal: i32,
}

impl Sql {
    fn publisher(&mut self, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        self.text.push_str(&format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{id}', '{name}');"
        ));
        id
    }

    fn imprint(&mut self, publisher_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        self.text.push_str(&format!(
            "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) \
             VALUES ('{id}', '{publisher_id}', 'Imprint {id}');"
        ));
        id
    }

    fn work(&mut self, imprint_id: Uuid, work_type: &str) -> Uuid {
        let id = Uuid::new_v4();
        let edition = if work_type == "book-chapter" {
            "NULL"
        } else {
            "1"
        };
        self.text.push_str(&format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             VALUES ('{id}', '{work_type}', 'forthcoming', '{imprint_id}', {edition});"
        ));
        id
    }

    fn series(
        &mut self,
        imprint_id: Uuid,
        name: &str,
        series_type: &str,
        issns: (Option<&str>, Option<&str>),
    ) -> Uuid {
        let id = Uuid::new_v4();
        let quote = |value: Option<&str>| value.map_or("NULL".to_string(), |v| format!("'{v}'"));
        self.text.push_str(&format!(
            "INSERT INTO series (series_id, series_type, series_name, issn_print, issn_digital, \
                                 imprint_id) \
             VALUES ('{id}', '{series_type}', '{name}', {}, {}, '{imprint_id}');",
            quote(issns.0),
            quote(issns.1)
        ));
        id
    }

    fn issue(&mut self, series_id: Uuid, work_id: Uuid) {
        self.ordinal += 1;
        self.text.push_str(&format!(
            "INSERT INTO issue (series_id, work_id, issue_ordinal) \
             VALUES ('{series_id}', '{work_id}', {});",
            self.ordinal
        ));
    }

    /// `child` `is-child-of` `parent`, stored in both directions.
    fn child_of(&mut self, child: Uuid, parent: Uuid) {
        self.ordinal += 1;
        self.text.push_str(&format!(
            "INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, \
                                        relation_ordinal) \
             VALUES ('{child}', '{parent}', 'is-child-of', 1), \
                    ('{parent}', '{child}', 'has-child', {});",
            self.ordinal
        ));
    }

    fn language(&mut self, work_id: Uuid, code: &str, relation: &str) {
        self.text.push_str(&format!(
            "INSERT INTO language (work_id, language_code, language_relation) \
             VALUES ('{work_id}', '{code}', '{relation}');"
        ));
    }

    fn institution(&mut self, name: &str, ror: Option<&str>) -> Uuid {
        self.institution_with_id(Uuid::new_v4(), name, ror)
    }

    fn institution_with_id(&mut self, id: Uuid, name: &str, ror: Option<&str>) -> Uuid {
        let ror = ror.map_or("NULL".to_string(), |r| format!("'{r}'"));
        self.text.push_str(&format!(
            "INSERT INTO institution (institution_id, institution_name, ror) \
             VALUES ('{id}', '{name}', {ror});"
        ));
        id
    }

    fn funding(&mut self, work_id: Uuid, institution_id: Uuid) {
        self.text.push_str(&format!(
            "INSERT INTO funding (work_id, institution_id) \
             VALUES ('{work_id}', '{institution_id}');"
        ));
    }

    /// A new contribution to `work_id` affiliated with `institution_id`.
    fn affiliation(&mut self, work_id: Uuid, institution_id: Uuid) {
        self.ordinal += 1;
        let contributor_id = Uuid::new_v4();
        let contribution_id = Uuid::new_v4();
        self.text.push_str(&format!(
            "INSERT INTO contributor (contributor_id, last_name, full_name) \
             VALUES ('{contributor_id}', 'Author', 'An Author'); \
             INSERT INTO contribution (contribution_id, work_id, contributor_id, \
                                       contribution_type, main_contribution, last_name, \
                                       full_name, contribution_ordinal) \
             VALUES ('{contribution_id}', '{work_id}', '{contributor_id}', 'author', TRUE, \
                     'Author', 'An Author', {}); \
             INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             VALUES ('{contribution_id}', '{institution_id}', 1);",
            self.ordinal
        ));
    }

    fn run(&self, pool: &PgPool) {
        pool.get()
            .expect("a connection")
            .batch_execute(&self.text)
            .unwrap_or_else(|error| panic!("fixture failed: {error}\n{}", self.text));
    }
}

fn fixture() -> (TestDbGuard, Fixture) {
    let (guard, pool) = setup_test_db();
    let mut sql = Sql::default();
    let p1 = sql.publisher("Alpha Press");
    let p2 = sql.publisher("Beta Books");
    let p3 = sql.publisher("Gamma Empty");
    let p4 = sql.publisher("Delta Other");
    let (i1, i2, i4) = (sql.imprint(p1), sql.imprint(p2), sql.imprint(p4));
    let _ = sql.imprint(p3);

    let w1a = sql.work(i1, "monograph");
    let w1b = sql.work(i1, "edited-book");
    let w1c = sql.work(i1, "book-chapter");
    let w2a = sql.work(i2, "textbook");
    let w4a = sql.work(i4, "monograph");
    let w4b = sql.work(i4, "journal-issue");
    let w4c = sql.work(i4, "book-set");

    let s1 = sql.series(
        i1,
        "Series One",
        "book-series",
        (Some("1111-1111"), Some("1111-111X")),
    );
    let s2 = sql.series(i2, "Series Two", "journal", (None, None));
    let s4 = sql.series(
        i4,
        "Series Four Elsewhere",
        "book-series",
        (None, Some("4444-4444")),
    );
    let s5 = sql.series(i4, "Series Five Elsewhere", "journal", (None, None));
    let s6 = sql.series(i4, "Series Six Elsewhere", "journal", (None, None));
    sql.issue(s1, w1a);
    sql.issue(s2, w2a);
    sql.issue(s4, w4a);
    sql.issue(s5, w4b);
    sql.issue(s6, w4c);
    // T2: the chapter reaches S4 through its parent; the monograph's
    // parent is in S5, which it must not reach.
    sql.child_of(w1c, w4a);
    sql.child_of(w1a, w4b);

    sql.language(w1a, "eng", "original");
    sql.language(w1b, "fre", "translated-from");
    sql.language(w2a, "eng", "translated-into");
    sql.language(w4a, "ger", "original");

    let f1 = sql.institution("Funder One", Some("https://ror.org/05abcde12"));
    let fa = sql.institution("Both Funder And Host", None);
    let f4 = sql.institution("Other Funder", None);
    let a1 = sql.institution("Host One", Some("https://ror.org/01bcdef34"));
    let a4 = sql.institution("Other Host", None);
    sql.funding(w1a, f1);
    sql.funding(w2a, f1);
    sql.funding(w1b, fa);
    sql.funding(w4a, f4);
    sql.affiliation(w1a, a1);
    sql.affiliation(w2a, fa);
    sql.affiliation(w4a, a4);
    sql.run(&pool);

    (
        guard,
        Fixture {
            pool,
            p1,
            p2,
            p3,
            p4,
            s1,
            s2,
            s4,
            s5,
            s6,
            f1,
            fa,
            f4,
            a1,
            a4,
        },
    )
}

// --------------------------------------------------------------------------
// Execution helpers
// --------------------------------------------------------------------------

const OPTIONS: &str = "query Options($ids: [Uuid!]!) { workFilterOptions(publisherIds: $ids) { \
    publishers { publisherId publisherName } \
    series { seriesId seriesName seriesType issnPrint issnDigital } \
    languageCodes workTypes \
    fundingInstitutions { institutionId institutionName ror } \
    affiliationInstitutions { institutionId institutionName ror } } }";

async fn execute(context: &Context, query: &str, variables: JsonValue) -> JsonValue {
    let request: GraphQLRequest =
        serde_json::from_value(json!({ "query": query, "variables": variables }))
            .expect("a GraphQL request");
    serde_json::to_value(request.execute(&create_schema(), context).await)
        .expect("a serialized response")
}

/// The full query for `ids`, anonymously, with a fresh context.
async fn options(pool: &Arc<PgPool>, ids: &[Uuid]) -> JsonValue {
    let context = test_context_anonymous(Arc::clone(pool));
    execute(&context, OPTIONS, json!({ "ids": ids })).await
}

fn served(response: &JsonValue) -> JsonValue {
    assert!(
        response.get("errors").is_none(),
        "unexpected errors: {response}"
    );
    response["data"]["workFilterOptions"].clone()
}

/// The one error of a response with null data: its type and message.
fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("an errors array");
    assert_eq!(errors.len(), 1, "exactly one error: {response}");
    assert_eq!(response["data"], JsonValue::Null, "{response}");
    (
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("an extensions.type")
            .to_string(),
        errors[0]["message"]
            .as_str()
            .expect("a message")
            .to_string(),
    )
}

fn error_types(response: &JsonValue) -> Vec<String> {
    response["errors"]
        .as_array()
        .map(|errors| {
            errors
                .iter()
                .map(|error| {
                    error["extensions"]["type"]
                        .as_str()
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn institution(id: Uuid, name: &str, ror: Option<&str>) -> JsonValue {
    json!({ "institutionId": id, "institutionName": name, "ror": ror })
}

fn ids_of(list: &JsonValue, key: &str) -> BTreeSet<Uuid> {
    list.as_array()
        .expect("a list")
        .iter()
        .map(|entry| Uuid::parse_str(entry[key].as_str().expect("an ID")).expect("a UUID"))
        .collect()
}

fn controls(slots: &AtomicUsize) -> WorkFilterReadControls<'_> {
    WorkFilterReadControls {
        slots,
        ..WorkFilterReadControls::production()
    }
}

fn type_and_message(error: WorkFilterOptionsError) -> (String, String) {
    let field_error = error.into_field_error();
    (
        extension_type(&field_error),
        field_error.message().to_string(),
    )
}

fn extension_type(field_error: &juniper::FieldError) -> String {
    field_error
        .extensions()
        .as_object_value()
        .and_then(|object| object.get_field_value("type"))
        .and_then(|value| value.as_string_value())
        .expect("an extensions.type")
        .to_string()
}

fn expected(kind: &str, message: &str) -> (String, String) {
    (kind.to_string(), message.to_string())
}

// --------------------------------------------------------------------------
// T1 to T4: the lists, their parity paths, deduplication and order
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t1_the_fixture_lists_exactly_the_selected_publishers_values() {
    let (_guard, fx) = fixture();
    let data = served(&options(&fx.pool, &[fx.p1, fx.p2, fx.p3]).await);
    assert_eq!(
        data,
        json!({
            "publishers": [
                { "publisherId": fx.p1, "publisherName": "Alpha Press" },
                { "publisherId": fx.p2, "publisherName": "Beta Books" },
                { "publisherId": fx.p3, "publisherName": "Gamma Empty" },
            ],
            "series": [
                { "seriesId": fx.s4, "seriesName": "Series Four Elsewhere",
                  "seriesType": "BOOK_SERIES", "issnPrint": null, "issnDigital": "4444-4444" },
                { "seriesId": fx.s1, "seriesName": "Series One",
                  "seriesType": "BOOK_SERIES", "issnPrint": "1111-1111",
                  "issnDigital": "1111-111X" },
                { "seriesId": fx.s2, "seriesName": "Series Two",
                  "seriesType": "JOURNAL", "issnPrint": null, "issnDigital": null },
            ],
            "languageCodes": ["ENG", "FRE"],
            "workTypes": ["BOOK_CHAPTER", "EDITED_BOOK", "MONOGRAPH", "TEXTBOOK"],
            "fundingInstitutions": [
                institution(fx.fa, "Both Funder And Host", None),
                institution(fx.f1, "Funder One", Some("https://ror.org/05abcde12")),
            ],
            "affiliationInstitutions": [
                institution(fx.fa, "Both Funder And Host", None),
                institution(fx.a1, "Host One", Some("https://ror.org/01bcdef34")),
            ],
        })
    );
    // P4's own values appear nowhere.
    let text = data.to_string();
    for absent in [fx.p4, fx.s5, fx.s6, fx.f4, fx.a4] {
        assert!(!text.contains(&absent.to_string()), "{absent}");
    }
    for absent in ["GER", "JOURNAL_ISSUE", "BOOK_SET"] {
        assert!(!text.contains(absent), "{absent}");
    }
    // P4 alone has its own values, and P3 alone has only itself.
    let p4 = served(&options(&fx.pool, &[fx.p4]).await);
    assert_eq!(
        ids_of(&p4["series"], "seriesId"),
        [fx.s4, fx.s5, fx.s6].into()
    );
    assert_eq!(p4["languageCodes"], json!(["GER"]));
    let p3 = served(&options(&fx.pool, &[fx.p3]).await);
    assert_eq!(
        p3,
        json!({
            "publishers": [{ "publisherId": fx.p3, "publisherName": "Gamma Empty" }],
            "series": [], "languageCodes": [], "workTypes": [],
            "fundingInstitutions": [], "affiliationInstitutions": [],
        })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t2_only_a_chapter_reaches_its_parents_series() {
    let (_guard, fx) = fixture();
    let data = served(&options(&fx.pool, &[fx.p1]).await);
    let series = ids_of(&data["series"], "seriesId");
    // S4 through the chapter's parent (another publisher's work); not S5,
    // the series of a non-chapter's parent; not S6, which no P1 work reaches.
    assert_eq!(series, [fx.s1, fx.s4].into());
    assert!(!series.contains(&fx.s5));
    assert!(!series.contains(&fx.s6));
    // P2 has no chapter: only its own series.
    let data = served(&options(&fx.pool, &[fx.p2]).await);
    assert_eq!(ids_of(&data["series"], "seriesId"), [fx.s2].into());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t3_values_are_listed_once_and_in_every_list_they_belong_to() {
    let (_guard, fx) = fixture();
    let data = served(&options(&fx.pool, &[fx.p1, fx.p2]).await);
    // F1 funds two works: listed once.
    let funders: Vec<&JsonValue> = data["fundingInstitutions"]
        .as_array()
        .expect("a list")
        .iter()
        .filter(|entry| entry["institutionId"] == json!(fx.f1))
        .collect();
    assert_eq!(funders.len(), 1);
    // FA funds a work and hosts a contributor: in both lists.
    assert!(ids_of(&data["fundingInstitutions"], "institutionId").contains(&fx.fa));
    assert!(ids_of(&data["affiliationInstitutions"], "institutionId").contains(&fx.fa));
    // `eng` is on two works: listed once.
    assert_eq!(data["languageCodes"], json!(["ENG", "FRE"]));
    // Monograph is a work type of one work only here, and listed once.
    let types = data["workTypes"].as_array().expect("a list");
    let distinct: BTreeSet<String> = types.iter().map(JsonValue::to_string).collect();
    assert_eq!(distinct.len(), types.len());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t4_lists_are_in_byte_order_with_id_tie_breaks_whatever_the_input_order() {
    let (_guard, pool) = setup_test_db();
    let mut sql = Sql::default();
    let p1 = sql.publisher("aa");
    let p2 = sql.publisher("Ab");
    let p3 = sql.publisher("a-b");
    let (i1, i2) = (sql.imprint(p1), sql.imprint(p2));
    let chapter = sql.work(i1, "book-chapter");
    let set = sql.work(i2, "book-set");
    let edited = sql.work(i1, "edited-book");
    // Two institutions with the same name, the higher ID inserted first.
    let high = Uuid::parse_str("10000000-0000-4000-8000-000000000000").expect("a UUID");
    let low = Uuid::parse_str("0f000000-0000-4000-8000-000000000000").expect("a UUID");
    sql.institution_with_id(high, "Same", None);
    sql.institution_with_id(low, "Same", None);
    let names = [
        ("aa", sql.institution("aa", None)),
        ("Ab", sql.institution("Ab", None)),
        ("a-b", sql.institution("a-b", None)),
    ];
    for work in [chapter, set, edited] {
        sql.funding(work, high);
        sql.funding(work, low);
        for (_, id) in names {
            sql.affiliation(work, id);
        }
    }
    let s_aa = sql.series(i1, "aa", "journal", (None, None));
    let s_ab = sql.series(i2, "Ab", "journal", (None, None));
    let s_a_b = sql.series(i1, "a-b", "journal", (None, None));
    sql.issue(s_aa, edited);
    sql.issue(s_ab, set);
    sql.issue(s_a_b, edited);
    sql.run(&pool);

    let first = options(&pool, &[p1, p2, p3]).await;
    let data = served(&first);
    let names_of = |list: &JsonValue, key: &str| -> Vec<String> {
        list.as_array()
            .expect("a list")
            .iter()
            .map(|entry| entry[key].as_str().expect("a name").to_string())
            .collect()
    };
    assert_eq!(
        names_of(&data["publishers"], "publisherName"),
        ["Ab", "a-b", "aa"]
    );
    assert_eq!(names_of(&data["series"], "seriesName"), ["Ab", "a-b", "aa"]);
    assert_eq!(
        data["fundingInstitutions"],
        json!([
            institution(low, "Same", None),
            institution(high, "Same", None)
        ])
    );
    assert_eq!(
        names_of(&data["affiliationInstitutions"], "institutionName"),
        ["Ab", "a-b", "aa"]
    );
    // Label byte order, not declaration order (BookChapter, Monograph,
    // EditedBook, ..., BookSet).
    assert_eq!(
        data["workTypes"],
        json!(["BOOK_CHAPTER", "BOOK_SET", "EDITED_BOOK"])
    );
    // The same IDs in other orders give identical JSON.
    for order in [[p3, p2, p1], [p2, p1, p3], [p3, p1, p2]] {
        assert_eq!(options(&pool, &order).await, first, "{order:?}");
    }
}

// --------------------------------------------------------------------------
// T5: input failures through the schema
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t5_each_input_failure_has_its_type_and_message_with_null_data() {
    let (_guard, fx) = fixture();
    let random = |count: usize| -> Vec<Uuid> { (0..count).map(|_| Uuid::new_v4()).collect() };
    let mut over_with_duplicate = random(100);
    over_with_duplicate.push(over_with_duplicate[7]);
    let invalid = "WORK_FILTER_OPTIONS_INVALID";
    for (ids, error) in [
        (
            vec![],
            expected(invalid, "publisherIds must contain at least one publisher."),
        ),
        (
            vec![fx.p1, fx.p2, fx.p1],
            expected(invalid, "publisherIds must not contain duplicate values."),
        ),
        (
            random(101),
            expected(invalid, "publisherIds may contain at most 100 publishers."),
        ),
        (
            over_with_duplicate,
            expected(invalid, "publisherIds must not contain duplicate values."),
        ),
    ] {
        assert_eq!(only_error(&options(&fx.pool, &ids).await), error, "{ids:?}");
    }

    // Unknown IDs: lower-case, sorted, separated by ", ", whatever the input
    // spelling and order.
    let high = "F0000000-0000-4000-8000-0000000000AB";
    let low = "0a000000-0000-4000-8000-000000000001";
    let context = test_context_anonymous(Arc::clone(&fx.pool));
    let response = execute(
        &context,
        OPTIONS,
        json!({ "ids": [high, fx.p1.to_string(), low] }),
    )
    .await;
    assert_eq!(
        only_error(&response),
        expected(
            "WORK_FILTER_OPTIONS_UNKNOWN_PUBLISHER",
            &format!(
                "publisherIds contains unknown publisher(s): {low}, {}.",
                high.to_lowercase()
            )
        )
    );
}

// --------------------------------------------------------------------------
// T6: the list bound, through the builder
// --------------------------------------------------------------------------

#[test]
fn t6_a_list_over_the_bound_fails_and_one_at_it_succeeds() {
    let (_guard, fx) = fixture();
    let slots = AtomicUsize::new(0);
    let with_bound = |max_entries: usize, ids: &[Uuid]| {
        work_filter_options_with(
            &fx.pool,
            ids,
            &WorkFilterReadControls {
                max_entries,
                ..controls(&slots)
            },
        )
    };
    let all = [fx.p1, fx.p2, fx.p3];
    // Sizes for P1 to P3: publishers 3, series 3, languageCodes 2,
    // workTypes 4, fundingInstitutions 2, affiliationInstitutions 2.
    let error = with_bound(3, &all).expect_err("workTypes is one over");
    assert_eq!(
        error,
        WorkFilterOptionsError::LimitExceeded {
            field: "workTypes",
            bound: 3
        }
    );
    assert_eq!(
        type_and_message(error),
        expected(
            "WORK_FILTER_OPTIONS_LIMIT_EXCEEDED",
            "workFilterOptions.workTypes exceeds 3 entries."
        )
    );
    // Exactly at the bound: served in full.
    let served: WorkFilterOptions = with_bound(4, &all).expect("at the bound");
    assert_eq!(served.work_types.len(), 4);
    assert_eq!(served.publishers.len(), 3);

    // Two lists over (series 3 and workTypes 4 for P1 and P2, publishers at
    // 2): the first in field order is named.
    let error = with_bound(2, &[fx.p1, fx.p2]).expect_err("two lists over");
    assert_eq!(
        type_and_message(error),
        expected(
            "WORK_FILTER_OPTIONS_LIMIT_EXCEEDED",
            "workFilterOptions.series exceeds 2 entries."
        )
    );

    // An unknown ID and a list over: the unknown ID wins.
    let unknown = Uuid::new_v4();
    assert_eq!(
        with_bound(2, &[fx.p1, fx.p2, unknown]),
        Err(WorkFilterOptionsError::UnknownPublisher(vec![unknown]))
    );
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

// --------------------------------------------------------------------------
// T7: parity with WORKS_SQL
// --------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct ResolvedRow {
    #[diesel(sql_type = Nullable<SqlUuid>)]
    work_id: Option<Uuid>,
}

#[derive(diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct LabelRow {
    #[diesel(sql_type = Text)]
    label: String,
}

/// One selector dimension of `WORKS_SQL`, set to a single value.
enum Dimension {
    Series(Uuid),
    WorkType(String),
    Language(String),
    Funding(Uuid),
    Affiliation(Uuid),
}

/// Whether `WORKS_SQL` with `$1 = publishers`, only `dimension` set and every
/// other array empty, and a positive limit, resolves at least one work.
fn works_sql_resolves(pool: &PgPool, publishers: &[Uuid], dimension: &Dimension) -> bool {
    let none: Vec<Uuid> = Vec::new();
    let no_labels: Vec<String> = Vec::new();
    let one_id = |id: &Uuid| vec![*id];
    let (series, work_types, languages, funding, affiliation) = match dimension {
        Dimension::Series(id) => (
            one_id(id),
            no_labels.clone(),
            no_labels.clone(),
            none.clone(),
            none.clone(),
        ),
        Dimension::WorkType(label) => (
            none.clone(),
            vec![label.clone()],
            no_labels.clone(),
            none.clone(),
            none.clone(),
        ),
        Dimension::Language(label) => (
            none.clone(),
            no_labels.clone(),
            vec![label.clone()],
            none.clone(),
            none.clone(),
        ),
        Dimension::Funding(id) => (
            none.clone(),
            no_labels.clone(),
            no_labels.clone(),
            one_id(id),
            none.clone(),
        ),
        Dimension::Affiliation(id) => (
            none.clone(),
            no_labels.clone(),
            no_labels.clone(),
            none.clone(),
            one_id(id),
        ),
    };
    let rows: Vec<ResolvedRow> = diesel::sql_query(WORKS_SQL)
        .bind::<Array<SqlUuid>, _>(publishers)
        .bind::<Array<SqlUuid>, _>(&none)
        .bind::<Array<SqlUuid>, _>(&series)
        .bind::<Array<SqlUuid>, _>(&none)
        .bind::<Array<Text>, _>(&work_types)
        .bind::<Array<Text>, _>(&languages)
        .bind::<Array<SqlUuid>, _>(&funding)
        .bind::<Array<SqlUuid>, _>(&affiliation)
        .bind::<SqlBigInt, _>(1_i64)
        .load(&mut pool.get().expect("a connection"))
        .expect("WORKS_SQL");
    rows.iter().any(|row| row.work_id.is_some())
}

fn ids_from(pool: &PgPool, sql: &str) -> Vec<Uuid> {
    let rows: Vec<IdRow> = diesel::sql_query(sql)
        .load(&mut pool.get().expect("a connection"))
        .expect("candidate IDs");
    rows.into_iter().map(|row| row.id).collect()
}

fn labels_from(pool: &PgPool, sql: &str) -> Vec<String> {
    let rows: Vec<LabelRow> = diesel::sql_query(sql)
        .load(&mut pool.get().expect("a connection"))
        .expect("candidate labels");
    rows.into_iter().map(|row| row.label).collect()
}

/// The GraphQL name of a database enum label (`book-chapter` ->
/// `BOOK_CHAPTER`, `eng` -> `ENG`).
fn graphql_name(label: &str) -> String {
    label.to_uppercase().replace('-', "_")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t7_every_candidate_is_listed_if_and_only_if_works_sql_resolves_it() {
    let (_guard, fx) = fixture();
    let selected = [fx.p1, fx.p2, fx.p3];
    let data = served(&options(&fx.pool, &selected).await);
    let pool = &fx.pool;
    let mut checked = 0;

    let mut check = |listed: bool, dimension: Dimension, what: String| {
        let resolves = works_sql_resolves(pool, &selected, &dimension);
        assert_eq!(
            listed, resolves,
            "{what}: listed={listed} resolves={resolves}"
        );
        checked += 1;
        listed
    };

    let series = ids_of(&data["series"], "seriesId");
    let mut outcomes = BTreeSet::new();
    for id in ids_from(pool, "SELECT series_id AS id FROM series") {
        outcomes.insert(check(
            series.contains(&id),
            Dimension::Series(id),
            format!("series {id}"),
        ));
    }
    assert_eq!(outcomes, [false, true].into(), "series has both outcomes");

    let funding = ids_of(&data["fundingInstitutions"], "institutionId");
    let affiliation = ids_of(&data["affiliationInstitutions"], "institutionId");
    let (mut funding_outcomes, mut affiliation_outcomes) = (BTreeSet::new(), BTreeSet::new());
    for id in ids_from(pool, "SELECT institution_id AS id FROM institution") {
        funding_outcomes.insert(check(
            funding.contains(&id),
            Dimension::Funding(id),
            format!("funding {id}"),
        ));
        affiliation_outcomes.insert(check(
            affiliation.contains(&id),
            Dimension::Affiliation(id),
            format!("affiliation {id}"),
        ));
    }
    assert_eq!(funding_outcomes, [false, true].into());
    assert_eq!(affiliation_outcomes, [false, true].into());

    let work_types: BTreeSet<String> = data["workTypes"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|value| value.as_str().expect("a name").to_string())
        .collect();
    let mut type_outcomes = BTreeSet::new();
    for label in labels_from(
        pool,
        "SELECT value::text AS label FROM unnest(enum_range(NULL::work_type)) AS value",
    ) {
        type_outcomes.insert(check(
            work_types.contains(&graphql_name(&label)),
            Dimension::WorkType(label.clone()),
            format!("work type {label}"),
        ));
    }
    assert_eq!(type_outcomes, [false, true].into());

    let languages: BTreeSet<String> = data["languageCodes"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|value| value.as_str().expect("a name").to_string())
        .collect();
    let mut codes = labels_from(
        pool,
        "SELECT DISTINCT language_code::text AS label FROM language",
    );
    assert!(!codes.contains(&"zul".to_string()));
    // One code on no language row.
    codes.push("zul".to_string());
    let mut language_outcomes = BTreeSet::new();
    for label in codes {
        language_outcomes.insert(check(
            languages.contains(&graphql_name(&label)),
            Dimension::Language(label.clone()),
            format!("language {label}"),
        ));
    }
    assert_eq!(language_outcomes, [false, true].into());
    // 5 series, 5 institutions twice, 6 work types, 4 language codes.
    assert_eq!(checked, 5 + 10 + 6 + 4);
}

// --------------------------------------------------------------------------
// T8: no authorization
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t8_anonymous_and_superuser_callers_get_identical_json() {
    let (_guard, fx) = fixture();
    for ids in [
        vec![fx.p1, fx.p2, fx.p3],
        vec![fx.p4],
        vec![],
        vec![Uuid::new_v4()],
    ] {
        let anonymous = options(&fx.pool, &ids).await;
        let superuser = execute(
            &test_context_with_user(Arc::clone(&fx.pool), test_superuser("superuser")),
            OPTIONS,
            json!({ "ids": ids }),
        )
        .await;
        assert_eq!(anonymous, superuser, "{ids:?}");
    }
}

#[test]
fn t8_the_resolver_reads_no_identity_and_no_role_guard() {
    let source = include_str!("query.rs");
    let start = source
        .find("fn work_filter_options(")
        .expect("the resolver");
    let body = &source[start..];
    let body = &body[..body.find("\n    }\n").expect("the resolver body")];
    for absent in ["context.user", "require_", "policy", "MetricReadError"] {
        assert!(!body.contains(absent), "{absent}: {body}");
    }
    assert!(body.contains("context.work_filter_options_claims"));
}

// --------------------------------------------------------------------------
// T9: the transaction settings and the statement timeout
// --------------------------------------------------------------------------

/// A statement that fails with `division by zero` unless `condition` holds.
fn require(condition: &str) -> String {
    format!("SELECT 1 / ({condition})::int")
}

#[test]
fn t9_the_transaction_is_read_only_read_committed_with_both_settings() {
    let (_guard, fx) = fixture();
    let slots = AtomicUsize::new(0);
    let run = |statement: &str| {
        work_filter_options_with(
            &fx.pool,
            &[fx.p1],
            &WorkFilterReadControls {
                test_statement: Some(statement),
                ..controls(&slots)
            },
        )
    };
    for condition in [
        "current_setting('plan_cache_mode') = 'force_custom_plan'",
        "current_setting('statement_timeout') = '10s'",
        "current_setting('transaction_read_only') = 'on'",
        "current_setting('transaction_isolation') = 'read committed'",
    ] {
        run(&require(condition)).unwrap_or_else(|error| panic!("{condition}: {error:?}"));
    }
    // The check discriminates: a false condition fails the read.
    for condition in [
        "current_setting('plan_cache_mode') = 'auto'",
        "current_setting('statement_timeout') = '0'",
        "current_setting('transaction_read_only') = 'off'",
    ] {
        assert_eq!(
            run(&require(condition)),
            Err(WorkFilterOptionsError::Unavailable),
            "{condition}"
        );
    }
    // Outside the read, the session keeps its own settings.
    let mut connection = fx.pool.get().expect("a connection");
    let rows: Vec<LabelRow> =
        diesel::sql_query("SELECT current_setting('statement_timeout') AS label")
            .load(&mut connection)
            .expect("the session timeout");
    assert_ne!(rows[0].label, "10s");
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

#[test]
fn t9_a_statement_timeout_is_timeout() {
    let (_guard, fx) = fixture();
    let slots = AtomicUsize::new(0);
    let error = work_filter_options_with(
        &fx.pool,
        &[fx.p1],
        &WorkFilterReadControls {
            statement_timeout_sql: "SET LOCAL statement_timeout = '100ms'",
            test_statement: Some("SELECT pg_sleep(2)"),
            ..controls(&slots)
        },
    )
    .expect_err("timed out");
    assert_eq!(error, WorkFilterOptionsError::Timeout);
    assert_eq!(
        type_and_message(error),
        expected(
            "WORK_FILTER_OPTIONS_TIMEOUT",
            "workFilterOptions timed out; try again shortly."
        )
    );
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

// --------------------------------------------------------------------------
// T10: the SDL contract
// --------------------------------------------------------------------------

/// A block's text without whitespace or descriptions.
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
fn t10_the_sdl_declares_exactly_the_approved_contract() {
    let sdl = create_schema().as_sdl();
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    assert!(
        query.contains("workFilterOptions(publisherIds:[Uuid!]!):WorkFilterOptions!"),
        "{query}"
    );
    for (declaration, expected) in [
        (
            "type WorkFilterOptions {",
            "publishers:[WorkFilterPublisher!]!series:[WorkFilterSeries!]!\
             languageCodes:[LanguageCode!]!workTypes:[WorkType!]!\
             fundingInstitutions:[WorkFilterInstitution!]!\
             affiliationInstitutions:[WorkFilterInstitution!]!",
        ),
        (
            "type WorkFilterPublisher {",
            "publisherId:Uuid!publisherName:String!",
        ),
        (
            "type WorkFilterSeries {",
            "seriesId:Uuid!seriesName:String!seriesType:SeriesType!issnPrint:String\
             issnDigital:String",
        ),
        (
            "type WorkFilterInstitution {",
            "institutionId:Uuid!institutionName:String!ror:Ror",
        ),
    ] {
        let block = sdl_block(&sdl, declaration);
        assert_eq!(signatures(block), expected, "{declaration}");
        // Every field carries a description.
        let fields = block
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('"'))
            .count();
        let descriptions = block
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with('"'))
            .count();
        assert_eq!(fields, descriptions, "{declaration}");
        // And the type itself.
        let before = &sdl[..sdl.find(declaration).expect("the declaration")];
        assert!(
            before.trim_end().ends_with('"'),
            "{declaration} has a description"
        );
    }
    // The resolver's description states the contract.
    let query_root = sdl_block(&sdl, "type QueryRoot {");
    let lines: Vec<&str> = query_root.lines().collect();
    let field = lines
        .iter()
        .position(|line| line.trim_start().starts_with("workFilterOptions("))
        .expect("the field");
    let description = lines[field - 1].trim();
    assert!(
        description.starts_with('"') && description.ends_with('"'),
        "a description: {description}"
    );
    for phrase in [
        "complete",
        "fails",
        "100 publishers",
        "only one workFilterOptions field",
    ] {
        assert!(description.contains(phrase), "{phrase}: {description}");
    }
}

// --------------------------------------------------------------------------
// T11: busy (the only test holding the process slots across calls)
// --------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t11_with_both_process_slots_held_a_read_is_busy_and_reads_nothing() {
    let (_guard, fx) = fixture();
    let (pool, log) = capturing_pool();
    let pool = Arc::new(pool);
    let first = try_take_slot(&WORK_FILTER_OPTIONS_SLOTS).expect("a first slot");
    let second = try_take_slot(&WORK_FILTER_OPTIONS_SLOTS).expect("a second slot");
    assert!(try_take_slot(&WORK_FILTER_OPTIONS_SLOTS).is_none());

    let response = options(&pool, &[fx.p1]).await;
    assert_eq!(
        only_error(&response),
        expected(
            "WORK_FILTER_OPTIONS_BUSY",
            "workFilterOptions is busy; try again shortly."
        )
    );
    assert!(
        log.lock().expect("the statement log").is_empty(),
        "no checkout and no statement"
    );
    assert_eq!(WORK_FILTER_OPTIONS_SLOTS.load(Ordering::SeqCst), 2);

    drop((first, second));
    assert_eq!(WORK_FILTER_OPTIONS_SLOTS.load(Ordering::SeqCst), 0);
    let data = served(&options(&pool, &[fx.p1]).await);
    assert_eq!(data["publishers"][0]["publisherId"], json!(fx.p1));
    assert_eq!(WORK_FILTER_OPTIONS_SLOTS.load(Ordering::SeqCst), 0);

    // A failed read releases its slot.
    let response = options(&pool, &[Uuid::new_v4()]).await;
    assert_eq!(
        only_error(&response).0,
        "WORK_FILTER_OPTIONS_UNKNOWN_PUBLISHER"
    );
    assert_eq!(WORK_FILTER_OPTIONS_SLOTS.load(Ordering::SeqCst), 0);
}

// --------------------------------------------------------------------------
// T12: one field per request
// --------------------------------------------------------------------------

fn selection(ids: &[String]) -> String {
    let ids: Vec<String> = ids.iter().map(|id| format!("\"{id}\"")).collect();
    format!(
        "workFilterOptions(publisherIds: [{}]) {{ publishers {{ publisherId }} }}",
        ids.join(", ")
    )
}

/// Execute one document with a fresh context on a capturing pool: the
/// response, the transactions it opened and the context's claim count.
async fn execute_captured(query: &str) -> (JsonValue, usize, usize) {
    let (pool, log) = capturing_pool();
    let context = test_context_anonymous(Arc::new(pool));
    let response = execute(&context, query, json!({})).await;
    let transactions = log
        .lock()
        .expect("the statement log")
        .iter()
        .filter(|statement| statement.starts_with("BEGIN"))
        .count();
    (
        response,
        transactions,
        context.work_filter_options_claims.load(Ordering::SeqCst),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t12_a_second_field_is_refused_and_only_one_read_runs() {
    let (_guard, fx) = fixture();
    let known = selection(&[fx.p1.to_string()]);
    let empty = selection(&[]);
    let unknown = selection(&[Uuid::new_v4().to_string()]);
    let one_per_request = expected(
        "WORK_FILTER_OPTIONS_ONE_PER_REQUEST",
        "Only one workFilterOptions field may be requested at a time.",
    );
    for query in [
        format!("{{ a: {known} b: {known} }}"),
        format!("{{ {known} {known} }}"),
        format!("{{ a: {known} b: {empty} }}"),
        format!("{{ a: {known} b: {unknown} }}"),
    ] {
        let (response, transactions, claims) = execute_captured(&query).await;
        assert_eq!(only_error(&response), one_per_request, "{query}");
        assert_eq!(transactions, 1, "{query}");
        assert_eq!(claims, 2, "{query}");
    }

    // Repeated fragments: the first read's data is kept, the second is
    // refused.
    for query in [
        format!("{{ ...F ...F }} fragment F on QueryRoot {{ {known} }}"),
        format!("{{ ... on QueryRoot {{ {known} }} ... on QueryRoot {{ {known} }} }}"),
    ] {
        let (response, transactions, claims) = execute_captured(&query).await;
        assert_eq!(
            error_types(&response),
            ["WORK_FILTER_OPTIONS_ONE_PER_REQUEST"],
            "{query}: {response}"
        );
        assert_eq!(
            response["errors"][0]["message"],
            json!(one_per_request.1),
            "{query}"
        );
        assert_eq!(
            response["data"]["workFilterOptions"]["publishers"],
            json!([{ "publisherId": fx.p1 }]),
            "{query}: {response}"
        );
        assert_eq!(transactions, 1, "{query}");
        assert_eq!(claims, 2, "{query}");
    }

    // Two separate requests each succeed.
    for _ in 0..2 {
        let (response, transactions, claims) = execute_captured(&format!("{{ {known} }}")).await;
        assert_eq!(
            served(&response)["publishers"],
            json!([{ "publisherId": fx.p1 }])
        );
        assert_eq!((transactions, claims), (1, 1));
    }
}

// --------------------------------------------------------------------------
// T13 and T14: unavailable and checkout failures, through the builder
// --------------------------------------------------------------------------

#[test]
fn t13_a_database_failure_is_unavailable_without_its_text() {
    let (_guard, fx) = fixture();
    let slots = AtomicUsize::new(0);
    let error = work_filter_options_with(
        &fx.pool,
        &[fx.p1],
        &WorkFilterReadControls {
            test_statement: Some("SELECT 1/0"),
            ..controls(&slots)
        },
    )
    .expect_err("unavailable");
    assert_eq!(error, WorkFilterOptionsError::Unavailable);
    let debug = format!("{error:?}");
    let field_error = error.into_field_error();
    let rendered = format!("{debug} {field_error:?} {}", field_error.message());
    assert!(!rendered.contains("division"), "{rendered}");
    assert_eq!(
        field_error.message(),
        "workFilterOptions could not be completed; try again shortly."
    );
    assert_eq!(
        extension_type(&field_error),
        "WORK_FILTER_OPTIONS_UNAVAILABLE"
    );
    assert_eq!(slots.load(Ordering::SeqCst), 0);
    // The connection went back to the pool in a usable state.
    work_filter_options_with(&fx.pool, &[fx.p1], &controls(&slots)).expect("served");
}

#[test]
fn t14_a_checkout_failure_is_busy() {
    let pool = failing_pool();
    let slots = AtomicUsize::new(0);
    let error = work_filter_options_with(
        &pool,
        &[Uuid::new_v4()],
        &WorkFilterReadControls {
            checkout_timeout: Duration::from_millis(50),
            ..controls(&slots)
        },
    )
    .expect_err("busy");
    assert_eq!(error, WorkFilterOptionsError::Busy);
    assert_eq!(
        type_and_message(error),
        expected(
            "WORK_FILTER_OPTIONS_BUSY",
            "workFilterOptions is busy; try again shortly."
        )
    );
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}
