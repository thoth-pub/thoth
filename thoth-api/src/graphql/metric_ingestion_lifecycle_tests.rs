//! `MET-WP2-02` protected managed-DRIVER ingestion lifecycle evidence at the
//! API boundary.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the exact five-operation SDL and object
//! shapes, the three exposed domain vocabularies, the fail-closed
//! `METRICS_INGEST_SERVICE`-only authorization for every operation and every
//! principal of the approved matrix, authorization before any operation-specific
//! database access, a complete lifecycle through GraphQL, and the bounded,
//! sanitized error extensions.
//!
//! `MET-WP2-03` adds the exact claim-time `platformCode` and period-manifest
//! cursor object shapes, their reachability only through the protected claim
//! result, the typed claim snapshot through GraphQL, and the refusal of any
//! caller-supplied cursor on the checkpoint update.
//!
//! Lease, concurrency, idempotency, progress and crash/retry evidence lives
//! with the durable state itself, in
//! `crate::model::metric_ingestion_lifecycle::tests`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_import::MetricImportStatus;
use crate::model::metric_ingestion::MetricIngestionErrorCode;
use crate::model::metric_ingestion_lifecycle::tests::{
    begin_input, day, setup, Fixture, ACCOUNT_A, DIGEST, SOURCE_CODE,
};
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification;
use crate::model::tests::db as test_db;
use crate::policy::Role;

const INGEST_USER: &str = "metrics-ingest-service-1";
const WORK_DOI: &str = "https://doi.org/10.12345/thoth-wp2-02";

/// The exact approved operation names.
const OPERATIONS: [&str; 5] = [
    "claimMetricSourceUnits",
    "updateMetricSourceCheckpoint",
    "beginMetricImport",
    "ingestMetricBatch",
    "completeMetricImport",
];

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

/// Assert one bounded error of the expected type that leaks nothing internal.
fn assert_error(response: &JsonValue, expected_type: &str) {
    let (message, kind) = only_error(response);
    assert_eq!(kind, expected_type, "{response}");
    assert!(message.len() <= 120, "unbounded message: {message}");
    for leaked in [
        "metric_",
        "SELECT",
        "UPDATE",
        "INSERT",
        "constraint",
        "violates",
        "postgres",
        "lease_owner",
        "password",
        "dist-a",
        "logs",
    ] {
        assert!(
            !message.contains(leaked),
            "`{leaked}` leaked into `{message}`"
        );
    }
}

fn assert_unauthorized(response: &JsonValue) {
    let (message, kind) = only_error(response);
    assert_eq!(
        kind, "NO_ACCESS",
        "expected a fail-closed denial: {response}"
    );
    assert_eq!(message, "Unauthorized");
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

fn context_for(pool: &Arc<PgPool>, user: Option<IntrospectedUser>) -> Context {
    match user {
        Some(user) => test_db::test_context_with_user(Arc::clone(pool), user),
        None => test_db::test_context_anonymous(Arc::clone(pool)),
    }
}

fn ingest_user() -> IntrospectedUser {
    user_with(INGEST_USER, &[(Role::MetricsIngestService, "org")])
}

/// Every caller of the approved negative matrix that must be denied.
fn denied(org: &str) -> Vec<(&'static str, Option<IntrospectedUser>)> {
    vec![
        ("anonymous / failed introspection", None),
        (
            "authenticated, no applicable role",
            Some(user_with("no-roles", &[])),
        ),
        (
            "PUBLISHER_USER",
            Some(user_with("owner", &[(Role::PublisherUser, org)])),
        ),
        (
            "PUBLISHER_ADMIN",
            Some(user_with("admin", &[(Role::PublisherAdmin, org)])),
        ),
        (
            "WORK_LIFECYCLE",
            Some(user_with("lifecycle", &[(Role::WorkLifecycle, org)])),
        ),
        (
            "CDN_WRITE",
            Some(user_with("cdn", &[(Role::CdnWrite, org)])),
        ),
        (
            "SUPERUSER only",
            Some(user_with("root", &[(Role::Superuser, org)])),
        ),
        (
            "DISSEMINATION_WORKER only",
            Some(user_with("worker", &[(Role::DisseminationWorker, org)])),
        ),
        (
            "METRICS_READ_SERVICE only",
            Some(user_with("reader", &[(Role::MetricsReadService, org)])),
        ),
        (
            "every non-ingest role together",
            Some(user_with(
                "everything-else",
                &[
                    (Role::Superuser, org),
                    (Role::PublisherAdmin, org),
                    (Role::PublisherUser, org),
                    (Role::DisseminationWorker, org),
                    (Role::MetricsReadService, org),
                    (Role::WorkLifecycle, org),
                    (Role::CdnWrite, org),
                ],
            )),
        ),
    ]
}

// --------------------------------------------------------------------------
// Operation documents
// --------------------------------------------------------------------------

fn claim_doc(limit: i32) -> String {
    format!(
        "mutation {{ claimMetricSourceUnits(input: {{ sourceCode: \"{SOURCE_CODE}\", limit: {limit} }}) \
           {{ partitionKey leaseToken leaseExpiresAt \
              source {{ sourceId code acquisitionType driverKey enabled }} \
              sourceAccount {{ sourceAccountId code expectedPublisherId configuration {{ kind cloudfrontLegacyS3 {{ hostname bucket prefix }} }} }} \
              checkpoint {{ sourceCheckpointId sourceAccountId partitionKey lastDiscoveredAt lastCompletedAt lastSuccessfulPeriodEnd leaseExpiresAt updatedAt }} }} }}"
    )
}

fn begin_doc(token: &str, upstream: &str, keys: &[&str]) -> String {
    let keys: Vec<String> = keys.iter().map(|key| format!("\"{key}\"")).collect();
    format!(
        "mutation {{ beginMetricImport(input: {{ sourceAccountCode: \"{ACCOUNT_A}\", leaseToken: \"{token}\", \
           upstreamReportId: \"{upstream}\", periodStart: \"2026-03-01\", periodEnd: \"2026-03-02\", \
           formatCode: \"cloudfront-legacy-s3\", formatVersion: \"1\", normalizerVersion: \"sphinx-cloudfront/1\", \
           manifestDigest: \"{DIGEST}\", expectedBatchKeys: [{}] }}) \
           {{ importId sourceAccountId publisherId periodStart periodEnd status completedAt }} }}",
        keys.join(", ")
    )
}

fn ingest_doc(import_id: &str, token: &str, key: &str, value: &str) -> String {
    format!(
        "mutation {{ ingestMetricBatch(input: {{ importId: \"{import_id}\", leaseToken: \"{token}\", \
           batchKey: \"{key}\", schemaVersion: \"thoth-normalized-metrics/1\", \
           observations: [{{ sourceAccountCode: \"{ACCOUNT_A}\", platformCode: \"cf\", measureCode: \"title_sessions\", \
             workDoi: \"{WORK_DOI}\", periodStart: \"2026-03-01\", periodEnd: \"2026-03-02\", reportingGrain: DAY, \
             value: \"{value}\", methodologyVersion: \"cloudfront-title-session/2\", sourceRowNumber: 0 }}], \
           coverage: [{{ platformCode: \"cf\", measureCode: \"title_sessions\", periodStart: \"2026-03-01\", \
             periodEnd: \"2026-03-02\", status: COMPLETE, countryCoverage: false, institutionCoverage: false }}] }}) \
           {{ importBatchId requestHash replayed rows {{ batchRowIndex classification reasonCode recordId identityHash contentHash }} }} }}"
    )
}

fn complete_doc(import_id: &str, token: &str) -> String {
    format!(
        "mutation {{ completeMetricImport(input: {{ importId: \"{import_id}\", leaseToken: \"{token}\" }}) \
           {{ importId status completedAt }} }}"
    )
}

fn update_doc(import_id: &str, token: &str) -> String {
    format!(
        "mutation {{ updateMetricSourceCheckpoint(input: {{ importId: \"{import_id}\", leaseToken: \"{token}\" }}) \
           {{ sourceCheckpointId lastCompletedAt lastSuccessfulPeriodEnd leaseExpiresAt }} }}"
    )
}

/// One of each operation document, addressed at real fixture state.
fn every_operation(import_id: &str, token: &str) -> Vec<(&'static str, String)> {
    vec![
        ("claimMetricSourceUnits", claim_doc(10)),
        (
            "beginMetricImport",
            begin_doc(token, "report-matrix", &["b1", "b2"]),
        ),
        (
            "ingestMetricBatch",
            ingest_doc(import_id, token, "b1", "10"),
        ),
        ("completeMetricImport", complete_doc(import_id, token)),
        ("updateMetricSourceCheckpoint", update_doc(import_id, token)),
    ]
}

// --------------------------------------------------------------------------
// Schema contract
// --------------------------------------------------------------------------

/// Signatures with descriptions removed and whitespace collapsed.
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

/// The value names of one SDL enum, in declaration order.
///
/// Every value of these enums carries a description, so a value is the final
/// token of a line whose preceding text closes a description string.
fn enum_values(sdl: &str, name: &str) -> Vec<String> {
    sdl_block(sdl, &format!("enum {name} {{"))
        .lines()
        .filter_map(|line| {
            let (prefix, value) = line.trim().rsplit_once(char::is_whitespace)?;
            (prefix.trim_end().ends_with('"')
                && value
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()))
            .then(|| value.to_string())
        })
        .collect()
}

#[test]
fn the_sdl_exposes_exactly_the_five_approved_operations_and_no_sixth() {
    let sdl = create_schema().as_sdl();
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    let mutation = signatures(mutation_root);
    for operation in [
        "claimMetricSourceUnits(input:ClaimMetricSourceUnitsInput!):[MetricSourceUnitClaim!]!",
        "updateMetricSourceCheckpoint(input:UpdateMetricSourceCheckpointInput!):MetricSourceCheckpoint!",
        "beginMetricImport(input:BeginMetricImportInput!):MetricImport!",
        "ingestMetricBatch(input:IngestMetricBatchInput!):MetricBatchResult!",
        "completeMetricImport(input:CompleteMetricImportInput!):MetricImport!",
    ] {
        assert_eq!(
            mutation.matches(operation).count(),
            1,
            "`{operation}` must be declared exactly once"
        );
    }
    // No sixth ingestion/lifecycle operation by analogy.
    let lifecycle_like: Vec<&str> = mutation_root
        .lines()
        .map(str::trim_start)
        .filter(|line| {
            [
                "claimMetricSource",
                "claimPendingMetric",
                "beginMetric",
                "ingestMetric",
                "completeMetricImport",
                "failMetric",
                "releaseMetric",
                "updateMetricSourceCheckpoint",
                "retryMetric",
                "cancelMetric",
                "recordMetric",
            ]
            .iter()
            .any(|prefix| line.starts_with(prefix))
        })
        .collect();
    assert_eq!(lifecycle_like.len(), 5, "{lifecycle_like:?}");
    // Nothing reaches QueryRoot or a public type.
    let query = sdl_block(&sdl, "type QueryRoot {");
    for name in OPERATIONS.iter().copied().chain([
        "MetricSourceUnitClaim",
        "MetricSourceCheckpoint",
        "MetricImport",
        "MetricBatchResult",
    ]) {
        assert!(!query.contains(name), "`{name}` must not reach QueryRoot");
    }
    for public_type in [
        "type Work {",
        "type Publisher {",
        "type Imprint {",
        "type Publication {",
    ] {
        let block = sdl_block(&sdl, public_type);
        for name in [
            "MetricImport",
            "MetricSourceCheckpoint",
            "MetricBatch",
            "leaseToken",
        ] {
            assert!(
                !block.contains(name),
                "`{public_type}` must not expose `{name}`"
            );
        }
    }
}

#[test]
fn the_lifecycle_objects_and_inputs_match_the_frozen_contract_exactly() {
    let sdl = create_schema().as_sdl();
    let exact = |declaration: &str, expected: &str| {
        assert_eq!(
            signatures(sdl_block(&sdl, declaration)),
            expected,
            "{declaration}"
        );
    };
    exact(
        "type MetricSourceUnitClaim {",
        "source:MetricSource!sourceAccount:MetricSourceAccount!checkpoint:MetricSourceCheckpoint!partitionKey:String!leaseToken:Uuid!leaseExpiresAt:Timestamp!platformCode:String!periodManifestCursor:MetricPeriodManifestCursor",
    );
    exact(
        "type MetricPeriodManifestCursor {",
        "schemaVersion:String!entries:[MetricPeriodManifestCursorEntry!]!",
    );
    exact(
        "type MetricPeriodManifestCursorEntry {",
        "periodStart:Date!manifestDigest:String!",
    );
    exact(
        "type MetricSourceCheckpoint {",
        "sourceCheckpointId:Uuid!sourceAccountId:Uuid!partitionKey:String!lastDiscoveredAt:TimestamplastCompletedAt:TimestamplastSuccessfulPeriodEnd:DateleaseExpiresAt:TimestampupdatedAt:Timestamp!",
    );
    exact(
        "type MetricImport {",
        "importId:Uuid!sourceAccountId:Uuid!publisherId:UuidperiodStart:DateperiodEnd:Datestatus:MetricImportStatus!completedAt:Timestamp",
    );
    exact(
        "type MetricBatchResult {",
        "importBatchId:Uuid!requestHash:String!replayed:Boolean!rows:[MetricIngestionRowResult!]!",
    );
    exact(
        "type MetricIngestionRowResult {",
        "batchRowIndex:Int!classification:MetricRecordProvenanceClassification!reasonCode:MetricIngestionErrorCoderecordId:UuididentityHash:StringcontentHash:String",
    );
    exact(
        "input ClaimMetricSourceUnitsInput {",
        "sourceCode:String!limit:Int=10leaseSeconds:Int=900",
    );
    exact(
        "input BeginMetricImportInput {",
        "sourceAccountCode:String!leaseToken:Uuid!upstreamReportId:String!periodStart:Date!periodEnd:Date!formatCode:String!formatVersion:String!normalizerVersion:String!rawSha256:StringmanifestDigest:String!expectedBatchKeys:[String!]!",
    );
    exact(
        "input IngestMetricBatchInput {",
        "importId:Uuid!leaseToken:Uuid!batchKey:String!schemaVersion:String!observations:[NormalizedMetricObservationInput!]!coverage:[NormalizedMetricCoverageAssertionInput!]!",
    );
    exact(
        "input NormalizedMetricObservationInput {",
        "sourceAccountCode:String!platformCode:String!measureCode:String!workDoi:String!publicationIsbn:StringpublicationType:PublicationTypeperiodStart:Date!periodEnd:Date!reportingGrain:MetricReportingGrain!countryCode:StringinstitutionRor:Stringvalue:String!sourceRecordId:StringmethodologyVersion:String!sourceRowNumber:Int",
    );
    exact(
        "input NormalizedMetricCoverageAssertionInput {",
        "platformCode:String!measureCode:String!periodStart:Date!periodEnd:Date!status:MetricCoverageStatus!countryCoverage:Boolean!institutionCoverage:Boolean!notes:String",
    );
    exact(
        "input CompleteMetricImportInput {",
        "importId:Uuid!leaseToken:Uuid!",
    );
    exact(
        "input UpdateMetricSourceCheckpointInput {",
        "importId:Uuid!leaseToken:Uuid!",
    );
    // The claim token is never persisted-state output, and no raw import or
    // checkpoint persistence field is exposed by implication.
    for absent in [
        "leaseOwner",
        "rawObjectKey",
        "upstreamReportId",
        "manifest",
        "createdBy",
        "receivedCount",
        "invalidCount",
        "lastError",
        "cursor:",
    ] {
        let objects = [
            sdl_block(&sdl, "type MetricSourceCheckpoint {"),
            sdl_block(&sdl, "type MetricImport {"),
        ]
        .join("\n");
        assert!(!objects.contains(absent), "`{absent}` must not be exposed");
    }
    // No new scalar was introduced for this surface.
    for scalar in ["scalar UUID", "scalar DateTime"] {
        assert!(!sdl.contains(scalar), "`{scalar}` must not be introduced");
    }
}

/// Every SDL line declaring a field or argument of exactly `type_name`,
/// ignoring list and non-null wrappers.
fn fields_of_type<'a>(sdl: &'a str, type_name: &str) -> Vec<&'a str> {
    sdl.lines()
        .map(str::trim)
        .filter(|line| {
            line.rsplit_once(": ").is_some_and(|(_, declared)| {
                declared.trim_matches(|c| matches!(c, '[' | ']' | '!')) == type_name
            })
        })
        .collect()
}

#[test]
fn the_claim_context_is_reachable_only_through_the_protected_claim_result() {
    let sdl = create_schema().as_sdl();

    // Each cursor object is returned by exactly one field in the whole schema...
    assert_eq!(
        fields_of_type(&sdl, "MetricPeriodManifestCursor"),
        ["periodManifestCursor: MetricPeriodManifestCursor"]
    );
    assert!(sdl_block(&sdl, "type MetricSourceUnitClaim {")
        .contains("\n  periodManifestCursor: MetricPeriodManifestCursor\n"));
    assert_eq!(
        fields_of_type(&sdl, "MetricPeriodManifestCursorEntry"),
        ["entries: [MetricPeriodManifestCursorEntry!]!"]
    );
    assert!(sdl_block(&sdl, "type MetricPeriodManifestCursor {")
        .contains("\n  entries: [MetricPeriodManifestCursorEntry!]!\n"));
    // ...and the claim itself only by the protected claim mutation.
    let claim_fields = fields_of_type(&sdl, "MetricSourceUnitClaim");
    assert_eq!(claim_fields.len(), 1, "{claim_fields:?}");
    assert!(claim_fields[0].starts_with("claimMetricSourceUnits("));
    assert!(sdl_block(&sdl, "type MutationRoot {").contains(claim_fields[0]));

    // `platformCode` is an output field of the claim and of no other object.
    let declaring_platform_code: Vec<&str> = sdl
        .lines()
        .filter_map(|line| line.strip_prefix("type ")?.strip_suffix(" {"))
        .filter(|name| {
            sdl_block(&sdl, &format!("type {name} {{"))
                .lines()
                .any(|line| line.trim_start().starts_with("platformCode:"))
        })
        .collect();
    assert_eq!(declaring_platform_code, ["MetricSourceUnitClaim"]);

    // No root, checkpoint, registry, import or public object mentions cursor
    // or manifest state.
    for declaration in [
        "type QueryRoot {",
        "type MutationRoot {",
        "type MetricSourceCheckpoint {",
        "type MetricSource {",
        "type MetricSourceAccount {",
        "type MetricPlatform {",
        "type MetricImport {",
        "type Work {",
        "type Publisher {",
        "type Imprint {",
        "type Publication {",
    ] {
        let block = sdl_block(&sdl, declaration).to_lowercase();
        for absent in ["cursor", "manifest"] {
            assert!(
                !block.contains(absent),
                "`{declaration}` must not mention `{absent}`"
            );
        }
    }
}

#[tokio::test]
async fn the_checkpoint_update_accepts_no_caller_supplied_cursor_or_manifest() {
    // An unreachable pool: a document that reached a resolver would report the
    // database classification instead of failing validation.
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    let context = context_for(&unreachable, Some(ingest_user()));
    for member in [
        "cursor: \"{}\"",
        "periodManifestCursor: { schemaVersion: \"thoth-period-manifest-cursor/1\", entries: [] }",
        "manifestDigest: \"0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0\"",
        "periodStart: \"2026-03-01\"",
    ] {
        let document = format!(
            "mutation {{ updateMetricSourceCheckpoint(input: {{ importId: \"{}\", leaseToken: \"{}\", {member} }}) \
               {{ sourceCheckpointId }} }}",
            Uuid::new_v4(),
            Uuid::new_v4()
        );
        let response = run(&schema, &context, &document).await;
        assert!(response["data"].is_null(), "{member}: {response}");
        let errors = response["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "{member}: {response}");
        assert!(
            !response.to_string().contains("INTERNAL_DATABASE_ERROR"),
            "{member} must fail validation before any resolver runs: {response}"
        );
    }
}

#[test]
fn the_three_domain_vocabularies_are_exposed_with_exactly_their_authoritative_values() {
    let sdl = create_schema().as_sdl();
    assert_eq!(
        enum_values(&sdl, "MetricImportStatus"),
        [
            "UPLOADED",
            "QUEUED",
            "PROCESSING",
            "COMPLETED",
            "COMPLETED_WITH_ERRORS",
            "FAILED"
        ]
    );
    for value in enum_values(&sdl, "MetricImportStatus") {
        assert_eq!(
            MetricImportStatus::from_str(&value).unwrap().to_string(),
            value
        );
    }
    assert_eq!(
        enum_values(&sdl, "MetricRecordProvenanceClassification"),
        ["WINNER", "DUPLICATE", "REVISION", "CONFLICT", "REJECTED"]
    );
    for value in enum_values(&sdl, "MetricRecordProvenanceClassification") {
        assert_eq!(
            MetricRecordProvenanceClassification::from_str(&value)
                .unwrap()
                .to_string(),
            value
        );
    }
    // Every Rust variant of the closed coordinator vocabulary, and no other
    // value, appears under its own serialized name.
    let source = include_str!("../model/metric_ingestion/error.rs");
    let body = source
        .split_once("pub enum MetricIngestionErrorCode {")
        .expect("enum")
        .1
        .split_once("\n}")
        .expect("enum end")
        .0;
    let rust_variants: Vec<String> = body
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.ends_with(',') && line.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        })
        .map(|line| line.trim_end_matches(',').to_string())
        .collect();
    let sdl_values = enum_values(&sdl, "MetricIngestionErrorCode");
    assert_eq!(sdl_values.len(), rust_variants.len());
    assert_eq!(sdl_values.len(), 47);
    for value in &sdl_values {
        let parsed = MetricIngestionErrorCode::from_str(value)
            .unwrap_or_else(|_| panic!("`{value}` is not a coordinator code"));
        assert_eq!(&parsed.to_string(), value);
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            JsonValue::String(value.clone())
        );
    }
    // No parallel transport vocabulary exists.
    for absent in [
        "enum MetricLifecycleErrorCode",
        "enum MetricBatchClassification",
        "enum MetricImportState",
    ] {
        assert!(!sdl.contains(absent), "`{absent}` must not exist");
    }
}

#[test]
fn every_resolver_authorizes_before_calling_the_lifecycle_coordinator() {
    let source = include_str!("mutation.rs");
    for (resolver, coordinator) in [
        (
            "fn claim_metric_source_units(",
            "claim_metric_source_units(&context.db",
        ),
        (
            "fn update_metric_source_checkpoint(",
            "update_metric_source_checkpoint(&context.db",
        ),
        ("fn begin_metric_import(", "begin_metric_import(&context.db"),
        (
            "fn ingest_metric_batch(",
            "ingest_metric_batch_under_claim(&context.db",
        ),
        (
            "fn complete_metric_import(",
            "complete_metric_import(&context.db",
        ),
    ] {
        let body = source
            .split_once(resolver)
            .unwrap_or_else(|| panic!("missing resolver {resolver}"))
            .1;
        let body = &body[..body.find("\n    }").expect("resolver end")];
        let guard = body
            .find("authorize_metric_ingestion_lifecycle(context)")
            .unwrap_or_else(|| panic!("{resolver} does not authorize"));
        let call = body
            .find(coordinator)
            .unwrap_or_else(|| panic!("{resolver} does not delegate"));
        assert!(guard < call, "{resolver} must authorize before delegating");
    }
    let helper = source
        .split_once("fn authorize_metric_ingestion_lifecycle(")
        .expect("helper")
        .1;
    let helper = &helper[..helper.find("\n}").expect("helper end")];
    assert!(helper.contains("context.require_metrics_ingest_service()?"));
    assert!(!helper.contains("require_superuser"));
    assert!(!helper.contains("require_metrics_read_service"));
}

// --------------------------------------------------------------------------
// Authorization
// --------------------------------------------------------------------------

/// A fixture with a live claim held by the ingest service and one import.
fn seeded() -> (test_db::TestDbGuard, Fixture, Uuid, Uuid) {
    let (guard, f) = setup();
    let claim = f.claim_a();
    let import = crate::model::metric_ingestion_lifecycle::begin_metric_import(
        &f.pool,
        INGEST_USER,
        &begin_input(claim.lease_token, "report-seeded", day(1)),
    )
    .expect("begin");
    (guard, f, claim.lease_token, import.import_id)
}

#[tokio::test]
async fn every_operation_denies_every_principal_but_the_ingest_service_without_touching_state() {
    let (_guard, f, token, import_id) = seeded();
    let schema = create_schema();
    let before = f.snapshot();
    for (label, user) in denied("org") {
        let context = context_for(&f.pool, user);
        for (operation, document) in every_operation(&import_id.to_string(), &token.to_string()) {
            let response = run(&schema, &context, &document).await;
            assert_unauthorized(&response);
            assert!(
                response["data"].is_null() || response["data"][operation].is_null(),
                "{label} / {operation}: {response}"
            );
        }
        assert_eq!(f.snapshot(), before, "{label}: denial must not touch state");
    }

    // The ingest service alone is admitted (the claim finds nothing new while
    // the seeded unit is leased, but it is authorized).
    let context = context_for(&f.pool, Some(ingest_user()));
    let response = run(&schema, &context, &claim_doc(1)).await;
    assert!(data(&response, "claimMetricSourceUnits").is_array());
}

#[tokio::test]
async fn authorization_is_decided_before_any_operation_specific_database_access() {
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    let token = Uuid::new_v4().to_string();
    let import_id = Uuid::new_v4().to_string();
    for (label, user) in denied("org") {
        let context = context_for(&unreachable, user);
        for (_operation, document) in every_operation(&import_id, &token) {
            assert_unauthorized(&run(&schema, &context, &document).await);
        }
        let _ = label;
    }
    // The authorized caller does reach the database, which is unreachable, and
    // gets the bounded database classification rather than an authorization
    // decision.
    let context = context_for(&unreachable, Some(ingest_user()));
    for (_operation, document) in every_operation(&import_id, &token) {
        assert_error(
            &run(&schema, &context, &document).await,
            "INTERNAL_DATABASE_ERROR",
        );
    }
}

// --------------------------------------------------------------------------
// Lifecycle through GraphQL, and error mapping
// --------------------------------------------------------------------------

#[tokio::test]
async fn a_complete_unit_runs_through_the_five_operations_with_bounded_errors() {
    let (_guard, f) = setup();
    let schema = create_schema();
    let context = context_for(&f.pool, Some(ingest_user()));

    // Claim.
    let response = run(&schema, &context, &claim_doc(1)).await;
    let claims = data(&response, "claimMetricSourceUnits")
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(claims.len(), 1);
    let claim = &claims[0];
    assert_eq!(claim["partitionKey"], "default");
    assert_eq!(claim["checkpoint"]["partitionKey"], "default");
    assert_eq!(claim["source"]["code"], SOURCE_CODE);
    assert_eq!(claim["source"]["acquisitionType"], "DRIVER");
    assert_eq!(claim["sourceAccount"]["code"], ACCOUNT_A);
    assert_eq!(
        claim["sourceAccount"]["configuration"]["kind"],
        "CLOUDFRONT_LEGACY_S3_V1"
    );
    assert_eq!(
        claim["checkpoint"]["leaseExpiresAt"],
        claim["leaseExpiresAt"]
    );
    assert!(claim["checkpoint"]["lastSuccessfulPeriodEnd"].is_null());
    let token = claim["leaseToken"].as_str().unwrap().to_string();
    assert!(Uuid::parse_str(&token).is_ok());

    // A foreign token is stale.
    let foreign = Uuid::new_v4().to_string();
    assert_error(
        &run(
            &schema,
            &context,
            &begin_doc(&foreign, "report-1", &["b1", "b2"]),
        )
        .await,
        "STALE_SOURCE_CLAIM",
    );

    // Begin, and an identical retry.
    let response = run(
        &schema,
        &context,
        &begin_doc(&token, "report-1", &["b1", "b2"]),
    )
    .await;
    let import = data(&response, "beginMetricImport").clone();
    assert_eq!(import["status"], "PROCESSING");
    assert_eq!(import["periodStart"], "2026-03-01");
    assert_eq!(import["publisherId"], f.publisher_id.to_string());
    assert_eq!(import["sourceAccountId"], f.account_a.to_string());
    let retry = run(
        &schema,
        &context,
        &begin_doc(&token, "report-1", &["b1", "b2"]),
    )
    .await;
    assert_eq!(data(&retry, "beginMetricImport"), &import);
    assert_error(
        &run(&schema, &context, &begin_doc(&token, "report-1", &["b1"])).await,
        "IMPORT_IDEMPOTENCY_MISMATCH",
    );
    assert_error(
        &run(&schema, &context, &begin_doc(&token, "report-bad", &[])).await,
        "LIFECYCLE_LIMIT_EXCEEDED",
    );
    let import_id = import["importId"].as_str().unwrap().to_string();

    // Batches: first, replay, reused key, bad value, unexpected key.
    let first = run(
        &schema,
        &context,
        &ingest_doc(&import_id, &token, "b1", "10"),
    )
    .await;
    let first = data(&first, "ingestMetricBatch").clone();
    assert_eq!(first["replayed"], false);
    assert_eq!(first["rows"][0]["batchRowIndex"], 0);
    assert_eq!(first["rows"][0]["classification"], "WINNER");
    assert!(first["rows"][0]["reasonCode"].is_null());
    let replay = run(
        &schema,
        &context,
        &ingest_doc(&import_id, &token, "b1", "10"),
    )
    .await;
    let replay = data(&replay, "ingestMetricBatch").clone();
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["importBatchId"], first["importBatchId"]);
    assert_eq!(replay["rows"], first["rows"]);
    assert_error(
        &run(
            &schema,
            &context,
            &ingest_doc(&import_id, &token, "b1", "11"),
        )
        .await,
        "IDEMPOTENCY_KEY_REUSED",
    );
    assert_error(
        &run(
            &schema,
            &context,
            &ingest_doc(&import_id, &token, "b2", "1.5"),
        )
        .await,
        "LIFECYCLE_LIMIT_EXCEEDED",
    );
    assert_error(
        &run(
            &schema,
            &context,
            &ingest_doc(&import_id, &token, "b9", "10"),
        )
        .await,
        "UNEXPECTED_BATCH_KEY",
    );
    assert_error(
        &run(&schema, &context, &complete_doc(&import_id, &token)).await,
        "IMPORT_INCOMPLETE",
    );
    assert_error(
        &run(&schema, &context, &update_doc(&import_id, &token)).await,
        "INVALID_IMPORT_STATE",
    );
    // The second batch repeats the same canonical observation: a DUPLICATE.
    let second = run(
        &schema,
        &context,
        &ingest_doc(&import_id, &token, "b2", "10"),
    )
    .await;
    assert_eq!(
        data(&second, "ingestMetricBatch")["rows"][0]["classification"],
        "DUPLICATE"
    );

    // Complete and record progress.
    let completed = run(&schema, &context, &complete_doc(&import_id, &token)).await;
    let completed = data(&completed, "completeMetricImport").clone();
    assert_eq!(completed["status"], "COMPLETED");
    assert!(completed["completedAt"].is_string());
    let checkpoint = run(&schema, &context, &update_doc(&import_id, &token)).await;
    let checkpoint = data(&checkpoint, "updateMetricSourceCheckpoint").clone();
    assert_eq!(checkpoint["lastSuccessfulPeriodEnd"], "2026-03-02");
    assert_eq!(checkpoint["lastCompletedAt"], completed["completedAt"]);
    assert!(
        checkpoint["leaseExpiresAt"].is_null(),
        "the claim is released"
    );

    // Unknown identifiers are bounded too.
    let unknown = Uuid::new_v4().to_string();
    assert_error(
        &run(&schema, &context, &complete_doc(&unknown, &token)).await,
        "IMPORT_NOT_FOUND",
    );
    let unknown_source = claim_doc(1).replace(SOURCE_CODE, "no-such-source");
    assert_error(
        &run(&schema, &context, &unknown_source).await,
        "SOURCE_NOT_FOUND",
    );
}

// --------------------------------------------------------------------------
// MET-WP2-03: claim-time platform code and period-manifest cursor
// --------------------------------------------------------------------------

fn claim_context_doc(limit: i32) -> String {
    format!(
        "mutation {{ claimMetricSourceUnits(input: {{ sourceCode: \"{SOURCE_CODE}\", limit: {limit} }}) \
           {{ leaseToken platformCode sourceAccount {{ code }} \
              periodManifestCursor {{ schemaVersion entries {{ periodStart manifestDigest }} }} }} }}"
    )
}

#[tokio::test]
async fn a_claim_returns_its_locked_platform_code_and_the_accepted_period_manifest_cursor() {
    let (_guard, f) = setup();
    let schema = create_schema();
    let context = context_for(&f.pool, Some(ingest_user()));

    // No accepted manifest history: SQL NULL is GraphQL null.
    let response = run(&schema, &context, &claim_context_doc(1)).await;
    let claims = data(&response, "claimMetricSourceUnits")
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0]["sourceAccount"]["code"], ACCOUNT_A);
    assert_eq!(claims[0]["platformCode"], "cf");
    assert!(claims[0]["periodManifestCursor"].is_null());
    let token = claims[0]["leaseToken"].as_str().unwrap().to_string();

    // One successful unit through the protected operations.
    let begun = run(&schema, &context, &begin_doc(&token, "report-1", &["b1"])).await;
    let import_id = data(&begun, "beginMetricImport")["importId"]
        .as_str()
        .unwrap()
        .to_string();
    let batch = run(
        &schema,
        &context,
        &ingest_doc(&import_id, &token, "b1", "10"),
    )
    .await;
    data(&batch, "ingestMetricBatch");
    let completed = run(&schema, &context, &complete_doc(&import_id, &token)).await;
    assert_eq!(
        data(&completed, "completeMetricImport")["status"],
        "COMPLETED"
    );
    let recorded = run(&schema, &context, &update_doc(&import_id, &token)).await;
    assert_eq!(
        data(&recorded, "updateMetricSourceCheckpoint")["lastSuccessfulPeriodEnd"],
        "2026-03-02"
    );

    // The next claim returns the accepted manifest, typed.
    let response = run(&schema, &context, &claim_context_doc(1)).await;
    let claim = data(&response, "claimMetricSourceUnits")[0].clone();
    assert_eq!(claim["sourceAccount"]["code"], ACCOUNT_A);
    assert_eq!(claim["platformCode"], "cf");
    assert_eq!(
        claim["periodManifestCursor"],
        json!({
            "schemaVersion": "thoth-period-manifest-cursor/1",
            "entries": [{"periodStart": "2026-03-01", "manifestDigest": DIGEST}],
        })
    );

    // A stored cursor outside the closed representation fails the claim with
    // the bounded, sanitized code, and no unit is leased.
    f.sql("UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL");
    f.sql(&format!(
        "UPDATE metric_source_checkpoint SET cursor = '{}'::jsonb WHERE source_account_id = '{}'",
        json!({
            "schemaVersion": "thoth-period-manifest-cursor/1",
            "entries": [{"periodStart": "2026-03-01", "manifestDigest": DIGEST, "objectKey": "leak-sentinel"}],
        }),
        f.account_a
    ));
    let before = f.snapshot();
    let response = run(&schema, &context, &claim_context_doc(10)).await;
    assert_error(&response, "INTERNAL_STATE_INCONSISTENCY");
    assert!(
        !response.to_string().contains("leak-sentinel") && !response.to_string().contains(DIGEST),
        "no stored cursor content may reach the caller: {response}"
    );
    assert_eq!(f.snapshot(), before);
}
