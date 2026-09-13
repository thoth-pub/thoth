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

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{sql_query, Connection, RunQueryDsl};
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
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_import::{MetricImport, MetricImportStatus};
use crate::model::metric_ingestion::MetricIngestionErrorCode as Code;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification as Class;
use crate::model::metric_source_checkpoint::MetricSourceCheckpoint;
use crate::model::tests::db::{test_db_url, TestDbGuard};

pub(crate) const SOURCE_CODE: &str = "cloudfront-driver";
pub(crate) const ACCOUNT_A: &str = "acct-a";
const ACCOUNT_B: &str = "acct-b";
const ACTOR: &str = "metrics-ingest-service-1";
const WORK_DOI: &str = "https://doi.org/10.12345/thoth-wp2-02";
const METHODOLOGY: &str = "cloudfront-title-session/2";
pub(crate) const DIGEST: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const RAW_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
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
                    last_discovered_at, last_completed_at, last_successful_period_end), ';' \
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
        let input = BeginMetricImportInput {
            expected_batch_keys: vec!["only".into()],
            ..begin_input(token, upstream, start)
        };
        let import = begin_metric_import(&self.pool, ACTOR, &input).expect("begin");
        self.ingest(
            import.import_id,
            token,
            "only",
            vec![observation("10", start)],
            coverage,
        )
        .expect("batch");
        self.complete(import.import_id, token).expect("complete")
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
        assert_eq!(
            (checkpoint.cursor, checkpoint.last_error),
            (None, None),
            "{label}"
        );
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
