//! The `MIG-01` administrative audit/backfill facade.
//!
//! This module is the single, deliberately narrow, workspace-visible public
//! entry point for the separately gated `MIG-01` controlled historical backfill
//! (owning issue [#828]). It owns **all** domain-sensitive behaviour required by
//! that operation:
//!
//! - reading and parsing the immutable approved input manifest;
//! - resolving each manifest publisher against canonical Thoth publisher
//!   identity;
//! - computing the desired subscription package and the desired, linked-group
//!   normalized enabled [`DistributionPlatform`] set;
//! - reading current canonical service-configuration state and the
//!   optimistic-concurrency configuration-version token;
//! - producing a deterministic, canonical, raw-byte-hashed dry-run plan;
//! - deterministically classifying every plan entry on apply/resume;
//! - applying only the pending entries through the **existing** canonical
//!   coordinator ([`crud::replace_publisher_service_configuration`]) with a
//!   fixed [`PublisherServiceConfigurationSource::MigrationBackfill`] source;
//! - the production-mode job-state preflight;
//! - the licence audit and bounded reconciliation reporting.
//!
//! It introduces **no** independent package/platform/audit SQL write path: every
//! committed change still passes through the one canonical coordinator, which
//! remains `pub(crate)`. The facade fixes the source internally, so no caller can
//! supply an arbitrary provenance, and it exposes no generic configuration
//! writer, no lifecycle primitive and no authentication bypass.
//!
//! Gate B authorises implementation and disposable/local testing only. No
//! production data is read and nothing is executed against production here.
//!
//! [#828]: https://github.com/thoth-pub/thoth/issues/828

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use thoth_errors::{ThothError, ThothResult};

use super::crud::{migration_backfill_history, normalize_requested_platforms};
use super::{
    CanonicalServiceConfigurationState, PublisherServiceConfigurationSource,
    ReplacePublisherServiceConfigurationInput, ServiceConfigurationWriteContext,
};
use crate::db::PgPool;
use crate::model::distribution_job::DistributionJobCreation;
use crate::model::publisher::{Publisher, ThothPackage};
use crate::model::publisher_distribution_platform::{
    DistributionPlatform, DistributionPlatformGroup, PublisherDistributionPlatform,
};
use crate::model::{Crud, Timestamp};
use crate::schema::{
    distribution_job, distribution_job_attempt, distribution_job_target, imprint, publisher, work,
};

/// The only manifest schema version this Gate-B tool accepts.
pub const MANIFEST_VERSION: u32 = 1;

/// The only plan schema version this Gate-B tool emits or accepts.
pub const PLAN_SCHEMA_VERSION: u32 = 1;

/// The fixed audit-actor namespace prefix. The full actor is
/// `MIG-01:<lowercase-plan-sha256>` and is derived **after** the plan bytes are
/// hashed, so the actor is never embedded in the hashed plan.
pub const AUDIT_ACTOR_PREFIX: &str = "MIG-01:";

// ---------------------------------------------------------------------------
// MIG-01-local canonical timestamp adapter
// ---------------------------------------------------------------------------

/// The plan's timestamp representation: UTC RFC3339 with exactly six
/// fractional-second digits and a literal `Z`, e.g. `2026-08-18T12:34:56.123456Z`.
///
/// This is a MIG-01-local serialization adapter. It deliberately does **not**
/// touch the repository-wide [`Timestamp`] `Display`/`serde` behaviour: the
/// canonical audit JSON and this canonical plan intentionally serialize
/// timestamps differently, which is exactly why resume classification compares
/// typed values rather than JSON text. PostgreSQL `timestamptz` precision is
/// microseconds, so six fractional digits preserve the stored instant exactly.
mod plan_timestamp {
    use super::Timestamp;
    use chrono::{DateTime, SecondsFormat, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // `Timestamp` is defined in the ancestor `crate::model` module, so its
        // private inner `DateTime<Utc>` is reachable from this descendant module
        // without changing the repository-wide serializer or `model/mod.rs`.
        serializer.serialize_str(&value.0.to_rfc3339_opts(SecondsFormat::Micros, true))
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let parsed = DateTime::parse_from_rfc3339(&raw)
            .map_err(serde::de::Error::custom)?
            .with_timezone(&Utc);
        Ok(Timestamp(parsed))
    }
}

// ---------------------------------------------------------------------------
// Immutable approved input manifest
// ---------------------------------------------------------------------------

/// The approved, sanitized, immutable MIG-01 input manifest.
///
/// Its identity is the SHA-256 of its exact raw bytes; it is never normalized
/// before hashing. Unknown fields are rejected so a manifest cannot silently
/// carry unreviewed instructions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationManifest {
    pub manifest_version: u32,
    #[serde(default)]
    pub description: Option<String>,
    pub publishers: Vec<ManifestPublisherEntry>,
    /// Reviewed dispositions for licence values, keyed by the exact stored
    /// licence string. A licence value observed in the catalogue with no entry
    /// here is reported as `UNREVIEWED` and is a blocking finding for a real run.
    #[serde(default)]
    pub licence_dispositions: Vec<ManifestLicenceDisposition>,
}

/// One approved publisher mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestPublisherEntry {
    /// The canonical Thoth publisher identity. MIG-01 resolves against the
    /// canonical UUID rather than guessing from a non-unique legacy label.
    pub publisher_id: Uuid,
    pub subscription_package: ThothPackage,
    /// The approved desired enabled platform set, before linked-group
    /// normalization. Normalization is applied by the backend, not the manifest.
    pub enabled_distribution_platforms: Vec<DistributionPlatform>,
    /// Free-text provenance explaining the mapping. Never parsed; retained only
    /// for the human report.
    #[serde(default)]
    pub provenance: Option<String>,
}

/// A reviewed disposition for one licence value.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestLicenceDisposition {
    pub value: String,
    pub disposition: LicenceDisposition,
}

/// The reviewed disposition of a licence value. MIG-01 Gate B audits and
/// reports these; it never executes a production licence-normalization write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenceDisposition {
    /// A supported canonical open licence; no action required.
    Supported,
    /// Requires a separately specified/reviewed/authorized normalization write.
    Normalize,
    /// Explicitly deferred (insufficient alone to close MIG-01).
    Defer,
    /// Reviewed as not permitted.
    Reject,
}

impl MigrationManifest {
    /// Version-validate the manifest and reject ambiguous reviewed input.
    ///
    /// A duplicated licence-disposition value is rejected outright — even when
    /// both copies carry the same disposition — so exactly one unambiguous
    /// reviewed disposition exists per stored licence value and no ordering-
    /// dependent first-wins choice is ever made. This runs before any write.
    fn validate(&self) -> ThothResult<()> {
        if self.manifest_version != MANIFEST_VERSION {
            return Err(ThothError::MigrationBackfillManifestInvalid(format!(
                "unsupported manifest version {} (expected {MANIFEST_VERSION})",
                self.manifest_version
            )));
        }
        let mut seen: HashSet<&str> = HashSet::new();
        for entry in &self.licence_dispositions {
            if !seen.insert(entry.value.as_str()) {
                return Err(ThothError::MigrationBackfillManifestInvalid(format!(
                    "licence value {:?} has more than one reviewed disposition; a licence value \
                     may appear at most once in licenceDispositions",
                    entry.value
                )));
            }
        }
        Ok(())
    }

    /// The single reviewed disposition for a licence value, if the manifest
    /// declares one. Validation has already proven at most one entry per value,
    /// so this lookup is unambiguous and order-independent.
    fn disposition_for(&self, value: &str) -> Option<LicenceDisposition> {
        self.licence_dispositions
            .iter()
            .find(|entry| entry.value == value)
            .map(|entry| entry.disposition)
    }
}

// ---------------------------------------------------------------------------
// Canonical plan (schema v1)
// ---------------------------------------------------------------------------

/// The deterministic, canonical, raw-byte-hashed apply plan.
///
/// The field order below is the fixed schema-v1 declaration order and is the
/// serialized byte order; no unordered map iteration ever determines the bytes.
/// Serialized with compact `serde_json` (UTF-8, no BOM, no insignificant
/// whitespace, no trailing newline). Entries are ordered by ascending canonical
/// publisher UUID and platform arrays by [`DistributionPlatform::ALL`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationPlan {
    pub schema_version: u32,
    pub manifest_sha256: String,
    pub entries: Vec<PlanEntry>,
    pub expected: PlanExpected,
}

/// One publisher's reviewed plan entry. Field order is fixed schema-v1 order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanEntry {
    pub publisher_id: Uuid,
    #[serde(with = "plan_timestamp")]
    pub reviewed_configuration_version: Timestamp,
    pub before: PlanBeforeState,
    pub desired: PlanDesiredState,
    pub classification: PlanClassification,
    pub affected_work_count: i64,
}

/// The reviewed before-state. Field order is fixed schema-v1 order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanBeforeState {
    pub subscription_package: ThothPackage,
    pub enabled_distribution_platforms: Vec<DistributionPlatform>,
    #[serde(with = "plan_timestamp")]
    pub configuration_version: Timestamp,
}

/// The normalized desired state. Field order is fixed schema-v1 order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanDesiredState {
    pub subscription_package: ThothPackage,
    pub enabled_distribution_platforms: Vec<DistributionPlatform>,
}

/// The reviewed classification the plan records for an entry. The runtime resume
/// classes `ALREADY_APPLIED_BY_THIS_PLAN` and `DRIFT` are computed at apply time
/// and are never stored in the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanClassification {
    /// The reviewed plan classifies the publisher as a no-op.
    ReviewedNoop,
    /// The reviewed plan classifies the publisher as a change to apply.
    Pending,
}

/// The aggregate reviewed expectations. Field order is fixed schema-v1 order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanExpected {
    pub publishers_considered: i64,
    pub publishers_changing: i64,
    pub publishers_noop: i64,
    pub package_changes: i64,
    pub package_noops: i64,
    pub assignment_inserts: i64,
    pub assignment_enables: i64,
    pub assignment_disables: i64,
    pub assignment_noops: i64,
    pub audit_records: i64,
    pub distribution_jobs: i64,
    pub distribution_job_targets: i64,
    pub distribution_job_attempts: i64,
    pub affected_publishers: i64,
    pub affected_works: i64,
    pub max_works_per_publisher: i64,
}

/// Serialize a plan to its exact canonical bytes.
pub fn canonical_plan_bytes(plan: &MigrationPlan) -> ThothResult<Vec<u8>> {
    // Compact `serde_json` on structs emits fields in declaration order with no
    // insignificant whitespace and no trailing newline: exactly the canonical
    // contract. No map is ever serialized, so ordering is fully determined.
    serde_json::to_vec(plan).map_err(Into::into)
}

/// The lowercase-hexadecimal SHA-256 of exactly `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// The exact audit actor derived from a reviewed plan's SHA-256.
pub fn audit_actor(plan_sha256: &str) -> String {
    format!("{AUDIT_ACTOR_PREFIX}{plan_sha256}")
}

/// Parse canonical plan bytes and require them to be exactly canonical.
///
/// A BOM is rejected outright. A semantically valid but noncanonical encoding
/// (extra whitespace, reordered keys, differing timestamp precision) parses but
/// fails the byte-for-byte re-serialization check, so it can never reach a write.
pub fn parse_canonical_plan(bytes: &[u8]) -> ThothResult<MigrationPlan> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ThothError::MigrationBackfillNoncanonicalPlan);
    }
    let plan: MigrationPlan = serde_json::from_slice(bytes).map_err(|error| {
        ThothError::MigrationBackfillManifestInvalid(format!(
            "plan is not valid schema-v{PLAN_SCHEMA_VERSION} JSON: {error}"
        ))
    })?;
    if plan.schema_version != PLAN_SCHEMA_VERSION {
        return Err(ThothError::MigrationBackfillManifestInvalid(format!(
            "unsupported plan schema version {} (expected {PLAN_SCHEMA_VERSION})",
            plan.schema_version
        )));
    }
    let reserialized = canonical_plan_bytes(&plan)?;
    if reserialized != bytes {
        return Err(ThothError::MigrationBackfillNoncanonicalPlan);
    }
    Ok(plan)
}

// ---------------------------------------------------------------------------
// Runtime resume classification
// ---------------------------------------------------------------------------

/// The apply-time classification of a single reviewed plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeClassification {
    /// Reviewed no-op and current state still matches; no write.
    ReviewedNoop,
    /// Reviewed change, current token/state still match the reviewed before
    /// state and no matching completed application exists; eligible to write.
    Pending,
    /// A matching completed `MIGRATION_BACKFILL` application by this exact plan
    /// already exists and current state equals its after-state; skip.
    AlreadyAppliedByThisPlan,
    /// Anything else. STOP before any new write.
    Drift,
}

/// Classify one reviewed plan entry against current canonical state and the
/// durable `MIGRATION_BACKFILL` configuration history.
///
/// This uses typed [`CanonicalServiceConfigurationState`] comparison throughout,
/// including `configuration_version`; it never compares jsonb text.
pub fn classify_entry(
    db: &PgPool,
    entry: &PlanEntry,
    actor: &str,
) -> ThothResult<ResumeClassification> {
    let current = current_state(db, entry.publisher_id)?;
    let reviewed_before = CanonicalServiceConfigurationState {
        subscription_package: entry.before.subscription_package,
        enabled_distribution_platforms: entry.before.enabled_distribution_platforms.clone(),
        configuration_version: entry.before.configuration_version,
    };
    let token_matches = current.configuration_version == entry.reviewed_configuration_version;

    match entry.classification {
        PlanClassification::ReviewedNoop => {
            // A reviewed no-op requires that the current typed state still equals
            // both the reviewed before state and the desired state (which are
            // equal for a no-op) and that the token has not moved.
            let desired_matches = current.subscription_package
                == entry.desired.subscription_package
                && current.enabled_distribution_platforms
                    == entry.desired.enabled_distribution_platforms;
            if token_matches && current == reviewed_before && desired_matches {
                Ok(ResumeClassification::ReviewedNoop)
            } else {
                Ok(ResumeClassification::Drift)
            }
        }
        PlanClassification::Pending => {
            if already_applied_by_this_plan(db, entry, actor, &current)? {
                Ok(ResumeClassification::AlreadyAppliedByThisPlan)
            } else if token_matches && current == reviewed_before {
                Ok(ResumeClassification::Pending)
            } else {
                Ok(ResumeClassification::Drift)
            }
        }
    }
}

/// Whether a durable `MIGRATION_BACKFILL` history row proves this exact reviewed
/// plan already applied this publisher and current state matches that outcome.
///
/// Requires all of: the exact derived actor; a before-state equal to the reviewed
/// before state including configuration version; an after-state whose package and
/// platform set equal the reviewed desired state; and current typed state,
/// including the current configuration version, equal to that after-state.
fn already_applied_by_this_plan(
    db: &PgPool,
    entry: &PlanEntry,
    actor: &str,
    current: &CanonicalServiceConfigurationState,
) -> ThothResult<bool> {
    let reviewed_before = CanonicalServiceConfigurationState {
        subscription_package: entry.before.subscription_package,
        enabled_distribution_platforms: entry.before.enabled_distribution_platforms.clone(),
        configuration_version: entry.before.configuration_version,
    };
    let mut connection = db.get()?;
    let history = migration_backfill_history(&mut connection, entry.publisher_id)?;
    for row in &history {
        if row.actor != actor {
            continue;
        }
        let before = typed_state(&row.before_state)?;
        let after = typed_state(&row.after_state)?;
        let after_matches_desired = after.subscription_package
            == entry.desired.subscription_package
            && after.enabled_distribution_platforms == entry.desired.enabled_distribution_platforms;
        if before == reviewed_before && after_matches_desired && *current == after {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Deserialize an audit-history JSON state into the typed canonical value, so
/// comparisons are semantic rather than textual.
fn typed_state(value: &serde_json::Value) -> ThothResult<CanonicalServiceConfigurationState> {
    serde_json::from_value(value.clone()).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Current canonical state / catalogue reads
// ---------------------------------------------------------------------------

/// One publisher's current canonical state, read through the existing model
/// accessors (package and token from the publisher row, enabled platforms in
/// canonical order from the assignment reader).
fn current_state(
    db: &PgPool,
    publisher_id: Uuid,
) -> ThothResult<CanonicalServiceConfigurationState> {
    let publisher = Publisher::from_id(db, &publisher_id)?;
    let enabled_distribution_platforms =
        PublisherDistributionPlatform::enabled_assignments(db, publisher_id)?
            .into_iter()
            .map(|assignment| assignment.platform)
            .collect();
    Ok(CanonicalServiceConfigurationState {
        subscription_package: publisher.subscription_package,
        enabled_distribution_platforms,
        configuration_version: publisher.service_configuration_updated_at,
    })
}

/// Every imprint id owned by a publisher.
fn imprint_ids(db: &PgPool, publisher_id: Uuid) -> ThothResult<Vec<Uuid>> {
    let mut connection = db.get()?;
    imprint::table
        .filter(imprint::publisher_id.eq(publisher_id))
        .select(imprint::imprint_id)
        .load::<Uuid>(&mut connection)
        .map_err(Into::into)
}

/// The number of works owned by a publisher through its imprints.
///
/// This is the lock-footprint estimate for the publisher's work-freshness
/// cascade. It is an operational estimate, not a strict configuration invariant.
fn work_count(db: &PgPool, publisher_id: Uuid) -> ThothResult<i64> {
    let ids = imprint_ids(db, publisher_id)?;
    if ids.is_empty() {
        return Ok(0);
    }
    let mut connection = db.get()?;
    work::table
        .filter(work::imprint_id.eq_any(ids))
        .count()
        .get_result::<i64>(&mut connection)
        .map_err(Into::into)
}

/// The distinct non-null licence values across a publisher's works.
fn distinct_licences(db: &PgPool, publisher_id: Uuid) -> ThothResult<Vec<String>> {
    let ids = imprint_ids(db, publisher_id)?;
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut connection = db.get()?;
    let values: Vec<Option<String>> = work::table
        .filter(work::imprint_id.eq_any(ids))
        .filter(work::license.is_not_null())
        .select(work::license)
        .distinct()
        .load::<Option<String>>(&mut connection)?;
    let mut licences: Vec<String> = values.into_iter().flatten().collect();
    licences.sort();
    licences.dedup();
    Ok(licences)
}

/// Every publisher id currently in the database, ascending.
fn all_publisher_ids(db: &PgPool) -> ThothResult<Vec<Uuid>> {
    let mut connection = db.get()?;
    publisher::table
        .select(publisher::publisher_id)
        .order(publisher::publisher_id.asc())
        .load::<Uuid>(&mut connection)
        .map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Production job-state preflight
// ---------------------------------------------------------------------------

/// Refuse to proceed unless the strict production job-state precondition holds:
/// automatic distribution-job creation effectively `OFF` and zero rows in each
/// of the three distribution-job tables.
///
/// This is defence in depth. `MIGRATION_BACKFILL` is job-free by source
/// semantics regardless, but the zero-row precondition additionally guarantees
/// the coordinator's existing assignment-disable cancellation path cannot mutate
/// any pre-existing pending job during a production apply.
///
/// Gate B authorises exercising this only against disposable/local state.
pub fn production_job_state_preflight(
    db: &PgPool,
    job_creation: DistributionJobCreation,
) -> ThothResult<()> {
    if job_creation != DistributionJobCreation::Off {
        return Err(ThothError::MigrationBackfillProductionPrecondition(
            "automatic distribution-job creation is not effectively OFF".to_string(),
        ));
    }
    let (jobs, targets, attempts) = job_table_counts(db)?;
    if jobs != 0 || targets != 0 || attempts != 0 {
        return Err(ThothError::MigrationBackfillProductionPrecondition(format!(
            "distribution job state is not empty (jobs={jobs}, targets={targets}, attempts={attempts})"
        )));
    }
    Ok(())
}

/// The mandatory production post-apply invariant: the three distribution-job
/// tables must still be zero at the end of a production apply invocation.
///
/// `MIGRATION_BACKFILL` is job-free by source semantics, so this must always
/// hold; the check is defence in depth. A violation fails closed — the apply
/// does not claim successful reconciliation, attempts no cross-publisher
/// rollback, and leaves the durable configuration audit rows as the recovery
/// evidence.
fn production_post_apply_job_invariant(db: &PgPool) -> ThothResult<()> {
    let (jobs, targets, attempts) = job_table_counts(db)?;
    if jobs != 0 || targets != 0 || attempts != 0 {
        return Err(ThothError::MigrationBackfillProductionPrecondition(
            format!(
                "post-apply invariant violated: distribution job state is not empty \
             (jobs={jobs}, targets={targets}, attempts={attempts})"
            ),
        ));
    }
    Ok(())
}

/// The row counts of the three distribution-job tables.
fn job_table_counts(db: &PgPool) -> ThothResult<(i64, i64, i64)> {
    let mut connection = db.get()?;
    let jobs: i64 = distribution_job::table
        .count()
        .get_result(&mut connection)?;
    let targets: i64 = distribution_job_target::table
        .count()
        .get_result(&mut connection)?;
    let attempts: i64 = distribution_job_attempt::table
        .count()
        .get_result(&mut connection)?;
    Ok((jobs, targets, attempts))
}

// ---------------------------------------------------------------------------
// Bounded reconciliation report
// ---------------------------------------------------------------------------

/// The bounded, sanitized reconciliation report. Written as human-readable JSON;
/// it is not the hashed machine plan and carries the additional #828 reporting
/// dimensions that are deliberately not part of canonical plan schema v1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationReport {
    pub mode: ReportMode,
    pub manifest_version: u32,
    pub manifest_sha256: String,
    pub plan_sha256: String,
    pub audit_actor: String,
    pub publishers_considered: i64,
    pub package_counts: Vec<PackageCount>,
    pub package_changes: i64,
    pub package_noops: i64,
    pub platform_assignment_counts: Vec<PlatformCount>,
    pub assignment_inserts: i64,
    pub assignment_enables: i64,
    pub assignment_disables: i64,
    pub assignment_noops: i64,
    pub linked_state_anomalies: Vec<LinkedStateAnomaly>,
    pub unsupported_licences: Vec<LicenceFinding>,
    pub omitted_publishers: Vec<OmittedPublisher>,
    pub expected_audit_records: i64,
    pub expected_distribution_jobs: i64,
    pub expected_distribution_job_targets: i64,
    pub expected_distribution_job_attempts: i64,
    pub affected_publishers: i64,
    pub affected_works: i64,
    pub max_works_per_publisher: i64,
    /// Present only for an apply report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<AppliedSummary>,
}

/// Whether the report describes a dry run or an apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportMode {
    DryRun,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCount {
    pub subscription_package: ThothPackage,
    pub publishers: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCount {
    pub platform: DistributionPlatform,
    pub publishers: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedStateAnomaly {
    pub publisher_id: Uuid,
    pub group: DistributionPlatformGroup,
    pub enabled_members: Vec<DistributionPlatform>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenceFinding {
    pub value: String,
    pub disposition: LicenceDispositionReport,
    pub work_count: i64,
}

/// The reported licence disposition, including the `UNREVIEWED` state a value has
/// when the manifest declares no disposition for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenceDispositionReport {
    Unreviewed,
    Normalize,
    Defer,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmittedPublisher {
    pub publisher_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedSummary {
    pub reviewed_noops: i64,
    pub already_applied: i64,
    pub written: i64,
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Inputs for a dry run. The caller (the thin CLI) supplies paths and simple
/// operational values; this module owns all reading, parsing, hashing, domain
/// mapping and serialization.
pub struct DryRunRequest<'a> {
    pub manifest_path: &'a Path,
    pub plan_out_path: &'a Path,
    pub report_out_path: &'a Path,
    /// When set, the production job-state preflight must pass first.
    pub run_production_preflight: bool,
    pub job_creation: DistributionJobCreation,
}

/// The result of a dry run.
#[derive(Debug, Clone)]
pub struct DryRunOutcome {
    pub manifest_sha256: String,
    pub plan_sha256: String,
    pub plan: MigrationPlan,
    pub report: ReconciliationReport,
}

/// The apply execution scope. This is a required, explicit, mutually exclusive
/// choice — never an omitted boolean — so that unsafe production combinations are
/// structurally unrepresentable.
///
/// In particular, `Production` **cannot** be constructed without a lock envelope:
/// there is no "production with no approved envelope" value. Production execution
/// additionally enforces the strict job-state preflight (before any write) and
/// the post-apply zero-job invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyExecutionMode {
    /// Disposable/local execution. No production job-state preflight and no
    /// production licence gate; an optional lock envelope may still be supplied
    /// for testing or pacing.
    Disposable {
        max_works_per_publisher: Option<i64>,
    },
    /// Production execution. A per-publisher work-count lock envelope is
    /// **required** by construction; the strict job-state preflight, the
    /// production licence fail-closed gate and the post-apply zero-job invariant
    /// all apply.
    Production { max_works_per_publisher: i64 },
}

impl ApplyExecutionMode {
    fn is_production(self) -> bool {
        matches!(self, ApplyExecutionMode::Production { .. })
    }

    /// The effective per-publisher lock envelope, if any. Always `Some` for
    /// production (required by construction); optional for disposable.
    fn lock_envelope(self) -> Option<i64> {
        match self {
            ApplyExecutionMode::Disposable {
                max_works_per_publisher,
            } => max_works_per_publisher,
            ApplyExecutionMode::Production {
                max_works_per_publisher,
            } => Some(max_works_per_publisher),
        }
    }
}

/// Inputs for an apply. The reviewed plan and its expected SHA-256 are mandatory;
/// the manifest is required so its recorded raw-byte hash can be re-verified.
pub struct ApplyRequest<'a> {
    pub manifest_path: &'a Path,
    pub plan_path: &'a Path,
    pub expected_plan_sha256: &'a str,
    pub report_out_path: &'a Path,
    /// The explicit execution scope. Gate B never invents a production threshold;
    /// the required production envelope is supplied by Gate D/E.
    pub mode: ApplyExecutionMode,
    pub job_creation: DistributionJobCreation,
}

/// The result of an apply.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub plan_sha256: String,
    pub reviewed_noops: usize,
    pub already_applied: usize,
    pub written: usize,
    pub report: ReconciliationReport,
}

/// Produce a deterministic dry-run plan and reconciliation report; write no
/// database change.
pub fn dry_run(db: &PgPool, request: &DryRunRequest<'_>) -> ThothResult<DryRunOutcome> {
    // Reject aliasing input/output artifacts before any read or write, so a
    // dry run can never overwrite its own manifest with the plan or the report.
    assert_distinct_artifacts(&[
        ("manifest", request.manifest_path),
        ("plan output", request.plan_out_path),
        ("report output", request.report_out_path),
    ])?;
    if request.run_production_preflight {
        production_job_state_preflight(db, request.job_creation)?;
    }
    let manifest_bytes = read_file(request.manifest_path)?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let manifest = parse_manifest(&manifest_bytes)?;

    let (plan, report_parts) = build_plan(db, &manifest, &manifest_sha256)?;
    let plan_bytes = canonical_plan_bytes(&plan)?;
    let plan_sha256 = sha256_hex(&plan_bytes);
    let report = assemble_report(
        ReportMode::DryRun,
        &manifest,
        &plan,
        &manifest_sha256,
        &plan_sha256,
        report_parts,
        None,
    );

    write_file(request.plan_out_path, &plan_bytes)?;
    write_report(request.report_out_path, &report)?;

    Ok(DryRunOutcome {
        manifest_sha256,
        plan_sha256,
        plan,
        report,
    })
}

/// Apply exactly the reviewed plan: verify the plan and manifest hashes, classify
/// every entry, and write only the pending entries through the canonical
/// coordinator with a fixed `MIGRATION_BACKFILL` source and derived actor.
pub fn apply(db: &PgPool, request: &ApplyRequest<'_>) -> ThothResult<ApplyOutcome> {
    let production = request.mode.is_production();

    // Reject aliasing input/output artifacts before any read or write, so an
    // interrupted apply can never destroy the reviewed manifest or plan it needs
    // for deterministic recovery. Inputs are untouched when this rejects.
    assert_distinct_artifacts(&[
        ("manifest", request.manifest_path),
        ("reviewed plan", request.plan_path),
        ("report output", request.report_out_path),
    ])?;

    // Production apply requires the strict job-state precondition (switch
    // effectively OFF and all three distribution-job tables empty) before any
    // write. The required lock envelope is guaranteed by construction of
    // `ApplyExecutionMode::Production`.
    if production {
        production_job_state_preflight(db, request.job_creation)?;
    }

    // 1. Hash the exact raw plan bytes and compare with the expected reviewed
    //    plan SHA-256 before parsing for execution.
    let plan_bytes = read_file(request.plan_path)?;
    let plan_sha256 = sha256_hex(&plan_bytes);
    if plan_sha256 != request.expected_plan_sha256.to_ascii_lowercase() {
        return Err(ThothError::MigrationBackfillPlanHashMismatch);
    }
    // 2-4. Parse, canonically reserialize and require byte-for-byte equality.
    let plan = parse_canonical_plan(&plan_bytes)?;
    // 5. Recheck the exact raw input-manifest SHA-256 recorded by the plan.
    let manifest_bytes = read_file(request.manifest_path)?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if manifest_sha256 != plan.manifest_sha256 {
        return Err(ThothError::MigrationBackfillManifestHashMismatch);
    }
    let manifest = parse_manifest(&manifest_bytes)?;

    // 6. Derive the exact audit actor from the reviewed plan hash.
    let actor = audit_actor(&plan_sha256);

    // 7. Classify every entry before any write. Any DRIFT stops the invocation.
    let mut classes: Vec<(usize, ResumeClassification)> = Vec::with_capacity(plan.entries.len());
    for (index, entry) in plan.entries.iter().enumerate() {
        let class = classify_entry(db, entry, &actor)?;
        if class == ResumeClassification::Drift {
            return Err(ThothError::MigrationBackfillDrift(format!(
                "publisher {} classified as drift; recovery requires a fresh reviewed plan",
                entry.publisher_id
            )));
        }
        classes.push((index, class));
    }

    // B2: a production apply re-audits the current licence state against the
    // exact immutable manifest (its raw-byte hash is already bound into the
    // reviewed plan). Licence state is not protected by the configuration token,
    // so a value that is unreviewed — or that carries a disposition requiring a
    // separate normalization/repair action — must STOP before the first write.
    // MIG-01 never rewrites a licence value.
    if production {
        enforce_reviewed_licences(db, &manifest, &plan.entries)?;
    }

    // The reconciliation report's breakdown is computed against the pre-write
    // state, so it describes the change this apply is about to make rather than
    // the post-write state (which is all no-ops).
    let (_, report_parts) = build_plan(db, &manifest, &manifest_sha256)?;

    // 8. Write only PENDING entries, in deterministic plan order.
    let write_context = ServiceConfigurationWriteContext {
        source: PublisherServiceConfigurationSource::MigrationBackfill,
        actor: &actor,
        job_creation: request.job_creation,
    };
    let mut reviewed_noops = 0usize;
    let mut already_applied = 0usize;
    let mut written = 0usize;
    for (index, class) in &classes {
        let entry = &plan.entries[*index];
        match class {
            ResumeClassification::ReviewedNoop => reviewed_noops += 1,
            ResumeClassification::AlreadyAppliedByThisPlan => already_applied += 1,
            ResumeClassification::Drift => unreachable!("drift stopped the run above"),
            ResumeClassification::Pending => {
                // Recompute the current work count immediately before this
                // publisher's write and stop if it exceeds the approved envelope.
                // Work count is an operational estimate, so a value within the
                // envelope — even if it drifted from the dry run — does not fail.
                if let Some(envelope) = request.mode.lock_envelope() {
                    let current = work_count(db, entry.publisher_id)?;
                    if current > envelope {
                        return Err(ThothError::MigrationBackfillLockEnvelopeExceeded(format!(
                            "publisher {} current work count {current} exceeds approved envelope {envelope}",
                            entry.publisher_id
                        )));
                    }
                }
                let input = ReplacePublisherServiceConfigurationInput {
                    publisher_id: entry.publisher_id,
                    subscription_package: entry.desired.subscription_package,
                    enabled_distribution_platforms: entry
                        .desired
                        .enabled_distribution_platforms
                        .clone(),
                    expected_updated_at: entry.reviewed_configuration_version,
                };
                // The coordinator rechecks the token under the publisher row lock
                // and fails closed on any late concurrent drift. Previously
                // committed publishers remain committed and are resumable.
                super::crud::replace_publisher_service_configuration(db, &write_context, &input)?;
                written += 1;
            }
        }
    }

    // B1: mandatory production post-apply invariant. Verify again that no
    // distribution job/target/attempt row exists before claiming success. A
    // violation fails closed with no reconciliation claim and no rollback.
    if production {
        production_post_apply_job_invariant(db)?;
    }

    let applied = AppliedSummary {
        reviewed_noops: reviewed_noops as i64,
        already_applied: already_applied as i64,
        written: written as i64,
    };
    let report = assemble_report(
        ReportMode::Apply,
        &manifest,
        &plan,
        &manifest_sha256,
        &plan_sha256,
        report_parts,
        Some(applied),
    );
    write_report(request.report_out_path, &report)?;

    Ok(ApplyOutcome {
        plan_sha256,
        reviewed_noops,
        already_applied,
        written,
        report,
    })
}

// ---------------------------------------------------------------------------
// Plan construction
// ---------------------------------------------------------------------------

/// Report-only detail collected during plan construction.
struct ReportParts {
    package_counts: Vec<PackageCount>,
    platform_counts: Vec<PlatformCount>,
    assignment_inserts: i64,
    assignment_enables: i64,
    assignment_disables: i64,
    assignment_noops: i64,
    linked_state_anomalies: Vec<LinkedStateAnomaly>,
    unsupported_licences: Vec<LicenceFinding>,
    omitted_publishers: Vec<OmittedPublisher>,
}

/// Parse and version-validate a manifest from its raw bytes.
pub fn parse_manifest(bytes: &[u8]) -> ThothResult<MigrationManifest> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ThothError::MigrationBackfillManifestInvalid(
            "manifest must not begin with a byte-order mark".to_string(),
        ));
    }
    let manifest: MigrationManifest = serde_json::from_slice(bytes).map_err(|error| {
        ThothError::MigrationBackfillManifestInvalid(format!("manifest is not valid JSON: {error}"))
    })?;
    manifest.validate()?;
    Ok(manifest)
}

/// Build the canonical plan and the report detail from a resolved manifest.
///
/// Ambiguous (duplicated) and unmatched publisher identifiers are surfaced as
/// blocking errors rather than guessed.
fn build_plan(
    db: &PgPool,
    manifest: &MigrationManifest,
    manifest_sha256: &str,
) -> ThothResult<(MigrationPlan, ReportParts)> {
    // Ambiguous mapping: the same canonical publisher named more than once.
    let mut seen: HashSet<Uuid> = HashSet::new();
    for entry in &manifest.publishers {
        if !seen.insert(entry.publisher_id) {
            return Err(ThothError::MigrationBackfillAmbiguousMapping(format!(
                "publisher {} is mapped more than once in the manifest",
                entry.publisher_id
            )));
        }
    }

    let mut entries: Vec<PlanEntry> = Vec::with_capacity(manifest.publishers.len());
    let mut assignment_inserts = 0i64;
    let mut assignment_enables = 0i64;
    let mut assignment_disables = 0i64;
    let mut assignment_noops = 0i64;
    let mut linked_state_anomalies: Vec<LinkedStateAnomaly> = Vec::new();
    let mut package_tally: BTreeMap<usize, i64> = BTreeMap::new();
    let mut platform_tally: BTreeMap<usize, i64> = BTreeMap::new();
    let mut licence_dispositions: Vec<LicenceFinding> = Vec::new();
    let mut licence_seen: HashSet<String> = HashSet::new();

    for manifest_entry in &manifest.publishers {
        let publisher_id = manifest_entry.publisher_id;
        // Unmatched legacy identifier: no canonical publisher resolves.
        let current = match current_state(db, publisher_id) {
            Ok(state) => state,
            Err(ThothError::EntityNotFound) => {
                return Err(ThothError::MigrationBackfillUnmatchedPublisher(
                    publisher_id.to_string(),
                ))
            }
            Err(error) => return Err(error),
        };

        // Validate the requested desired set before normalization.
        for platform in &manifest_entry.enabled_distribution_platforms {
            if !platform.is_assignable() {
                return Err(ThothError::MigrationBackfillManifestInvalid(format!(
                    "platform {platform} requested for publisher {publisher_id} is not assignable"
                )));
            }
        }
        let desired_platforms =
            normalize_requested_platforms(&manifest_entry.enabled_distribution_platforms);

        // Linked-state anomaly detection at the canonical enabled-set level:
        // a linked group with exactly one enabled member before normalization.
        for anomaly in linked_anomalies(publisher_id, &current.enabled_distribution_platforms) {
            linked_state_anomalies.push(anomaly);
        }

        let is_noop = current.subscription_package == manifest_entry.subscription_package
            && current.enabled_distribution_platforms == desired_platforms;
        let classification = if is_noop {
            PlanClassification::ReviewedNoop
        } else {
            PlanClassification::Pending
        };

        // Assignment-level breakdown, distinguishing a first-ever insert from a
        // re-enable of a previously disabled row.
        let ever_rows: HashSet<DistributionPlatform> =
            PublisherDistributionPlatform::all_for_publisher(db, publisher_id)?
                .into_iter()
                .map(|row| row.platform)
                .collect();
        let current_enabled: HashSet<DistributionPlatform> = current
            .enabled_distribution_platforms
            .iter()
            .copied()
            .collect();
        let desired_set: HashSet<DistributionPlatform> =
            desired_platforms.iter().copied().collect();
        for platform in &desired_set {
            if current_enabled.contains(platform) {
                assignment_noops += 1;
            } else if ever_rows.contains(platform) {
                assignment_enables += 1;
            } else {
                assignment_inserts += 1;
            }
        }
        for platform in &current_enabled {
            if !desired_set.contains(platform) {
                assignment_disables += 1;
            }
        }

        // Desired package/platform tallies for the report.
        *package_tally
            .entry(package_index(manifest_entry.subscription_package))
            .or_insert(0) += 1;
        for platform in &desired_platforms {
            *platform_tally.entry(platform_index(*platform)).or_insert(0) += 1;
        }

        // Licence audit across this publisher's works.
        for value in distinct_licences(db, publisher_id)? {
            if licence_seen.insert(value.clone()) {
                if let Some(finding) = licence_finding(manifest, db, &value)? {
                    licence_dispositions.push(finding);
                }
            }
        }

        let affected_work_count = work_count(db, publisher_id)?;

        entries.push(PlanEntry {
            publisher_id,
            reviewed_configuration_version: current.configuration_version,
            before: PlanBeforeState {
                subscription_package: current.subscription_package,
                enabled_distribution_platforms: current.enabled_distribution_platforms.clone(),
                configuration_version: current.configuration_version,
            },
            desired: PlanDesiredState {
                subscription_package: manifest_entry.subscription_package,
                enabled_distribution_platforms: desired_platforms,
            },
            classification,
            affected_work_count,
        });
    }

    // Deterministic order: ascending canonical publisher UUID.
    entries.sort_by_key(|entry| entry.publisher_id);

    let publishers_considered = entries.len() as i64;
    let publishers_changing = entries
        .iter()
        .filter(|entry| entry.classification == PlanClassification::Pending)
        .count() as i64;
    let publishers_noop = publishers_considered - publishers_changing;
    let package_changes = entries
        .iter()
        .filter(|entry| entry.before.subscription_package != entry.desired.subscription_package)
        .count() as i64;
    let package_noops = publishers_considered - package_changes;
    let affected_works: i64 = entries
        .iter()
        .filter(|entry| entry.classification == PlanClassification::Pending)
        .map(|entry| entry.affected_work_count)
        .sum();
    let max_works_per_publisher = entries
        .iter()
        .filter(|entry| entry.classification == PlanClassification::Pending)
        .map(|entry| entry.affected_work_count)
        .max()
        .unwrap_or(0);

    let expected = PlanExpected {
        publishers_considered,
        publishers_changing,
        publishers_noop,
        package_changes,
        package_noops,
        assignment_inserts,
        assignment_enables,
        assignment_disables,
        assignment_noops,
        audit_records: publishers_changing,
        distribution_jobs: 0,
        distribution_job_targets: 0,
        distribution_job_attempts: 0,
        affected_publishers: publishers_changing,
        affected_works,
        max_works_per_publisher,
    };

    let plan = MigrationPlan {
        schema_version: PLAN_SCHEMA_VERSION,
        manifest_sha256: manifest_sha256.to_string(),
        entries,
        expected,
    };

    let package_counts = package_tally
        .into_iter()
        .map(|(index, publishers)| PackageCount {
            subscription_package: ALL_PACKAGES[index],
            publishers,
        })
        .collect();
    let platform_counts = platform_tally
        .into_iter()
        .map(|(index, publishers)| PlatformCount {
            platform: DistributionPlatform::ALL[index],
            publishers,
        })
        .collect();
    let omitted_publishers = omitted_publishers(db, &seen)?;

    Ok((
        plan,
        ReportParts {
            package_counts,
            platform_counts,
            assignment_inserts,
            assignment_enables,
            assignment_disables,
            assignment_noops,
            linked_state_anomalies,
            unsupported_licences: licence_dispositions,
            omitted_publishers,
        },
    ))
}

/// Every publisher present in the database but absent from the approved manifest,
/// each with an explicit reason.
fn omitted_publishers(db: &PgPool, mapped: &HashSet<Uuid>) -> ThothResult<Vec<OmittedPublisher>> {
    let mut omitted: Vec<OmittedPublisher> = all_publisher_ids(db)?
        .into_iter()
        .filter(|id| !mapped.contains(id))
        .map(|publisher_id| OmittedPublisher {
            publisher_id,
            reason: "publisher is not present in the approved MIG-01 manifest".to_string(),
        })
        .collect();
    omitted.sort_by_key(|entry| entry.publisher_id);
    Ok(omitted)
}

/// The linked-group anomalies visible in a canonical enabled set: a group with
/// exactly one enabled member.
fn linked_anomalies(
    publisher_id: Uuid,
    enabled: &[DistributionPlatform],
) -> Vec<LinkedStateAnomaly> {
    let enabled_set: HashSet<DistributionPlatform> = enabled.iter().copied().collect();
    let mut anomalies = Vec::new();
    let mut reported_groups: HashSet<DistributionPlatformGroup> = HashSet::new();
    for platform in DistributionPlatform::ALL {
        let Some(group) = platform.linked_group() else {
            continue;
        };
        if reported_groups.contains(&group) {
            continue;
        }
        let members = platform.linked_members();
        let enabled_members: Vec<DistributionPlatform> = members
            .iter()
            .copied()
            .filter(|member| enabled_set.contains(member))
            .collect();
        if !enabled_members.is_empty() && enabled_members.len() != members.len() {
            anomalies.push(LinkedStateAnomaly {
                publisher_id,
                group,
                enabled_members,
            });
            reported_groups.insert(group);
        }
    }
    anomalies
}

/// The licence finding for a value, if it is not a reviewed supported value.
fn licence_finding(
    manifest: &MigrationManifest,
    db: &PgPool,
    value: &str,
) -> ThothResult<Option<LicenceFinding>> {
    let disposition = manifest.disposition_for(value);
    let reported = match disposition {
        Some(LicenceDisposition::Supported) => return Ok(None),
        Some(LicenceDisposition::Normalize) => LicenceDispositionReport::Normalize,
        Some(LicenceDisposition::Defer) => LicenceDispositionReport::Defer,
        Some(LicenceDisposition::Reject) => LicenceDispositionReport::Reject,
        None => LicenceDispositionReport::Unreviewed,
    };
    Ok(Some(LicenceFinding {
        value: value.to_string(),
        disposition: reported,
        work_count: works_with_licence(db, value)?,
    }))
}

/// The number of works carrying an exact licence value across the catalogue.
fn works_with_licence(db: &PgPool, value: &str) -> ThothResult<i64> {
    let mut connection = db.get()?;
    work::table
        .filter(work::license.eq(value))
        .count()
        .get_result::<i64>(&mut connection)
        .map_err(Into::into)
}

/// Fail closed if any considered publisher's current catalogue licence state is
/// not reviewed as supported by the exact immutable manifest (`B2`).
///
/// A production package/platform apply must not proceed while a required licence
/// remains unresolved. A value is blocking when it is:
///
/// - `UNREVIEWED` — the manifest declares no disposition for it; or
/// - `NORMALIZE`/`DEFER`/`REJECT` — a reviewed disposition that, per the approved
///   #828 licence semantics, requires a separate normalization/repair action or
///   leaves the value unresolved (a recorded `defer` alone does not satisfy
///   MIG-01).
///
/// Only `SUPPORTED` permits the apply to continue. This performs **no** licence
/// write and never rewrites a value; unresolved licence normalization is a
/// distinct, separately authorized gate (and is out of scope for Gate B).
fn enforce_reviewed_licences(
    db: &PgPool,
    manifest: &MigrationManifest,
    entries: &[PlanEntry],
) -> ThothResult<()> {
    let mut checked: HashSet<String> = HashSet::new();
    for entry in entries {
        for value in distinct_licences(db, entry.publisher_id)? {
            if !checked.insert(value.clone()) {
                continue;
            }
            match manifest.disposition_for(&value) {
                Some(LicenceDisposition::Supported) => {}
                Some(LicenceDisposition::Normalize) => {
                    return Err(ThothError::MigrationBackfillUnresolvedLicence(format!(
                        "licence {value:?} is reviewed NORMALIZE and requires a separate, \
                         separately authorized normalization action before a production apply"
                    )))
                }
                Some(LicenceDisposition::Defer) => {
                    return Err(ThothError::MigrationBackfillUnresolvedLicence(format!(
                        "licence {value:?} is reviewed DEFER and remains unresolved; a deferred \
                         disposition alone does not permit a production apply"
                    )))
                }
                Some(LicenceDisposition::Reject) => {
                    return Err(ThothError::MigrationBackfillUnresolvedLicence(format!(
                        "licence {value:?} is reviewed REJECT and is not permitted"
                    )))
                }
                None => {
                    return Err(ThothError::MigrationBackfillUnresolvedLicence(format!(
                        "licence {value:?} has no reviewed disposition in the immutable manifest \
                         (UNREVIEWED)"
                    )))
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Report assembly and file I/O
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn assemble_report(
    mode: ReportMode,
    manifest: &MigrationManifest,
    plan: &MigrationPlan,
    manifest_sha256: &str,
    plan_sha256: &str,
    parts: ReportParts,
    applied: Option<AppliedSummary>,
) -> ReconciliationReport {
    ReconciliationReport {
        mode,
        manifest_version: manifest.manifest_version,
        manifest_sha256: manifest_sha256.to_string(),
        plan_sha256: plan_sha256.to_string(),
        audit_actor: audit_actor(plan_sha256),
        publishers_considered: plan.expected.publishers_considered,
        package_counts: parts.package_counts,
        package_changes: plan.expected.package_changes,
        package_noops: plan.expected.package_noops,
        platform_assignment_counts: parts.platform_counts,
        assignment_inserts: parts.assignment_inserts,
        assignment_enables: parts.assignment_enables,
        assignment_disables: parts.assignment_disables,
        assignment_noops: parts.assignment_noops,
        linked_state_anomalies: parts.linked_state_anomalies,
        unsupported_licences: parts.unsupported_licences,
        omitted_publishers: parts.omitted_publishers,
        expected_audit_records: plan.expected.audit_records,
        expected_distribution_jobs: plan.expected.distribution_jobs,
        expected_distribution_job_targets: plan.expected.distribution_job_targets,
        expected_distribution_job_attempts: plan.expected.distribution_job_attempts,
        affected_publishers: plan.expected.affected_publishers,
        affected_works: plan.expected.affected_works,
        max_works_per_publisher: plan.expected.max_works_per_publisher,
        applied,
    }
}

/// Reject any two `MIG-01` artifact paths that resolve to the same filesystem
/// location (`B5`), before any read or write, so an interrupted run can never
/// destroy a reviewed manifest or plan it needs for deterministic recovery.
///
/// Resolution handles identical lexical paths, relative-path normalization and
/// existing symlink aliases: an existing path is canonicalized; a not-yet-
/// existing output is resolved through its existing parent directory combined
/// with its filename. This stays deliberately MIG-01-local.
fn assert_distinct_artifacts(artifacts: &[(&str, &Path)]) -> ThothResult<()> {
    let mut resolved: Vec<(&str, std::path::PathBuf)> = Vec::with_capacity(artifacts.len());
    for (label, path) in artifacts {
        let identity = resolve_artifact_identity(path)?;
        if let Some((other, _)) = resolved.iter().find(|(_, existing)| *existing == identity) {
            return Err(ThothError::MigrationBackfillArtifactAlias(format!(
                "the {label} path and the {other} path resolve to the same location {}",
                identity.display()
            )));
        }
        resolved.push((label, identity));
    }
    Ok(())
}

/// The resolved filesystem identity of an artifact path, following symlinks and
/// normalizing relative components. An existing path canonicalizes directly; a
/// not-yet-existing output resolves through its existing parent directory.
fn resolve_artifact_identity(path: &Path) -> ThothResult<std::path::PathBuf> {
    if let Ok(canonical) = path.canonicalize() {
        return Ok(canonical);
    }
    let file_name = path.file_name().ok_or_else(|| {
        ThothError::MigrationBackfillArtifactAlias(format!(
            "{} is not a usable file path",
            path.display()
        ))
    })?;
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let parent = parent.canonicalize().map_err(|error| {
        ThothError::MigrationBackfillArtifactAlias(format!(
            "cannot resolve the directory for {}: {error}",
            path.display()
        ))
    })?;
    Ok(parent.join(file_name))
}

fn read_file(path: &Path) -> ThothResult<Vec<u8>> {
    std::fs::read(path).map_err(|error| {
        ThothError::MigrationBackfillManifestInvalid(format!(
            "could not read {}: {error}",
            path.display()
        ))
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> ThothResult<()> {
    std::fs::write(path, bytes).map_err(Into::into)
}

fn write_report(path: &Path, report: &ReconciliationReport) -> ThothResult<()> {
    let rendered = serde_json::to_vec_pretty(report)?;
    std::fs::write(path, rendered).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Canonical enum ordering helpers
// ---------------------------------------------------------------------------

/// Packages in a fixed order, so report tallies are deterministic.
const ALL_PACKAGES: [ThothPackage; 4] = [
    ThothPackage::Oasis,
    ThothPackage::Obelisk,
    ThothPackage::Sphinx,
    ThothPackage::Pyramid,
];

fn package_index(package: ThothPackage) -> usize {
    ALL_PACKAGES
        .iter()
        .position(|candidate| *candidate == package)
        .expect("every ThothPackage is listed in ALL_PACKAGES")
}

fn platform_index(platform: DistributionPlatform) -> usize {
    DistributionPlatform::ALL
        .iter()
        .position(|candidate| *candidate == platform)
        .expect("every DistributionPlatform is listed in DistributionPlatform::ALL")
}

#[cfg(all(test, feature = "backend"))]
mod tests {
    //! `MIG-01` facade evidence, run against a real disposable PostgreSQL
    //! database with the migration applied. Every DB-backed test resets the
    //! database and drives the same canonical coordinator production uses.

    use super::super::crud::replace_publisher_service_configuration;
    use super::super::{
        NewPublisherServiceConfigurationHistory, PublisherServiceConfigurationHistory,
    };
    use super::*;
    use crate::model::publisher_distribution_platform::PublisherDistributionPlatform;
    use crate::model::tests::db::{create_imprint, create_publisher, create_work, setup_test_db};
    use diesel::sql_query;

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    fn tmp_path(suffix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("mig01-{}-{suffix}", Uuid::new_v4()));
        path
    }

    fn write_manifest(value: &serde_json::Value) -> std::path::PathBuf {
        let path = tmp_path("manifest.json");
        std::fs::write(&path, serde_json::to_vec(value).expect("manifest bytes")).unwrap();
        path
    }

    /// A manifest mapping the given `(publisher, package, platforms)` tuples.
    fn manifest_value(entries: &[(Uuid, &str, &[&str])]) -> serde_json::Value {
        let publishers: Vec<serde_json::Value> = entries
            .iter()
            .map(|(id, package, platforms)| {
                serde_json::json!({
                    "publisherId": id.to_string(),
                    "subscriptionPackage": package,
                    "enabledDistributionPlatforms": platforms,
                })
            })
            .collect();
        serde_json::json!({ "manifestVersion": 1, "publishers": publishers })
    }

    fn run_dry_run(pool: &PgPool, manifest_path: &std::path::Path) -> (DryRunOutcome, Vec<u8>) {
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            manifest_path,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        let outcome = dry_run(pool, &request).expect("dry run");
        let bytes = std::fs::read(&plan_out).expect("plan bytes");
        (outcome, bytes)
    }

    /// A dry run returning the raw result (for failure-path assertions), using
    /// fresh temp output paths.
    fn try_dry_run(pool: &PgPool, manifest_path: &std::path::Path) -> ThothResult<DryRunOutcome> {
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            manifest_path,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        dry_run(pool, &request)
    }

    /// Disposable-mode apply with an optional lock envelope.
    fn apply_plan(
        pool: &PgPool,
        manifest_path: &std::path::Path,
        plan_bytes: &[u8],
        expected_sha: &str,
        job_creation: DistributionJobCreation,
        max_works: Option<i64>,
    ) -> ThothResult<ApplyOutcome> {
        apply_plan_mode(
            pool,
            manifest_path,
            plan_bytes,
            expected_sha,
            job_creation,
            ApplyExecutionMode::Disposable {
                max_works_per_publisher: max_works,
            },
        )
    }

    /// Apply with an explicit execution mode (used by the production-path tests).
    fn apply_plan_mode(
        pool: &PgPool,
        manifest_path: &std::path::Path,
        plan_bytes: &[u8],
        expected_sha: &str,
        job_creation: DistributionJobCreation,
        mode: ApplyExecutionMode,
    ) -> ThothResult<ApplyOutcome> {
        let plan_path = tmp_path("reviewed-plan.json");
        std::fs::write(&plan_path, plan_bytes).unwrap();
        let report_out = tmp_path("apply-report.json");
        let request = ApplyRequest {
            manifest_path,
            plan_path: &plan_path,
            expected_plan_sha256: expected_sha,
            report_out_path: &report_out,
            mode,
            job_creation,
        };
        apply(pool, &request)
    }

    /// Drive the canonical coordinator to establish a publisher's current state.
    fn set_state(
        pool: &PgPool,
        publisher_id: Uuid,
        package: ThothPackage,
        platforms: &[DistributionPlatform],
    ) {
        let context = ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::SuperuserApi,
            actor: "test-superuser",
            job_creation: DistributionJobCreation::On,
        };
        let token = Publisher::from_id(pool, &publisher_id)
            .unwrap()
            .service_configuration_updated_at;
        let input = ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: package,
            enabled_distribution_platforms: platforms.to_vec(),
            expected_updated_at: token,
        };
        replace_publisher_service_configuration(pool, &context, &input).expect("set state");
    }

    fn enabled_now(pool: &PgPool, publisher_id: Uuid) -> Vec<DistributionPlatform> {
        PublisherDistributionPlatform::enabled_assignments(pool, publisher_id)
            .unwrap()
            .into_iter()
            .map(|assignment| assignment.platform)
            .collect()
    }

    fn package_now(pool: &PgPool, publisher_id: Uuid) -> ThothPackage {
        Publisher::from_id(pool, &publisher_id)
            .unwrap()
            .subscription_package
    }

    fn mig_history(pool: &PgPool, publisher_id: Uuid) -> Vec<PublisherServiceConfigurationHistory> {
        let mut connection = pool.get().unwrap();
        migration_backfill_history(&mut connection, publisher_id).unwrap()
    }

    fn table_count(pool: &PgPool, sql: &str) -> i64 {
        #[derive(diesel::QueryableByName)]
        struct Count {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }
        let mut connection = pool.get().unwrap();
        let rows: Vec<Count> = diesel::sql_query(sql).load(&mut connection).unwrap();
        rows[0].count
    }

    // ---------------------------------------------------------------------
    // Mapping and plan generation
    // ---------------------------------------------------------------------

    #[test]
    fn dry_run_is_deterministic_in_bytes_and_hash() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));

        let (first, first_bytes) = run_dry_run(&pool, &manifest);
        let (second, second_bytes) = run_dry_run(&pool, &manifest);
        assert_eq!(
            first_bytes, second_bytes,
            "canonical plan bytes must be stable"
        );
        assert_eq!(first.plan_sha256, second.plan_sha256);
        assert_eq!(first.manifest_sha256, second.manifest_sha256);
        // The hash is a hash of exactly the emitted bytes.
        assert_eq!(first.plan_sha256, sha256_hex(&first_bytes));
    }

    #[test]
    fn canonical_plan_bytes_have_no_bom_no_trailing_newline_and_fixed_order() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (_outcome, bytes) = run_dry_run(&pool, &manifest);

        assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]), "no BOM");
        assert_ne!(bytes.last(), Some(&b'\n'), "no trailing newline");
        assert!(
            !bytes.windows(2).any(|w| w == b": "),
            "no insignificant whitespace"
        );

        let text = std::str::from_utf8(&bytes).unwrap();
        // Top-level order.
        let top = ["schemaVersion", "manifestSha256", "entries", "expected"];
        assert!(ordered(text, &top), "top-level field order fixed");
        // Per-entry order.
        let entry = [
            "publisherId",
            "reviewedConfigurationVersion",
            "before",
            "desired",
            "classification",
            "affectedWorkCount",
        ];
        assert!(ordered(text, &entry), "per-entry field order fixed");
        // `expected` order (first three).
        assert!(
            ordered(
                text,
                &[
                    "publishersConsidered",
                    "publishersChanging",
                    "publishersNoop"
                ]
            ),
            "expected field order fixed"
        );
    }

    fn ordered(text: &str, keys: &[&str]) -> bool {
        let mut last = 0usize;
        for key in keys {
            let needle = format!("\"{key}\"");
            match text.find(&needle) {
                Some(index) if index >= last => last = index,
                _ => return false,
            }
        }
        true
    }

    #[test]
    fn plan_timestamps_use_exactly_six_fractional_digits_and_z() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        // Move the token so the configuration version is a real, non-default value.
        set_state(
            &pool,
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
        );
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "SPHINX",
            &["ZENODO"],
        )]));
        let (_outcome, bytes) = run_dry_run(&pool, &manifest);
        let text = std::str::from_utf8(&bytes).unwrap();

        let pattern = regex::Regex::new(
            r#""configurationVersion":"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z""#,
        )
        .unwrap();
        assert!(
            pattern.is_match(text),
            "configurationVersion must be six-digit Z: {text}"
        );
        let reviewed = regex::Regex::new(
            r#""reviewedConfigurationVersion":"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z""#,
        )
        .unwrap();
        assert!(
            reviewed.is_match(text),
            "reviewedConfigurationVersion must be six-digit Z"
        );
    }

    #[test]
    fn plan_round_trips_canonically_and_rejects_noncanonical_bytes() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (_outcome, bytes) = run_dry_run(&pool, &manifest);

        // Canonical bytes round-trip to the identical bytes and hash.
        let parsed = parse_canonical_plan(&bytes).expect("canonical parse");
        let reserialized = canonical_plan_bytes(&parsed).unwrap();
        assert_eq!(reserialized, bytes);
        assert_eq!(sha256_hex(&reserialized), sha256_hex(&bytes));

        // A BOM is rejected.
        let mut with_bom = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice(&bytes);
        assert!(matches!(
            parse_canonical_plan(&with_bom),
            Err(ThothError::MigrationBackfillNoncanonicalPlan)
        ));

        // Pretty-printed (whitespace) bytes are rejected even though semantically equal.
        let pretty = serde_json::to_vec_pretty(&parsed).unwrap();
        assert!(matches!(
            parse_canonical_plan(&pretty),
            Err(ThothError::MigrationBackfillNoncanonicalPlan)
        ));

        // Key-reordered bytes are rejected.
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let reordered = serde_json::to_vec(&value).unwrap();
        assert_ne!(reordered, bytes, "reordering must actually differ");
        assert!(matches!(
            parse_canonical_plan(&reordered),
            Err(ThothError::MigrationBackfillNoncanonicalPlan)
        ));
    }

    #[test]
    fn unmatched_publisher_is_a_blocking_failure() {
        let (_guard, pool) = setup_test_db();
        let manifest = write_manifest(&manifest_value(&[(Uuid::new_v4(), "OBELISK", &["ZENODO"])]));
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            manifest_path: &manifest,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            dry_run(&pool, &request),
            Err(ThothError::MigrationBackfillUnmatchedPublisher(_))
        ));
    }

    #[test]
    fn ambiguous_duplicate_mapping_is_a_blocking_failure() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let manifest = write_manifest(&manifest_value(&[
            (publisher.publisher_id, "OBELISK", &["ZENODO"]),
            (publisher.publisher_id, "SPHINX", &["INTERNET_ARCHIVE"]),
        ]));
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            manifest_path: &manifest,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            dry_run(&pool, &request),
            Err(ThothError::MigrationBackfillAmbiguousMapping(_))
        ));
    }

    #[test]
    fn package_platform_and_combined_changes_and_true_noop_classify_correctly() {
        let (_guard, pool) = setup_test_db();
        // package-only
        let a = create_publisher(&pool);
        set_state(
            &pool,
            a.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Zenodo],
        );
        // platform-only
        let b = create_publisher(&pool);
        set_state(
            &pool,
            b.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
        );
        // combined
        let c = create_publisher(&pool);
        set_state(&pool, c.publisher_id, ThothPackage::Oasis, &[]);
        // true no-op
        let d = create_publisher(&pool);
        set_state(
            &pool,
            d.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::InternetArchive],
        );

        let manifest = write_manifest(&manifest_value(&[
            (a.publisher_id, "OBELISK", &["ZENODO"]), // package only
            (b.publisher_id, "OBELISK", &["INTERNET_ARCHIVE"]), // platform only
            (c.publisher_id, "SPHINX", &["ZENODO"]),  // combined
            (d.publisher_id, "SPHINX", &["INTERNET_ARCHIVE"]), // no-op
        ]));
        let (outcome, _bytes) = run_dry_run(&pool, &manifest);
        let by_id = |id: Uuid| {
            outcome
                .plan
                .entries
                .iter()
                .find(|entry| entry.publisher_id == id)
                .unwrap()
                .clone()
        };
        assert_eq!(
            by_id(a.publisher_id).classification,
            PlanClassification::Pending
        );
        assert_eq!(
            by_id(b.publisher_id).classification,
            PlanClassification::Pending
        );
        assert_eq!(
            by_id(c.publisher_id).classification,
            PlanClassification::Pending
        );
        assert_eq!(
            by_id(d.publisher_id).classification,
            PlanClassification::ReviewedNoop
        );
        assert_eq!(outcome.plan.expected.publishers_considered, 4);
        assert_eq!(outcome.plan.expected.publishers_changing, 3);
        assert_eq!(outcome.plan.expected.publishers_noop, 1);
        assert_eq!(outcome.plan.expected.package_changes, 2); // a and c
    }

    #[test]
    fn linked_oapen_doab_group_is_normalized() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        // Request only OAPEN; the desired set must close over the linked group.
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["OAPEN"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let entry = &outcome.plan.entries[0];
        assert_eq!(
            entry.desired.enabled_distribution_platforms,
            vec![DistributionPlatform::Oapen, DistributionPlatform::Doab],
            "desired set closes over OAPEN/DOAB in canonical order"
        );

        let sha = outcome.plan_sha256.clone();
        apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .unwrap();
        let mut enabled = enabled_now(&pool, publisher.publisher_id);
        enabled.sort_by_key(|p| platform_index(*p));
        assert_eq!(
            enabled,
            vec![DistributionPlatform::Oapen, DistributionPlatform::Doab]
        );
    }

    // ---------------------------------------------------------------------
    // Apply, audit provenance, idempotency, resume
    // ---------------------------------------------------------------------

    #[test]
    fn apply_writes_migration_backfill_provenance_and_derived_actor() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let applied = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .expect("apply");
        assert_eq!(applied.written, 1);

        let history = mig_history(&pool, publisher.publisher_id);
        assert_eq!(history.len(), 1);
        let row = &history[0];
        assert_eq!(
            row.source,
            PublisherServiceConfigurationSource::MigrationBackfill
        );
        assert_eq!(row.actor, audit_actor(&sha));
        let before = typed_state(&row.before_state).unwrap();
        let after = typed_state(&row.after_state).unwrap();
        assert_eq!(before.subscription_package, ThothPackage::Oasis);
        assert_eq!(after.subscription_package, ThothPackage::Obelisk);
        assert_eq!(
            after.enabled_distribution_platforms,
            vec![DistributionPlatform::Zenodo]
        );
        // Current state equals the recorded after-state.
        assert_eq!(
            package_now(&pool, publisher.publisher_id),
            ThothPackage::Obelisk
        );
    }

    #[test]
    fn apply_is_idempotent_and_post_apply_dry_run_reports_zero_changes() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        let first = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .unwrap();
        assert_eq!(first.written, 1);
        // Re-applying the same reviewed plan writes nothing.
        let second = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .unwrap();
        assert_eq!(second.written, 0);
        assert_eq!(second.already_applied, 1);
        assert_eq!(
            mig_history(&pool, publisher.publisher_id).len(),
            1,
            "no second audit row"
        );

        // A fresh dry run against applied state reports every publisher a no-op.
        let (post, _bytes) = run_dry_run(&pool, &manifest);
        assert_eq!(post.plan.expected.publishers_changing, 0);
        assert_eq!(post.plan.expected.package_changes, 0);
        assert!(post
            .plan
            .entries
            .iter()
            .all(|entry| entry.classification == PlanClassification::ReviewedNoop));
    }

    #[test]
    fn resume_recognizes_a_prior_commit_by_this_plan_and_writes_only_the_remainder() {
        let (_guard, pool) = setup_test_db();
        let a = create_publisher(&pool);
        let b = create_publisher(&pool);
        set_state(&pool, a.publisher_id, ThothPackage::Oasis, &[]);
        set_state(&pool, b.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[
            (a.publisher_id, "OBELISK", &["ZENODO"]),
            (b.publisher_id, "SPHINX", &["INTERNET_ARCHIVE"]),
        ]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let actor = audit_actor(&sha);

        // Simulate an interrupted run: publisher A committed by this exact plan
        // (its derived actor) but the process failed before the caller observed it.
        let entry_a = outcome
            .plan
            .entries
            .iter()
            .find(|entry| entry.publisher_id == a.publisher_id)
            .unwrap();
        let context = ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::MigrationBackfill,
            actor: &actor,
            job_creation: DistributionJobCreation::Off,
        };
        let input = ReplacePublisherServiceConfigurationInput {
            publisher_id: a.publisher_id,
            subscription_package: entry_a.desired.subscription_package,
            enabled_distribution_platforms: entry_a.desired.enabled_distribution_platforms.clone(),
            expected_updated_at: entry_a.reviewed_configuration_version,
        };
        replace_publisher_service_configuration(&pool, &context, &input).unwrap();

        // Resuming the same reviewed plan recognises A as already applied and
        // writes only B.
        let resumed = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .unwrap();
        assert_eq!(resumed.already_applied, 1);
        assert_eq!(resumed.written, 1);
        assert_eq!(
            mig_history(&pool, a.publisher_id).len(),
            1,
            "A not written twice"
        );
        assert_eq!(package_now(&pool, b.publisher_id), ThothPackage::Sphinx);
    }

    #[test]
    fn history_from_another_plan_is_not_accepted_as_applied() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, _bytes) = run_dry_run(&pool, &manifest);
        let entry = outcome.plan.entries[0].clone();
        let real_actor = audit_actor(&outcome.plan_sha256);

        // Insert a lookalike MIGRATION_BACKFILL row whose actor belongs to a
        // *different* plan, with the same before/after content. It must not be
        // accepted as this plan's application.
        let before = CanonicalServiceConfigurationState {
            subscription_package: entry.before.subscription_package,
            enabled_distribution_platforms: entry.before.enabled_distribution_platforms.clone(),
            configuration_version: entry.before.configuration_version,
        };
        let after = CanonicalServiceConfigurationState {
            subscription_package: entry.desired.subscription_package,
            enabled_distribution_platforms: entry.desired.enabled_distribution_platforms.clone(),
            configuration_version: entry.before.configuration_version,
        };
        let lookalike = NewPublisherServiceConfigurationHistory {
            publisher_id: publisher.publisher_id,
            actor: "MIG-01:0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            source: PublisherServiceConfigurationSource::MigrationBackfill,
            before_state: serde_json::to_value(&before).unwrap(),
            after_state: serde_json::to_value(&after).unwrap(),
        };
        {
            let mut connection = pool.get().unwrap();
            diesel::insert_into(crate::schema::publisher_service_configuration_history::table)
                .values(&lookalike)
                .execute(&mut connection)
                .expect("insert lookalike history");
        }
        assert_ne!(real_actor, lookalike.actor);

        // The publisher's real current state is still the reviewed before state,
        // so classification is PENDING (eligible to write), never already-applied.
        let class = classify_entry(&pool, &entry, &real_actor).unwrap();
        assert_eq!(class, ResumeClassification::Pending);
    }

    #[test]
    fn drift_stops_before_any_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        // A concurrent superuser change moves the token after review: the reviewed
        // token is now stale and the entry classifies as drift.
        set_state(&pool, publisher.publisher_id, ThothPackage::Pyramid, &[]);
        let result = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        );
        assert!(matches!(result, Err(ThothError::MigrationBackfillDrift(_))));
        // No MIG-01 write occurred; the only state is the concurrent superuser one.
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
        assert_eq!(
            package_now(&pool, publisher.publisher_id),
            ThothPackage::Pyramid
        );
    }

    // ---------------------------------------------------------------------
    // Hash integrity
    // ---------------------------------------------------------------------

    #[test]
    fn plan_hash_mismatch_fails_before_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (_outcome, bytes) = run_dry_run(&pool, &manifest);
        let wrong = "0".repeat(64);
        let result = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &wrong,
            DistributionJobCreation::Off,
            None,
        );
        assert!(matches!(
            result,
            Err(ThothError::MigrationBackfillPlanHashMismatch)
        ));
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
    }

    #[test]
    fn manifest_hash_mismatch_fails_before_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        // Tamper with the manifest bytes after the plan recorded their hash.
        let tampered = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "SPHINX",
            &["ZENODO"],
        )]));
        let result = apply_plan(
            &pool,
            &tampered,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        );
        assert!(matches!(
            result,
            Err(ThothError::MigrationBackfillManifestHashMismatch)
        ));
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
    }

    // ---------------------------------------------------------------------
    // Job safety
    // ---------------------------------------------------------------------

    #[test]
    fn migration_backfill_creates_no_job_with_switch_off_or_on() {
        for switch in [DistributionJobCreation::Off, DistributionJobCreation::On] {
            let (_guard, pool) = setup_test_db();
            let publisher = create_publisher(&pool);
            set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
            // ZENODO is an AUTOMATIC_PUSH destination: a SUPERUSER_API activation
            // would qualify for a job. MIGRATION_BACKFILL must not create one.
            let manifest = write_manifest(&manifest_value(&[(
                publisher.publisher_id,
                "OBELISK",
                &["ZENODO"],
            )]));
            let (outcome, bytes) = run_dry_run(&pool, &manifest);
            let sha = outcome.plan_sha256.clone();
            apply_plan(&pool, &manifest, &bytes, &sha, switch, None).unwrap();
            assert_eq!(
                table_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
                0
            );
            assert_eq!(
                table_count(
                    &pool,
                    "SELECT count(*) AS count FROM distribution_job_target"
                ),
                0
            );
            assert_eq!(
                table_count(
                    &pool,
                    "SELECT count(*) AS count FROM distribution_job_attempt"
                ),
                0
            );
        }
    }

    #[test]
    fn production_preflight_passes_only_when_off_and_empty() {
        let (_guard, pool) = setup_test_db();
        // OFF + empty job state passes.
        production_job_state_preflight(&pool, DistributionJobCreation::Off)
            .expect("clean preflight");
        // Switch ON is refused.
        assert!(matches!(
            production_job_state_preflight(&pool, DistributionJobCreation::On),
            Err(ThothError::MigrationBackfillProductionPrecondition(_))
        ));
    }

    #[test]
    fn production_preflight_rejects_nonzero_job_target_and_attempt() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let activation = Uuid::new_v4();
        // The stored deduplication key must satisfy the database formula check.
        let dedup_key = format!(
            "PUBLISHER_BACK_CATALOGUE:{}:{activation}",
            publisher.publisher_id
        );
        {
            let mut connection = pool.get().unwrap();
            sql_query(format!(
                "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
                 VALUES ('PUBLISHER_BACK_CATALOGUE', '{}', '{activation}', '{dedup_key}')",
                publisher.publisher_id
            ))
            .execute(&mut connection)
            .unwrap();
        }
        let job_id = table_count(&pool, "SELECT count(*) AS count FROM distribution_job");
        assert_eq!(job_id, 1);
        let job_uuid: String = {
            #[derive(diesel::QueryableByName)]
            struct Row {
                #[diesel(sql_type = diesel::sql_types::Text)]
                id: String,
            }
            let mut connection = pool.get().unwrap();
            let rows: Vec<Row> = diesel::sql_query(
                "SELECT distribution_job_id::text AS id FROM distribution_job LIMIT 1",
            )
            .load(&mut connection)
            .unwrap();
            rows[0].id.clone()
        };
        {
            let mut connection = pool.get().unwrap();
            sql_query(format!(
                "INSERT INTO distribution_job_target (distribution_job_id, platform) \
                 VALUES ('{job_uuid}', 'ZENODO')"
            ))
            .execute(&mut connection)
            .unwrap();
            sql_query(format!(
                "INSERT INTO distribution_job_attempt \
                 (distribution_job_id, attempt_number, claim_token, claimed_by) \
                 VALUES ('{job_uuid}', 1, gen_random_uuid(), 'worker')"
            ))
            .execute(&mut connection)
            .unwrap();
        }
        // Even with the switch OFF, any non-empty job state is refused, and the
        // message reflects all three table counts.
        let error = production_job_state_preflight(&pool, DistributionJobCreation::Off)
            .expect_err("nonempty job state must be refused");
        let message = error.to_string();
        assert!(message.contains("jobs=1"), "{message}");
        assert!(message.contains("targets=1"), "{message}");
        assert!(message.contains("attempts=1"), "{message}");
    }

    // ---------------------------------------------------------------------
    // Lock envelope and synthetic volume
    // ---------------------------------------------------------------------

    /// Configurable synthetic-volume fixture: create `count` works under a fresh
    /// publisher/imprint and return the publisher id.
    fn seed_publisher_with_works(pool: &PgPool, count: i64) -> Uuid {
        let publisher = create_publisher(pool);
        let imprint = create_imprint(pool, &publisher);
        for _ in 0..count {
            create_work(pool, &imprint);
        }
        publisher.publisher_id
    }

    #[test]
    fn synthetic_work_volume_is_reflected_in_affected_work_count() {
        // A configurable fixture exercised across several representative sizes,
        // rather than a single hard-coded volume.
        for volume in [0_i64, 1, 5, 25] {
            let (_guard, pool) = setup_test_db();
            let publisher_id = seed_publisher_with_works(&pool, volume);
            set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
            let manifest =
                write_manifest(&manifest_value(&[(publisher_id, "OBELISK", &["ZENODO"])]));
            let (outcome, _bytes) = run_dry_run(&pool, &manifest);
            let entry = &outcome.plan.entries[0];
            assert_eq!(entry.affected_work_count, volume, "volume {volume}");
            assert_eq!(
                outcome.plan.expected.affected_works, volume,
                "volume {volume}"
            );
            assert_eq!(
                outcome.plan.expected.max_works_per_publisher, volume,
                "volume {volume}"
            );
        }
    }

    #[test]
    fn work_count_drift_after_dry_run_is_operational_not_a_config_token() {
        // Work count is an operational estimate, not a configuration concurrency
        // token: works added AFTER the dry run do not invalidate the reviewed
        // configuration state, and apply succeeds when the NEW current count is
        // within the approved envelope.
        let (_guard, pool) = setup_test_db();
        let publisher_id = seed_publisher_with_works(&pool, 2);
        set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
        let imprint = create_imprint(&pool, &Publisher::from_id(&pool, &publisher_id).unwrap());
        let manifest = write_manifest(&manifest_value(&[(publisher_id, "OBELISK", &["ZENODO"])]));

        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        assert_eq!(outcome.plan.entries[0].affected_work_count, 2);
        let sha = outcome.plan_sha256.clone();

        // Two more works appear after the dry run: current count becomes 4.
        for _ in 0..2 {
            create_work(&pool, &imprint);
        }
        // Envelope 5 >= new current count 4: apply still succeeds (the reviewed
        // configuration state and token are unchanged).
        let applied = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            Some(5),
        )
        .expect("within-envelope apply succeeds despite work drift");
        assert_eq!(applied.written, 1);
        assert_eq!(package_now(&pool, publisher_id), ThothPackage::Obelisk);
    }

    #[test]
    fn work_count_growth_past_the_envelope_after_dry_run_stops_before_write() {
        let (_guard, pool) = setup_test_db();
        let publisher_id = seed_publisher_with_works(&pool, 2);
        set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
        let imprint = create_imprint(&pool, &Publisher::from_id(&pool, &publisher_id).unwrap());
        let manifest = write_manifest(&manifest_value(&[(publisher_id, "OBELISK", &["ZENODO"])]));

        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        assert_eq!(outcome.plan.entries[0].affected_work_count, 2);
        let sha = outcome.plan_sha256.clone();

        // Works added after the dry run push the current count to 6, above the
        // approved envelope of 4: apply stops before writing that publisher.
        for _ in 0..4 {
            create_work(&pool, &imprint);
        }
        let stopped = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            Some(4),
        );
        assert!(matches!(
            stopped,
            Err(ThothError::MigrationBackfillLockEnvelopeExceeded(_))
        ));
        assert!(mig_history(&pool, publisher_id).is_empty());
    }

    #[test]
    fn work_count_within_envelope_applies_and_over_envelope_stops() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        for _ in 0..3 {
            create_work(&pool, &imprint);
        }
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        // Over-envelope (envelope 2 < 3 works) stops before any write.
        let stopped = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            Some(2),
        );
        assert!(matches!(
            stopped,
            Err(ThothError::MigrationBackfillLockEnvelopeExceeded(_))
        ));
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());

        // Within-envelope (envelope 3) applies.
        let applied = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            Some(3),
        )
        .unwrap();
        assert_eq!(applied.written, 1);
    }

    // ---------------------------------------------------------------------
    // Licence audit and omissions
    // ---------------------------------------------------------------------

    #[test]
    fn unsupported_licence_values_are_reported_with_their_disposition() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work = create_work(&pool, &imprint);
        {
            let mut connection = pool.get().unwrap();
            sql_query(format!(
                "UPDATE work SET license = 'https://example.org/legacy-licence' WHERE work_id = '{}'",
                work.work_id
            ))
            .execute(&mut connection)
            .unwrap();
        }
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);

        let manifest_value = serde_json::json!({
            "manifestVersion": 1,
            "publishers": [{
                "publisherId": publisher.publisher_id.to_string(),
                "subscriptionPackage": "OBELISK",
                "enabledDistributionPlatforms": ["ZENODO"],
            }],
            "licenceDispositions": [
                { "value": "https://example.org/legacy-licence", "disposition": "NORMALIZE" }
            ]
        });
        let manifest = write_manifest(&manifest_value);
        let (outcome, _bytes) = run_dry_run(&pool, &manifest);
        assert_eq!(outcome.report.unsupported_licences.len(), 1);
        let finding = &outcome.report.unsupported_licences[0];
        assert_eq!(finding.value, "https://example.org/legacy-licence");
        assert_eq!(finding.disposition, LicenceDispositionReport::Normalize);
        assert_eq!(finding.work_count, 1);
    }

    #[test]
    fn publishers_absent_from_the_manifest_are_reported_as_omissions() {
        let (_guard, pool) = setup_test_db();
        let mapped = create_publisher(&pool);
        let omitted = create_publisher(&pool);
        set_state(&pool, mapped.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            mapped.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, _bytes) = run_dry_run(&pool, &manifest);
        assert!(outcome
            .report
            .omitted_publishers
            .iter()
            .any(|entry| entry.publisher_id == omitted.publisher_id));
        assert!(!outcome
            .report
            .omitted_publishers
            .iter()
            .any(|entry| entry.publisher_id == mapped.publisher_id));
    }

    // ---------------------------------------------------------------------
    // B3 - duplicate licence dispositions
    // ---------------------------------------------------------------------

    fn manifest_with_licences(publisher_id: Uuid, licences: &[(&str, &str)]) -> serde_json::Value {
        let dispositions: Vec<serde_json::Value> = licences
            .iter()
            .map(|(value, disposition)| serde_json::json!({ "value": value, "disposition": disposition }))
            .collect();
        serde_json::json!({
            "manifestVersion": 1,
            "publishers": [{
                "publisherId": publisher_id.to_string(),
                "subscriptionPackage": "OBELISK",
                "enabledDistributionPlatforms": ["ZENODO"],
            }],
            "licenceDispositions": dispositions,
        })
    }

    #[test]
    fn duplicate_licence_dispositions_are_rejected_regardless_of_agreement() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);

        // Same value, different disposition — ambiguous, rejected.
        let conflicting = write_manifest(&manifest_with_licences(
            publisher.publisher_id,
            &[("cc-by", "SUPPORTED"), ("cc-by", "NORMALIZE")],
        ));
        assert!(matches!(
            try_dry_run(&pool, &conflicting),
            Err(ThothError::MigrationBackfillManifestInvalid(_))
        ));

        // Same value, same disposition — still rejected (no silent dedup).
        let duplicated = write_manifest(&manifest_with_licences(
            publisher.publisher_id,
            &[("cc-by", "SUPPORTED"), ("cc-by", "SUPPORTED")],
        ));
        assert!(matches!(
            try_dry_run(&pool, &duplicated),
            Err(ThothError::MigrationBackfillManifestInvalid(_))
        ));

        // Distinct values — accepted.
        let unique = write_manifest(&manifest_with_licences(
            publisher.publisher_id,
            &[("cc-by", "SUPPORTED"), ("cc-by-nc", "NORMALIZE")],
        ));
        assert!(try_dry_run(&pool, &unique).is_ok());
    }

    // ---------------------------------------------------------------------
    // B1 - fail-closed production apply controls
    // ---------------------------------------------------------------------

    /// Establish a publisher whose only work carries an explicit licence value.
    fn publisher_with_licence(pool: &PgPool, licence: &str) -> Uuid {
        let publisher = create_publisher(pool);
        let imprint = create_imprint(pool, &publisher);
        let work = create_work(pool, &imprint);
        let mut connection = pool.get().unwrap();
        sql_query(format!(
            "UPDATE work SET license = '{licence}' WHERE work_id = '{}'",
            work.work_id
        ))
        .execute(&mut connection)
        .unwrap();
        publisher.publisher_id
    }

    #[test]
    fn production_apply_succeeds_only_off_empty_and_leaves_zero_jobs() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        let applied = apply_plan_mode(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            ApplyExecutionMode::Production {
                max_works_per_publisher: 10,
            },
        )
        .expect("production apply succeeds under OFF + empty job state");
        assert_eq!(applied.written, 1);
        // The post-apply invariant held: job tables remain empty.
        assert_eq!(
            table_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
            0
        );
    }

    #[test]
    fn production_apply_rejects_switch_on_before_any_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let result = apply_plan_mode(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::On,
            ApplyExecutionMode::Production {
                max_works_per_publisher: 10,
            },
        );
        assert!(matches!(
            result,
            Err(ThothError::MigrationBackfillProductionPrecondition(_))
        ));
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
    }

    #[test]
    fn production_apply_rejects_nonzero_job_state_before_any_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        // A pre-existing job makes the strict production precondition fail.
        let activation = Uuid::new_v4();
        let dedup_key = format!(
            "PUBLISHER_BACK_CATALOGUE:{}:{activation}",
            publisher.publisher_id
        );
        {
            let mut connection = pool.get().unwrap();
            sql_query(format!(
                "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
                 VALUES ('PUBLISHER_BACK_CATALOGUE', '{}', '{activation}', '{dedup_key}')",
                publisher.publisher_id
            ))
            .execute(&mut connection)
            .unwrap();
        }
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let result = apply_plan_mode(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            ApplyExecutionMode::Production {
                max_works_per_publisher: 10,
            },
        );
        assert!(matches!(
            result,
            Err(ThothError::MigrationBackfillProductionPrecondition(_))
        ));
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
    }

    #[test]
    fn post_apply_job_invariant_detects_a_nonzero_job_table() {
        // The end-of-invocation invariant fails closed if a job row exists, and
        // passes on an empty job state.
        let (_guard, pool) = setup_test_db();
        production_post_apply_job_invariant(&pool).expect("empty job state passes");
        let publisher = create_publisher(&pool);
        let activation = Uuid::new_v4();
        let dedup_key = format!(
            "PUBLISHER_BACK_CATALOGUE:{}:{activation}",
            publisher.publisher_id
        );
        {
            let mut connection = pool.get().unwrap();
            sql_query(format!(
                "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
                 VALUES ('PUBLISHER_BACK_CATALOGUE', '{}', '{activation}', '{dedup_key}')",
                publisher.publisher_id
            ))
            .execute(&mut connection)
            .unwrap();
        }
        assert!(matches!(
            production_post_apply_job_invariant(&pool),
            Err(ThothError::MigrationBackfillProductionPrecondition(_))
        ));
    }

    // ---------------------------------------------------------------------
    // B2 - production licence fail-closed
    // ---------------------------------------------------------------------

    #[test]
    fn production_apply_stops_on_an_unreviewed_licence_before_any_write() {
        let (_guard, pool) = setup_test_db();
        let licence = "https://example.org/unreviewed-licence";
        let publisher_id = publisher_with_licence(&pool, licence);
        set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
        // The manifest declares NO disposition for the observed licence.
        let manifest = write_manifest(&manifest_value(&[(publisher_id, "OBELISK", &["ZENODO"])]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();

        let result = apply_plan_mode(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            ApplyExecutionMode::Production {
                max_works_per_publisher: 10,
            },
        );
        assert!(matches!(
            result,
            Err(ThothError::MigrationBackfillUnresolvedLicence(_))
        ));
        // No configuration write occurred, and the licence value is untouched.
        assert!(mig_history(&pool, publisher_id).is_empty());
        assert_eq!(package_now(&pool, publisher_id), ThothPackage::Oasis);
        let stored = licence_of_only_work(&pool, publisher_id);
        assert_eq!(
            stored.as_deref(),
            Some(licence),
            "licence must not be rewritten"
        );
    }

    #[test]
    fn production_apply_uses_the_exact_reviewed_disposition() {
        let licence = "https://example.org/reviewed-licence";
        // NORMALIZE (requires a separate action) blocks the production apply.
        {
            let (_guard, pool) = setup_test_db();
            let publisher_id = publisher_with_licence(&pool, licence);
            set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
            let manifest = write_manifest(&manifest_with_licences(
                publisher_id,
                &[(licence, "NORMALIZE")],
            ));
            let (outcome, bytes) = run_dry_run(&pool, &manifest);
            let sha = outcome.plan_sha256.clone();
            let result = apply_plan_mode(
                &pool,
                &manifest,
                &bytes,
                &sha,
                DistributionJobCreation::Off,
                ApplyExecutionMode::Production {
                    max_works_per_publisher: 10,
                },
            );
            assert!(matches!(
                result,
                Err(ThothError::MigrationBackfillUnresolvedLicence(_))
            ));
            assert!(mig_history(&pool, publisher_id).is_empty());
        }
        // SUPPORTED lets the production apply proceed.
        {
            let (_guard, pool) = setup_test_db();
            let publisher_id = publisher_with_licence(&pool, licence);
            set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
            let manifest = write_manifest(&manifest_with_licences(
                publisher_id,
                &[(licence, "SUPPORTED")],
            ));
            let (outcome, bytes) = run_dry_run(&pool, &manifest);
            let sha = outcome.plan_sha256.clone();
            let applied = apply_plan_mode(
                &pool,
                &manifest,
                &bytes,
                &sha,
                DistributionJobCreation::Off,
                ApplyExecutionMode::Production {
                    max_works_per_publisher: 10,
                },
            )
            .expect("supported licence permits production apply");
            assert_eq!(applied.written, 1);
            // The licence value is still not rewritten.
            assert_eq!(
                licence_of_only_work(&pool, publisher_id).as_deref(),
                Some(licence)
            );
        }
    }

    #[test]
    fn disposable_apply_does_not_apply_the_production_licence_gate() {
        // The licence fail-closed gate is a production-mode control; disposable
        // apply (used for configuration-mechanics testing) is not gated by it.
        let (_guard, pool) = setup_test_db();
        let publisher_id = publisher_with_licence(&pool, "https://example.org/unreviewed");
        set_state(&pool, publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(publisher_id, "OBELISK", &["ZENODO"])]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let applied = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        )
        .unwrap();
        assert_eq!(applied.written, 1);
    }

    fn licence_of_only_work(pool: &PgPool, publisher_id: Uuid) -> Option<String> {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            license: Option<String>,
        }
        let mut connection = pool.get().unwrap();
        let rows: Vec<Row> = diesel::sql_query(format!(
            "SELECT w.license FROM work w JOIN imprint i ON w.imprint_id = i.imprint_id \
             WHERE i.publisher_id = '{publisher_id}' LIMIT 1"
        ))
        .load(&mut connection)
        .unwrap();
        rows.into_iter().next().and_then(|row| row.license)
    }

    // ---------------------------------------------------------------------
    // B4A - late drift after an earlier commit, fail-closed + forward repair
    // ---------------------------------------------------------------------

    #[test]
    fn late_drift_after_an_earlier_commit_fails_closed_and_enables_forward_repair() {
        let (_guard, pool) = setup_test_db();
        let a = create_publisher(&pool);
        let b = create_publisher(&pool);
        set_state(&pool, a.publisher_id, ThothPackage::Oasis, &[]);
        set_state(&pool, b.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[
            (a.publisher_id, "OBELISK", &["ZENODO"]),
            (b.publisher_id, "SPHINX", &["INTERNET_ARCHIVE"]),
        ]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let actor = audit_actor(&sha);

        // Publisher A durably commits under this exact plan's derived actor.
        let entry_a = outcome
            .plan
            .entries
            .iter()
            .find(|entry| entry.publisher_id == a.publisher_id)
            .unwrap();
        let context = ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::MigrationBackfill,
            actor: &actor,
            job_creation: DistributionJobCreation::Off,
        };
        replace_publisher_service_configuration(
            &pool,
            &context,
            &ReplacePublisherServiceConfigurationInput {
                publisher_id: a.publisher_id,
                subscription_package: entry_a.desired.subscription_package,
                enabled_distribution_platforms: entry_a
                    .desired
                    .enabled_distribution_platforms
                    .clone(),
                expected_updated_at: entry_a.reviewed_configuration_version,
            },
        )
        .unwrap();

        // A genuine late drift invalidates publisher B's reviewed before-state.
        set_state(&pool, b.publisher_id, ThothPackage::Pyramid, &[]);

        // Re-applying the SAME reviewed plan fails closed on B's drift and writes
        // nothing new. A is not double-written; B is not written.
        let result = apply_plan(
            &pool,
            &manifest,
            &bytes,
            &sha,
            DistributionJobCreation::Off,
            None,
        );
        assert!(matches!(result, Err(ThothError::MigrationBackfillDrift(_))));
        assert_eq!(
            mig_history(&pool, a.publisher_id).len(),
            1,
            "A committed exactly once"
        );
        assert!(
            mig_history(&pool, b.publisher_id).is_empty(),
            "B must not be written under the invalidated plan"
        );
        assert_eq!(package_now(&pool, b.publisher_id), ThothPackage::Pyramid);

        // A remains recognizable as applied-by-this-plan: exact source, derived
        // actor, before, after and current state all line up.
        let row = &mig_history(&pool, a.publisher_id)[0];
        assert_eq!(
            row.source,
            PublisherServiceConfigurationSource::MigrationBackfill
        );
        assert_eq!(row.actor, actor);
        let before = typed_state(&row.before_state).unwrap();
        let after = typed_state(&row.after_state).unwrap();
        assert_eq!(before.subscription_package, ThothPackage::Oasis);
        assert_eq!(after.subscription_package, ThothPackage::Obelisk);
        let current_a = current_state(&pool, a.publisher_id).unwrap();
        assert_eq!(current_a, after);
        assert_eq!(
            classify_entry(&pool, entry_a, &actor).unwrap(),
            ResumeClassification::AlreadyAppliedByThisPlan
        );

        // Forward repair: a fresh dry run against the drifted state yields a new
        // reviewed plan where A is a no-op and B reflects its NEW before-state.
        let (repair, _bytes) = run_dry_run(&pool, &manifest);
        let repair_a = repair
            .plan
            .entries
            .iter()
            .find(|entry| entry.publisher_id == a.publisher_id)
            .unwrap();
        let repair_b = repair
            .plan
            .entries
            .iter()
            .find(|entry| entry.publisher_id == b.publisher_id)
            .unwrap();
        assert_eq!(repair_a.classification, PlanClassification::ReviewedNoop);
        assert_eq!(repair_b.classification, PlanClassification::Pending);
        assert_eq!(
            repair_b.before.subscription_package,
            ThothPackage::Pyramid,
            "the forward-repair plan captures B's genuinely new before-state"
        );
        assert_ne!(
            repair.plan_sha256, sha,
            "forward repair is a fresh reviewed plan, not the invalidated one"
        );
    }

    // ---------------------------------------------------------------------
    // B5 - artifact alias protection
    // ---------------------------------------------------------------------

    #[test]
    fn dry_run_rejects_aliasing_output_and_input_paths() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let manifest_before = std::fs::read(&manifest).unwrap();
        let report = tmp_path("report.json");

        // Manifest == plan output.
        let request = DryRunRequest {
            manifest_path: &manifest,
            plan_out_path: &manifest,
            report_out_path: &report,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            dry_run(&pool, &request),
            Err(ThothError::MigrationBackfillArtifactAlias(_))
        ));
        // Plan output == report output.
        let plan_out = tmp_path("shared.json");
        let request = DryRunRequest {
            manifest_path: &manifest,
            plan_out_path: &plan_out,
            report_out_path: &plan_out,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            dry_run(&pool, &request),
            Err(ThothError::MigrationBackfillArtifactAlias(_))
        ));
        // The manifest was left untouched by the rejected invocations.
        assert_eq!(std::fs::read(&manifest).unwrap(), manifest_before);
        assert!(!plan_out.exists(), "no output written on rejection");
    }

    #[test]
    fn apply_rejects_aliasing_plan_manifest_and_report_paths() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);
        let manifest = write_manifest(&manifest_value(&[(
            publisher.publisher_id,
            "OBELISK",
            &["ZENODO"],
        )]));
        let (outcome, bytes) = run_dry_run(&pool, &manifest);
        let sha = outcome.plan_sha256.clone();
        let plan_path = tmp_path("reviewed-plan.json");
        std::fs::write(&plan_path, &bytes).unwrap();
        let plan_before = std::fs::read(&plan_path).unwrap();

        // Reviewed plan == report output would destroy the reviewed plan.
        let request = ApplyRequest {
            manifest_path: &manifest,
            plan_path: &plan_path,
            expected_plan_sha256: &sha,
            report_out_path: &plan_path,
            mode: ApplyExecutionMode::Disposable {
                max_works_per_publisher: None,
            },
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            apply(&pool, &request),
            Err(ThothError::MigrationBackfillArtifactAlias(_))
        ));

        // Manifest == report output.
        let request = ApplyRequest {
            manifest_path: &manifest,
            plan_path: &plan_path,
            expected_plan_sha256: &sha,
            report_out_path: &manifest,
            mode: ApplyExecutionMode::Disposable {
                max_works_per_publisher: None,
            },
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            apply(&pool, &request),
            Err(ThothError::MigrationBackfillArtifactAlias(_))
        ));

        // Manifest == reviewed plan (caught before parsing).
        let request = ApplyRequest {
            manifest_path: &manifest,
            plan_path: &manifest,
            expected_plan_sha256: &sha,
            report_out_path: &tmp_path("r.json"),
            mode: ApplyExecutionMode::Disposable {
                max_works_per_publisher: None,
            },
            job_creation: DistributionJobCreation::Off,
        };
        assert!(matches!(
            apply(&pool, &request),
            Err(ThothError::MigrationBackfillArtifactAlias(_))
        ));

        // No write occurred and the reviewed plan is intact.
        assert_eq!(std::fs::read(&plan_path).unwrap(), plan_before);
        assert!(mig_history(&pool, publisher.publisher_id).is_empty());
    }

    #[test]
    fn artifact_alias_detection_normalizes_relative_and_symlink_paths() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        set_state(&pool, publisher.publisher_id, ThothPackage::Oasis, &[]);

        // Relative alias: a bare filename and its "./name" form resolve equal.
        let dir = std::env::temp_dir();
        let unique = format!("mig01-alias-{}.json", Uuid::new_v4());
        let manifest_value = manifest_value(&[(publisher.publisher_id, "OBELISK", &["ZENODO"])]);
        let absolute = dir.join(&unique);
        std::fs::write(&absolute, serde_json::to_vec(&manifest_value).unwrap()).unwrap();
        let alias = dir.join(format!("./{unique}"));
        let report = tmp_path("report.json");
        let request = DryRunRequest {
            manifest_path: &absolute,
            plan_out_path: &alias,
            report_out_path: &report,
            run_production_preflight: false,
            job_creation: DistributionJobCreation::Off,
        };
        assert!(
            matches!(
                dry_run(&pool, &request),
                Err(ThothError::MigrationBackfillArtifactAlias(_))
            ),
            "a relative-path alias of the manifest must be rejected"
        );

        // Symlink alias: a symlink pointing at the manifest resolves equal to it.
        let link = dir.join(format!("mig01-link-{}.json", Uuid::new_v4()));
        if std::os::unix::fs::symlink(&absolute, &link).is_ok() {
            let request = DryRunRequest {
                manifest_path: &absolute,
                plan_out_path: &link,
                report_out_path: &report,
                run_production_preflight: false,
                job_creation: DistributionJobCreation::Off,
            };
            assert!(
                matches!(
                    dry_run(&pool, &request),
                    Err(ThothError::MigrationBackfillArtifactAlias(_))
                ),
                "a symlink alias of the manifest must be rejected"
            );
            let _ = std::fs::remove_file(&link);
        }
        let _ = std::fs::remove_file(&absolute);
    }
}
