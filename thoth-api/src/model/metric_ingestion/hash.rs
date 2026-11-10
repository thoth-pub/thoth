//! Deterministic durable hashing for the canonical ingestion coordinator
//! (`MET-WP2-01B`).
//!
//! Four durable hashes are derived here, all SHA-256 encoded as lowercase
//! hexadecimal and all built from one typed, length-prefixed, big-endian byte
//! encoding under a version/domain-separated preimage:
//!
//! - the canonical **identity** hash stored in `metric_record.identity_hash`;
//! - the canonical **content** hash stored in
//!   `metric_record_revision.content_hash`;
//! - the durable batch **request** hash stored in
//!   `metric_import_batch.request_hash`;
//! - the dimensional-cell **advisory lock key** passed to
//!   `pg_advisory_xact_lock(bigint)`.
//!
//! No JSON or map serialization, `Debug` formatting, locale formatting,
//! platform-native integer representation or Rust `DefaultHasher` participates
//! in any of them. A change to any encoding here is a new hash version, never a
//! silent alteration of `v1`.
//!
//! Every domain separator ends in one real `0x00` byte: the Rust byte-string
//! escape `\0` below is exactly that byte, not the two characters backslash and
//! zero.

use chrono::NaiveDate;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{MetricIngestionBatch, NormalizedMetricCoverageAssertion, NormalizedMetricObservation};

/// Domain separator of the canonical identity hash.
pub const IDENTITY_DOMAIN: &[u8] = b"thoth-metric-record-identity/v1\0";
/// Domain separator of the canonical content hash.
pub const CONTENT_DOMAIN: &[u8] = b"thoth-metric-record-content/v1\0";
/// Domain separator of the durable batch request hash.
pub const REQUEST_DOMAIN: &[u8] = b"thoth-metric-import-batch-request/v1\0";
/// Domain separator of the dimensional-cell advisory lock key.
pub const CELL_LOCK_DOMAIN: &[u8] = b"thoth-metric-cell-lock/v1\0";

/// The date encoding origin: dates are encoded as the signed number of
/// calendar days from this day.
const EPOCH: NaiveDate = match NaiveDate::from_ymd_opt(1970, 1, 1) {
    Some(date) => date,
    None => panic!("1970-01-01 is a valid date"),
};

/// The resolved canonical identity of one observation.
///
/// This is exactly the approved identity field tuple: source account, value,
/// methodology, reporting grain and source record/row identifiers are all
/// excluded. `country_code`, when present, is the validated uppercase
/// ISO 3166-1 alpha-2 string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalIdentity {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub country_code: Option<String>,
    pub institution_id: Option<Uuid>,
}

/// The typed canonical byte encoder shared by every durable hash.
///
/// Primitives, exactly as specified:
///
/// - string: unsigned `u64` big-endian UTF-8 byte length, then those bytes;
/// - UUID: the 16 raw bytes;
/// - date: signed `i64` big-endian days from 1970-01-01;
/// - signed integer: signed `i64` two's-complement big-endian;
/// - bool: `0x00` false, `0x01` true;
/// - optional: `0x00` for NULL, or `0x01` followed by the encoded value;
/// - list: unsigned `u64` big-endian item count, then the items in order.
pub(crate) struct CanonicalEncoder {
    bytes: Vec<u8>,
}

impl CanonicalEncoder {
    pub(crate) fn with_domain(domain: &[u8]) -> Self {
        CanonicalEncoder {
            bytes: domain.to_vec(),
        }
    }

    pub(crate) fn string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.bytes
            .extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        self.bytes.extend_from_slice(bytes);
    }

    pub(crate) fn optional_string(&mut self, value: Option<&str>) {
        match value {
            None => self.bytes.push(0x00),
            Some(value) => {
                self.bytes.push(0x01);
                self.string(value);
            }
        }
    }

    pub(crate) fn uuid(&mut self, value: Uuid) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    pub(crate) fn optional_uuid(&mut self, value: Option<Uuid>) {
        match value {
            None => self.bytes.push(0x00),
            Some(value) => {
                self.bytes.push(0x01);
                self.uuid(value);
            }
        }
    }

    pub(crate) fn date(&mut self, value: NaiveDate) {
        self.i64(days_from_epoch(value));
    }

    pub(crate) fn i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn optional_i64(&mut self, value: Option<i64>) {
        match value {
            None => self.bytes.push(0x00),
            Some(value) => {
                self.bytes.push(0x01);
                self.i64(value);
            }
        }
    }

    pub(crate) fn bool(&mut self, value: bool) {
        self.bytes.push(u8::from(value));
    }

    pub(crate) fn count(&mut self, value: usize) {
        self.bytes.extend_from_slice(&(value as u64).to_be_bytes());
    }

    /// A closed enum, encoded as the string primitive over its exact
    /// SCREAMING_SNAKE_CASE serde code. See [`enum_code`].
    pub(crate) fn enum_code<T: Serialize>(&mut self, value: &T) {
        self.string(&enum_code(value));
    }

    pub(crate) fn optional_enum_code<T: Serialize>(&mut self, value: Option<&T>) {
        match value {
            None => self.bytes.push(0x00),
            Some(value) => {
                self.bytes.push(0x01);
                self.enum_code(value);
            }
        }
    }

    pub(crate) fn digest(self) -> [u8; 32] {
        Sha256::digest(&self.bytes).into()
    }

    pub(crate) fn hex(self) -> String {
        hex::encode(self.digest())
    }

    #[cfg(test)]
    pub(crate) fn preimage(&self) -> &[u8] {
        &self.bytes
    }
}

/// Signed calendar days from 1970-01-01.
pub(crate) fn days_from_epoch(date: NaiveDate) -> i64 {
    date.signed_duration_since(EPOCH).num_days()
}

/// The one canonical durable string of a closed enum: its exact
/// SCREAMING_SNAKE_CASE serde variant code.
///
/// Every enum hashed here — `MetricReportingGrain`, `MetricCoverageStatus`
/// and `PublicationType` — is a closed unit enum whose serde representation is
/// `rename_all = "SCREAMING_SNAKE_CASE"`, so this is the stable semantic code
/// and never a display label, database presentation label or caller spelling.
pub(crate) fn enum_code<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(code)) => code,
        _ => unreachable!("closed unit enums serialize as plain strings"),
    }
}

/// Encode the eight canonical identity fields in their fixed order.
fn encode_identity(encoder: &mut CanonicalEncoder, identity: &CanonicalIdentity) {
    encoder.uuid(identity.platform_id);
    encoder.uuid(identity.measure_id);
    encoder.uuid(identity.work_id);
    encoder.optional_uuid(identity.publication_id);
    encoder.date(identity.period_start);
    encoder.date(identity.period_end);
    encoder.optional_string(identity.country_code.as_deref());
    encoder.optional_uuid(identity.institution_id);
}

/// The canonical identity hash: SHA-256 lowercase hex over
/// [`IDENTITY_DOMAIN`] followed by the eight identity fields in order.
pub fn identity_hash(identity: &CanonicalIdentity) -> String {
    let mut encoder = CanonicalEncoder::with_domain(IDENTITY_DOMAIN);
    encode_identity(&mut encoder, identity);
    encoder.hex()
}

/// The canonical content hash: SHA-256 lowercase hex over
/// [`CONTENT_DOMAIN`], the same eight identity fields in the same order, the
/// normalized signed `value` and the exact `methodology_version`.
///
/// The identity fields are encoded directly; the textual identity-hash result
/// does not participate.
pub fn content_hash(identity: &CanonicalIdentity, value: i64, methodology_version: &str) -> String {
    let mut encoder = CanonicalEncoder::with_domain(CONTENT_DOMAIN);
    encode_identity(&mut encoder, identity);
    encoder.i64(value);
    encoder.string(methodology_version);
    encoder.hex()
}

/// The dimensional-cell advisory lock key: SHA-256 over [`CELL_LOCK_DOMAIN`]
/// and the six cell dimensions (platform, measure, work, publication/NULL,
/// country/NULL, institution/NULL — the period is excluded), with the first
/// eight digest bytes interpreted as one signed big-endian `i64`.
///
/// A key collision between two different cells can only over-serialize.
pub fn cell_lock_key(identity: &CanonicalIdentity) -> i64 {
    let mut encoder = CanonicalEncoder::with_domain(CELL_LOCK_DOMAIN);
    encoder.uuid(identity.platform_id);
    encoder.uuid(identity.measure_id);
    encoder.uuid(identity.work_id);
    encoder.optional_uuid(identity.publication_id);
    encoder.optional_string(identity.country_code.as_deref());
    encoder.optional_uuid(identity.institution_id);
    let digest = encoder.digest();
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(head)
}

/// Encode one observation's fifteen normalized input fields in the exact
/// Amendment 2 A3 order, using the supplied strings rather than any resolved
/// database identity.
fn encode_observation(encoder: &mut CanonicalEncoder, observation: &NormalizedMetricObservation) {
    encoder.string(&observation.source_account_code);
    encoder.string(&observation.platform_code);
    encoder.string(&observation.measure_code);
    encoder.string(&observation.work_doi);
    encoder.optional_string(observation.publication_isbn.as_deref());
    encoder.optional_enum_code(observation.publication_type.as_ref());
    encoder.date(observation.period_start);
    encoder.date(observation.period_end);
    encoder.enum_code(&observation.reporting_grain);
    encoder.optional_string(observation.country_code.as_deref());
    encoder.optional_string(observation.institution_ror.as_deref());
    encoder.i64(observation.value);
    encoder.optional_string(observation.source_record_id.as_deref());
    encoder.string(&observation.methodology_version);
    encoder.optional_i64(observation.source_row_number);
}

/// Encode one coverage assertion's eight fields in the exact Amendment 2 A4
/// order.
fn encode_coverage(encoder: &mut CanonicalEncoder, coverage: &NormalizedMetricCoverageAssertion) {
    encoder.string(&coverage.platform_code);
    encoder.string(&coverage.measure_code);
    encoder.date(coverage.period_start);
    encoder.date(coverage.period_end);
    encoder.enum_code(&coverage.status);
    encoder.bool(coverage.country_coverage);
    encoder.bool(coverage.institution_coverage);
    encoder.optional_string(coverage.notes.as_deref());
}

/// The durable batch request hash: SHA-256 lowercase hex over
/// [`REQUEST_DOMAIN`], the exact `schema_version`, the observation count, every
/// observation in submitted order, the coverage count and every coverage
/// assertion in submitted order.
///
/// `import_id`, `batch_key`, the hash itself and every database-generated
/// identifier or timestamp are excluded: `(import_id, batch_key)` is already the
/// durable idempotency key and this hash identifies the supplied payload
/// underneath it.
pub fn request_hash(batch: &MetricIngestionBatch) -> String {
    let mut encoder = CanonicalEncoder::with_domain(REQUEST_DOMAIN);
    encoder.string(&batch.schema_version);
    encoder.count(batch.observations.len());
    for observation in &batch.observations {
        encode_observation(&mut encoder, observation);
    }
    encoder.count(batch.coverage.len());
    for coverage in &batch.coverage {
        encode_coverage(&mut encoder, coverage);
    }
    encoder.hex()
}
