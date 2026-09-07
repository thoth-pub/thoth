//! The canonical reconciliation issue ledger (`MET-WP1-11`).
//!
//! This module owns the persisted `metric_reconciliation_issue` model: one
//! machine-readable reconciliation finding belonging to exactly one
//! [`crate::model::metric_reconciliation_run`]. The approved Metrics design
//! makes Thoth the sole canonical owner of durable reconciliation outcomes,
//! and requires reconciliation to produce machine-readable issues; this row is
//! that durable evidence.
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
//! **No remote-event relationship (reviewed and load-bearing.)** There is
//! deliberately no foreign key from `remote_event_id` to the merged
//! `metric_operas_import` or `metric_operas_export` ledgers, and no global
//! uniqueness on it. `MET-WP1-10` established canonical remote identity as the
//! composite `(remote_instance, remote_event_id)` and deliberately established
//! that a bare `remote_event_id` is **not** globally unique. The approved
//! reconciliation shorthand carries no `remote_instance`, and reconciliation
//! may also refer to outbound or legacy remote evidence, so a single-column key
//! to the inbound ledger would contradict that merged identity contract and
//! could make later reconciliation ambiguous or falsely authoritative. No
//! `remote_instance`, `operas_import_id`, `export_id`, `mapping_id` or
//! `record_revision_id` column is added to manufacture a relationship the
//! approved design does not name; a later WP9 runtime may carry richer evidence
//! inside `details` and may propose an additive amendment only from a concrete
//! demonstrated access or integrity requirement.
//!
//! **Indexing boundary (reviewed and closed.)** The complete `MET-WP1-11`
//! index inventory is this table's primary key and the reconciliation run's
//! primary key, and nothing else — in particular there is no index on `run_id`,
//! `record_id`, `remote_event_id`, `issue_type`, `severity` or `resolved_at`.
//! PostgreSQL does not require a child-side referencing index to enforce these
//! foreign keys during ordinary child insert or update, the merged
//! `metric_import_error (import_id)` precedent likewise carries none, and this
//! slice has no reconciliation reader or writer whose query plan could justify
//! one. WP9 owns any later operational indexing.
//!
//! ADR-0001 remains the entitlement authority and ADR-0002 remains binding, as
//! for the reconciliation run: nothing here evaluates a capability, converts
//! `MetricPlatform` to `DistributionPlatform`, or grants Sphinx direct
//! canonical database authority.
//!
//! No reconciliation issue is seeded, and no real issue type, severity, remote
//! event identifier or detail payload is approved or guessed.

use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted reconciliation issue.
///
/// `issue_id` is the repository-standard Metrics UUID surrogate primary key
/// with the standard generation default. No uniqueness over
/// `(run_id, issue_type, record_id, remote_event_id)` or any other inferred
/// identity is added, because runtime deduplication and reopening semantics
/// are not fixed by the approved design and remain WP9-owned.
///
/// `run_id` is required, with a single-column non-cascading foreign key to
/// `metric_reconciliation_run (run_id)`. Every issue is durable evidence
/// belonging to exactly one run, and deleting a run while its issues exist
/// fails rather than silently cascade-deleting that evidence.
///
/// `issue_type` and `severity` are required `String` carrying only the
/// existing Metrics required-text CHECK, which rejects blank and
/// whitespace-only values. Neither has a PostgreSQL enum, closed vocabulary,
/// default or transition rule: the approved design's prose examples — missing
/// export, unexpected remote record, value divergence, unmapped measure,
/// unresolved work, late source change, stale coverage — are illustrative and
/// are not an exhaustive enum, and its operational mention of high-severity
/// reconciliation alerts does not define a closed severity domain. A stored
/// value is evidence of nonblank text and never of a recognised issue class or
/// severity level.
///
/// `record_id` is optional, with a single-column non-cascading foreign key to
/// `metric_record (record_id)` when present. Optional, because several
/// design-named issue categories can exist before or without a canonical
/// record — unexpected remote records, unmapped measures and unresolved works
/// — so requiring it would make legitimate reconciliation evidence
/// unrepresentable. Referentially enforced when supplied, so a named record
/// cannot be a nonexistent one. No key to `metric_record_revision` is added,
/// because the approved shorthand names `record_id`, not `record_revision_id`.
///
/// `remote_event_id` is optional opaque text carrying the nullable form of the
/// required-text idiom — `None`, or at least one non-whitespace character —
/// and nothing stronger: no syntax, length or uniqueness rule, and no foreign
/// key, as documented on the module above.
///
/// `details` is required generic JSONB defaulting to an empty object, matching
/// the merged `metric_record_provenance.details` idiom, so an issue is
/// machine-readable without an unapproved universal evidence column set. It
/// defines no required keys and gives an empty object no semantic
/// interpretation.
///
/// `resolved_at` is the design-named optional resolution timestamp and nothing
/// more. No `resolved_by`, `resolution`, issue `status`, `updated_at`,
/// reopening counter or cross-column invariant is invented: this slice defines
/// neither what "resolved" means nor whether an issue may reopen.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricReconciliationIssue {
    pub issue_id: Uuid,
    pub run_id: Uuid,
    pub issue_type: String,
    pub severity: String,
    pub record_id: Option<Uuid>,
    pub remote_event_id: Option<String>,
    pub details: serde_json::Value,
    pub resolved_at: Option<Timestamp>,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
