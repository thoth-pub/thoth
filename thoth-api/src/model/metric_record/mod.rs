//! Canonical metric records (`MET-WP1-04`).
//!
//! This module owns the persisted `metric_record` model: the stable identity
//! of one canonical reporting cell. A record fixes the platform, measure,
//! work, optional publication, half-open reporting period and optional
//! country/institution dimensions that together identify what is being
//! counted, together with the source account whose submission won first
//! arrival and a pointer to the record's current value revision.
//!
//! Per the approved Metrics design, canonical identity deliberately excludes
//! both the acquisition route and the value itself, so uniqueness is carried
//! by `identity_hash` rather than by a natural composite key. Current
//! publisher, imprint and series attribution is likewise **not** stored here:
//! it is derived from live Thoth metadata by later rollup and query design.
//!
//! `MET-WP1-04` is an inactive additive foundation. It seeds no record and
//! implements **no runtime behaviour**: there is no identity or content
//! hashing, no normalized-observation validation, no DOI/ISBN/ROR resolution,
//! no first-arrival, duplicate, revision, conflict or retraction transaction,
//! no managed-source revision authorization, no publisher finality, no rollup
//! delta and no period-overlap detection or concurrency protocol. Those belong
//! to the later bounded WP2 ingestion work. The module exposes no GraphQL or
//! administration surface, so nothing here is a `juniper::GraphQLEnum`.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::Timestamp;

/// One persisted canonical metric-record row.
///
/// The database enforces that `identity_hash` is unique and non-blank, that
/// the half-open period is ordered (`period_end > period_start`), and that
/// `country_code` — when supplied — is exactly two uppercase ASCII letters.
/// No hash algorithm, encoding or length rule is imposed in this slice, and
/// no full ISO 3166-1 alpha-2 membership check is performed: that belongs to
/// later WP2 normalized-observation validation.
///
/// `country_code` is a **separate** Metrics alpha-2 representation persisted
/// as `CHAR(2)`. It deliberately neither reuses nor changes Thoth's existing
/// bibliographic alpha-3 `CountryCode` enum.
///
/// Every foreign key is non-cascading, so deleting a referenced work,
/// publication, platform, measure, institution or source account fails rather
/// than silently deleting canonical history.
///
/// `current_revision_id` stays optional so a record row can be created before
/// its first revision inside a later WP2 transaction. When it is set, a
/// composite foreign key over `(record_id, current_revision_id)` guarantees
/// the named revision belongs to *this* record. Nothing in this slice ties
/// that pointer to a revision's status: WP2 owns the transaction that inserts
/// a revision, supersedes its predecessor and moves the pointer atomically.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRecord {
    pub record_id: Uuid,
    pub identity_hash: String,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub reporting_grain: MetricReportingGrain,
    pub country_code: Option<String>,
    pub institution_id: Option<Uuid>,
    pub winning_source_account_id: Uuid,
    pub current_revision_id: Option<Uuid>,
    pub first_received_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
