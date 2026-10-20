//! The canonical reconciliation run ledger (`MET-WP1-11`).
//!
//! This module owns the persisted `metric_reconciliation_run` model: the
//! durable record of one reconciliation execution, what it covered and how it
//! ended. The approved Metrics design makes Thoth the sole canonical owner of
//! durable Metrics state and reconciliation outcomes, and Sphinx performs
//! reconciliation orchestration while the durable evidence stays here.
//!
//! `MET-WP1-11` stores reconciliation structure only. It implements **no**
//! reconciliation execution, comparison of source manifests, canonical
//! records, rollups or OPERAS ledgers, runtime creation of a run or an issue,
//! issue classification, closed status/type/severity vocabulary, resolution or
//! reopening workflow, OPERAS loop prevention, divergence handling, snapshot or
//! rolling scan, completeness determination, claim, lease, retry or backoff
//! behaviour, and no `recordMetricReconciliation` or any other API operation:
//! those belong to the later bounded WP9 work. Nothing creates or reads a
//! reconciliation row at runtime, and the module exposes no GraphQL,
//! authorization or administration surface.
//!
//! **Completeness boundary (section 15.5, reviewed and load-bearing.)**
//! Persisting reconciliation runs and issues does **not** solve, weaken or
//! narrow the OPERAS inbound-completeness blocker. Guaranteed inbound
//! completeness remains *externally blocked* without an adequate cursor or
//! created-at event stream, replication, a complete snapshot/export, or an
//! equivalent reliable incremental mechanism. This slice therefore adds no
//! completeness flag, coverage assertion, cursor, scan or snapshot identifier
//! and determines no completeness. A populated reconciliation ledger would be
//! evidence only of the comparisons a later WP9 runtime actually performed —
//! never evidence that reconciliation was complete. WP9 owns completeness
//! reporting and must surface unverified completeness rather than claim it.
//!
//! **Indexing boundary (reviewed and closed.)** The complete `MET-WP1-11`
//! index inventory is the primary key of this table and the primary key of
//! [`crate::model::metric_reconciliation_issue`], and nothing else. No
//! secondary index is authorized on any reconciliation column in this slice;
//! WP9 may add one only from an actual access pattern with query-plan
//! evidence.
//!
//! ADR-0001 remains the entitlement authority: later reconciliation work must
//! consume the shared capability machinery through
//! `ThothPackage`/`PublisherCapability` rather than any Metrics-specific
//! entitlement state, and nothing here evaluates a capability. ADR-0002
//! likewise remains binding: `MetricPlatform` is not `DistributionPlatform`,
//! and this module introduces no conversion between them. Under ADR-0002 and
//! the approved design, Sphinx stays stateless orchestration and holds no
//! direct canonical database authority; this slice grants it none and creates
//! no contract it could consume.
//!
//! No reconciliation run is seeded, and no real reconciliation scope, status or
//! summary value is approved or guessed.

use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted reconciliation run.
///
/// `run_id` is the repository-standard Metrics UUID surrogate primary key with
/// the standard generation default. The approved design names a standalone
/// `run_id` and defines no natural composite identity, and no externally
/// supplied reconciliation-run identifier is part of the approved contract, so
/// no secondary natural-key uniqueness is invented.
///
/// `scope` is required generic JSONB with no database-level schema and no
/// default: a run must state what it covered, and this slice defines no
/// default coverage. Reconciliation may cover different combinations of
/// source, canonical, rollup and OPERAS state, so no fixed scalar vocabulary,
/// required key, source/platform/measure identifier or completeness flag is
/// imposed. `summary` is required generic JSONB defaulting to an empty object,
/// matching the merged `metric_import.manifest` and
/// `metric_record_provenance.details` idiom, so a run exists before its
/// machine-readable outcome is known; it defines no required keys and gives an
/// empty object no semantic interpretation.
///
/// `status` is required `String` carrying only the existing Metrics
/// required-text CHECK, which rejects blank and whitespace-only values. There
/// is no PostgreSQL enum, closed vocabulary, default, trigger or transition
/// graph: the approved design names `status` but supplies no closed states, so
/// a stored value is evidence of nonblank text and never of a recognised
/// reconciliation state. WP9 owns that vocabulary.
///
/// `started_at` is required and deliberately carries **no** database default,
/// unlike the repository-standard current-time `created_at` used elsewhere in
/// Metrics. It is the actual reconciliation-execution start supplied by the
/// writer, and the approved design does not establish that Thoth's durable
/// insertion time and the reconciliation execution start are the same event,
/// so the database must never silently substitute the current time: an insert
/// omitting it fails, and an explicitly supplied value round-trips exactly.
/// `completed_at` is optional with no default, because the row exists before
/// completion, and no status/timestamp state-machine invariant ties the two
/// together — the status transition graph is not design-fixed and remains
/// WP9-owned. No `created_at`, `updated_at`, retry, lease or heartbeat
/// timestamp is added.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricReconciliationRun {
    pub run_id: Uuid,
    pub scope: serde_json::Value,
    pub status: String,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub summary: serde_json::Value,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
