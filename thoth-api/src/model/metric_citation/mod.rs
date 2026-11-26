//! Citation links (`CITATIONS-01`, thoth#1074), slice C1: storage only.
//!
//! Five tables hold the works that cite a Thoth work, as reported by a
//! source (v1: OpenAlex):
//!
//! - `metric_citation_work_state`: one row per (cited work, source) with the
//!   outcome of the last attempt that resolved the DOI, the last MATCHED
//!   values (a later `NOT_FOUND` or failed attempt never erases them), the
//!   enumerated link count and the `unconfirmed` flag;
//! - `metric_citation_link`: one row per (cited work, source, citing work),
//!   `ACTIVE` or `WITHDRAWN`, never deleted;
//! - `metric_citation_refresh`, `metric_citation_refresh_seen` and
//!   `metric_citation_batch`: the audit of a claimed refresh run, the citing
//!   ids it has seen per work, and replay safety for its batches.
//!
//! This slice adds no write path, no read and no GraphQL surface: those are
//! C2 and C3. The registry (platform, source, source account) is created
//! after deploy through the registry write path.
//!
//! All text columns hold plain text. Thoth's write path (C2) sanitises and
//! truncates third-party text before it reaches these columns; the CHECK
//! constraints are the last line of defence, not the sanitiser.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::model::Timestamp;

/// Outcome of the last attempt that resolved a work's DOI at the source.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricCitationMatchStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricCitationMatchStatus {
    #[cfg_attr(feature = "backend", db_rename = "MATCHED")]
    Matched,
    #[cfg_attr(feature = "backend", db_rename = "NOT_FOUND")]
    NotFound,
}

/// Whether a citation link is currently listed by its source.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricCitationLinkStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricCitationLinkStatus {
    #[cfg_attr(feature = "backend", db_rename = "ACTIVE")]
    Active,
    #[cfg_attr(feature = "backend", db_rename = "WITHDRAWN")]
    Withdrawn,
}

/// Lifecycle of one claimed refresh run.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricCitationRefreshStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricCitationRefreshStatus {
    #[cfg_attr(feature = "backend", db_rename = "RUNNING")]
    Running,
    #[cfg_attr(feature = "backend", db_rename = "COMPLETED")]
    Completed,
    #[cfg_attr(feature = "backend", db_rename = "COMPLETED_WITH_ERRORS")]
    CompletedWithErrors,
    #[cfg_attr(feature = "backend", db_rename = "FAILED")]
    Failed,
}

/// One `metric_citation_work_state` row.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCitationWorkState {
    pub cited_work_id: Uuid,
    pub source_id: Uuid,
    pub match_status: MetricCitationMatchStatus,
    pub last_matched_at: Option<Timestamp>,
    pub external_work_id: Option<String>,
    pub reported_count: Option<i32>,
    pub enumerated_count: Option<i32>,
    pub guard_held_count: Option<i32>,
    pub unconfirmed: bool,
    pub last_attempted_at: Option<Timestamp>,
    pub last_refreshed_at: Option<Timestamp>,
    pub last_error_at: Option<Timestamp>,
    pub last_refresh_id: Option<Uuid>,
}

/// One `metric_citation_link` row.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq)]
pub struct MetricCitationLink {
    pub link_id: Uuid,
    pub cited_work_id: Uuid,
    pub source_id: Uuid,
    pub citing_external_id: String,
    pub citing_doi: Option<String>,
    pub citing_title: Option<String>,
    pub citing_type: Option<String>,
    pub citing_publication_year: Option<i16>,
    pub citing_publication_date: Option<NaiveDate>,
    pub citing_sort_date: NaiveDate,
    pub citing_venue: Option<String>,
    pub citing_authors: Option<serde_json::Value>,
    pub citing_url: Option<String>,
    pub status: MetricCitationLinkStatus,
    pub first_seen_at: Timestamp,
    pub last_seen_at: Timestamp,
    pub withdrawn_at: Option<Timestamp>,
    pub last_refresh_id: Option<Uuid>,
}

#[cfg(all(test, feature = "backend"))]
mod tests;
