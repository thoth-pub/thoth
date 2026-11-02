//! The canonical inbound OPERAS import ledger (`MET-WP1-10`).
//!
//! This module owns the persisted `metric_operas_import` model: the durable
//! record of one remote OPERAS event observed on one remote OPERAS instance,
//! together with the identity of the payload that event carried and an
//! optional link to the canonical `MET-WP1-03` import that normalized it. The
//! approved Metrics design assigns the OPERAS synchronization ledgers to
//! Thoth, and this row is the durable remote-event evidence a later inbound
//! synchronization path must record *before* normalization.
//!
//! `MET-WP1-10` stores inbound-ledger structure only. It implements **no**
//! OPERAS network or API access, provider or runtime inspection, discovery
//! cursor, rolling scan, snapshot import, remote polling or scheduling,
//! normalization or metric ingestion, automatic creation or completion of a
//! `metric_import`, `direct_collection` eligibility enforcement,
//! configured-uploader matching, Thoth-export echo detection, linking or
//! skipping, loop-prevention behaviour, payload-divergence handling,
//! reconciliation run or issue, inbound status vocabulary or transition graph,
//! worker claim, lease, retry or `FOR UPDATE SKIP LOCKED` logic: those belong
//! to the later bounded WP9 work. Nothing creates or reads an inbound-ledger
//! row at runtime, and the module exposes no GraphQL, authorization or
//! administration surface. The `metric_reconciliation_run` and
//! `metric_reconciliation_issue` ledgers remain approved future architecture
//! and are deliberately not created by this slice.
//!
//! **Inbound-completeness boundary (section 15.5, reviewed and load-bearing.)**
//! The existence of this ledger does **not** imply guaranteed inbound
//! discovery. Guaranteed inbound completeness remains *externally blocked*
//! without an adequate cursor or created-at event stream, replication, a
//! complete snapshot/export, or an equivalent reliable incremental mechanism,
//! and nothing in this slice removes that blocker. Accordingly this table adds
//! no cursor field, no remote-created-at field and no scan or snapshot
//! identifier; it implements no rolling-scan or snapshot behaviour; and it
//! performs no provider or API access. WP9 owns inbound discovery modes, loop
//! prevention, reconciliation and completeness reporting, and must surface
//! unverified completeness rather than claim it. A populated
//! `metric_operas_import` is evidence only of the remote events that were
//! actually observed and recorded — never evidence that all of them were.
//!
//! **Indexing boundary (section 14.4, reviewed.)** The design's generic
//! requirement for import status and creation-time indexing is already
//! satisfied by the merged `metric_import_status_created_at_idx` on
//! `metric_import`. `MET-WP1-10` therefore carries no outstanding OPERAS-import
//! operational index requirement and adds no speculative secondary index. WP9
//! may add operational indexes only from actual query and query-plan evidence.
//!
//! ADR-0001 remains the entitlement authority: later OPERAS synchronization
//! must consume the shared capability machinery through
//! `ThothPackage`/`PublisherCapability` rather than any Metrics-specific
//! entitlement state, and nothing here evaluates a capability. ADR-0002
//! likewise remains binding: `MetricPlatform` is not `DistributionPlatform`,
//! and this module introduces no conversion between them. Under ADR-0002 and
//! the approved design, Sphinx stays stateless orchestration and holds no
//! direct canonical database authority; this slice grants it none and creates
//! no contract it could consume.
//!
//! No inbound-ledger row is seeded, and no real OPERAS instance, event
//! identifier, payload hash or status value is approved or guessed.

use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted inbound OPERAS import-ledger row.
///
/// Identity is the composite `(remote_instance, remote_event_id)` primary key
/// and nothing else. The approved design names no surrogate inbound-ledger ID
/// and deliberately carries `remote_instance` alongside `remote_event_id`, so
/// a bare remote event identifier is **not** established as globally unique:
/// the same `remote_event_id` stays representable for two distinct remote
/// instances, while one remote event observed repeatedly resolves to the same
/// durable row rather than creating duplicate remote-event evidence.
///
/// `remote_instance`, `remote_event_id`, `payload_hash` and `status` are
/// required `String` carrying only the existing Metrics required-text CHECK,
/// which rejects blank and whitespace-only values. No URI, hostname, tenant,
/// environment, enum, registry, normalization or case-folding rule constrains
/// `remote_instance`; no syntax, length, UUID/URI or global-uniqueness rule
/// constrains `remote_event_id`; no algorithm, encoding, case, length or
/// uniqueness rule constrains `payload_hash`; and no vocabulary, default,
/// transition graph, trigger or cross-column rule constrains `status`. A
/// stored value is evidence of nonblank text and never of a recognised remote
/// instance, a valid remote event identifier, a computed payload hash or a
/// recognised state. WP9 owns those semantics.
///
/// `payload_hash` is deliberately **not** unique. The approved design requires
/// the payload hash to be stored before normalization so a later changed
/// payload for an already-known remote identity can become a
/// divergence/reconciliation outcome; two genuinely different remote events
/// may legitimately carry equal payload content, and forbidding that would
/// make ordinary duplicate content unrepresentable.
///
/// `import_id` is nullable and non-unique, with a single-column non-cascading
/// foreign key to `metric_import (import_id)`. It is nullable because the
/// approved design requires the remote event and its payload hash to be
/// recorded *before* normalization, and because an event that is linked or
/// skipped for loop prevention may never require a canonical import job of its
/// own; a durable remote-event row must therefore be able to exist before any
/// `metric_import` does. It is non-unique because one `metric_import` may
/// represent an API response or batch containing many distinct remote events,
/// so several inbound-ledger rows must be able to reference the same import.
/// The database decides neither when `import_id` is populated nor whether a
/// particular `status` requires it. Non-cascading deletion means removing a
/// canonical import while durable remote-event evidence references it fails
/// rather than silently erasing that evidence.
///
/// `created_at` uses the repository-standard current-time default and records
/// when Thoth stored the row. It is deliberately the only timestamp: no
/// remote-created-at, discovery, scan, snapshot, normalized-at, updated-at or
/// completion timestamp is added, because the approved design names none and
/// because such a field would imply the inbound discovery semantics section
/// 15.5 leaves externally blocked and WP9-owned.
///
/// The row carries no relationship to `metric_operas_export` and duplicates no
/// export, platform, measure or mapping identifier. Loop prevention stays WP9
/// runtime and reconciliation logic rather than a stored relational identity.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricOperasImport {
    pub remote_instance: String,
    pub remote_event_id: String,
    pub payload_hash: String,
    pub import_id: Option<Uuid>,
    pub status: String,
    pub created_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
