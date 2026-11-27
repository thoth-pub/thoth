//! `CITATIONS-01` slice C3 at the API boundary: the exact additive SDL, the
//! `METRICS_READ_SERVICE`-only role matrix, authorization before the switch
//! and before any database access, the switch answer for authorized callers,
//! and one read through GraphQL.

#![cfg(all(test, feature = "backend"))]

use std::sync::Arc;

use serde_json::Value as JsonValue;
use uuid::Uuid;

use super::create_schema;
use super::metric_ingestion_lifecycle_tests::{
    assert_unauthorized, context_for, data, denied, ingest_user, run, signatures, user_with,
};
use super::sdl_support::sdl_block;
use crate::model::metric_citation::write_tests::{exec, setup};
use crate::model::metric_citation::MetricCitationsServing;
use crate::model::tests::db as test_db;
use crate::policy::Role;

fn reader() -> zitadel::actix::introspection::IntrospectedUser {
    user_with("reader", &[(Role::MetricsReadService, "org")])
}

fn document(publisher: Uuid) -> String {
    format!(
        "{{ metricCitations(input: {{ publisherIds: [\"{publisher}\"], first: 2 }}) {{ \
             summary {{ linkCount citedWorkCount matchedWorkCount unconfirmedWorkCount \
                        notFoundWorkCount notYetRefreshedWorkCount reportedCountTotal \
                        lastRefreshedAt byCitingYear {{ year count }} }} \
             links {{ linkId citingExternalId citingTitle citingSortDate source {{ code }} \
                      citedWork {{ workId doi workType }} firstSeenAt }} \
             pageInfo {{ hasNextPage endCursor }} }} }}"
    )
}

fn error_type(response: &JsonValue) -> &str {
    response["errors"][0]["extensions"]["type"]
        .as_str()
        .unwrap_or_else(|| panic!("expected an error: {response}"))
}

#[test]
fn the_sdl_adds_exactly_one_read_operation_and_no_mutation() {
    let sdl = create_schema().as_sdl();
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    assert_eq!(
        query
            .matches("metricCitations(input:MetricCitationsInput!):MetricCitations!")
            .count(),
        1
    );
    let mutation = signatures(sdl_block(&sdl, "type MutationRoot {"));
    assert!(!mutation.contains("metricCitations"));
    // The row-level write shape is unchanged and no other read is added.
    for absent in [
        "claimCitation",
        "withdrawCitation",
        "deleteCitation",
        "citationLinks(",
    ] {
        assert!(!sdl.contains(absent), "{absent}");
    }
    let input = signatures(sdl_block(&sdl, "input MetricCitationsInput {"));
    for field in [
        "publisherIds:[Uuid!]!",
        "workIds:[Uuid!]",
        "citingFrom:Date",
        "citingTo:Date",
        "sourceCodes:[String!]",
        "first:Int",
        "after:String",
    ] {
        assert!(input.contains(field), "{field} in {input}");
    }
}

#[tokio::test]
async fn only_the_read_service_is_served_and_denial_precedes_the_switch_and_the_database() {
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    let publisher = Uuid::new_v4();
    // Every principal but METRICS_READ_SERVICE is denied, whatever the switch.
    let principals = || {
        let mut principals: Vec<_> = denied("org")
            .into_iter()
            // Both of these principals hold METRICS_READ_SERVICE, so they are
            // served, not denied.
            .filter(|(label, _)| {
                *label != "METRICS_READ_SERVICE only" && *label != "every non-ingest role together"
            })
            .collect();
        principals.push(("METRICS_INGEST_SERVICE only", Some(ingest_user())));
        principals
    };
    for serving in [MetricCitationsServing::Off, MetricCitationsServing::On] {
        for (_label, user) in principals() {
            let context = context_for(&unreachable, user).with_metric_citations_serving(serving);
            let response = run(&schema, &context, &document(publisher)).await;
            assert_unauthorized(&response);
        }
    }
    // The authorized caller learns the switch, before any database access.
    let off = context_for(&unreachable, Some(reader()))
        .with_metric_citations_serving(MetricCitationsServing::Off);
    assert_eq!(
        error_type(&run(&schema, &off, &document(publisher)).await),
        "METRIC_CITATIONS_NOT_SERVED"
    );
    // With the switch on, the same caller reaches the (unreachable) database.
    let on = context_for(&unreachable, Some(reader()))
        .with_metric_citations_serving(MetricCitationsServing::On);
    assert_eq!(
        error_type(&run(&schema, &on, &document(publisher)).await),
        "INTERNAL_ERROR"
    );
}

#[tokio::test]
async fn a_read_runs_through_graphql_and_a_bad_request_gets_a_bounded_classified_error() {
    let (_guard, f) = setup();
    exec(
        &f.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'SPHINX' WHERE publisher_id = '{}'",
            f.publisher_id
        ),
    );
    exec(
        &f.pool,
        &format!(
            "INSERT INTO metric_citation_work_state (cited_work_id, source_id, match_status, \
               last_matched_at, external_work_id, reported_count, last_attempted_at, last_refreshed_at) \
             VALUES ('{}', '{}', 'MATCHED', now(), 'W1', 3, now(), now())",
            f.works[0], f.source_id
        ),
    );
    for (n, id) in ["A", "B", "C"].iter().enumerate() {
        exec(
            &f.pool,
            &format!(
                "INSERT INTO metric_citation_link (cited_work_id, source_id, citing_external_id, \
                   citing_title, citing_publication_year, citing_sort_date) \
                 VALUES ('{}', '{}', '{id}', 'Citing {id}', {}, '{}-01-01')",
                f.works[0],
                f.source_id,
                2020 + n,
                2020 + n
            ),
        );
    }
    let schema = create_schema();
    let context = context_for(&f.pool, Some(reader()))
        .with_metric_citations_serving(MetricCitationsServing::On);

    let page = run(&schema, &context, &document(f.publisher_id)).await;
    let page = data(&page, "metricCitations");
    assert_eq!(page["summary"]["linkCount"], 3);
    assert_eq!(page["summary"]["matchedWorkCount"], 1);
    assert_eq!(page["summary"]["reportedCountTotal"], 3);
    assert_eq!(page["links"].as_array().unwrap().len(), 2);
    assert_eq!(page["links"][0]["citingExternalId"], "C");
    assert_eq!(page["links"][0]["source"]["code"], "openalex-citations");
    assert_eq!(page["links"][0]["citedWork"]["workType"], "MONOGRAPH");
    assert_eq!(page["pageInfo"]["hasNextPage"], true);
    let cursor = page["pageInfo"]["endCursor"].as_str().unwrap().to_string();

    // The next page continues from the cursor.
    let next = format!(
        "{{ metricCitations(input: {{ publisherIds: [\"{}\"], first: 2, after: \"{cursor}\" }}) \
           {{ links {{ citingExternalId }} pageInfo {{ hasNextPage endCursor }} }} }}",
        f.publisher_id
    );
    let next = run(&schema, &context, &next).await;
    let next = data(&next, "metricCitations");
    assert_eq!(next["links"].as_array().unwrap().len(), 1);
    assert_eq!(next["links"][0]["citingExternalId"], "A");
    assert_eq!(next["pageInfo"]["hasNextPage"], false);

    // A foreign publisher's work is refused with a bounded, classified error.
    let foreign = format!(
        "{{ metricCitations(input: {{ publisherIds: [\"{}\"], workIds: [\"{}\"] }}) \
           {{ links {{ linkId }} }} }}",
        f.publisher_id, f.foreign_work
    );
    let refused = run(&schema, &context, &foreign).await;
    let errors = refused["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{refused}");
    assert_eq!(errors[0]["extensions"]["type"], "METRIC_QUERY_INVALID");
    assert!(errors[0]["message"].as_str().unwrap().len() <= 120);

    // An unentitled publisher is refused as NO_ACCESS.
    exec(
        &f.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'OBELISK' WHERE publisher_id = '{}'",
            f.publisher_id
        ),
    );
    assert_unauthorized(&run(&schema, &context, &document(f.publisher_id)).await);
}
