//! Durable unresolved-DOI quarantine (`MET-WP7-PREREQ-02`).
//!
//! This module owns the persisted `metric_identifier_quarantine` model: the
//! durable, reduced normalized evidence of one CloudFront observation whose DOI
//! is syntactically valid but resolves to no Thoth work. It is **not** a second
//! canonical Metrics store and **not** a classification ledger:
//!
//! - the observation remains a `REJECTED` / `UNKNOWN_DOI` row in
//!   [`metric_record_provenance`](crate::model::metric_record_provenance), which
//!   stays the sole authoritative per-row classification;
//! - it creates no `metric_record`, no `metric_record_revision` and no rollup
//!   delta, and it is counted in `metric_import.invalid_count`, so its import
//!   stays `COMPLETED_WITH_ERRORS`;
//! - it keeps only what a later, separately bounded reconciliation needs to
//!   revalidate the observation once its DOI resolves.
//!
//! Rows are written only by the backend canonical ingestion coordinator
//! (`metric_ingestion`), in the same transaction as the rejected provenance,
//! its sanitized import error and the invalid counter, and only when the
//! locked source's `driver_key` is exactly
//! [`CLOUDFRONT_DRIVER_KEY`](crate::model::metric_source_account::CLOUDFRONT_DRIVER_KEY)
//! and the observation carries none of `publication_isbn`, `publication_type`,
//! `institution_ror`, `source_record_id` or `source_row_number`. No caller flag
//! can request quarantine. The managed checkpoint update
//! (`metric_ingestion_lifecycle`) reads these rows, joined to provenance, as
//! evidence for its CloudFront quarantine-only manifest acceptance.
//!
//! Reconciliation, resolution state and read-quality warnings are deliberately
//! absent. The module exposes no GraphQL or administration surface, and the
//! table stores no request identity, raw log row or credential.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::Timestamp;

/// One persisted unresolved-DOI quarantine row.
///
/// `record_provenance_id` is unique: a quarantine row is the evidence of
/// exactly one `REJECTED` / `UNKNOWN_DOI` provenance row. `source_account_id`,
/// `platform_id` and `measure_id` are the locked canonical rows the coordinator
/// validated the observation against. `schema_version` is the batch envelope's
/// normalized schema version.
///
/// `work_doi` is the observation's DOI exactly as supplied after successful
/// application-level validation: it is not lowercased, re-prefixed or
/// otherwise rewritten, and the database deliberately imposes no DOI-format
/// rule on it. The period is half-open, and `country_code` is the validated
/// ISO 3166-1 alpha-2 code when the observation is country-level.
///
/// Every foreign key is non-cascading, and the database rejects blank
/// `schema_version`, `work_doi` and `methodology_version` values and any
/// period whose end does not follow its start.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricIdentifierQuarantine {
    pub identifier_quarantine_id: Uuid,
    pub record_provenance_id: Uuid,
    pub source_account_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub schema_version: String,
    pub work_doi: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub reporting_grain: MetricReportingGrain,
    pub country_code: Option<String>,
    pub value: i64,
    pub methodology_version: String,
    pub created_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
