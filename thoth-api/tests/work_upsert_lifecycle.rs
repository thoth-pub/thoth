#![cfg(feature = "backend")]

//! `BE-06` conformance transcript (R52B sections 25.17 T258 and 28.9; Amendment 3 section 9.6.1 and 9.6.2: R13, R14,
//! R15, B4).
//!
//! A scripted client drives the four routes end to end through the GraphQL API, against the test database and a
//! stubbed provider, through every branch of R52B section 28.7 that the API can observe. The client holds only what
//! the protocol gives it: the reservation, the finalisation result and its own run state, and never an activation.
//! The prepared artifact is a stand-in with the serializer's document structure built from the reservation's
//! timestamp, batch id and DOIs; the export server's prepared route itself is exercised by that crate's tests (T109,
//! T111, T246). Nothing here contacts Crossref: the provider is a function in this file.

#[allow(dead_code)]
mod support;

use std::sync::Arc;

use diesel::connection::SimpleConnection;
use diesel::sql_types::Text;
use diesel::{PgConnection, QueryableByName, RunQueryDsl};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thoth_api::db::PgPool;
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

// ---------------------------------------------------------------------------------------------------------------------
// The job-call table and the protocol-stop set (Amendment 3 section 9.6.2 rules 1 and 5; R15).
// ---------------------------------------------------------------------------------------------------------------------

const PROTOCOL_STOP: [&str; 2] = [
    "CROSSREF_PERMIT_CLAIM_STALE",
    "CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED",
];

/// The complete job-call table: every code a job call may carry, with its one retryability.
const JOB_CALL_TABLE: [(&str, bool); 15] = [
    ("CROSSREF_PERMIT_VOIDED_RETRYABLE", true),
    ("CROSSREF_ARTIFACT_REFUSED", true),
    ("CROSSREF_PREPARED_FETCH_FAILED", true),
    ("CROSSREF_PROVIDER_NONE_ATTEMPTED", true),
    ("CROSSREF_PROVIDER_INDETERMINATE", false),
    ("CROSSREF_PERMIT_BLOCKED", true),
    ("CROSSREF_BINDING_MOVED_RETRY", true),
    ("WORK_UPSERT_EXECUTION_NOT_PERMITTED", true),
    ("CROSSREF_RESERVATION_JOB_KIND_MISMATCH", false),
    ("WORK_UPSERT_PROFILE_NOT_ADMITTED", false),
    ("CROSSREF_PERMIT_EMPTY_DOI_SET", false),
    ("CROSSREF_DOI_NOT_CANONICALISABLE", false),
    ("CROSSREF_TIMESTAMP_NOT_INCREASING", false),
    ("CROSSREF_TIMESTAMP_NOT_DECODABLE", false),
    ("CROSSREF_TIMESTAMP_OVERFLOW", false),
];

#[derive(Debug, Clone, PartialEq)]
enum JobCall {
    Complete,
    Fail { code: &'static str, retryable: bool },
}

impl JobCall {
    /// Constructing a failure with a protocol-stop code, or with a code outside the table, is refused.
    fn fail(code: &str) -> Result<JobCall, String> {
        if PROTOCOL_STOP.contains(&code) {
            return Err(format!("{code} is a protocol-stop: no job call"));
        }
        JOB_CALL_TABLE
            .iter()
            .find(|(member, _)| *member == code)
            .map(|(member, retryable)| JobCall::Fail {
                code: member,
                retryable: *retryable,
            })
            .ok_or_else(|| format!("{code} is not a job-call code"))
    }
}

#[test]
fn r15_the_job_call_table_and_the_protocol_stop_set() {
    let codes: std::collections::BTreeSet<&str> = JOB_CALL_TABLE.iter().map(|(c, _)| *c).collect();
    assert_eq!(codes.len(), 15, "each code has exactly one retryability");
    for stop in PROTOCOL_STOP {
        assert!(
            !codes.contains(stop),
            "the table and the protocol-stop set are disjoint"
        );
        assert!(
            JobCall::fail(stop).is_err(),
            "no job call is constructed with {stop}"
        );
    }
    let retryable: Vec<&str> = JOB_CALL_TABLE
        .iter()
        .filter(|(_, r)| *r)
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(
        retryable,
        vec![
            "CROSSREF_PERMIT_VOIDED_RETRYABLE",
            "CROSSREF_ARTIFACT_REFUSED",
            "CROSSREF_PREPARED_FETCH_FAILED",
            "CROSSREF_PROVIDER_NONE_ATTEMPTED",
            "CROSSREF_PERMIT_BLOCKED",
            "CROSSREF_BINDING_MOVED_RETRY",
            "WORK_UPSERT_EXECUTION_NOT_PERMITTED",
        ]
    );
    assert_eq!(
        JobCall::fail("CROSSREF_PROVIDER_INDETERMINATE"),
        Ok(JobCall::Fail {
            code: "CROSSREF_PROVIDER_INDETERMINATE",
            retryable: false
        })
    );
    assert!(JobCall::fail("SOMETHING_ELSE").is_err());
}

// ---------------------------------------------------------------------------------------------------------------------
// The stubbed provider, the prepared-artifact stand-in and the section 16.11 extractor.
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Provider {
    Accepts,
    TimesOut,
}

const SUCCESS_TEXT: &str = "Your batch submission was successfully received.";

fn post(provider: Provider, _bytes: &[u8]) -> Option<&'static str> {
    match provider {
        Provider::Accepts => Some(SUCCESS_TEXT),
        Provider::TimesOut => None,
    }
}

fn prepared_artifact(reservation: &Value) -> Vec<u8> {
    let dois = reservation["dois"].as_array().expect("dois");
    let (root, chapters) = dois.split_first().expect("a registration DOI");
    let doi_data = |doi: &Value| {
        format!(
            "<doi_data><doi>{}</doi><resource>https://example.org</resource></doi_data>",
            doi.as_str()
                .expect("doi")
                .trim_start_matches("https://doi.org/")
        )
    };
    let content_items: String = chapters
        .iter()
        .map(|doi| format!("<content_item component_type=\"chapter\"><titles><title>C</title></titles>{}</content_item>", doi_data(doi)))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<doi_batch xmlns=\"http://www.crossref.org/schema/5.4.0\" version=\"5.4.0\">\
         <head><doi_batch_id>{}</doi_batch_id><timestamp>{}</timestamp></head>\
         <body><book book_type=\"monograph\"><book_metadata language=\"en\"><titles><title>T</title></titles>{}\
         <citation_list><citation key=\"r1\"><doi>10.99999/foreign</doi></citation></citation_list></book_metadata>{}</book></body></doi_batch>",
        reservation["doiBatchId"].as_str().expect("batch"),
        reservation["crossrefTimestamp"].as_str().expect("timestamp"),
        doi_data(root),
        content_items
    )
    .into_bytes()
}

#[derive(Debug)]
struct Extracted {
    dois: Vec<String>,
    batch_id: String,
    timestamp: String,
    digest: String,
}

/// R52B section 16.11, behaviour for behaviour, plus the head values of step E.
fn extract(bytes: &[u8]) -> Result<Extracted, String> {
    use quick_xml::events::Event;
    use quick_xml::name::ResolveResult;
    use quick_xml::NsReader;
    const NS: &[u8] = b"http://www.crossref.org/schema/5.4.0";
    if bytes.windows(9).any(|w| w == b"<!DOCTYPE") || bytes.windows(8).any(|w| w == b"<!ENTITY") {
        return Err("refused: DTD".into());
    }
    let prefix = regex::Regex::new(r"(?i)^(https?://)?(www\.)?(dx\.)?doi\.org/").expect("regex");
    let ident = regex::Regex::new(r"^10\.[0-9]{4,9}/[-._;()/:a-zA-Z0-9<>+\[\]]+$").expect("regex");
    let mut reader = NsReader::from_reader(bytes);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<String> = Vec::new();
    let (mut bodies, mut books, mut metas) = (0, 0, 0);
    let mut text = String::new();
    let (mut batch_id, mut timestamp) = (String::new(), String::new());
    let mut dois = std::collections::BTreeSet::new();
    let mut buffer = Vec::new();
    loop {
        match reader.read_resolved_event_into(&mut buffer) {
            Err(error) => return Err(format!("refused: {error}")),
            Ok((resolved, Event::Start(start))) => {
                let in_ns = matches!(resolved, ResolveResult::Bound(ns) if ns.as_ref() == NS);
                let local = String::from_utf8_lossy(start.local_name().as_ref()).to_string();
                if stack.is_empty() && !(in_ns && local == "doi_batch") {
                    return Err("refused: root/namespace".into());
                }
                stack.push(if in_ns { local } else { String::new() });
                match stack.join("/").as_str() {
                    "doi_batch/body" => bodies += 1,
                    "doi_batch/body/book" => books += 1,
                    "doi_batch/body/book/book_metadata"
                    | "doi_batch/body/book/book_series_metadata" => metas += 1,
                    _ => {}
                }
                text.clear();
            }
            Ok((_, Event::Text(t))) => text.push_str(&t.unescape().map_err(|e| e.to_string())?),
            Ok((_, Event::End(_))) => {
                let path = stack.join("/");
                match path.as_str() {
                    "doi_batch/head/doi_batch_id" => batch_id = text.clone(),
                    "doi_batch/head/timestamp" => timestamp = text.clone(),
                    "doi_batch/body/book/book_metadata/doi_data/doi"
                    | "doi_batch/body/book/book_series_metadata/doi_data/doi"
                    | "doi_batch/body/book/content_item/doi_data/doi" => {
                        let raw = prefix.replacen(&text, 1, "").to_string();
                        if text.is_empty() || !ident.is_match(&raw) {
                            return Err("refused: registration doi not canonicalisable".into());
                        }
                        dois.insert(format!("https://doi.org/{}", raw.to_lowercase()));
                    }
                    _ => {}
                }
                stack.pop();
            }
            Ok((_, Event::Eof)) => break,
            Ok(_) => {}
        }
        buffer.clear();
    }
    if bodies != 1 || books != 1 || metas != 1 {
        return Err("refused: container count".into());
    }
    if dois.is_empty() {
        return Err("refused: no registration doi".into());
    }
    Ok(Extracted {
        dois: dois.into_iter().collect(),
        batch_id,
        timestamp,
        digest: hex::encode(Sha256::digest(bytes)),
    })
}

// ---------------------------------------------------------------------------------------------------------------------
// The client.
// ---------------------------------------------------------------------------------------------------------------------

fn worker() -> IntrospectedUser {
    support::user_with_roles("conformance-worker", &[("DISSEMINATION_WORKER", "org")])
}

fn operator() -> IntrospectedUser {
    support::superuser("conformance-operator")
}

struct Client {
    pool: Arc<PgPool>,
    principal: fn() -> IntrospectedUser,
    /// The client's transcript: every API and provider act, in order. Tokens are never recorded.
    transcript: Vec<String>,
    /// Every job call the client made, for the protocol-stop assertions.
    job_calls: Vec<JobCall>,
    /// Claims received and not yet worked on, by Work: the worker's run state.
    claims: std::collections::HashMap<String, (String, String)>,
}

impl Client {
    fn new(pool: &Arc<PgPool>, principal: fn() -> IntrospectedUser) -> Self {
        Client {
            pool: pool.clone(),
            principal,
            transcript: Vec::new(),
            job_calls: Vec::new(),
            claims: std::collections::HashMap::new(),
        }
    }

    async fn call(
        &mut self,
        operation: &str,
        query: &str,
        variables: Value,
    ) -> Result<Value, String> {
        let response = support::execute_graphql(
            self.pool.clone(),
            Some((self.principal)()),
            query,
            Some(variables),
        )
        .await;
        let result = match support::first_error_type(&response) {
            Some(code) => Err(code.to_string()),
            None => Ok(response["data"].clone()),
        };
        let recorded = match &result {
            Ok(_) => "ok".to_string(),
            Err(code) => code.clone(),
        };
        self.transcript.push(format!("{operation} -> {recorded}"));
        result
    }

    async fn reserve_work_upsert(&mut self, job: &str, claim: &str) -> Result<Value, String> {
        self.call(
            "reserveWorkUpsertCrossrefWrite",
            "mutation($d: ReserveWorkUpsertCrossrefWriteInput!) { r: reserveWorkUpsertCrossrefWrite(data: $d) { permitId reservationToken crossrefTimestamp doiBatchId dois publisherIdentity rootWorkIdentity } }",
            json!({"d": {"distributionJobId": job, "claimToken": claim}}),
        )
        .await
        .map(|d| d["r"].clone())
    }

    async fn reserve_back_catalogue(
        &mut self,
        job: &str,
        claim: &str,
        root: Uuid,
    ) -> Result<Value, String> {
        self.call(
            "reserveBackCatalogueCrossrefWrite",
            "mutation($d: ReserveBackCatalogueCrossrefWriteInput!) { r: reserveBackCatalogueCrossrefWrite(data: $d) { permitId reservationToken crossrefTimestamp doiBatchId dois publisherIdentity rootWorkIdentity } }",
            json!({"d": {"distributionJobId": job, "claimToken": claim, "rootWorkId": root}}),
        )
        .await
        .map(|d| d["r"].clone())
    }

    async fn reserve_legacy(&mut self, root: Uuid) -> Result<Value, String> {
        self.call(
            "reserveLegacyScheduledCrossrefWrite",
            "mutation($d: ReserveLegacyScheduledCrossrefWriteInput!) { r: reserveLegacyScheduledCrossrefWrite(data: $d) { permitId reservationToken crossrefTimestamp doiBatchId dois publisherIdentity rootWorkIdentity } }",
            json!({"d": {"rootWorkId": root}}),
        )
        .await
        .map(|d| d["r"].clone())
    }

    async fn reserve_manual(&mut self, root: Uuid, reference: &str) -> Result<Value, String> {
        self.call(
            "reserveManualRecoveryCrossrefWrite",
            "mutation($d: ReserveManualRecoveryCrossrefWriteInput!) { r: reserveManualRecoveryCrossrefWrite(data: $d) { permitId reservationToken crossrefTimestamp doiBatchId dois publisherIdentity rootWorkIdentity } }",
            json!({"d": {"rootWorkId": root, "operatorAuthorizationReference": reference}}),
        )
        .await
        .map(|d| d["r"].clone())
    }

    async fn reserve_jobless(&mut self, route: &str, root: Uuid) -> Result<Value, String> {
        if route == "LEGACY_SCHEDULED" {
            self.reserve_legacy(root).await
        } else {
            self.reserve_manual(root, "INC-MANUAL").await
        }
    }

    /// Step Z, from the reservation and the extraction alone (B4).
    async fn finalise(
        &mut self,
        reservation: &Value,
        claim: Option<&str>,
        extracted: &Extracted,
    ) -> Result<Value, String> {
        self.call(
            "finaliseCrossrefWrite",
            "mutation($d: FinaliseCrossrefWriteInput!) { f: finaliseCrossrefWrite(data: $d) { outcome voidReason permit { state publisherIdentity rootWorkIdentity } } }",
            json!({"d": {
                "permitId": reservation["permitId"],
                "reservationToken": reservation["reservationToken"],
                "claimToken": claim,
                "observedDois": extracted.dois,
                "observedDoiBatchId": extracted.batch_id,
                "observedCrossrefTimestamp": extracted.timestamp,
                "payloadDigest": extracted.digest,
            }}),
        )
        .await
        .map(|d| d["f"].clone())
    }

    async fn report(&mut self, reservation: &Value, outcome: &str) -> Result<Value, String> {
        self.call(
            "reportCrossrefWrite",
            "mutation($d: ReportCrossrefWriteInput!) { p: reportCrossrefWrite(data: $d) { state } }",
            json!({"d": {"permitId": reservation["permitId"], "reservationToken": reservation["reservationToken"], "outcome": outcome}}),
        )
        .await
        .map(|d| d["p"].clone())
    }

    async fn void(&mut self, reservation: &Value, detail: &str) -> Result<Value, String> {
        self.call(
            "voidCrossrefWriteReservation",
            "mutation($d: VoidCrossrefWriteReservationInput!) { p: voidCrossrefWriteReservation(data: $d) { state } }",
            json!({"d": {"permitId": reservation["permitId"], "reservationToken": reservation["reservationToken"], "detail": detail}}),
        )
        .await
        .map(|d| d["p"].clone())
    }

    async fn job_call(&mut self, job: &str, claim: &str, call: JobCall) -> Result<Value, String> {
        assert!(
            !self
                .transcript
                .last()
                .is_some_and(|last| PROTOCOL_STOP.iter().any(|stop| last.ends_with(stop))),
            "no job call follows a protocol-stop"
        );
        self.job_calls.push(call.clone());
        match call {
            JobCall::Complete => self
                .call(
                    "completeDistributionJob",
                    "mutation($d: CompleteDistributionJobInput!) { j: completeDistributionJob(data: $d) { status } }",
                    json!({"d": {"distributionJobId": job, "claimToken": claim}}),
                )
                .await
                .map(|d| d["j"].clone()),
            JobCall::Fail { code, retryable } => self
                .call(
                    "failDistributionJob",
                    "mutation($d: FailDistributionJobInput!) { j: failDistributionJob(data: $d) { status } }",
                    json!({"d": {"distributionJobId": job, "claimToken": claim, "errorCode": code, "retryable": retryable}}),
                )
                .await
                .map(|d| d["j"].clone()),
        }
    }

    /// Steps F, E, Z and, on AUTHORIZED, D, P and O, from the reservation and the finalisation result alone (B4).
    /// Returns the finalisation outcome and, when a report was made, the reported outcome.
    async fn run_protocol(
        &mut self,
        reservation: &Value,
        claim: Option<&str>,
        artifact: Vec<u8>,
        tamper_before_post: bool,
        provider: Provider,
    ) -> ProtocolEnd {
        let extracted = match extract(&artifact) {
            Ok(extracted) => extracted,
            Err(reason) => {
                self.transcript.push(format!("extract -> {reason}"));
                self.void(reservation, "artifact refused")
                    .await
                    .expect("void");
                return ProtocolEnd::ArtifactRefused;
            }
        };
        let result = match self.finalise(reservation, claim, &extracted).await {
            Ok(result) => result,
            Err(code) => {
                if code == "CROSSREF_PERMIT_CLAIM_STALE" {
                    let _ = self.void(reservation, "claim went stale").await;
                }
                return ProtocolEnd::Refused(code);
            }
        };
        match result["outcome"].as_str().expect("outcome") {
            "AUTHORIZED" => {}
            other => {
                return ProtocolEnd::Voided(
                    other.to_string(),
                    result["voidReason"].as_str().unwrap_or("").to_string(),
                )
            }
        }
        // D: recheck the bytes about to be sent against the bound digest.
        let sending = if tamper_before_post {
            [artifact.as_slice(), b" "].concat()
        } else {
            artifact
        };
        if hex::encode(Sha256::digest(&sending)) != extracted.digest {
            self.transcript
                .push("recheck -> mismatch, no POST".to_string());
            self.report(reservation, "NONE_ATTEMPTED")
                .await
                .expect("report");
            return ProtocolEnd::Reported("NONE_ATTEMPTED");
        }
        // P, then O by the report rule.
        let outcome = match post(provider, &sending) {
            Some(body) if body.contains(SUCCESS_TEXT) => "ACCEPTED",
            _ => "INDETERMINATE",
        };
        self.transcript.push(format!("POST -> {outcome}"));
        self.report(reservation, outcome).await.expect("report");
        ProtocolEnd::Reported(outcome)
    }
}

#[derive(Debug, PartialEq)]
enum ProtocolEnd {
    ArtifactRefused,
    Refused(String),
    Voided(String, String),
    Reported(&'static str),
}

// ---------------------------------------------------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------------------------------------------------

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

fn sql(connection: &mut PgConnection, statement: &str) {
    connection
        .batch_execute(statement)
        .unwrap_or_else(|error| panic!("{statement}: {error}"));
}

fn text(connection: &mut PgConnection, query: &str) -> String {
    diesel::sql_query(query)
        .get_result::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("{query}: {error}"))
        .value
}

struct Publisher {
    publisher: Uuid,
    imprint: Uuid,
}

fn covered_publisher(connection: &mut PgConnection) -> Publisher {
    let (publisher, imprint) = (Uuid::new_v4(), Uuid::new_v4());
    sql(
        connection,
        &format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{publisher}', 'Conformance {publisher}');
             INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint}', '{publisher}', 'Conformance {imprint}');
             INSERT INTO publisher_distribution_platform (publisher_id, platform, enabled, activation_id, enabled_at)
             VALUES ('{publisher}', 'CROSSREF', true, gen_random_uuid(), now());"
        ),
    );
    Publisher { publisher, imprint }
}

fn eligible_work(connection: &mut PgConnection, imprint: Uuid) -> Uuid {
    let work = Uuid::new_v4();
    let suffix = work.simple();
    sql(
        connection,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id, doi, publication_date, landing_page)
             VALUES ('{work}', 'monograph', 'active', 1, '{imprint}', 'https://doi.org/10.12345/conf-{suffix}', '2026-01-01', 'https://example.org/{suffix}');
             INSERT INTO title (work_id, locale_code, full_title, title, canonical) VALUES ('{work}', 'en', 'Conformance', 'Conformance', true);
             INSERT INTO publication (publication_type, work_id, isbn) VALUES ('Paperback', '{work}', '978-3-16-148410-0');"
        ),
    );
    work
}

async fn enable_and_admit(pool: &Arc<PgPool>, publisher: Uuid) {
    let mut operator = Client::new(pool, operator);
    operator
        .call(
            "enableWorkUpsertCapture",
            "mutation { enableWorkUpsertCapture(executionProfile: CROSSREF) { captureEnabled } }",
            json!({}),
        )
        .await
        .expect("capture");
    operator
        .call("setWorkUpsertExecution", "mutation { setWorkUpsertExecution(executionProfile: CROSSREF, enabled: true) { executionEnabled } }", json!({}))
        .await
        .expect("execution");
    operator
        .call(
            "admitCrossrefWorkUpsert",
            "mutation($d: AdmitCrossrefWorkUpsertInput!) { admitCrossrefWorkUpsert(data: $d) { actor } }",
            json!({"d": {"publisherId": publisher, "evidenceReference": "CONFORMANCE"}}),
        )
        .await
        .expect("admission");
}

/// Materialize and claim: `(job, claim token)` for `work`.
async fn claimed(client: &mut Client, work: Uuid) -> (String, String) {
    if let Some(held) = client.claims.remove(&work.to_string()) {
        return held;
    }
    client
        .call(
            "materializeWorkUpsertJobs",
            "mutation { materializeWorkUpsertJobs(data: {executionProfiles: [CROSSREF], limit: 100}) { created } }",
            json!({}),
        )
        .await
        .expect("materialize");
    let claimed = client
        .call(
            "claimWorkUpsertJobs",
            "mutation { c: claimWorkUpsertJobs(data: {executionProfiles: [CROSSREF], limit: 10, leaseSeconds: 900}) { claimToken job { distributionJobId workId } } }",
            json!({}),
        )
        .await
        .expect("claim");
    for claim in claimed["c"].as_array().expect("claims") {
        client.claims.insert(
            claim["job"]["workId"].as_str().expect("work").to_string(),
            (
                claim["job"]["distributionJobId"]
                    .as_str()
                    .expect("job")
                    .to_string(),
                claim["claimToken"].as_str().expect("token").to_string(),
            ),
        );
    }
    client
        .claims
        .remove(&work.to_string())
        .unwrap_or_else(|| panic!("the job for {work} was claimed: {claimed}"))
}

fn expire_lease(connection: &mut PgConnection, job: &str) {
    sql(
        connection,
        &format!("UPDATE distribution_job SET lease_expires_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"),
    );
}

fn permit_state(connection: &mut PgConnection, reservation: &Value) -> String {
    text(
        connection,
        &format!(
            "SELECT state::text AS value FROM crossref_write_permit WHERE permit_id = '{}'",
            reservation["permitId"].as_str().expect("permit")
        ),
    )
}

fn attempts(connection: &mut PgConnection, job: &str) -> String {
    text(
        connection,
        &format!(
            "SELECT coalesce(string_agg(coalesce(result::text, 'OPEN') || '/' || (fenced_at IS NOT NULL)::text || '/' || (recovery_cleared_at IS NOT NULL)::text || '/' || coalesce(error_code, '-'), ',' ORDER BY started_at), '') AS value \
             FROM distribution_job_attempt WHERE distribution_job_id = '{job}'"
        ),
    )
}

fn setup() -> (support::TestDbGuard, Arc<PgPool>) {
    let guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    (guard, pool)
}

// ---------------------------------------------------------------------------------------------------------------------
// WORK_UPSERT: the section 28.7 branches, R13 and R14.
// ---------------------------------------------------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn t258_work_upsert_route_every_observable_branch() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    let works: Vec<Uuid> = (0..9)
        .map(|_| eligible_work(&mut connection, fixture.imprint))
        .collect();
    enable_and_admit(&pool, fixture.publisher).await;
    let mut client = Client::new(&pool, worker);

    // 1. Success.
    let (job, claim) = claimed(&mut client, works[0]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    assert_eq!(reservation["rootWorkIdentity"], json!(works[0].to_string()));
    assert_eq!(
        reservation["publisherIdentity"],
        json!(fixture.publisher.to_string()),
        "B3/B4"
    );
    assert_eq!(
        reservation["crossrefTimestamp"].as_str().expect("t").len(),
        17
    );
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            false,
            Provider::Accepts,
        )
        .await;
    assert_eq!(end, ProtocolEnd::Reported("ACCEPTED"));
    assert_eq!(
        client
            .job_call(&job, &claim, JobCall::Complete)
            .await
            .expect("complete")["status"],
        json!("SUCCEEDED")
    );

    // 2a. VOIDED_RETRYABLE: the source changed during preparation.
    let (job, claim) = claimed(&mut client, works[1]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    sql(
        &mut connection,
        &format!(
            "UPDATE work SET place = 'Changed' WHERE work_id = '{}'",
            works[1]
        ),
    );
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            false,
            Provider::Accepts,
        )
        .await;
    assert_eq!(
        end,
        ProtocolEnd::Voided(
            "VOIDED_RETRYABLE".into(),
            "SOURCE_CHANGED_DURING_PREPARATION".into()
        )
    );
    let call = JobCall::fail("CROSSREF_PERMIT_VOIDED_RETRYABLE").expect("job call");
    assert_eq!(
        client.job_call(&job, &claim, call).await.expect("fail")["status"],
        json!("PENDING")
    );

    // 2b. VOIDED_JOB_RETIRED: the binding moved; no job call.
    let (job, claim) = claimed(&mut client, works[2]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let other = covered_publisher(&mut connection);
    sql(
        &mut connection,
        &format!(
            "UPDATE work SET imprint_id = '{}' WHERE work_id = '{}'",
            other.imprint, works[2]
        ),
    );
    let calls_before = client.job_calls.len();
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            false,
            Provider::Accepts,
        )
        .await;
    assert_eq!(
        end,
        ProtocolEnd::Voided("VOIDED_JOB_RETIRED".into(), "BINDING_SUPERSEDED".into())
    );
    assert_eq!(
        client.job_calls.len(),
        calls_before,
        "never failDistributionJob after VOIDED_JOB_RETIRED"
    );

    // 3. Group C: the claim went stale (administrative cancellation); void with the token, then stop.
    let (job, claim) = claimed(&mut client, works[3]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let mut operator_client = Client::new(&pool, operator);
    operator_client
        .call("cancelDistributionJob", "mutation($d: CancelDistributionJobInput!) { cancelDistributionJob(data: $d) { status } }", json!({"d": {"distributionJobId": job}}))
        .await
        .expect("cancel");
    let calls_before = client.job_calls.len();
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            false,
            Provider::Accepts,
        )
        .await;
    assert_eq!(
        end,
        ProtocolEnd::Refused("CROSSREF_PERMIT_CLAIM_STALE".into())
    );
    assert_eq!(
        permit_state(&mut connection, &reservation),
        "VOIDED",
        "voided with the reservation token"
    );
    assert_eq!(
        client.job_calls.len(),
        calls_before,
        "no job call after a protocol-stop"
    );

    // 4. An artifact refused by extraction: void, then fail retryably.
    let (job, claim) = claimed(&mut client, works[4]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let mut refused = prepared_artifact(&reservation);
    refused.splice(0..0, b"<!DOCTYPE doi_batch>".iter().copied());
    assert_eq!(
        client
            .run_protocol(
                &reservation,
                Some(&claim),
                refused,
                false,
                Provider::Accepts
            )
            .await,
        ProtocolEnd::ArtifactRefused
    );
    assert_eq!(permit_state(&mut connection, &reservation), "VOIDED");
    let call = JobCall::fail("CROSSREF_ARTIFACT_REFUSED").expect("job call");
    assert_eq!(
        client.job_call(&job, &claim, call).await.expect("fail")["status"],
        json!("PENDING")
    );

    // 5. A digest-recheck failure: NONE_ATTEMPTED, then fail retryably.
    let (job, claim) = claimed(&mut client, works[5]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            true,
            Provider::Accepts,
        )
        .await;
    assert_eq!(end, ProtocolEnd::Reported("NONE_ATTEMPTED"));
    let call = JobCall::fail("CROSSREF_PROVIDER_NONE_ATTEMPTED").expect("job call");
    assert_eq!(
        client.job_call(&job, &claim, call).await.expect("fail")["status"],
        json!("PENDING")
    );

    // 6. A POST timeout: INDETERMINATE, fail non-retryably, then a superuser reconciliation.
    let (job, claim) = claimed(&mut client, works[6]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let end = client
        .run_protocol(
            &reservation,
            Some(&claim),
            prepared_artifact(&reservation),
            false,
            Provider::TimesOut,
        )
        .await;
    assert_eq!(end, ProtocolEnd::Reported("INDETERMINATE"));
    let call = JobCall::fail("CROSSREF_PROVIDER_INDETERMINATE").expect("job call");
    assert_eq!(
        client.job_call(&job, &claim, call).await.expect("fail")["status"],
        json!("FAILED")
    );
    operator_client
        .call(
            "reconcileCrossrefWritePermit",
            "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
            json!({"d": {"permitId": reservation["permitId"], "outcome": "ACCEPTED", "reconciliationState": "RECONCILED", "authorizationReference": "RECON-6"}}),
        )
        .await
        .expect("reconcile");
    assert_eq!(permit_state(&mut connection, &reservation), "ACCEPTED");

    // 7. A crash before finalisation, cleaned up by the owner void (a worker keeping a restart-safe copy) and,
    //    separately, by the superuser void found through crossrefWritePermits by job and attempt identity.
    for (work, owner) in [(works[7], true), (works[8], false)] {
        let (job, claim) = claimed(&mut client, work).await;
        let reservation = client
            .reserve_work_upsert(&job, &claim)
            .await
            .expect("reserve");
        expire_lease(&mut connection, &job);
        let reclaimed = client
            .call("claimWorkUpsertJobs", "mutation { c: claimWorkUpsertJobs(data: {executionProfiles: [CROSSREF]}) { claimToken } }", json!({}))
            .await
            .expect("claim");
        assert!(
            reclaimed["c"].as_array().expect("claims").is_empty(),
            "clause 7 keeps the job unclaimable"
        );
        assert!(attempts(&mut connection, &job).starts_with("ABANDONED/false/false"));
        if owner {
            client
                .void(&reservation, "crash cleanup")
                .await
                .expect("owner void");
        } else {
            let found = operator_client
                .call(
                    "crossrefWritePermits",
                    "query($j: Uuid) { crossrefWritePermits(jobIdentity: $j, states: [RESERVED]) { permitId attemptIdentity } }",
                    json!({"j": job}),
                )
                .await
                .expect("find");
            let permit = found["crossrefWritePermits"][0]["permitId"].clone();
            assert_eq!(permit, reservation["permitId"]);
            operator_client
                .call(
                    "voidCrossrefWriteReservationAsSuperuser",
                    "mutation($d: VoidCrossrefWriteReservationAsSuperuserInput!) { voidCrossrefWriteReservationAsSuperuser(data: $d) { state } }",
                    json!({"d": {"permitId": permit, "detail": "crash cleanup", "authorizationReference": "INC-7"}}),
                )
                .await
                .expect("superuser void");
        }
        let (again, next_claim) = claimed(&mut client, work).await;
        assert_eq!(again, job);
        assert!(
            client
                .reserve_work_upsert(&again, &next_claim)
                .await
                .is_ok(),
            "the next attempt reserves afresh"
        );
    }
    assert!(
        client
            .transcript
            .iter()
            .all(|entry| !entry.contains("reservationToken")),
        "no token is logged"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn t258_work_upsert_crash_after_authorized_and_a_lost_finalisation_response() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    let works: Vec<Uuid> = (0..2)
        .map(|_| eligible_work(&mut connection, fixture.imprint))
        .collect();
    enable_and_admit(&pool, fixture.publisher).await;
    let mut client = Client::new(&pool, worker);
    let mut operator_client = Client::new(&pool, operator);

    // 8. A crash after AUTHORIZED: the attempt is abandoned fenced; reconciliation clears it; the next attempt deposits
    //    at a strictly later timestamp.
    let (job, claim) = claimed(&mut client, works[0]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let extracted = extract(&prepared_artifact(&reservation)).expect("extract");
    assert_eq!(
        client
            .finalise(&reservation, Some(&claim), &extracted)
            .await
            .expect("finalise")["outcome"],
        json!("AUTHORIZED")
    );
    expire_lease(&mut connection, &job);
    let reclaimed = client
        .call("claimWorkUpsertJobs", "mutation { c: claimWorkUpsertJobs(data: {executionProfiles: [CROSSREF]}) { claimToken } }", json!({}))
        .await
        .expect("claim");
    assert!(reclaimed["c"].as_array().expect("claims").is_empty());
    assert!(attempts(&mut connection, &job).starts_with("ABANDONED/true/false"));
    operator_client
        .call(
            "reconcileCrossrefWritePermit",
            "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
            json!({"d": {"permitId": reservation["permitId"], "outcome": "NONE_ATTEMPTED", "reconciliationState": "RECONCILED", "authorizationReference": "RECON-8"}}),
        )
        .await
        .expect("reconcile");
    assert!(
        attempts(&mut connection, &job).starts_with("ABANDONED/true/true"),
        "the clearance is observed"
    );
    let (again, next_claim) = claimed(&mut client, works[0]).await;
    assert_eq!(again, job);
    let next = client
        .reserve_work_upsert(&again, &next_claim)
        .await
        .expect("reserve again");
    let later = |v: &Value| {
        v["crossrefTimestamp"]
            .as_str()
            .expect("t")
            .parse::<i64>()
            .expect("n")
    };
    assert!(
        later(&next) > later(&reservation),
        "a strictly later timestamp"
    );

    // 9. A lost finalisation response: the retry with the identical digest replays.
    let (job, claim) = claimed(&mut client, works[1]).await;
    let reservation = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    let extracted = extract(&prepared_artifact(&reservation)).expect("extract");
    let first = client
        .finalise(&reservation, Some(&claim), &extracted)
        .await
        .expect("finalise");
    let replay = client
        .finalise(&reservation, Some(&claim), &extracted)
        .await
        .expect("replay");
    assert_eq!(first, replay);
    assert_eq!(replay["outcome"], json!("AUTHORIZED"));
}

#[tokio::test(flavor = "current_thread")]
async fn r13_after_already_reserved_every_permit_state_recovers_token_free() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    let states = [
        "RESERVED",
        "VOIDED",
        "AUTHORIZED",
        "INDETERMINATE",
        "ACCEPTED",
        "NONE_ATTEMPTED",
    ];
    let works: Vec<Uuid> = states
        .iter()
        .map(|_| eligible_work(&mut connection, fixture.imprint))
        .collect();
    enable_and_admit(&pool, fixture.publisher).await;
    let mut operator_client = Client::new(&pool, operator);
    let mut dispatcher = Client::new(&pool, worker);

    for (state, work) in states.iter().zip(&works) {
        // The worker's own reservation, brought to the state; then its run state is lost.
        let mut client = Client::new(&pool, worker);
        let (job, claim) = claimed(&mut dispatcher, *work).await;
        let reservation = client
            .reserve_work_upsert(&job, &claim)
            .await
            .expect("reserve");
        match *state {
            "RESERVED" => {}
            "VOIDED" => {
                client
                    .void(&reservation, "step F failed")
                    .await
                    .expect("void");
            }
            _ => {
                let extracted = extract(&prepared_artifact(&reservation)).expect("extract");
                client
                    .finalise(&reservation, Some(&claim), &extracted)
                    .await
                    .expect("finalise");
                if *state != "AUTHORIZED" {
                    client.report(&reservation, state).await.expect("report");
                }
            }
        }
        assert_eq!(permit_state(&mut connection, &reservation), *state);

        // A worker without its pair repeats the reservation: the protocol-stop, and no job call.
        let mut restarted = Client::new(&pool, worker);
        assert_eq!(
            restarted.reserve_work_upsert(&job, &claim).await,
            Err("CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED".to_string()),
            "{state}"
        );
        assert!(restarted.job_calls.is_empty(), "{state}: no job call");
        // failDistributionJob would be refused while RESERVED or AUTHORIZED anyway.
        if matches!(*state, "RESERVED" | "AUTHORIZED") {
            let expected = if *state == "RESERVED" {
                "ATTEMPT_HAS_OPEN_RESERVATION"
            } else {
                "ATTEMPT_HAS_AUTHORIZED_PERMIT"
            };
            let probe = support::execute_graphql(
                pool.clone(),
                Some(worker()),
                "mutation($d: FailDistributionJobInput!) { failDistributionJob(data: $d) { status } }",
                Some(json!({"d": {"distributionJobId": job, "claimToken": claim, "errorCode": "CROSSREF_PERMIT_VOIDED_RETRYABLE", "retryable": true}})),
            )
            .await;
            assert_eq!(
                support::first_error_type(&probe),
                Some(expected),
                "{state}: {probe}"
            );
        }

        // The lease lapses; recovery closes the attempt ABANDONED, unfenced for RESERVED and VOIDED, fenced otherwise.
        expire_lease(&mut connection, &job);
        let reclaimed = restarted
            .call("claimWorkUpsertJobs", "mutation { c: claimWorkUpsertJobs(data: {executionProfiles: [CROSSREF]}) { claimToken job { distributionJobId } } }", json!({}))
            .await
            .expect("claim");
        let fenced = !matches!(*state, "RESERVED" | "VOIDED");
        assert!(
            attempts(&mut connection, &job).starts_with(&format!("ABANDONED/{fenced}/false")),
            "{state}: {}",
            attempts(&mut connection, &job)
        );
        let claimable_now = reclaimed["c"]
            .as_array()
            .expect("claims")
            .iter()
            .any(|c| c["job"]["distributionJobId"] == json!(job));
        assert_eq!(
            claimable_now,
            *state == "VOIDED",
            "{state}: only VOIDED is claimable before recovery"
        );

        // The table's token-free act.
        match *state {
            "RESERVED" => {
                operator_client
                    .call(
                        "voidCrossrefWriteReservationAsSuperuser",
                        "mutation($d: VoidCrossrefWriteReservationAsSuperuserInput!) { voidCrossrefWriteReservationAsSuperuser(data: $d) { state } }",
                        json!({"d": {"permitId": reservation["permitId"], "detail": "R13 cleanup", "authorizationReference": "INC-R13"}}),
                    )
                    .await
                    .expect("superuser void");
            }
            "VOIDED" => {}
            "AUTHORIZED" | "INDETERMINATE" | "ACCEPTED" | "NONE_ATTEMPTED" => {
                let outcome = if *state == "NONE_ATTEMPTED" {
                    "NONE_ATTEMPTED"
                } else {
                    "ACCEPTED"
                };
                operator_client
                    .call(
                        "reconcileCrossrefWritePermit",
                        "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
                        json!({"d": {"permitId": reservation["permitId"], "outcome": outcome, "reconciliationState": "RECONCILED", "authorizationReference": "RECON-R13"}}),
                    )
                    .await
                    .expect("reconcile");
            }
            _ => unreachable!(),
        }
        // The next claim's attempt reserves; the refused attempt carries no protocol-stop code.
        let next_claim = if claimable_now {
            reclaimed["c"]
                .as_array()
                .expect("claims")
                .iter()
                .find(|c| c["job"]["distributionJobId"] == json!(job))
                .expect("claimed")["claimToken"]
                .as_str()
                .expect("token")
                .to_string()
        } else {
            let (again, token) = claimed(&mut restarted, *work).await;
            assert_eq!(again, job, "{state}");
            token
        };
        assert!(
            restarted
                .reserve_work_upsert(&job, &next_claim)
                .await
                .is_ok(),
            "{state}: the next attempt reserves"
        );
        let codes = text(
            &mut connection,
            &format!(
                "SELECT coalesce(string_agg(DISTINCT coalesce(error_code, '-'), ','), '') AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'"
            ),
        );
        assert!(!codes.contains("ALREADY_RESERVED"), "{state}: {codes}");
        assert!(
            !attempts(&mut connection, &job).contains("FAILED"),
            "{state}: no first attempt is FAILED"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn r14_a_lost_reservation_response_is_recovered_without_the_token() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    let work = eligible_work(&mut connection, fixture.imprint);
    let uncommitted = eligible_work(&mut connection, fixture.imprint);
    enable_and_admit(&pool, fixture.publisher).await;
    let mut client = Client::new(&pool, worker);
    let mut operator_client = Client::new(&pool, operator);

    // The reservation commits and its response is dropped: the client keeps nothing.
    let (job, claim) = claimed(&mut client, work).await;
    let _dropped = client
        .reserve_work_upsert(&job, &claim)
        .await
        .expect("reserve");
    drop(_dropped);
    // The identical reservation, once: refused, and the client records the protocol-stop and stops.
    assert_eq!(
        client.reserve_work_upsert(&job, &claim).await,
        Err("CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED".into())
    );
    assert!(client.job_calls.is_empty());
    // No query returns the token (T170).
    let permits = operator_client
        .call("crossrefWritePermits", "query($j: Uuid) { crossrefWritePermits(jobIdentity: $j) { permitId attemptIdentity state } }", json!({"j": job}))
        .await
        .expect("permits");
    assert!(!permits.to_string().contains("reservationToken"));
    let permit = permits["crossrefWritePermits"][0]["permitId"].clone();
    // The lease lapses; the operator voids as superuser; the next claim's attempt reserves.
    expire_lease(&mut connection, &job);
    operator_client
        .call(
            "voidCrossrefWriteReservationAsSuperuser",
            "mutation($d: VoidCrossrefWriteReservationAsSuperuserInput!) { voidCrossrefWriteReservationAsSuperuser(data: $d) { state } }",
            json!({"d": {"permitId": permit, "detail": "lost response", "authorizationReference": "INC-R14"}}),
        )
        .await
        .expect("superuser void");
    let (again, next_claim) = claimed(&mut client, work).await;
    assert_eq!(again, job);
    assert!(client
        .reserve_work_upsert(&again, &next_claim)
        .await
        .is_ok());
    // The same (old) attempt is refused throughout.
    assert_eq!(
        client.reserve_work_upsert(&job, &claim).await,
        Err("CROSSREF_PERMIT_CLAIM_STALE".into())
    );

    // The variant where the first reservation did not commit: the repeat reserves.
    // A one-shot test-only trigger fails the first reservation's membership insert inside its transaction, so the
    // reservation rolls back and the client sees an INTERNAL_ERROR: an unknown outcome.
    let (job, claim) = claimed(&mut client, uncommitted).await;
    sql(
        &mut connection,
        "CREATE SCHEMA IF NOT EXISTS be06_test;
         CREATE TABLE IF NOT EXISTS be06_test.fail_once (armed boolean PRIMARY KEY);
         INSERT INTO be06_test.fail_once VALUES (true) ON CONFLICT DO NOTHING;
         CREATE OR REPLACE FUNCTION be06_test.fail_once() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN
             IF EXISTS (SELECT 1 FROM be06_test.fail_once) THEN
                 DELETE FROM be06_test.fail_once;
                 RAISE EXCEPTION 'r14 simulated failure before commit';
             END IF;
             RETURN NEW;
         END $$;
         CREATE TRIGGER be06_test_r14_fail_once BEFORE INSERT ON public.crossref_write_permit_doi
             FOR EACH ROW EXECUTE FUNCTION be06_test.fail_once();",
    );
    let first = client.reserve_work_upsert(&job, &claim).await;
    // The DELETE of the arming row rolled back with the reservation; disarm it outside that transaction.
    sql(
        &mut connection,
        "DROP TRIGGER be06_test_r14_fail_once ON public.crossref_write_permit_doi; DELETE FROM be06_test.fail_once;",
    );
    assert_eq!(
        first,
        Err("INTERNAL_ERROR".to_string()),
        "an unknown outcome"
    );
    assert_eq!(
        text(
            &mut connection,
            &format!("SELECT count(*)::text AS value FROM crossref_write_permit WHERE job_identity = '{job}'")
        ),
        "0",
        "the first reservation did not commit"
    );
    assert!(
        client.reserve_work_upsert(&job, &claim).await.is_ok(),
        "the repeat reserves"
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// The other three routes.
// ---------------------------------------------------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn t258_legacy_and_manual_routes_every_observable_branch() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    for (route, principal) in [
        ("LEGACY_SCHEDULED", worker as fn() -> IntrospectedUser),
        ("MANUAL_RECOVERY", operator as fn() -> IntrospectedUser),
    ] {
        let mut client = Client::new(&pool, principal);
        let mut operator_client = Client::new(&pool, operator);
        let works: Vec<Uuid> = (0..6)
            .map(|_| eligible_work(&mut connection, fixture.imprint))
            .collect();
        // Success; the run ends.
        let reservation = client
            .reserve_jobless(route, works[0])
            .await
            .expect("reserve");
        assert_eq!(
            client
                .run_protocol(
                    &reservation,
                    None,
                    prepared_artifact(&reservation),
                    false,
                    Provider::Accepts
                )
                .await,
            ProtocolEnd::Reported("ACCEPTED"),
            "{route}"
        );
        // VOIDED_RETRYABLE: the run ends without a POST.
        let reservation = client
            .reserve_jobless(route, works[1])
            .await
            .expect("reserve");
        sql(
            &mut connection,
            &format!(
                "UPDATE work SET place = 'Changed' WHERE work_id = '{}'",
                works[1]
            ),
        );
        assert_eq!(
            client
                .run_protocol(
                    &reservation,
                    None,
                    prepared_artifact(&reservation),
                    false,
                    Provider::Accepts
                )
                .await,
            ProtocolEnd::Voided(
                "VOIDED_RETRYABLE".into(),
                "SOURCE_CHANGED_DURING_PREPARATION".into()
            ),
            "{route}"
        );
        // An artifact refused by extraction.
        let reservation = client
            .reserve_jobless(route, works[2])
            .await
            .expect("reserve");
        let wrong_namespace = String::from_utf8(prepared_artifact(&reservation))
            .expect("utf8")
            .replace("5.4.0", "5.3.1")
            .into_bytes();
        assert_eq!(
            client
                .run_protocol(
                    &reservation,
                    None,
                    wrong_namespace,
                    false,
                    Provider::Accepts
                )
                .await,
            ProtocolEnd::ArtifactRefused,
            "{route}"
        );
        // A digest-recheck failure.
        let reservation = client
            .reserve_jobless(route, works[3])
            .await
            .expect("reserve");
        assert_eq!(
            client
                .run_protocol(
                    &reservation,
                    None,
                    prepared_artifact(&reservation),
                    true,
                    Provider::Accepts
                )
                .await,
            ProtocolEnd::Reported("NONE_ATTEMPTED"),
            "{route}"
        );
        // A POST timeout, then reconciliation.
        let reservation = client
            .reserve_jobless(route, works[4])
            .await
            .expect("reserve");
        assert_eq!(
            client
                .run_protocol(
                    &reservation,
                    None,
                    prepared_artifact(&reservation),
                    false,
                    Provider::TimesOut
                )
                .await,
            ProtocolEnd::Reported("INDETERMINATE"),
            "{route}"
        );
        operator_client
            .call(
                "reconcileCrossrefWritePermit",
                "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
                json!({"d": {"permitId": reservation["permitId"], "outcome": "ACCEPTED", "reconciliationState": "RECONCILED", "authorizationReference": "RECON"}}),
            )
            .await
            .expect("reconcile");
        // A crash before finalisation: the permit blocks until a superuser voids it; a lost finalisation response
        // replays.
        let reservation = client
            .reserve_jobless(route, works[5])
            .await
            .expect("reserve");
        assert_eq!(
            client.reserve_jobless(route, works[5]).await,
            Err("CROSSREF_PERMIT_BLOCKED".into()),
            "{route}: blocking"
        );
        operator_client
            .call(
                "voidCrossrefWriteReservationAsSuperuser",
                "mutation($d: VoidCrossrefWriteReservationAsSuperuserInput!) { voidCrossrefWriteReservationAsSuperuser(data: $d) { state } }",
                json!({"d": {"permitId": reservation["permitId"], "detail": "crash", "authorizationReference": "INC"}}),
            )
            .await
            .expect("superuser void");
        let reservation = client
            .reserve_jobless(route, works[5])
            .await
            .expect("reserve again");
        let extracted = extract(&prepared_artifact(&reservation)).expect("extract");
        let first = client
            .finalise(&reservation, None, &extracted)
            .await
            .expect("finalise");
        assert_eq!(
            client
                .finalise(&reservation, None, &extracted)
                .await
                .expect("replay"),
            first,
            "{route}"
        );
        // A crash after AUTHORIZED: reconciliation resolves the blocking permit.
        assert_eq!(
            client.reserve_jobless(route, works[5]).await,
            Err("CROSSREF_PERMIT_BLOCKED".into())
        );
        operator_client
            .call(
                "reconcileCrossrefWritePermit",
                "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
                json!({"d": {"permitId": reservation["permitId"], "outcome": "NONE_ATTEMPTED", "reconciliationState": "RECONCILED", "authorizationReference": "RECON-CRASH"}}),
            )
            .await
            .expect("reconcile");
        assert!(
            client.reserve_jobless(route, works[5]).await.is_ok(),
            "{route}: resolved"
        );
        assert!(client.job_calls.is_empty(), "{route}: jobless");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn t258_back_catalogue_route_units_and_the_outer_job() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let fixture = covered_publisher(&mut connection);
    let units: Vec<Uuid> = (0..4)
        .map(|_| eligible_work(&mut connection, fixture.imprint))
        .collect();
    sql(
        &mut connection,
        &format!(
            "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
             SELECT 'PUBLISHER_BACK_CATALOGUE', publisher_id, activation_id, 'PUBLISHER_BACK_CATALOGUE:' || publisher_id || ':' || activation_id \
               FROM publisher_distribution_platform WHERE publisher_id = '{p}' AND platform = 'CROSSREF'; \
             INSERT INTO distribution_job_target (distribution_job_id, platform) \
             SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE publisher_id = '{p}'",
            p = fixture.publisher
        ),
    );
    let mut client = Client::new(&pool, worker);
    let claimed = client
        .call("claimDistributionJobs", "mutation { c: claimDistributionJobs(data: {kinds: [PUBLISHER_BACK_CATALOGUE]}) { claimToken job { distributionJobId } } }", json!({}))
        .await
        .expect("claim");
    let outer = claimed["c"][0]["job"]["distributionJobId"]
        .as_str()
        .expect("job")
        .to_string();
    let claim = claimed["c"][0]["claimToken"]
        .as_str()
        .expect("claim")
        .to_string();

    // Unit 1 accepted; unit 1 again is already deposited in this outer job.
    let reservation = client
        .reserve_back_catalogue(&outer, &claim, units[0])
        .await
        .expect("unit 1");
    assert_eq!(
        client
            .run_protocol(
                &reservation,
                Some(&claim),
                prepared_artifact(&reservation),
                false,
                Provider::Accepts
            )
            .await,
        ProtocolEnd::Reported("ACCEPTED")
    );
    assert_eq!(
        client
            .reserve_back_catalogue(&outer, &claim, units[0])
            .await,
        Err("CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB".into())
    );
    // Unit 2 VOIDED_RETRYABLE: skip it and record it.
    let reservation = client
        .reserve_back_catalogue(&outer, &claim, units[1])
        .await
        .expect("unit 2");
    sql(
        &mut connection,
        &format!(
            "UPDATE work SET place = 'Changed' WHERE work_id = '{}'",
            units[1]
        ),
    );
    assert!(matches!(
        client.run_protocol(&reservation, Some(&claim), prepared_artifact(&reservation), false, Provider::Accepts).await,
        ProtocolEnd::Voided(outcome, _) if outcome == "VOIDED_RETRYABLE"
    ));
    // Unit 3 extraction refused; unit 4 timed out and blocks the outer completion until reconciled.
    let reservation = client
        .reserve_back_catalogue(&outer, &claim, units[2])
        .await
        .expect("unit 3");
    let mut refused = prepared_artifact(&reservation);
    refused.splice(0..0, b"<!DOCTYPE x>".iter().copied());
    assert_eq!(
        client
            .run_protocol(
                &reservation,
                Some(&claim),
                refused,
                false,
                Provider::Accepts
            )
            .await,
        ProtocolEnd::ArtifactRefused
    );
    let reservation = client
        .reserve_back_catalogue(&outer, &claim, units[3])
        .await
        .expect("unit 4");
    assert_eq!(
        client
            .run_protocol(
                &reservation,
                Some(&claim),
                prepared_artifact(&reservation),
                false,
                Provider::TimesOut
            )
            .await,
        ProtocolEnd::Reported("INDETERMINATE")
    );
    let blocked = client.job_call(&outer, &claim, JobCall::Complete).await;
    assert_eq!(
        blocked,
        Err("OUTER_ATTEMPT_HAS_OPEN_PERMITS".into()),
        "no aggregate conceals an ambiguous unit"
    );
    let mut operator_client = Client::new(&pool, operator);
    operator_client
        .call(
            "reconcileCrossrefWritePermit",
            "mutation($d: ReconcileCrossrefWritePermitInput!) { reconcileCrossrefWritePermit(data: $d) { state } }",
            json!({"d": {"permitId": reservation["permitId"], "outcome": "ACCEPTED", "reconciliationState": "RECONCILED", "authorizationReference": "RECON-BC"}}),
        )
        .await
        .expect("reconcile");
    assert_eq!(
        client
            .job_call(&outer, &claim, JobCall::Complete)
            .await
            .expect("complete")["status"],
        json!("SUCCEEDED")
    );
    // Group C after the outer job ended: the claim is stale.
    assert_eq!(
        client
            .reserve_back_catalogue(&outer, &claim, units[1])
            .await,
        Err("CROSSREF_PERMIT_CLAIM_STALE".into())
    );
}
