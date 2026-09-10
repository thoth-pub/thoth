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
//! `MET-WP2-01A` later added the globally unique stable [`code`] required by
//! the canonical ingestion contract, introduced additively over an already
//! populated table: pre-existing rows were deterministically backfilled with
//! their own `source_account_id::text`, which is migration identity for those
//! rows only and prescribes nothing about how a real stable code is chosen.
//! Rows created after that migration must supply an explicit code. Neither
//! `external_key` nor source identity was redefined.
//!
//! [`code`]: MetricSourceAccount::code
//!
//! `MET-WP1-02` is an inactive additive foundation: it seeds no account row,
//! infers no account from existing platform assignments or operational
//! configuration, and exposes no GraphQL or administration surface. The
//! foreign keys to source, platform and (optionally) publisher are
//! deliberately non-cascading.

use uuid::Uuid;

/// One persisted metric-source-account row.
///
/// `code` is the globally unique stable account identifier required by
/// `thoth-normalized-metrics/1` (`MET-WP2-01A`). It is exact PostgreSQL
/// `TEXT`: the database rejects blank and duplicate codes, and nothing —
/// here or at any later entry point — may trim, case-fold,
/// Unicode-normalize or alias it. It sits alongside, and does not replace,
/// the source-scoped `(source_id, external_key)` identity below.
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
    pub code: String,
    pub source_id: Uuid,
    pub platform_id: Uuid,
    pub external_key: String,
    pub expected_publisher_id: Option<Uuid>,
    pub configuration: serde_json::Value,
    pub enabled: bool,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
