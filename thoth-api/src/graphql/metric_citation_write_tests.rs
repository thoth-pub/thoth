//! `CITATIONS-01` slice C2 at the API boundary: the exact additive SDL, the
//! `METRICS_INGEST_SERVICE`-only role matrix for every new operation, the
//! authorization-before-database order, and one lifecycle through GraphQL.

#![cfg(all(test, feature = "backend"))]

use std::sync::Arc;

use serde_json::Value as JsonValue;
use uuid::Uuid;

use super::create_schema;
use super::metric_ingestion_lifecycle_tests::{
    assert_unauthorized, context_for, data, denied, ingest_user, run, signatures,
};
use super::sdl_support::sdl_block;
use crate::model::metric_citation::write_tests::{claim, setup, ACCOUNT};
use crate::model::tests::db as test_db;

fn begin_doc(token: &str) -> String {
    format!(
        "mutation {{ beginCitationRefresh(input: {{ sourceAccountCode: \"{ACCOUNT}\", leaseToken: \"{token}\" }}) \
           {{ refreshId status worksSeen worksMatched worksNotFound linksUpserted linksWithdrawn linksRefused linksMerged }} }}"
    )
}

fn works_doc(token: &str, refresh: &str) -> String {
    format!(
        "{{ citationRefreshWorks(input: {{ sourceAccountCode: \"{ACCOUNT}\", leaseToken: \"{token}\", refreshId: \"{refresh}\", limit: 2 }}) \
           {{ done nextAfter works {{ workId doi }} }} }}"
    )
}

fn submit_doc(token: &str, refresh: &str, work: &str) -> String {
    format!(
        "mutation {{ submitCitationBatch(input: {{ sourceAccountCode: \"{ACCOUNT}\", leaseToken: \"{token}\", refreshId: \"{refresh}\", batchKey: \"b1\", \
           works: [{{ workId: \"{work}\", matchStatus: MATCHED, externalWorkId: \"W1\", reportedCount: 1, enumerationComplete: true, enumeratedTotal: 1, \
             links: [{{ citingExternalId: \"C1\", citingDoi: \"10.1/c\", citingTitle: \"Citing\", citingAuthorNames: [\"A. B.\"], citingAuthorsTotal: 1 }}] }}] }}) \
           {{ refreshId batchKey replayed worksAccepted linksUpserted linksRefused linksMerged }} }}"
    )
}

fn complete_doc(token: &str, refresh: &str) -> String {
    format!(
        "mutation {{ completeCitationRefresh(input: {{ sourceAccountCode: \"{ACCOUNT}\", leaseToken: \"{token}\", refreshId: \"{refresh}\" }}) \
           {{ refreshId status linksUpserted linksWithdrawn }} }}"
    )
}

fn every_operation(token: &str, refresh: &str, work: &str) -> Vec<String> {
    vec![
        begin_doc(token),
        works_doc(token, refresh),
        submit_doc(token, refresh, work),
        complete_doc(token, refresh),
    ]
}

#[test]
fn the_sdl_adds_exactly_the_four_citation_operations() {
    let sdl = create_schema().as_sdl();
    let mutation = signatures(sdl_block(&sdl, "type MutationRoot {"));
    for operation in [
        "beginCitationRefresh(input:BeginCitationRefreshInput!):CitationRefresh!",
        "submitCitationBatch(input:SubmitCitationBatchInput!):CitationBatchResult!",
        "completeCitationRefresh(input:CompleteCitationRefreshInput!):CitationRefresh!",
    ] {
        assert_eq!(mutation.matches(operation).count(), 1, "{operation}");
    }
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    assert_eq!(
        query
            .matches(
                "citationRefreshWorks(input:CitationRefreshWorksInput!):CitationRefreshWorksPage!"
            )
            .count(),
        1
    );
    // No read of citation links exists yet (C3), and no second claim operation.
    for absent in [
        "metricCitations",
        "claimCitation",
        "withdrawCitation",
        "deleteCitation",
    ] {
        assert!(!sdl.contains(absent), "{absent}");
    }
    for value in ["RUNNING", "COMPLETED", "COMPLETED_WITH_ERRORS", "FAILED"] {
        assert!(sdl_block(&sdl, "enum MetricCitationRefreshStatus {").contains(value));
    }
    for value in ["MATCHED", "NOT_FOUND", "FAILED"] {
        assert!(sdl_block(&sdl, "enum CitationLookupStatus {").contains(value));
    }
}

#[tokio::test]
async fn every_citation_operation_denies_every_principal_but_the_ingest_service_before_any_database_access(
) {
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    let token = Uuid::new_v4().to_string();
    let refresh = Uuid::new_v4().to_string();
    let work = Uuid::new_v4().to_string();
    for (_label, user) in denied("org") {
        let context = context_for(&unreachable, user);
        for document in every_operation(&token, &refresh, &work) {
            assert_unauthorized(&run(&schema, &context, &document).await);
        }
    }
    // The authorized caller reaches the (unreachable) database instead.
    let context = context_for(&unreachable, Some(ingest_user()));
    for document in every_operation(&token, &refresh, &work) {
        let response = run(&schema, &context, &document).await;
        assert!(
            response.to_string().contains("INTERNAL_DATABASE_ERROR"),
            "{response}"
        );
    }
}

#[tokio::test]
async fn a_refresh_runs_through_graphql_with_bounded_errors() {
    let (_guard, f) = setup();
    let schema = create_schema();
    let context = context_for(&f.pool, Some(ingest_user()));
    let claim = claim(&f);
    let token = claim.lease_token.to_string();

    let began = run(&schema, &context, &begin_doc(&token)).await;
    let refresh = data(&began, "beginCitationRefresh");
    assert_eq!(refresh["status"], "RUNNING");
    let refresh_id = refresh["refreshId"].as_str().unwrap().to_string();

    let page = run(&schema, &context, &works_doc(&token, &refresh_id)).await;
    let page = data(&page, "citationRefreshWorks");
    assert_eq!(page["works"].as_array().unwrap().len(), 2);
    assert_eq!(page["done"], false);
    let work = page["works"][0]["workId"].as_str().unwrap().to_string();

    let batch = run(&schema, &context, &submit_doc(&token, &refresh_id, &work)).await;
    let batch = data(&batch, "submitCitationBatch");
    assert_eq!(
        (batch["linksUpserted"].as_i64(), batch["replayed"].as_bool()),
        (Some(1), Some(false))
    );
    let replay = run(&schema, &context, &submit_doc(&token, &refresh_id, &work)).await;
    assert_eq!(data(&replay, "submitCitationBatch")["replayed"], true);

    // A foreign work is refused with a bounded, classified error.
    let foreign = f.foreign_work.to_string();
    let foreign_doc =
        submit_doc(&token, &refresh_id, &foreign).replace("batchKey: \"b1\"", "batchKey: \"b2\"");
    let refused = run(&schema, &context, &foreign_doc).await;
    let errors = refused["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{refused}");
    assert_eq!(
        errors[0]["extensions"]["type"],
        "CITATION_WORK_NOT_IN_PUBLISHER"
    );
    assert!(errors[0]["message"].as_str().unwrap().len() <= 120);

    let done = run(&schema, &context, &complete_doc(&token, &refresh_id)).await;
    let done = data(&done, "completeCitationRefresh");
    assert_eq!(done["status"], "COMPLETED");
    assert_eq!(done["linksUpserted"], 1);

    // The released claim is stale for every further call, with a bounded error.
    let stale = run(&schema, &context, &complete_doc(&token, &refresh_id)).await;
    assert_eq!(
        stale["errors"][0]["extensions"]["type"],
        "STALE_SOURCE_CLAIM"
    );
    let _: &JsonValue = &stale;
}
