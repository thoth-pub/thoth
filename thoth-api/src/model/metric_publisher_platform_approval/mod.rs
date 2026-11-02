//! Publisher-platform approval foundation (`MET-WP1-06`).
//!
//! This module owns the persisted `metric_publisher_platform_approval`
//! model: one durable record of whether one publisher has an approval
//! relationship with one Metrics platform, and whether that approval
//! permits usage submissions, sales submissions, or both.
//!
//! `MET-WP1-06` is an inactive additive foundation. It seeds no approval row
//! and implements **no runtime behaviour**: there is no approval
//! creation/transition/revocation service, no `PUBLISHER_CONTROLLED`
//! platform-ownership enforcement, no package/capability entitlement check
//! and no publisher-import authorization. Those belong to later bounded
//! WP3/WP5 work. The module exposes no GraphQL or administration surface,
//! so the enum below is deliberately **not** a `juniper::GraphQLEnum`.
//!
//! `approved_by` is preserved from the approved design as nullable `UUID`
//! with deliberately **no** foreign key, no ZITADEL-string-to-UUID
//! conversion rule and no invented user/account identity model: current
//! Thoth authentication exposes authenticated actor identity as
//! string-based application identity, and no canonical local user
//! table/relationship has been approved for this field. A later separately
//! reviewed administrative approval write-path specification must resolve
//! actor/audit semantics before this field is populated by any write path.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// Whether a publisher-platform approval relationship is pending, approved
/// or revoked.
///
/// The inventory is closed and matches the approved design exactly. There is
/// deliberately no `OTHER` or `Default` variant: an unrecognised database,
/// serde or string value must fail rather than silently resolve to a nearest
/// state.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricPublisherPlatformApprovalStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricPublisherPlatformApprovalStatus {
    /// The relationship has been proposed but not yet decided.
    #[cfg_attr(feature = "backend", db_rename = "PENDING")]
    Pending,
    /// The relationship is currently approved.
    #[cfg_attr(feature = "backend", db_rename = "APPROVED")]
    Approved,
    /// A previously approved relationship has been revoked.
    #[cfg_attr(feature = "backend", db_rename = "REVOKED")]
    Revoked,
}

/// One persisted publisher-platform approval row.
///
/// The database enforces non-null `usage_submission_enabled`,
/// `sales_submission_enabled` and `approval_status`, and exactly one row per
/// `(publisher_id, platform_id)`. Every foreign key — to canonical
/// `publisher` and to `metric_platform` — is direct and non-cascading, so
/// deleting a referenced publisher or platform fails instead of silently
/// deleting approval/audit state. There is deliberately no
/// approval-specific secondary index beyond the primary key and the
/// `(publisher_id, platform_id)` uniqueness index at this foundation stage.
///
/// `usage_submission_enabled` and `sales_submission_enabled` are
/// independently representable, non-null booleans with no invented default.
/// `approved_by`, `approved_at` and `notes` remain nullable, with no
/// invented default; `approved_by` deliberately carries no foreign key (see
/// the module documentation).
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricPublisherPlatformApproval {
    pub publisher_platform_approval_id: Uuid,
    pub publisher_id: Uuid,
    pub platform_id: Uuid,
    pub usage_submission_enabled: bool,
    pub sales_submission_enabled: bool,
    pub approval_status: MetricPublisherPlatformApprovalStatus,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<Timestamp>,
    pub notes: Option<String>,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
