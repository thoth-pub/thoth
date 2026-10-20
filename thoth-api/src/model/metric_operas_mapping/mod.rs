//! Configured Thoth-to-OPERAS platform/measure mappings (`MET-WP1-08`).
//!
//! This module owns the persisted `metric_operas_mapping` model: the canonical
//! durable configuration that names, for one registered Metrics
//! platform/measure pair, the OPERAS event, measure and uploader URIs to use
//! and whether that mapping is enabled. The approved Metrics design treats
//! `event_uri` as pre-registered mapping configuration rather than a
//! per-record value, and later outbound eligibility requires an enabled OPERAS
//! mapping for the platform/measure.
//!
//! `MET-WP1-08` stores mapping state only. It implements **no** OPERAS export,
//! import or reconciliation ledger, no payload construction or projection, no
//! delivery, no claiming, lease, attempt, retry, backoff or status state
//! machine, no remote event ID, request hash or delivery error, no inbound
//! synchronization or loop prevention, no reconciliation, no cursor/snapshot
//! discovery, and no outbound eligibility or capability enforcement: those
//! belong to the later bounded WP5/WP9 work. The `metric_operas_export`,
//! `metric_operas_import`, `metric_reconciliation_run` and
//! `metric_reconciliation_issue` ledgers remain approved future architecture
//! and are deliberately not created by this slice. The module exposes no
//! GraphQL, authorization or administration surface.
//!
//! ADR-0001 remains the entitlement authority: later OPERAS export eligibility
//! must consume the shared `METRICS_OPERAS_EXPORT` capability through
//! `ThothPackage`/`PublisherCapability` rather than any Metrics-specific
//! entitlement state, and nothing here evaluates a capability. ADR-0002
//! likewise remains binding: `MetricPlatform` is not `DistributionPlatform`,
//! and this module introduces no name-based or enum-order conversion between
//! them.
//!
//! No mapping is seeded. Real OPERAS event, measure and uploader URI values
//! remain unapproved external inputs, recorded as unresolved in
//! `docs/metrics/source-inventory.md`, and this slice must not guess them.

use uuid::Uuid;

/// One persisted Thoth-to-OPERAS platform/measure mapping row.
///
/// The database enforces `UNIQUE(platform_id, measure_id)`, so at most one
/// canonical OPERAS mapping exists per registered platform/measure pair and
/// later `mapping_id` selection cannot be ambiguous. A composite foreign key
/// over `(platform_id, measure_id)` against the MET-WP1-01
/// `metric_platform_measure (platform_id, measure_id)` unique key guarantees a
/// mapping names a pair already admitted to the Metrics registry, rather than
/// a real platform and a real measure that are not registered together. That
/// foreign key is non-cascading, so deleting a registry pair that still has
/// mapping configuration fails instead of silently erasing it.
///
/// `mapping_id` is a stable surrogate identity: the approved design's
/// shorthand mapping list names no primary key, but its later conceptual
/// `metric_operas_export` row refers to a `mapping_id`, and a surrogate key
/// supplies that referential target without forcing an export row to repeat
/// mutable configuration text.
///
/// `event_uri`, `measure_uri` and `uploader_uri` are plain required `String`
/// configuration text. The database rejects blank and whitespace-only values
/// through the existing Metrics required-text idiom, and that is the **only**
/// integrity rule applied: there is deliberately no URI scheme restriction, no
/// parsing or normalization, no hostname rule, no trailing-slash handling, no
/// remote validation and no uniqueness on any URI column, so nothing here may
/// be read as a claim that a stored value is a valid or reachable OPERAS URI.
///
/// `enabled` is a required `bool` with deliberately **no** database default: a
/// mapping's activation state must be stated explicitly by whatever later
/// reviewed administrative write path creates it. Nothing in this slice
/// consumes it, and `metric_platform_measure.direct_collection` — not this
/// table — remains the canonical direct-collection flag.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricOperasMapping {
    pub mapping_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub event_uri: String,
    pub measure_uri: String,
    pub uploader_uri: String,
    pub enabled: bool,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
