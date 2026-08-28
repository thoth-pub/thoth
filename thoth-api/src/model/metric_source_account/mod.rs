//! Metric source accounts (`MET-WP1-02`).
//!
//! This module owns the persisted `metric_source_account` model: one concrete
//! partition/account of a [`metric_source`](crate::model::metric_source),
//! routed to the [`metric_platform`](crate::model::metric_platform) on which
//! its activity was observed. Account identity is `(source_id, external_key)`
//! and is enforced by the database.
//!
//! `configuration` is generic **non-secret** routing/configuration JSON only.
//! Credentials must never be stored in it; because this slice ships no
//! application write path, allowed-field validation belongs to the later
//! protected source-account administration specification, and no
//! source-specific JSON schema is imposed on this generic table.
//!
//! `MET-WP1-02` is an inactive additive foundation: it seeds no account row,
//! infers no account from existing platform assignments or operational
//! configuration, and exposes no GraphQL or administration surface. The
//! foreign keys to source, platform and (optionally) publisher are
//! deliberately non-cascading.

use uuid::Uuid;

/// One persisted metric-source-account row.
///
/// `external_key` is the source-side partition/account identifier; the
/// database rejects blank keys and duplicate `(source_id, external_key)`
/// pairs. `expected_publisher_id` optionally pins the canonical publisher the
/// account is expected to report for. The approved design deliberately omits
/// `created_at`/`updated_at` on this table.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceAccount {
    pub source_account_id: Uuid,
    pub source_id: Uuid,
    pub platform_id: Uuid,
    pub external_key: String,
    pub expected_publisher_id: Option<Uuid>,
    pub configuration: serde_json::Value,
    pub enabled: bool,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
