//! Focused `MET-WP2-01B` tests for the canonical ingestion coordinator.
//!
//! Three families of evidence live here:
//!
//! - pure unit tests of the deterministic encodings, the country list, the
//!   grain/period rules and the closed code vocabulary (no database);
//! - database tests of every approved outcome, rejection, coverage,
//!   idempotency and request-level boundary, asserting final persisted state;
//! - real multi-connection PostgreSQL races, asserting final persisted state
//!   rather than thread scheduling.
//!
//! Failure injection (Amendment 5 D5) uses ephemeral trigger functions that
//! exist only in the disposable local test database, are defined only in this
//! file, are removed by a `Drop` guard, and can never be installed or invoked
//! by production runtime code.

use std::collections::BTreeSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Barrier};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{sql_query, Connection, RunQueryDsl};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::country::{is_assigned_alpha2, ASSIGNED_ALPHA2};
use super::hash::{
    cell_lock_key, content_hash, days_from_epoch, enum_code, identity_hash, request_hash,
    CanonicalEncoder, CanonicalIdentity, CELL_LOCK_DOMAIN, CONTENT_DOMAIN, IDENTITY_DOMAIN,
    REQUEST_DOMAIN,
};
use super::{
    classify_unique, ingest_metric_batch, period_matches_grain, MetricIngestionBatch,
    MetricIngestionError, MetricIngestionErrorCode as Code, MetricIngestionOutcome,
    NormalizedMetricCoverageAssertion, NormalizedMetricObservation, MAX_BATCH_KEY_BYTES,
    MAX_COVERAGE_ASSERTIONS, MAX_OBSERVATIONS, PROVENANCE_DETAILS_SCHEMA, ROLLUP_DELTA_PENDING,
    SUPPORTED_SCHEMA_VERSION,
};
use crate::db::PgPool;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification as Class;
use crate::model::publication::PublicationType;
use crate::model::tests::db::{failing_pool, test_db_url, TestDbGuard};

// ---------------------------------------------------------------------------
// Constants shared by the golden vectors and the fixture
// ---------------------------------------------------------------------------

const PLATFORM_CODE: &str = "cf";
const ACCOUNT_A: &str = "acct-a";
const ACCOUNT_B: &str = "acct-b";
const WORK_DOI: &str = "https://doi.org/10.12345/thoth-01b";
const OTHER_DOI: &str = "https://doi.org/10.12345/thoth-01b-other";
const PDF_ISBN: &str = "978-3-16-148410-0";
const OTHER_ISBN: &str = "978-0-306-40615-7";
const THIRD_ISBN: &str = "978-1-4028-9462-6";
const ROR: &str = "https://ror.org/02mhbdp94";
const OTHER_ROR: &str = "https://ror.org/03yrm5c26";
const TITLE_SESSIONS_METHODOLOGY: &str = "cloudfront-title-session/2";

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
}

fn uuid(text: &str) -> Uuid {
    Uuid::parse_str(text).expect("valid test uuid")
}

fn golden_identity() -> CanonicalIdentity {
    CanonicalIdentity {
        platform_id: uuid("11111111-1111-4111-8111-111111111111"),
        measure_id: uuid("22222222-2222-4222-8222-222222222222"),
        work_id: uuid("33333333-3333-4333-8333-333333333333"),
        publication_id: Some(uuid("44444444-4444-4444-8444-444444444444")),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        country_code: Some("GB".to_string()),
        institution_id: Some(uuid("55555555-5555-4555-8555-555555555555")),
    }
}

fn golden_observation() -> NormalizedMetricObservation {
    NormalizedMetricObservation {
        source_account_code: ACCOUNT_A.to_string(),
        platform_code: PLATFORM_CODE.to_string(),
        measure_code: "title_sessions".to_string(),
        work_doi: WORK_DOI.to_string(),
        publication_isbn: Some(PDF_ISBN.to_string()),
        publication_type: Some(PublicationType::Pdf),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 3, 2),
        reporting_grain: MetricReportingGrain::Day,
        country_code: Some("GB".to_string()),
        institution_ror: Some(ROR.to_string()),
        value: 7,
        source_record_id: Some("rec-1".to_string()),
        methodology_version: TITLE_SESSIONS_METHODOLOGY.to_string(),
        source_row_number: Some(3),
    }
}

fn golden_coverage() -> NormalizedMetricCoverageAssertion {
    NormalizedMetricCoverageAssertion {
        platform_code: PLATFORM_CODE.to_string(),
        measure_code: "title_sessions".to_string(),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        status: MetricCoverageStatus::Complete,
        country_coverage: true,
        institution_coverage: false,
        notes: Some("n".to_string()),
    }
}

fn golden_batch() -> MetricIngestionBatch {
    MetricIngestionBatch {
        import_id: uuid("66666666-6666-4666-8666-666666666666"),
        batch_key: "golden".to_string(),
        schema_version: SUPPORTED_SCHEMA_VERSION.to_string(),
        observations: vec![golden_observation()],
        coverage: vec![golden_coverage()],
    }
}

/// Independent re-encoding of the specified primitives, used to prove field
/// order without going through `CanonicalEncoder`.
mod manual {
    use super::*;

    pub fn string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    pub fn opt_string(bytes: &mut Vec<u8>, value: Option<&str>) {
        match value {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                string(bytes, value);
            }
        }
    }
    pub fn date(bytes: &mut Vec<u8>, value: NaiveDate) {
        let epoch = super::date(1970, 1, 1);
        bytes.extend_from_slice(&value.signed_duration_since(epoch).num_days().to_be_bytes());
    }
    pub fn i64(bytes: &mut Vec<u8>, value: i64) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    pub fn opt_i64(bytes: &mut Vec<u8>, value: Option<i64>) {
        match value {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                i64(bytes, value);
            }
        }
    }
    pub fn hex(bytes: &[u8]) -> String {
        hex::encode(Sha256::digest(bytes))
    }
}

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

#[test]
fn every_domain_separator_ends_in_a_real_nul_byte() {
    for (name, domain) in [
        ("identity", IDENTITY_DOMAIN),
        ("content", CONTENT_DOMAIN),
        ("request", REQUEST_DOMAIN),
        ("cell lock", CELL_LOCK_DOMAIN),
    ] {
        assert_eq!(domain.last(), Some(&0u8), "{name} domain must end in 0x00");
        assert_eq!(
            domain.iter().filter(|byte| **byte == 0).count(),
            1,
            "{name} domain must contain exactly one NUL"
        );
        assert!(
            !domain.windows(2).any(|pair| pair == b"\\0"),
            "{name} domain must not spell backslash-zero"
        );
    }
    assert_eq!(IDENTITY_DOMAIN, b"thoth-metric-record-identity/v1\x00");
    assert_eq!(CONTENT_DOMAIN, b"thoth-metric-record-content/v1\x00");
    assert_eq!(REQUEST_DOMAIN, b"thoth-metric-import-batch-request/v1\x00");
    assert_eq!(CELL_LOCK_DOMAIN, b"thoth-metric-cell-lock/v1\x00");
}

#[test]
fn primitive_encodings_match_the_specification() {
    let mut encoder = CanonicalEncoder::with_domain(b"d\0");
    encoder.string("hé");
    encoder.optional_string(None);
    encoder.optional_string(Some(""));
    encoder.uuid(uuid("00112233-4455-6677-8899-aabbccddeeff"));
    encoder.optional_uuid(None);
    encoder.date(date(1969, 12, 31));
    encoder.i64(-2);
    encoder.optional_i64(Some(1));
    encoder.bool(false);
    encoder.bool(true);
    encoder.count(3);
    let mut expected = b"d\0".to_vec();
    expected.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 3, b'h', 0xC3, 0xA9]);
    expected.push(0);
    expected.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, 0]);
    expected.extend_from_slice(&[
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF,
    ]);
    expected.push(0);
    expected.extend_from_slice(&(-1i64).to_be_bytes());
    expected.extend_from_slice(&(-2i64).to_be_bytes());
    expected.push(1);
    expected.extend_from_slice(&1i64.to_be_bytes());
    expected.extend_from_slice(&[0, 1]);
    expected.extend_from_slice(&3u64.to_be_bytes());
    assert_eq!(encoder.preimage(), expected.as_slice());
}

#[test]
fn date_encoding_is_signed_days_from_the_unix_epoch() {
    assert_eq!(days_from_epoch(date(1970, 1, 1)), 0);
    assert_eq!(days_from_epoch(date(1969, 12, 31)), -1);
    assert_eq!(days_from_epoch(date(2026, 3, 1)), 20_513);
    assert_eq!(days_from_epoch(date(2026, 4, 1)), 20_544);
}

#[test]
fn golden_identity_hash_vector() {
    assert_eq!(
        identity_hash(&golden_identity()),
        "d62b755cfa08b51f8302369799de141b35f888e67ada1eb33ba8ea52170129e4"
    );
}

#[test]
fn golden_content_hash_vector() {
    assert_eq!(
        content_hash(&golden_identity(), 42, TITLE_SESSIONS_METHODOLOGY),
        "3f55e93f3801b00db0447f558146e609441872322f98ff052f88f67a149bef44"
    );
}

#[test]
fn golden_cell_lock_key_vector() {
    assert_eq!(
        cell_lock_key(&golden_identity()),
        -5_184_695_916_552_390_869
    );
}

#[test]
fn golden_batch_request_hash_vector() {
    assert_eq!(
        request_hash(&golden_batch()),
        "a88c0e281571be31370206f607713017e4a9a70811fa09d332ee200e86ca638f"
    );
}

#[test]
fn hashes_are_lowercase_sha256_hex() {
    for hash in [
        identity_hash(&golden_identity()),
        content_hash(&golden_identity(), 1, "m"),
        request_hash(&golden_batch()),
    ] {
        assert_eq!(hash.len(), 64);
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}

#[test]
fn explicit_null_markers_distinguish_absent_from_empty_and_present() {
    let mut without_country = golden_identity();
    without_country.country_code = None;
    let mut empty_country = golden_identity();
    empty_country.country_code = Some(String::new());
    let hashes: BTreeSet<String> = [golden_identity(), without_country, empty_country]
        .iter()
        .map(identity_hash)
        .collect();
    assert_eq!(hashes.len(), 3);

    let mut no_notes = golden_batch();
    no_notes.coverage[0].notes = None;
    let mut empty_notes = golden_batch();
    empty_notes.coverage[0].notes = Some(String::new());
    let hashes: BTreeSet<String> = [golden_batch(), no_notes, empty_notes]
        .iter()
        .map(request_hash)
        .collect();
    assert_eq!(hashes.len(), 3);
}

#[test]
fn observation_reorder_changes_the_request_hash() {
    let mut second = golden_observation();
    second.value = 8;
    let ordered = MetricIngestionBatch {
        observations: vec![golden_observation(), second.clone()],
        ..golden_batch()
    };
    let reordered = MetricIngestionBatch {
        observations: vec![second, golden_observation()],
        ..golden_batch()
    };
    assert_ne!(request_hash(&ordered), request_hash(&reordered));
}

#[test]
fn import_id_and_batch_key_do_not_affect_the_request_hash() {
    let other = MetricIngestionBatch {
        import_id: Uuid::new_v4(),
        batch_key: "another-key".to_string(),
        ..golden_batch()
    };
    assert_eq!(request_hash(&golden_batch()), request_hash(&other));
}

#[test]
fn value_and_methodology_affect_content_but_not_identity() {
    let identity = golden_identity();
    let base = content_hash(&identity, 42, TITLE_SESSIONS_METHODOLOGY);
    assert_ne!(
        base,
        content_hash(&identity, 43, TITLE_SESSIONS_METHODOLOGY)
    );
    assert_ne!(base, content_hash(&identity, 42, "other/1"));
    assert_eq!(identity_hash(&identity), identity_hash(&golden_identity()));
    // Content hashes the identity fields directly, never the identity hex.
    let mut bytes = CONTENT_DOMAIN.to_vec();
    let mut encoder = CanonicalEncoder::with_domain(b"");
    encoder.uuid(identity.platform_id);
    encoder.uuid(identity.measure_id);
    encoder.uuid(identity.work_id);
    encoder.optional_uuid(identity.publication_id);
    encoder.date(identity.period_start);
    encoder.date(identity.period_end);
    encoder.optional_string(identity.country_code.as_deref());
    encoder.optional_uuid(identity.institution_id);
    encoder.i64(42);
    encoder.string(TITLE_SESSIONS_METHODOLOGY);
    bytes.extend_from_slice(encoder.preimage());
    assert_eq!(base, manual::hex(&bytes));
}

#[test]
fn reporting_grain_affects_the_request_hash_but_neither_canonical_hash() {
    // Canonical identity and content are built from `CanonicalIdentity`,
    // which carries no grain, source account, value-independent source
    // record or row identifier.
    let mut other_grain = golden_batch();
    other_grain.observations[0].reporting_grain = MetricReportingGrain::ReportingPeriod;
    assert_ne!(request_hash(&golden_batch()), request_hash(&other_grain));
    let _: CanonicalIdentity = CanonicalIdentity {
        platform_id: Uuid::nil(),
        measure_id: Uuid::nil(),
        work_id: Uuid::nil(),
        publication_id: None,
        period_start: date(2026, 1, 1),
        period_end: date(2026, 1, 2),
        country_code: None,
        institution_id: None,
    };
}

#[test]
fn durable_enums_hash_through_their_canonical_serde_codes() {
    assert_eq!(enum_code(&MetricReportingGrain::Day), "DAY");
    assert_eq!(enum_code(&MetricReportingGrain::Month), "MONTH");
    assert_eq!(
        enum_code(&MetricReportingGrain::ReportingPeriod),
        "REPORTING_PERIOD"
    );
    assert_eq!(enum_code(&MetricCoverageStatus::Complete), "COMPLETE");
    assert_eq!(enum_code(&MetricCoverageStatus::Partial), "PARTIAL");
    assert_eq!(enum_code(&MetricCoverageStatus::Unknown), "UNKNOWN");
    assert_eq!(enum_code(&PublicationType::Pdf), "PDF");
    assert_eq!(enum_code(&PublicationType::Paperback), "PAPERBACK");
    assert_eq!(enum_code(&PublicationType::FictionBook), "FICTION_BOOK");
    assert_eq!(enum_code(&PublicationType::Azw3), "AZW3");
    // Not the database presentation label or the display label.
    assert_ne!(enum_code(&PublicationType::Paperback), "Paperback");
}

#[test]
fn observation_request_hash_field_order_matches_amendment_2_a3() {
    let observation = golden_observation();
    let mut bytes = REQUEST_DOMAIN.to_vec();
    manual::string(&mut bytes, SUPPORTED_SCHEMA_VERSION);
    bytes.extend_from_slice(&1u64.to_be_bytes());
    manual::string(&mut bytes, &observation.source_account_code);
    manual::string(&mut bytes, &observation.platform_code);
    manual::string(&mut bytes, &observation.measure_code);
    manual::string(&mut bytes, &observation.work_doi);
    manual::opt_string(&mut bytes, observation.publication_isbn.as_deref());
    manual::opt_string(&mut bytes, Some("PDF"));
    manual::date(&mut bytes, observation.period_start);
    manual::date(&mut bytes, observation.period_end);
    manual::string(&mut bytes, "DAY");
    manual::opt_string(&mut bytes, observation.country_code.as_deref());
    manual::opt_string(&mut bytes, observation.institution_ror.as_deref());
    manual::i64(&mut bytes, observation.value);
    manual::opt_string(&mut bytes, observation.source_record_id.as_deref());
    manual::string(&mut bytes, &observation.methodology_version);
    manual::opt_i64(&mut bytes, observation.source_row_number);
    bytes.extend_from_slice(&0u64.to_be_bytes());
    let batch = MetricIngestionBatch {
        coverage: vec![],
        ..golden_batch()
    };
    assert_eq!(request_hash(&batch), manual::hex(&bytes));
}

#[test]
fn coverage_request_hash_field_order_matches_amendment_2_a4() {
    let coverage = golden_coverage();
    let mut bytes = REQUEST_DOMAIN.to_vec();
    manual::string(&mut bytes, SUPPORTED_SCHEMA_VERSION);
    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(&1u64.to_be_bytes());
    manual::string(&mut bytes, &coverage.platform_code);
    manual::string(&mut bytes, &coverage.measure_code);
    manual::date(&mut bytes, coverage.period_start);
    manual::date(&mut bytes, coverage.period_end);
    manual::string(&mut bytes, "COMPLETE");
    bytes.push(1);
    bytes.push(0);
    manual::opt_string(&mut bytes, coverage.notes.as_deref());
    let batch = MetricIngestionBatch {
        observations: vec![],
        ..golden_batch()
    };
    assert_eq!(request_hash(&batch), manual::hex(&bytes));
}

type ObservationMutation = Box<dyn Fn(&mut NormalizedMetricObservation)>;
type CoverageMutation = Box<dyn Fn(&mut NormalizedMetricCoverageAssertion)>;

#[test]
fn changing_any_single_included_field_changes_the_request_hash() {
    let base = request_hash(&golden_batch());
    let observation_mutations: Vec<ObservationMutation> = vec![
        Box::new(|o| o.source_account_code.push('x')),
        Box::new(|o| o.platform_code.push('x')),
        Box::new(|o| o.measure_code.push('x')),
        Box::new(|o| o.work_doi.push('x')),
        Box::new(|o| o.publication_isbn = None),
        Box::new(|o| o.publication_type = Some(PublicationType::Epub)),
        Box::new(|o| o.period_start = date(2026, 2, 28)),
        Box::new(|o| o.period_end = date(2026, 3, 3)),
        Box::new(|o| o.reporting_grain = MetricReportingGrain::Month),
        Box::new(|o| o.country_code = Some("US".to_string())),
        Box::new(|o| o.institution_ror = None),
        Box::new(|o| o.value += 1),
        Box::new(|o| o.source_record_id = None),
        Box::new(|o| o.methodology_version.push('x')),
        Box::new(|o| o.source_row_number = Some(4)),
    ];
    let coverage_mutations: Vec<CoverageMutation> = vec![
        Box::new(|c| c.platform_code.push('x')),
        Box::new(|c| c.measure_code.push('x')),
        Box::new(|c| c.period_start = date(2026, 2, 1)),
        Box::new(|c| c.period_end = date(2026, 5, 1)),
        Box::new(|c| c.status = MetricCoverageStatus::Partial),
        Box::new(|c| c.country_coverage = false),
        Box::new(|c| c.institution_coverage = true),
        Box::new(|c| c.notes = None),
    ];
    let mut seen = BTreeSet::new();
    seen.insert(base);
    for mutate in &observation_mutations {
        let mut batch = golden_batch();
        mutate(&mut batch.observations[0]);
        assert!(
            seen.insert(request_hash(&batch)),
            "observation field change must alter the hash"
        );
    }
    for mutate in &coverage_mutations {
        let mut batch = golden_batch();
        mutate(&mut batch.coverage[0]);
        assert!(
            seen.insert(request_hash(&batch)),
            "coverage field change must alter the hash"
        );
    }
    assert_eq!(
        seen.len(),
        1 + observation_mutations.len() + coverage_mutations.len()
    );
}

// ---------------------------------------------------------------------------
// Country, grain, classification, code vocabulary
// ---------------------------------------------------------------------------

#[test]
fn the_alpha2_list_is_the_249_assigned_codes_sorted_unique_and_uppercase() {
    assert_eq!(ASSIGNED_ALPHA2.len(), 249);
    let set: BTreeSet<&str> = ASSIGNED_ALPHA2.iter().copied().collect();
    assert_eq!(set.len(), 249);
    assert!(ASSIGNED_ALPHA2.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(ASSIGNED_ALPHA2
        .iter()
        .all(|code| code.len() == 2 && code.bytes().all(|b| b.is_ascii_uppercase())));
}

#[test]
fn country_membership_is_exact_full_alpha2_validation() {
    for accepted in ["AD", "GB", "US", "DE", "TW", "SS", "ZW"] {
        assert!(is_assigned_alpha2(accepted), "{accepted}");
    }
    for rejected in [
        "gb", "Gb", "UK", "EU", "XK", "GBR", " GB", "GB ", "", "AA", "ZZ",
    ] {
        assert!(!is_assigned_alpha2(rejected), "{rejected:?}");
    }
}

#[test]
fn period_grain_rules_are_exact() {
    use MetricReportingGrain::*;
    assert!(period_matches_grain(
        Day,
        date(2026, 3, 1),
        date(2026, 3, 2)
    ));
    assert!(period_matches_grain(
        Day,
        date(2026, 12, 31),
        date(2027, 1, 1)
    ));
    assert!(!period_matches_grain(
        Day,
        date(2026, 3, 1),
        date(2026, 3, 3)
    ));
    assert!(!period_matches_grain(
        Day,
        date(2026, 3, 1),
        date(2026, 3, 1)
    ));
    assert!(!period_matches_grain(
        Day,
        date(2026, 3, 2),
        date(2026, 3, 1)
    ));
    assert!(period_matches_grain(
        Month,
        date(2026, 3, 1),
        date(2026, 4, 1)
    ));
    assert!(period_matches_grain(
        Month,
        date(2026, 12, 1),
        date(2027, 1, 1)
    ));
    assert!(period_matches_grain(
        Month,
        date(2024, 2, 1),
        date(2024, 3, 1)
    ));
    assert!(!period_matches_grain(
        Month,
        date(2026, 3, 2),
        date(2026, 4, 1)
    ));
    assert!(!period_matches_grain(
        Month,
        date(2026, 3, 1),
        date(2026, 3, 31)
    ));
    assert!(!period_matches_grain(
        Month,
        date(2026, 3, 1),
        date(2026, 5, 1)
    ));
    assert!(period_matches_grain(
        ReportingPeriod,
        date(2026, 3, 1),
        date(2026, 3, 2)
    ));
    assert!(period_matches_grain(
        ReportingPeriod,
        date(2026, 3, 3),
        date(2026, 9, 17)
    ));
    assert!(!period_matches_grain(
        ReportingPeriod,
        date(2026, 3, 1),
        date(2026, 3, 1)
    ));
    assert!(!period_matches_grain(
        ReportingPeriod,
        date(2026, 3, 2),
        date(2026, 3, 1)
    ));
}

#[test]
fn resolution_classification_is_deterministic_including_unreachable_ambiguity() {
    // Amendment 5 D3: DOI and publication-type ambiguity cannot exist in a
    // valid persisted database, so the classification branch is proven here.
    assert_eq!(
        classify_unique::<Uuid>(&[], Code::UnknownDoi, Code::AmbiguousDoi),
        Err(Code::UnknownDoi)
    );
    let one = Uuid::new_v4();
    assert_eq!(
        classify_unique(&[one], Code::UnknownDoi, Code::AmbiguousDoi),
        Ok(one)
    );
    assert_eq!(
        classify_unique(&[one, Uuid::new_v4()], Code::UnknownDoi, Code::AmbiguousDoi),
        Err(Code::AmbiguousDoi)
    );
    assert_eq!(
        classify_unique(
            &[one, one, one],
            Code::UnknownPublication,
            Code::AmbiguousPublication
        ),
        Err(Code::AmbiguousPublication)
    );
    assert_eq!(
        classify_unique::<Uuid>(&[], Code::UnknownPublication, Code::AmbiguousPublication),
        Err(Code::UnknownPublication)
    );
}

#[test]
fn error_codes_are_stable_screaming_snake_case_and_round_trip() {
    let expected = [
        (Code::EmptyBatch, "EMPTY_BATCH"),
        (Code::InvalidBatchKey, "INVALID_BATCH_KEY"),
        (Code::BatchLimitExceeded, "BATCH_LIMIT_EXCEEDED"),
        (Code::UnsupportedSchemaVersion, "UNSUPPORTED_SCHEMA_VERSION"),
        (Code::ImportNotFound, "IMPORT_NOT_FOUND"),
        (Code::ImportNotProcessing, "IMPORT_NOT_PROCESSING"),
        (
            Code::ImportSourceScopeMismatch,
            "IMPORT_SOURCE_SCOPE_MISMATCH",
        ),
        (Code::IdempotencyKeyReused, "IDEMPOTENCY_KEY_REUSED"),
        (
            Code::InternalStateInconsistency,
            "INTERNAL_STATE_INCONSISTENCY",
        ),
        (Code::InternalDatabaseError, "INTERNAL_DATABASE_ERROR"),
        (
            Code::ConcurrentAuthorityChange,
            "CONCURRENT_AUTHORITY_CHANGE",
        ),
        (Code::AcquisitionTypeDeferred, "ACQUISITION_TYPE_DEFERRED"),
        (Code::SourceAccountMismatch, "SOURCE_ACCOUNT_MISMATCH"),
        (Code::SourceAccountDisabled, "SOURCE_ACCOUNT_DISABLED"),
        (Code::SourceDisabled, "SOURCE_DISABLED"),
        (Code::PlatformMismatch, "PLATFORM_MISMATCH"),
        (Code::PlatformDisabled, "PLATFORM_DISABLED"),
        (Code::MeasureNotFound, "MEASURE_NOT_FOUND"),
        (Code::MeasureDisabled, "MEASURE_DISABLED"),
        (Code::PlatformMeasureNotFound, "PLATFORM_MEASURE_NOT_FOUND"),
        (Code::PlatformMeasureDisabled, "PLATFORM_MEASURE_DISABLED"),
        (Code::NotDirectCollection, "NOT_DIRECT_COLLECTION"),
        (
            Code::UnsupportedReportingGrain,
            "UNSUPPORTED_REPORTING_GRAIN",
        ),
        (
            Code::InvalidReportingGrainPeriod,
            "INVALID_REPORTING_GRAIN_PERIOD",
        ),
        (
            Code::UnsupportedPublicationDimension,
            "UNSUPPORTED_PUBLICATION_DIMENSION",
        ),
        (
            Code::UnsupportedCountryDimension,
            "UNSUPPORTED_COUNTRY_DIMENSION",
        ),
        (
            Code::UnsupportedInstitutionDimension,
            "UNSUPPORTED_INSTITUTION_DIMENSION",
        ),
        (Code::InvalidDoi, "INVALID_DOI"),
        (Code::UnknownDoi, "UNKNOWN_DOI"),
        (Code::AmbiguousDoi, "AMBIGUOUS_DOI"),
        (Code::InvalidIsbn, "INVALID_ISBN"),
        (Code::UnknownPublication, "UNKNOWN_PUBLICATION"),
        (Code::AmbiguousPublication, "AMBIGUOUS_PUBLICATION"),
        (Code::InvalidRor, "INVALID_ROR"),
        (Code::UnknownRor, "UNKNOWN_ROR"),
        (Code::AmbiguousRor, "AMBIGUOUS_ROR"),
        (Code::InvalidCountry, "INVALID_COUNTRY"),
        (Code::InvalidValue, "INVALID_VALUE"),
        (Code::MethodologyRequired, "METHODOLOGY_REQUIRED"),
        (Code::MethodologyMismatch, "METHODOLOGY_MISMATCH"),
        (Code::PublisherScopeMismatch, "PUBLISHER_SCOPE_MISMATCH"),
        (
            Code::MetricsCollectNotEntitled,
            "METRICS_COLLECT_NOT_ENTITLED",
        ),
        (Code::OverlappingPeriod, "OVERLAPPING_PERIOD"),
        (Code::SourceConflict, "SOURCE_CONFLICT"),
        (Code::ConflictingFinalRecord, "CONFLICTING_FINAL_RECORD"),
        (Code::InvalidCoveragePeriod, "INVALID_COVERAGE_PERIOD"),
        (Code::InvalidCoverageDimension, "INVALID_COVERAGE_DIMENSION"),
    ];
    for (code, text) in expected {
        assert_eq!(code.to_string(), text);
        assert_eq!(text.parse::<Code>(), Ok(code));
        assert_eq!(
            serde_json::to_value(code).unwrap(),
            serde_json::Value::String(text.into())
        );
    }
    assert!("something_else".parse::<Code>().is_err());
    assert_eq!(
        MetricIngestionError::new(Code::EmptyBatch).to_string(),
        "metric ingestion request failed: EMPTY_BATCH"
    );
}

#[test]
fn import_error_field_names_and_messages_are_static_and_bounded() {
    assert_eq!(Code::UnknownDoi.field_name(false), Some("work_doi"));
    assert_eq!(
        Code::UnknownPublication.field_name(true),
        Some("publication_isbn")
    );
    assert_eq!(
        Code::UnknownPublication.field_name(false),
        Some("publication_type")
    );
    assert_eq!(Code::InvalidCountry.field_name(false), Some("country_code"));
    assert_eq!(
        Code::OverlappingPeriod.field_name(false),
        Some("period_start")
    );
    assert_eq!(Code::SourceConflict.field_name(false), None);
    assert!(Code::InvalidValue.message().len() < 120);
    assert!(!Code::InvalidValue.message().trim().is_empty());
}

// ---------------------------------------------------------------------------
// Request shape: decided before any database access
// ---------------------------------------------------------------------------

#[test]
fn request_shape_failures_precede_database_access() {
    let pool = failing_pool();
    let base = golden_batch();
    let expect = |batch: MetricIngestionBatch, code: Code| {
        assert_eq!(
            ingest_metric_batch(&pool, &batch),
            Err(MetricIngestionError::new(code)),
            "{code}"
        );
    };
    expect(
        MetricIngestionBatch {
            batch_key: "   ".into(),
            ..base.clone()
        },
        Code::InvalidBatchKey,
    );
    expect(
        MetricIngestionBatch {
            batch_key: "k".repeat(MAX_BATCH_KEY_BYTES + 1),
            ..base.clone()
        },
        Code::InvalidBatchKey,
    );
    expect(
        MetricIngestionBatch {
            batch_key: "é".repeat(129),
            ..base.clone()
        },
        Code::InvalidBatchKey,
    );
    expect(
        MetricIngestionBatch {
            schema_version: "thoth-normalized-metrics/2".into(),
            ..base.clone()
        },
        Code::UnsupportedSchemaVersion,
    );
    expect(
        MetricIngestionBatch {
            observations: vec![],
            coverage: vec![],
            ..base.clone()
        },
        Code::EmptyBatch,
    );
    expect(
        MetricIngestionBatch {
            observations: vec![golden_observation(); MAX_OBSERVATIONS + 1],
            ..base.clone()
        },
        Code::BatchLimitExceeded,
    );
    expect(
        MetricIngestionBatch {
            coverage: vec![golden_coverage(); MAX_COVERAGE_ASSERTIONS + 1],
            ..base.clone()
        },
        Code::BatchLimitExceeded,
    );
    // Within limits, the failing pool proves the next step is the database.
    assert_eq!(
        ingest_metric_batch(&pool, &base),
        Err(MetricIngestionError::new(Code::InternalDatabaseError))
    );
}

// ---------------------------------------------------------------------------
// Database fixture and helpers
// ---------------------------------------------------------------------------

fn establish() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database")
}

fn exec(connection: &mut PgConnection, sql: &str) {
    sql_query(sql)
        .execute(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn exec_uuid(connection: &mut PgConnection, sql: &str, id: Uuid) {
    sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn count(pool: &PgPool, table: &str, predicate: &str) -> i64 {
    scalar_i64(
        pool,
        &format!("(SELECT COUNT(*) FROM {table} WHERE {predicate})"),
    )
}

fn text(pool: &PgPool, query: &str) -> Option<String> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Nullable<diesel::sql_types::Text>,
    >(query))
    .get_result(&mut connection)
    .expect("Failed to run text query")
}

/// The canonical Thoth and Metrics rows one MOM-1 DRIVER ingestion needs.
struct Fixture {
    pool: Arc<PgPool>,
    platform_id: Uuid,
    account_a: Uuid,
    account_b: Uuid,
    publisher_id: Uuid,
    other_publisher_id: Uuid,
    imprint_id: Uuid,
    other_imprint_id: Uuid,
    work_id: Uuid,
    other_work_id: Uuid,
    publication_id: Uuid,
    institution_id: Uuid,
    title_sessions_id: Uuid,
    net_units_id: Uuid,
    mapping_title_sessions_id: Uuid,
    import_a: Uuid,
    import_b: Uuid,
}

/// A fresh registry database plus the complete fixture.
///
/// `publisher` carries the `OBELISK` package (which has `METRICS_COLLECT`);
/// `other_publisher` stays `OASIS`. The `title_sessions` mapping supports every
/// grain and every dimension; the `net_units` mapping supports only `MONTH`
/// and no optional dimension. `orphan` is an enabled measure with no mapping.
fn setup_fixture() -> (TestDbGuard, Fixture) {
    let (guard, pool) = setup_registry_db();
    let mut c = pool.get().expect("Failed to get DB connection");
    let ids: Vec<Uuid> = (0..14).map(|_| Uuid::new_v4()).collect();
    let [source_id, platform_id, account_a, account_b, publisher_id, other_publisher_id, imprint_id, other_imprint_id, work_id, other_work_id, publication_id, institution_id, import_a, import_b] =
        ids[..]
    else {
        unreachable!()
    };

    exec_uuid(&mut c, "INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) VALUES ($1, 'cloudfront-driver', 'DRIVER', 'cloudfront', TRUE)", source_id);
    exec_uuid(&mut c, "INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ($1, 'cf', 'CloudFront', 'THOTH_MANAGED', TRUE)", platform_id);
    exec_uuid(&mut c, "INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ($1, 'Expected publisher', 'OBELISK')", publisher_id);
    exec_uuid(&mut c, "INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ($1, 'Other publisher', 'OASIS')", other_publisher_id);
    exec(&mut c, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, enabled) VALUES ('{account_a}', 'acct-a', '{source_id}', '{platform_id}', 'dist-a', '{publisher_id}', TRUE)"));
    exec(&mut c, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, enabled) VALUES ('{account_b}', 'acct-b', '{source_id}', '{platform_id}', 'dist-b', '{publisher_id}', TRUE)"));
    exec(&mut c, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint_id}', '{publisher_id}', 'Imprint')"));
    exec(&mut c, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{other_imprint_id}', '{other_publisher_id}', 'Other imprint')"));
    exec(&mut c, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{work_id}', 'monograph', 'forthcoming', '{imprint_id}', 1, '{WORK_DOI}')"));
    exec(&mut c, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{other_work_id}', 'monograph', 'forthcoming', '{imprint_id}', 1, '{OTHER_DOI}')"));
    exec(&mut c, &format!("INSERT INTO publication (publication_id, publication_type, work_id, isbn) VALUES ('{publication_id}', 'PDF', '{work_id}', '{PDF_ISBN}')"));
    exec(&mut c, &format!("INSERT INTO institution (institution_id, institution_name, ror) VALUES ('{institution_id}', 'Institution', '{ROR}')"));

    let title_sessions_id = uuid(
        &text(
            &pool,
            "(SELECT measure_id::text FROM metric_measure WHERE code = 'title_sessions')",
        )
        .unwrap(),
    );
    let net_units_id = uuid(
        &text(
            &pool,
            "(SELECT measure_id::text FROM metric_measure WHERE code = 'net_units')",
        )
        .unwrap(),
    );
    exec(&mut c, "INSERT INTO metric_measure (code, display_name, category, unit, allow_negative, additive_across_time, additive_across_works, definition, enabled) VALUES ('orphan', 'Orphan', 'USAGE', 'COUNT', FALSE, TRUE, TRUE, 'No mapping', TRUE)");
    let mapping_title_sessions_id = Uuid::new_v4();
    let mapping_net_units_id = Uuid::new_v4();
    exec(&mut c, &format!("INSERT INTO metric_platform_measure (platform_measure_id, platform_id, measure_id, supported_grains, supports_country, supports_institution, supports_publication, direct_collection, enabled) VALUES ('{mapping_title_sessions_id}', '{platform_id}', '{title_sessions_id}', ARRAY['DAY','MONTH','REPORTING_PERIOD']::metric_reporting_grain[], TRUE, TRUE, TRUE, TRUE, TRUE)"));
    exec(&mut c, &format!("INSERT INTO metric_platform_measure (platform_measure_id, platform_id, measure_id, supported_grains, supports_country, supports_institution, supports_publication, direct_collection, enabled) VALUES ('{mapping_net_units_id}', '{platform_id}', '{net_units_id}', ARRAY['MONTH']::metric_reporting_grain[], FALSE, FALSE, FALSE, TRUE, TRUE)"));

    exec(&mut c, &format!("INSERT INTO metric_import (import_id, source_account_id, publisher_id, format_code, format_version, status, normalizer_version, created_by) VALUES ('{import_a}', '{account_a}', '{publisher_id}', 'cloudfront', '1', 'PROCESSING', 'normalizer/1', 'test')"));
    exec(&mut c, &format!("INSERT INTO metric_import (import_id, source_account_id, publisher_id, format_code, format_version, status, normalizer_version, created_by) VALUES ('{import_b}', '{account_b}', '{publisher_id}', 'cloudfront', '1', 'PROCESSING', 'normalizer/1', 'test')"));
    drop(c);

    (
        guard,
        Fixture {
            pool,
            platform_id,
            account_a,
            account_b,
            publisher_id,
            other_publisher_id,
            imprint_id,
            other_imprint_id,
            work_id,
            other_work_id,
            publication_id,
            institution_id,
            title_sessions_id,
            net_units_id,
            mapping_title_sessions_id,
            import_a,
            import_b,
        },
    )
}

impl Fixture {
    fn sql(&self, sql: &str) {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        exec(&mut c, sql);
    }

    /// One valid `title_sessions` DAY observation from account A.
    fn observation(&self) -> NormalizedMetricObservation {
        NormalizedMetricObservation {
            source_account_code: ACCOUNT_A.into(),
            platform_code: PLATFORM_CODE.into(),
            measure_code: "title_sessions".into(),
            work_doi: WORK_DOI.into(),
            publication_isbn: None,
            publication_type: None,
            period_start: date(2026, 3, 1),
            period_end: date(2026, 3, 2),
            reporting_grain: MetricReportingGrain::Day,
            country_code: None,
            institution_ror: None,
            value: 10,
            source_record_id: Some("row-1".into()),
            methodology_version: TITLE_SESSIONS_METHODOLOGY.into(),
            source_row_number: Some(1),
        }
    }

    /// An observation eligible for unresolved-DOI quarantine: a syntactically
    /// valid DOI no work carries, and none of the five optional fields the
    /// reduced quarantine representation cannot hold.
    fn unresolved(&self, doi: &str) -> NormalizedMetricObservation {
        NormalizedMetricObservation {
            work_doi: doi.into(),
            source_record_id: None,
            source_row_number: None,
            ..self.observation()
        }
    }

    /// The same observation as presented by account B.
    fn observation_b(&self) -> NormalizedMetricObservation {
        NormalizedMetricObservation {
            source_account_code: ACCOUNT_B.into(),
            ..self.observation()
        }
    }

    fn batch(
        &self,
        key: &str,
        observations: Vec<NormalizedMetricObservation>,
    ) -> MetricIngestionBatch {
        MetricIngestionBatch {
            import_id: self.import_a,
            batch_key: key.into(),
            schema_version: SUPPORTED_SCHEMA_VERSION.into(),
            observations,
            coverage: vec![],
        }
    }

    fn batch_b(
        &self,
        key: &str,
        observations: Vec<NormalizedMetricObservation>,
    ) -> MetricIngestionBatch {
        MetricIngestionBatch {
            import_id: self.import_b,
            ..self.batch(key, observations)
        }
    }

    fn ingest(
        &self,
        batch: &MetricIngestionBatch,
    ) -> Result<MetricIngestionOutcome, MetricIngestionError> {
        ingest_metric_batch(&self.pool, batch)
    }

    fn accept(&self, batch: &MetricIngestionBatch) -> MetricIngestionOutcome {
        self.ingest(batch).expect("batch must commit")
    }

    fn snapshot(&self) -> DurableState {
        DurableState::capture(&self.pool)
    }

    fn counters(&self, import_id: Uuid) -> [i64; 6] {
        let row = text(&self.pool, &format!("(SELECT received_count || ',' || accepted_count || ',' || duplicate_count || ',' || revision_count || ',' || conflict_count || ',' || invalid_count FROM metric_import WHERE import_id = '{import_id}')")).unwrap();
        let values: Vec<i64> = row.split(',').map(|v| v.parse().unwrap()).collect();
        values.try_into().unwrap()
    }

    fn record_id(&self, identity_hash: &str) -> Option<Uuid> {
        text(&self.pool, &format!("(SELECT record_id::text FROM metric_record WHERE identity_hash = '{identity_hash}')")).map(|id| uuid(&id))
    }

    fn current_value(&self, record_id: Uuid) -> i64 {
        scalar_i64(&self.pool, &format!("(SELECT r.value FROM metric_record m JOIN metric_record_revision r ON r.record_revision_id = m.current_revision_id WHERE m.record_id = '{record_id}')"))
    }
}

/// Every durable consequence a batch could have, for zero-consequence proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DurableState {
    batches: i64,
    provenance: i64,
    errors: i64,
    coverage: i64,
    records: i64,
    revisions: i64,
    deltas: i64,
    counters: Vec<String>,
    institutions: i64,
    quarantine: i64,
}

impl DurableState {
    fn capture(pool: &PgPool) -> Self {
        let mut c = pool.get().expect("Failed to get DB connection");
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            line: String,
        }
        let counters = sql_query("SELECT import_id::text || ':' || received_count || ',' || accepted_count || ',' || duplicate_count || ',' || revision_count || ',' || conflict_count || ',' || invalid_count AS line FROM metric_import ORDER BY import_id")
            .load::<Row>(&mut c)
            .unwrap()
            .into_iter()
            .map(|row| row.line)
            .collect();
        drop(c);
        DurableState {
            batches: count(pool, "metric_import_batch", "TRUE"),
            provenance: count(pool, "metric_record_provenance", "TRUE"),
            errors: count(pool, "metric_import_error", "TRUE"),
            coverage: count(pool, "metric_coverage", "TRUE"),
            records: count(pool, "metric_record", "TRUE"),
            revisions: count(pool, "metric_record_revision", "TRUE"),
            deltas: count(pool, "metric_rollup_delta", "TRUE"),
            counters,
            institutions: count(pool, "institution", "TRUE"),
            quarantine: count(pool, "metric_identifier_quarantine", "TRUE"),
        }
    }
}

/// A batch outcome must be a request-level failure with zero durable effect.
fn assert_request_failure(fixture: &Fixture, batch: &MetricIngestionBatch, code: Code) {
    let before = fixture.snapshot();
    assert_eq!(
        fixture.ingest(batch),
        Err(MetricIngestionError::new(code)),
        "{code}"
    );
    assert_eq!(
        fixture.snapshot(),
        before,
        "{code} must leave no durable consequence"
    );
}

// ---------------------------------------------------------------------------
// Test-only failure injection and pause points (Amendment 5 D5)
// ---------------------------------------------------------------------------

/// An ephemeral PL/pgSQL trigger installed in the disposable test database and
/// removed on drop. Defined only here; production runtime code cannot install
/// or invoke it.
struct TestTrigger {
    name: String,
    table: String,
}

impl TestTrigger {
    /// A trigger whose body raises, so the statement it guards fails.
    fn failing(pool: &PgPool, timing_and_event: &str, table: &str, when: Option<&str>) -> Self {
        Self::install(
            pool,
            timing_and_event,
            table,
            when,
            "RAISE EXCEPTION 'thoth test failure injection'; RETURN NULL;",
            false,
        )
    }

    /// A deferred constraint trigger that raises at COMMIT time.
    fn failing_at_commit(pool: &PgPool, event: &str, table: &str) -> Self {
        Self::install(
            pool,
            event,
            table,
            None,
            "RAISE EXCEPTION 'thoth test commit failure injection'; RETURN NULL;",
            true,
        )
    }

    /// A trigger that blocks on the transaction-level advisory lock `key`
    /// until the test releases its session-level hold of the same key.
    fn pausing(pool: &PgPool, timing_and_event: &str, table: &str, key: i64) -> Self {
        Self::install(
            pool,
            timing_and_event,
            table,
            None,
            &format!("PERFORM pg_advisory_xact_lock({key}); RETURN NEW;"),
            false,
        )
    }

    fn install(
        pool: &PgPool,
        timing_and_event: &str,
        table: &str,
        when: Option<&str>,
        body: &str,
        deferred: bool,
    ) -> Self {
        let name = format!("thoth_test_{}", Uuid::new_v4().simple());
        let mut c = pool.get().expect("Failed to get DB connection");
        exec(&mut c, &format!("CREATE FUNCTION {name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN {body} END $$"));
        let when = when.map(|w| format!(" WHEN ({w})")).unwrap_or_default();
        if deferred {
            exec(&mut c, &format!("CREATE CONSTRAINT TRIGGER {name} AFTER {timing_and_event} ON {table} DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION {name}()"));
        } else {
            exec(&mut c, &format!("CREATE TRIGGER {name} {timing_and_event} ON {table} FOR EACH ROW{when} EXECUTE FUNCTION {name}()"));
        }
        TestTrigger {
            name,
            table: table.into(),
        }
    }
}

impl Drop for TestTrigger {
    fn drop(&mut self) {
        let mut c = establish();
        exec(
            &mut c,
            &format!("DROP TRIGGER IF EXISTS {} ON {}", self.name, self.table),
        );
        exec(&mut c, &format!("DROP FUNCTION IF EXISTS {}()", self.name));
    }
}

const PAUSE_KEY: i64 = 987_654_321_001;

/// Hold the pause key on a dedicated session so a pausing trigger blocks.
struct PauseHold {
    connection: PgConnection,
}

impl PauseHold {
    fn acquire() -> Self {
        let mut connection = establish();
        exec(
            &mut connection,
            &format!("SELECT pg_advisory_lock({PAUSE_KEY})"),
        );
        PauseHold { connection }
    }

    fn release(mut self) {
        exec(
            &mut self.connection,
            &format!("SELECT pg_advisory_unlock({PAUSE_KEY})"),
        );
    }
}

// ---------------------------------------------------------------------------
// Real-connection concurrency helpers
// ---------------------------------------------------------------------------

fn pool_of(size: u32) -> Arc<PgPool> {
    Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(size)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to build a test pool"),
    )
}

fn backend_pid(connection: &mut PgConnection) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
        "pg_backend_pid()",
    ))
    .get_result::<i32>(connection)
    .expect("pg_backend_pid") as i64
}

/// The backend pid a single-connection pool will hand to the coordinator.
fn pinned_pool() -> (Arc<PgPool>, i64) {
    let pool = pool_of(1);
    let pid = backend_pid(&mut pool.get().unwrap());
    (pool, pid)
}

/// Poll `pg_stat_activity` until `pid` is waiting on a heavyweight lock.
fn wait_until_blocked(pid: i64) {
    let mut monitor = establish();
    let started = Instant::now();
    loop {
        let blocked: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
            "(SELECT COUNT(*) FROM pg_stat_activity WHERE pid = {pid} AND wait_event_type = 'Lock')"
        )))
        .get_result(&mut monitor)
        .unwrap();
        if blocked == 1 {
            return;
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "backend {pid} never blocked on a lock"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

/// A transaction on its own connection that runs `statements`, then waits
/// for `release` before committing. The statements themselves may block.
struct HeldTransaction {
    pid: i64,
    release: Sender<()>,
    handle: JoinHandle<()>,
}

impl HeldTransaction {
    fn start(statements: Vec<String>) -> Self {
        let (release, wait): (Sender<()>, Receiver<()>) = channel();
        let (pid_tx, pid_rx) = channel();
        let handle = thread::spawn(move || {
            let mut connection = establish();
            pid_tx.send(backend_pid(&mut connection)).unwrap();
            connection
                .transaction::<_, diesel::result::Error, _>(|connection| {
                    for statement in &statements {
                        sql_query(statement).execute(connection)?;
                    }
                    wait.recv().expect("release signal");
                    Ok(())
                })
                .expect("held transaction must commit");
        });
        let pid = pid_rx.recv().unwrap();
        HeldTransaction {
            pid,
            release,
            handle,
        }
    }

    /// Wait until this transaction's current statement is itself blocked.
    fn wait_until_blocked(&self) {
        wait_until_blocked(self.pid);
    }

    fn commit(self) {
        self.release.send(()).unwrap();
        self.handle.join().unwrap();
    }
}

fn run_concurrently<F1, F2, R1, R2>(first: F1, second: F2) -> (R1, R2)
where
    F1: FnOnce() -> R1 + Send + 'static,
    F2: FnOnce() -> R2 + Send + 'static,
    R1: Send + 'static,
    R2: Send + 'static,
{
    let barrier = Arc::new(Barrier::new(2));
    let barrier_one = Arc::clone(&barrier);
    let one = thread::spawn(move || {
        barrier_one.wait();
        first()
    });
    let two = thread::spawn(move || {
        barrier.wait();
        second()
    });
    (one.join().unwrap(), two.join().unwrap())
}

fn deadlocks(pool: &PgPool) -> i64 {
    scalar_i64(
        pool,
        "(SELECT deadlocks FROM pg_stat_database WHERE datname = current_database())",
    )
}

// ---------------------------------------------------------------------------
// Canonical outcomes
// ---------------------------------------------------------------------------

#[test]
fn first_arrival_creates_record_revision_provenance_delta_batch_and_counters() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch("b1", vec![f.observation()]);
    let outcome = f.accept(&batch);

    assert!(!outcome.replayed);
    assert_eq!(outcome.request_hash, request_hash(&batch));
    assert_eq!(outcome.rows.len(), 1);
    let row = &outcome.rows[0];
    assert_eq!(row.batch_row_index, 0);
    assert_eq!(row.classification, Class::Winner);
    assert_eq!(row.reason_code, None);
    let record_id = row.record_id.expect("winner has a record");

    let identity = CanonicalIdentity {
        platform_id: f.platform_id,
        measure_id: f.title_sessions_id,
        work_id: f.work_id,
        publication_id: None,
        period_start: date(2026, 3, 1),
        period_end: date(2026, 3, 2),
        country_code: None,
        institution_id: None,
    };
    let expected_identity = identity_hash(&identity);
    let expected_content = content_hash(&identity, 10, TITLE_SESSIONS_METHODOLOGY);
    assert_eq!(
        row.identity_hash.as_deref(),
        Some(expected_identity.as_str())
    );
    assert_eq!(row.content_hash.as_deref(), Some(expected_content.as_str()));

    assert_eq!(f.record_id(&expected_identity), Some(record_id));
    assert_eq!(count(&f.pool, "metric_record", &format!("record_id = '{record_id}' AND work_id = '{}' AND platform_id = '{}' AND measure_id = '{}' AND publication_id IS NULL AND country_code IS NULL AND institution_id IS NULL AND period_start = DATE '2026-03-01' AND period_end = DATE '2026-03-02' AND reporting_grain = 'DAY' AND winning_source_account_id = '{}' AND current_revision_id IS NOT NULL", f.work_id, f.platform_id, f.title_sessions_id, f.account_a)), 1);
    assert_eq!(count(&f.pool, "metric_record_revision", &format!("record_id = '{record_id}' AND revision_number = 1 AND import_id = '{}' AND value = 10 AND content_hash = '{expected_content}' AND status = 'CURRENT' AND supersedes_revision_id IS NULL", f.import_a)), 1);
    assert_eq!(count(&f.pool, "metric_record", &format!("record_id = '{record_id}' AND current_revision_id = (SELECT record_revision_id FROM metric_record_revision WHERE record_id = '{record_id}')")), 1);
    assert_eq!(count(&f.pool, "metric_import_batch", &format!("import_batch_id = '{}' AND import_id = '{}' AND batch_key = 'b1' AND request_hash = '{}'", outcome.import_batch_id, f.import_a, outcome.request_hash)), 1);
    assert_eq!(count(&f.pool, "metric_record_provenance", &format!("record_id = '{record_id}' AND import_id = '{}' AND source_record_id = 'row-1' AND source_row_number = 1 AND identity_hash = '{expected_identity}' AND content_hash = '{expected_content}' AND classification = 'WINNER' AND import_batch_id = '{}' AND batch_row_index = 0", f.import_a, outcome.import_batch_id)), 1);
    assert_eq!(
        text(&f.pool, &format!("(SELECT details::text FROM metric_record_provenance WHERE record_id = '{record_id}')")).unwrap(),
        format!(r#"{{"schema": "{PROVENANCE_DETAILS_SCHEMA}", "reason_code": null, "reporting_grain": "DAY"}}"#)
    );
    assert_eq!(count(&f.pool, "metric_rollup_delta", &format!("record_id = '{record_id}' AND revision_id = (SELECT current_revision_id FROM metric_record WHERE record_id = '{record_id}') AND delta_value = 10 AND status = '{ROLLUP_DELTA_PENDING}' AND applied_at IS NULL")), 1);
    assert_eq!(count(&f.pool, "metric_import_error", "TRUE"), 0);
    assert_eq!(f.counters(f.import_a), [1, 1, 0, 0, 0, 0]);
    assert_eq!(f.counters(f.import_b), [0, 0, 0, 0, 0, 0]);
}

#[test]
fn exact_duplicate_from_the_winner_creates_duplicate_provenance_only() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let before = f.snapshot();
    let second = f.accept(&f.batch("b2", vec![f.observation()]));
    let after = f.snapshot();

    assert_eq!(second.rows[0].classification, Class::Duplicate);
    assert_eq!(second.rows[0].reason_code, None);
    assert_eq!(second.rows[0].record_id, first.rows[0].record_id);
    assert_eq!(second.rows[0].identity_hash, first.rows[0].identity_hash);
    assert_eq!(after.records, before.records);
    assert_eq!(after.revisions, before.revisions);
    assert_eq!(after.deltas, before.deltas);
    assert_eq!(after.batches, before.batches + 1);
    assert_eq!(after.provenance, before.provenance + 1);
    assert_eq!(after.errors, 0);
    assert_eq!(f.counters(f.import_a), [2, 1, 1, 0, 0, 0]);
    assert_eq!(
        count(
            &f.pool,
            "metric_record_provenance",
            "classification = 'DUPLICATE' AND details->>'reason_code' IS NULL"
        ),
        1
    );
}

#[test]
fn exact_duplicate_from_another_eligible_driver_account_is_a_duplicate() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let second = f.accept(&f.batch_b("b1", vec![f.observation_b()]));
    assert_eq!(second.rows[0].classification, Class::Duplicate);
    assert_eq!(second.rows[0].record_id, first.rows[0].record_id);
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("winning_source_account_id = '{}'", f.account_a)
        ),
        1
    );
    assert_eq!(count(&f.pool, "metric_record_revision", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_rollup_delta", "TRUE"), 1);
    assert_eq!(f.counters(f.import_a), [1, 1, 0, 0, 0, 0]);
    assert_eq!(f.counters(f.import_b), [1, 0, 1, 0, 0, 0]);
}

#[test]
fn changed_content_from_the_winning_account_revises_atomically() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let revised = NormalizedMetricObservation {
        value: 25,
        ..f.observation()
    };
    let second = f.accept(&f.batch("b2", vec![revised]));

    assert_eq!(second.rows[0].classification, Class::Revision);
    assert_eq!(second.rows[0].record_id, Some(record_id));
    assert_eq!(second.rows[0].identity_hash, first.rows[0].identity_hash);
    assert_ne!(second.rows[0].content_hash, first.rows[0].content_hash);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record_revision", &format!("record_id = '{record_id}' AND revision_number = 1 AND status = 'SUPERSEDED' AND value = 10")), 1);
    assert_eq!(count(&f.pool, "metric_record_revision", &format!("record_id = '{record_id}' AND revision_number = 2 AND status = 'CURRENT' AND value = 25 AND import_id = '{}' AND supersedes_revision_id = (SELECT record_revision_id FROM metric_record_revision WHERE record_id = '{record_id}' AND revision_number = 1)", f.import_a)), 1);
    assert_eq!(count(&f.pool, "metric_record", &format!("record_id = '{record_id}' AND current_revision_id = (SELECT record_revision_id FROM metric_record_revision WHERE record_id = '{record_id}' AND revision_number = 2) AND winning_source_account_id = '{}'", f.account_a)), 1);
    assert_eq!(count(&f.pool, "metric_record_provenance", &format!("record_id = '{record_id}' AND classification = 'REVISION' AND details->>'reason_code' IS NULL")), 1);
    assert_eq!(count(&f.pool, "metric_rollup_delta", &format!("record_id = '{record_id}' AND delta_value = 15 AND status = 'PENDING' AND revision_id = (SELECT current_revision_id FROM metric_record WHERE record_id = '{record_id}')")), 1);
    assert_eq!(count(&f.pool, "metric_rollup_delta", "TRUE"), 2);
    assert_eq!(f.counters(f.import_a), [2, 1, 0, 1, 0, 0]);
    assert_eq!(f.current_value(record_id), 25);
}

#[test]
fn methodology_only_change_is_a_revision_with_a_zero_delta() {
    let (_guard, f) = setup_fixture();
    let net_units = |methodology: &str| NormalizedMetricObservation {
        measure_code: "net_units".into(),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        reporting_grain: MetricReportingGrain::Month,
        value: -3,
        methodology_version: methodology.into(),
        ..f.observation()
    };
    let first = f.accept(&f.batch("b1", vec![net_units("sales/1")]));
    assert_eq!(first.rows[0].classification, Class::Winner);
    let second = f.accept(&f.batch("b2", vec![net_units("sales/2")]));
    assert_eq!(second.rows[0].classification, Class::Revision);
    let record_id = first.rows[0].record_id.unwrap();
    assert_eq!(
        count(
            &f.pool,
            "metric_rollup_delta",
            &format!("record_id = '{record_id}' AND delta_value = 0")
        ),
        1
    );
    assert_eq!(count(&f.pool, "metric_record_revision", &format!("record_id = '{record_id}' AND revision_number = 2 AND status = 'CURRENT' AND value = -3")), 1);
    assert_eq!(f.current_value(record_id), -3);
}

#[test]
fn changed_content_from_a_non_winning_account_is_conflict_evidence_only() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let before = f.snapshot();
    let conflicting = NormalizedMetricObservation {
        value: 99,
        ..f.observation_b()
    };
    let second = f.accept(&f.batch_b("b1", vec![conflicting]));
    let after = f.snapshot();

    assert_eq!(second.rows[0].classification, Class::Conflict);
    assert_eq!(second.rows[0].reason_code, Some(Code::SourceConflict));
    assert_eq!(second.rows[0].record_id, Some(record_id));
    assert!(second.rows[0].identity_hash.is_some() && second.rows[0].content_hash.is_some());
    assert_eq!(after.records, before.records);
    assert_eq!(after.revisions, before.revisions);
    assert_eq!(after.deltas, before.deltas);
    assert_eq!(after.errors, 0);
    assert_eq!(f.current_value(record_id), 10);
    assert_eq!(count(&f.pool, "metric_record", &format!("record_id = '{record_id}' AND winning_source_account_id = '{}' AND current_revision_id = (SELECT record_revision_id FROM metric_record_revision WHERE revision_number = 1 AND record_id = '{record_id}')", f.account_a)), 1);
    assert_eq!(count(&f.pool, "metric_record_provenance", &format!("record_id = '{record_id}' AND classification = 'CONFLICT' AND details->>'reason_code' = 'SOURCE_CONFLICT' AND import_id = '{}'", f.import_b)), 1);
    assert_eq!(f.counters(f.import_b), [1, 0, 0, 0, 1, 0]);
}

#[test]
fn revision_delta_overflow_rejects_only_that_observation() {
    // Both overflow directions around i64::MIN / i64::MAX (Amendment 5 D2).
    for (seed, revised) in [(-10, i64::MAX), (10, i64::MIN)] {
        let (_guard, f) = setup_fixture();
        let net_units = |value: i64| NormalizedMetricObservation {
            measure_code: "net_units".into(),
            period_start: date(2026, 3, 1),
            period_end: date(2026, 4, 1),
            reporting_grain: MetricReportingGrain::Month,
            value,
            methodology_version: "sales/1".into(),
            ..f.observation()
        };
        let first = f.accept(&f.batch("b1", vec![net_units(seed)]));
        let record_id = first.rows[0].record_id.unwrap();
        let before = f.snapshot();
        let valid = NormalizedMetricObservation {
            period_start: date(2026, 5, 1),
            period_end: date(2026, 6, 1),
            ..net_units(1)
        };
        let second = f.accept(&f.batch("b2", vec![net_units(revised), valid]));
        let after = f.snapshot();
        assert_eq!(second.rows[0].classification, Class::Rejected);
        assert_eq!(second.rows[0].reason_code, Some(Code::InvalidValue));
        assert!(second.rows[0].identity_hash.is_some() && second.rows[0].content_hash.is_some());
        assert_eq!(second.rows[1].classification, Class::Winner);
        assert_eq!(f.current_value(record_id), seed);
        assert_eq!(
            count(
                &f.pool,
                "metric_record_revision",
                &format!("record_id = '{record_id}'")
            ),
            1
        );
        assert_eq!(
            count(
                &f.pool,
                "metric_rollup_delta",
                &format!("record_id = '{record_id}'")
            ),
            1
        );
        assert_eq!(after.records, before.records + 1);
        assert_eq!(after.revisions, before.revisions + 1);
        assert_eq!(
            count(
                &f.pool,
                "metric_import_error",
                "error_code = 'INVALID_VALUE' AND field_name = 'value' AND raw_value IS NULL"
            ),
            1
        );
        assert_eq!(f.counters(f.import_a), [3, 2, 0, 0, 0, 1]);
    }
}

#[test]
fn overlapping_period_in_the_same_cell_is_rejected_and_adjacent_periods_accepted() {
    let (_guard, f) = setup_fixture();
    let period =
        |start: NaiveDate, end: NaiveDate, country: Option<&str>| NormalizedMetricObservation {
            period_start: start,
            period_end: end,
            reporting_grain: MetricReportingGrain::ReportingPeriod,
            country_code: country.map(Into::into),
            ..f.observation()
        };
    let first = f.accept(&f.batch(
        "b1",
        vec![period(date(2026, 3, 1), date(2026, 3, 10), None)],
    ));
    assert_eq!(first.rows[0].classification, Class::Winner);
    let second = f.accept(&f.batch(
        "b2",
        vec![
            period(date(2026, 3, 5), date(2026, 3, 15), None),
            period(date(2026, 2, 20), date(2026, 3, 2), None),
            period(date(2026, 3, 10), date(2026, 3, 20), None),
            period(date(2026, 2, 20), date(2026, 3, 1), None),
            period(date(2026, 3, 5), date(2026, 3, 15), Some("GB")),
        ],
    ));
    let classes: Vec<_> = second
        .rows
        .iter()
        .map(|row| (row.classification, row.reason_code))
        .collect();
    assert_eq!(
        classes,
        vec![
            (Class::Rejected, Some(Code::OverlappingPeriod)),
            (Class::Rejected, Some(Code::OverlappingPeriod)),
            (Class::Winner, None),
            (Class::Winner, None),
            (Class::Winner, None),
        ]
    );
    assert!(second.rows[0].identity_hash.is_some() && second.rows[0].content_hash.is_some());
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 4);
    assert_eq!(
        count(
            &f.pool,
            "metric_import_error",
            "error_code = 'OVERLAPPING_PERIOD' AND field_name = 'period_start'"
        ),
        2
    );
    assert_eq!(f.counters(f.import_a), [6, 4, 0, 0, 0, 2]);
    // Overlap inside one batch is also prevented: the second row of a pair
    // sees the first row's record.
    let third = f.accept(&f.batch(
        "b3",
        vec![
            period(date(2026, 6, 1), date(2026, 6, 10), None),
            period(date(2026, 6, 5), date(2026, 6, 6), None),
        ],
    ));
    assert_eq!(third.rows[1].reason_code, Some(Code::OverlappingPeriod));
}

#[test]
fn a_different_grain_label_for_the_same_identity_is_not_a_new_identity() {
    let (_guard, f) = setup_fixture();
    let month = NormalizedMetricObservation {
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        reporting_grain: MetricReportingGrain::Month,
        ..f.observation()
    };
    let reporting_period = NormalizedMetricObservation {
        reporting_grain: MetricReportingGrain::ReportingPeriod,
        ..month.clone()
    };
    let first = f.accept(&f.batch("b1", vec![month]));
    let second = f.accept(&f.batch("b2", vec![reporting_period]));
    assert_eq!(second.rows[0].classification, Class::Duplicate);
    assert_eq!(second.rows[0].record_id, first.rows[0].record_id);
    assert_eq!(
        count(&f.pool, "metric_record", "reporting_grain = 'MONTH'"),
        1
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_provenance",
            "classification = 'DUPLICATE' AND details->>'reporting_grain' = 'REPORTING_PERIOD'"
        ),
        1
    );
}

#[test]
fn rejected_and_valid_rows_in_one_batch_both_commit_with_exact_counters() {
    let (_guard, f) = setup_fixture();
    let invalid = NormalizedMetricObservation {
        country_code: Some("gb".into()),
        source_row_number: Some(7),
        ..f.observation()
    };
    let outcome = f.accept(&f.batch("b1", vec![invalid, f.observation()]));
    assert_eq!(outcome.rows[0].classification, Class::Rejected);
    assert_eq!(outcome.rows[0].reason_code, Some(Code::InvalidCountry));
    assert_eq!(outcome.rows[0].identity_hash, None);
    assert_eq!(outcome.rows[0].content_hash, None);
    assert_eq!(outcome.rows[1].classification, Class::Winner);
    assert_eq!(count(&f.pool, "metric_record_provenance", &format!("classification = 'REJECTED' AND record_id IS NULL AND identity_hash IS NULL AND content_hash IS NULL AND batch_row_index = 0 AND import_batch_id = '{}' AND details->>'reason_code' = 'INVALID_COUNTRY'", outcome.import_batch_id)), 1);
    assert_eq!(count(&f.pool, "metric_import_error", &format!("import_id = '{}' AND row_number = 7 AND error_code = 'INVALID_COUNTRY' AND severity = 'ERROR' AND field_name = 'country_code' AND raw_value IS NULL AND message = '{}'", f.import_a, Code::InvalidCountry.message())), 1);
    assert_eq!(count(&f.pool, "metric_import_error", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
    assert_eq!(f.counters(f.import_a), [2, 1, 0, 0, 0, 1]);
}

// ---------------------------------------------------------------------------
// Per-observation rejections
// ---------------------------------------------------------------------------

fn assert_rejected(
    f: &Fixture,
    key: &str,
    observation: NormalizedMetricObservation,
    code: Code,
    field: &str,
) {
    let before = f.snapshot();
    let coded_errors = |f: &Fixture| {
        count(
            &f.pool,
            "metric_import_error",
            &format!("error_code = '{code}' AND field_name = '{field}' AND raw_value IS NULL"),
        )
    };
    let coded_before = coded_errors(f);
    let counters_before = f.counters(f.import_a);
    let outcome = f.accept(&f.batch(key, vec![observation]));
    assert_eq!(outcome.rows[0].classification, Class::Rejected, "{code}");
    assert_eq!(outcome.rows[0].reason_code, Some(code), "{code}");
    let after = f.snapshot();
    assert_eq!(
        after.records, before.records,
        "{code} must create no record"
    );
    assert_eq!(after.deltas, before.deltas, "{code} must create no delta");
    assert_eq!(after.provenance, before.provenance + 1, "{code}");
    assert_eq!(after.errors, before.errors + 1, "{code}");
    assert_eq!(coded_errors(f), coded_before + 1, "{code} field {field}");
    let [received, accepted, duplicate, revision, conflict, invalid] = counters_before;
    assert_eq!(
        f.counters(f.import_a),
        [
            received + 1,
            accepted,
            duplicate,
            revision,
            conflict,
            invalid + 1
        ],
        "{code} must increment exactly received and invalid"
    );
}

#[test]
fn per_observation_input_rejections_cover_the_approved_inventory() {
    let (_guard, f) = setup_fixture();
    let o = |f: &Fixture| f.observation();
    let net_units = || NormalizedMetricObservation {
        measure_code: "net_units".into(),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        reporting_grain: MetricReportingGrain::Month,
        methodology_version: "sales/1".into(),
        ..o(&f)
    };
    let cases: Vec<(NormalizedMetricObservation, Code, &str)> = vec![
        (
            NormalizedMetricObservation {
                source_account_code: "acct-b".into(),
                ..o(&f)
            },
            Code::SourceAccountMismatch,
            "source_account_code",
        ),
        (
            NormalizedMetricObservation {
                source_account_code: "acct-a ".into(),
                ..o(&f)
            },
            Code::SourceAccountMismatch,
            "source_account_code",
        ),
        (
            NormalizedMetricObservation {
                platform_code: "CF".into(),
                ..o(&f)
            },
            Code::PlatformMismatch,
            "platform_code",
        ),
        (
            NormalizedMetricObservation {
                measure_code: "Title_Sessions".into(),
                ..o(&f)
            },
            Code::MeasureNotFound,
            "measure_code",
        ),
        (
            NormalizedMetricObservation {
                measure_code: "orphan".into(),
                ..o(&f)
            },
            Code::PlatformMeasureNotFound,
            "measure_code",
        ),
        (
            NormalizedMetricObservation {
                reporting_grain: MetricReportingGrain::Day,
                period_start: date(2026, 3, 1),
                period_end: date(2026, 3, 2),
                ..net_units()
            },
            Code::UnsupportedReportingGrain,
            "reporting_grain",
        ),
        (
            NormalizedMetricObservation {
                period_end: date(2026, 3, 3),
                ..o(&f)
            },
            Code::InvalidReportingGrainPeriod,
            "period_start",
        ),
        (
            NormalizedMetricObservation {
                period_start: date(2026, 3, 2),
                period_end: date(2026, 4, 1),
                ..net_units()
            },
            Code::InvalidReportingGrainPeriod,
            "period_start",
        ),
        (
            NormalizedMetricObservation {
                reporting_grain: MetricReportingGrain::ReportingPeriod,
                period_end: date(2026, 3, 1),
                ..o(&f)
            },
            Code::InvalidReportingGrainPeriod,
            "period_start",
        ),
        (
            NormalizedMetricObservation {
                publication_isbn: Some(PDF_ISBN.into()),
                ..net_units()
            },
            Code::UnsupportedPublicationDimension,
            "publication_isbn",
        ),
        (
            NormalizedMetricObservation {
                publication_type: Some(PublicationType::Pdf),
                ..net_units()
            },
            Code::UnsupportedPublicationDimension,
            "publication_type",
        ),
        (
            NormalizedMetricObservation {
                country_code: Some("GB".into()),
                ..net_units()
            },
            Code::UnsupportedCountryDimension,
            "country_code",
        ),
        (
            NormalizedMetricObservation {
                institution_ror: Some(ROR.into()),
                ..net_units()
            },
            Code::UnsupportedInstitutionDimension,
            "institution_ror",
        ),
        (
            NormalizedMetricObservation {
                country_code: Some("gb".into()),
                ..o(&f)
            },
            Code::InvalidCountry,
            "country_code",
        ),
        (
            NormalizedMetricObservation {
                country_code: Some("XK".into()),
                ..o(&f)
            },
            Code::InvalidCountry,
            "country_code",
        ),
        (
            NormalizedMetricObservation {
                country_code: Some("GBR".into()),
                ..o(&f)
            },
            Code::InvalidCountry,
            "country_code",
        ),
        (
            NormalizedMetricObservation { value: -1, ..o(&f) },
            Code::InvalidValue,
            "value",
        ),
        (
            NormalizedMetricObservation {
                methodology_version: " \t".into(),
                ..o(&f)
            },
            Code::MethodologyRequired,
            "methodology_version",
        ),
        (
            NormalizedMetricObservation {
                methodology_version: "cloudfront-title-session/1".into(),
                ..o(&f)
            },
            Code::MethodologyMismatch,
            "methodology_version",
        ),
        (
            NormalizedMetricObservation {
                methodology_version: " cloudfront-title-session/2".into(),
                ..o(&f)
            },
            Code::MethodologyMismatch,
            "methodology_version",
        ),
        (
            NormalizedMetricObservation {
                work_doi: "not-a-doi".into(),
                ..o(&f)
            },
            Code::InvalidDoi,
            "work_doi",
        ),
        (
            NormalizedMetricObservation {
                work_doi: "https://doi.org/10.12345/missing".into(),
                ..o(&f)
            },
            Code::UnknownDoi,
            "work_doi",
        ),
        (
            NormalizedMetricObservation {
                publication_isbn: Some("978-3-16-148410-1".into()),
                ..o(&f)
            },
            Code::InvalidIsbn,
            "publication_isbn",
        ),
        (
            NormalizedMetricObservation {
                publication_isbn: Some(OTHER_ISBN.into()),
                ..o(&f)
            },
            Code::UnknownPublication,
            "publication_isbn",
        ),
        (
            NormalizedMetricObservation {
                publication_type: Some(PublicationType::Hardback),
                ..o(&f)
            },
            Code::UnknownPublication,
            "publication_type",
        ),
        (
            NormalizedMetricObservation {
                institution_ror: Some("https://ror.org/nope".into()),
                ..o(&f)
            },
            Code::InvalidRor,
            "institution_ror",
        ),
        (
            NormalizedMetricObservation {
                institution_ror: Some(OTHER_ROR.into()),
                ..o(&f)
            },
            Code::UnknownRor,
            "institution_ror",
        ),
    ];
    for (index, (observation, code, field)) in cases.into_iter().enumerate() {
        assert_rejected(&f, &format!("reject-{index}"), observation, code, field);
    }
    // Zero is a valid value and the grain/period rules accept the boundaries.
    let zero = f.accept(&f.batch(
        "zero",
        vec![NormalizedMetricObservation {
            value: 0,
            period_start: date(2026, 12, 31),
            period_end: date(2027, 1, 1),
            ..o(&f)
        }],
    ));
    assert_eq!(zero.rows[0].classification, Class::Winner);
    assert_eq!(
        count(&f.pool, "institution", "TRUE"),
        1,
        "no institution is ever created"
    );
}

#[test]
fn disabled_registry_state_rejects_every_affected_observation() {
    let (_guard, f) = setup_fixture();
    let cases: [(&str, &str, Code, &str); 4] = [
        (
            "UPDATE metric_platform SET enabled = FALSE",
            "UPDATE metric_platform SET enabled = TRUE",
            Code::PlatformDisabled,
            "platform_code",
        ),
        (
            "UPDATE metric_measure SET enabled = FALSE WHERE code = 'title_sessions'",
            "UPDATE metric_measure SET enabled = TRUE WHERE code = 'title_sessions'",
            Code::MeasureDisabled,
            "measure_code",
        ),
        (
            "UPDATE metric_platform_measure SET enabled = FALSE",
            "UPDATE metric_platform_measure SET enabled = TRUE",
            Code::PlatformMeasureDisabled,
            "measure_code",
        ),
        (
            "UPDATE metric_platform_measure SET direct_collection = FALSE",
            "UPDATE metric_platform_measure SET direct_collection = TRUE",
            Code::NotDirectCollection,
            "measure_code",
        ),
    ];
    for (index, (disable, restore, code, field)) in cases.into_iter().enumerate() {
        f.sql(disable);
        let outcome = f.accept(&f.batch(
            &format!("disabled-{index}"),
            vec![
                f.observation(),
                NormalizedMetricObservation {
                    period_start: date(2026, 3, 2),
                    period_end: date(2026, 3, 3),
                    ..f.observation()
                },
            ],
        ));
        assert!(
            outcome
                .rows
                .iter()
                .all(|row| row.classification == Class::Rejected && row.reason_code == Some(code)),
            "{code}"
        );
        assert_eq!(
            count(
                &f.pool,
                "metric_import_error",
                &format!("error_code = '{code}' AND field_name = '{field}'")
            ),
            2
        );
        f.sql(restore);
    }
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
    assert_eq!(f.counters(f.import_a), [8, 0, 0, 0, 0, 8]);
}

// ---------------------------------------------------------------------------
// Request-level authority
// ---------------------------------------------------------------------------

#[test]
fn request_level_authority_failures_commit_nothing() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch("auth", vec![f.observation()]);
    let cases: Vec<(String, String, Code)> = vec![
        (format!("UPDATE metric_source_account SET enabled = FALSE WHERE source_account_id = '{}'", f.account_a), format!("UPDATE metric_source_account SET enabled = TRUE WHERE source_account_id = '{}'", f.account_a), Code::SourceAccountDisabled),
        ("UPDATE metric_source SET enabled = FALSE".into(), "UPDATE metric_source SET enabled = TRUE".into(), Code::SourceDisabled),
        ("UPDATE metric_source SET acquisition_type = 'PUBLISHER_UPLOAD', driver_key = NULL".into(), "UPDATE metric_source SET acquisition_type = 'DRIVER', driver_key = 'cloudfront'".into(), Code::AcquisitionTypeDeferred),
        ("UPDATE metric_source SET acquisition_type = 'OPERAS', driver_key = NULL".into(), "UPDATE metric_source SET acquisition_type = 'DRIVER', driver_key = 'cloudfront'".into(), Code::AcquisitionTypeDeferred),
        ("UPDATE metric_source SET acquisition_type = 'ADMIN_IMPORT', driver_key = NULL".into(), "UPDATE metric_source SET acquisition_type = 'DRIVER', driver_key = 'cloudfront'".into(), Code::AcquisitionTypeDeferred),
        (format!("UPDATE metric_source_account SET expected_publisher_id = NULL WHERE source_account_id = '{}'", f.account_a), format!("UPDATE metric_source_account SET expected_publisher_id = '{}' WHERE source_account_id = '{}'", f.publisher_id, f.account_a), Code::PublisherScopeMismatch),
        (format!("UPDATE metric_import SET publisher_id = NULL WHERE import_id = '{}'", f.import_a), format!("UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{}'", f.publisher_id, f.import_a), Code::PublisherScopeMismatch),
        (format!("UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{}'", f.other_publisher_id, f.import_a), format!("UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{}'", f.publisher_id, f.import_a), Code::PublisherScopeMismatch),
        (format!("UPDATE publisher SET subscription_package = 'OASIS' WHERE publisher_id = '{}'", f.publisher_id), format!("UPDATE publisher SET subscription_package = 'OBELISK' WHERE publisher_id = '{}'", f.publisher_id), Code::MetricsCollectNotEntitled),
        (format!("UPDATE metric_import SET status = 'QUEUED' WHERE import_id = '{}'", f.import_a), format!("UPDATE metric_import SET status = 'PROCESSING' WHERE import_id = '{}'", f.import_a), Code::ImportNotProcessing),
        (format!("UPDATE metric_import SET status = 'COMPLETED' WHERE import_id = '{}'", f.import_a), format!("UPDATE metric_import SET status = 'PROCESSING' WHERE import_id = '{}'", f.import_a), Code::ImportNotProcessing),
    ];
    for (mutate, restore, code) in cases {
        f.sql(&mutate);
        assert_request_failure(&f, &batch, code);
        f.sql(&restore);
    }
    assert_request_failure(
        &f,
        &MetricIngestionBatch {
            import_id: Uuid::new_v4(),
            ..batch.clone()
        },
        Code::ImportNotFound,
    );
    // Every package with METRICS_COLLECT is accepted after the restores.
    for package in ["OBELISK", "SPHINX", "PYRAMID"] {
        f.sql(&format!(
            "UPDATE publisher SET subscription_package = '{package}' WHERE publisher_id = '{}'",
            f.publisher_id
        ));
        let outcome = f.accept(&f.batch(&format!("pkg-{package}"), vec![f.observation()]));
        assert_ne!(outcome.rows[0].classification, Class::Rejected);
    }
}

#[test]
fn a_work_owned_by_another_publisher_is_rejected_per_observation() {
    let (_guard, f) = setup_fixture();
    f.sql(&format!(
        "UPDATE work SET imprint_id = '{}' WHERE work_id = '{}'",
        f.other_imprint_id, f.other_work_id
    ));
    let foreign = NormalizedMetricObservation {
        work_doi: OTHER_DOI.into(),
        ..f.observation()
    };
    let outcome = f.accept(&f.batch("b1", vec![foreign, f.observation()]));
    assert_eq!(outcome.rows[0].classification, Class::Rejected);
    assert_eq!(
        outcome.rows[0].reason_code,
        Some(Code::PublisherScopeMismatch)
    );
    assert!(outcome.rows[0].identity_hash.is_some() && outcome.rows[0].content_hash.is_some());
    assert_eq!(outcome.rows[1].classification, Class::Winner);
    assert_eq!(count(&f.pool, "metric_record_provenance", "classification = 'REJECTED' AND identity_hash IS NOT NULL AND content_hash IS NOT NULL AND details->>'reason_code' = 'PUBLISHER_SCOPE_MISMATCH'"), 1);
    assert_eq!(
        count(
            &f.pool,
            "metric_import_error",
            "error_code = 'PUBLISHER_SCOPE_MISMATCH' AND field_name = 'work_doi'"
        ),
        1
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
}

// ---------------------------------------------------------------------------
// Identifier resolution
// ---------------------------------------------------------------------------

#[test]
fn publication_resolution_uses_isbn_precedence_type_fallback_and_real_ambiguity() {
    let (_guard, f) = setup_fixture();
    let by_isbn_and_wrong_type = NormalizedMetricObservation {
        publication_isbn: Some(PDF_ISBN.into()),
        publication_type: Some(PublicationType::Hardback),
        ..f.observation()
    };
    let by_type = NormalizedMetricObservation {
        publication_type: Some(PublicationType::Pdf),
        period_start: date(2026, 3, 2),
        period_end: date(2026, 3, 3),
        ..f.observation()
    };
    let by_hyphenless_isbn = NormalizedMetricObservation {
        publication_isbn: Some(PDF_ISBN.replace('-', "")),
        period_start: date(2026, 3, 3),
        period_end: date(2026, 3, 4),
        ..f.observation()
    };
    let outcome = f.accept(&f.batch(
        "b1",
        vec![by_isbn_and_wrong_type, by_type, by_hyphenless_isbn],
    ));
    assert!(
        outcome
            .rows
            .iter()
            .all(|row| row.classification == Class::Winner),
        "{outcome:?}"
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("publication_id = '{}'", f.publication_id)
        ),
        3
    );
    // The hyphenless form normalizes to the same canonical publication but
    // is a different request payload, so it is a distinct batch request.
    assert_ne!(
        request_hash(&f.batch(
            "x",
            vec![NormalizedMetricObservation {
                publication_isbn: Some(PDF_ISBN.into()),
                ..f.observation()
            }]
        )),
        request_hash(&f.batch(
            "x",
            vec![NormalizedMetricObservation {
                publication_isbn: Some(PDF_ISBN.replace('-', "")),
                ..f.observation()
            }]
        ))
    );

    // Real database ambiguity: the schema permits two publications of one
    // work, of different types, to share an ISBN.
    f.sql(&format!("INSERT INTO publication (publication_type, work_id, isbn) VALUES ('Epub', '{}', '{PDF_ISBN}')", f.work_id));
    assert_rejected(
        &f,
        "amb",
        NormalizedMetricObservation {
            publication_isbn: Some(PDF_ISBN.into()),
            period_start: date(2026, 4, 1),
            period_end: date(2026, 4, 2),
            ..f.observation()
        },
        Code::AmbiguousPublication,
        "publication_isbn",
    );
    // The publication must belong to the resolved work.
    f.sql(&format!("INSERT INTO publication (publication_type, work_id, isbn) VALUES ('PDF', '{}', '{OTHER_ISBN}')", f.other_work_id));
    assert_rejected(
        &f,
        "other-work",
        NormalizedMetricObservation {
            publication_isbn: Some(OTHER_ISBN.into()),
            ..f.observation()
        },
        Code::UnknownPublication,
        "publication_isbn",
    );
}

#[test]
fn ror_resolution_accepts_canonical_forms_and_detects_real_ambiguity_without_creating_institutions()
{
    let (_guard, f) = setup_fixture();
    let with_ror = |ror: &str, day: u32| NormalizedMetricObservation {
        institution_ror: Some(ror.into()),
        period_start: date(2026, 3, day),
        period_end: date(2026, 3, day + 1),
        ..f.observation()
    };
    let outcome = f.accept(&f.batch(
        "b1",
        vec![
            with_ror(ROR, 1),
            with_ror("02mhbdp94", 2),
            with_ror("ror.org/02mhbdp94", 3),
        ],
    ));
    assert!(
        outcome
            .rows
            .iter()
            .all(|row| row.classification == Class::Winner),
        "{outcome:?}"
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("institution_id = '{}'", f.institution_id)
        ),
        3
    );
    f.sql(&format!(
        "INSERT INTO institution (institution_name, ror) VALUES ('Twin', '{ROR}')"
    ));
    assert_rejected(
        &f,
        "amb",
        with_ror(ROR, 10),
        Code::AmbiguousRor,
        "institution_ror",
    );
    assert_eq!(count(&f.pool, "institution", "TRUE"), 2);
}

// ---------------------------------------------------------------------------
// Coverage
// ---------------------------------------------------------------------------

fn coverage(status: MetricCoverageStatus) -> NormalizedMetricCoverageAssertion {
    NormalizedMetricCoverageAssertion {
        platform_code: PLATFORM_CODE.into(),
        measure_code: "title_sessions".into(),
        period_start: date(2026, 3, 1),
        period_end: date(2026, 4, 1),
        status,
        country_coverage: true,
        institution_coverage: true,
        notes: Some("complete March feed".into()),
    }
}

#[test]
fn coverage_and_observations_commit_atomically_with_exact_coverage_rows() {
    let (_guard, f) = setup_fixture();
    let batch = MetricIngestionBatch {
        coverage: vec![
            coverage(MetricCoverageStatus::Complete),
            NormalizedMetricCoverageAssertion {
                measure_code: "net_units".into(),
                country_coverage: false,
                institution_coverage: false,
                notes: None,
                ..coverage(MetricCoverageStatus::Partial)
            },
        ],
        ..f.batch("b1", vec![f.observation()])
    };
    let outcome = f.accept(&batch);
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    assert_eq!(count(&f.pool, "metric_coverage", &format!("source_account_id = '{}' AND import_id = '{}' AND platform_id = '{}' AND measure_id = '{}' AND period_start = DATE '2026-03-01' AND period_end = DATE '2026-04-01' AND coverage_status = 'COMPLETE' AND country_coverage AND institution_coverage AND notes = 'complete March feed'", f.account_a, f.import_a, f.platform_id, f.title_sessions_id)), 1);
    assert_eq!(count(&f.pool, "metric_coverage", &format!("measure_id = '{}' AND coverage_status = 'PARTIAL' AND NOT country_coverage AND NOT institution_coverage AND notes IS NULL", f.net_units_id)), 1);
    assert_eq!(count(&f.pool, "metric_coverage", "TRUE"), 2);
    assert_eq!(
        f.counters(f.import_a),
        [1, 1, 0, 0, 0, 0],
        "coverage never touches row counters"
    );
}

#[test]
fn a_coverage_only_batch_commits_coverage_and_a_batch_row_but_no_row_state() {
    let (_guard, f) = setup_fixture();
    let batch = MetricIngestionBatch {
        coverage: vec![coverage(MetricCoverageStatus::Unknown)],
        ..f.batch("cov", vec![])
    };
    let outcome = f.accept(&batch);
    assert!(outcome.rows.is_empty());
    let state = f.snapshot();
    assert_eq!(
        (
            state.batches,
            state.coverage,
            state.provenance,
            state.errors,
            state.records,
            state.deltas
        ),
        (1, 1, 0, 0, 0, 0)
    );
    assert_eq!(f.counters(f.import_a), [0, 0, 0, 0, 0, 0]);
    assert_eq!(
        count(&f.pool, "metric_coverage", "coverage_status = 'UNKNOWN'"),
        1
    );
    // A replay creates no second coverage row.
    let replay = f.accept(&batch);
    assert!(replay.replayed);
    assert_eq!(replay.import_batch_id, outcome.import_batch_id);
    assert_eq!(f.snapshot(), state);
}

#[test]
fn coverage_status_round_trips_all_three_values() {
    let (_guard, f) = setup_fixture();
    for (index, status) in [
        MetricCoverageStatus::Complete,
        MetricCoverageStatus::Partial,
        MetricCoverageStatus::Unknown,
    ]
    .into_iter()
    .enumerate()
    {
        f.accept(&MetricIngestionBatch {
            coverage: vec![coverage(status)],
            ..f.batch(&format!("cov-{index}"), vec![])
        });
        assert_eq!(
            count(
                &f.pool,
                "metric_coverage",
                &format!("coverage_status = '{}'", enum_code(&status))
            ),
            1
        );
    }
    assert_eq!(
        count(&f.pool, "metric_coverage", "TRUE"),
        3,
        "different batch keys are distinct durable submissions"
    );
}

#[test]
fn any_invalid_coverage_assertion_rolls_back_the_whole_first_time_batch() {
    let (_guard, f) = setup_fixture();
    let cases: Vec<(
        NormalizedMetricCoverageAssertion,
        Option<&str>,
        Option<&str>,
        Code,
    )> = vec![
        (
            NormalizedMetricCoverageAssertion {
                platform_code: "other".into(),
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::PlatformMismatch,
        ),
        (
            coverage(MetricCoverageStatus::Complete),
            Some("UPDATE metric_platform SET enabled = FALSE"),
            Some("UPDATE metric_platform SET enabled = TRUE"),
            Code::PlatformDisabled,
        ),
        (
            NormalizedMetricCoverageAssertion {
                measure_code: "missing".into(),
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::MeasureNotFound,
        ),
        (
            coverage(MetricCoverageStatus::Complete),
            Some("UPDATE metric_measure SET enabled = FALSE WHERE code = 'title_sessions'"),
            Some("UPDATE metric_measure SET enabled = TRUE WHERE code = 'title_sessions'"),
            Code::MeasureDisabled,
        ),
        (
            NormalizedMetricCoverageAssertion {
                measure_code: "orphan".into(),
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::PlatformMeasureNotFound,
        ),
        (
            coverage(MetricCoverageStatus::Complete),
            Some("UPDATE metric_platform_measure SET enabled = FALSE"),
            Some("UPDATE metric_platform_measure SET enabled = TRUE"),
            Code::PlatformMeasureDisabled,
        ),
        (
            coverage(MetricCoverageStatus::Complete),
            Some("UPDATE metric_platform_measure SET direct_collection = FALSE"),
            Some("UPDATE metric_platform_measure SET direct_collection = TRUE"),
            Code::NotDirectCollection,
        ),
        (
            NormalizedMetricCoverageAssertion {
                period_end: date(2026, 3, 1),
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::InvalidCoveragePeriod,
        ),
        (
            NormalizedMetricCoverageAssertion {
                measure_code: "net_units".into(),
                country_coverage: true,
                institution_coverage: false,
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::InvalidCoverageDimension,
        ),
        (
            NormalizedMetricCoverageAssertion {
                measure_code: "net_units".into(),
                country_coverage: false,
                institution_coverage: true,
                ..coverage(MetricCoverageStatus::Complete)
            },
            None,
            None,
            Code::InvalidCoverageDimension,
        ),
    ];
    for (assertion, mutate, restore, code) in cases {
        if let Some(mutate) = mutate {
            f.sql(mutate);
        }
        let batch = MetricIngestionBatch {
            coverage: vec![coverage(MetricCoverageStatus::Complete), assertion],
            ..f.batch("cov", vec![f.observation()])
        };
        assert_request_failure(&f, &batch, code);
        if let Some(restore) = restore {
            f.sql(restore);
        }
    }
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 0);
}

#[test]
fn batch_limits_and_key_length_are_enforced_exactly() {
    let (_guard, f) = setup_fixture();
    let observations: Vec<NormalizedMetricObservation> = (0..MAX_OBSERVATIONS as i64)
        .map(|day| {
            let start = date(2020, 1, 1) + chrono::Duration::days(day);
            NormalizedMetricObservation {
                period_start: start,
                period_end: start.succ_opt().unwrap(),
                ..f.observation()
            }
        })
        .collect();
    let coverage_rows: Vec<NormalizedMetricCoverageAssertion> = (0..MAX_COVERAGE_ASSERTIONS as i64)
        .map(|day| {
            let start = date(2020, 1, 1) + chrono::Duration::days(day);
            NormalizedMetricCoverageAssertion {
                period_start: start,
                period_end: start.succ_opt().unwrap(),
                ..coverage(MetricCoverageStatus::Complete)
            }
        })
        .collect();

    let mut over_observations = observations.clone();
    over_observations.push(NormalizedMetricObservation {
        period_start: date(2030, 1, 1),
        period_end: date(2030, 1, 2),
        ..f.observation()
    });
    assert_request_failure(
        &f,
        &f.batch("over", over_observations),
        Code::BatchLimitExceeded,
    );
    let mut over_coverage = coverage_rows.clone();
    over_coverage.push(coverage(MetricCoverageStatus::Complete));
    assert_request_failure(
        &f,
        &MetricIngestionBatch {
            coverage: over_coverage,
            ..f.batch("over", vec![])
        },
        Code::BatchLimitExceeded,
    );
    assert_request_failure(
        &f,
        &f.batch(&"k".repeat(MAX_BATCH_KEY_BYTES + 1), vec![f.observation()]),
        Code::InvalidBatchKey,
    );

    let full = f.accept(&MetricIngestionBatch {
        coverage: coverage_rows,
        ..f.batch(&"k".repeat(MAX_BATCH_KEY_BYTES), observations)
    });
    assert_eq!(full.rows.len(), MAX_OBSERVATIONS);
    assert!(full
        .rows
        .iter()
        .all(|row| row.classification == Class::Winner));
    assert_eq!(
        count(&f.pool, "metric_record", "TRUE"),
        MAX_OBSERVATIONS as i64
    );
    assert_eq!(
        count(&f.pool, "metric_coverage", "TRUE"),
        MAX_COVERAGE_ASSERTIONS as i64
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_import_batch",
            &format!("octet_length(batch_key) = {MAX_BATCH_KEY_BYTES}")
        ),
        1
    );
    assert_eq!(f.counters(f.import_a), [500, 500, 0, 0, 0, 0]);
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

#[test]
fn an_exact_replay_returns_the_committed_outcome_and_writes_nothing() {
    let (_guard, f) = setup_fixture();
    let invalid = NormalizedMetricObservation {
        work_doi: "bad".into(),
        ..f.observation()
    };
    let batch = MetricIngestionBatch {
        coverage: vec![coverage(MetricCoverageStatus::Complete)],
        ..f.batch("b1", vec![f.observation(), invalid, f.observation()])
    };
    let first = f.accept(&batch);
    let state = f.snapshot();
    // Immediate replay, replay from a fresh pool (restart equivalent), and
    // an uncertain-timeout-after-commit retry all read the same outcome.
    let fresh_pool = pool_of(2);
    for replay in [
        f.accept(&batch),
        ingest_metric_batch(&fresh_pool, &batch).unwrap(),
        f.accept(&batch),
    ] {
        assert!(replay.replayed);
        assert_eq!(replay.import_batch_id, first.import_batch_id);
        assert_eq!(replay.request_hash, first.request_hash);
        assert_eq!(replay.rows, first.rows);
        assert_eq!(f.snapshot(), state);
    }
    assert_eq!(
        first
            .rows
            .iter()
            .map(|row| row.classification)
            .collect::<Vec<_>>(),
        vec![Class::Winner, Class::Rejected, Class::Duplicate]
    );
    assert_eq!(first.rows[1].reason_code, Some(Code::InvalidDoi));
    assert_eq!(f.counters(f.import_a), [3, 1, 1, 0, 0, 1]);
}

#[test]
fn replay_survives_import_completion_and_configuration_changes() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch("b1", vec![f.observation()]);
    let first = f.accept(&batch);
    let state = f.snapshot();
    f.sql(&format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = CURRENT_TIMESTAMP WHERE import_id = '{}'", f.import_a));
    f.sql("UPDATE metric_platform SET enabled = FALSE");
    f.sql(&format!(
        "UPDATE metric_source_account SET enabled = FALSE WHERE source_account_id = '{}'",
        f.account_a
    ));
    f.sql(
        "UPDATE metric_source SET acquisition_type = 'ADMIN_IMPORT', driver_key = NULL, enabled = FALSE",
    );
    f.sql(&format!(
        "UPDATE publisher SET subscription_package = 'OASIS' WHERE publisher_id = '{}'",
        f.publisher_id
    ));
    let replay = f.accept(&batch);
    assert!(replay.replayed);
    assert_eq!(replay.rows, first.rows);
    assert_eq!(f.snapshot(), state);
    // A new first-time batch under the same import now fails closed.
    assert_request_failure(
        &f,
        &f.batch("b2", vec![f.observation()]),
        Code::ImportNotProcessing,
    );
}

#[test]
fn the_same_key_with_a_different_payload_is_idempotency_key_reused() {
    let (_guard, f) = setup_fixture();
    f.accept(&f.batch("b1", vec![f.observation()]));
    let different = f.batch(
        "b1",
        vec![NormalizedMetricObservation {
            value: 11,
            ..f.observation()
        }],
    );
    assert_request_failure(&f, &different, Code::IdempotencyKeyReused);
    // Semantically equivalent but differently supplied input is also reuse.
    let hyphenless = f.batch(
        "b1",
        vec![NormalizedMetricObservation {
            work_doi: WORK_DOI.to_uppercase(),
            ..f.observation()
        }],
    );
    assert_request_failure(&f, &hyphenless, Code::IdempotencyKeyReused);
    // The same key under another import is a different batch.
    let outcome = f.accept(&f.batch_b("b1", vec![f.observation_b()]));
    assert_eq!(outcome.rows[0].classification, Class::Duplicate);
}

#[test]
fn inconsistent_persisted_batch_state_fails_replay_closed() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch(
        "b1",
        vec![
            f.observation(),
            NormalizedMetricObservation {
                period_start: date(2026, 3, 2),
                period_end: date(2026, 3, 3),
                ..f.observation()
            },
        ],
    );
    let first = f.accept(&batch);
    // A corrupted reason code.
    f.sql(&format!("UPDATE metric_record_provenance SET details = jsonb_set(details, '{{reason_code}}', '\"NOT_A_CODE\"') WHERE import_batch_id = '{}' AND batch_row_index = 1", first.import_batch_id));
    assert_request_failure(&f, &batch, Code::InternalStateInconsistency);
    f.sql(&format!("UPDATE metric_record_provenance SET details = jsonb_set(details, '{{reason_code}}', 'null') WHERE import_batch_id = '{}' AND batch_row_index = 1", first.import_batch_id));
    assert!(f.accept(&batch).replayed);
    // A missing row.
    f.sql(&format!("UPDATE metric_record_provenance SET import_batch_id = NULL, batch_row_index = NULL WHERE import_batch_id = '{}' AND batch_row_index = 1", first.import_batch_id));
    assert_request_failure(&f, &batch, Code::InternalStateInconsistency);
}

#[test]
fn import_source_scope_mismatch_is_reserved_and_never_emitted() {
    // Amendment 5 D1: the code exists in the closed vocabulary but the DRIVER
    // coordinator has no emission condition for it. The coordinator source
    // itself must not construct it.
    assert_eq!(
        Code::ImportSourceScopeMismatch.to_string(),
        "IMPORT_SOURCE_SCOPE_MISMATCH"
    );
    let coordinator = include_str!("mod.rs");
    assert!(!coordinator.contains("ImportSourceScopeMismatch"));
    assert!(!coordinator.contains("ConflictingFinalRecord"));
}

// ---------------------------------------------------------------------------
// Failure atomicity (Amendment 5 D5)
// ---------------------------------------------------------------------------

fn assert_database_failure_leaves_nothing(f: &Fixture, batch: &MetricIngestionBatch) {
    let before = f.snapshot();
    assert_eq!(
        f.ingest(batch),
        Err(MetricIngestionError::new(Code::InternalDatabaseError))
    );
    assert_eq!(f.snapshot(), before);
}

#[test]
fn a_record_is_never_committed_without_its_revision() {
    let (_guard, f) = setup_fixture();
    let _trigger = TestTrigger::failing(&f.pool, "BEFORE INSERT", "metric_record_revision", None);
    assert_database_failure_leaves_nothing(&f, &f.batch("b1", vec![f.observation()]));
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn a_revision_is_never_committed_without_the_pointer_move() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let _trigger = TestTrigger::failing(
        &f.pool,
        "BEFORE UPDATE OF current_revision_id",
        "metric_record",
        None,
    );
    assert_database_failure_leaves_nothing(
        &f,
        &f.batch(
            "b2",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}' AND status = 'CURRENT' AND revision_number = 1")
        ),
        1
    );
    assert_eq!(f.current_value(record_id), 10);
    drop(_trigger);
    // The same revision commits once the injected failure is gone.
    assert_eq!(
        f.accept(&f.batch(
            "b2",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }]
        ))
        .rows[0]
            .classification,
        Class::Revision
    );
}

#[test]
fn a_revision_is_never_committed_without_its_provenance() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let _trigger = TestTrigger::failing(
        &f.pool,
        "BEFORE INSERT",
        "metric_record_provenance",
        Some("NEW.classification = 'REVISION'"),
    );
    assert_database_failure_leaves_nothing(
        &f,
        &f.batch(
            "b2",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}'")
        ),
        1
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}' AND status = 'SUPERSEDED'")
        ),
        0
    );
}

#[test]
fn a_revision_is_never_committed_without_its_rollup_delta() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("b1", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let _trigger = TestTrigger::failing(&f.pool, "BEFORE INSERT", "metric_rollup_delta", None);
    assert_database_failure_leaves_nothing(
        &f,
        &f.batch(
            "b2",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_rollup_delta",
            &format!("record_id = '{record_id}'")
        ),
        1
    );
    assert_eq!(f.current_value(record_id), 10);
    // A first arrival without its delta rolls back too.
    assert_database_failure_leaves_nothing(
        &f,
        &f.batch(
            "b3",
            vec![NormalizedMetricObservation {
                period_start: date(2026, 3, 2),
                period_end: date(2026, 3, 3),
                ..f.observation()
            }],
        ),
    );
}

#[test]
fn counter_mutation_is_never_committed_without_the_durable_outcome() {
    let (_guard, f) = setup_fixture();
    // Raised at COMMIT, after every write of the attempt has been issued.
    let _trigger = TestTrigger::failing_at_commit(&f.pool, "UPDATE", "metric_import");
    assert_database_failure_leaves_nothing(
        &f,
        &MetricIngestionBatch {
            coverage: vec![coverage(MetricCoverageStatus::Complete)],
            ..f.batch(
                "b1",
                vec![
                    f.observation(),
                    NormalizedMetricObservation {
                        work_doi: "bad".into(),
                        ..f.observation()
                    },
                ],
            )
        },
    );
    assert_eq!(f.counters(f.import_a), [0, 0, 0, 0, 0, 0]);
}

#[test]
fn a_batch_row_is_never_committed_without_its_canonical_consequences() {
    let (_guard, f) = setup_fixture();
    let _trigger = TestTrigger::failing(&f.pool, "BEFORE INSERT", "metric_record_provenance", None);
    assert_database_failure_leaves_nothing(&f, &f.batch("b1", vec![f.observation()]));
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 0);
}

#[test]
fn coverage_is_never_committed_without_the_batch_and_canonical_commit() {
    let (_guard, f) = setup_fixture();
    let _trigger = TestTrigger::failing_at_commit(&f.pool, "INSERT", "metric_coverage");
    assert_database_failure_leaves_nothing(
        &f,
        &MetricIngestionBatch {
            coverage: vec![coverage(MetricCoverageStatus::Complete)],
            ..f.batch("b1", vec![f.observation()])
        },
    );
    assert_eq!(count(&f.pool, "metric_coverage", "TRUE"), 0);
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 0);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
    drop(_trigger);
    assert!(f
        .accept(&MetricIngestionBatch {
            coverage: vec![coverage(MetricCoverageStatus::Complete)],
            ..f.batch("b1", vec![f.observation()])
        })
        .rows[0]
        .record_id
        .is_some());
}

// ---------------------------------------------------------------------------
// Real multi-connection PostgreSQL races: final persisted state only
// ---------------------------------------------------------------------------

/// Two concurrent coordinator calls on independent pooled connections.
fn race(
    f: &Fixture,
    one: MetricIngestionBatch,
    two: MetricIngestionBatch,
) -> (
    Result<MetricIngestionOutcome, MetricIngestionError>,
    Result<MetricIngestionOutcome, MetricIngestionError>,
) {
    let pool = pool_of(4);
    let pool_one = Arc::clone(&pool);
    let pool_two = Arc::clone(&pool);
    let deadlocks_before = deadlocks(&f.pool);
    let outcomes = run_concurrently(
        move || ingest_metric_batch(&pool_one, &one),
        move || ingest_metric_batch(&pool_two, &two),
    );
    assert_eq!(
        deadlocks(&f.pool),
        deadlocks_before,
        "the coordinator must not create deadlocks"
    );
    outcomes
}

fn classes(outcome: &MetricIngestionOutcome) -> Vec<Class> {
    outcome.rows.iter().map(|row| row.classification).collect()
}

#[test]
fn simultaneous_first_arrival_from_two_accounts_yields_one_record_one_winner_one_duplicate() {
    let (_guard, f) = setup_fixture();
    let (a, b) = race(
        &f,
        f.batch("b1", vec![f.observation()]),
        f.batch_b("b1", vec![f.observation_b()]),
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    let mut all = [classes(&a), classes(&b)];
    all.sort_by_key(|classes| classes.iter().map(ToString::to_string).collect::<Vec<_>>());
    assert_eq!(all, [vec![Class::Duplicate], vec![Class::Winner]]);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record_revision", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_rollup_delta", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record_provenance", "TRUE"), 2);
    let winner_import = text(
        &f.pool,
        "(SELECT import_id::text FROM metric_record_provenance WHERE classification = 'WINNER')",
    )
    .unwrap();
    let winner_account = text(
        &f.pool,
        "(SELECT winning_source_account_id::text FROM metric_record)",
    )
    .unwrap();
    let (winner, loser) = if uuid(&winner_import) == f.import_a {
        (f.import_a, f.import_b)
    } else {
        (f.import_b, f.import_a)
    };
    assert_eq!(
        winner_account,
        if winner == f.import_a {
            f.account_a
        } else {
            f.account_b
        }
        .to_string()
    );
    assert_eq!(f.counters(winner), [1, 1, 0, 0, 0, 0]);
    assert_eq!(f.counters(loser), [1, 0, 1, 0, 0, 0]);
}

#[test]
fn simultaneous_first_arrival_and_duplicate_from_one_account_are_consistent() {
    let (_guard, f) = setup_fixture();
    let (a, b) = race(
        &f,
        f.batch("b1", vec![f.observation()]),
        f.batch("b2", vec![f.observation()]),
    );
    let mut all = [classes(&a.unwrap()), classes(&b.unwrap())];
    all.sort_by_key(|classes| classes.iter().map(ToString::to_string).collect::<Vec<_>>());
    assert_eq!(all, [vec![Class::Duplicate], vec![Class::Winner]]);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 2);
    assert_eq!(f.counters(f.import_a), [2, 1, 1, 0, 0, 0]);
}

#[test]
fn simultaneous_differing_winner_revisions_serialize_into_one_current_revision() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("seed", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let (a, b) = race(
        &f,
        f.batch(
            "b1",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
        f.batch(
            "b2",
            vec![NormalizedMetricObservation {
                value: 30,
                ..f.observation()
            }],
        ),
    );
    assert_eq!(classes(&a.unwrap()), vec![Class::Revision]);
    assert_eq!(classes(&b.unwrap()), vec![Class::Revision]);
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}'")
        ),
        3
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}' AND status = 'CURRENT' AND revision_number = 3")
        ),
        1
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}' AND status = 'SUPERSEDED'")
        ),
        2
    );
    assert_eq!(count(&f.pool, "metric_record_revision", &format!("record_id = '{record_id}' AND revision_number = 3 AND supersedes_revision_id = (SELECT record_revision_id FROM metric_record_revision WHERE record_id = '{record_id}' AND revision_number = 2)")), 1);
    let current = f.current_value(record_id);
    assert!(current == 20 || current == 30);
    assert_eq!(scalar_i64(&f.pool, &format!("(SELECT SUM(delta_value)::bigint FROM metric_rollup_delta WHERE record_id = '{record_id}')")), current, "deltas must sum to the current value");
    assert_eq!(
        count(
            &f.pool,
            "metric_rollup_delta",
            &format!("record_id = '{record_id}'")
        ),
        3
    );
    assert_eq!(f.counters(f.import_a), [3, 1, 0, 2, 0, 0]);
}

#[test]
fn simultaneous_revision_and_duplicate_leave_a_consistent_history() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("seed", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let (a, b) = race(
        &f,
        f.batch(
            "b1",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
        f.batch_b("b1", vec![f.observation_b()]),
    );
    assert_eq!(classes(&a.unwrap()), vec![Class::Revision]);
    // Account B's original content is a duplicate if it arrived before the
    // revision and a conflict if after: both are correct, and neither
    // mutates canonical state.
    let b_class = classes(&b.unwrap())[0];
    assert!(
        matches!(b_class, Class::Duplicate | Class::Conflict),
        "{b_class:?}"
    );
    assert_eq!(f.current_value(record_id), 20);
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}'")
        ),
        2
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}' AND status = 'CURRENT'")
        ),
        1
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_rollup_delta",
            &format!("record_id = '{record_id}'")
        ),
        2
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!(
                "record_id = '{record_id}' AND winning_source_account_id = '{}'",
                f.account_a
            )
        ),
        1
    );
}

#[test]
fn simultaneous_winner_revision_and_non_winning_conflict_never_let_the_conflict_mutate() {
    let (_guard, f) = setup_fixture();
    let first = f.accept(&f.batch("seed", vec![f.observation()]));
    let record_id = first.rows[0].record_id.unwrap();
    let (a, b) = race(
        &f,
        f.batch(
            "b1",
            vec![NormalizedMetricObservation {
                value: 20,
                ..f.observation()
            }],
        ),
        f.batch_b(
            "b1",
            vec![NormalizedMetricObservation {
                value: 30,
                ..f.observation_b()
            }],
        ),
    );
    assert_eq!(classes(&a.unwrap()), vec![Class::Revision]);
    let b = b.unwrap();
    assert_eq!(classes(&b), vec![Class::Conflict]);
    assert_eq!(b.rows[0].reason_code, Some(Code::SourceConflict));
    assert_eq!(f.current_value(record_id), 20);
    assert_eq!(
        count(
            &f.pool,
            "metric_record_revision",
            &format!("record_id = '{record_id}'")
        ),
        2
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_rollup_delta",
            &format!("record_id = '{record_id}'")
        ),
        2
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_provenance",
            &format!(
                "record_id = '{record_id}' AND classification = 'CONFLICT' AND import_id = '{}'",
                f.import_b
            )
        ),
        1
    );
    assert_eq!(f.counters(f.import_b), [1, 0, 0, 0, 1, 0]);
}

#[test]
fn simultaneous_overlapping_periods_accept_exactly_one_record() {
    let (_guard, f) = setup_fixture();
    let period = |start: u32, end: u32| NormalizedMetricObservation {
        period_start: date(2026, 3, start),
        period_end: date(2026, 3, end),
        reporting_grain: MetricReportingGrain::ReportingPeriod,
        ..f.observation()
    };
    let (a, b) = race(
        &f,
        f.batch("b1", vec![period(1, 10)]),
        f.batch("b2", vec![period(5, 15)]),
    );
    let mut all = [classes(&a.unwrap()), classes(&b.unwrap())];
    all.sort_by_key(|classes| classes.iter().map(ToString::to_string).collect::<Vec<_>>());
    assert_eq!(all, [vec![Class::Rejected], vec![Class::Winner]]);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
    assert_eq!(
        count(
            &f.pool,
            "metric_import_error",
            "error_code = 'OVERLAPPING_PERIOD'"
        ),
        1
    );
    assert_eq!(count(&f.pool, "metric_record_provenance", "classification = 'REJECTED' AND identity_hash IS NOT NULL AND content_hash IS NOT NULL"), 1);
    assert_eq!(f.counters(f.import_a), [2, 1, 0, 0, 0, 1]);
}

#[test]
fn the_same_batch_submitted_twice_concurrently_commits_once_and_replays_once() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch(
        "b1",
        vec![
            f.observation(),
            NormalizedMetricObservation {
                work_doi: "bad".into(),
                ..f.observation()
            },
        ],
    );
    let (a, b) = race(&f, batch.clone(), batch.clone());
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_ne!(
        a.replayed, b.replayed,
        "exactly one call commits and the other replays"
    );
    assert_eq!(a.rows, b.rows);
    assert_eq!(a.import_batch_id, b.import_batch_id);
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record_provenance", "TRUE"), 2);
    assert_eq!(count(&f.pool, "metric_import_error", "TRUE"), 1);
    assert_eq!(f.counters(f.import_a), [2, 1, 0, 0, 0, 1]);
}

#[test]
fn multi_cell_batches_in_opposite_order_do_not_deadlock() {
    let (_guard, f) = setup_fixture();
    // Three cells across two works and both optional dimensions, each
    // presented in reverse order by the second account.
    let cells = vec![
        f.observation(),
        NormalizedMetricObservation {
            work_doi: OTHER_DOI.into(),
            country_code: Some("GB".into()),
            ..f.observation()
        },
        NormalizedMetricObservation {
            publication_isbn: Some(PDF_ISBN.into()),
            institution_ror: Some(ROR.into()),
            ..f.observation()
        },
    ];
    let reversed: Vec<NormalizedMetricObservation> = cells
        .iter()
        .rev()
        .map(|o| NormalizedMetricObservation {
            source_account_code: ACCOUNT_B.into(),
            ..o.clone()
        })
        .collect();
    let (a, b) = race(&f, f.batch("b1", cells), f.batch_b("b1", reversed));
    let (a, b) = (a.expect("no deadlock"), b.expect("no deadlock"));
    assert!(a
        .rows
        .iter()
        .chain(b.rows.iter())
        .all(|row| matches!(row.classification, Class::Winner | Class::Duplicate)));
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 3);
    assert_eq!(
        count(
            &f.pool,
            "metric_record_provenance",
            "classification = 'WINNER'"
        ),
        3
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record_provenance",
            "classification = 'DUPLICATE'"
        ),
        3
    );
    // Sorted cell keys: the lock order is the same for both regardless of
    // input order.
    let identities: Vec<CanonicalIdentity> = vec![
        CanonicalIdentity {
            platform_id: f.platform_id,
            measure_id: f.title_sessions_id,
            work_id: f.work_id,
            publication_id: None,
            period_start: date(2026, 3, 1),
            period_end: date(2026, 3, 2),
            country_code: None,
            institution_id: None,
        },
        CanonicalIdentity {
            platform_id: f.platform_id,
            measure_id: f.title_sessions_id,
            work_id: f.other_work_id,
            publication_id: None,
            period_start: date(2026, 3, 1),
            period_end: date(2026, 3, 2),
            country_code: Some("GB".into()),
            institution_id: None,
        },
        CanonicalIdentity {
            platform_id: f.platform_id,
            measure_id: f.title_sessions_id,
            work_id: f.work_id,
            publication_id: Some(f.publication_id),
            period_start: date(2026, 3, 1),
            period_end: date(2026, 3, 2),
            country_code: None,
            institution_id: Some(f.institution_id),
        },
    ];
    let keys: BTreeSet<i64> = identities.iter().map(cell_lock_key).collect();
    assert_eq!(keys.len(), 3);
}

#[test]
fn two_ingestions_spanning_multiple_works_and_parents_in_opposite_order_do_not_deadlock() {
    let (_guard, f) = setup_fixture();
    // Work 1 (imprint 1, PDF publication, institution) and work 2 moved to a
    // second imprint of the same publisher with its own publication.
    let second_imprint = Uuid::new_v4();
    f.sql(&format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{second_imprint}', '{}', 'Second imprint')", f.publisher_id));
    f.sql(&format!(
        "UPDATE work SET imprint_id = '{second_imprint}' WHERE work_id = '{}'",
        f.other_work_id
    ));
    f.sql(&format!("INSERT INTO publication (publication_type, work_id, isbn) VALUES ('PDF', '{}', '{OTHER_ISBN}')", f.other_work_id));
    f.sql(&format!("INSERT INTO institution (institution_name, ror) VALUES ('Second institution', '{OTHER_ROR}')"));
    let rows = vec![
        NormalizedMetricObservation {
            publication_isbn: Some(PDF_ISBN.into()),
            institution_ror: Some(ROR.into()),
            ..f.observation()
        },
        NormalizedMetricObservation {
            work_doi: OTHER_DOI.into(),
            publication_isbn: Some(OTHER_ISBN.into()),
            institution_ror: Some(OTHER_ROR.into()),
            ..f.observation()
        },
    ];
    let reversed: Vec<NormalizedMetricObservation> = rows
        .iter()
        .rev()
        .map(|o| NormalizedMetricObservation {
            source_account_code: ACCOUNT_B.into(),
            value: 99,
            ..o.clone()
        })
        .collect();
    let (a, b) = race(&f, f.batch("b1", rows), f.batch_b("b1", reversed));
    let (a, b) = (a.expect("no deadlock"), b.expect("no deadlock"));
    let mut all = [classes(&a), classes(&b)];
    all.sort_by_key(|classes| classes.iter().map(ToString::to_string).collect::<Vec<_>>());
    assert_eq!(
        all,
        [
            vec![Class::Conflict, Class::Conflict],
            vec![Class::Winner, Class::Winner]
        ]
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 2);
}

// ---------------------------------------------------------------------------
// Authority-row linearization (Amendments 3 B7 and 4 C10)
// ---------------------------------------------------------------------------

/// Run ingestion on a pinned connection while `held` holds conflicting row
/// locks; prove ingestion blocks, commit `held`, and return the outcome that
/// observed the committed change.
fn ingest_after_held_commit(
    batch: MetricIngestionBatch,
    held: HeldTransaction,
) -> Result<MetricIngestionOutcome, MetricIngestionError> {
    let (pool, pid) = pinned_pool();
    let ingestion = thread::spawn(move || ingest_metric_batch(&pool, &batch));
    wait_until_blocked(pid);
    held.commit();
    ingestion.join().unwrap()
}

#[test]
fn ingestion_racing_a_platform_disable_has_a_deterministic_after_outcome() {
    let (_guard, f) = setup_fixture();
    let held = HeldTransaction::start(vec!["UPDATE metric_platform SET enabled = FALSE".into()]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::PlatformDisabled));
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn ingestion_racing_measure_changes_has_a_deterministic_after_outcome() {
    let (_guard, f) = setup_fixture();
    let held = HeldTransaction::start(vec!["UPDATE metric_measure SET methodology_version = 'cloudfront-title-session/3' WHERE code = 'title_sessions'".into()]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::MethodologyMismatch));
    let held = HeldTransaction::start(vec![
        "UPDATE metric_measure SET enabled = FALSE WHERE code = 'title_sessions'".into(),
    ]);
    let outcome = ingest_after_held_commit(f.batch("b2", vec![f.observation()]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::MeasureDisabled));
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn ingestion_racing_mapping_changes_has_a_deterministic_after_outcome() {
    let (_guard, f) = setup_fixture();
    let mapping = f.mapping_title_sessions_id;
    let cases = [
        (format!("UPDATE metric_platform_measure SET enabled = FALSE WHERE platform_measure_id = '{mapping}'"), Code::PlatformMeasureDisabled),
        (format!("UPDATE metric_platform_measure SET enabled = TRUE, direct_collection = FALSE WHERE platform_measure_id = '{mapping}'"), Code::NotDirectCollection),
        (format!("UPDATE metric_platform_measure SET direct_collection = TRUE, supported_grains = ARRAY['MONTH']::metric_reporting_grain[] WHERE platform_measure_id = '{mapping}'"), Code::UnsupportedReportingGrain),
        (format!("UPDATE metric_platform_measure SET supported_grains = ARRAY['DAY']::metric_reporting_grain[], supports_country = FALSE WHERE platform_measure_id = '{mapping}'"), Code::UnsupportedCountryDimension),
    ];
    for (index, (statement, code)) in cases.into_iter().enumerate() {
        let held = HeldTransaction::start(vec![statement]);
        let outcome = ingest_after_held_commit(
            f.batch(
                &format!("b{index}"),
                vec![NormalizedMetricObservation {
                    country_code: Some("GB".into()),
                    ..f.observation()
                }],
            ),
            held,
        )
        .unwrap();
        assert_eq!(outcome.rows[0].reason_code, Some(code));
    }
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn ingestion_cannot_commit_stale_entitlement_after_a_package_downgrade_wins_the_publisher_lock() {
    let (_guard, f) = setup_fixture();
    let held = HeldTransaction::start(vec![format!(
        "UPDATE publisher SET subscription_package = 'OASIS' WHERE publisher_id = '{}'",
        f.publisher_id
    )]);
    let before = f.snapshot();
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held);
    assert_eq!(
        outcome,
        Err(MetricIngestionError::new(Code::MetricsCollectNotEntitled))
    );
    assert_eq!(f.snapshot(), before);
}

#[test]
fn a_package_downgrade_waits_behind_an_ingestion_that_already_holds_the_publisher_lock() {
    let (_guard, f) = setup_fixture();
    // Pause ingestion after every authority lock is held, at its first
    // canonical write.
    let _pause = TestTrigger::pausing(&f.pool, "BEFORE INSERT", "metric_record", PAUSE_KEY);
    let hold = PauseHold::acquire();
    let (pool, pid) = pinned_pool();
    let batch = f.batch("b1", vec![f.observation()]);
    let ingestion = thread::spawn(move || ingest_metric_batch(&pool, &batch));
    wait_until_blocked(pid);

    let downgrade = HeldTransaction::start(vec![format!(
        "UPDATE publisher SET subscription_package = 'OASIS' WHERE publisher_id = '{}'",
        f.publisher_id
    )]);
    downgrade.wait_until_blocked();
    assert_eq!(
        text(
            &f.pool,
            &format!(
                "(SELECT subscription_package::text FROM publisher WHERE publisher_id = '{}')",
                f.publisher_id
            )
        )
        .unwrap(),
        "OBELISK"
    );
    // The same holds for registry administration writers.
    let disable = HeldTransaction::start(vec![
        "UPDATE metric_platform_measure SET enabled = FALSE".into(),
    ]);
    disable.wait_until_blocked();

    hold.release();
    let outcome = ingestion.join().unwrap().unwrap();
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    downgrade.commit();
    disable.commit();
    assert_eq!(
        text(
            &f.pool,
            &format!(
                "(SELECT subscription_package::text FROM publisher WHERE publisher_id = '{}')",
                f.publisher_id
            )
        )
        .unwrap(),
        "OASIS"
    );
    assert_eq!(
        count(&f.pool, "metric_record", "TRUE"),
        1,
        "the accepted record was committed under the entitled package"
    );
}

#[test]
fn an_imprint_ownership_change_that_wins_the_lock_is_observed_at_the_barrier() {
    let (_guard, f) = setup_fixture();
    let held = HeldTransaction::start(vec![format!(
        "UPDATE imprint SET publisher_id = '{}' WHERE imprint_id = '{}'",
        f.other_publisher_id, f.imprint_id
    )]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held).unwrap();
    assert_eq!(
        outcome.rows[0].reason_code,
        Some(Code::PublisherScopeMismatch)
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn a_work_imprint_movement_between_discovery_and_lock_restarts_and_is_observed() {
    let (_guard, f) = setup_fixture();
    // The work row is locked by the held update, so ingestion discovers the
    // original imprint, locks it, then blocks on the work lock; after the
    // commit the work references an imprint that was never locked: drift,
    // restart, and the new ownership is observed.
    let held = HeldTransaction::start(vec![format!(
        "UPDATE work SET imprint_id = '{}' WHERE work_id = '{}'",
        f.other_imprint_id, f.work_id
    )]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held).unwrap();
    assert!(!outcome.replayed);
    assert_eq!(
        outcome.rows[0].reason_code,
        Some(Code::PublisherScopeMismatch)
    );
    assert_eq!(count(&f.pool, "metric_import_batch", "TRUE"), 1);
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
}

#[test]
fn a_doi_change_between_discovery_and_work_lock_is_re_resolved() {
    let (_guard, f) = setup_fixture();
    // Nulled DOI: still no new authority row needed, so the row is rejected.
    let held = HeldTransaction::start(vec![format!(
        "UPDATE work SET doi = NULL WHERE work_id = '{}'",
        f.work_id
    )]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![f.observation()]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownDoi));
    // DOI moved to another work: drift, one restart, accepted on that work.
    f.sql(&format!(
        "UPDATE work SET doi = '{WORK_DOI}' WHERE work_id = '{}'",
        f.work_id
    ));
    let held = HeldTransaction::start(vec![
        format!("UPDATE work SET doi = NULL WHERE work_id = '{}'", f.work_id),
        format!(
            "UPDATE work SET doi = '{WORK_DOI}' WHERE work_id = '{}'",
            f.other_work_id
        ),
    ]);
    let outcome = ingest_after_held_commit(f.batch("b2", vec![f.observation()]), held).unwrap();
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("work_id = '{}'", f.other_work_id)
        ),
        1
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 1);
}

#[test]
fn publication_insert_update_and_delete_races_are_serialized_before_the_work_lock() {
    let (_guard, f) = setup_fixture();
    let with_isbn = |isbn: &str, day: u32| NormalizedMetricObservation {
        publication_isbn: Some(isbn.into()),
        period_start: date(2026, 3, day),
        period_end: date(2026, 3, day + 1),
        ..f.observation()
    };
    // INSERT: the trigger's work update holds the work lock. The ISBN row is
    // unknown at discovery; the plain row locks the work and blocks; after
    // the commit the ISBN resolves to an unlocked publication: drift, restart,
    // both accepted.
    let held = HeldTransaction::start(vec![format!("INSERT INTO publication (publication_type, work_id, isbn) VALUES ('Epub', '{}', '{OTHER_ISBN}')", f.work_id)]);
    let outcome = ingest_after_held_commit(
        f.batch("b1", vec![with_isbn(OTHER_ISBN, 1), f.observation()]),
        held,
    )
    .unwrap();
    assert_eq!(classes(&outcome), vec![Class::Winner, Class::Winner]);
    // UPDATE: the publication lock is held; after the commit its ISBN is gone.
    let held = HeldTransaction::start(vec![format!(
        "UPDATE publication SET isbn = NULL WHERE publication_id = '{}'",
        f.publication_id
    )]);
    let outcome =
        ingest_after_held_commit(f.batch("b2", vec![with_isbn(PDF_ISBN, 5)]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownPublication));
    // DELETE: the locked row is gone at lock time. A fresh, unreferenced
    // publication is used because an accepted record references the first.
    f.sql(&format!(
        "INSERT INTO publication (publication_type, work_id, isbn) VALUES ('Mobi', '{}', '{THIRD_ISBN}')",
        f.work_id
    ));
    let held = HeldTransaction::start(vec![format!(
        "DELETE FROM publication WHERE isbn = '{THIRD_ISBN}'"
    )]);
    let outcome =
        ingest_after_held_commit(f.batch("b3", vec![with_isbn(THIRD_ISBN, 10)]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownPublication));
    assert_eq!(
        count(&f.pool, "publication", &format!("isbn = '{THIRD_ISBN}'")),
        0,
        "the competing DELETE committed"
    );
    assert_eq!(
        count(&f.pool, "metric_record", "period_start = DATE '2026-03-10'"),
        0,
        "no record was accepted against the deleted publication"
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_import_error",
            "error_code = 'UNKNOWN_PUBLICATION' AND field_name = 'publication_isbn'"
        ),
        2,
        "the UPDATE and DELETE races each left exactly one sanitized error"
    );
    // Movement: the ISBN moves to a sibling publication between discovery and
    // the lock barrier: drift, restart, accepted on the sibling.
    let sibling = Uuid::new_v4();
    f.sql(&format!("INSERT INTO publication (publication_id, publication_type, work_id) VALUES ('{sibling}', 'Hardback', '{}')", f.work_id));
    f.sql(&format!(
        "UPDATE publication SET isbn = '{PDF_ISBN}' WHERE publication_id = '{}'",
        f.publication_id
    ));
    let held = HeldTransaction::start(vec![
        format!(
            "UPDATE publication SET isbn = NULL WHERE publication_id = '{}'",
            f.publication_id
        ),
        format!("UPDATE publication SET isbn = '{PDF_ISBN}' WHERE publication_id = '{sibling}'"),
    ]);
    let outcome =
        ingest_after_held_commit(f.batch("b4", vec![with_isbn(PDF_ISBN, 15)]), held).unwrap();
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("publication_id = '{sibling}'")
        ),
        1
    );
}

#[test]
fn publication_work_id_movement_between_discovery_and_barrier_is_re_resolved() {
    // Amendment 4 C4/C10: `publication.work_id` is revalidated at the locked
    // re-resolution barrier, and a concurrent publication identity/work
    // movement is a real independent-connection race. Both directions are
    // exercised: a publication moved OUT of the observed work after it was
    // provisionally resolved, and a publication moved INTO the observed work
    // after it was provisionally unresolvable.
    let (_guard, f) = setup_fixture();
    let with_isbn = |isbn: &str, day: u32| NormalizedMetricObservation {
        publication_isbn: Some(isbn.into()),
        period_start: date(2026, 3, day),
        period_end: date(2026, 3, day + 1),
        ..f.observation()
    };

    // Direction 1: the PDF publication provisionally resolves under the
    // observed work; the competing transaction moves it to another work and
    // holds the publication row (and, through the existing publication
    // trigger, both work rows) until ingestion is observed blocked on its
    // publication FOR SHARE lock. After the commit the barrier no longer finds
    // the ISBN within the observed work: no stale publication-to-work
    // authority is accepted, and accepting would need no unlocked row, so the
    // approved classification is a REJECTED row, not a restart.
    let held = HeldTransaction::start(vec![format!(
        "UPDATE publication SET work_id = '{}' WHERE publication_id = '{}'",
        f.other_work_id, f.publication_id
    )]);
    let (pool, ingestion_pid) = pinned_pool();
    assert_ne!(
        held.pid, ingestion_pid,
        "the competing update and the coordinator run on distinct backends"
    );
    let batch = f.batch("b1", vec![with_isbn(PDF_ISBN, 1)]);
    let ingestion = thread::spawn(move || ingest_metric_batch(&pool, &batch));
    wait_until_blocked(ingestion_pid);
    assert_eq!(
        count(
            &f.pool,
            "publication",
            &format!(
                "publication_id = '{}' AND work_id = '{}'",
                f.publication_id, f.work_id
            )
        ),
        1,
        "the movement is not yet visible while the competing transaction is open"
    );
    held.commit();
    let outcome = ingestion.join().unwrap().unwrap();
    assert!(!outcome.replayed);
    assert_eq!(outcome.rows[0].classification, Class::Rejected);
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownPublication));
    assert_eq!(
        outcome.rows[0].identity_hash, None,
        "no identity was formed from the moved publication"
    );
    assert_eq!(
        count(
            &f.pool,
            "publication",
            &format!(
                "publication_id = '{}' AND work_id = '{}'",
                f.publication_id, f.other_work_id
            )
        ),
        1,
        "the competing publication movement committed"
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 0);
    assert_eq!(
        count(&f.pool, "metric_record_provenance", "classification = 'REJECTED' AND details->>'reason_code' = 'UNKNOWN_PUBLICATION' AND batch_row_index = 0"),
        1
    );
    assert_eq!(
        count(&f.pool, "metric_import_error", "error_code = 'UNKNOWN_PUBLICATION' AND field_name = 'publication_isbn' AND raw_value IS NULL"),
        1
    );
    assert_eq!(count(&f.pool, "metric_import_batch", "batch_key = 'b1'"), 1);
    assert_eq!(f.counters(f.import_a), [1, 0, 0, 0, 0, 1]);

    // Direction 2: a publication of another work carries the ISBN; it is
    // unresolvable under the observed work at discovery. The competing
    // transaction moves it into the observed work and holds the work row via
    // the publication trigger, so the batch's plain observation blocks on the
    // work FOR UPDATE lock. After the commit the barrier resolves the ISBN to
    // a publication that was never locked in its prescribed phase: the attempt
    // rolls back and restarts, and the restarted attempt locks the moved
    // publication before the work and accepts both rows. No late publication
    // lock is taken inside the first attempt.
    let moved_in = Uuid::new_v4();
    f.sql(&format!(
        "INSERT INTO publication (publication_id, publication_type, work_id, isbn) VALUES ('{moved_in}', 'Epub', '{}', '{OTHER_ISBN}')",
        f.other_work_id
    ));
    let held = HeldTransaction::start(vec![format!(
        "UPDATE publication SET work_id = '{}' WHERE publication_id = '{moved_in}'",
        f.work_id
    )]);
    let (pool, ingestion_pid) = pinned_pool();
    assert_ne!(held.pid, ingestion_pid);
    let batch = f.batch(
        "b2",
        vec![
            with_isbn(OTHER_ISBN, 5),
            NormalizedMetricObservation {
                period_start: date(2026, 3, 6),
                period_end: date(2026, 3, 7),
                ..f.observation()
            },
        ],
    );
    let ingestion = thread::spawn(move || ingest_metric_batch(&pool, &batch));
    wait_until_blocked(ingestion_pid);
    held.commit();
    let outcome = ingestion.join().unwrap().unwrap();
    assert!(!outcome.replayed);
    assert_eq!(classes(&outcome), vec![Class::Winner, Class::Winner]);
    assert_eq!(
        count(
            &f.pool,
            "publication",
            &format!(
                "publication_id = '{moved_in}' AND work_id = '{}'",
                f.work_id
            )
        ),
        1,
        "the competing publication movement committed"
    );
    assert_eq!(
        count(&f.pool, "metric_record", &format!("publication_id = '{moved_in}' AND work_id = '{}' AND period_start = DATE '2026-03-05'", f.work_id)),
        1,
        "the accepted record references the moved publication under the observed work"
    );
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("publication_id = '{}'", f.publication_id)
        ),
        0
    );
    assert_eq!(count(&f.pool, "metric_record", "TRUE"), 2);
    assert_eq!(
        count(&f.pool, "metric_import_batch", "batch_key = 'b2'"),
        1,
        "one durable batch row despite the restart"
    );
    assert_eq!(f.counters(f.import_a), [3, 2, 0, 0, 0, 1]);
}

#[test]
fn institution_update_and_ror_movement_races_are_serialized_before_the_work_lock() {
    let (_guard, f) = setup_fixture();
    let with_ror = |day: u32| NormalizedMetricObservation {
        institution_ror: Some(ROR.into()),
        period_start: date(2026, 3, day),
        period_end: date(2026, 3, day + 1),
        ..f.observation()
    };
    let held = HeldTransaction::start(vec![format!(
        "UPDATE institution SET ror = NULL WHERE institution_id = '{}'",
        f.institution_id
    )]);
    let outcome = ingest_after_held_commit(f.batch("b1", vec![with_ror(1)]), held).unwrap();
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownRor));
    let other = Uuid::new_v4();
    f.sql(&format!("INSERT INTO institution (institution_id, institution_name) VALUES ('{other}', 'Successor')"));
    f.sql(&format!(
        "UPDATE institution SET ror = '{ROR}' WHERE institution_id = '{}'",
        f.institution_id
    ));
    let held = HeldTransaction::start(vec![
        format!(
            "UPDATE institution SET ror = NULL WHERE institution_id = '{}'",
            f.institution_id
        ),
        format!("UPDATE institution SET ror = '{ROR}' WHERE institution_id = '{other}'"),
    ]);
    let outcome = ingest_after_held_commit(f.batch("b2", vec![with_ror(5)]), held).unwrap();
    assert_eq!(outcome.rows[0].classification, Class::Winner);
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("institution_id = '{other}'")
        ),
        1
    );
    assert_eq!(count(&f.pool, "institution", "TRUE"), 2);
}

#[test]
fn repeated_lock_set_drift_through_three_attempts_fails_closed_and_stays_retryable() {
    let (_guard, f) = setup_fixture();
    let third_imprint = Uuid::new_v4();
    f.sql(&format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{third_imprint}', '{}', 'Third imprint')", f.publisher_id));
    let move_to = |imprint: Uuid| {
        format!(
            "UPDATE work SET imprint_id = '{imprint}' WHERE work_id = '{}'",
            f.work_id
        )
    };
    let imprints = [f.other_imprint_id, third_imprint, f.imprint_id];

    // Each held update keeps the work row locked from before an attempt's
    // discovery until that attempt is blocked on the work lock, so every
    // attempt observes a moved imprint at its barrier. Queued updaters are
    // granted the row lock ahead of the coordinator's next attempt.
    let (pool, pid) = pinned_pool();
    let before = f.snapshot();
    let mut holder = HeldTransaction::start(vec![move_to(imprints[0])]);
    let batch = f.batch("b1", vec![f.observation()]);
    let ingestion = thread::spawn(move || ingest_metric_batch(&pool, &batch));
    for next in [imprints[1], imprints[2]] {
        wait_until_blocked(pid);
        let queued = HeldTransaction::start(vec![move_to(next)]);
        queued.wait_until_blocked();
        holder.commit();
        holder = queued;
    }
    wait_until_blocked(pid);
    holder.commit();
    let outcome = ingestion.join().unwrap();
    assert_eq!(
        outcome,
        Err(MetricIngestionError::new(Code::ConcurrentAuthorityChange))
    );
    assert_eq!(f.snapshot(), before, "zero durable consequence");
    // The same request is retryable, and no late parent lock was ever used to
    // repair a stale lock set: the work now sits on its final imprint.
    let retry = f.accept(&f.batch("b1", vec![f.observation()]));
    assert!(!retry.replayed);
    assert_eq!(retry.rows[0].classification, Class::Winner);
}

// ---------------------------------------------------------------------------
// Unresolved-DOI quarantine (MET-WP7-PREREQ-02)
// ---------------------------------------------------------------------------

/// A syntactically valid DOI no work carries, in a deliberately non-canonical
/// spelling, so byte-exact storage is observable.
const UNRESOLVED_DOI: &str = "HTTP://DX.DOI.ORG/10.12345/Unresolved-Case";

/// The number of quarantine rows linked to one committed batch row.
fn quarantined_rows(f: &Fixture, import_batch_id: Uuid, batch_row_index: i64) -> i64 {
    count(
        &f.pool,
        "metric_identifier_quarantine q JOIN metric_record_provenance p \
         ON p.record_provenance_id = q.record_provenance_id",
        &format!(
            "p.import_batch_id = '{import_batch_id}' AND p.batch_row_index = {batch_row_index} \
             AND p.classification = 'REJECTED' AND p.details->>'reason_code' = 'UNKNOWN_DOI'"
        ),
    )
}

#[test]
fn an_eligible_cloudfront_unknown_doi_is_rejected_with_exactly_one_quarantine_row() {
    let (_guard, f) = setup_fixture();
    let observation = NormalizedMetricObservation {
        country_code: Some("GB".into()),
        value: 42,
        ..f.unresolved(UNRESOLVED_DOI)
    };
    let before = f.snapshot();
    let outcome = f.accept(&f.batch("b1", vec![observation]));

    // The external outcome is the ordinary rejection, unchanged.
    let row = &outcome.rows[0];
    assert_eq!(row.classification, Class::Rejected);
    assert_eq!(row.reason_code, Some(Code::UnknownDoi));
    assert_eq!(row.record_id, None);
    assert_eq!(row.identity_hash, None);
    assert_eq!(row.content_hash, None);

    // Nothing canonical, one provenance row, one sanitized error, the invalid
    // counter, and exactly one quarantine row.
    let after = f.snapshot();
    assert_eq!(after.records, before.records);
    assert_eq!(after.revisions, before.revisions);
    assert_eq!(after.deltas, before.deltas);
    assert_eq!(after.batches, before.batches + 1);
    assert_eq!(after.provenance, before.provenance + 1);
    assert_eq!(after.errors, before.errors + 1);
    assert_eq!(after.quarantine, before.quarantine + 1);
    assert_eq!(f.counters(f.import_a), [1, 0, 0, 0, 0, 1]);
    assert_eq!(count(&f.pool, "metric_record_provenance", &format!("classification = 'REJECTED' AND record_id IS NULL AND identity_hash IS NULL AND content_hash IS NULL AND source_record_id IS NULL AND source_row_number IS NULL AND details->>'reason_code' = 'UNKNOWN_DOI' AND import_batch_id = '{}' AND batch_row_index = 0", outcome.import_batch_id)), 1);
    assert_eq!(count(&f.pool, "metric_import_error", &format!("import_id = '{}' AND row_number IS NULL AND error_code = 'UNKNOWN_DOI' AND severity = 'ERROR' AND field_name = 'work_doi' AND raw_value IS NULL AND message = '{}'", f.import_a, Code::UnknownDoi.message())), 1);

    // The quarantine row carries exactly the approved reduced observation,
    // resolved to the locked canonical rows, linked to that provenance row.
    assert_eq!(quarantined_rows(&f, outcome.import_batch_id, 0), 1);
    assert_eq!(count(&f.pool, "metric_identifier_quarantine", &format!("source_account_id = '{}' AND platform_id = '{}' AND measure_id = '{}' AND schema_version = '{SUPPORTED_SCHEMA_VERSION}' AND period_start = DATE '2026-03-01' AND period_end = DATE '2026-03-02' AND reporting_grain = 'DAY' AND country_code = 'GB' AND value = 42 AND methodology_version = '{TITLE_SESSIONS_METHODOLOGY}' AND created_at IS NOT NULL", f.account_a, f.platform_id, f.title_sessions_id)), 1);
    // The DOI is stored byte for byte as supplied: not lowercased, not
    // re-prefixed and not converted to the canonical work-table form.
    assert_eq!(
        text(
            &f.pool,
            "(SELECT work_doi FROM metric_identifier_quarantine)"
        )
        .as_deref(),
        Some(UNRESOLVED_DOI)
    );
}

#[test]
fn a_known_cloudfront_doi_is_unchanged_and_never_quarantined() {
    let (_guard, f) = setup_fixture();
    // A known DOI with every quarantine-excluded field absent.
    let known = NormalizedMetricObservation {
        source_record_id: None,
        source_row_number: None,
        ..f.observation()
    };
    let winner = f.accept(&f.batch("b1", vec![known.clone()]));
    assert_eq!(classes(&winner), vec![Class::Winner]);
    let duplicate = f.accept(&f.batch("b2", vec![known.clone()]));
    assert_eq!(classes(&duplicate), vec![Class::Duplicate]);
    let revision = f.accept(&f.batch(
        "b3",
        vec![NormalizedMetricObservation { value: 11, ..known }],
    ));
    assert_eq!(classes(&revision), vec![Class::Revision]);
    let state = f.snapshot();
    assert_eq!(
        (state.records, state.revisions, state.deltas),
        (1, 2, 2),
        "known-DOI canonical writes are unchanged"
    );
    assert_eq!(state.errors, 0);
    assert_eq!(state.quarantine, 0, "a resolved DOI is never quarantined");
    assert_eq!(f.counters(f.import_a), [3, 1, 1, 1, 0, 0]);
}

#[test]
fn a_replayed_quarantine_batch_writes_nothing_and_never_duplicates_quarantine() {
    let (_guard, f) = setup_fixture();
    let batch = MetricIngestionBatch {
        coverage: vec![coverage(MetricCoverageStatus::Complete)],
        ..f.batch("b1", vec![f.observation(), f.unresolved(UNRESOLVED_DOI)])
    };
    let first = f.accept(&batch);
    let state = f.snapshot();
    assert_eq!(state.quarantine, 1);
    let fresh_pool = pool_of(2);
    for replay in [
        f.accept(&batch),
        ingest_metric_batch(&fresh_pool, &batch).unwrap(),
        f.accept(&batch),
    ] {
        assert!(replay.replayed);
        assert_eq!(replay.import_batch_id, first.import_batch_id);
        assert_eq!(replay.rows, first.rows);
        assert_eq!(replay.rows[1].reason_code, Some(Code::UnknownDoi));
        assert_eq!(f.snapshot(), state, "a replay writes nothing");
    }

    // The same eligible batch submitted twice concurrently commits once and
    // replays once: exactly one more quarantine row.
    let concurrent = f.batch("b2", vec![f.unresolved("10.12345/concurrent")]);
    let (one, two) = race(&f, concurrent.clone(), concurrent);
    let (one, two) = (one.unwrap(), two.unwrap());
    assert_eq!(one.rows, two.rows);
    assert_ne!(one.replayed, two.replayed, "exactly one call commits");
    assert_eq!(count(&f.pool, "metric_identifier_quarantine", "TRUE"), 2);
    assert_eq!(
        count(
            &f.pool,
            "metric_identifier_quarantine",
            "work_doi = '10.12345/concurrent'"
        ),
        1
    );
    assert_eq!(f.counters(f.import_a), [3, 1, 0, 0, 0, 2]);
}

#[test]
fn a_failed_quarantine_write_commits_no_provenance_error_counter_or_batch() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch("b1", vec![f.observation(), f.unresolved(UNRESOLVED_DOI)]);
    {
        let _trigger = TestTrigger::failing(
            &f.pool,
            "BEFORE INSERT",
            "metric_identifier_quarantine",
            None,
        );
        assert_database_failure_leaves_nothing(&f, &batch);
    }
    {
        // Raised at COMMIT, after every write of the attempt has been issued.
        let _trigger =
            TestTrigger::failing_at_commit(&f.pool, "INSERT", "metric_identifier_quarantine");
        assert_database_failure_leaves_nothing(&f, &batch);
    }
    {
        // A failed rejected-provenance or import-error write leaves no
        // orphaned quarantine row either.
        let _trigger = TestTrigger::failing(
            &f.pool,
            "BEFORE INSERT",
            "metric_record_provenance",
            Some("NEW.classification = 'REJECTED'"),
        );
        assert_database_failure_leaves_nothing(&f, &batch);
    }
    {
        let _trigger = TestTrigger::failing(&f.pool, "BEFORE INSERT", "metric_import_error", None);
        assert_database_failure_leaves_nothing(&f, &batch);
    }
    {
        let _trigger = TestTrigger::failing_at_commit(&f.pool, "UPDATE", "metric_import");
        assert_database_failure_leaves_nothing(&f, &batch);
    }
    let state = f.snapshot();
    assert_eq!(
        (
            state.batches,
            state.provenance,
            state.errors,
            state.records,
            state.quarantine
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(f.counters(f.import_a), [0, 0, 0, 0, 0, 0]);

    // Without an injected failure the same batch commits every part together.
    let outcome = f.accept(&batch);
    assert!(!outcome.replayed);
    assert_eq!(quarantined_rows(&f, outcome.import_batch_id, 1), 1);
    assert_eq!(f.snapshot().quarantine, 1);
    assert_eq!(f.snapshot().errors, 1);
    assert_eq!(f.counters(f.import_a), [2, 1, 0, 0, 0, 1]);
}

#[test]
fn an_invalid_doi_is_never_quarantined() {
    let (_guard, f) = setup_fixture();
    for (index, doi) in [
        "",
        "not-a-doi",
        "10.123/registrant-too-short",
        "https://doi.org/11.12345/wrong-directory",
        " 10.12345/leading-space",
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = f.accept(&f.batch(&format!("b{index}"), vec![f.unresolved(doi)]));
        assert_eq!(outcome.rows[0].classification, Class::Rejected, "{doi:?}");
        assert_eq!(
            outcome.rows[0].reason_code,
            Some(Code::InvalidDoi),
            "{doi:?}"
        );
    }
    assert_eq!(
        count(&f.pool, "metric_import_error", "error_code = 'INVALID_DOI'"),
        5
    );
    assert_eq!(f.snapshot().quarantine, 0);
    assert_eq!(f.counters(f.import_a), [5, 0, 0, 0, 0, 5]);
}

#[test]
fn a_non_cloudfront_driver_unknown_doi_is_never_quarantined() {
    let (_guard, f) = setup_fixture();
    // Only the exact locked driver key proves the CloudFront policy: a
    // different key, a case variant or a whitespace variant never qualifies.
    for (index, driver_key) in [
        "crossref-events",
        "CloudFront",
        "CLOUDFRONT",
        "cloudfront ",
        " cloudfront",
        "cloudfront/2",
    ]
    .into_iter()
    .enumerate()
    {
        f.sql(&format!(
            "UPDATE metric_source SET driver_key = '{driver_key}'"
        ));
        let outcome = f.accept(&f.batch(&format!("b{index}"), vec![f.unresolved(UNRESOLVED_DOI)]));
        assert_eq!(
            outcome.rows[0].classification,
            Class::Rejected,
            "{driver_key}"
        );
        assert_eq!(
            outcome.rows[0].reason_code,
            Some(Code::UnknownDoi),
            "{driver_key}"
        );
        assert_eq!(
            f.snapshot().quarantine,
            0,
            "{driver_key:?} must not quarantine"
        );
    }
    assert_eq!(f.counters(f.import_a), [6, 0, 0, 0, 0, 6]);

    // Codes and routing play no part: renaming the source and changing the
    // account's external key leave an exact `cloudfront` key eligible.
    f.sql("UPDATE metric_source SET driver_key = 'cloudfront', code = 'generic-source'");
    f.sql(&format!(
        "UPDATE metric_source_account SET external_key = 'not-a-distribution' WHERE source_account_id = '{}'",
        f.account_a
    ));
    let outcome = f.accept(&f.batch("eligible", vec![f.unresolved(UNRESOLVED_DOI)]));
    assert_eq!(outcome.rows[0].reason_code, Some(Code::UnknownDoi));
    assert_eq!(quarantined_rows(&f, outcome.import_batch_id, 0), 1);
}

#[test]
fn a_mixed_batch_keeps_every_valid_canonical_row_and_quarantines_only_eligible_unknowns() {
    let (_guard, f) = setup_fixture();
    let batch = f.batch(
        "b1",
        vec![
            f.observation(),
            f.unresolved(UNRESOLVED_DOI),
            NormalizedMetricObservation {
                work_doi: OTHER_DOI.into(),
                ..f.observation()
            },
            f.unresolved("not-a-doi"),
            NormalizedMetricObservation {
                source_row_number: Some(9),
                ..f.unresolved("10.12345/ineligible-row-number")
            },
            NormalizedMetricObservation {
                country_code: Some("FR".into()),
                ..f.unresolved("10.12345/second-unresolved")
            },
        ],
    );
    let outcome = f.accept(&batch);
    assert_eq!(
        classes(&outcome),
        vec![
            Class::Winner,
            Class::Rejected,
            Class::Winner,
            Class::Rejected,
            Class::Rejected,
            Class::Rejected
        ]
    );
    assert_eq!(
        outcome
            .rows
            .iter()
            .map(|row| row.reason_code)
            .collect::<Vec<_>>(),
        vec![
            None,
            Some(Code::UnknownDoi),
            None,
            Some(Code::InvalidDoi),
            Some(Code::UnknownDoi),
            Some(Code::UnknownDoi)
        ]
    );
    // Both valid rows are canonical, with their records, revisions and deltas.
    let state = f.snapshot();
    assert_eq!((state.records, state.revisions, state.deltas), (2, 2, 2));
    assert_eq!(
        count(
            &f.pool,
            "metric_record",
            &format!("work_id IN ('{}', '{}')", f.work_id, f.other_work_id)
        ),
        2
    );
    assert_eq!((state.provenance, state.errors), (6, 4));
    // Only the two eligible unknowns are quarantined.
    assert_eq!(state.quarantine, 2);
    for (index, quarantined) in [(1, 1), (3, 0), (4, 0), (5, 1)] {
        assert_eq!(
            quarantined_rows(&f, outcome.import_batch_id, index)
                + count(&f.pool, "metric_identifier_quarantine q JOIN metric_record_provenance p ON p.record_provenance_id = q.record_provenance_id", &format!("p.import_batch_id = '{}' AND p.batch_row_index = {index} AND p.details->>'reason_code' <> 'UNKNOWN_DOI'", outcome.import_batch_id)),
            quarantined,
            "row {index}"
        );
    }
    assert_eq!(f.counters(f.import_a), [6, 2, 0, 0, 0, 4]);
}

#[test]
fn each_quarantine_excluded_optional_field_keeps_an_ordinary_rejection() {
    let (_guard, f) = setup_fixture();
    let cases: Vec<(&str, NormalizedMetricObservation)> = vec![
        (
            "publication_isbn",
            NormalizedMetricObservation {
                publication_isbn: Some(PDF_ISBN.into()),
                ..f.unresolved(UNRESOLVED_DOI)
            },
        ),
        (
            "publication_type",
            NormalizedMetricObservation {
                publication_type: Some(PublicationType::Pdf),
                ..f.unresolved(UNRESOLVED_DOI)
            },
        ),
        (
            "institution_ror",
            NormalizedMetricObservation {
                institution_ror: Some(ROR.into()),
                ..f.unresolved(UNRESOLVED_DOI)
            },
        ),
        (
            "source_record_id",
            NormalizedMetricObservation {
                source_record_id: Some("row-7".into()),
                ..f.unresolved(UNRESOLVED_DOI)
            },
        ),
        (
            "source_row_number",
            NormalizedMetricObservation {
                source_row_number: Some(0),
                ..f.unresolved(UNRESOLVED_DOI)
            },
        ),
    ];
    for (index, (field, observation)) in cases.into_iter().enumerate() {
        let before = f.snapshot();
        let counters_before = f.counters(f.import_a);
        let outcome = f.accept(&f.batch(&format!("excluded-{index}"), vec![observation]));
        // Still the ordinary UNKNOWN_DOI rejection with its normal evidence...
        assert_eq!(outcome.rows[0].classification, Class::Rejected, "{field}");
        assert_eq!(
            outcome.rows[0].reason_code,
            Some(Code::UnknownDoi),
            "{field}"
        );
        let after = f.snapshot();
        assert_eq!(after.provenance, before.provenance + 1, "{field}");
        assert_eq!(after.errors, before.errors + 1, "{field}");
        assert_eq!(after.records, before.records, "{field}");
        let [received, _, _, _, _, invalid] = counters_before;
        assert_eq!(f.counters(f.import_a)[0], received + 1, "{field}");
        assert_eq!(f.counters(f.import_a)[5], invalid + 1, "{field}");
        // ...but never quarantined.
        assert_eq!(after.quarantine, 0, "{field} must prevent quarantine");
    }
    // The same observation without any excluded field is quarantined.
    let outcome = f.accept(&f.batch("eligible", vec![f.unresolved(UNRESOLVED_DOI)]));
    assert_eq!(quarantined_rows(&f, outcome.import_batch_id, 0), 1);
    assert_eq!(f.snapshot().quarantine, 1);
}

#[test]
fn only_unknown_doi_rejections_are_ever_quarantined() {
    let (_guard, f) = setup_fixture();
    f.sql(&format!(
        "UPDATE work SET imprint_id = '{}' WHERE work_id = '{}'",
        f.other_imprint_id, f.other_work_id
    ));
    let cases: Vec<(NormalizedMetricObservation, Code)> = vec![
        (
            NormalizedMetricObservation {
                work_doi: OTHER_DOI.into(),
                ..f.unresolved(UNRESOLVED_DOI)
            },
            Code::PublisherScopeMismatch,
        ),
        (
            NormalizedMetricObservation {
                country_code: Some("gb".into()),
                ..f.unresolved(UNRESOLVED_DOI)
            },
            Code::InvalidCountry,
        ),
        (
            NormalizedMetricObservation {
                value: -1,
                ..f.unresolved(UNRESOLVED_DOI)
            },
            Code::InvalidValue,
        ),
        (
            NormalizedMetricObservation {
                methodology_version: "cloudfront-title-session/1".into(),
                ..f.unresolved(UNRESOLVED_DOI)
            },
            Code::MethodologyMismatch,
        ),
        (
            NormalizedMetricObservation {
                measure_code: "orphan".into(),
                ..f.unresolved(UNRESOLVED_DOI)
            },
            Code::PlatformMeasureNotFound,
        ),
    ];
    for (index, (observation, code)) in cases.into_iter().enumerate() {
        let outcome = f.accept(&f.batch(&format!("b{index}"), vec![observation]));
        assert_eq!(outcome.rows[0].reason_code, Some(code));
    }
    assert_eq!(f.snapshot().quarantine, 0);
}
