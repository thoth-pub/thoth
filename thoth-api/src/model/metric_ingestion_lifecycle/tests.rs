//! Focused `MET-WP2-02` PostgreSQL evidence for the managed-DRIVER ingestion
//! lifecycle.
//!
//! Every test runs against the disposable test database and asserts final
//! persisted state: checkpoint bootstrap and leasing, real multi-connection
//! claim races, expiry and reclaim, stale and foreign tokens, import creation
//! and recovery, the two-connection batch guard, completion, non-vacuous
//! checkpoint progress, monotonicity and the crash/retry boundaries of the
//! approved lifecycle. Canonical ingestion semantics themselves are proven in
//! `crate::model::metric_ingestion::tests`; here they are only observed
//! through the unchanged coordinator.
//!
//! The one failure-injection device (a sleeping trigger that holds a batch
//! mid-transaction) exists only in the disposable test database, is defined
//! only in this file and is removed by a `Drop` guard.
//!
//! `MET-WP2-03` evidence for the claim-time platform code and the closed
//! `thoth-period-manifest-cursor/1` checkpoint cursor is asserted here against
//! the persisted cursor itself: first advancement, same-period replacement,
//! bounded retention, the successful-period predicate, stale tokens, replay
//! after release, fail-closed stored state and a real current-versus-stale
//! update race.

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Days, NaiveDate};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{sql_query, Connection, RunQueryDsl};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::{
    begin_metric_import, claim_metric_source_units, complete_metric_import, expected_batch_keys,
    ingest_metric_batch_under_claim, update_metric_source_checkpoint, BeginMetricImportInput,
    ClaimMetricSourceUnitsInput, CompleteMetricImportInput, IngestMetricBatchInput,
    MetricLifecycleError as E, MetricSourceUnitClaim, NormalizedMetricCoverageAssertionInput,
    NormalizedMetricObservationInput, UpdateMetricSourceCheckpointInput, DEFAULT_PARTITION_KEY,
    MANAGED_IMPORT_MANIFEST_SCHEMA,
};
use crate::db::PgPool;
use crate::model::distribution_job::DistributionJobCreation;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_import::{MetricImport, MetricImportStatus};
use crate::model::metric_ingestion::MetricIngestionErrorCode as Code;
use crate::model::metric_platform::crud::update_metric_platform;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_platform::PatchMetricPlatform;
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification as Class;
use crate::model::metric_source::crud::update_metric_source;
use crate::model::metric_source::PatchMetricSource;
use crate::model::metric_source_account::crud::update_metric_source_account;
use crate::model::metric_source_account::{
    MetricCloudFrontLegacyS3ConfigurationInput, MetricSourceAccount,
    MetricSourceAccountConfigurationInput, MetricSourceAccountConfigurationKind,
    PatchMetricSourceAccount,
};
use crate::model::metric_source_checkpoint::MetricSourceCheckpoint;
use crate::model::publisher::{Publisher, ThothPackage};
use crate::model::publisher_service_configuration::crud::replace_publisher_service_configuration;
use crate::model::publisher_service_configuration::{
    PublisherServiceConfigurationSource, ReplacePublisherServiceConfigurationInput,
    ServiceConfigurationWriteContext,
};
use crate::model::tests::db::{test_db_url, TestDbGuard};

pub(crate) const SOURCE_CODE: &str = "cloudfront-driver";
pub(crate) const ACCOUNT_A: &str = "acct-a";
const ACCOUNT_B: &str = "acct-b";
const ACTOR: &str = "metrics-ingest-service-1";
const WORK_DOI: &str = "https://doi.org/10.12345/thoth-wp2-02";
const METHODOLOGY: &str = "cloudfront-title-session/2";
pub(crate) const DIGEST: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const RAW_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
/// The persisted cursor schema, spelled out so the stored contract is asserted
/// independently of the implementation's constant.
const CURSOR_SCHEMA: &str = "thoth-period-manifest-cursor/1";

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

/// A distinct valid manifest digest: 64 lowercase hexadecimal characters.
fn digest(seed: u64) -> String {
    format!("{seed:064x}")
}

/// The canonical stored cursor for `entries`, which must already be in
/// strictly ascending period order.
fn cursor_json(entries: &[(NaiveDate, String)]) -> JsonValue {
    json!({
        "schemaVersion": CURSOR_SCHEMA,
        "entries": entries
            .iter()
            .map(|(period, digest)| json!({
                "periodStart": period.format("%Y-%m-%d").to_string(),
                "manifestDigest": digest,
            }))
            .collect::<Vec<_>>(),
    })
}

/// Stored cursor values the closed decoder must refuse, each labelled.
fn unsupported_cursors() -> Vec<(&'static str, JsonValue)> {
    let entry =
        |period: &str, digest: &str| json!({"periodStart": period, "manifestDigest": digest});
    let cursor =
        |entries: Vec<JsonValue>| json!({"schemaVersion": CURSOR_SCHEMA, "entries": entries});
    // Hexadecimal letters, so an uppercase variant really differs.
    let valid = digest(0xabcdef);
    vec![
        ("a JSON null rather than SQL NULL", JsonValue::Null),
        ("an empty object", json!({})),
        ("an array", json!([entry("2026-03-01", &valid)])),
        (
            "an unsupported schema version",
            json!({"schemaVersion": "thoth-period-manifest-cursor/2", "entries": [entry("2026-03-01", &valid)]}),
        ),
        ("no entries member", json!({"schemaVersion": CURSOR_SCHEMA})),
        (
            "an extension member",
            json!({"schemaVersion": CURSOR_SCHEMA, "entries": [entry("2026-03-01", &valid)], "objects": ["cf/a.gz"]}),
        ),
        ("zero entries", cursor(vec![])),
        (
            "an entry extension member",
            cursor(vec![
                json!({"periodStart": "2026-03-01", "manifestDigest": valid, "requestId": "r-1"}),
            ]),
        ),
        (
            "a non-canonical date",
            cursor(vec![entry("2026-3-01", &valid)]),
        ),
        (
            "an impossible date",
            cursor(vec![entry("2026-02-30", &valid)]),
        ),
        (
            "an uppercase digest",
            cursor(vec![entry("2026-03-01", &valid.to_uppercase())]),
        ),
        (
            "a short digest",
            cursor(vec![entry("2026-03-01", &valid[1..])]),
        ),
        (
            "periods out of order",
            cursor(vec![
                entry("2026-03-02", &valid),
                entry("2026-03-01", &valid),
            ]),
        ),
        (
            "a duplicated period",
            cursor(vec![
                entry("2026-03-01", &valid),
                entry("2026-03-01", &valid),
            ]),
        ),
        (
            "65 entries",
            cursor(
                (0..65)
                    .map(|offset| {
                        entry(
                            &date(2026, 1, 1)
                                .checked_add_days(Days::new(offset))
                                .unwrap()
                                .format("%Y-%m-%d")
                                .to_string(),
                            &valid,
                        )
                    })
                    .collect(),
            ),
        ),
    ]
}

pub(crate) fn day(n: u32) -> NaiveDate {
    date(2026, 3, n)
}

/// The canonical rows one managed-DRIVER source needs, on a pool large enough
/// for the two-connection batch guard and for concurrent claimers.
pub(crate) struct Fixture {
    pub(crate) pool: Arc<PgPool>,
    pub(crate) source_id: Uuid,
    pub(crate) platform_id: Uuid,
    pub(crate) publisher_id: Uuid,
    pub(crate) account_a: Uuid,
    pub(crate) account_b: Uuid,
}

fn exec(pool: &PgPool, sql: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(sql)
        .execute(&mut connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn text(pool: &PgPool, query: &str) -> Option<String> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Nullable<diesel::sql_types::Text>,
    >(query))
    .get_result(&mut connection)
    .expect("Failed to run text query")
}

fn cloudfront_configuration(external_key: &str) -> String {
    format!(
        "{{\"schemaVersion\": \"cloudfront-source-account/1\", \"hostname\": \"{external_key}\", \
         \"logging\": {{\"mode\": \"LEGACY_S3\", \"bucket\": \"logs\", \"prefix\": \"cf/\"}}}}"
    )
}

/// A fresh registry database with one enabled CloudFront `DRIVER` source, two
/// eligible accounts (`acct-a`, `acct-b`), an `OBELISK` publisher (which holds
/// `METRICS_COLLECT`), one work and a DAY-grain `title_sessions` mapping.
pub(crate) fn setup() -> (TestDbGuard, Fixture) {
    let (guard, _registry_pool) = setup_registry_db();
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(12)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to create a lifecycle test pool"),
    );
    let source_id = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    let publisher_id = Uuid::new_v4();
    let imprint_id = Uuid::new_v4();
    let account_a = Uuid::new_v4();
    let account_b = Uuid::new_v4();
    exec(&pool, &format!("INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) VALUES ('{source_id}', '{SOURCE_CODE}', 'DRIVER', 'cloudfront', TRUE)"));
    exec(&pool, &format!("INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ('{platform_id}', 'cf', 'CloudFront', 'THOTH_MANAGED', TRUE)"));
    exec(&pool, &format!("INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ('{publisher_id}', 'Managed publisher', 'OBELISK')"));
    for (id, code, key) in [
        (account_a, ACCOUNT_A, "dist-a"),
        (account_b, ACCOUNT_B, "dist-b"),
    ] {
        exec(&pool, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, configuration, enabled) VALUES ('{id}', '{code}', '{source_id}', '{platform_id}', '{key}', '{publisher_id}', '{}'::jsonb, TRUE)", cloudfront_configuration(key)));
    }
    exec(&pool, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint_id}', '{publisher_id}', 'Imprint')"));
    exec(&pool, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{}', 'monograph', 'forthcoming', '{imprint_id}', 1, '{WORK_DOI}')", Uuid::new_v4()));
    exec(&pool, &format!("INSERT INTO metric_platform_measure (platform_id, measure_id, supported_grains, supports_country, supports_institution, supports_publication, direct_collection, enabled) SELECT '{platform_id}', measure_id, ARRAY['DAY']::metric_reporting_grain[], TRUE, FALSE, FALSE, TRUE, TRUE FROM metric_measure WHERE code = 'title_sessions'"));
    (
        guard,
        Fixture {
            pool,
            source_id,
            platform_id,
            publisher_id,
            account_a,
            account_b,
        },
    )
}

fn claim_input(limit: Option<i32>, lease_seconds: Option<i32>) -> ClaimMetricSourceUnitsInput {
    ClaimMetricSourceUnitsInput {
        source_code: SOURCE_CODE.into(),
        limit,
        lease_seconds,
    }
}

impl Fixture {
    pub(crate) fn sql(&self, sql: &str) {
        exec(&self.pool, sql);
    }

    fn count(&self, table: &str, predicate: &str) -> i64 {
        scalar_i64(
            &self.pool,
            &format!("(SELECT COUNT(*) FROM {table} WHERE {predicate})"),
        )
    }

    fn claim(&self, limit: i32) -> Result<Vec<MetricSourceUnitClaim>, E> {
        claim_metric_source_units(&self.pool, &claim_input(Some(limit), None))
    }

    /// Claim exactly account A (the first account by code) and return its claim.
    pub(crate) fn claim_a(&self) -> MetricSourceUnitClaim {
        let claims = self.claim(1).expect("claim must succeed");
        assert_eq!(claims.len(), 1, "exactly one unit must be claimable");
        assert_eq!(claims[0].source_account.source_account_id, self.account_a);
        claims.into_iter().next().unwrap()
    }

    fn checkpoint(&self, account: Uuid) -> MetricSourceCheckpoint {
        use crate::schema::metric_source_checkpoint::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl};
        let mut connection = self.pool.get().unwrap();
        metric_source_checkpoint
            .filter(source_account_id.eq(account))
            .first(&mut connection)
            .expect("checkpoint must exist")
    }

    fn import(&self, import_id: Uuid) -> MetricImport {
        use diesel::QueryDsl;
        let mut connection = self.pool.get().unwrap();
        crate::schema::metric_import::table
            .find(import_id)
            .first(&mut connection)
            .expect("import must exist")
    }

    /// Force the account's lease to have expired one second ago.
    fn expire(&self, account: Uuid) {
        self.sql(&format!("UPDATE metric_source_checkpoint SET lease_expires_at = transaction_timestamp() - interval '1 second' WHERE source_account_id = '{account}'"));
    }

    /// The account checkpoint's stored cursor exactly as PostgreSQL renders it;
    /// `None` is SQL `NULL`.
    fn stored_cursor_text(&self, account: Uuid) -> Option<String> {
        text(
            &self.pool,
            &format!("(SELECT cursor::text FROM metric_source_checkpoint WHERE source_account_id = '{account}')"),
        )
    }

    /// The account checkpoint's stored cursor as JSON; `None` is SQL `NULL`.
    fn stored_cursor(&self, account: Uuid) -> Option<JsonValue> {
        self.stored_cursor_text(account)
            .map(|stored| serde_json::from_str(&stored).expect("a stored cursor is JSON"))
    }

    /// Store `cursor` verbatim on the account's checkpoint, bypassing every
    /// lifecycle rule.
    fn store_cursor(&self, account: Uuid, cursor: &JsonValue) {
        self.sql(&format!("UPDATE metric_source_checkpoint SET cursor = '{cursor}'::jsonb WHERE source_account_id = '{account}'"));
    }

    fn begin(&self, token: Uuid, upstream: &str, start: NaiveDate) -> Result<MetricImport, E> {
        begin_metric_import(&self.pool, ACTOR, &begin_input(token, upstream, start))
    }

    fn ingest(
        &self,
        import_id: Uuid,
        token: Uuid,
        key: &str,
        observations: Vec<NormalizedMetricObservationInput>,
        coverage: Vec<NormalizedMetricCoverageAssertionInput>,
    ) -> Result<super::MetricBatchResult, E> {
        ingest_metric_batch_under_claim(
            &self.pool,
            &IngestMetricBatchInput {
                import_id,
                lease_token: token,
                batch_key: key.into(),
                schema_version: "thoth-normalized-metrics/1".into(),
                observations,
                coverage,
            },
        )
    }

    fn complete(&self, import_id: Uuid, token: Uuid) -> Result<MetricImport, E> {
        complete_metric_import(
            &self.pool,
            &CompleteMetricImportInput {
                import_id,
                lease_token: token,
            },
        )
    }

    fn update(&self, import_id: Uuid, token: Uuid) -> Result<MetricSourceCheckpoint, E> {
        update_metric_source_checkpoint(
            &self.pool,
            &UpdateMetricSourceCheckpointInput {
                import_id,
                lease_token: token,
            },
        )
    }

    /// Every durable row a lifecycle call could create or change, for
    /// no-consequence proofs.
    pub(crate) fn snapshot(&self) -> String {
        text(
            &self.pool,
            "(SELECT concat_ws('|', \
                (SELECT COUNT(*) FROM metric_import), \
                (SELECT COUNT(*) FROM metric_import_batch), \
                (SELECT COUNT(*) FROM metric_record), \
                (SELECT COUNT(*) FROM metric_record_provenance), \
                (SELECT COUNT(*) FROM metric_coverage), \
                (SELECT COUNT(*) FROM metric_rollup_delta), \
                (SELECT string_agg(concat_ws(',', source_account_id, lease_owner, lease_expires_at, \
                    last_discovered_at, last_completed_at, last_successful_period_end, cursor::text), ';' \
                    ORDER BY source_account_id) FROM metric_source_checkpoint), \
                (SELECT string_agg(concat_ws(',', import_id, status, completed_at, received_count, \
                    accepted_count, invalid_count), ';' ORDER BY import_id) FROM metric_import)))",
        )
        .unwrap_or_default()
    }

    /// Run one complete unit through begin, its expected batches and
    /// completion, returning the terminal import.
    fn run_unit(
        &self,
        token: Uuid,
        upstream: &str,
        start: NaiveDate,
        coverage: Vec<NormalizedMetricCoverageAssertionInput>,
    ) -> MetricImport {
        self.run_unit_with(token, upstream, start, DIGEST, "10", coverage)
    }

    /// `run_unit` with a chosen manifest digest and observation value.
    fn run_unit_with(
        &self,
        token: Uuid,
        upstream: &str,
        start: NaiveDate,
        manifest_digest: &str,
        value: &str,
        coverage: Vec<NormalizedMetricCoverageAssertionInput>,
    ) -> MetricImport {
        let input = BeginMetricImportInput {
            expected_batch_keys: vec!["only".into()],
            manifest_digest: manifest_digest.into(),
            ..begin_input(token, upstream, start)
        };
        let import = begin_metric_import(&self.pool, ACTOR, &input).expect("begin");
        self.ingest(
            import.import_id,
            token,
            "only",
            vec![observation(value, start)],
            coverage,
        )
        .expect("batch");
        self.complete(import.import_id, token).expect("complete")
    }

    /// Claim account A, run one unit for `start` that satisfies the successful
    /// period predicate, and record it, returning the claim token, the import
    /// and the checkpoint the update returned.
    fn record_success(
        &self,
        upstream: &str,
        start: NaiveDate,
        manifest_digest: &str,
        value: &str,
    ) -> (Uuid, MetricImport, MetricSourceCheckpoint) {
        let claim = self.claim_a();
        let import = self.run_unit_with(
            claim.lease_token,
            upstream,
            start,
            manifest_digest,
            value,
            vec![complete_day(start)],
        );
        assert_eq!(import.status, MetricImportStatus::Completed, "{upstream}");
        let checkpoint = self
            .update(import.import_id, claim.lease_token)
            .unwrap_or_else(|error| panic!("{upstream}: {error:?}"));
        (claim.lease_token, import, checkpoint)
    }
}

pub(crate) fn begin_input(token: Uuid, upstream: &str, start: NaiveDate) -> BeginMetricImportInput {
    BeginMetricImportInput {
        source_account_code: ACCOUNT_A.into(),
        lease_token: token,
        upstream_report_id: upstream.into(),
        period_start: start,
        period_end: start.succ_opt().unwrap(),
        format_code: "cloudfront-legacy-s3".into(),
        format_version: "1".into(),
        normalizer_version: "sphinx-cloudfront/1".into(),
        raw_sha256: Some(RAW_SHA.into()),
        manifest_digest: DIGEST.into(),
        expected_batch_keys: vec!["b1".into(), "b2".into()],
    }
}

pub(crate) fn observation(value: &str, start: NaiveDate) -> NormalizedMetricObservationInput {
    NormalizedMetricObservationInput {
        source_account_code: ACCOUNT_A.into(),
        platform_code: "cf".into(),
        measure_code: "title_sessions".into(),
        work_doi: WORK_DOI.into(),
        publication_isbn: None,
        publication_type: None,
        period_start: start,
        period_end: start.succ_opt().unwrap(),
        reporting_grain: MetricReportingGrain::Day,
        country_code: None,
        institution_ror: None,
        value: value.into(),
        source_record_id: None,
        methodology_version: METHODOLOGY.into(),
        source_row_number: Some(1),
    }
}

fn coverage(
    status: MetricCoverageStatus,
    start: NaiveDate,
    end: NaiveDate,
) -> NormalizedMetricCoverageAssertionInput {
    NormalizedMetricCoverageAssertionInput {
        platform_code: "cf".into(),
        measure_code: "title_sessions".into(),
        period_start: start,
        period_end: end,
        status,
        country_coverage: false,
        institution_coverage: false,
        notes: None,
    }
}

pub(crate) fn complete_day(start: NaiveDate) -> NormalizedMetricCoverageAssertionInput {
    coverage(
        MetricCoverageStatus::Complete,
        start,
        start.succ_opt().unwrap(),
    )
}

// --------------------------------------------------------------------------
// Claim: bootstrap, bounds, eligibility
// --------------------------------------------------------------------------

#[test]
fn a_non_positive_limit_claims_nothing_and_writes_nothing() {
    let (_guard, f) = setup();
    for limit in [0, -1, i32::MIN] {
        assert!(f.claim(limit).expect("no error").is_empty());
    }
    assert_eq!(f.count("metric_source_checkpoint", "TRUE"), 0);
}

#[test]
fn the_first_claim_bootstraps_one_default_checkpoint_per_account_and_leases_it_under_a_fresh_token()
{
    let (_guard, f) = setup();
    let claims = f.claim(10).expect("claim");
    assert_eq!(claims.len(), 2);
    // Ascending account-code order, each exactly once.
    assert_eq!(claims[0].source_account.code, ACCOUNT_A);
    assert_eq!(claims[1].source_account.code, ACCOUNT_B);
    assert_ne!(claims[0].lease_token, claims[1].lease_token);
    for claim in &claims {
        assert_eq!(claim.source.source_id, f.source_id);
        assert_eq!(claim.checkpoint.partition_key, DEFAULT_PARTITION_KEY);
        assert_eq!(
            claim.checkpoint.lease_owner.as_deref(),
            Some(claim.lease_token.to_string().as_str()),
            "lease_owner stores the canonical lowercase token text"
        );
        assert_eq!(
            claim.checkpoint.lease_expires_at,
            Some(claim.lease_expires_at)
        );
        assert_eq!(claim.checkpoint.cursor, None);
        assert_eq!(claim.period_manifest_cursor, None, "SQL NULL is no history");
        assert_eq!(claim.platform_code, "cf");
        assert_eq!(claim.checkpoint.last_error, None);
        assert_eq!(claim.checkpoint.last_discovered_at, None);
        assert_eq!(claim.checkpoint.last_successful_period_end, None);
    }
    assert_eq!(
        f.count(
            "metric_source_checkpoint",
            "partition_key = 'default' AND lease_owner IS NOT NULL"
        ),
        2
    );
    // A live lease is not claimable again, and no second checkpoint appears.
    assert!(f.claim(10).expect("claim").is_empty());
    assert_eq!(f.count("metric_source_checkpoint", "TRUE"), 2);
}

#[test]
fn claim_limits_and_lease_durations_are_clamped_exactly() {
    let (_guard, f) = setup();
    let lease_of = |account: Uuid| {
        scalar_i64(
            &f.pool,
            &format!("(SELECT EXTRACT(EPOCH FROM lease_expires_at - updated_at)::bigint FROM metric_source_checkpoint WHERE source_account_id = '{account}')"),
        )
    };
    let claims = claim_metric_source_units(&f.pool, &claim_input(Some(1), Some(1))).unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(lease_of(f.account_a), 60, "a lease below 60s is clamped up");
    let claims =
        claim_metric_source_units(&f.pool, &claim_input(Some(i32::MAX), Some(i32::MAX))).unwrap();
    assert_eq!(claims.len(), 1, "a limit above 50 is clamped, not rejected");
    assert_eq!(
        lease_of(f.account_b),
        3600,
        "a lease above 3600s is clamped down"
    );

    f.sql("UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL");
    claim_metric_source_units(&f.pool, &claim_input(None, None)).unwrap();
    assert_eq!(lease_of(f.account_a), 900, "the default lease is 900s");
}

#[test]
fn more_than_fifty_eligible_accounts_yield_at_most_fifty_claims() {
    let (_guard, f) = setup();
    for index in 0..55 {
        let key = format!("dist-{index:02}");
        f.sql(&format!("INSERT INTO metric_source_account (code, source_id, platform_id, external_key, expected_publisher_id, configuration, enabled) VALUES ('extra-{index:02}', '{}', '{}', '{key}', '{}', '{}'::jsonb, TRUE)", f.source_id, f.platform_id, f.publisher_id, cloudfront_configuration(&key)));
    }
    assert_eq!(f.claim(1000).unwrap().len(), 50);
    assert_eq!(f.claim(1000).unwrap().len(), 7);
}

#[test]
fn source_resolution_and_eligibility_fail_closed() {
    let (_guard, f) = setup();
    let unknown = ClaimMetricSourceUnitsInput {
        source_code: "CLOUDFRONT-DRIVER".into(),
        ..claim_input(Some(10), None)
    };
    assert_eq!(
        claim_metric_source_units(&f.pool, &unknown),
        Err(E::SourceNotFound),
        "source codes are matched exactly"
    );

    f.sql("UPDATE metric_source SET enabled = FALSE");
    assert_eq!(f.claim(10), Err(E::SourceNotEligible));
    f.sql("UPDATE metric_source SET enabled = TRUE");

    // Per-account ineligibility skips the account and creates no checkpoint.
    let cases: [(&str, &str, &str); 5] = [
        (
            "disabled account",
            "UPDATE metric_source_account SET enabled = FALSE WHERE code = 'acct-a'",
            "UPDATE metric_source_account SET enabled = TRUE WHERE code = 'acct-a'",
        ),
        (
            "disabled platform",
            "UPDATE metric_platform SET enabled = FALSE",
            "UPDATE metric_platform SET enabled = TRUE",
        ),
        (
            "no pinned publisher",
            "UPDATE metric_source_account SET expected_publisher_id = NULL WHERE code = 'acct-a'",
            "UPDATE metric_source_account SET expected_publisher_id = (SELECT publisher_id FROM publisher LIMIT 1) WHERE code = 'acct-a'",
        ),
        (
            "publisher without METRICS_COLLECT",
            "UPDATE publisher SET subscription_package = 'OASIS'",
            "UPDATE publisher SET subscription_package = 'OBELISK'",
        ),
        (
            "unsupported stored configuration",
            "UPDATE metric_source_account SET configuration = '{\"schemaVersion\": \"unknown/1\"}'::jsonb WHERE code = 'acct-a'",
            "UPDATE metric_source_account SET configuration = '{\"schemaVersion\": \"cloudfront-source-account/1\", \"hostname\": \"dist-a\", \"logging\": {\"mode\": \"LEGACY_S3\", \"bucket\": \"logs\", \"prefix\": \"cf/\"}}'::jsonb WHERE code = 'acct-a'",
        ),
    ];
    for (label, break_sql, repair_sql) in cases {
        f.sql(break_sql);
        let claimed: Vec<String> = f
            .claim(10)
            .unwrap_or_else(|error| panic!("{label}: {error:?}"))
            .into_iter()
            .map(|claim| claim.source_account.code)
            .collect();
        assert!(
            !claimed.contains(&ACCOUNT_A.to_string()),
            "{label}: account A must not be claimed"
        );
        assert_eq!(
            f.count(
                "metric_source_checkpoint",
                &format!("source_account_id = '{}'", f.account_a)
            ),
            0,
            "{label}: an ineligible account gains no checkpoint"
        );
        f.sql(repair_sql);
        f.sql("UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL");
    }
    // A machine role never substitutes for publisher entitlement: with no
    // METRICS_COLLECT capability, nothing at all is claimable.
    f.sql("UPDATE publisher SET subscription_package = 'OASIS'");
    assert!(f.claim(10).unwrap().is_empty());

    // Only DRIVER sources are claimable.
    f.sql("UPDATE publisher SET subscription_package = 'OBELISK'");
    f.sql("UPDATE metric_source SET acquisition_type = 'ADMIN_IMPORT', driver_key = NULL");
    assert_eq!(f.claim(10), Err(E::SourceNotEligible));
}

#[test]
fn a_claim_carries_each_units_own_locked_platform_code_and_its_decoded_cursor() {
    let (_guard, f) = setup();
    // Account B reports through a second enabled platform.
    let platform_eu = Uuid::new_v4();
    f.sql(&format!("INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ('{platform_eu}', 'cf-eu', 'CloudFront EU', 'THOTH_MANAGED', TRUE)"));
    f.sql(&format!(
        "UPDATE metric_source_account SET platform_id = '{platform_eu}' WHERE source_account_id = '{}'",
        f.account_b
    ));
    assert_eq!(f.claim(10).expect("claim").len(), 2);
    reset_leases(&f);

    // Account A holds a full canonical cursor; account B has no history.
    let stored = cursor_json(
        &(0..64)
            .map(|offset| {
                (
                    date(2026, 1, 1)
                        .checked_add_days(Days::new(offset))
                        .unwrap(),
                    digest(offset),
                )
            })
            .collect::<Vec<_>>(),
    );
    f.store_cursor(f.account_a, &stored);

    let claims = f.claim(10).expect("claim");
    assert_eq!(account_codes(&claims), [ACCOUNT_A, ACCOUNT_B]);
    let (a, b) = (&claims[0], &claims[1]);
    assert_eq!(a.platform_code, "cf");
    assert_eq!(b.platform_code, "cf-eu");
    let cursor = a
        .period_manifest_cursor
        .as_ref()
        .expect("a stored cursor is decoded into the claim");
    assert_eq!(cursor.retained_entries().len(), 64);
    assert_eq!(
        cursor.encode(),
        stored,
        "the typed snapshot is the stored cursor"
    );
    assert_eq!(a.checkpoint.cursor, Some(stored));
    assert_eq!(b.period_manifest_cursor, None);
    assert_eq!(cursor.retained_entries()[63].period(), date(2026, 3, 5));
    assert_eq!(cursor.retained_entries()[63].digest(), digest(63));
}

#[test]
fn an_unsupported_stored_cursor_fails_the_claim_closed_before_any_lease_is_written() {
    let (_guard, f) = setup();
    // Bootstrap A's and B's checkpoints, then release them. A third eligible
    // account, C, has no checkpoint yet.
    assert_eq!(f.claim(10).expect("claim").len(), 2);
    reset_leases(&f);
    f.sql(&format!("INSERT INTO metric_source_account (code, source_id, platform_id, external_key, expected_publisher_id, configuration, enabled) VALUES ('acct-c', '{}', '{}', 'dist-c', '{}', '{}'::jsonb, TRUE)", f.source_id, f.platform_id, f.publisher_id, cloudfront_configuration("dist-c")));
    let checkpoints_of_c = "source_account_id = (SELECT source_account_id FROM metric_source_account WHERE code = 'acct-c')";

    // The unsupported cursor is on B. By the time the claim decodes it, the same
    // transaction has already bootstrapped C's checkpoint and leased A, and the
    // failure must undo both.
    for (label, unsupported) in unsupported_cursors() {
        f.store_cursor(f.account_b, &unsupported);
        let stored = f.stored_cursor_text(f.account_b);
        let before = f.snapshot();
        assert_eq!(
            f.claim(10).map(|claims| account_codes(&claims)),
            Err(E::Ingestion(Code::InternalStateInconsistency)),
            "{label}"
        );
        assert_eq!(
            f.snapshot(),
            before,
            "{label}: no lease, no bootstrapped checkpoint and no repair survive"
        );
        assert_eq!(f.stored_cursor_text(f.account_b), stored, "{label}");
        assert_eq!(
            f.count("metric_source_checkpoint", checkpoints_of_c),
            0,
            "{label}"
        );
    }

    // Non-vacuity: frozen at its first lease write, the claim is proven to have
    // reached A's lease before B's cursor, and it still leaves nothing behind.
    f.store_cursor(f.account_b, &json!({}));
    let before = f.snapshot();
    let (claims, ()) = claim_while_paused(&f, "UPDATE", |_| ());
    assert_eq!(
        claims.map(|claims| account_codes(&claims)),
        Err(E::Ingestion(Code::InternalStateInconsistency))
    );
    assert_eq!(f.snapshot(), before);

    // Only a unit the claim may return is decoded. An account skipped at
    // enumeration is never read...
    f.sql("UPDATE metric_source_checkpoint SET cursor = NULL");
    f.store_cursor(f.account_a, &json!({}));
    f.sql("UPDATE metric_source_account SET enabled = FALSE WHERE code = 'acct-a'");
    assert_eq!(
        f.claim(10).map(|claims| account_codes(&claims)),
        Ok(vec![ACCOUNT_B.to_string(), "acct-c".to_string()])
    );
    reset_leases(&f);
    f.sql("UPDATE metric_source_account SET enabled = TRUE WHERE code = 'acct-a'");
    // ...and neither is one skipped by the locked revalidation, whose checkpoint
    // stays unleased.
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| disable_account_a(&f.pool));
    assert_eq!(
        claims.map(|claims| account_codes(&claims)),
        Ok(vec![ACCOUNT_B.to_string(), "acct-c".to_string()])
    );
    assert_eq!(f.checkpoint(f.account_a).lease_owner, None);
}

// --------------------------------------------------------------------------
// Claim concurrency, expiry, reclaim and stale tokens
// --------------------------------------------------------------------------

#[test]
fn concurrent_first_claimers_bootstrap_one_checkpoint_and_exactly_one_wins_each_unit() {
    let (_guard, f) = setup();
    const WORKERS: usize = 8;
    let barrier = Arc::new(Barrier::new(WORKERS));
    let handles: Vec<_> = (0..WORKERS)
        .map(|_| {
            let pool = Arc::clone(&f.pool);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                claim_metric_source_units(&pool, &claim_input(Some(1), None))
            })
        })
        .collect();
    let mut won: Vec<(Uuid, Uuid)> = Vec::new();
    for handle in handles {
        for claim in handle.join().expect("worker").expect("claim must not fail") {
            won.push((claim.source_account.source_account_id, claim.lease_token));
        }
    }
    // Each account is held by at most one worker, and its stored token is the
    // winner's.
    for account in [f.account_a, f.account_b] {
        let winners: Vec<_> = won.iter().filter(|(id, _)| *id == account).collect();
        assert!(winners.len() <= 1, "two workers hold {account}: {won:?}");
        if let Some((_, token)) = winners.first() {
            assert_eq!(f.checkpoint(account).lease_owner, Some(token.to_string()));
        }
    }
    assert_eq!(won.len(), 2, "both units are claimed by someone: {won:?}");
    assert_eq!(
        f.count("metric_source_checkpoint", "TRUE"),
        2,
        "exactly one checkpoint per account survives concurrent bootstrap"
    );
}

#[test]
fn concurrent_reclaimers_of_one_expired_unit_produce_exactly_one_live_claimant() {
    let (_guard, f) = setup();
    f.sql(&format!(
        "UPDATE metric_source_account SET enabled = FALSE WHERE source_account_id = '{}'",
        f.account_b
    ));
    let original = f.claim_a();
    for _round in 0..5 {
        f.expire(f.account_a);
        const WORKERS: usize = 6;
        let barrier = Arc::new(Barrier::new(WORKERS));
        let handles: Vec<_> = (0..WORKERS)
            .map(|_| {
                let pool = Arc::clone(&f.pool);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    claim_metric_source_units(&pool, &claim_input(Some(5), None))
                })
            })
            .collect();
        let tokens: Vec<Uuid> = handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap().unwrap())
            .map(|claim| claim.lease_token)
            .collect();
        assert_eq!(tokens.len(), 1, "exactly one reclaimer wins: {tokens:?}");
        assert_ne!(tokens[0], original.lease_token);
        assert_eq!(
            f.checkpoint(f.account_a).lease_owner,
            Some(tokens[0].to_string())
        );
    }
}

#[test]
fn an_expired_lease_is_reclaimed_under_a_new_token_and_every_old_or_foreign_token_is_stale() {
    let (_guard, f) = setup();
    let first = f.claim_a();
    let import = f
        .begin(first.lease_token, "report-1", day(1))
        .expect("begin");

    // Not yet expired: not reclaimable.
    assert!(f
        .claim(1)
        .unwrap()
        .iter()
        .all(|c| c.source_account.code != ACCOUNT_A));

    f.expire(f.account_a);
    // An expired but not yet reclaimed lease authorizes nothing either.
    let before = f.snapshot();
    assert_eq!(
        f.begin(first.lease_token, "report-1", day(1)),
        Err(E::StaleSourceClaim)
    );
    assert_eq!(f.snapshot(), before);

    let second = f.claim_a();
    assert_ne!(second.lease_token, first.lease_token);

    let foreign = Uuid::new_v4();
    let before = f.snapshot();
    for token in [first.lease_token, foreign] {
        assert_eq!(f.begin(token, "report-1", day(1)), Err(E::StaleSourceClaim));
        assert_eq!(
            f.ingest(
                import.import_id,
                token,
                "b1",
                vec![observation("1", day(1))],
                vec![]
            )
            .map(|r| r.replayed),
            Err(E::StaleSourceClaim)
        );
        assert_eq!(
            f.complete(import.import_id, token),
            Err(E::StaleSourceClaim)
        );
        assert_eq!(f.update(import.import_id, token), Err(E::StaleSourceClaim));
    }
    assert_eq!(
        f.snapshot(),
        before,
        "a stale or foreign token changes nothing"
    );

    // The new token works and recovers the same import.
    let recovered = f
        .begin(second.lease_token, "report-1", day(1))
        .expect("begin");
    assert_eq!(recovered.import_id, import.import_id);
}

// --------------------------------------------------------------------------
// beginMetricImport
// --------------------------------------------------------------------------

#[test]
fn begin_creates_a_processing_import_from_canonical_authority_and_records_discovery() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f
        .begin(claim.lease_token, "report-1", day(1))
        .expect("begin");
    assert_eq!(import.status, MetricImportStatus::Processing);
    assert_eq!(import.source_account_id, f.account_a);
    assert_eq!(import.publisher_id, Some(f.publisher_id));
    assert_eq!(import.created_by, ACTOR);
    assert_eq!(import.raw_object_key, None);
    assert_eq!(import.raw_sha256.as_deref(), Some(RAW_SHA));
    assert_eq!(import.upstream_report_id.as_deref(), Some("report-1"));
    assert_eq!(
        (import.period_start, import.period_end),
        (Some(day(1)), Some(day(2)))
    );
    assert_eq!(
        [
            import.received_count,
            import.accepted_count,
            import.invalid_count
        ],
        [0, 0, 0]
    );
    assert_eq!(
        import.manifest,
        serde_json::json!({
            "schemaVersion": MANAGED_IMPORT_MANIFEST_SCHEMA,
            "manifestDigest": DIGEST,
            "expectedBatchKeys": ["b1", "b2"],
        })
    );
    assert_eq!(
        expected_batch_keys(&import.manifest)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["b1".to_string(), "b2".to_string()]
    );
    assert!(f.checkpoint(f.account_a).last_discovered_at.is_some());
}

#[test]
fn an_identical_begin_retry_returns_the_same_import_without_moving_discovery() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let first = f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    let discovered = f.checkpoint(f.account_a).last_discovered_at;
    let before = f.snapshot();

    let retry = f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    assert_eq!(retry, first);
    // The expected keys are a set: another order is the same request.
    let reordered = BeginMetricImportInput {
        expected_batch_keys: vec!["b2".into(), "b1".into()],
        ..begin_input(claim.lease_token, "report-1", day(1))
    };
    assert_eq!(
        begin_metric_import(&f.pool, "another-actor", &reordered).unwrap(),
        first
    );
    assert_eq!(f.snapshot(), before);
    assert_eq!(f.checkpoint(f.account_a).last_discovered_at, discovered);
    assert_eq!(f.count("metric_import", "TRUE"), 1);
}

#[test]
fn a_changed_begin_request_for_the_same_upstream_report_fails_closed() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    let base = begin_input(claim.lease_token, "report-1", day(1));
    let changes: Vec<(&str, BeginMetricImportInput)> = vec![
        (
            "period",
            BeginMetricImportInput {
                period_start: day(2),
                period_end: day(3),
                ..base.clone()
            },
        ),
        (
            "format code",
            BeginMetricImportInput {
                format_code: "other".into(),
                ..base.clone()
            },
        ),
        (
            "format version",
            BeginMetricImportInput {
                format_version: "2".into(),
                ..base.clone()
            },
        ),
        (
            "normalizer",
            BeginMetricImportInput {
                normalizer_version: "sphinx-cloudfront/2".into(),
                ..base.clone()
            },
        ),
        (
            "raw hash",
            BeginMetricImportInput {
                raw_sha256: None,
                ..base.clone()
            },
        ),
        (
            "manifest digest",
            BeginMetricImportInput {
                manifest_digest: "1".repeat(64),
                ..base.clone()
            },
        ),
        (
            "expected keys",
            BeginMetricImportInput {
                expected_batch_keys: vec!["b1".into()],
                ..base.clone()
            },
        ),
    ];
    let before = f.snapshot();
    for (label, changed) in changes {
        assert_eq!(
            begin_metric_import(&f.pool, ACTOR, &changed),
            Err(E::ImportIdempotencyMismatch),
            "{label}"
        );
    }
    assert_eq!(f.snapshot(), before);
}

#[test]
fn begin_input_bounds_are_enforced_before_any_database_access() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let base = begin_input(claim.lease_token, "report-1", day(1));
    let key = |n: usize| format!("k{n}");
    let invalid: Vec<(&str, BeginMetricImportInput)> = vec![
        (
            "blank upstream",
            BeginMetricImportInput {
                upstream_report_id: "  ".into(),
                ..base.clone()
            },
        ),
        (
            "upstream over 512 bytes",
            BeginMetricImportInput {
                upstream_report_id: "é".repeat(257),
                ..base.clone()
            },
        ),
        (
            "two-day period",
            BeginMetricImportInput {
                period_end: day(3),
                ..base.clone()
            },
        ),
        (
            "reversed period",
            BeginMetricImportInput {
                period_end: day(1),
                ..base.clone()
            },
        ),
        (
            "blank format",
            BeginMetricImportInput {
                format_code: "".into(),
                ..base.clone()
            },
        ),
        (
            "format over 128 bytes",
            BeginMetricImportInput {
                format_version: "v".repeat(129),
                ..base.clone()
            },
        ),
        (
            "blank normalizer",
            BeginMetricImportInput {
                normalizer_version: "\t".into(),
                ..base.clone()
            },
        ),
        (
            "uppercase raw hash",
            BeginMetricImportInput {
                raw_sha256: Some("A".repeat(64)),
                ..base.clone()
            },
        ),
        (
            "short raw hash",
            BeginMetricImportInput {
                raw_sha256: Some("a".repeat(63)),
                ..base.clone()
            },
        ),
        (
            "digest not hex",
            BeginMetricImportInput {
                manifest_digest: "g".repeat(64),
                ..base.clone()
            },
        ),
        (
            "no expected keys",
            BeginMetricImportInput {
                expected_batch_keys: vec![],
                ..base.clone()
            },
        ),
        (
            "101 expected keys",
            BeginMetricImportInput {
                expected_batch_keys: (0..101).map(key).collect(),
                ..base.clone()
            },
        ),
        (
            "duplicate keys",
            BeginMetricImportInput {
                expected_batch_keys: vec!["b1".into(), "b1".into()],
                ..base.clone()
            },
        ),
        (
            "blank key",
            BeginMetricImportInput {
                expected_batch_keys: vec![" ".into()],
                ..base.clone()
            },
        ),
        (
            "key over 256 bytes",
            BeginMetricImportInput {
                expected_batch_keys: vec!["k".repeat(257)],
                ..base.clone()
            },
        ),
    ];
    let before = f.snapshot();
    for (label, input) in invalid {
        assert_eq!(
            begin_metric_import(&f.pool, ACTOR, &input),
            Err(E::LifecycleLimitExceeded),
            "{label}"
        );
    }
    assert_eq!(f.snapshot(), before);
    // Bounds validation precedes database access: an unreachable pool still
    // reports the bound, not a database failure.
    let unreachable = crate::model::tests::db::failing_pool();
    assert_eq!(
        begin_metric_import(
            &unreachable,
            ACTOR,
            &BeginMetricImportInput {
                expected_batch_keys: vec![],
                ..base.clone()
            }
        ),
        Err(E::LifecycleLimitExceeded)
    );
    // The exact boundaries are accepted.
    let edge = BeginMetricImportInput {
        upstream_report_id: "r".repeat(512),
        format_code: "f".repeat(128),
        raw_sha256: None,
        expected_batch_keys: (0..100).map(key).collect(),
        ..base
    };
    begin_metric_import(&f.pool, ACTOR, &edge).expect("exact bounds are accepted");
}

#[test]
fn begin_revalidates_account_authority_and_entitlement_under_the_claim() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let before = f.snapshot();

    f.sql("UPDATE publisher SET subscription_package = 'OASIS'");
    assert_eq!(
        f.begin(claim.lease_token, "report-1", day(1)),
        Err(E::MetricsCollectNotEntitled)
    );
    f.sql("UPDATE publisher SET subscription_package = 'OBELISK'");

    f.sql("UPDATE metric_source_account SET enabled = FALSE WHERE code = 'acct-a'");
    assert_eq!(
        f.begin(claim.lease_token, "report-1", day(1)),
        Err(E::SourceNotEligible)
    );
    f.sql("UPDATE metric_source_account SET enabled = TRUE WHERE code = 'acct-a'");

    let unknown = BeginMetricImportInput {
        source_account_code: "ACCT-A".into(),
        ..begin_input(claim.lease_token, "report-1", day(1))
    };
    assert_eq!(
        begin_metric_import(&f.pool, ACTOR, &unknown),
        Err(E::SourceAccountNotFound)
    );
    // Account B's code with account A's token is a claim B does not hold.
    let other = BeginMetricImportInput {
        source_account_code: ACCOUNT_B.into(),
        ..begin_input(claim.lease_token, "report-1", day(1))
    };
    assert_eq!(
        begin_metric_import(&f.pool, ACTOR, &other),
        Err(E::StaleSourceClaim)
    );
    assert_eq!(f.snapshot(), before);
}

#[test]
fn a_terminal_import_is_returned_by_begin_and_never_reopened() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let completed = f.run_unit(
        claim.lease_token,
        "report-1",
        day(1),
        vec![complete_day(day(1))],
    );
    assert_eq!(completed.status, MetricImportStatus::Completed);
    let input = BeginMetricImportInput {
        expected_batch_keys: vec!["only".into()],
        ..begin_input(claim.lease_token, "report-1", day(1))
    };
    let before = f.snapshot();
    assert_eq!(
        begin_metric_import(&f.pool, ACTOR, &input).unwrap(),
        completed
    );
    assert_eq!(f.snapshot(), before);
}

// --------------------------------------------------------------------------
// ingestMetricBatch through the unchanged coordinator
// --------------------------------------------------------------------------

#[test]
fn a_batch_is_delegated_to_the_coordinator_and_its_replay_repeats_no_write() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();

    let first = f
        .ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("10", day(1))],
            vec![complete_day(day(1))],
        )
        .expect("batch");
    assert!(!first.replayed);
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.rows[0].batch_row_index, 0);
    assert_eq!(first.rows[0].classification, Class::Winner);
    assert_eq!(first.rows[0].reason_code, None);
    assert!(first.rows[0].record_id.is_some());
    let after_first = f.snapshot();
    assert_eq!(f.count("metric_rollup_delta", "TRUE"), 1);

    // A timeout-after-commit retry replays the persisted outcome.
    let replay = f
        .ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("10", day(1))],
            vec![complete_day(day(1))],
        )
        .expect("replay");
    assert!(replay.replayed);
    assert_eq!(replay.import_batch_id, first.import_batch_id);
    assert_eq!(replay.request_hash, first.request_hash);
    assert_eq!(replay.rows, first.rows);
    assert_eq!(
        f.snapshot(),
        after_first,
        "a replay performs no canonical write"
    );

    // The same key with a different payload keeps the coordinator's own code.
    assert_eq!(
        f.ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("11", day(1))],
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::Ingestion(Code::IdempotencyKeyReused))
    );
    // A row-level rejection is the coordinator's classification, unchanged.
    let rejected = f
        .ingest(
            import.import_id,
            claim.lease_token,
            "b2",
            vec![NormalizedMetricObservationInput {
                work_doi: "https://doi.org/10.12345/unknown".into(),
                ..observation("3", day(1))
            }],
            vec![],
        )
        .expect("batch");
    assert_eq!(rejected.rows[0].classification, Class::Rejected);
    assert_eq!(rejected.rows[0].reason_code, Some(Code::UnknownDoi));
    assert_eq!(f.import(import.import_id).invalid_count, 1);
}

#[test]
fn batch_transport_bounds_and_expected_keys_fail_before_the_coordinator() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    let before = f.snapshot();

    assert_eq!(
        f.ingest(
            import.import_id,
            claim.lease_token,
            "b3",
            vec![observation("1", day(1))],
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::UnexpectedBatchKey)
    );
    for value in [
        "1.5",
        "+1",
        "01",
        "-0",
        " 1",
        "9223372036854775808",
        "",
        "1e3",
    ] {
        assert_eq!(
            f.ingest(
                import.import_id,
                claim.lease_token,
                "b1",
                vec![observation(value, day(1))],
                vec![]
            )
            .map(|r| r.replayed),
            Err(E::LifecycleLimitExceeded),
            "value {value:?}"
        );
    }
    let negative_row = NormalizedMetricObservationInput {
        source_row_number: Some(-1),
        ..observation("1", day(1))
    };
    assert_eq!(
        f.ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![negative_row],
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::LifecycleLimitExceeded)
    );
    assert_eq!(
        f.ingest(
            Uuid::new_v4(),
            claim.lease_token,
            "b1",
            vec![observation("1", day(1))],
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::ImportNotFound)
    );
    // Inherited coordinator limits keep their authoritative codes.
    assert_eq!(
        f.ingest(import.import_id, claim.lease_token, "b1", vec![], vec![])
            .map(|r| r.replayed),
        Err(E::Ingestion(Code::EmptyBatch))
    );
    assert_eq!(
        f.ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            (0..501).map(|_| observation("1", day(1))).collect(),
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::Ingestion(Code::BatchLimitExceeded))
    );
    let wrong_schema = IngestMetricBatchInput {
        import_id: import.import_id,
        lease_token: claim.lease_token,
        batch_key: "b1".into(),
        schema_version: "thoth-normalized-metrics/2".into(),
        observations: vec![observation("1", day(1))],
        coverage: vec![],
    };
    assert_eq!(
        ingest_metric_batch_under_claim(&f.pool, &wrong_schema).map(|r| r.replayed),
        Err(E::Ingestion(Code::UnsupportedSchemaVersion))
    );
    assert_eq!(f.snapshot(), before);

    // A non-managed import (no managed manifest envelope) is refused.
    f.sql(&format!(
        "UPDATE metric_import SET manifest = '{{}}'::jsonb WHERE import_id = '{}'",
        import.import_id
    ));
    assert_eq!(
        f.ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("1", day(1))],
            vec![]
        )
        .map(|r| r.replayed),
        Err(E::InvalidImportState)
    );
}

/// Drops the ephemeral pause trigger and function on scope exit.
struct PauseTrigger {
    pool: Arc<PgPool>,
}

impl Drop for PauseTrigger {
    fn drop(&mut self) {
        if let Ok(mut connection) = self.pool.get() {
            let _ = sql_query("DROP TRIGGER IF EXISTS wp2_02_test_pause ON metric_import_batch")
                .execute(&mut connection);
            let _ =
                sql_query("DROP FUNCTION IF EXISTS wp2_02_test_pause()").execute(&mut connection);
        }
    }
}

#[test]
fn the_batch_guard_holds_the_checkpoint_lock_for_the_whole_coordinator_call() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();

    let _pause = PauseTrigger {
        pool: Arc::clone(&f.pool),
    };
    f.sql("CREATE FUNCTION wp2_02_test_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(3); RETURN NEW; END $$");
    f.sql("CREATE TRIGGER wp2_02_test_pause BEFORE INSERT ON metric_import_batch FOR EACH ROW EXECUTE FUNCTION wp2_02_test_pause()");

    let pool = Arc::clone(&f.pool);
    let token = claim.lease_token;
    let import_id = import.import_id;
    let worker = thread::spawn(move || {
        ingest_metric_batch_under_claim(
            &pool,
            &IngestMetricBatchInput {
                import_id,
                lease_token: token,
                batch_key: "b1".into(),
                schema_version: "thoth-normalized-metrics/1".into(),
                observations: vec![observation("10", day(1))],
                coverage: vec![],
            },
        )
    });

    // Wait until the coordinator is inside its transaction, paused.
    let deadline = Instant::now() + Duration::from_secs(10);
    while scalar_i64(&f.pool, "(SELECT COUNT(*) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND query LIKE '%INSERT INTO \"metric_import_batch\"%' AND state = 'active')") == 0 {
        assert!(Instant::now() < deadline, "the coordinator never reached its batch insert");
        thread::sleep(Duration::from_millis(20));
    }

    let mut probe = f.pool.get().unwrap();
    // The checkpoint row is locked while the coordinator runs...
    let locked = probe.transaction::<_, diesel::result::Error, _>(|connection| {
        sql_query(format!("SELECT 1 FROM metric_source_checkpoint WHERE source_account_id = '{}' FOR UPDATE NOWAIT", f.account_a))
            .execute(connection)
    });
    assert!(
        locked.is_err(),
        "the guard must hold the checkpoint row lock"
    );
    // ...so the lease cannot be expired or replaced meanwhile...
    let expired = probe.transaction::<_, diesel::result::Error, _>(|connection| {
        sql_query("SET LOCAL lock_timeout = '200ms'").execute(connection)?;
        sql_query(format!("UPDATE metric_source_checkpoint SET lease_expires_at = transaction_timestamp() - interval '1 second' WHERE source_account_id = '{}'", f.account_a)).execute(connection)
    });
    assert!(
        expired.is_err(),
        "no writer may change the lease during the coordinator call"
    );
    // ...and a concurrent claimer skips the locked row instead of waiting.
    let started = Instant::now();
    assert!(f
        .claim(10)
        .unwrap()
        .iter()
        .all(|c| c.source_account.code != ACCOUNT_A));
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "SKIP LOCKED must not wait"
    );

    let outcome = worker.join().unwrap().expect("the paused batch commits");
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    assert_eq!(
        f.checkpoint(f.account_a).lease_owner,
        Some(token.to_string())
    );
}

// --------------------------------------------------------------------------
// completeMetricImport
// --------------------------------------------------------------------------

#[test]
fn completion_requires_every_expected_batch_and_derives_the_terminal_status() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();

    let before = f.snapshot();
    assert_eq!(
        f.complete(import.import_id, claim.lease_token),
        Err(E::ImportIncomplete)
    );
    f.ingest(
        import.import_id,
        claim.lease_token,
        "b1",
        vec![observation("10", day(1))],
        vec![],
    )
    .unwrap();
    assert_ne!(f.snapshot(), before);
    let partial = f.snapshot();
    assert_eq!(
        f.complete(import.import_id, claim.lease_token),
        Err(E::ImportIncomplete)
    );
    assert_eq!(
        f.snapshot(),
        partial,
        "an incomplete import stays PROCESSING"
    );

    f.ingest(
        import.import_id,
        claim.lease_token,
        "b2",
        vec![observation("10", day(1))],
        vec![],
    )
    .unwrap();
    let completed = f
        .complete(import.import_id, claim.lease_token)
        .expect("complete");
    assert_eq!(completed.status, MetricImportStatus::Completed);
    assert!(completed.completed_at.is_some());

    // Repeating returns the terminal import unchanged.
    let after = f.snapshot();
    assert_eq!(
        f.complete(import.import_id, claim.lease_token).unwrap(),
        completed
    );
    assert_eq!(f.snapshot(), after);

    // A terminal import accepts no further first-time batch.
    let other = f.begin(claim.lease_token, "report-2", day(2)).unwrap();
    f.ingest(
        other.import_id,
        claim.lease_token,
        "b1",
        vec![NormalizedMetricObservationInput {
            work_doi: "https://doi.org/10.12345/unknown".into(),
            ..observation("1", day(2))
        }],
        vec![],
    )
    .unwrap();
    f.ingest(
        other.import_id,
        claim.lease_token,
        "b2",
        vec![observation("5", day(2))],
        vec![],
    )
    .unwrap();
    assert_eq!(
        f.complete(other.import_id, claim.lease_token)
            .unwrap()
            .status,
        MetricImportStatus::CompletedWithErrors,
        "a rejected row makes the import COMPLETED_WITH_ERRORS"
    );
}

#[test]
fn completion_refuses_unmanaged_failed_and_escaped_state() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();

    f.sql(&format!(
        "UPDATE metric_import SET status = 'FAILED' WHERE import_id = '{}'",
        import.import_id
    ));
    assert_eq!(
        f.complete(import.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );
    f.sql(&format!(
        "UPDATE metric_import SET status = 'PROCESSING' WHERE import_id = '{}'",
        import.import_id
    ));

    // A committed batch outside the expected set is refused, never completed.
    f.sql(&format!("INSERT INTO metric_import_batch (import_id, batch_key, request_hash) VALUES ('{}', 'escaped', 'hash')", import.import_id));
    assert_eq!(
        f.complete(import.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );
    assert_eq!(
        f.import(import.import_id).status,
        MetricImportStatus::Processing
    );
    assert_eq!(
        f.complete(Uuid::new_v4(), claim.lease_token),
        Err(E::ImportNotFound)
    );
}

#[test]
fn concurrent_completions_of_one_import_close_it_exactly_once() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    for key in ["b1", "b2"] {
        f.ingest(
            import.import_id,
            claim.lease_token,
            key,
            vec![observation("10", day(1))],
            vec![],
        )
        .unwrap();
    }
    const WORKERS: usize = 6;
    let barrier = Arc::new(Barrier::new(WORKERS));
    let handles: Vec<_> = (0..WORKERS)
        .map(|_| {
            let pool = Arc::clone(&f.pool);
            let barrier = Arc::clone(&barrier);
            let input = CompleteMetricImportInput {
                import_id: import.import_id,
                lease_token: claim.lease_token,
            };
            thread::spawn(move || {
                barrier.wait();
                complete_metric_import(&pool, &input)
            })
        })
        .collect();
    let results: Vec<MetricImport> = handles
        .into_iter()
        .map(|h| h.join().unwrap().expect("complete"))
        .collect();
    assert!(
        results.windows(2).all(|pair| pair[0] == pair[1]),
        "every caller sees the one terminal state"
    );
    assert_eq!(results[0].status, MetricImportStatus::Completed);
    assert_eq!(
        f.import(import.import_id).completed_at,
        results[0].completed_at
    );
}

// --------------------------------------------------------------------------
// updateMetricSourceCheckpoint
// --------------------------------------------------------------------------

#[test]
fn only_a_completed_import_with_exact_complete_coverage_advances_successful_progress() {
    let (_guard, f) = setup();
    // Each case's unit is day 10 + index; its coverage is built for that day.
    type Coverage = fn(NaiveDate) -> Vec<NormalizedMetricCoverageAssertionInput>;
    let cases: Vec<(&str, Coverage, bool)> = vec![
        ("zero coverage rows", |_| vec![], false),
        ("one exact COMPLETE row", |d| vec![complete_day(d)], true),
        (
            "two exact COMPLETE rows",
            |d| vec![complete_day(d), complete_day(d)],
            true,
        ),
        (
            "PARTIAL",
            |d| {
                vec![coverage(
                    MetricCoverageStatus::Partial,
                    d,
                    d.succ_opt().unwrap(),
                )]
            },
            false,
        ),
        (
            "UNKNOWN",
            |d| {
                vec![coverage(
                    MetricCoverageStatus::Unknown,
                    d,
                    d.succ_opt().unwrap(),
                )]
            },
            false,
        ),
        (
            "COMPLETE and PARTIAL",
            |d| {
                vec![
                    complete_day(d),
                    coverage(MetricCoverageStatus::Partial, d, d.succ_opt().unwrap()),
                ]
            },
            false,
        ),
        (
            "COMPLETE over a longer, mismatched period",
            |d| {
                vec![coverage(
                    MetricCoverageStatus::Complete,
                    d,
                    d.succ_opt().unwrap().succ_opt().unwrap(),
                )]
            },
            false,
        ),
        (
            "exact COMPLETE row plus a mismatched COMPLETE row",
            |d| {
                vec![
                    complete_day(d),
                    coverage(MetricCoverageStatus::Complete, d.pred_opt().unwrap(), d),
                ]
            },
            false,
        ),
    ];
    for (index, (label, assertions, advances)) in cases.into_iter().enumerate() {
        f.sql("DELETE FROM metric_source_checkpoint");
        let claim = f.claim_a();
        let unit_day = day(10 + index as u32);
        let completed = f.run_unit(
            claim.lease_token,
            &format!("report-{index}"),
            unit_day,
            assertions(unit_day),
        );
        assert_eq!(completed.status, MetricImportStatus::Completed, "{label}");
        let checkpoint = f
            .update(completed.import_id, claim.lease_token)
            .unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert_eq!(
            checkpoint.last_completed_at, completed.completed_at,
            "{label}"
        );
        assert_eq!(
            checkpoint.last_successful_period_end,
            advances.then_some(completed.period_end.unwrap()),
            "{label}"
        );
        assert_eq!(
            (checkpoint.lease_owner, checkpoint.lease_expires_at),
            (None, None),
            "{label}: released"
        );
        assert_eq!(checkpoint.last_error, None, "{label}");
        // The cursor advances under exactly the same predicate, in the same
        // write that recorded progress and released the claim.
        let expected_cursor = advances.then(|| cursor_json(&[(unit_day, DIGEST.to_string())]));
        assert_eq!(checkpoint.cursor, expected_cursor, "{label}");
        assert_eq!(f.stored_cursor(f.account_a), expected_cursor, "{label}");
    }
}

#[test]
fn completed_with_errors_releases_the_lease_but_never_claims_success() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let input = BeginMetricImportInput {
        expected_batch_keys: vec!["only".into()],
        ..begin_input(claim.lease_token, "report-1", day(1))
    };
    let import = begin_metric_import(&f.pool, ACTOR, &input).unwrap();
    f.ingest(
        import.import_id,
        claim.lease_token,
        "only",
        vec![NormalizedMetricObservationInput {
            work_doi: "https://doi.org/10.12345/unknown".into(),
            ..observation("1", day(1))
        }],
        vec![complete_day(day(1))],
    )
    .unwrap();
    let completed = f.complete(import.import_id, claim.lease_token).unwrap();
    assert_eq!(completed.status, MetricImportStatus::CompletedWithErrors);
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.last_successful_period_end, None);
    assert_eq!(checkpoint.last_completed_at, completed.completed_at);
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(
        checkpoint.cursor, None,
        "COMPLETED_WITH_ERRORS accepts no manifest"
    );
}

#[test]
fn checkpoint_progress_waits_for_a_terminal_import_and_keeps_the_lease_until_then() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();
    let before = f.snapshot();
    assert_eq!(
        f.update(import.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );
    assert_eq!(
        f.snapshot(),
        before,
        "no progress and no release before completion"
    );
    assert_eq!(
        f.update(Uuid::new_v4(), claim.lease_token),
        Err(E::ImportNotFound)
    );
}

#[test]
fn checkpoint_progress_is_monotonic_and_a_released_repeat_is_read_only() {
    let (_guard, f) = setup();
    // A later day first.
    let claim = f.claim_a();
    let later = f.run_unit(
        claim.lease_token,
        "report-late",
        day(20),
        vec![complete_day(day(20))],
    );
    let after_later = f.update(later.import_id, claim.lease_token).unwrap();
    assert_eq!(after_later.last_successful_period_end, Some(day(21)));

    // Repeating after release returns the recorded checkpoint and writes nothing.
    let before = f.snapshot();
    assert_eq!(
        f.update(later.import_id, claim.lease_token).unwrap(),
        after_later
    );
    assert_eq!(f.snapshot(), before);

    // An earlier day recorded afterwards moves nothing backwards.
    let claim = f.claim_a();
    let earlier = f.run_unit(
        claim.lease_token,
        "report-early",
        day(5),
        vec![complete_day(day(5))],
    );
    f.sql(&format!("UPDATE metric_import SET completed_at = completed_at - interval '1 day' WHERE import_id = '{}'", earlier.import_id));
    let after_earlier = f.update(earlier.import_id, claim.lease_token).unwrap();
    assert_eq!(
        after_earlier.last_successful_period_end,
        Some(day(21)),
        "never backwards"
    );
    assert_eq!(
        after_earlier.last_completed_at, after_later.last_completed_at,
        "never backwards"
    );

    // Once another worker holds the unit, the old token is stale even for a
    // recorded import.
    let newer = f.claim_a();
    let before = f.snapshot();
    assert_eq!(
        f.update(later.import_id, claim.lease_token),
        Err(E::StaleSourceClaim)
    );
    assert_eq!(f.snapshot(), before);
    // A released repeat for an import whose progress was never recorded is stale.
    let claim_free = newer.lease_token;
    let unrecorded = f.run_unit(
        claim_free,
        "report-unrecorded",
        day(25),
        vec![complete_day(day(25))],
    );
    f.sql("UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL");
    assert_eq!(
        f.update(unrecorded.import_id, claim_free),
        Err(E::StaleSourceClaim)
    );
}

// --------------------------------------------------------------------------
// MET-WP2-03: the period-manifest cursor on updateMetricSourceCheckpoint
// --------------------------------------------------------------------------

#[test]
fn a_successful_update_records_its_period_manifest_and_replaces_only_that_period() {
    let (_guard, f) = setup();
    let accepted = |entries: &[(u32, u64)]| {
        Some(cursor_json(
            &entries
                .iter()
                .map(|(unit_day, seed)| (day(*unit_day), digest(*seed)))
                .collect::<Vec<_>>(),
        ))
    };

    // SQL NULL is the only representation of no accepted history; the first
    // successful update turns it into a one-entry cursor derived from the
    // import's own period and stored manifest digest.
    f.claim_a();
    assert_eq!(f.stored_cursor(f.account_a), None);
    reset_leases(&f);
    let (_, _, first) = f.record_success("report-d5", day(5), &digest(5), "10");
    assert_eq!(first.last_successful_period_end, Some(day(6)));
    assert_eq!(f.stored_cursor(f.account_a), accepted(&[(5, 5)]));
    assert_eq!(first.cursor, f.stored_cursor(f.account_a));

    // Another period inserts exactly one entry and keeps period order,
    // whichever order the periods succeed in.
    f.record_success("report-d9", day(9), &digest(9), "10");
    let (_, _, earlier) = f.record_success("report-d2", day(2), &digest(2), "10");
    assert_eq!(
        f.stored_cursor(f.account_a),
        accepted(&[(2, 2), (5, 5), (9, 9)])
    );
    assert_eq!(
        earlier.last_successful_period_end,
        Some(day(10)),
        "an older successful period never moves progress backwards"
    );

    // Reprocessing a period under the same manifest leaves the stored value
    // byte-identical.
    let settled = f.stored_cursor_text(f.account_a);
    f.record_success("report-d5-again", day(5), &digest(5), "10");
    assert_eq!(f.stored_cursor_text(f.account_a), settled);

    // A genuine managed revision of an older period under a changed manifest
    // replaces that period's digest and nothing else.
    let (_, revised, after_revision) =
        f.record_success("report-d5-revised", day(5), &digest(55), "11");
    assert_eq!(
        scalar_i64(
            &f.pool,
            &format!("(SELECT COUNT(*) FROM metric_record_provenance WHERE import_id = '{}' AND classification = 'REVISION')", revised.import_id),
        ),
        1,
        "the reprocessed period carries a real managed revision"
    );
    assert_eq!(
        f.stored_cursor(f.account_a),
        accepted(&[(2, 2), (5, 55), (9, 9)])
    );
    assert_eq!(after_revision.last_successful_period_end, Some(day(10)));
}

#[test]
fn no_update_outside_the_successful_period_predicate_changes_an_accepted_cursor() {
    let (_guard, f) = setup();
    f.record_success("report-accepted", day(1), &digest(1), "10");
    let accepted = f.stored_cursor_text(f.account_a);
    assert!(accepted.is_some());

    // COMPLETED imports whose coverage does not prove the period complete.
    type Coverage = fn(NaiveDate) -> Vec<NormalizedMetricCoverageAssertionInput>;
    let cases: Vec<(&str, Coverage)> = vec![
        ("zero coverage rows", |_| vec![]),
        ("PARTIAL", |d| {
            vec![coverage(
                MetricCoverageStatus::Partial,
                d,
                d.succ_opt().unwrap(),
            )]
        }),
        ("UNKNOWN", |d| {
            vec![coverage(
                MetricCoverageStatus::Unknown,
                d,
                d.succ_opt().unwrap(),
            )]
        }),
        ("COMPLETE and UNKNOWN", |d| {
            vec![
                complete_day(d),
                coverage(MetricCoverageStatus::Unknown, d, d.succ_opt().unwrap()),
            ]
        }),
    ];
    for (index, (label, assertions)) in cases.into_iter().enumerate() {
        let claim = f.claim_a();
        let unit_day = day(10 + index as u32);
        let import = f.run_unit_with(
            claim.lease_token,
            &format!("report-{index}"),
            unit_day,
            &digest(100 + index as u64),
            "10",
            assertions(unit_day),
        );
        assert_eq!(import.status, MetricImportStatus::Completed, "{label}");
        let checkpoint = f
            .update(import.import_id, claim.lease_token)
            .unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert_eq!(checkpoint.lease_owner, None, "{label}: released");
        assert_eq!(checkpoint.last_completed_at, import.completed_at, "{label}");
        assert_eq!(
            checkpoint.last_successful_period_end,
            Some(day(2)),
            "{label}"
        );
        assert_eq!(
            f.stored_cursor_text(f.account_a),
            accepted,
            "{label}: no manifest is accepted"
        );
    }

    // COMPLETED_WITH_ERRORS, even with exact COMPLETE coverage.
    let claim = f.claim_a();
    let input = BeginMetricImportInput {
        expected_batch_keys: vec!["only".into()],
        manifest_digest: digest(200),
        ..begin_input(claim.lease_token, "report-errors", day(20))
    };
    let import = begin_metric_import(&f.pool, ACTOR, &input).unwrap();
    f.ingest(
        import.import_id,
        claim.lease_token,
        "only",
        vec![NormalizedMetricObservationInput {
            work_doi: "https://doi.org/10.12345/unknown".into(),
            ..observation("1", day(20))
        }],
        vec![complete_day(day(20))],
    )
    .unwrap();
    let completed = f.complete(import.import_id, claim.lease_token).unwrap();
    assert_eq!(completed.status, MetricImportStatus::CompletedWithErrors);
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(f.stored_cursor_text(f.account_a), accepted);

    // Non-terminal and FAILED imports keep their existing refusal and mutate
    // nothing.
    let claim = f.claim_a();
    let pending = f
        .begin(claim.lease_token, "report-pending", day(21))
        .unwrap();
    let before = f.snapshot();
    assert_eq!(
        f.update(pending.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );
    assert_eq!(f.snapshot(), before);
    f.sql(&format!(
        "UPDATE metric_import SET status = 'FAILED' WHERE import_id = '{}'",
        pending.import_id
    ));
    let before = f.snapshot();
    assert_eq!(
        f.update(pending.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );
    assert_eq!(f.snapshot(), before);
    assert_eq!(f.stored_cursor_text(f.account_a), accepted);
}

#[test]
fn stale_expired_reclaimed_and_foreign_tokens_never_write_the_cursor() {
    let (_guard, f) = setup();
    let first = f.claim_a();
    let import = f.run_unit_with(
        first.lease_token,
        "report-1",
        day(1),
        &digest(1),
        "10",
        vec![complete_day(day(1))],
    );

    // Expired, not yet reclaimed.
    f.expire(f.account_a);
    let before = f.snapshot();
    assert_eq!(
        f.update(import.import_id, first.lease_token),
        Err(E::StaleSourceClaim)
    );
    assert_eq!(f.snapshot(), before);

    // Reclaimed: the old token and a foreign token are both stale.
    let second = f.claim_a();
    let before = f.snapshot();
    for token in [first.lease_token, Uuid::new_v4()] {
        assert_eq!(f.update(import.import_id, token), Err(E::StaleSourceClaim));
    }
    assert_eq!(f.snapshot(), before);
    assert_eq!(f.stored_cursor(f.account_a), None);

    // Only the live holder records the same terminal import.
    let checkpoint = f.update(import.import_id, second.lease_token).unwrap();
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(1), digest(1))]))
    );
}

#[test]
fn the_sixty_fifth_retained_period_evicts_only_the_oldest_and_a_released_replay_never_resurrects_it(
) {
    let (_guard, f) = setup();
    let period = |offset: u64| {
        date(2026, 1, 1)
            .checked_add_days(Days::new(offset))
            .unwrap()
    };

    // The oldest period is accepted through the real lifecycle.
    let (oldest_token, oldest, _) = f.record_success("report-oldest", period(0), &digest(0), "10");
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(period(0), digest(0))]))
    );
    // Sixty-three later accepted periods fill the storage bound exactly.
    let mut retained: Vec<(NaiveDate, String)> = (0..64)
        .map(|offset| (period(offset), digest(offset)))
        .collect();
    f.store_cursor(f.account_a, &cursor_json(&retained));

    // The sixty-fifth period is accepted through the real lifecycle and evicts
    // the chronologically oldest entry only.
    let (newest_token, newest, recorded) =
        f.record_success("report-newest", period(64), &digest(64), "10");
    retained.remove(0);
    retained.push((period(64), digest(64)));
    assert_eq!(f.stored_cursor(f.account_a), Some(cursor_json(&retained)));
    assert_eq!(recorded.last_successful_period_end, Some(period(65)));

    // A released replay of either recorded import is read-only: the evicted
    // oldest period is not resurrected and nothing is re-encoded.
    let settled = f.snapshot();
    for (import_id, token) in [
        (oldest.import_id, oldest_token),
        (newest.import_id, newest_token),
    ] {
        assert_eq!(f.update(import_id, token).unwrap(), recorded);
        assert_eq!(f.snapshot(), settled);
    }

    // A successful period older than every retained one is inserted and
    // evicted at once: absence means unknown, never unchanged.
    let (_, _, older) = f.record_success("report-older", date(2025, 12, 31), &digest(999), "10");
    assert_eq!(f.stored_cursor(f.account_a), Some(cursor_json(&retained)));
    assert_eq!(older.last_successful_period_end, Some(period(65)));
}

#[test]
fn an_unsupported_cursor_or_import_envelope_fails_the_update_with_no_partial_write() {
    let (_guard, f) = setup();
    let refused = Err(E::Ingestion(Code::InternalStateInconsistency));
    let claim = f.claim_a();
    let successful = f.run_unit_with(
        claim.lease_token,
        "report-1",
        day(1),
        &digest(1),
        "10",
        vec![complete_day(day(1))],
    );

    // An unsupported stored cursor is never read as empty or repaired: no
    // cursor, progress or release write happens.
    for (label, unsupported) in unsupported_cursors() {
        f.store_cursor(f.account_a, &unsupported);
        let before = f.snapshot();
        assert_eq!(
            f.update(successful.import_id, claim.lease_token),
            refused,
            "{label}"
        );
        assert_eq!(f.snapshot(), before, "{label}");
    }
    f.sql(&format!(
        "UPDATE metric_source_checkpoint SET cursor = NULL WHERE source_account_id = '{}'",
        f.account_a
    ));

    // A terminal import whose stored managed envelope does not decode.
    let envelope = f.import(successful.import_id).manifest;
    let set_manifest = |import_id: Uuid, manifest: &JsonValue| {
        f.sql(&format!(
            "UPDATE metric_import SET manifest = '{manifest}'::jsonb WHERE import_id = '{import_id}'"
        ))
    };
    let unsupported_envelopes = [
        ("an empty object", json!({})),
        (
            "an unsupported envelope schema",
            json!({"schemaVersion": "thoth-managed-driver-import/2", "manifestDigest": digest(1), "expectedBatchKeys": ["only"]}),
        ),
        (
            "an uppercase digest",
            json!({"schemaVersion": MANAGED_IMPORT_MANIFEST_SCHEMA, "manifestDigest": digest(0xabcdef).to_uppercase(), "expectedBatchKeys": ["only"]}),
        ),
        (
            "no digest",
            json!({"schemaVersion": MANAGED_IMPORT_MANIFEST_SCHEMA, "expectedBatchKeys": ["only"]}),
        ),
        (
            "an extension member",
            json!({"schemaVersion": MANAGED_IMPORT_MANIFEST_SCHEMA, "manifestDigest": digest(1), "expectedBatchKeys": ["only"], "cursor": {}}),
        ),
    ];
    for (label, manifest) in &unsupported_envelopes {
        set_manifest(successful.import_id, manifest);
        let before = f.snapshot();
        assert_eq!(
            f.update(successful.import_id, claim.lease_token),
            refused,
            "{label}"
        );
        assert_eq!(f.snapshot(), before, "{label}");
    }

    // Restored, the live holder records the import.
    set_manifest(successful.import_id, &envelope);
    f.update(successful.import_id, claim.lease_token).unwrap();
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(1), digest(1))]))
    );

    // A released replay of that import fails closed the same way if its
    // envelope stops decoding, and stays read-only.
    set_manifest(successful.import_id, &json!({}));
    let before = f.snapshot();
    assert_eq!(f.update(successful.import_id, claim.lease_token), refused);
    assert_eq!(f.snapshot(), before);
    set_manifest(successful.import_id, &envelope);

    // An unsuccessful terminal import is held to the same stored-state rules.
    let claim = f.claim_a();
    let unsuccessful = f.run_unit_with(
        claim.lease_token,
        "report-2",
        day(2),
        &digest(2),
        "10",
        vec![],
    );
    f.store_cursor(f.account_a, &json!({}));
    let before = f.snapshot();
    assert_eq!(
        f.update(unsuccessful.import_id, claim.lease_token),
        refused,
        "an unsupported cursor"
    );
    assert_eq!(f.snapshot(), before);
    f.store_cursor(f.account_a, &cursor_json(&[(day(1), digest(1))]));
    set_manifest(unsuccessful.import_id, &json!({}));
    let before = f.snapshot();
    assert_eq!(
        f.update(unsuccessful.import_id, claim.lease_token),
        refused,
        "an unsupported envelope"
    );
    assert_eq!(f.snapshot(), before);

    // A non-terminal import keeps its existing refusal whatever its envelope.
    let pending = f.begin(claim.lease_token, "report-3", day(3)).unwrap();
    set_manifest(pending.import_id, &json!({}));
    assert_eq!(
        f.update(pending.import_id, claim.lease_token),
        Err(E::InvalidImportState)
    );

    // The client-facing failure is the fixed, sanitized coordinator message.
    let error = E::Ingestion(Code::InternalStateInconsistency);
    assert_eq!(error.code(), "INTERNAL_STATE_INCONSISTENCY");
    assert_eq!(error.message(), "The metric ingestion request was refused.");
}

#[test]
fn a_failed_cursor_write_commits_no_progress_and_no_release_either() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    let import = f.run_unit_with(
        claim.lease_token,
        "report-1",
        day(1),
        &digest(1),
        "10",
        vec![complete_day(day(1))],
    );

    // A failure injected into the checkpoint write whenever it would change the
    // cursor. The trigger exists only in this test's disposable database.
    struct Cleanup(Arc<PgPool>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            if let Ok(mut c) = self.0.get() {
                let _ = sql_query(
                    "DROP TRIGGER IF EXISTS wp2_03_test_fail_cursor ON metric_source_checkpoint",
                )
                .execute(&mut c);
                let _ =
                    sql_query("DROP FUNCTION IF EXISTS wp2_03_test_fail_cursor()").execute(&mut c);
            }
        }
    }
    let cleanup = Cleanup(Arc::clone(&f.pool));
    f.sql("CREATE FUNCTION wp2_03_test_fail_cursor() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.cursor IS DISTINCT FROM OLD.cursor THEN RAISE EXCEPTION 'injected cursor write failure'; END IF; RETURN NEW; END $$");
    f.sql("CREATE TRIGGER wp2_03_test_fail_cursor BEFORE UPDATE ON metric_source_checkpoint FOR EACH ROW EXECUTE FUNCTION wp2_03_test_fail_cursor()");

    let before = f.snapshot();
    assert_eq!(
        f.update(import.import_id, claim.lease_token),
        Err(E::Ingestion(Code::InternalDatabaseError))
    );
    assert_eq!(
        f.snapshot(),
        before,
        "cursor, progress and release commit together or not at all"
    );
    assert_eq!(
        f.checkpoint(f.account_a).lease_owner,
        Some(claim.lease_token.to_string())
    );

    // Without the injected failure the same live update records all three.
    drop(cleanup);
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(1), digest(1))]))
    );
}

// --------------------------------------------------------------------------
// Crash and retry at every lifecycle boundary
// --------------------------------------------------------------------------

#[test]
fn every_lifecycle_crash_boundary_recovers_without_a_duplicate_durable_effect() {
    let (_guard, f) = setup();

    // Crash after the claim, before begin: expiry permits a reclaim and the
    // unit is begun once.
    let lost = f.claim_a();
    f.expire(f.account_a);
    let claim = f.claim_a();
    assert_ne!(claim.lease_token, lost.lease_token);
    let import = f.begin(claim.lease_token, "report-1", day(1)).unwrap();

    // Timeout after begin: the same request recovers the same import.
    assert_eq!(
        f.begin(claim.lease_token, "report-1", day(1))
            .unwrap()
            .import_id,
        import.import_id
    );
    assert_eq!(f.count("metric_import", "TRUE"), 1);

    // Timeout after one committed batch: the replay writes nothing more.
    let first = f
        .ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("10", day(1))],
            vec![complete_day(day(1))],
        )
        .unwrap();
    let replay = f
        .ingest(
            import.import_id,
            claim.lease_token,
            "b1",
            vec![observation("10", day(1))],
            vec![complete_day(day(1))],
        )
        .unwrap();
    assert!(replay.replayed && replay.import_batch_id == first.import_batch_id);
    f.ingest(
        import.import_id,
        claim.lease_token,
        "b2",
        vec![],
        vec![complete_day(day(1))],
    )
    .unwrap();

    // Crash after the last batch, before completion: a reclaimer completes the
    // same import.
    f.expire(f.account_a);
    let claim = f.claim_a();
    let batches_before = f.count("metric_import_batch", "TRUE");
    let completed = f.complete(import.import_id, claim.lease_token).unwrap();
    assert_eq!(completed.status, MetricImportStatus::Completed);

    // Crash after completion, before the checkpoint: a reclaimer finds the
    // same terminal import through begin, completion is idempotent, and the
    // checkpoint advances under the new token.
    f.expire(f.account_a);
    let claim = f.claim_a();
    assert_eq!(
        f.begin(claim.lease_token, "report-1", day(1)).unwrap(),
        completed
    );
    assert_eq!(
        f.complete(import.import_id, claim.lease_token).unwrap(),
        completed
    );
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));

    // Timeout after the checkpoint update: the repeat is read-only.
    let settled = f.snapshot();
    assert_eq!(
        f.update(import.import_id, claim.lease_token).unwrap(),
        checkpoint
    );
    assert_eq!(f.snapshot(), settled);

    // No step produced a second canonical effect.
    assert_eq!(f.count("metric_import", "TRUE"), 1);
    assert_eq!(f.count("metric_import_batch", "TRUE"), batches_before);
    assert_eq!(f.count("metric_record", "TRUE"), 1);
    assert_eq!(f.count("metric_rollup_delta", "TRUE"), 1);
    assert_eq!(f.count("metric_coverage", "TRUE"), 2);
    assert_eq!(f.import(import.import_id).received_count, 1);
}

#[test]
fn a_failed_lifecycle_transaction_leaves_no_partial_state() {
    let (_guard, f) = setup();
    let claim = f.claim_a();
    // Make the discovery write fail after the import insert, inside begin's
    // single transaction. The failing trigger exists only in this test.
    struct Cleanup(Arc<PgPool>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            if let Ok(mut c) = self.0.get() {
                let _ = sql_query(
                    "DROP TRIGGER IF EXISTS wp2_02_test_fail ON metric_source_checkpoint",
                )
                .execute(&mut c);
                let _ = sql_query("DROP FUNCTION IF EXISTS wp2_02_test_fail()").execute(&mut c);
            }
        }
    }
    let _cleanup = Cleanup(Arc::clone(&f.pool));
    f.sql("CREATE FUNCTION wp2_02_test_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.last_discovered_at IS DISTINCT FROM OLD.last_discovered_at THEN RAISE EXCEPTION 'injected secret-looking failure: password=hunter2'; END IF; RETURN NEW; END $$");
    f.sql("CREATE TRIGGER wp2_02_test_fail BEFORE UPDATE ON metric_source_checkpoint FOR EACH ROW EXECUTE FUNCTION wp2_02_test_fail()");
    let before = f.snapshot();
    let error = f.begin(claim.lease_token, "report-1", day(1)).unwrap_err();
    assert_eq!(error, E::Ingestion(Code::InternalDatabaseError));
    assert_eq!(error.message(), "The metric ingestion request was refused.");
    assert_eq!(
        f.snapshot(),
        before,
        "the import insert rolled back with the failed write"
    );
}

// --------------------------------------------------------------------------
// CR-1: claim-time canonical authority races (#908 review 5654942925)
// --------------------------------------------------------------------------
//
// Each race is driven by PostgreSQL lock coordination, never by a sleep. A
// claim is frozen at an exact statement by a test-only pausing trigger that
// waits on a transaction-level advisory lock the test holds session-level, and
// a writer is proven to be queued by reading `pg_stat_activity` for its pinned
// backend. Every writer below is the repository's real canonical writer for
// that authority: the #904 source and source-account coordinators, the
// MET-WP1-12 platform coordinator and the BE-01 service-configuration
// coordinator, which is the only production write path for
// `publisher.subscription_package` (`PatchPublisher` has no package field).

/// The advisory key the CR-1 pause point blocks on. Distinct from the #900
/// coordinator tests' key so the two suites never share a hold.
const CR1_PAUSE_KEY: i64 = 987_654_321_202;
const CR1_ADMIN: &str = "cr1-superuser";

/// A deterministic pause point inside one claim transaction.
///
/// `BEFORE INSERT` on `metric_source_checkpoint` freezes the claim after its
/// unlocked eligibility enumeration and before its checkpoint lock (the
/// bootstrap insert fires the trigger for an existing row too, before
/// `ON CONFLICT DO NOTHING` discards it). `BEFORE UPDATE` freezes it at the
/// first lease write. The claim resumes only on `release`. The trigger, its
/// function and the session-level hold exist only in the disposable test
/// database and are removed on drop.
struct ClaimPause {
    pool: Arc<PgPool>,
    hold: PgConnection,
    name: String,
}

impl ClaimPause {
    fn before(pool: &Arc<PgPool>, event: &str) -> Self {
        let mut hold = PgConnection::establish(&test_db_url()).expect("pause session");
        sql_query(format!("SELECT pg_advisory_lock({CR1_PAUSE_KEY})"))
            .execute(&mut hold)
            .expect("session-level hold");
        let name = format!("wp2_02_cr1_pause_{}", Uuid::new_v4().simple());
        exec(pool, &format!("CREATE FUNCTION {name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock({CR1_PAUSE_KEY}); RETURN NEW; END $$"));
        exec(pool, &format!("CREATE TRIGGER {name} BEFORE {event} ON metric_source_checkpoint FOR EACH ROW EXECUTE FUNCTION {name}()"));
        ClaimPause {
            pool: Arc::clone(pool),
            hold,
            name,
        }
    }

    fn release(&mut self) {
        sql_query(format!("SELECT pg_advisory_unlock({CR1_PAUSE_KEY})"))
            .execute(&mut self.hold)
            .expect("release the hold");
    }
}

impl Drop for ClaimPause {
    fn drop(&mut self) {
        // Release the hold first. On unwind the claim may still be frozen on
        // it while holding the table lock the `DROP`s below need, so dropping
        // first would wait on a claim that can never resume.
        let _ = sql_query("SELECT pg_advisory_unlock_all()").execute(&mut self.hold);
        if let Ok(mut connection) = self.pool.get() {
            let _ = sql_query(format!(
                "DROP TRIGGER IF EXISTS {} ON metric_source_checkpoint",
                self.name
            ))
            .execute(&mut connection);
            let _ = sql_query(format!("DROP FUNCTION IF EXISTS {}()", self.name))
                .execute(&mut connection);
        }
    }
}

fn backend_pid(connection: &mut PgConnection) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
        "pg_backend_pid()",
    ))
    .get_result::<i32>(connection)
    .expect("pg_backend_pid") as i64
}

/// A one-connection pool, so the backend pid a coordinator will run on is
/// known before it runs.
fn pinned_pool() -> (Arc<PgPool>, i64) {
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(1)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to build a pinned pool"),
    );
    let pid = backend_pid(&mut pool.get().unwrap());
    (pool, pid)
}

/// Whether backend `pid` is waiting on a heavyweight lock, optionally of one
/// `wait_event` kind.
fn is_lock_waiting(f: &Fixture, pid: i64, wait_event: Option<&str>) -> bool {
    let event = wait_event
        .map(|event| format!(" AND wait_event = '{event}'"))
        .unwrap_or_default();
    scalar_i64(
        &f.pool,
        &format!("(SELECT COUNT(*) FROM pg_stat_activity WHERE pid = {pid} AND wait_event_type = 'Lock'{event})"),
    ) == 1
}

/// Block until the claim on backend `pid` is frozen at the pause point. The
/// deadline is a hang guard only; nothing is decided by elapsed time.
fn wait_until_paused(f: &Fixture, pid: i64) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !is_lock_waiting(f, pid, Some("advisory")) {
        assert!(
            Instant::now() < deadline,
            "the claim never reached its pause point"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WriterOutcome {
    /// The writer is queued on a row lock the claim holds.
    Blocked,
    /// The writer committed while the claim was still inside its transaction.
    Finished,
}

/// Which of the two outcomes the writer on backend `pid` reaches first.
fn blocked_or_finished<T>(f: &Fixture, pid: i64, writer: &thread::JoinHandle<T>) -> WriterOutcome {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if writer.is_finished() {
            return WriterOutcome::Finished;
        }
        if is_lock_waiting(f, pid, None) {
            return WriterOutcome::Blocked;
        }
        assert!(
            Instant::now() < deadline,
            "the writer neither blocked nor finished"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

/// Run one full-limit claim on its own pinned connection, frozen at `event`,
/// execute `during` while it is frozen, then let it finish.
fn claim_while_paused<R>(
    f: &Fixture,
    event: &str,
    during: impl FnOnce(&Fixture) -> R,
) -> (Result<Vec<MetricSourceUnitClaim>, E>, R) {
    let mut pause = ClaimPause::before(&f.pool, event);
    let (pool, pid) = pinned_pool();
    let claim =
        thread::spawn(move || claim_metric_source_units(&pool, &claim_input(Some(10), None)));
    wait_until_paused(f, pid);
    let during_result = during(f);
    pause.release();
    let result = claim.join().expect("claim thread");
    drop(pause);
    (result, during_result)
}

/// Start `writer` on a pinned connection while the claim is frozen at its
/// first lease write, and report whether it queues behind the claim or
/// commits inside the claim's window.
fn writer_against_frozen_lease_write(
    f: &Fixture,
    writer: impl FnOnce(&PgPool) + Send + 'static,
) -> (Result<Vec<MetricSourceUnitClaim>, E>, WriterOutcome) {
    let (claims, (handle, outcome)) = claim_while_paused(f, "UPDATE", |f| {
        let (pool, pid) = pinned_pool();
        let handle = thread::spawn(move || writer(&pool));
        let outcome = blocked_or_finished(f, pid, &handle);
        (handle, outcome)
    });
    handle.join().expect("writer thread");
    (claims, outcome)
}

fn account_codes(claims: &[MetricSourceUnitClaim]) -> Vec<String> {
    claims
        .iter()
        .map(|claim| claim.source_account.code.clone())
        .collect()
}

fn reset_leases(f: &Fixture) {
    f.sql("UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL");
}

fn account_row(f: &Fixture, account: Uuid) -> MetricSourceAccount {
    use diesel::QueryDsl;
    let mut connection = f.pool.get().unwrap();
    crate::schema::metric_source_account::table
        .find(account)
        .first(&mut connection)
        .expect("account row")
}

fn account_a_patch(bucket: &str, enabled: bool) -> PatchMetricSourceAccount {
    PatchMetricSourceAccount {
        code: ACCOUNT_A.into(),
        configuration: MetricSourceAccountConfigurationInput {
            kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
            cloudfront_legacy_s3: Some(MetricCloudFrontLegacyS3ConfigurationInput {
                hostname: "dist-a".into(),
                bucket: bucket.into(),
                prefix: "cf/".into(),
            }),
        },
        enabled,
    }
}

/// #904 source-account writer: `FOR UPDATE` on the account row.
fn disable_account_a(pool: &PgPool) {
    update_metric_source_account(pool, CR1_ADMIN, &account_a_patch("logs", false))
        .expect("the #904 account writer must commit");
}

/// #904 source-account writer: replace the configuration, keeping A enabled.
fn replace_account_a_bucket(pool: &PgPool, bucket: &str) {
    update_metric_source_account(pool, CR1_ADMIN, &account_a_patch(bucket, true))
        .expect("the #904 account writer must commit");
}

/// #904 source writer: `FOR UPDATE` on the source row.
fn disable_source(pool: &PgPool) {
    update_metric_source(
        pool,
        CR1_ADMIN,
        &PatchMetricSource {
            code: SOURCE_CODE.into(),
            enabled: false,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        },
    )
    .expect("the #904 source writer must commit");
}

/// MET-WP1-12 platform writer: `FOR UPDATE` on the platform row.
fn disable_platform(pool: &PgPool) {
    update_metric_platform(
        pool,
        CR1_ADMIN,
        &PatchMetricPlatform {
            code: "cf".into(),
            display_name: "CloudFront".into(),
            enabled: false,
            public_description: None,
        },
    )
    .expect("the platform writer must commit");
}

/// BE-01 service-configuration writer, the only production path that changes
/// `publisher.subscription_package`: `FOR UPDATE` on the publisher row first.
fn revoke_metrics_collect(pool: &PgPool, publisher_id: Uuid) {
    use crate::model::Crud;
    let token = Publisher::from_id(pool, &publisher_id)
        .expect("publisher row")
        .service_configuration_updated_at;
    replace_publisher_service_configuration(
        pool,
        &ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::SuperuserApi,
            actor: CR1_ADMIN,
            job_creation: DistributionJobCreation::Off,
        },
        &ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: ThothPackage::Oasis,
            enabled_distribution_platforms: vec![],
            expected_updated_at: token,
        },
    )
    .expect("the BE-01 package writer must commit");
}

#[test]
fn cr1_source_account_authority_cannot_go_stale_between_eligibility_and_lease_grant() {
    let (_guard, f) = setup();

    // Direction 1: the account writer commits inside the claim's window. The
    // lease for A may not be granted from the authority read before it.
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| disable_account_a(&f.pool));
    let claims = claims.expect("claim");
    assert_eq!(
        account_codes(&claims),
        vec![ACCOUNT_B.to_string()],
        "account A was disabled before its lease could be granted"
    );
    assert_eq!(
        f.checkpoint(f.account_a).lease_owner,
        None,
        "A's bootstrapped checkpoint stays unleased"
    );
    assert!(!account_row(&f, f.account_a).enabled);

    // A configuration replacement committed inside the window is what the
    // fresh claim must carry: never the pre-replacement configuration.
    reset_leases(&f);
    f.sql("UPDATE metric_source_account SET enabled = TRUE WHERE code = 'acct-a'");
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| {
        replace_account_a_bucket(&f.pool, "logs-replaced")
    });
    let claims = claims.expect("claim");
    let a = claims
        .iter()
        .find(|claim| claim.source_account.code == ACCOUNT_A)
        .expect("A stays eligible after a configuration replacement");
    assert_eq!(
        a.source_account.configuration,
        serde_json::json!({
            "schemaVersion": "cloudfront-source-account/1",
            "hostname": "dist-a",
            "logging": {"mode": "LEGACY_S3", "bucket": "logs-replaced", "prefix": "cf/"},
        }),
        "the claim carries the configuration committed before the lease grant"
    );
    assert_eq!(a.source_account, account_row(&f, f.account_a));

    // Direction 2: the claim holds A's authority first, so the writer queues
    // behind the claim's commit and the lease is granted from valid state.
    reset_leases(&f);
    let (claims, outcome) = writer_against_frozen_lease_write(&f, disable_account_a);
    assert_eq!(
        outcome,
        WriterOutcome::Blocked,
        "the account writer must wait for the claim's authority lock"
    );
    let claims = claims.expect("claim");
    let a = claims
        .iter()
        .find(|claim| claim.source_account.code == ACCOUNT_A)
        .expect("A was eligible for the whole claim transaction");
    assert!(a.source_account.enabled);
    assert_eq!(
        f.checkpoint(f.account_a).lease_owner,
        Some(a.lease_token.to_string())
    );
    assert!(
        !account_row(&f, f.account_a).enabled,
        "the disable committed after the claim"
    );
    // The post-grant change is caught by begin's existing revalidation.
    assert_eq!(
        f.begin(a.lease_token, "report-1", day(1)),
        Err(E::SourceNotEligible)
    );
}

#[test]
fn cr1_source_authority_cannot_go_stale_between_eligibility_and_lease_grant() {
    let (_guard, f) = setup();

    // Direction 1: the source writer commits inside the window. The claim
    // fails closed for the whole source and leases nothing.
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| disable_source(&f.pool));
    assert_eq!(
        claims,
        Err(E::SourceNotEligible),
        "a source disabled before the lease grant yields no fresh claim"
    );
    assert_eq!(
        f.count("metric_source_checkpoint", "lease_owner IS NOT NULL"),
        0
    );

    // Direction 2: the claim holds the source row first.
    f.sql("UPDATE metric_source SET enabled = TRUE");
    reset_leases(&f);
    let (claims, outcome) = writer_against_frozen_lease_write(&f, disable_source);
    assert_eq!(outcome, WriterOutcome::Blocked);
    let claims = claims.expect("claim");
    assert_eq!(
        account_codes(&claims),
        vec![ACCOUNT_A.to_string(), ACCOUNT_B.to_string()]
    );
    assert!(claims.iter().all(|claim| claim.source.enabled));
    assert_eq!(f.count("metric_source", "enabled = FALSE"), 1);
    assert_eq!(
        f.checkpoint(f.account_a).lease_owner,
        Some(claims[0].lease_token.to_string())
    );
    assert_eq!(
        f.begin(claims[0].lease_token, "report-1", day(1)),
        Err(E::SourceNotEligible)
    );
}

#[test]
fn cr1_platform_authority_cannot_go_stale_between_eligibility_and_lease_grant() {
    let (_guard, f) = setup();

    // Direction 1: the platform is disabled inside the window; every account
    // on it is skipped and no lease is written.
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| disable_platform(&f.pool));
    assert_eq!(
        claims.expect("claim"),
        vec![],
        "a platform disabled before the lease grant yields no fresh claim"
    );
    assert_eq!(
        f.count("metric_source_checkpoint", "lease_owner IS NOT NULL"),
        0
    );

    // Direction 2: the claim holds the platform row first.
    f.sql("UPDATE metric_platform SET enabled = TRUE");
    reset_leases(&f);
    let (claims, outcome) = writer_against_frozen_lease_write(&f, disable_platform);
    assert_eq!(outcome, WriterOutcome::Blocked);
    let claims = claims.expect("claim");
    assert_eq!(
        account_codes(&claims),
        vec![ACCOUNT_A.to_string(), ACCOUNT_B.to_string()]
    );
    // MET-WP2-03: each claim's platform code is the one it copied from the
    // platform row it held locked and enabled while the writer waited.
    assert!(claims.iter().all(|claim| claim.platform_code == "cf"));
    assert_eq!(f.count("metric_platform", "enabled = FALSE"), 1);
    assert_eq!(
        f.begin(claims[0].lease_token, "report-1", day(1)),
        Err(E::SourceNotEligible)
    );
}

#[test]
fn cr1_publisher_capability_cannot_go_stale_between_eligibility_and_lease_grant() {
    let (_guard, f) = setup();
    let publisher_id = f.publisher_id;

    // Direction 1: METRICS_COLLECT is revoked inside the window (OBELISK ->
    // OASIS through the BE-01 coordinator); nothing is leased.
    let (claims, ()) = claim_while_paused(&f, "INSERT", |f| {
        revoke_metrics_collect(&f.pool, publisher_id)
    });
    assert_eq!(
        claims.expect("claim"),
        vec![],
        "a publisher that lost METRICS_COLLECT before the lease grant yields no fresh claim"
    );
    assert_eq!(
        f.count("metric_source_checkpoint", "lease_owner IS NOT NULL"),
        0
    );

    // Direction 2: the claim holds the publisher row first.
    f.sql("UPDATE publisher SET subscription_package = 'OBELISK'");
    reset_leases(&f);
    let (claims, outcome) = writer_against_frozen_lease_write(&f, move |pool| {
        revoke_metrics_collect(pool, publisher_id)
    });
    assert_eq!(outcome, WriterOutcome::Blocked);
    let claims = claims.expect("claim");
    assert_eq!(
        account_codes(&claims),
        vec![ACCOUNT_A.to_string(), ACCOUNT_B.to_string()]
    );
    assert_eq!(
        f.count("publisher", "subscription_package = 'OASIS'"),
        1,
        "the revocation committed after the claim"
    );
    assert_eq!(
        f.begin(claims[0].lease_token, "report-1", day(1)),
        Err(E::MetricsCollectNotEntitled)
    );
}

/// Whether a cleanup `DROP` of pause `name` is queued on a heavyweight lock.
fn pause_cleanup_is_lock_waiting(f: &Fixture, name: &str) -> bool {
    scalar_i64(
        &f.pool,
        &format!("(SELECT COUNT(*) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND wait_event_type = 'Lock' AND query LIKE 'DROP % {name}%')"),
    ) > 0
}

/// Whether backend `pid` still holds a session-level advisory lock.
fn holds_advisory_lock(f: &Fixture, pid: i64) -> bool {
    scalar_i64(
        &f.pool,
        &format!("(SELECT COUNT(*) FROM pg_locks WHERE locktype = 'advisory' AND granted AND pid = {pid})"),
    ) > 0
}

#[test]
fn cr1_pause_cleanup_on_unwind_releases_its_hold_before_dropping_the_trigger() {
    let (_guard, f) = setup();
    let mut pause = ClaimPause::before(&f.pool, "INSERT");
    let name = pause.name.clone();
    let hold_pid = backend_pid(&mut pause.hold);
    let (pool, pid) = pinned_pool();
    let claim =
        thread::spawn(move || claim_metric_source_units(&pool, &claim_input(Some(10), None)));
    wait_until_paused(&f, pid);

    // A failure while the claim is frozen unwinds through the pause without
    // `release`, so its `Drop` is the only cleanup that runs.
    let unwind = thread::spawn(move || {
        let _pause = pause;
        panic!("simulated failure while the claim is paused");
    });

    // The frozen claim holds a table lock the cleanup `DROP` needs, and waits on
    // the pause hold. The queued `DROP` is read before the hold, so a hold still
    // granted afterwards was retained while cleanup DDL was already waiting.
    let deadline = Instant::now() + Duration::from_secs(20);
    let retained_hold_during_cleanup = loop {
        if unwind.is_finished() {
            break false;
        }
        if pause_cleanup_is_lock_waiting(&f, &name) && holds_advisory_lock(&f, hold_pid) {
            break true;
        }
        assert!(
            Instant::now() < deadline,
            "the pause cleanup neither finished nor queued"
        );
        thread::sleep(Duration::from_millis(10));
    };
    if retained_hold_during_cleanup {
        // Break the cycle so the defect reports as a failure rather than a hang.
        scalar_i64(
            &f.pool,
            &format!(
                "(SELECT COUNT(*) FROM (SELECT pg_terminate_backend({hold_pid})) AS terminated)"
            ),
        );
    }
    assert!(
        unwind.join().is_err(),
        "the simulated failure unwound through the pause"
    );
    let claims = claim.join().expect("claim thread");
    assert!(
        !retained_hold_during_cleanup,
        "pause cleanup attempted DDL while retaining its own session advisory hold"
    );
    assert_eq!(
        account_codes(&claims.expect("claim")),
        vec![ACCOUNT_A.to_string(), ACCOUNT_B.to_string()],
        "the released claim completes"
    );
    assert_eq!(f.count("pg_trigger", &format!("tgname = '{name}'")), 0);
    assert_eq!(f.count("pg_proc", &format!("proname = '{name}'")), 0);
}

// --------------------------------------------------------------------------
// MET-WP2-03: a real current-versus-stale checkpoint update race
// --------------------------------------------------------------------------

/// A session holding one account's checkpoint row `FOR UPDATE` in an open
/// transaction, so later updates queue on the canonical checkpoint lock in an
/// order the test establishes. Dropping it unreleased closes the session,
/// which releases the lock.
struct CheckpointRowHold {
    connection: PgConnection,
}

impl CheckpointRowHold {
    fn acquire(account: Uuid) -> Self {
        let mut connection = PgConnection::establish(&test_db_url()).expect("hold session");
        sql_query("BEGIN")
            .execute(&mut connection)
            .expect("open the hold");
        sql_query(format!(
            "SELECT 1 FROM metric_source_checkpoint WHERE source_account_id = '{account}' FOR UPDATE"
        ))
        .execute(&mut connection)
        .expect("lock the checkpoint row");
        CheckpointRowHold { connection }
    }

    fn release(mut self) {
        sql_query("COMMIT")
            .execute(&mut self.connection)
            .expect("release the hold");
    }
}

/// Start one checkpoint update on its own pinned connection and return once
/// PostgreSQL reports it queued on a heavyweight lock. The deadline is a hang
/// guard only; nothing is decided by elapsed time.
fn queued_update(
    f: &Fixture,
    import_id: Uuid,
    lease_token: Uuid,
) -> thread::JoinHandle<Result<MetricSourceCheckpoint, E>> {
    let (pool, pid) = pinned_pool();
    let handle = thread::spawn(move || {
        update_metric_source_checkpoint(
            &pool,
            &UpdateMetricSourceCheckpointInput {
                import_id,
                lease_token,
            },
        )
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    while !is_lock_waiting(f, pid, None) {
        assert!(
            !handle.is_finished(),
            "the update finished without queuing on the checkpoint lock"
        );
        assert!(
            Instant::now() < deadline,
            "the update never queued on the checkpoint lock"
        );
        thread::sleep(Duration::from_millis(10));
    }
    handle
}

#[test]
fn a_current_and_a_stale_checkpoint_update_race_to_one_durable_outcome() {
    let (_guard, f) = setup();
    let mut accepted: Vec<(NaiveDate, String)> = Vec::new();
    for (round, stale_first) in [(0u32, true), (10, false)] {
        // A unit completed under a lease that then expired and was reclaimed,
        // and a unit completed under the reclaim.
        let stale = f.claim_a();
        let stale_import = f.run_unit_with(
            stale.lease_token,
            &format!("stale-{round}"),
            day(1 + round),
            &digest(u64::from(1 + round)),
            "10",
            vec![complete_day(day(1 + round))],
        );
        f.expire(f.account_a);
        let current = f.claim_a();
        let current_import = f.run_unit_with(
            current.lease_token,
            &format!("current-{round}"),
            day(2 + round),
            &digest(u64::from(2 + round)),
            "10",
            vec![complete_day(day(2 + round))],
        );

        // Both updates queue on the one canonical checkpoint row lock.
        let hold = CheckpointRowHold::acquire(f.account_a);
        let (first, second) = if stale_first {
            let first = queued_update(&f, stale_import.import_id, stale.lease_token);
            (
                first,
                queued_update(&f, current_import.import_id, current.lease_token),
            )
        } else {
            let first = queued_update(&f, current_import.import_id, current.lease_token);
            (
                first,
                queued_update(&f, stale_import.import_id, stale.lease_token),
            )
        };
        hold.release();
        let first = first.join().expect("first update thread");
        let second = second.join().expect("second update thread");
        let (stale_result, current_result) = if stale_first {
            (first, second)
        } else {
            (second, first)
        };

        // The live holder records its import whichever update the lock
        // admitted first.
        let recorded = current_result.expect("the live holder records its import");
        assert_eq!(recorded.lease_owner, None);
        assert_eq!(recorded.last_successful_period_end, Some(day(3 + round)));
        // The stale token writes nothing: it is refused while the live claim
        // is held, and answered read-only once that claim has been released.
        if stale_first {
            assert_eq!(stale_result, Err(E::StaleSourceClaim));
        } else {
            assert_eq!(stale_result, Ok(recorded.clone()));
        }

        // Final durable state: only the live holder's manifest was accepted,
        // and nothing was written after its update.
        accepted.push((day(2 + round), digest(u64::from(2 + round))));
        assert_eq!(f.stored_cursor(f.account_a), Some(cursor_json(&accepted)));
        assert_eq!(f.checkpoint(f.account_a), recorded);
        assert_eq!(
            recorded.last_completed_at,
            f.import(current_import.import_id).completed_at
        );
    }
}

// --------------------------------------------------------------------------
// MET-WP7-PREREQ-02: CloudFront quarantine-only manifest acceptance
// --------------------------------------------------------------------------

/// A syntactically valid DOI no work carries.
const UNRESOLVED_DOI: &str = "https://doi.org/10.12345/unresolved-a";
const SECOND_UNRESOLVED_DOI: &str = "10.12345/Unresolved-B";

/// An observation of `start` eligible for quarantine: an unresolved DOI and
/// none of the five quarantine-excluded optional fields.
fn unresolved(doi: &str, start: NaiveDate) -> NormalizedMetricObservationInput {
    NormalizedMetricObservationInput {
        work_doi: doi.into(),
        source_record_id: None,
        source_row_number: None,
        ..observation("1", start)
    }
}

impl Fixture {
    /// Run one single-batch unit of `observations` and `coverage` for `start`
    /// under `token`, returning the terminal import.
    fn run_batch_unit(
        &self,
        token: Uuid,
        upstream: &str,
        start: NaiveDate,
        manifest_digest: &str,
        observations: Vec<NormalizedMetricObservationInput>,
        coverage: Vec<NormalizedMetricCoverageAssertionInput>,
    ) -> MetricImport {
        let input = BeginMetricImportInput {
            expected_batch_keys: vec!["only".into()],
            manifest_digest: manifest_digest.into(),
            ..begin_input(token, upstream, start)
        };
        let import = begin_metric_import(&self.pool, ACTOR, &input).expect("begin");
        self.ingest(import.import_id, token, "only", observations, coverage)
            .expect("batch");
        self.complete(import.import_id, token).expect("complete")
    }

    /// Quarantine rows linked to provenance of `import_id`.
    fn quarantined(&self, import_id: Uuid) -> i64 {
        self.count(
            "metric_identifier_quarantine q JOIN metric_record_provenance p \
             ON p.record_provenance_id = q.record_provenance_id",
            &format!("p.import_id = '{import_id}'"),
        )
    }

    /// Claim account A and run one eligible quarantine-only unit for `start`.
    fn quarantine_only_unit(
        &self,
        upstream: &str,
        start: NaiveDate,
        manifest_digest: &str,
    ) -> (Uuid, MetricImport) {
        let claim = self.claim_a();
        let import = self.run_batch_unit(
            claim.lease_token,
            upstream,
            start,
            manifest_digest,
            vec![
                unresolved(UNRESOLVED_DOI, start),
                unresolved(SECOND_UNRESOLVED_DOI, start),
            ],
            vec![complete_day(start)],
        );
        assert_eq!(import.status, MetricImportStatus::CompletedWithErrors);
        assert_eq!((import.invalid_count, import.conflict_count), (2, 0));
        assert_eq!(self.quarantined(import.import_id), 2);
        (claim.lease_token, import)
    }
}

#[test]
fn an_eligible_quarantine_only_import_records_its_manifest_but_never_claims_success() {
    let (_guard, f) = setup();

    // On a fresh checkpoint: the manifest is accepted, success is not.
    let (token, import) = f.quarantine_only_unit("report-q5", day(5), &digest(5));
    let checkpoint = f.update(import.import_id, token).unwrap();
    assert_eq!(
        f.import(import.import_id).status,
        MetricImportStatus::CompletedWithErrors,
        "quarantine is not canonical acceptance"
    );
    assert_eq!(checkpoint.last_completed_at, import.completed_at);
    assert_eq!(checkpoint.last_successful_period_end, None);
    assert_eq!(
        (checkpoint.lease_owner.clone(), checkpoint.lease_expires_at),
        (None, None),
        "released"
    );
    assert_eq!(checkpoint.last_error, None);
    let expected = Some(cursor_json(&[(day(5), digest(5))]));
    assert_eq!(f.stored_cursor(f.account_a), expected);
    assert_eq!(checkpoint.cursor, expected);
    assert_eq!(
        f.count("metric_record", "TRUE"),
        0,
        "quarantine creates no canonical record"
    );

    // A released replay is read-only and recognizes the recorded completion
    // from last_completed_at alone.
    let settled = f.snapshot();
    assert_eq!(f.update(import.import_id, token).unwrap(), checkpoint);
    assert_eq!(f.snapshot(), settled);

    // Alongside canonical success: a later clean day advances the successful
    // period, and a later quarantine-only day adds its manifest without moving
    // the successful period.
    f.record_success("report-d7", day(7), &digest(7), "10");
    let (token, later) = f.quarantine_only_unit("report-q9", day(9), &digest(9));
    let checkpoint = f.update(later.import_id, token).unwrap();
    assert_eq!(checkpoint.last_successful_period_end, Some(day(8)));
    assert_eq!(checkpoint.last_completed_at, later.completed_at);
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[
            (day(5), digest(5)),
            (day(7), digest(7)),
            (day(9), digest(9))
        ]))
    );
}

#[test]
fn a_quarantine_only_manifest_replaces_its_period_and_keeps_the_retention_bound() {
    let (_guard, f) = setup();
    // A period accepted by a clean import is replaced by a quarantine-only
    // reprocessing of the same period under a changed manifest.
    f.record_success("report-d3", day(3), &digest(3), "10");
    let (token, reprocessed) = f.quarantine_only_unit("report-d3-q", day(3), &digest(33));
    let checkpoint = f.update(reprocessed.import_id, token).unwrap();
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(3), digest(33))]))
    );
    assert_eq!(checkpoint.last_successful_period_end, Some(day(4)));

    // With 64 retained periods, a newer quarantine-only period evicts exactly
    // the chronologically oldest entry.
    let period = |offset: u64| {
        date(2026, 1, 1)
            .checked_add_days(Days::new(offset))
            .unwrap()
    };
    let mut retained: Vec<(NaiveDate, String)> = (0..64)
        .map(|offset| (period(offset), digest(offset)))
        .collect();
    f.store_cursor(f.account_a, &cursor_json(&retained));
    let (token, newest) = f.quarantine_only_unit("report-newest-q", period(64), &digest(64));
    let recorded = f.update(newest.import_id, token).unwrap();
    retained.remove(0);
    retained.push((period(64), digest(64)));
    assert_eq!(f.stored_cursor(f.account_a), Some(cursor_json(&retained)));
    assert_eq!(recorded.last_successful_period_end, Some(day(4)));

    // Released replays of either quarantine-only import never resurrect or
    // re-encode anything.
    let settled = f.snapshot();
    for import_id in [reprocessed.import_id, newest.import_id] {
        assert_eq!(f.update(import_id, token).unwrap(), recorded);
        assert_eq!(f.snapshot(), settled);
    }
}

#[test]
fn a_consistent_but_ineligible_completed_with_errors_import_releases_without_a_manifest() {
    let (_guard, f) = setup();
    f.record_success("report-accepted", day(1), &digest(1), "10");
    let accepted = f.stored_cursor_text(f.account_a);
    assert!(accepted.is_some());

    type Case = fn(
        NaiveDate,
    ) -> (
        Vec<NormalizedMetricObservationInput>,
        Vec<NormalizedMetricCoverageAssertionInput>,
    );
    let cases: Vec<(&str, Case, i64)> = vec![
        (
            "an unknown DOI the coordinator did not quarantine",
            |d| {
                (
                    vec![NormalizedMetricObservationInput {
                        work_doi: UNRESOLVED_DOI.into(),
                        ..observation("1", d)
                    }],
                    vec![complete_day(d)],
                )
            },
            0,
        ),
        (
            "a quarantined unknown DOI and an invalid DOI",
            |d| {
                (
                    vec![unresolved(UNRESOLVED_DOI, d), unresolved("not-a-doi", d)],
                    vec![complete_day(d)],
                )
            },
            1,
        ),
        (
            "a quarantined unknown DOI and an unquarantined one",
            |d| {
                (
                    vec![
                        unresolved(UNRESOLVED_DOI, d),
                        NormalizedMetricObservationInput {
                            source_record_id: Some("row-2".into()),
                            ..unresolved(SECOND_UNRESOLVED_DOI, d)
                        },
                    ],
                    vec![complete_day(d)],
                )
            },
            1,
        ),
        (
            "quarantine only, PARTIAL coverage",
            |d| {
                (
                    vec![unresolved(UNRESOLVED_DOI, d)],
                    vec![coverage(
                        MetricCoverageStatus::Partial,
                        d,
                        d.succ_opt().unwrap(),
                    )],
                )
            },
            1,
        ),
        (
            "quarantine only, UNKNOWN coverage",
            |d| {
                (
                    vec![unresolved(UNRESOLVED_DOI, d)],
                    vec![coverage(
                        MetricCoverageStatus::Unknown,
                        d,
                        d.succ_opt().unwrap(),
                    )],
                )
            },
            1,
        ),
        (
            "quarantine only, zero coverage rows",
            |d| (vec![unresolved(UNRESOLVED_DOI, d)], vec![]),
            1,
        ),
        (
            "quarantine only, COMPLETE and PARTIAL",
            |d| {
                (
                    vec![unresolved(UNRESOLVED_DOI, d)],
                    vec![
                        complete_day(d),
                        coverage(MetricCoverageStatus::Partial, d, d.succ_opt().unwrap()),
                    ],
                )
            },
            1,
        ),
        (
            "quarantine only, COMPLETE over a mismatched period",
            |d| {
                (
                    vec![unresolved(UNRESOLVED_DOI, d)],
                    vec![coverage(
                        MetricCoverageStatus::Complete,
                        d,
                        d.succ_opt().unwrap().succ_opt().unwrap(),
                    )],
                )
            },
            1,
        ),
    ];
    for (index, (label, case, quarantined)) in cases.into_iter().enumerate() {
        let claim = f.claim_a();
        let unit_day = day(10 + index as u32);
        let (observations, assertions) = case(unit_day);
        let import = f.run_batch_unit(
            claim.lease_token,
            &format!("report-{index}"),
            unit_day,
            &digest(100 + index as u64),
            observations,
            assertions,
        );
        assert_eq!(
            import.status,
            MetricImportStatus::CompletedWithErrors,
            "{label}"
        );
        assert_eq!(f.quarantined(import.import_id), quarantined, "{label}");
        let checkpoint = f
            .update(import.import_id, claim.lease_token)
            .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_eq!(checkpoint.last_completed_at, import.completed_at, "{label}");
        assert_eq!(checkpoint.lease_owner, None, "{label}: released");
        assert_eq!(
            checkpoint.last_successful_period_end,
            Some(day(2)),
            "{label}"
        );
        assert_eq!(
            f.stored_cursor_text(f.account_a),
            accepted,
            "{label}: no manifest is accepted"
        );
    }

    // A conflict: account B wins the cell first, then account A's import of
    // the same day conflicts with it while its only rejection is quarantined.
    let claims = f.claim(10).unwrap();
    assert_eq!(claims.len(), 2);
    let (claim_a, claim_b) = (&claims[0], &claims[1]);
    assert_eq!(claim_b.source_account.source_account_id, f.account_b);
    let conflict_day = day(25);
    let winner = begin_metric_import(
        &f.pool,
        ACTOR,
        &BeginMetricImportInput {
            source_account_code: ACCOUNT_B.into(),
            expected_batch_keys: vec!["only".into()],
            ..begin_input(claim_b.lease_token, "report-b", conflict_day)
        },
    )
    .unwrap();
    f.ingest(
        winner.import_id,
        claim_b.lease_token,
        "only",
        vec![NormalizedMetricObservationInput {
            source_account_code: ACCOUNT_B.into(),
            ..observation("10", conflict_day)
        }],
        vec![],
    )
    .unwrap();
    let import = f.run_batch_unit(
        claim_a.lease_token,
        "report-conflict",
        conflict_day,
        &digest(250),
        vec![
            observation("11", conflict_day),
            unresolved(UNRESOLVED_DOI, conflict_day),
        ],
        vec![complete_day(conflict_day)],
    );
    assert_eq!(import.status, MetricImportStatus::CompletedWithErrors);
    assert_eq!((import.invalid_count, import.conflict_count), (1, 1));
    assert_eq!(f.quarantined(import.import_id), 1);
    let checkpoint = f.update(import.import_id, claim_a.lease_token).unwrap();
    assert_eq!(checkpoint.last_completed_at, import.completed_at);
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));
    assert_eq!(f.stored_cursor_text(f.account_a), accepted, "a conflict");
}

#[test]
fn provably_inconsistent_quarantine_evidence_fails_closed_with_no_write() {
    let (_guard, f) = setup();
    let refused = Err(E::Ingestion(Code::InternalStateInconsistency));
    let claim = f.claim_a();
    let import = f.run_batch_unit(
        claim.lease_token,
        "report-1",
        day(1),
        &digest(1),
        vec![
            observation("10", day(1)),
            unresolved(UNRESOLVED_DOI, day(1)),
            unresolved(SECOND_UNRESOLVED_DOI, day(1)),
        ],
        vec![complete_day(day(1))],
    );
    assert_eq!(import.status, MetricImportStatus::CompletedWithErrors);
    let id = import.import_id;
    let rejected_row = format!(
        "record_provenance_id = (SELECT record_provenance_id FROM metric_record_provenance \
          WHERE import_id = '{id}' AND classification = 'REJECTED' ORDER BY batch_row_index LIMIT 1)"
    );
    let set_reason = |details: &str| {
        format!("UPDATE metric_record_provenance SET details = {details} WHERE {rejected_row}")
    };
    let restore_reason = set_reason("jsonb_set(details, '{reason_code}', '\"UNKNOWN_DOI\"', true)");
    let foreign_quarantine = format!(
        "INSERT INTO metric_identifier_quarantine \
             (record_provenance_id, source_account_id, platform_id, measure_id, schema_version, \
              work_doi, period_start, period_end, reporting_grain, value, methodology_version) \
         SELECT p.record_provenance_id, q.source_account_id, q.platform_id, q.measure_id, \
                q.schema_version, q.work_doi, q.period_start, q.period_end, q.reporting_grain, \
                q.value, q.methodology_version \
           FROM metric_record_provenance p, \
                (SELECT * FROM metric_identifier_quarantine LIMIT 1) q \
          WHERE p.import_id = '{id}' AND p.classification = 'WINNER'"
    );
    let cases: Vec<(&str, String, String)> = vec![
        (
            "invalid_count above the rejected rows",
            format!("UPDATE metric_import SET invalid_count = invalid_count + 1 WHERE import_id = '{id}'"),
            format!("UPDATE metric_import SET invalid_count = invalid_count - 1 WHERE import_id = '{id}'"),
        ),
        (
            "invalid_count below the rejected rows",
            format!("UPDATE metric_import SET invalid_count = invalid_count - 1 WHERE import_id = '{id}'"),
            format!("UPDATE metric_import SET invalid_count = invalid_count + 1 WHERE import_id = '{id}'"),
        ),
        (
            "a quarantine row on WINNER provenance",
            foreign_quarantine,
            format!("DELETE FROM metric_identifier_quarantine WHERE record_provenance_id IN (SELECT record_provenance_id FROM metric_record_provenance WHERE import_id = '{id}' AND classification = 'WINNER')"),
        ),
        (
            "a quarantine row on a rejection of another reason",
            set_reason("jsonb_set(details, '{reason_code}', '\"INVALID_DOI\"')"),
            restore_reason.clone(),
        ),
        (
            "a missing rejection reason",
            set_reason("details - 'reason_code'"),
            restore_reason.clone(),
        ),
        (
            "a JSON null rejection reason",
            set_reason("jsonb_set(details, '{reason_code}', 'null')"),
            restore_reason.clone(),
        ),
        (
            "a non-string rejection reason",
            set_reason("jsonb_set(details, '{reason_code}', '7')"),
            restore_reason.clone(),
        ),
        (
            "an unparseable rejection reason",
            set_reason("jsonb_set(details, '{reason_code}', '\"NOT_A_CODE\"')"),
            restore_reason.clone(),
        ),
        (
            "a rejection reason outside the exact vocabulary spelling",
            set_reason("jsonb_set(details, '{reason_code}', '\"unknown_doi\"')"),
            restore_reason.clone(),
        ),
        (
            "rejection details that are not an object",
            set_reason("'[\"UNKNOWN_DOI\"]'::jsonb"),
            set_reason(
                "'{\"schema\": \"thoth-metric-provenance-details/1\", \"reason_code\": \"UNKNOWN_DOI\", \"reporting_grain\": \"DAY\"}'::jsonb",
            ),
        ),
    ];
    let quarantine_state = |f: &Fixture| {
        text(
            &f.pool,
            "(SELECT string_agg(record_provenance_id::text, ',' ORDER BY record_provenance_id) \
               FROM metric_identifier_quarantine)",
        )
    };
    for (label, corrupt, restore) in &cases {
        f.sql(corrupt);
        let before = (f.snapshot(), quarantine_state(&f));
        assert_eq!(f.update(id, claim.lease_token), refused, "{label}");
        assert_eq!(
            (f.snapshot(), quarantine_state(&f)),
            before,
            "{label}: no progress, cursor, successful-period or release write"
        );
        assert_eq!(
            f.checkpoint(f.account_a).lease_owner,
            Some(claim.lease_token.to_string()),
            "{label}: the lease stays held"
        );
        f.sql(restore);
    }

    // Restored, the same live update records the quarantine-only manifest.
    let checkpoint = f.update(id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(checkpoint.last_successful_period_end, None);
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(1), digest(1))]))
    );

    // A released replay derives the same predicate first, so it fails closed
    // the same way and stays read-only.
    for (label, corrupt, restore) in &cases {
        f.sql(corrupt);
        let before = (f.snapshot(), quarantine_state(&f));
        assert_eq!(
            f.update(id, claim.lease_token),
            refused,
            "released: {label}"
        );
        assert_eq!(
            (f.snapshot(), quarantine_state(&f)),
            before,
            "released: {label}"
        );
        f.sql(restore);
    }
    assert_eq!(f.update(id, claim.lease_token).unwrap(), checkpoint);
}

#[test]
fn a_non_cloudfront_completed_with_errors_import_keeps_its_existing_behaviour() {
    let (_guard, f) = setup();
    // A managed DRIVER source that is not CloudFront, with the configuration
    // such a source requires.
    f.sql("UPDATE metric_source SET driver_key = 'crossref-events'");
    f.sql("UPDATE metric_source_account SET configuration = '{}'::jsonb");
    f.record_success("report-accepted", day(1), &digest(1), "10");
    let accepted = f.stored_cursor_text(f.account_a);

    // An eligible-shaped unknown DOI is an ordinary rejection: no quarantine,
    // and the update releases without a manifest.
    let claim = f.claim_a();
    let import = f.run_batch_unit(
        claim.lease_token,
        "report-2",
        day(2),
        &digest(2),
        vec![unresolved(UNRESOLVED_DOI, day(2))],
        vec![complete_day(day(2))],
    );
    assert_eq!(import.status, MetricImportStatus::CompletedWithErrors);
    assert_eq!(f.quarantined(import.import_id), 0);
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.last_completed_at, import.completed_at);
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));
    assert_eq!(f.stored_cursor_text(f.account_a), accepted);

    // The quarantine predicate never runs for it: evidence that would be
    // provably inconsistent for CloudFront neither fails nor accepts anything.
    let claim = f.claim_a();
    let import = f.run_batch_unit(
        claim.lease_token,
        "report-3",
        day(3),
        &digest(3),
        vec![unresolved(UNRESOLVED_DOI, day(3))],
        vec![complete_day(day(3))],
    );
    f.sql(&format!(
        "UPDATE metric_import SET invalid_count = invalid_count + 5 WHERE import_id = '{}'",
        import.import_id
    ));
    let checkpoint = f.update(import.import_id, claim.lease_token).unwrap();
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(checkpoint.last_successful_period_end, Some(day(2)));
    assert_eq!(f.stored_cursor_text(f.account_a), accepted);
}

#[test]
fn stale_expired_reclaimed_and_foreign_tokens_never_record_a_quarantine_only_manifest() {
    let (_guard, f) = setup();
    let (first, import) = f.quarantine_only_unit("report-1", day(1), &digest(1));

    // Expired, not yet reclaimed.
    f.expire(f.account_a);
    let before = f.snapshot();
    assert_eq!(f.update(import.import_id, first), Err(E::StaleSourceClaim));
    assert_eq!(f.snapshot(), before);

    // Reclaimed: the old token and a foreign token are both stale.
    let second = f.claim_a();
    let before = f.snapshot();
    for token in [first, Uuid::new_v4()] {
        assert_eq!(f.update(import.import_id, token), Err(E::StaleSourceClaim));
    }
    assert_eq!(f.snapshot(), before);
    assert_eq!(f.stored_cursor(f.account_a), None);

    // Only the live holder records the same terminal import.
    let checkpoint = f.update(import.import_id, second.lease_token).unwrap();
    assert_eq!(checkpoint.last_successful_period_end, None);
    assert_eq!(checkpoint.lease_owner, None);
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[(day(1), digest(1))]))
    );
}

/// A session holding the rows `query` selects `FOR UPDATE` in an open
/// transaction, as a canonical authority writer would.
struct RowHold {
    connection: PgConnection,
}

impl RowHold {
    fn acquire(query: &str) -> Self {
        let mut connection = PgConnection::establish(&test_db_url()).expect("hold session");
        sql_query("BEGIN")
            .execute(&mut connection)
            .expect("open the hold");
        sql_query(format!("{query} FOR UPDATE"))
            .execute(&mut connection)
            .expect("lock the held row");
        RowHold { connection }
    }

    fn release(mut self) {
        sql_query("COMMIT")
            .execute(&mut self.connection)
            .expect("release the hold");
    }
}

/// Whether another session can lock the rows `query` selects `FOR UPDATE
/// NOWAIT` right now. The probe commits at once, releasing what it took.
fn row_is_free(query: &str) -> bool {
    let mut connection = PgConnection::establish(&test_db_url()).expect("probe session");
    connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            sql_query(format!("{query} FOR UPDATE NOWAIT")).execute(connection)?;
            Ok(())
        })
        .is_ok()
}

#[test]
fn the_checkpoint_update_locks_checkpoint_import_account_then_source() {
    let (_guard, f) = setup();
    let checkpoint_row = format!(
        "SELECT 1 FROM metric_source_checkpoint WHERE source_account_id = '{}'",
        f.account_a
    );
    let account_row = format!(
        "SELECT 1 FROM metric_source_account WHERE source_account_id = '{}'",
        f.account_a
    );
    let source_row = format!(
        "SELECT 1 FROM metric_source WHERE source_id = '{}'",
        f.source_id
    );
    // Each round holds one authority row the way a canonical writer would,
    // lets the update queue on it, and probes which rows the queued update
    // already holds and which it has not yet reached.
    let rounds: [(&str, &str, [bool; 3]); 3] = [
        // (held row, label, [import free, account free, source free])
        ("import", "the import", [false, true, true]),
        ("account", "the source account", [false, false, true]),
        ("source", "the source", [false, false, false]),
    ];
    for (index, (held, label, [import_free, account_free, source_free])) in
        rounds.into_iter().enumerate()
    {
        let (token, import) = f.quarantine_only_unit(
            &format!("report-lock-{index}"),
            day(1 + index as u32),
            &digest(1 + index as u64),
        );
        let import_row = format!(
            "SELECT 1 FROM metric_import WHERE import_id = '{}'",
            import.import_id
        );
        let hold = RowHold::acquire(match held {
            "import" => &import_row,
            "account" => &account_row,
            _ => &source_row,
        });
        let deadlocks_before = scalar_i64(
            &f.pool,
            "(SELECT deadlocks FROM pg_stat_database WHERE datname = current_database())",
        );
        let update = queued_update(&f, import.import_id, token);
        assert!(
            !row_is_free(&checkpoint_row),
            "{label}: the checkpoint is locked first"
        );
        if held != "import" {
            assert_eq!(row_is_free(&import_row), import_free, "{label}: import");
        }
        if held != "account" {
            assert_eq!(row_is_free(&account_row), account_free, "{label}: account");
        }
        if held != "source" {
            assert_eq!(row_is_free(&source_row), source_free, "{label}: source");
        }
        hold.release();
        let checkpoint = update
            .join()
            .expect("update thread")
            .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_eq!(checkpoint.lease_owner, None, "{label}");
        assert_eq!(
            scalar_i64(
                &f.pool,
                "(SELECT deadlocks FROM pg_stat_database WHERE datname = current_database())",
            ),
            deadlocks_before,
            "{label}: no deadlock"
        );
    }
    assert_eq!(
        f.stored_cursor(f.account_a),
        Some(cursor_json(&[
            (day(1), digest(1)),
            (day(2), digest(2)),
            (day(3), digest(3))
        ]))
    );
}
