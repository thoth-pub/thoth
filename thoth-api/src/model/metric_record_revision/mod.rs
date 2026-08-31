//! Canonical metric-record revisions (`MET-WP1-04`).
//!
//! This module owns the persisted `metric_record_revision` model: one
//! immutable version of the value held by a canonical metric record. The
//! approved Metrics design preserves canonical history by appending revisions
//! and provenance rather than by destructive replacement, so a managed-source
//! correction supersedes its predecessor instead of rewriting it.
//!
//! `MET-WP1-04` stores revision state only. It implements **no** transition
//! authorization, retraction command, managed-source correction policy,
//! rollup-delta generation or revision state-machine API, and deliberately
//! installs no trigger or stored procedure tying `status` to
//! `metric_record.current_revision_id`. WP2 owns the transaction that inserts
//! a revision, supersedes its predecessor and moves the record pointer
//! atomically. The module exposes no GraphQL or administration surface, so the
//! enum below is deliberately **not** a `juniper::GraphQLEnum`.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// The state of one canonical metric-record revision.
///
/// The inventory is closed and matches the approved design exactly. There is
/// deliberately no `OTHER`, `UNKNOWN` or `Default` variant: an unrecognised
/// database, serde or string value must fail rather than silently resolve to
/// a nearest state.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricRecordRevisionStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricRecordRevisionStatus {
    /// The revision currently carries the record's value. A partial unique
    /// index permits at most one of these per record.
    #[cfg_attr(feature = "backend", db_rename = "CURRENT")]
    Current,
    /// The revision has been replaced by a later revision of the same record.
    #[cfg_attr(feature = "backend", db_rename = "SUPERSEDED")]
    Superseded,
    /// The revision has been withdrawn. Retraction commands and their
    /// authorization are deliberately absent from this slice.
    #[cfg_attr(feature = "backend", db_rename = "RETRACTED")]
    Retracted,
}

/// One persisted canonical metric-record revision row.
///
/// The database enforces `revision_number > 0`, uniqueness of
/// `(record_id, revision_number)`, a non-blank `content_hash` and at most one
/// `CURRENT` revision per record. No hash algorithm, encoding or length rule
/// is imposed here.
///
/// `value` stays a signed `i64`. There is deliberately **no** blanket
/// non-negative constraint: usage measures reject negatives while sales
/// measures may report signed net units, so measure-specific validation
/// belongs to WP2 ingestion against `metric_measure.allow_negative` rather
/// than to a schema-wide rule.
///
/// `supersedes_revision_id` stays optional so an initial revision needs no
/// predecessor. When it is set, a self-referential composite foreign key over
/// `(record_id, supersedes_revision_id)` guarantees the superseded revision
/// belongs to the *same* record. Both foreign keys — to the owning record and
/// to the MET-WP1-03 import that produced the value — are non-cascading, so
/// durable canonical history cannot be erased through parent deletion.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRecordRevision {
    pub record_revision_id: Uuid,
    pub record_id: Uuid,
    pub revision_number: i32,
    pub import_id: Uuid,
    pub value: i64,
    pub content_hash: String,
    pub status: MetricRecordRevisionStatus,
    pub supersedes_revision_id: Option<Uuid>,
    pub created_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
