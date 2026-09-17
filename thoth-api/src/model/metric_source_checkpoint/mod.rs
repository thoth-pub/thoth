//! Metric source checkpoints (`MET-WP1-02`).
//!
//! This module owns the persisted `metric_source_checkpoint` model: durable
//! per-partition checkpoint, progress and lease **storage** for one
//! [`metric_source_account`](crate::model::metric_source_account). Checkpoint
//! identity is `(source_account_id, partition_key)` and is enforced by the
//! database. PostgreSQL is the sole durable owner of this state: Sphinx and
//! other orchestration must never keep canonical checkpoints in local files,
//! S3 or CI state.
//!
//! This slice establishes the durable columns only. The operation-level
//! concurrency protocol — claim tokens, lease acquisition/release,
//! `FOR UPDATE SKIP LOCKED`, stale-lease recovery, retries — is deliberately
//! **not** implemented or modelled here; it belongs to the later bounded
//! internal claim/checkpoint API task, together with its own tests.
//!
//! `cursor` is generic nullable JSONB in the database. `MET-WP2-03` gives it
//! exactly one producer-owned, source-independent interpretation, the closed
//! `thoth-period-manifest-cursor/1` [`MetricPeriodManifestCursor`], and this
//! module is the only place its decoding, encoding and retention rules are
//! implemented. `updated_at` uses the repository-standard
//! `diesel_manage_updated_at` trigger; the approved design specifies no
//! `created_at` column.

use chrono::NaiveDate;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted metric-source-checkpoint row.
///
/// All progress, lease and error fields are nullable as designed: a fresh
/// checkpoint records only its identity. The database rejects blank
/// `partition_key` values and duplicate `(source_account_id, partition_key)`
/// pairs.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceCheckpoint {
    pub source_checkpoint_id: Uuid,
    pub source_account_id: Uuid,
    pub partition_key: String,
    pub cursor: Option<serde_json::Value>,
    pub last_discovered_at: Option<Timestamp>,
    pub last_completed_at: Option<Timestamp>,
    pub last_successful_period_end: Option<NaiveDate>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<Timestamp>,
    pub last_error: Option<String>,
    pub updated_at: Timestamp,
}

/// The only supported period-manifest cursor representation (`MET-WP2-03`).
pub const PERIOD_MANIFEST_CURSOR_SCHEMA: &str = "thoth-period-manifest-cursor/1";

/// The most periods one stored cursor retains (`MET-WP2-03`).
///
/// This is a storage-safety bound, not a lookback limit: a period outside the
/// retained entries is unknown and may be rediscovered, and its absence never
/// means unchanged.
pub const PERIOD_MANIFEST_CURSOR_MAX_ENTRIES: usize = 64;

/// The accepted source-manifest memory of one checkpoint (`MET-WP2-03`): for
/// each retained period, the SHA-256 digest of the deterministic source
/// manifest that a successful managed import recorded for it.
///
/// A value exists only as the result of [`decode`](Self::decode) or
/// [`record_accepted_manifest`](Self::record_accepted_manifest), so every value
/// holds 1 to [`PERIOD_MANIFEST_CURSOR_MAX_ENTRIES`] entries, strictly ascending
/// and therefore unique by period, each with a period that has a canonical
/// `YYYY-MM-DD` rendering and a digest of exactly 64 lowercase hexadecimal
/// characters. No accepted history is SQL `NULL`, represented by the absence of
/// a value (`None`), never by an empty cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricPeriodManifestCursor {
    entries: Vec<MetricPeriodManifestCursorEntry>,
}

/// One retained period and the manifest digest accepted for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricPeriodManifestCursorEntry {
    period_start: NaiveDate,
    manifest_digest: String,
}

/// A stored cursor, or a manifest to record, outside the closed
/// representation. It deliberately carries nothing of the offending value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricPeriodManifestCursorError;

impl std::fmt::Display for MetricPeriodManifestCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unsupported period-manifest cursor")
    }
}

impl std::error::Error for MetricPeriodManifestCursorError {}

/// The canonical ten-character `YYYY-MM-DD` rendering of `date`. Years outside
/// 0000 to 9999 render with a sign or a fifth digit and have none.
fn canonical_date(date: NaiveDate) -> Option<String> {
    let rendered = date.format("%Y-%m-%d").to_string();
    (rendered.len() == 10).then_some(rendered)
}

/// Parse a period only if rendering it back is byte-identical, so no
/// parser-normalized alternative spelling is accepted.
fn parse_canonical_date(value: &str) -> Option<NaiveDate> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
    (canonical_date(date)? == value).then_some(date)
}

fn is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl MetricPeriodManifestCursorEntry {
    /// First day of the retained period.
    pub fn period(&self) -> NaiveDate {
        self.period_start
    }

    /// The manifest digest accepted for the period.
    pub fn digest(&self) -> &str {
        &self.manifest_digest
    }
}

impl MetricPeriodManifestCursor {
    /// Decode one stored `metric_source_checkpoint.cursor` value.
    ///
    /// SQL `NULL` (`None`) is no accepted history. Otherwise exactly one form is
    /// supported: an object with exactly the members `schemaVersion`, equal to
    /// [`PERIOD_MANIFEST_CURSOR_SCHEMA`], and `entries`, an array of 1 to
    /// [`PERIOD_MANIFEST_CURSOR_MAX_ENTRIES`] objects with exactly the members
    /// `periodStart` and `manifestDigest`, strictly ascending by period.
    /// Anything else, including a JSON `null`, fails closed: it is never read
    /// as empty and never repaired.
    pub fn decode(
        stored: Option<&JsonValue>,
    ) -> Result<Option<Self>, MetricPeriodManifestCursorError> {
        let Some(stored) = stored else {
            return Ok(None);
        };
        let object = stored.as_object().ok_or(MetricPeriodManifestCursorError)?;
        if object.len() != 2
            || object.get("schemaVersion").and_then(JsonValue::as_str)
                != Some(PERIOD_MANIFEST_CURSOR_SCHEMA)
        {
            return Err(MetricPeriodManifestCursorError);
        }
        let stored_entries = object
            .get("entries")
            .and_then(JsonValue::as_array)
            .filter(|entries| {
                !entries.is_empty() && entries.len() <= PERIOD_MANIFEST_CURSOR_MAX_ENTRIES
            })
            .ok_or(MetricPeriodManifestCursorError)?;
        let mut entries: Vec<MetricPeriodManifestCursorEntry> =
            Vec::with_capacity(stored_entries.len());
        for stored_entry in stored_entries {
            let member = stored_entry
                .as_object()
                .filter(|member| member.len() == 2)
                .ok_or(MetricPeriodManifestCursorError)?;
            let period_start = member
                .get("periodStart")
                .and_then(JsonValue::as_str)
                .and_then(parse_canonical_date)
                .ok_or(MetricPeriodManifestCursorError)?;
            let manifest_digest = member
                .get("manifestDigest")
                .and_then(JsonValue::as_str)
                .filter(|digest| is_lowercase_sha256_hex(digest))
                .ok_or(MetricPeriodManifestCursorError)?;
            if entries
                .last()
                .is_some_and(|previous| previous.period_start >= period_start)
            {
                return Err(MetricPeriodManifestCursorError);
            }
            entries.push(MetricPeriodManifestCursorEntry {
                period_start,
                manifest_digest: manifest_digest.to_string(),
            });
        }
        Ok(Some(Self { entries }))
    }

    /// The canonical stored representation, built from the validated entries
    /// alone, so no member of any previously stored value is ever carried over.
    pub fn encode(&self) -> JsonValue {
        json!({
            "schemaVersion": PERIOD_MANIFEST_CURSOR_SCHEMA,
            "entries": self
                .entries
                .iter()
                .map(|entry| json!({
                    "periodStart": entry.period_start.format("%Y-%m-%d").to_string(),
                    "manifestDigest": entry.manifest_digest,
                }))
                .collect::<Vec<_>>(),
        })
    }

    /// The retained entries, strictly ascending by period.
    pub fn retained_entries(&self) -> &[MetricPeriodManifestCursorEntry] {
        &self.entries
    }

    /// The cursor after accepting `manifest_digest` for the period starting on
    /// `period_start`.
    ///
    /// The period's entry is inserted, or replaced when the period is already
    /// retained, keeping entries strictly ascending by period. When more than
    /// [`PERIOD_MANIFEST_CURSOR_MAX_ENTRIES`] entries result, the
    /// chronologically oldest are evicted. Retention depends on the period
    /// alone, never on insertion order or completion time, so a period older
    /// than every retained one is evicted at once. A period without a
    /// canonical rendering, or a digest outside the representation, fails
    /// closed, so the result always encodes to a value [`decode`](Self::decode)
    /// accepts.
    pub fn record_accepted_manifest(
        current: Option<&Self>,
        period_start: NaiveDate,
        manifest_digest: &str,
    ) -> Result<Self, MetricPeriodManifestCursorError> {
        if canonical_date(period_start).is_none() || !is_lowercase_sha256_hex(manifest_digest) {
            return Err(MetricPeriodManifestCursorError);
        }
        let mut entries = current
            .map(|cursor| cursor.entries.clone())
            .unwrap_or_default();
        match entries.binary_search_by_key(&period_start, |entry| entry.period_start) {
            Ok(index) => entries[index].manifest_digest = manifest_digest.to_string(),
            Err(index) => entries.insert(
                index,
                MetricPeriodManifestCursorEntry {
                    period_start,
                    manifest_digest: manifest_digest.to_string(),
                },
            ),
        }
        let excess = entries
            .len()
            .saturating_sub(PERIOD_MANIFEST_CURSOR_MAX_ENTRIES);
        entries.drain(..excess);
        Ok(Self { entries })
    }
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
