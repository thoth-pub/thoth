//! The `MIG-01-LIC-NORM-01` administrative licence-normalization facade.
//!
//! This module is the single, deliberately narrow, workspace-visible entry
//! point for the separately gated deterministic production licence
//! normalization (owning issue [#828]). It is a controlled administrative
//! operation over exactly the 24 reviewed deterministic replacement rules —
//! **not** a generic licence canonicalizer — and it owns all domain-sensitive
//! behaviour required by that operation:
//!
//! - reading, raw-byte-SHA-256-verifying and parsing the three immutable
//!   reviewed inputs (deterministic normalization manifest, manual-resolution
//!   register and the bound MIG-01 production manifest);
//! - mechanically enforcing every reviewed representation-only invariant on
//!   the deterministic rules, including canonical-target shape, prefix and
//!   discarded-suffix constraints, and validation of every canonical target
//!   through the repository-authoritative `cc_license` parser;
//! - requiring every distinct deterministic target to be exact-string
//!   `SUPPORTED` in the exact bound MIG-01 production manifest;
//! - resolving the exact affected Works and failing closed on any publisher
//!   outside the bound MIG-01 publisher scope;
//! - emitting a deterministic, canonical, raw-byte-hashed dry-run plan and a
//!   bounded reconciliation report, with **no** database write;
//! - deterministically classifying every plan entry before the first write
//!   (`PENDING` / `ALREADY_APPLIED_BY_THIS_PLAN` / `DRIFT`);
//! - applying pending entries one Work per transaction: row lock, reviewed
//!   state recheck under the lock, exactly one normal `work_history` row from
//!   the complete pre-update Work with the plan-derived actor, and a
//!   single-column `work.license` update whose business-data SET surface is
//!   only the licence;
//! - post-write reconciliation after the last write transaction and before
//!   any successful APPLY outcome: the deterministic-source residual query
//!   must come back empty (fail-closed otherwise, with committed Works
//!   preserved), and the report's resulting target counts and manual
//!   residuals are one consistent current snapshot, distinct from the
//!   reviewed planned conversion tallies.
//!
//! The 28 manual-resolution values are **never** executable here: they are
//! read, counted and reported only. This path deliberately does not use
//! GraphQL `updateWork`, `Crud::update` or `PatchWork`, and it does not invoke
//! `WorkPolicy`: authorization derives from the separately reviewed and
//! authorized administrative gate, substituted by the bounded controls above
//! (immutable hashed inputs and plan, exact source-value membership, typed
//! update-token comparison, row locks, single-column writes, history evidence
//! and deterministic reconciliation). That exception is private to this module
//! and its thin CLI wiring; no GraphQL surface reaches it.
//!
//! A successful licence write changes ordinary Work `updated_at` /
//! `updated_at_with_relations` freshness through the existing database
//! triggers, so stale export/specification caches for affected publishers
//! regenerate on their next request. That expected read-time regeneration load
//! is recorded in the report; it is not dissemination, creates no distribution
//! job, and this module never triggers cache regeneration itself.
//!
//! Gate B authorises implementation and disposable/local testing only. No
//! production data is read and nothing is executed against production here.
//!
//! [#828]: https://github.com/thoth-pub/thoth/issues/828

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use thoth_errors::{ThothError, ThothResult};

use super::Work;
use crate::db::PgPool;
use crate::model::publisher_service_configuration::migration_backfill::{
    parse_manifest as parse_mig01_manifest, sha256_hex, LicenceDisposition, MigrationManifest,
};
use crate::model::{DbInsert, HistoryEntry, Timestamp};
use crate::schema::{imprint, work, work_history};

/// The exact reviewed task identity every input artifact must declare.
pub const TASK_ID: &str = "MIG-01-LIC-NORM-01";

/// The only deterministic-manifest schema version this tool accepts.
pub const DETERMINISTIC_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// The only manual-register schema version this tool accepts.
pub const MANUAL_REGISTER_SCHEMA_VERSION: u32 = 1;

/// The only plan schema version this tool emits or accepts.
pub const PLAN_SCHEMA_VERSION: u32 = 1;

/// The reviewed deterministic rule count. Any other count is a STOP: changing
/// the executable mapping set requires a freshly hashed manifest and review.
pub const DETERMINISTIC_RULE_COUNT: usize = 24;

/// The reviewed manual-resolution value count. Any other count is a STOP.
pub const MANUAL_VALUE_COUNT: usize = 28;

/// The reviewed distinct canonical target count. Any other count is a STOP.
pub const DETERMINISTIC_TARGET_COUNT: usize = 9;

/// The fixed audit-actor namespace prefix. The full actor is
/// `MIG-01-LIC-NORM:<lowercase-plan-sha256>`, derived **after** the plan bytes
/// are hashed, so the actor is never embedded in the hashed plan.
pub const AUDIT_ACTOR_PREFIX: &str = "MIG-01-LIC-NORM:";

/// The fixed operational note recorded in every report: the expected
/// export/specification cache effect of successful licence writes.
pub const EXPORT_CACHE_EFFECT_NOTE: &str =
    "Successful licence writes change ordinary Work updated_at/updated_at_with_relations \
     freshness through the existing database triggers, so stale export/specification caches \
     for affected publisher/specification combinations regenerate on their next request. This \
     is expected read-time regeneration load only: it is not dissemination, creates no \
     distribution job, and no cache regeneration is triggered by this tool.";

lazy_static! {
    /// The reviewed canonical-target shape: an https Creative Commons
    /// `licenses`/`publicdomain` URL with exactly two further path segments and
    /// a trailing slash.
    static ref CANONICAL_TARGET: Regex =
        Regex::new(r"^https://creativecommons\.org/(licenses|publicdomain)/[^/]+/[^/]+/$")
            .expect("canonical-target pattern compiles");
    /// The reviewed discarded-suffix shape: a `deed`/`legalcode` token with an
    /// optional language tag and optional trailing slash. Anything else — in
    /// particular a jurisdiction path segment — is not a removable suffix.
    static ref DISCARDED_SUFFIX: Regex =
        Regex::new(r"^(deed|legalcode)(\.[A-Za-z][A-Za-z0-9-]*)?/?$")
            .expect("discarded-suffix pattern compiles");
}

// ---------------------------------------------------------------------------
// Immutable reviewed input artifacts
// ---------------------------------------------------------------------------

/// The reviewed deterministic normalization manifest.
///
/// Its identity is the SHA-256 of its exact raw bytes; it is never normalized
/// before hashing. Unknown fields are rejected so the manifest cannot silently
/// carry unreviewed instructions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeterministicManifest {
    pub schema_version: u32,
    pub task: String,
    #[serde(default)]
    pub description: Option<String>,
    pub replacements: Vec<ReplacementRule>,
}

/// One reviewed representation-only replacement rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplacementRule {
    /// The exact stored production value this rule replaces.
    pub from: String,
    /// The exact canonical value it becomes.
    pub to: String,
    /// Free-text review provenance. Never parsed.
    #[serde(default)]
    pub reason: Option<String>,
    /// Publisher labels recorded at review time. Never parsed; scope is bound
    /// to the MIG-01 manifest's canonical publisher UUIDs, not these labels.
    #[serde(default)]
    pub observed_publishers: Vec<String>,
}

/// The reviewed manual-resolution register: non-executable evidence only.
///
/// Values in this register MUST NOT be used as an automatic replacement
/// source. This module only counts and reports them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualResolutionRegister {
    pub schema_version: u32,
    pub task: String,
    #[serde(default)]
    pub description: Option<String>,
    pub values: Vec<ManualValue>,
}

/// One manual-resolution value. Never executable.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualValue {
    pub value: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub observed_publishers: Vec<String>,
}

/// The three verified immutable inputs, with their exact raw-byte hashes.
#[derive(Debug)]
pub struct VerifiedInputs {
    pub deterministic: DeterministicManifest,
    pub deterministic_sha256: String,
    pub manual: ManualResolutionRegister,
    pub manual_sha256: String,
    pub mig01: MigrationManifest,
    pub mig01_sha256: String,
}

impl VerifiedInputs {
    /// The reviewed `from -> to` replacement mapping, ordered by source value.
    fn rules(&self) -> BTreeMap<&str, &str> {
        self.deterministic
            .replacements
            .iter()
            .map(|rule| (rule.from.as_str(), rule.to.as_str()))
            .collect()
    }

    /// The distinct canonical targets, ascending.
    fn targets(&self) -> BTreeSet<&str> {
        self.deterministic
            .replacements
            .iter()
            .map(|rule| rule.to.as_str())
            .collect()
    }

    /// The bound MIG-01 canonical publisher scope.
    fn publisher_scope(&self) -> HashSet<Uuid> {
        self.mig01
            .publishers
            .iter()
            .map(|entry| entry.publisher_id)
            .collect()
    }
}

/// Parse and mechanically validate the deterministic manifest from raw bytes.
pub fn parse_deterministic_manifest(bytes: &[u8]) -> ThothResult<DeterministicManifest> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ThothError::LicenceNormalizationInvalidInput(
            "the deterministic manifest must not begin with a byte-order mark".to_string(),
        ));
    }
    let manifest: DeterministicManifest = serde_json::from_slice(bytes).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "the deterministic manifest is not valid JSON: {error}"
        ))
    })?;
    validate_deterministic_manifest(&manifest)?;
    Ok(manifest)
}

/// Enforce every reviewed mechanical invariant on the deterministic manifest.
///
/// Nothing in this validator infers, repairs or reinterprets a value: a rule
/// either is an exact reviewed representation-only `deed`/`legalcode` suffix
/// removal onto a `cc_license`-parseable canonical target, or it is rejected.
fn validate_deterministic_manifest(manifest: &DeterministicManifest) -> ThothResult<()> {
    if manifest.schema_version != DETERMINISTIC_MANIFEST_SCHEMA_VERSION {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "unsupported deterministic manifest schema version {} (expected {})",
            manifest.schema_version, DETERMINISTIC_MANIFEST_SCHEMA_VERSION
        )));
    }
    if manifest.task != TASK_ID {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "the deterministic manifest declares task {:?} (expected {TASK_ID:?})",
            manifest.task
        )));
    }
    // Set-level checks run first so a self-map, duplicate or replacement
    // chain is named as such rather than surfacing as an incidental
    // shape violation of one of its rules.
    for (index, rule) in manifest.replacements.iter().enumerate() {
        if rule.from == rule.to {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "rule {index} maps a value to itself: {:?}",
                rule.from
            )));
        }
    }
    let mut froms: HashSet<&str> = HashSet::new();
    for rule in &manifest.replacements {
        if !froms.insert(rule.from.as_str()) {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "duplicate deterministic source value {:?}",
                rule.from
            )));
        }
    }
    for rule in &manifest.replacements {
        if froms.contains(rule.to.as_str()) {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "replacement chain: target {:?} also appears as a source value",
                rule.to
            )));
        }
    }
    for (index, rule) in manifest.replacements.iter().enumerate() {
        validate_rule(index, rule)?;
    }
    if manifest.replacements.len() != DETERMINISTIC_RULE_COUNT {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "the deterministic manifest carries {} rules (the reviewed set has exactly \
             {DETERMINISTIC_RULE_COUNT})",
            manifest.replacements.len()
        )));
    }
    let targets: BTreeSet<&str> = manifest
        .replacements
        .iter()
        .map(|rule| rule.to.as_str())
        .collect();
    if targets.len() != DETERMINISTIC_TARGET_COUNT {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "the deterministic manifest collapses to {} distinct targets (the reviewed set \
             has exactly {DETERMINISTIC_TARGET_COUNT})",
            targets.len()
        )));
    }
    Ok(())
}

/// Enforce the reviewed representation-only invariants on one rule.
fn validate_rule(index: usize, rule: &ReplacementRule) -> ThothResult<()> {
    if rule.from == rule.to {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "rule {index} maps a value to itself: {:?}",
            rule.from
        )));
    }
    if !CANONICAL_TARGET.is_match(&rule.to) {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "rule {index} target {:?} is not a canonical Creative Commons URL",
            rule.to
        )));
    }
    if !rule.from.starts_with(&rule.to) {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "rule {index} source {:?} does not start with its target {:?}",
            rule.from, rule.to
        )));
    }
    let suffix = &rule.from[rule.to.len()..];
    if !DISCARDED_SUFFIX.is_match(suffix) {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "rule {index} discards suffix {suffix:?}, which is not an approved deed/legalcode \
             token; jurisdiction, path, rights or version segments are never stripped"
        )));
    }
    if cc_license::License::from_url(&rule.to).is_err() {
        return Err(ThothError::LicenceNormalizationUnsupportedTarget(format!(
            "rule {index} target {:?} is not accepted by the canonical cc-license parser",
            rule.to
        )));
    }
    Ok(())
}

/// Parse and validate the manual-resolution register from raw bytes.
pub fn parse_manual_register(bytes: &[u8]) -> ThothResult<ManualResolutionRegister> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ThothError::LicenceNormalizationInvalidInput(
            "the manual-resolution register must not begin with a byte-order mark".to_string(),
        ));
    }
    let register: ManualResolutionRegister = serde_json::from_slice(bytes).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "the manual-resolution register is not valid JSON: {error}"
        ))
    })?;
    if register.schema_version != MANUAL_REGISTER_SCHEMA_VERSION {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "unsupported manual-register schema version {} (expected {})",
            register.schema_version, MANUAL_REGISTER_SCHEMA_VERSION
        )));
    }
    if register.task != TASK_ID {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "the manual-resolution register declares task {:?} (expected {TASK_ID:?})",
            register.task
        )));
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for value in &register.values {
        if !seen.insert(value.value.as_str()) {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "duplicate manual-resolution value {:?}",
                value.value
            )));
        }
    }
    if register.values.len() != MANUAL_VALUE_COUNT {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "the manual-resolution register carries {} values (the reviewed register has \
             exactly {MANUAL_VALUE_COUNT})",
            register.values.len()
        )));
    }
    Ok(register)
}

/// Require every distinct deterministic target to be exact-string `SUPPORTED`
/// in the exact bound MIG-01 production manifest.
///
/// The MIG-01 manifest parser has already rejected duplicate dispositions, so
/// each value has at most one unambiguous reviewed disposition.
fn verify_targets_supported(
    deterministic: &DeterministicManifest,
    mig01: &MigrationManifest,
) -> ThothResult<()> {
    let targets: BTreeSet<&str> = deterministic
        .replacements
        .iter()
        .map(|rule| rule.to.as_str())
        .collect();
    for target in targets {
        let disposition = mig01
            .licence_dispositions
            .iter()
            .find(|entry| entry.value == target)
            .map(|entry| entry.disposition);
        match disposition {
            Some(LicenceDisposition::Supported) => {}
            Some(_) => {
                return Err(ThothError::LicenceNormalizationUnsupportedTarget(format!(
                    "target {target:?} is present in the bound MIG-01 manifest but its reviewed \
                     disposition is not SUPPORTED"
                )))
            }
            None => {
                return Err(ThothError::LicenceNormalizationUnsupportedTarget(format!(
                    "target {target:?} is absent from the bound MIG-01 manifest"
                )))
            }
        }
    }
    Ok(())
}

/// Reject any overlap between the executable deterministic source values and
/// the non-executable manual-resolution values.
fn verify_no_overlap(
    deterministic: &DeterministicManifest,
    manual: &ManualResolutionRegister,
) -> ThothResult<()> {
    let manual_values: HashSet<&str> = manual
        .values
        .iter()
        .map(|value| value.value.as_str())
        .collect();
    for rule in &deterministic.replacements {
        if manual_values.contains(rule.from.as_str()) {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "value {:?} appears both as a deterministic source and in the manual-resolution \
                 register; a manual value is never executable",
                rule.from
            )));
        }
    }
    Ok(())
}

/// Read, hash-verify, parse and cross-validate the three immutable inputs.
///
/// Every expected hash is compared against the SHA-256 of the exact raw file
/// bytes **before** the file is parsed. Artifact bytes are never normalized
/// before hashing.
fn load_and_verify_inputs(
    deterministic_manifest_path: &Path,
    expected_deterministic_manifest_sha256: &str,
    manual_register_path: &Path,
    expected_manual_register_sha256: &str,
    mig01_manifest_path: &Path,
    expected_mig01_manifest_sha256: &str,
) -> ThothResult<VerifiedInputs> {
    let deterministic_bytes = read_artifact("deterministic manifest", deterministic_manifest_path)?;
    let deterministic_sha256 = sha256_hex(&deterministic_bytes);
    if deterministic_sha256 != expected_deterministic_manifest_sha256.to_ascii_lowercase() {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the deterministic manifest does not match the expected reviewed hash".to_string(),
        ));
    }
    let manual_bytes = read_artifact("manual-resolution register", manual_register_path)?;
    let manual_sha256 = sha256_hex(&manual_bytes);
    if manual_sha256 != expected_manual_register_sha256.to_ascii_lowercase() {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the manual-resolution register does not match the expected reviewed hash".to_string(),
        ));
    }
    let mig01_bytes = read_artifact("MIG-01 manifest", mig01_manifest_path)?;
    let mig01_sha256 = sha256_hex(&mig01_bytes);
    if mig01_sha256 != expected_mig01_manifest_sha256.to_ascii_lowercase() {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the MIG-01 manifest does not match the expected reviewed hash".to_string(),
        ));
    }

    let deterministic = parse_deterministic_manifest(&deterministic_bytes)?;
    let manual = parse_manual_register(&manual_bytes)?;
    let mig01 = parse_mig01_manifest(&mig01_bytes).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "the bound MIG-01 manifest is invalid: {error}"
        ))
    })?;

    verify_no_overlap(&deterministic, &manual)?;
    verify_targets_supported(&deterministic, &mig01)?;

    Ok(VerifiedInputs {
        deterministic,
        deterministic_sha256,
        manual,
        manual_sha256,
        mig01,
        mig01_sha256,
    })
}

// ---------------------------------------------------------------------------
// Canonical plan (schema v1)
// ---------------------------------------------------------------------------

/// The deterministic, canonical, raw-byte-hashed normalization plan.
///
/// The field order below is the fixed schema-v1 declaration order and is the
/// serialized byte order; no unordered map iteration ever determines the
/// bytes. Serialized with compact `serde_json` (UTF-8, no BOM, no
/// insignificant whitespace, no trailing newline). Entries are ordered by
/// ascending `work_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizationPlan {
    pub schema_version: u32,
    pub deterministic_manifest_sha256: String,
    pub manual_resolution_sha256: String,
    pub mig01_manifest_sha256: String,
    pub entries: Vec<NormalizationPlanEntry>,
    pub expected: NormalizationPlanExpected,
}

/// One affected Work's reviewed plan entry. Every entry is implicitly
/// `PENDING` at dry-run construction: the manifest forbids `from == to`, so
/// every planned Work must change.
///
/// `reviewed_updated_at` is the typed repository [`Timestamp`], serialized
/// directly by its derived serde implementation (the chrono UTC `Z` RFC 3339
/// form, preserving stored sub-second precision). No `Display`,
/// `to_rfc3339()` or alternate formatter is ever used for canonical plan
/// emission, and apply compares typed timestamps, never JSON text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizationPlanEntry {
    pub work_id: Uuid,
    pub publisher_id: Uuid,
    pub reviewed_updated_at: Timestamp,
    pub from: String,
    pub to: String,
}

/// The aggregate reviewed expectations. Field order is fixed schema-v1 order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizationPlanExpected {
    pub works_considered: i64,
    pub works_changing: i64,
    pub history_rows: i64,
    /// Affected-work counts per deterministic source value, ascending by
    /// value; all 24 reviewed source values are present, zeros included.
    pub source_value_counts: Vec<ValueCount>,
    /// Affected-work counts per canonical target, ascending by value; all 9
    /// reviewed targets are present, zeros included.
    pub target_value_counts: Vec<ValueCount>,
    pub manual_unresolved_value_count: i64,
    pub manual_unresolved_work_count: i64,
}

/// A licence value with the number of Works currently carrying it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValueCount {
    pub value: String,
    pub works: i64,
}

/// Serialize a plan to its exact canonical bytes.
pub fn canonical_plan_bytes(plan: &NormalizationPlan) -> ThothResult<Vec<u8>> {
    // Compact `serde_json` on structs emits fields in declaration order with
    // no insignificant whitespace and no trailing newline: exactly the
    // canonical contract. No map is ever serialized, so ordering is fully
    // determined by the struct declarations and the sorted entry vector.
    serde_json::to_vec(plan).map_err(Into::into)
}

/// The exact audit actor derived from a reviewed plan's SHA-256.
pub fn audit_actor(plan_sha256: &str) -> String {
    format!("{AUDIT_ACTOR_PREFIX}{plan_sha256}")
}

/// Parse canonical plan bytes and require them to be exactly canonical.
///
/// A BOM is rejected outright. A semantically valid but noncanonical encoding
/// (extra whitespace, reordered keys, a differing timestamp representation)
/// parses but fails the byte-for-byte re-serialization check, so it can never
/// reach a write. Entries must be strictly ascending by `work_id`.
pub fn parse_canonical_plan(bytes: &[u8]) -> ThothResult<NormalizationPlan> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ThothError::LicenceNormalizationNoncanonicalPlan);
    }
    let plan: NormalizationPlan = serde_json::from_slice(bytes).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "the plan is not valid schema-v{PLAN_SCHEMA_VERSION} JSON: {error}"
        ))
    })?;
    if plan.schema_version != PLAN_SCHEMA_VERSION {
        return Err(ThothError::LicenceNormalizationInvalidInput(format!(
            "unsupported plan schema version {} (expected {PLAN_SCHEMA_VERSION})",
            plan.schema_version
        )));
    }
    for pair in plan.entries.windows(2) {
        if pair[0].work_id >= pair[1].work_id {
            return Err(ThothError::LicenceNormalizationNoncanonicalPlan);
        }
    }
    let reserialized = canonical_plan_bytes(&plan)?;
    if reserialized != bytes {
        return Err(ThothError::LicenceNormalizationNoncanonicalPlan);
    }
    Ok(plan)
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// One current Work carrying a deterministic source value.
struct AffectedWork {
    work_id: Uuid,
    publisher_id: Uuid,
    license: String,
    updated_at: Timestamp,
}

/// Every current Work carrying any deterministic source value, with its owning
/// publisher, exact stored licence and full-precision typed update token.
fn affected_works(db: &PgPool, sources: &[&str]) -> ThothResult<Vec<AffectedWork>> {
    let mut connection = db.get()?;
    let rows: Vec<(Uuid, Option<String>, Timestamp, Uuid)> = work::table
        .inner_join(imprint::table)
        .filter(work::license.eq_any(sources))
        .select((
            work::work_id,
            work::license,
            work::updated_at,
            imprint::publisher_id,
        ))
        .load(&mut connection)?;
    let mut affected: Vec<AffectedWork> = rows
        .into_iter()
        .map(
            |(work_id, license, updated_at, publisher_id)| AffectedWork {
                work_id,
                publisher_id,
                license: license.unwrap_or_default(),
                updated_at,
            },
        )
        .collect();
    affected.sort_by_key(|entry| entry.work_id);
    Ok(affected)
}

/// Current Work counts for each given exact licence value, restricted to the
/// bound MIG-01 canonical publisher scope, ascending by value; every supplied
/// value is present in the result, zeros included. Report-only.
///
/// The MIG-01 publisher scope is the reporting boundary because it is the
/// scope V3 binds every LIC-NORM operation and the downstream MIG-01
/// package/platform licence gate to; a Work outside that scope is a scope
/// mismatch for deterministic sources and out of reporting scope for
/// residual counting.
fn scoped_value_counts(
    db: &PgPool,
    values: &[&str],
    scope: &HashSet<Uuid>,
) -> ThothResult<Vec<ValueCount>> {
    let mut connection = db.get()?;
    let rows: Vec<(Option<String>, Uuid)> = work::table
        .inner_join(imprint::table)
        .filter(work::license.eq_any(values))
        .select((work::license, imprint::publisher_id))
        .load(&mut connection)?;
    let mut tally: BTreeMap<&str, i64> = values.iter().map(|value| (*value, 0)).collect();
    for (license, publisher_id) in rows {
        if !scope.contains(&publisher_id) {
            continue;
        }
        if let Some(count) = license.as_deref().and_then(|value| tally.get_mut(value)) {
            *count += 1;
        }
    }
    Ok(tally
        .into_iter()
        .map(|(value, works)| ValueCount {
            value: value.to_string(),
            works,
        })
        .collect())
}

/// Current Work counts for each manual-resolution value within the bound
/// MIG-01 publisher scope, ascending by value; all reviewed values are
/// present, zeros included. Report-only: manual values are never executable.
fn manual_value_counts(
    db: &PgPool,
    register: &ManualResolutionRegister,
    scope: &HashSet<Uuid>,
) -> ThothResult<Vec<ValueCount>> {
    let values: Vec<&str> = register
        .values
        .iter()
        .map(|value| value.value.as_str())
        .collect();
    scoped_value_counts(db, &values, scope)
}

/// The number of distinct manual values currently carried by at least one
/// in-scope Work, derived from the exact per-value count vector reported.
fn manual_distinct_value_count(counts: &[ValueCount]) -> i64 {
    counts.iter().filter(|entry| entry.works > 0).count() as i64
}

/// The number of in-scope Works currently carrying any manual value, derived
/// from the exact per-value count vector reported.
fn manual_work_count(counts: &[ValueCount]) -> i64 {
    counts.iter().map(|entry| entry.works).sum()
}

// ---------------------------------------------------------------------------
// Post-write reconciliation
// ---------------------------------------------------------------------------

/// The post-write reconciliation snapshot: current catalogue evidence taken
/// after the last write transaction has committed and **before** any
/// successful APPLY outcome or report exists.
///
/// This is actual current state, never a planned delta: a snapshot only
/// exists when the post-write deterministic-source query came back empty.
struct PostWriteReconciliation {
    /// The count of current Works still carrying any deterministic source
    /// value, from the post-write query. Provably `0` on every path that
    /// constructs this value; a non-empty result fails closed instead.
    deterministic_source_works_remaining: i64,
    /// Actual current in-scope Work counts for every reviewed canonical
    /// target — catalogue state after the write loop, distinct from the
    /// reviewed planned conversion tallies.
    resulting_target_value_counts: Vec<ValueCount>,
    /// Current in-scope manual-resolution residuals, from the same snapshot.
    manual_unresolved_values: Vec<ValueCount>,
}

/// Take the post-write reconciliation snapshot, failing closed on any
/// residual deterministic source value.
///
/// There is a real concurrency interval between the pre-write membership
/// check and the completion of all per-Work write transactions: another
/// transaction can introduce a Work carrying a deterministic source value
/// while this apply is running. Successful reconciliation therefore requires
/// re-querying every current Work carrying any of the 24 source values after
/// the write loop, with the same bound MIG-01 publisher-scope semantics the
/// reviewed operation uses. On any residual: no successful report is emitted,
/// no cross-Work rollback is attempted, previously committed Works remain
/// committed, and recovery is deterministic same-plan resume where valid,
/// otherwise separately reviewed forward repair.
fn post_write_reconciliation(
    db: &PgPool,
    inputs: &VerifiedInputs,
) -> ThothResult<PostWriteReconciliation> {
    let rules = inputs.rules();
    let sources: Vec<&str> = rules.keys().copied().collect();
    let scope = inputs.publisher_scope();

    let residual = affected_works(db, &sources)?;
    if let Some(out_of_scope) = residual
        .iter()
        .find(|current| !scope.contains(&current.publisher_id))
    {
        return Err(ThothError::LicenceNormalizationScopeMismatch(format!(
            "after the write loop, work {} carries a deterministic source value under \
             publisher {}, which is absent from the bound MIG-01 manifest; previously \
             committed Works remain committed and remediation requires an approved MIG-01 \
             manifest/programme amendment with a newly frozen hash and fresh review",
            out_of_scope.work_id, out_of_scope.publisher_id
        )));
    }
    if !residual.is_empty() {
        return Err(ThothError::LicenceNormalizationDrift(format!(
            "{} work(s) still carry a deterministic source value after the write loop \
             (first: {}); no successful reconciliation is claimed and no cross-Work \
             rollback is attempted — previously committed Works remain committed; recovery \
             is deterministic same-plan resume where valid, otherwise separately reviewed \
             forward repair",
            residual.len(),
            residual[0].work_id
        )));
    }

    let targets: Vec<&str> = inputs.targets().into_iter().collect();
    let resulting_target_value_counts = scoped_value_counts(db, &targets, &scope)?;
    let manual_unresolved_values = manual_value_counts(db, &inputs.manual, &scope)?;

    Ok(PostWriteReconciliation {
        deterministic_source_works_remaining: residual.len() as i64,
        resulting_target_value_counts,
        manual_unresolved_values,
    })
}

// ---------------------------------------------------------------------------
// Plan construction
// ---------------------------------------------------------------------------

/// Report-only detail collected during plan construction.
struct ReportParts {
    manual_unresolved_values: Vec<ValueCount>,
    affected_publisher_ids: Vec<Uuid>,
}

/// Build the canonical plan and report detail from the verified inputs.
///
/// Every current Work carrying any deterministic source value is included. An
/// affected Work whose publisher is absent from the bound MIG-01 manifest is a
/// blocking scope mismatch, never an inferred inclusion or a silent omission:
/// remediation is an approved MIG-01 manifest/programme amendment with a newly
/// frozen hash and fresh independent review.
fn build_plan(
    db: &PgPool,
    inputs: &VerifiedInputs,
) -> ThothResult<(NormalizationPlan, ReportParts)> {
    let rules = inputs.rules();
    let sources: Vec<&str> = rules.keys().copied().collect();
    let scope = inputs.publisher_scope();

    let affected = affected_works(db, &sources)?;
    let mut entries: Vec<NormalizationPlanEntry> = Vec::with_capacity(affected.len());
    let mut source_tally: BTreeMap<&str, i64> = rules.keys().map(|from| (*from, 0)).collect();
    let mut target_tally: BTreeMap<&str, i64> = inputs
        .targets()
        .into_iter()
        .map(|target| (target, 0))
        .collect();
    let mut affected_publishers: BTreeSet<Uuid> = BTreeSet::new();

    for work_row in &affected {
        let Some(target) = rules.get(work_row.license.as_str()) else {
            // Unreachable by construction: the query filters on the source set.
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "work {} matched the source query but no deterministic rule",
                work_row.work_id
            )));
        };
        if !scope.contains(&work_row.publisher_id) {
            return Err(ThothError::LicenceNormalizationScopeMismatch(format!(
                "work {} belongs to publisher {}, which is absent from the bound MIG-01 \
                 manifest; remediation requires an approved MIG-01 manifest/programme \
                 amendment with a newly frozen hash and fresh review",
                work_row.work_id, work_row.publisher_id
            )));
        }
        *source_tally
            .get_mut(work_row.license.as_str())
            .expect("every source value is tallied") += 1;
        *target_tally
            .get_mut(*target)
            .expect("every target is tallied") += 1;
        affected_publishers.insert(work_row.publisher_id);
        entries.push(NormalizationPlanEntry {
            work_id: work_row.work_id,
            publisher_id: work_row.publisher_id,
            reviewed_updated_at: work_row.updated_at,
            from: work_row.license.clone(),
            to: (*target).to_string(),
        });
    }

    let manual_unresolved_values = manual_value_counts(db, &inputs.manual, &scope)?;
    let manual_unresolved_value_count = manual_distinct_value_count(&manual_unresolved_values);
    let manual_unresolved_work_count = manual_work_count(&manual_unresolved_values);

    let works_considered = entries.len() as i64;
    let expected = NormalizationPlanExpected {
        works_considered,
        works_changing: works_considered,
        history_rows: works_considered,
        source_value_counts: source_tally
            .into_iter()
            .map(|(value, works)| ValueCount {
                value: value.to_string(),
                works,
            })
            .collect(),
        target_value_counts: target_tally
            .into_iter()
            .map(|(value, works)| ValueCount {
                value: value.to_string(),
                works,
            })
            .collect(),
        manual_unresolved_value_count,
        manual_unresolved_work_count,
    };

    let plan = NormalizationPlan {
        schema_version: PLAN_SCHEMA_VERSION,
        deterministic_manifest_sha256: inputs.deterministic_sha256.clone(),
        manual_resolution_sha256: inputs.manual_sha256.clone(),
        mig01_manifest_sha256: inputs.mig01_sha256.clone(),
        entries,
        expected,
    };

    Ok((
        plan,
        ReportParts {
            manual_unresolved_values,
            affected_publisher_ids: affected_publishers.into_iter().collect(),
        },
    ))
}

// ---------------------------------------------------------------------------
// Runtime classification
// ---------------------------------------------------------------------------

/// The apply-time classification of a single reviewed plan entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryClassification {
    /// Current Work exists, its licence exactly equals the reviewed `from`,
    /// and its typed `updated_at` exactly equals `reviewed_updated_at`.
    Pending,
    /// The complete exact history proof succeeds and the current licence
    /// exactly equals the reviewed `to`.
    AlreadyAppliedByThisPlan,
    /// Every other state: deletion, a different current licence, a token
    /// mismatch (including sub-second-only differences), or missing,
    /// unusable or ambiguous history proof. STOP before any new write.
    Drift(String),
}

/// Classify one reviewed plan entry against current typed state and the
/// actor-filtered `work_history` evidence.
pub fn classify_entry(
    db: &PgPool,
    entry: &NormalizationPlanEntry,
    actor: &str,
) -> ThothResult<EntryClassification> {
    let mut connection = db.get()?;
    classify_entry_on(&mut connection, entry, actor)
}

fn classify_entry_on(
    connection: &mut PgConnection,
    entry: &NormalizationPlanEntry,
    actor: &str,
) -> ThothResult<EntryClassification> {
    let current: Option<Work> = work::table
        .find(entry.work_id)
        .first::<Work>(connection)
        .optional()?;
    let Some(current) = current else {
        return Ok(EntryClassification::Drift(format!(
            "planned work {} no longer exists",
            entry.work_id
        )));
    };
    if current.license.as_deref() == Some(entry.from.as_str()) {
        if current.updated_at == entry.reviewed_updated_at {
            return Ok(EntryClassification::Pending);
        }
        return Ok(EntryClassification::Drift(format!(
            "work {} carries the reviewed source licence but its typed updated_at differs \
             from the reviewed token",
            entry.work_id
        )));
    }
    if current.license.as_deref() == Some(entry.to.as_str()) {
        return already_applied_proof(connection, entry, actor);
    }
    Ok(EntryClassification::Drift(format!(
        "work {} carries neither the reviewed source licence nor the reviewed target",
        entry.work_id
    )))
}

/// The exact `ALREADY_APPLIED_BY_THIS_PLAN` proof.
///
/// Candidate history rows are filtered by both `work_id` and the exact
/// plan-derived actor **before** any payload is examined, so legacy rows
/// written by other actors are never parsed at all. The historic pre-state
/// `license` and `updatedAt` are then extracted tolerantly from the inner
/// serialized JSON — historical payloads are never required to deserialize
/// into today's complete [`Work`] struct. A current licence equal to the
/// target without this complete proof is `DRIFT`, never already-applied.
fn already_applied_proof(
    connection: &mut PgConnection,
    entry: &NormalizationPlanEntry,
    actor: &str,
) -> ThothResult<EntryClassification> {
    let rows: Vec<serde_json::Value> = work_history::table
        .filter(work_history::work_id.eq(entry.work_id))
        .filter(work_history::user_id.eq(actor))
        .select(work_history::data)
        .load(connection)?;
    if rows.is_empty() {
        return Ok(EntryClassification::Drift(format!(
            "work {} carries the target licence but no history row with the exact plan actor \
             exists",
            entry.work_id
        )));
    }
    let mut matching = 0usize;
    for row in &rows {
        let Some((license, updated_at)) = history_prestate(row) else {
            return Ok(EntryClassification::Drift(format!(
                "work {} has a history row with the exact plan actor whose required pre-state \
                 licence/updatedAt evidence is unusable",
                entry.work_id
            )));
        };
        if license.as_deref() == Some(entry.from.as_str())
            && updated_at == entry.reviewed_updated_at
        {
            matching += 1;
        }
    }
    match matching {
        1 => Ok(EntryClassification::AlreadyAppliedByThisPlan),
        0 => Ok(EntryClassification::Drift(format!(
            "work {} carries the target licence but no plan-actor history row proves the \
             reviewed pre-normalization licence and timestamp",
            entry.work_id
        ))),
        _ => Ok(EntryClassification::Drift(format!(
            "work {} has more than one plan-actor history row matching the reviewed \
             pre-state; the evidence is ambiguous",
            entry.work_id
        ))),
    }
}

/// Tolerantly extract the historic pre-state `license` and `updatedAt` from a
/// `work_history.data` payload.
///
/// The current writer stores a JSON string containing serialized JSON; a
/// payload stored as a JSON object directly is tolerated too. Only the two
/// required scalars are read; unrelated legacy fields are never interpreted
/// and no full `Work` deserialization is attempted.
fn history_prestate(data: &serde_json::Value) -> Option<(Option<String>, Timestamp)> {
    let inner: serde_json::Value = match data {
        serde_json::Value::String(text) => serde_json::from_str(text).ok()?,
        serde_json::Value::Object(_) => data.clone(),
        _ => return None,
    };
    let license = match inner.get("license")? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Null => None,
        _ => return None,
    };
    let updated_at = Timestamp::parse_from_rfc3339(inner.get("updatedAt")?.as_str()?).ok()?;
    Some((license, updated_at))
}

// ---------------------------------------------------------------------------
// Bounded reconciliation report
// ---------------------------------------------------------------------------

/// Whether a report describes a dry run or an apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportMode {
    DryRun,
    Apply,
}

/// The bounded, human-readable reconciliation report. It is not the hashed
/// machine plan; its identity, where bound (the production reviewed dry-run
/// report), is the SHA-256 of its exact raw bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizationReport {
    pub mode: ReportMode,
    pub deterministic_manifest_sha256: String,
    pub manual_resolution_sha256: String,
    pub mig01_manifest_sha256: String,
    pub plan_sha256: String,
    pub audit_actor: String,
    pub works_considered: i64,
    pub works_changing: i64,
    pub expected_history_rows: i64,
    /// Reviewed/planned affected-Work counts per deterministic source value,
    /// from the reviewed plan.
    pub source_value_counts: Vec<ValueCount>,
    /// Reviewed/planned conversion tallies per canonical target, from the
    /// reviewed plan. These are deltas the plan intends to produce, **not**
    /// resulting catalogue state; see `resulting_target_value_counts`.
    pub target_value_counts: Vec<ValueCount>,
    /// Actual current in-scope Work counts for every reviewed canonical
    /// target, queried from catalogue state after the write loop. Present
    /// only on a successful APPLY report (post-write reconciliation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_target_value_counts: Option<Vec<ValueCount>>,
    /// The count of current Works still carrying any deterministic source
    /// value, from the post-write reconciliation query. Present only on a
    /// successful APPLY report, where it is provably `0`: a non-empty
    /// post-write residual fails closed and no successful report exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic_source_works_remaining: Option<i64>,
    /// Current in-scope Work counts for every manual-resolution value: the
    /// dry-run snapshot for a DRY_RUN report, the post-write reconciliation
    /// snapshot for an APPLY report. Report-only: these values are never
    /// executable and are never modified by this task.
    pub manual_unresolved_values: Vec<ValueCount>,
    /// Distinct manual values currently carried by at least one in-scope
    /// Work. Always derived from the exact `manual_unresolved_values` vector
    /// in this same report, never from a different snapshot.
    pub manual_unresolved_value_count: i64,
    /// In-scope Works currently carrying any manual value. Always derived
    /// from the exact `manual_unresolved_values` vector in this same report.
    pub manual_unresolved_work_count: i64,
    pub affected_publisher_ids: Vec<Uuid>,
    /// The expected operational export/specification cache effect of the
    /// licence writes. Recorded only; nothing is triggered.
    pub expected_export_cache_effect: ExpectedExportCacheEffect,
    /// Database writes performed by this invocation (always 0 for a dry run).
    pub writes_performed: i64,
    /// `work_history` rows inserted by this invocation (always 0 for a dry run).
    pub history_rows_written: i64,
    /// Present only for an apply report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<NormalizationAppliedSummary>,
}

/// The expected read-time export-cache regeneration load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedExportCacheEffect {
    pub affected_publishers: i64,
    pub affected_works: i64,
    pub note: String,
}

/// The apply-only outcome summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizationAppliedSummary {
    pub written: i64,
    pub already_applied: i64,
}

#[allow(clippy::too_many_arguments)]
fn assemble_report(
    mode: ReportMode,
    inputs: &VerifiedInputs,
    plan: &NormalizationPlan,
    plan_sha256: &str,
    parts: &ReportParts,
    reconciliation: Option<&PostWriteReconciliation>,
    writes_performed: i64,
    history_rows_written: i64,
    applied: Option<NormalizationAppliedSummary>,
) -> NormalizationReport {
    // The manual aggregates are derived from the exact per-value vector this
    // report embeds — one snapshot, internally consistent — never from the
    // dry-run `plan.expected` aggregates, which describe an earlier snapshot.
    let manual_unresolved_values = parts.manual_unresolved_values.clone();
    let manual_unresolved_value_count = manual_distinct_value_count(&manual_unresolved_values);
    let manual_unresolved_work_count = manual_work_count(&manual_unresolved_values);
    NormalizationReport {
        mode,
        deterministic_manifest_sha256: inputs.deterministic_sha256.clone(),
        manual_resolution_sha256: inputs.manual_sha256.clone(),
        mig01_manifest_sha256: inputs.mig01_sha256.clone(),
        plan_sha256: plan_sha256.to_string(),
        audit_actor: audit_actor(plan_sha256),
        works_considered: plan.expected.works_considered,
        works_changing: plan.expected.works_changing,
        expected_history_rows: plan.expected.history_rows,
        source_value_counts: plan.expected.source_value_counts.clone(),
        target_value_counts: plan.expected.target_value_counts.clone(),
        resulting_target_value_counts: reconciliation
            .map(|snapshot| snapshot.resulting_target_value_counts.clone()),
        deterministic_source_works_remaining: reconciliation
            .map(|snapshot| snapshot.deterministic_source_works_remaining),
        manual_unresolved_values,
        manual_unresolved_value_count,
        manual_unresolved_work_count,
        affected_publisher_ids: parts.affected_publisher_ids.clone(),
        expected_export_cache_effect: ExpectedExportCacheEffect {
            affected_publishers: parts.affected_publisher_ids.len() as i64,
            affected_works: plan.expected.works_changing,
            note: EXPORT_CACHE_EFFECT_NOTE.to_string(),
        },
        writes_performed,
        history_rows_written,
        applied,
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Inputs for a dry run. The caller (the thin CLI) supplies paths and expected
/// reviewed hashes; this module owns all reading, hashing, parsing, domain
/// mapping and serialization.
pub struct DryRunRequest<'a> {
    pub deterministic_manifest_path: &'a Path,
    pub expected_deterministic_manifest_sha256: &'a str,
    pub manual_register_path: &'a Path,
    pub expected_manual_register_sha256: &'a str,
    pub mig01_manifest_path: &'a Path,
    pub expected_mig01_manifest_sha256: &'a str,
    pub plan_out_path: &'a Path,
    pub report_out_path: &'a Path,
}

/// The result of a dry run.
#[derive(Debug, Clone)]
pub struct DryRunOutcome {
    pub deterministic_manifest_sha256: String,
    pub manual_register_sha256: String,
    pub mig01_manifest_sha256: String,
    pub plan_sha256: String,
    pub plan: NormalizationPlan,
    pub report: NormalizationReport,
}

/// Produce the deterministic canonical plan and reconciliation report; perform
/// no database write.
pub fn dry_run(db: &PgPool, request: &DryRunRequest<'_>) -> ThothResult<DryRunOutcome> {
    assert_distinct_artifacts(&[
        (
            "deterministic manifest",
            request.deterministic_manifest_path,
        ),
        ("manual-resolution register", request.manual_register_path),
        ("MIG-01 manifest", request.mig01_manifest_path),
        ("plan output", request.plan_out_path),
        ("report output", request.report_out_path),
    ])?;
    let inputs = load_and_verify_inputs(
        request.deterministic_manifest_path,
        request.expected_deterministic_manifest_sha256,
        request.manual_register_path,
        request.expected_manual_register_sha256,
        request.mig01_manifest_path,
        request.expected_mig01_manifest_sha256,
    )?;

    let (plan, parts) = build_plan(db, &inputs)?;
    let plan_bytes = canonical_plan_bytes(&plan)?;
    let plan_sha256 = sha256_hex(&plan_bytes);
    let report = assemble_report(
        ReportMode::DryRun,
        &inputs,
        &plan,
        &plan_sha256,
        &parts,
        None,
        0,
        0,
        None,
    );

    write_file(request.plan_out_path, &plan_bytes)?;
    write_report(request.report_out_path, &report)?;

    Ok(DryRunOutcome {
        deterministic_manifest_sha256: inputs.deterministic_sha256,
        manual_register_sha256: inputs.manual_sha256,
        mig01_manifest_sha256: inputs.mig01_sha256,
        plan_sha256,
        plan,
        report,
    })
}

/// The apply execution scope: a required, explicit, mutually exclusive choice,
/// never an omitted boolean, so unsafe production combinations are
/// structurally unrepresentable.
///
/// `Production` **cannot** be constructed without the exact independently
/// reviewed dry-run report identity: there is no "production without reviewed
/// evidence" value, and disposable mode carries no field that could smuggle
/// one in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyExecutionMode<'a> {
    /// Disposable/local execution against a disposable database. Not usable as
    /// a production shorthand: it carries no reviewed-report evidence at all.
    Disposable,
    /// Production execution. Requires by construction the exact independently
    /// reviewed `DRY_RUN` reconciliation report and its expected raw-byte
    /// SHA-256, bound at the production authorization gate.
    Production {
        reviewed_report_path: &'a Path,
        expected_reviewed_report_sha256: &'a str,
    },
}

/// Inputs for an apply. The three immutable inputs and the reviewed plan are
/// all identified by expected raw-byte hashes verified before parsing.
pub struct ApplyRequest<'a> {
    pub deterministic_manifest_path: &'a Path,
    pub expected_deterministic_manifest_sha256: &'a str,
    pub manual_register_path: &'a Path,
    pub expected_manual_register_sha256: &'a str,
    pub mig01_manifest_path: &'a Path,
    pub expected_mig01_manifest_sha256: &'a str,
    pub plan_path: &'a Path,
    pub expected_plan_sha256: &'a str,
    pub report_out_path: &'a Path,
    pub mode: ApplyExecutionMode<'a>,
}

/// The result of an apply.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub plan_sha256: String,
    pub written: usize,
    pub already_applied: usize,
    pub report: NormalizationReport,
}

/// Apply exactly the reviewed plan.
///
/// The whole plan is classified before the first database write; any `DRIFT`
/// stops the invocation with no new write. Writes are processed in ascending
/// `work_id`, one bounded transaction per Work, each rechecking the reviewed
/// state under its row lock. After the last write and before any successful
/// outcome, post-write reconciliation re-queries the deterministic-source
/// residual (which must be empty) and snapshots the actual resulting target
/// counts and current manual residuals for the report. Previously committed
/// Works remain committed on any failure; recovery is deterministic resume or
/// separately authorized forward repair, never cross-work rollback.
pub fn apply(db: &PgPool, request: &ApplyRequest<'_>) -> ThothResult<ApplyOutcome> {
    let mut artifacts: Vec<(&str, &Path)> = vec![
        (
            "deterministic manifest",
            request.deterministic_manifest_path,
        ),
        ("manual-resolution register", request.manual_register_path),
        ("MIG-01 manifest", request.mig01_manifest_path),
        ("reviewed plan", request.plan_path),
        ("report output", request.report_out_path),
    ];
    if let ApplyExecutionMode::Production {
        reviewed_report_path,
        ..
    } = request.mode
    {
        artifacts.push(("reviewed dry-run report", reviewed_report_path));
    }
    assert_distinct_artifacts(&artifacts)?;

    // 1. Verify and parse the three immutable inputs, revalidating every
    //    mechanical invariant, cc-license target parse and MIG-01 SUPPORTED
    //    binding exactly as dry run does.
    let inputs = load_and_verify_inputs(
        request.deterministic_manifest_path,
        request.expected_deterministic_manifest_sha256,
        request.manual_register_path,
        request.expected_manual_register_sha256,
        request.mig01_manifest_path,
        request.expected_mig01_manifest_sha256,
    )?;

    // 2. Hash the exact raw plan bytes against the expected reviewed hash
    //    before parsing, then require canonical bytes.
    let plan_bytes = read_artifact("reviewed plan", request.plan_path)?;
    let plan_sha256 = sha256_hex(&plan_bytes);
    if plan_sha256 != request.expected_plan_sha256.to_ascii_lowercase() {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed plan does not match the expected reviewed hash".to_string(),
        ));
    }
    let plan = parse_canonical_plan(&plan_bytes)?;

    // 3. The plan must bind exactly the verified input identities.
    if plan.deterministic_manifest_sha256 != inputs.deterministic_sha256 {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed plan records a different deterministic-manifest hash".to_string(),
        ));
    }
    if plan.manual_resolution_sha256 != inputs.manual_sha256 {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed plan records a different manual-resolution hash".to_string(),
        ));
    }
    if plan.mig01_manifest_sha256 != inputs.mig01_sha256 {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed plan records a different MIG-01 manifest hash".to_string(),
        ));
    }

    // 4. Every plan entry must be one of the 24 reviewed deterministic rules.
    //    A plan can never touch a manual-resolution value or invent a mapping.
    let rules = inputs.rules();
    for entry in &plan.entries {
        if rules.get(entry.from.as_str()).copied() != Some(entry.to.as_str()) {
            return Err(ThothError::LicenceNormalizationInvalidInput(format!(
                "plan entry for work {} is not one of the reviewed deterministic rules",
                entry.work_id
            )));
        }
    }

    // 5. Production mode binds the exact independently reviewed dry-run report.
    if let ApplyExecutionMode::Production {
        reviewed_report_path,
        expected_reviewed_report_sha256,
    } = request.mode
    {
        verify_reviewed_report(
            reviewed_report_path,
            expected_reviewed_report_sha256,
            &inputs,
            &plan_sha256,
        )?;
    }

    // 6. Derive the exact audit actor from the reviewed plan hash.
    let actor = audit_actor(&plan_sha256);

    // 7. Classify every entry before the first write. Any DRIFT stops the run.
    let mut classes: Vec<EntryClassification> = Vec::with_capacity(plan.entries.len());
    for entry in &plan.entries {
        let class = classify_entry(db, entry, &actor)?;
        if let EntryClassification::Drift(reason) = &class {
            return Err(ThothError::LicenceNormalizationDrift(format!(
                "{reason}; recovery requires a fresh reviewed plan"
            )));
        }
        classes.push(class);
    }

    // 8. Recompute the exact current source-value membership: a Work carrying
    //    a deterministic source value that is not in the reviewed plan stops
    //    the run before any write.
    let sources: Vec<&str> = rules.keys().copied().collect();
    let planned: HashSet<Uuid> = plan.entries.iter().map(|entry| entry.work_id).collect();
    let unplanned: Vec<Uuid> = affected_works(db, &sources)?
        .into_iter()
        .filter(|current| !planned.contains(&current.work_id))
        .map(|current| current.work_id)
        .collect();
    if !unplanned.is_empty() {
        return Err(ThothError::LicenceNormalizationUnplannedWork(format!(
            "{} work(s) currently carry a deterministic source value but are absent from the \
             reviewed plan (first: {}); recovery requires a fresh dry run and review",
            unplanned.len(),
            unplanned[0]
        )));
    }

    // 9. Write only PENDING entries, in ascending work_id (the canonical plan
    //    order), one bounded transaction per Work.
    let mut written = 0usize;
    let mut already_applied = 0usize;
    for (entry, class) in plan.entries.iter().zip(&classes) {
        match class {
            EntryClassification::AlreadyAppliedByThisPlan => already_applied += 1,
            EntryClassification::Drift(_) => unreachable!("drift stopped the run above"),
            EntryClassification::Pending => {
                let mut connection = db.get()?;
                apply_entry(&mut connection, entry, &actor)?;
                written += 1;
            }
        }
    }

    // 10. Post-write reconciliation, before any successful outcome exists:
    //     re-query every current Work carrying a deterministic source value
    //     (fail closed on any residual, with committed Works preserved), and
    //     take one consistent current snapshot of the resulting target counts
    //     and the manual residuals. The manual detail and aggregates in the
    //     report all derive from this one snapshot.
    let reconciliation = post_write_reconciliation(db, &inputs)?;
    let parts = ReportParts {
        manual_unresolved_values: reconciliation.manual_unresolved_values.clone(),
        affected_publisher_ids: plan
            .entries
            .iter()
            .map(|entry| entry.publisher_id)
            .collect::<BTreeSet<Uuid>>()
            .into_iter()
            .collect(),
    };

    let applied = NormalizationAppliedSummary {
        written: written as i64,
        already_applied: already_applied as i64,
    };
    let report = assemble_report(
        ReportMode::Apply,
        &inputs,
        &plan,
        &plan_sha256,
        &parts,
        Some(&reconciliation),
        written as i64,
        written as i64,
        Some(applied),
    );
    write_report(request.report_out_path, &report)?;

    Ok(ApplyOutcome {
        plan_sha256,
        written,
        already_applied,
        report,
    })
}

/// Apply one reviewed entry inside one bounded transaction.
///
/// The transaction: locks the exact Work row (`SELECT ... FOR UPDATE`),
/// rechecks the reviewed source licence and typed update token under the lock,
/// inserts exactly one normal `work_history` row from the complete pre-update
/// Work with the plan-derived actor, issues a single-column
/// `diesel::update(work.find(id)).set(work::license.eq(target))`, and re-reads
/// the Work to verify the exact target licence. Any failure rolls back the
/// history row and the licence update together. Timestamps are never set
/// explicitly: the existing Work triggers own `updated_at` /
/// `updated_at_with_relations`.
fn apply_entry(
    connection: &mut PgConnection,
    entry: &NormalizationPlanEntry,
    actor: &str,
) -> ThothResult<()> {
    connection.transaction(|connection| {
        let current: Option<Work> = work::table
            .find(entry.work_id)
            .for_update()
            .first::<Work>(connection)
            .optional()?;
        let Some(current) = current else {
            return Err(ThothError::LicenceNormalizationDrift(format!(
                "planned work {} no longer exists; the run stops before further writes",
                entry.work_id
            )));
        };
        if current.license.as_deref() != Some(entry.from.as_str())
            || current.updated_at != entry.reviewed_updated_at
        {
            return Err(ThothError::LicenceNormalizationDrift(format!(
                "work {} drifted from the reviewed source licence/token under the row lock; \
                 the run stops before further writes",
                entry.work_id
            )));
        }
        current.new_history_entry(actor).insert(connection)?;
        let updated_rows = diesel::update(work::table.find(entry.work_id))
            .set(work::license.eq(entry.to.as_str()))
            .execute(connection)?;
        if updated_rows != 1 {
            return Err(ThothError::LicenceNormalizationWriteVerification(format!(
                "the licence update for work {} affected {updated_rows} rows instead of 1",
                entry.work_id
            )));
        }
        let reread: Work = work::table.find(entry.work_id).first::<Work>(connection)?;
        if reread.license.as_deref() != Some(entry.to.as_str()) {
            return Err(ThothError::LicenceNormalizationWriteVerification(format!(
                "work {} does not carry the exact reviewed target licence after the update",
                entry.work_id
            )));
        }
        Ok(())
    })
}

/// Bind the exact independently reviewed dry-run report to a production apply.
///
/// Hash first, parse second: the report's identity is the SHA-256 of its exact
/// raw bytes. The parsed report must be a `DRY_RUN` report whose recorded
/// input and plan identities all match this exact apply.
fn verify_reviewed_report(
    reviewed_report_path: &Path,
    expected_reviewed_report_sha256: &str,
    inputs: &VerifiedInputs,
    plan_sha256: &str,
) -> ThothResult<()> {
    let report_bytes = read_artifact("reviewed dry-run report", reviewed_report_path)?;
    let report_sha256 = sha256_hex(&report_bytes);
    if report_sha256 != expected_reviewed_report_sha256.to_ascii_lowercase() {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed dry-run report does not match the expected reviewed hash".to_string(),
        ));
    }
    let report: NormalizationReport = serde_json::from_slice(&report_bytes).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "the reviewed report is not a parseable licence-normalization report: {error}"
        ))
    })?;
    if report.mode != ReportMode::DryRun {
        return Err(ThothError::LicenceNormalizationInvalidInput(
            "the reviewed report is not a DRY_RUN report".to_string(),
        ));
    }
    if report.deterministic_manifest_sha256 != inputs.deterministic_sha256
        || report.manual_resolution_sha256 != inputs.manual_sha256
        || report.mig01_manifest_sha256 != inputs.mig01_sha256
    {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed report records different immutable input hashes".to_string(),
        ));
    }
    if report.plan_sha256 != plan_sha256 {
        return Err(ThothError::LicenceNormalizationHashMismatch(
            "the reviewed report records a different reviewed plan hash".to_string(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// File I/O
// ---------------------------------------------------------------------------

/// Reject any two artifact paths that resolve to the same filesystem location,
/// before any read or write, so an interrupted run can never destroy a
/// reviewed input it needs for deterministic recovery. An existing path
/// canonicalizes directly; a not-yet-existing output resolves through its
/// existing parent directory combined with its file name.
fn assert_distinct_artifacts(artifacts: &[(&str, &Path)]) -> ThothResult<()> {
    let mut resolved: Vec<(&str, std::path::PathBuf)> = Vec::with_capacity(artifacts.len());
    for (label, path) in artifacts {
        let identity = resolve_artifact_identity(path)?;
        if let Some((other, _)) = resolved.iter().find(|(_, existing)| *existing == identity) {
            return Err(ThothError::LicenceNormalizationArtifactAlias(format!(
                "the {label} path and the {other} path resolve to the same location {}",
                identity.display()
            )));
        }
        resolved.push((label, identity));
    }
    Ok(())
}

fn resolve_artifact_identity(path: &Path) -> ThothResult<std::path::PathBuf> {
    if let Ok(canonical) = path.canonicalize() {
        return Ok(canonical);
    }
    let file_name = path.file_name().ok_or_else(|| {
        ThothError::LicenceNormalizationArtifactAlias(format!(
            "{} is not a usable file path",
            path.display()
        ))
    })?;
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let parent = parent.canonicalize().map_err(|error| {
        ThothError::LicenceNormalizationArtifactAlias(format!(
            "cannot resolve the directory for {}: {error}",
            path.display()
        ))
    })?;
    Ok(parent.join(file_name))
}

fn read_artifact(label: &str, path: &Path) -> ThothResult<Vec<u8>> {
    std::fs::read(path).map_err(|error| {
        ThothError::LicenceNormalizationInvalidInput(format!(
            "could not read the {label} at {}: {error}",
            path.display()
        ))
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> ThothResult<()> {
    std::fs::write(path, bytes).map_err(Into::into)
}

fn write_report(path: &Path, report: &NormalizationReport) -> ThothResult<()> {
    let rendered = serde_json::to_vec_pretty(report)?;
    std::fs::write(path, rendered).map_err(Into::into)
}

#[cfg(all(test, feature = "backend"))]
mod tests {
    //! `MIG-01-LIC-NORM-01` facade evidence. Artifact/mechanism tests run
    //! against the exact embedded reviewed artifacts (their raw-byte SHA-256
    //! is asserted against the authorized identities before anything else is
    //! trusted) and against deliberately corrupted fixtures; database tests
    //! run against a real disposable PostgreSQL database with the migrations
    //! applied. No test reads or writes production data.

    use super::*;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_db_url,
    };
    use crate::model::Crud;
    use diesel::sql_query;

    /// Statement-level SQL capture through Diesel's own instrumentation hook.
    ///
    /// The repository's existing `SqlProbe` harness lives in
    /// `graphql::dataloader::fixture`, which is private to the `graphql`
    /// module; exposing it would require a visibility change outside this
    /// task's approved write budget. This is therefore the same minimal
    /// pattern — `set_default_instrumentation` capturing `StartQuery` events
    /// on a dedicated pool — kept test-local and torn down on drop.
    mod sql_capture {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Mutex, OnceLock};

        use diesel::connection::{
            set_default_instrumentation, Instrumentation, InstrumentationEvent,
        };
        use diesel::pg::PgConnection;
        use diesel::r2d2::ConnectionManager;

        use crate::db::PgPool;

        static CAPTURED_SQL: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();
        static CAPTURE_ARMED: AtomicUsize = AtomicUsize::new(0);

        fn captured() -> Arc<Mutex<Vec<String>>> {
            CAPTURED_SQL
                .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
                .clone()
        }

        fn capturing_instrumentation() -> Option<Box<dyn Instrumentation>> {
            Some(Box::new(|event: InstrumentationEvent<'_>| {
                if CAPTURE_ARMED.load(Ordering::SeqCst) == 0 {
                    return;
                }
                if let InstrumentationEvent::StartQuery { query, .. } = event {
                    if let Ok(mut sink) = captured().lock() {
                        sink.push(query.to_string());
                    }
                }
            }))
        }

        fn no_instrumentation() -> Option<Box<dyn Instrumentation>> {
            None
        }

        pub(super) struct SqlCapture {
            pub(super) pool: Arc<PgPool>,
        }

        impl SqlCapture {
            pub(super) fn install(database_url: &str) -> Self {
                set_default_instrumentation(capturing_instrumentation)
                    .expect("failed to install Diesel instrumentation");
                let manager = ConnectionManager::<PgConnection>::new(database_url);
                let pool = diesel::r2d2::Pool::builder()
                    .max_size(4)
                    .build(manager)
                    .expect("failed to build captured pool");
                Self {
                    pool: Arc::new(pool),
                }
            }

            pub(super) fn start(&self) {
                captured().lock().expect("SQL capture lock").clear();
                CAPTURE_ARMED.store(1, Ordering::SeqCst);
            }

            pub(super) fn captured_statements(&self) -> Vec<String> {
                CAPTURE_ARMED.store(0, Ordering::SeqCst);
                captured().lock().expect("SQL capture lock").clone()
            }
        }

        impl Drop for SqlCapture {
            fn drop(&mut self) {
                CAPTURE_ARMED.store(0, Ordering::SeqCst);
                let _ = set_default_instrumentation(no_instrumentation);
            }
        }
    }

    use sql_capture::SqlCapture;

    // -----------------------------------------------------------------
    // The exact reviewed artifacts and their authorized identities
    // -----------------------------------------------------------------

    /// SHA-256 of the exact raw bytes of
    /// `MIG-01-LIC-NORM-01-deterministic-manifest-v2.json`.
    const AUTHORIZED_DETERMINISTIC_SHA256: &str =
        "8e40e2bd83dc2a5263d1a4b087fbe0813ac2b82bd51280fc49521dd49daa327c";
    /// SHA-256 of the exact raw bytes of
    /// `MIG-01-LIC-NORM-01-manual-resolution.json`.
    const AUTHORIZED_MANUAL_SHA256: &str =
        "2061c4f467069ea563e1f02a53a15bd1759efc9354d87793cf79c91a67f8a2b0";
    /// SHA-256 of the exact raw bytes of
    /// `MIG-01-production-manifest-candidate.json`.
    const AUTHORIZED_MIG01_SHA256: &str =
        "4b47738d7167b4b83d076e8006e471fa140a7dc5c0c693f049245cc1cb91d26f";

    /// The 9 reviewed distinct canonical targets, ascending.
    const REVIEWED_TARGETS: [&str; 9] = [
        "https://creativecommons.org/licenses/by-nc-nd/2.0/",
        "https://creativecommons.org/licenses/by-nc-nd/3.0/",
        "https://creativecommons.org/licenses/by-nc-nd/4.0/",
        "https://creativecommons.org/licenses/by-nc-sa/4.0/",
        "https://creativecommons.org/licenses/by-nc/4.0/",
        "https://creativecommons.org/licenses/by-nd/4.0/",
        "https://creativecommons.org/licenses/by-sa/4.0/",
        "https://creativecommons.org/licenses/by/4.0/",
        "https://creativecommons.org/publicdomain/zero/1.0/",
    ];

    // ------------------------------------------------------------------
    // The exact raw bytes of the three reviewed artifacts, embedded so the
    // suite is hermetic. Their SHA-256 is asserted against the authorized
    // identities above before any content is trusted; a single changed byte
    // fails the identity tests.
    // ------------------------------------------------------------------

    /// `MIG-01-LIC-NORM-01-deterministic-manifest-v2.json`, exact raw bytes.
    const REAL_DETERMINISTIC_MANIFEST: &str = r#"{"schemaVersion":1,"task":"MIG-01-LIC-NORM-01","description":"Deterministic representation-only licence normalizations. Exact raw bytes are review evidence; this manifest does not authorize execution.","replacements":[{"from":"https://creativecommons.org/licenses/by-nc-nd/2.0/deed.es","to":"https://creativecommons.org/licenses/by-nc-nd/2.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Tinta Limón Ediciones"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/3.0/deed.i","to":"https://creativecommons.org/licenses/by-nc-nd/3.0/","reason":"malformed deed-language suffix discarded, not repaired; retained path prefix unambiguously preserves CC BY-NC-ND 3.0 rights/version","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/3.0/deed.it","to":"https://creativecommons.org/licenses/by-nc-nd/3.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/3.0/legalcode","to":"https://creativecommons.org/licenses/by-nc-nd/3.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Leuven University Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/4.0/deed.en","to":"https://creativecommons.org/licenses/by-nc-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press","The White Horse Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/4.0/deed.it","to":"https://creativecommons.org/licenses/by-nc-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode","to":"https://creativecommons.org/licenses/by-nc-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Leuven University Press","University of London Press"]},{"from":"https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode.es","to":"https://creativecommons.org/licenses/by-nc-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Open Book Publishers"]},{"from":"https://creativecommons.org/licenses/by-nc-sa/4.0/deed.en","to":"https://creativecommons.org/licenses/by-nc-sa/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Erratum Press","Mattering Press","University of Groningen Press"]},{"from":"https://creativecommons.org/licenses/by-nc-sa/4.0/deed.it","to":"https://creativecommons.org/licenses/by-nc-sa/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.nl","to":"https://creativecommons.org/licenses/by-nc-sa/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["University of Groningen Press"]},{"from":"https://creativecommons.org/licenses/by-nc/4.0/deed.en","to":"https://creativecommons.org/licenses/by-nc/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Iowa State University Digital Press","LSE Press","LSHTM Press","The White Horse Press"]},{"from":"https://creativecommons.org/licenses/by-nc/4.0/legalcode","to":"https://creativecommons.org/licenses/by-nc/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Leuven University Press","Open Book Publishers","University of London Press"]},{"from":"https://creativecommons.org/licenses/by-nd/4.0/deed.en","to":"https://creativecommons.org/licenses/by-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Iowa State University Digital Press"]},{"from":"https://creativecommons.org/licenses/by-nd/4.0/legalcode","to":"https://creativecommons.org/licenses/by-nd/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Leuven University Press"]},{"from":"https://creativecommons.org/licenses/by-sa/4.0/deed.de","to":"https://creativecommons.org/licenses/by-sa/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["HsH Applied Academics"]},{"from":"https://creativecommons.org/licenses/by-sa/4.0/deed.it","to":"https://creativecommons.org/licenses/by-sa/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by/4.0/deed.de","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["HsH Applied Academics","Universitätsverlag Potsdam","Verlag Westfälisches Dampfboot","adocs publishing"]},{"from":"https://creativecommons.org/licenses/by/4.0/deed.en","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Editorial Mar Caribe","Iowa State University Digital Press","The White Horse Press","Verlag Westfälisches Dampfboot"]},{"from":"https://creativecommons.org/licenses/by/4.0/deed.es","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Editorial Universitaria UTE"]},{"from":"https://creativecommons.org/licenses/by/4.0/deed.it","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Milano University Press"]},{"from":"https://creativecommons.org/licenses/by/4.0/legalcode","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Leuven University Press"]},{"from":"https://creativecommons.org/licenses/by/4.0/legalcode.de","to":"https://creativecommons.org/licenses/by/4.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["Universitätsverlag Potsdam"]},{"from":"https://creativecommons.org/publicdomain/zero/1.0/deed.en","to":"https://creativecommons.org/publicdomain/zero/1.0/","reason":"representation-only Creative Commons deed/legalcode canonicalization; rights and version unchanged","observedPublishers":["mediastudies.press"]}]}"#;

    /// `MIG-01-LIC-NORM-01-manual-resolution.json`, exact raw bytes.
    const REAL_MANUAL_REGISTER: &str = r#"{"schemaVersion":1,"task":"MIG-01-LIC-NORM-01","description":"Manual-resolution register. Values in this file MUST NOT be automatically normalized or written without separate reviewed work-level evidence.","values":[{"value":"[https://creativecommons.org/licenses/by-nc/4.0/](https://creativecommons.org/licenses/by-nc/4.0/) (introduction, chapter introductions); [https://creativecommons.org/public-domain/cc0/](https://creativecommons.org/public-domain/cc0/) (other materials)","reason":"non-current Creative Commons public-domain path","observedPublishers":["mediastudies.press"]},{"value":"http://creativecommons.org/licenses/by-nc-nd/3.0/es/","reason":"jurisdiction-specific Creative Commons URL not accepted by current parser","observedPublishers":["Universidad Pontificia Comillas"]},{"value":"https://books.scielo.org/id/r3dbw/pdf/lisboa-9786556308791.pdf","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://creativecommons.org/licences/by-nc-nc/4.0/","reason":"Creative Commons path typo (licences)","observedPublishers":["adocs publishing"]},{"value":"https://creativecommons.org/licenses/by-nc-nd/3.0/ec/","reason":"jurisdiction-specific Creative Commons URL not accepted by current parser","observedPublishers":["Editorial FLACSO Ecuador"]},{"value":"https://creativecommons.org/licenses/by-nc-sa/4.0//by/4.0","reason":"malformed Creative Commons URL","observedPublishers":["TU Delft OPEN Publishing"]},{"value":"https://creativecommons.org/licenses/by-nc/3.0/de/deed.en","reason":"Creative Commons deed/legalcode URL not accepted by current parser","observedPublishers":["adocs publishing"]},{"value":"https://creativecommons.org/licenses/cc0/4.0/","reason":"CC0 encoded under licenses/cc0 path; current parser expects publicdomain/zero/1.0","observedPublishers":["Open Book Publishers"]},{"value":"https://creativecommons.org/public-domain/cc0/","reason":"non-current Creative Commons public-domain path","observedPublishers":["Open Book Publishers"]},{"value":"https://creativecommons.org/publicdomain/mark/1.0/","reason":"Public Domain Mark is not accepted as a cc-license licence","observedPublishers":["BookHub Nigeria","mediastudies.press"]},{"value":"https://doi.org/10.7476/9786556308791","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/bitstream/ri/35674/4/Extens%C3%A3o%20e%20Pesquisa%20em%20Alimenta%C3%A7%C3%A3o-repositorio.pdf","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/bitstream/ri/36555/1/a-cuia-e-a-bengala-RI.pdf","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/31234","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/35102","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/35315","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/35548","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/35799","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36057","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36058","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36126","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36279","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36280","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36281","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36282","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36394","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://repositorio.ufba.br/handle/ri/36844","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["Editora da Universidade Federal da Bahia"]},{"value":"https://ujonlinepress.uj.ac.za/index.php/ujp/catalog/view/323/1374/6287","reason":"non-Creative-Commons URL stored in licence field","observedPublishers":["UJ Press"]}]}"#;

    /// `MIG-01-production-manifest-candidate.json`, exact raw bytes.
    const REAL_MIG01_MANIFEST: &str = r#"{
  "manifestVersion": 1,
  "description": "MIG-01 production publisher service configuration backfill; package/platform desired state approved from JAR + CTO decisions + thoth-dissemination GitHub variables; exact production licence audit dispositions included for Gate-C dry-run review.",
  "publishers": [
    {
      "publisherId": "01b39a08-3333-4207-b232-9ccf193fa4d2",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "034c5548-ba4f-4cd0-9545-284dc79fe408",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "056061d2-963e-4078-b83f-058660530648",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "057e9281-c700-41a7-8f8f-f508692c7528",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "05edd9b5-eba3-4cde-894b-b8965e71da09",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "ZENODO",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "0834ab96-e8f1-443d-987f-4f5131e22a24",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "09b93b4c-355d-4a62-83ea-b61fec0ae062",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "0a7919ae-c318-4349-8258-a538893c62a5",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "1189e12a-3dfd-4fd8-b80d-2b8f212df1b1",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "12cfb403-873a-41d5-a9da-5b3e4902ffdc",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "1566b370-e9f0-4906-9178-3c1c5c316275",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "1794e004-24fe-446e-bcb0-b0821d0ff6cd",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "17d701c1-307e-4228-83ca-d8e90d7b87a6",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "1baa59cc-04b4-44a9-b168-d970849dd929",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "23946b61-c24e-4cbd-ba71-a3ca99729af5",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "261d7fd8-a0b6-479e-a227-67e58fffa905",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "28422631-e751-43c0-bdff-94f4a8c3b250",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "295c85ff-b1ff-45a0-8d5a-80565d74ea4c",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "29698e28-1227-4330-823b-a3fea51e8b03",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "2b703604-29f4-4b54-8004-3727685d69c2",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "320e82e4-57be-48d6-8903-1556c98bf4f2",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "37687932-6743-4f9e-af55-edb777af966f",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "41104893-1a1d-4903-8d7c-e1f3f168d509",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "448f9390-ede0-42e3-aa94-6c80971582b8",
      "subscriptionPackage": "SPHINX",
      "enabledDistributionPlatforms": [
        "OAPEN",
        "PROQUEST_EBOOK_CENTRAL",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "466fcdca-1366-4416-861d-7923fe813a78",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "46c8022d-09de-4361-85f9-2fd69df1df5e",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: prior Obelisk subscription cancelled; current package OASIS; stale CR/PQ GitHub memberships removed; no enabled distribution platforms."
    },
    {
      "publisherId": "46d5b2ae-ca4f-41c2-b029-c4bffe65b81b",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "47ba9e33-a4ea-444a-8906-7c96aa0c1931",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "CROSSREF"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "49ee65b7-5be0-4686-9631-77b7e53302cf",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "4ab3bec2-c491-46d4-8731-47a5d9b33cc5",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "EBSCO_HOST",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "4c575b1d-13ae-43ed-8aac-a3af7c1832cd",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "4ed3c70c-63d6-49e8-ab81-96365f870a1b",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "535b4a7a-0265-48f5-a736-3e8d9634bcd2",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "585d66b9-4ea5-4edd-bc0b-e84a1f6fef59",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "5b130f79-a87d-4256-b61d-6a6a288d2f42",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "5bb8d0d5-18c4-4467-8711-64a6dcee411a",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "5da5617c-306b-4f68-b60b-abc74faf49f8",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "ZENODO",
        "PROJECT_MUSE",
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "60296b9e-3b2a-47b8-a615-c402e0f4179a",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "61eb98b8-f767-46a5-a733-6e459e729707",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "644b4b98-5f27-4ef0-874b-e14507aa68b5",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "64d5ffa0-cfd4-4ad6-8c07-55bf7510f7ea",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "67d14e6c-8922-4cf9-9bc7-1c03a9357144",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "699975d0-30ae-4968-808b-16faba149700",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "6b115ab2-62b5-4ae5-8134-e6bca0969b8a",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "6c6d94aa-5a10-46ad-ad52-9652af4b17b9",
      "subscriptionPackage": "SPHINX",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "72a3a465-c561-44fe-9d51-0dddd674fd5e",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "75f24fa6-2709-48e5-86b9-99e057e4d84c",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "7999b32c-7148-4250-8d56-826a5ddc9255",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "7c0b2d93-3264-4d83-beb5-7c18898d995a",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "7c7dfdd0-a8ff-45d7-941f-d69c35ab4710",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "7ec3811c-667b-419e-b96c-a726acac610c",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "82dc7886-de2a-4089-a9cc-f2e178313553",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "84f6d19e-bb32-499d-a5dd-86753dd54251",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "85fd969a-a16c-480b-b641-cb9adf979c3b",
      "subscriptionPackage": "PYRAMID",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CAMBRIDGE_UNIVERSITY_LIBRARY",
        "CROSSREF",
        "FIGSHARE",
        "ZENODO",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "8ad30b96-0a93-42a3-b802-49a206a8de7c",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "8b4e6967-497e-40b5-aafa-954b4983ec1f",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "8d630c5e-ff4a-47b2-8839-326c9927bbc8",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "8fb430a6-e227-44f2-96d3-6804bebc4073",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "906b7f88-68cf-4a13-b5cf-7b34970b432a",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "918ab354-8dbc-4e12-b273-53d35bfdbfcb",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROQUEST_EBOOK_CENTRAL"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "930da856-0134-4e4c-93e9-2647da4d32ab",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "931b15d1-fb8a-4e38-a518-82515fa938da",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "935f920c-94a5-4fd5-a7ac-e339af4460dc",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "94a8eb8b-6922-46cd-88c0-a97e8d434737",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "CROSSREF"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "9587e01b-dec2-4047-ac7f-0a5a703a9bbd",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "96324986-4f67-4b2e-99b5-9472502ce28e",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "96e22f7f-4680-43da-971f-2cade94191e2",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "9c41b13c-cecc-4f6a-a151-be4682915ef5",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CAMBRIDGE_UNIVERSITY_LIBRARY",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "9d0849d9-348b-4033-ad62-ef0f8e81242d",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "9f7b3005-f8bd-44d2-b6cc-72b9ff894c06",
      "subscriptionPackage": "SPHINX",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "ZENODO",
        "JSTOR",
        "EBSCO_HOST",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "a4453761-dbc8-4871-b092-9dcc06f5e935",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "a6551c38-1743-4416-aea9-302358733db5",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "a6f4b20a-b1ef-43e2-b9ec-919f2e3dcf25",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "a78d75f5-3ec6-4955-980c-a98a36b9a819",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "a7b392d5-fed8-4bcb-8feb-952c83cb0c11",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "a8391556-4f3a-4a0b-9316-0eb02cae2cda",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "ab3a1120-d7e2-4d58-8d99-72fa626e1e70",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "ada2403c-d390-4e40-878d-a7a2006acf78",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "b04c1564-2a65-437a-87fa-3335ab4a4152",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "b1b11523-2a41-4ed6-96f7-2c5382a578f4",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "b4d02e9c-c0f9-4a0b-bb59-ceecf331e832",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "b61217e4-3134-4bfe-8695-30e047ed3f57",
      "subscriptionPackage": "SPHINX",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "EBSCO_HOST",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "ba67afbb-2b43-4ef8-b1cc-d7333706d54e",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "JSTOR",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "bc025ea4-df75-47cb-aa04-01e4f47a2d72",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "bc5a9069-b123-49c1-bd31-981f8ae0c5dd",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "bcd94fc2-d5bd-4164-8121-ade48d77d350",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "c0642f59-971d-4570-b6cd-757ea3c11220",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "c3923a0b-f066-4a0e-bfe5-d32d9813285d",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "c40a1ed1-ccd5-41b8-8b12-733c963492ee",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "c478ef4b-0d80-476e-9382-abc85f78e4f9",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "cdd2763f-3fd1-4fe4-9f97-12b16b6ad25c",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d02a6d2a-ddb8-496d-98c2-b46ffabc22b9",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d2450d30-9138-464c-9afb-8b6b476514d1",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d2459c17-ae6c-4179-a0ec-9aebd4c2d0be",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d2afd231-c8b0-4d04-ac5a-e0f8365eeec4",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d545e4d3-0536-4bcb-8850-3afe38292516",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "d7fca980-7951-4df9-9942-c16064b7acb4",
      "subscriptionPackage": "SPHINX",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "ZENODO",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "e09df7f0-7f50-4082-a154-64c9cad878db",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "e109b9a8-4288-4223-9499-fd8afa7ebf07",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "e33de497-abc7-4966-a51f-ab6d4b76b0b7",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "e390b0a2-29a6-46b6-9625-1fb0a5195a47",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "e5efe207-ef2c-4748-915c-4855f6bbaa68",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "OAPEN",
        "CROSSREF",
        "ZENODO",
        "PROJECT_MUSE",
        "JSTOR",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY",
        "BKCI"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "e7e9503d-3cd5-4f1b-9c64-c27cca1cfbcf",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "ea0ad5ff-dd59-48f8-8da7-95dfd40c90d8",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "eb087394-61f4-4dea-9f36-5d78c8b39ca6",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "f01e4743-c817-4f95-81ae-0e9c7e9a5e87",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "f0ae98da-c433-45b8-af3f-5c709ad0221b",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    },
    {
      "publisherId": "f2229e70-e973-4e89-b60f-1055fa3d7505",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "f298ddda-0310-4461-8d22-ab2fe6243d43",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "f3619a46-0bc5-4e55-9549-c15b8fff9223",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fa06179c-a8b4-448e-b2b9-93f943d8c9d8",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fa1230a4-8ad4-4834-8b97-e1cf41228e90",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fadd857f-687a-4df2-8cb6-74faa64a7be0",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fb586475-4db9-46ba-8798-1d2b9f600bb4",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fbc88fba-8f14-48a8-8190-bba49c9ae733",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fcdaf4ca-20aa-4a4b-bd0c-e8513cb56086",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fd8aa670-3f50-4bf1-a161-306d8bf6e365",
      "subscriptionPackage": "OASIS",
      "enabledDistributionPlatforms": [],
      "provenance": "CTO 2026-08-20: production publishers outside the reconciled legacy dissemination membership set are OASIS with no enabled distribution platforms."
    },
    {
      "publisherId": "fe991571-b966-4606-8ac5-aa7b22a89d87",
      "subscriptionPackage": "OBELISK",
      "enabledDistributionPlatforms": [
        "INTERNET_ARCHIVE",
        "CROSSREF",
        "ZENODO",
        "EBSCO_HOST",
        "PROQUEST_EBOOK_CENTRAL",
        "GOOGLE_PLAY"
      ],
      "provenance": "Package: JAR current subscription package reconciled for MIG-01; platforms: 2026-08-20 thoth-dissemination GitHub publisher variables supplied by CTO; absence from those variables means disabled; ScienceOpen ignored. OAPEN is linked-normalized with DOAB by Thoth."
    }
  ],
  "licenceDispositions": [
    {
      "value": "[https://creativecommons.org/licenses/by-nc/4.0/](https://creativecommons.org/licenses/by-nc/4.0/) (introduction, chapter introductions); [https://creativecommons.org/public-domain/cc0/](https://creativecommons.org/public-domain/cc0/) (other materials)",
      "disposition": "NORMALIZE"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-nd/2.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-nd/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-nd/3.0/es/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-nd/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-nd/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc-sa/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nc/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nd/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-nd/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by-sa/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by/2.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "http://creativecommons.org/licenses/by/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://books.scielo.org/id/r3dbw/pdf/lisboa-9786556308791.pdf",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licences/by-nc-nc/4.0/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/2.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/2.0/deed.es",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/3.0/deed.i",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/3.0/deed.it",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/3.0/ec/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/3.0/legalcode",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0/deed.it",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode.es",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0//by/4.0",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0/deed.it",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.nl",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/2.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/3.0/de/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/4.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nc/4.0/legalcode",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nd/2.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nd/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nd/4.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-nd/4.0/legalcode",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-sa/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-sa/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by-sa/4.0/deed.de",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by-sa/4.0/deed.it",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/3.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/deed.de",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/deed.es",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/deed.it",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/legalcode",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/by/4.0/legalcode.de",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/licenses/cc0/4.0/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/public-domain/cc0/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/publicdomain/mark/1.0/",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://creativecommons.org/publicdomain/zero/1.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://creativecommons.org/publicdomain/zero/1.0/deed.en",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://doi.org/10.7476/9786556308791",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/bitstream/ri/35674/4/Extens%C3%A3o%20e%20Pesquisa%20em%20Alimenta%C3%A7%C3%A3o-repositorio.pdf",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/bitstream/ri/36555/1/a-cuia-e-a-bengala-RI.pdf",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/31234",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/35102",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/35315",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/35548",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/35799",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36057",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36058",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36126",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36279",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36280",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36281",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36282",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36394",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://repositorio.ufba.br/handle/ri/36844",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://ujonlinepress.uj.ac.za/index.php/ujp/catalog/view/323/1374/6287",
      "disposition": "NORMALIZE"
    },
    {
      "value": "https://www.creativecommons.org/licenses/by-sa/4.0/",
      "disposition": "SUPPORTED"
    },
    {
      "value": "https://www.creativecommons.org/licenses/by/4.0/",
      "disposition": "SUPPORTED"
    }
  ]
}
"#;

    // -----------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------

    fn tmp_path(suffix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("licnorm-{}-{suffix}", Uuid::new_v4()));
        path
    }

    fn write_artifact(bytes: &[u8], suffix: &str) -> std::path::PathBuf {
        let path = tmp_path(suffix);
        std::fs::write(&path, bytes).expect("write artifact");
        path
    }

    /// A MIG-01 manifest fixture scoping the given canonical publishers, with
    /// every reviewed deterministic target SUPPORTED.
    fn fixture_mig01(publishers: &[Uuid]) -> Vec<u8> {
        let publishers: Vec<serde_json::Value> = publishers
            .iter()
            .map(|id| {
                serde_json::json!({
                    "publisherId": id.to_string(),
                    "subscriptionPackage": "OASIS",
                    "enabledDistributionPlatforms": [],
                })
            })
            .collect();
        let dispositions: Vec<serde_json::Value> = REVIEWED_TARGETS
            .iter()
            .map(|target| serde_json::json!({ "value": target, "disposition": "SUPPORTED" }))
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "manifestVersion": 1,
            "publishers": publishers,
            "licenceDispositions": dispositions,
        }))
        .expect("fixture manifest bytes")
    }

    /// The three input files for a database test: the exact real deterministic
    /// manifest and manual register plus a publisher-scoped MIG-01 fixture.
    struct InputFiles {
        deterministic: std::path::PathBuf,
        deterministic_sha256: String,
        manual: std::path::PathBuf,
        manual_sha256: String,
        mig01: std::path::PathBuf,
        mig01_sha256: String,
    }

    fn input_files(publishers: &[Uuid]) -> InputFiles {
        let deterministic_bytes = REAL_DETERMINISTIC_MANIFEST.as_bytes();
        let manual_bytes = REAL_MANUAL_REGISTER.as_bytes();
        let mig01_bytes = fixture_mig01(publishers);
        InputFiles {
            deterministic: write_artifact(deterministic_bytes, "deterministic.json"),
            deterministic_sha256: sha256_hex(deterministic_bytes),
            manual: write_artifact(manual_bytes, "manual.json"),
            manual_sha256: sha256_hex(manual_bytes),
            mig01: write_artifact(&mig01_bytes, "mig01.json"),
            mig01_sha256: sha256_hex(&mig01_bytes),
        }
    }

    fn run_dry_run(
        pool: &PgPool,
        files: &InputFiles,
    ) -> (DryRunOutcome, Vec<u8>, std::path::PathBuf, String) {
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            deterministic_manifest_path: &files.deterministic,
            expected_deterministic_manifest_sha256: &files.deterministic_sha256,
            manual_register_path: &files.manual,
            expected_manual_register_sha256: &files.manual_sha256,
            mig01_manifest_path: &files.mig01,
            expected_mig01_manifest_sha256: &files.mig01_sha256,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
        };
        let outcome = dry_run(pool, &request).expect("dry run");
        let plan_bytes = std::fs::read(&plan_out).expect("plan bytes");
        let report_sha = sha256_hex(&std::fs::read(&report_out).expect("report bytes"));
        (outcome, plan_bytes, report_out, report_sha)
    }

    fn try_dry_run(pool: &PgPool, files: &InputFiles) -> ThothResult<DryRunOutcome> {
        let plan_out = tmp_path("plan.json");
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            deterministic_manifest_path: &files.deterministic,
            expected_deterministic_manifest_sha256: &files.deterministic_sha256,
            manual_register_path: &files.manual,
            expected_manual_register_sha256: &files.manual_sha256,
            mig01_manifest_path: &files.mig01,
            expected_mig01_manifest_sha256: &files.mig01_sha256,
            plan_out_path: &plan_out,
            report_out_path: &report_out,
        };
        dry_run(pool, &request)
    }

    fn apply_plan(
        pool: &PgPool,
        files: &InputFiles,
        plan_bytes: &[u8],
        expected_plan_sha256: &str,
        mode: ApplyExecutionMode,
    ) -> ThothResult<ApplyOutcome> {
        let plan_path = write_artifact(plan_bytes, "reviewed-plan.json");
        let report_out = tmp_path("apply-report.json");
        let request = ApplyRequest {
            deterministic_manifest_path: &files.deterministic,
            expected_deterministic_manifest_sha256: &files.deterministic_sha256,
            manual_register_path: &files.manual,
            expected_manual_register_sha256: &files.manual_sha256,
            mig01_manifest_path: &files.mig01,
            expected_mig01_manifest_sha256: &files.mig01_sha256,
            plan_path: &plan_path,
            expected_plan_sha256,
            report_out_path: &report_out,
            mode,
        };
        apply(pool, &request)
    }

    fn set_licence(pool: &PgPool, work_id: Uuid, value: Option<&str>) {
        let mut connection = pool.get().unwrap();
        diesel::update(work::table.find(work_id))
            .set(work::license.eq(value))
            .execute(&mut connection)
            .expect("set licence");
    }

    /// Create a Work under the imprint carrying the given licence, returning
    /// its re-read state (so the token reflects the licence write).
    fn create_licensed_work(
        pool: &PgPool,
        imprint: &crate::model::imprint::Imprint,
        licence: &str,
    ) -> Work {
        let created = create_work(pool, imprint);
        set_licence(pool, created.work_id, Some(licence));
        Work::from_id(pool, &created.work_id).expect("re-read work")
    }

    fn history_rows(pool: &PgPool, work_id: Uuid, actor: &str) -> Vec<serde_json::Value> {
        let mut connection = pool.get().unwrap();
        work_history::table
            .filter(work_history::work_id.eq(work_id))
            .filter(work_history::user_id.eq(actor))
            .select(work_history::data)
            .load(&mut connection)
            .unwrap()
    }

    fn history_count(pool: &PgPool, work_id: Uuid) -> i64 {
        let mut connection = pool.get().unwrap();
        work_history::table
            .filter(work_history::work_id.eq(work_id))
            .count()
            .get_result(&mut connection)
            .unwrap()
    }

    fn insert_history_row(pool: &PgPool, work_id: Uuid, user_id: &str, data: serde_json::Value) {
        let mut connection = pool.get().unwrap();
        crate::model::work::NewWorkHistory {
            work_id,
            user_id: user_id.to_string(),
            data,
        }
        .insert(&mut connection)
        .expect("insert history row");
    }

    /// Bump a Work's `updated_at` by exactly one microsecond, leaving the
    /// licence untouched: a sub-second-only token difference.
    fn bump_updated_at_one_microsecond(pool: &PgPool, work_id: Uuid) {
        let mut connection = pool.get().unwrap();
        sql_query(format!(
            "UPDATE work SET updated_at = updated_at + interval '1 microsecond' \
             WHERE work_id = '{work_id}'"
        ))
        .execute(&mut connection)
        .expect("bump updated_at");
    }

    // -----------------------------------------------------------------
    // Real reviewed artifacts: identity and mechanical invariants
    // -----------------------------------------------------------------

    #[test]
    fn embedded_deterministic_manifest_matches_the_authorized_identity() {
        assert_eq!(
            sha256_hex(REAL_DETERMINISTIC_MANIFEST.as_bytes()),
            AUTHORIZED_DETERMINISTIC_SHA256,
            "the embedded deterministic manifest must be byte-identical to the reviewed artifact"
        );
    }

    #[test]
    fn embedded_manual_register_matches_the_authorized_identity() {
        assert_eq!(
            sha256_hex(REAL_MANUAL_REGISTER.as_bytes()),
            AUTHORIZED_MANUAL_SHA256,
            "the embedded manual register must be byte-identical to the reviewed artifact"
        );
    }

    #[test]
    fn embedded_mig01_manifest_matches_the_authorized_identity() {
        assert_eq!(
            sha256_hex(REAL_MIG01_MANIFEST.as_bytes()),
            AUTHORIZED_MIG01_SHA256,
            "the embedded MIG-01 manifest must be byte-identical to the bound artifact"
        );
    }

    #[test]
    fn the_real_deterministic_manifest_passes_every_mechanical_invariant() {
        let manifest = parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes())
            .expect("the reviewed manifest validates");
        assert_eq!(manifest.replacements.len(), DETERMINISTIC_RULE_COUNT);
        let targets: BTreeSet<&str> = manifest
            .replacements
            .iter()
            .map(|rule| rule.to.as_str())
            .collect();
        assert_eq!(targets.len(), DETERMINISTIC_TARGET_COUNT);
        let expected: BTreeSet<&str> = REVIEWED_TARGETS.iter().copied().collect();
        assert_eq!(targets, expected, "the 9 reviewed targets are exact");
    }

    #[test]
    fn every_real_target_parses_through_the_canonical_cc_license_crate() {
        for target in REVIEWED_TARGETS {
            assert!(
                cc_license::License::from_url(target).is_ok(),
                "cc_license 0.1.0 must accept {target}"
            );
        }
    }

    #[test]
    fn the_real_manual_register_parses_and_carries_exactly_28_values() {
        let register = parse_manual_register(REAL_MANUAL_REGISTER.as_bytes())
            .expect("the reviewed register validates");
        assert_eq!(register.values.len(), MANUAL_VALUE_COUNT);
    }

    #[test]
    fn the_real_deterministic_and_manual_sets_do_not_overlap() {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let register = parse_manual_register(REAL_MANUAL_REGISTER.as_bytes()).unwrap();
        verify_no_overlap(&manifest, &register).expect("no deterministic/manual overlap");
    }

    #[test]
    fn every_real_target_is_exact_string_supported_in_the_bound_mig01_manifest() {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let mig01 = parse_mig01_manifest(REAL_MIG01_MANIFEST.as_bytes())
            .expect("the bound MIG-01 manifest parses");
        verify_targets_supported(&manifest, &mig01)
            .expect("all 9 reviewed targets are SUPPORTED in the bound manifest");
    }

    #[test]
    fn the_real_artifacts_pass_the_complete_hash_bound_input_verification() {
        let deterministic = write_artifact(REAL_DETERMINISTIC_MANIFEST.as_bytes(), "d.json");
        let manual = write_artifact(REAL_MANUAL_REGISTER.as_bytes(), "m.json");
        let mig01 = write_artifact(REAL_MIG01_MANIFEST.as_bytes(), "p.json");
        let inputs = load_and_verify_inputs(
            &deterministic,
            AUTHORIZED_DETERMINISTIC_SHA256,
            &manual,
            AUTHORIZED_MANUAL_SHA256,
            &mig01,
            AUTHORIZED_MIG01_SHA256,
        )
        .expect("the exact reviewed artifacts verify end-to-end");
        assert_eq!(inputs.deterministic_sha256, AUTHORIZED_DETERMINISTIC_SHA256);
        assert_eq!(inputs.manual_sha256, AUTHORIZED_MANUAL_SHA256);
        assert_eq!(inputs.mig01_sha256, AUTHORIZED_MIG01_SHA256);
        assert_eq!(inputs.rules().len(), DETERMINISTIC_RULE_COUNT);
        assert_eq!(inputs.targets().len(), DETERMINISTIC_TARGET_COUNT);
    }

    // -----------------------------------------------------------------
    // Hash binding rejects any artifact mismatch, before parsing
    // -----------------------------------------------------------------

    #[test]
    fn a_wrong_deterministic_manifest_hash_is_rejected() {
        let deterministic = write_artifact(REAL_DETERMINISTIC_MANIFEST.as_bytes(), "d.json");
        let manual = write_artifact(REAL_MANUAL_REGISTER.as_bytes(), "m.json");
        let mig01 = write_artifact(REAL_MIG01_MANIFEST.as_bytes(), "p.json");
        let error = load_and_verify_inputs(
            &deterministic,
            AUTHORIZED_MANUAL_SHA256, // deliberately the wrong artifact's hash
            &manual,
            AUTHORIZED_MANUAL_SHA256,
            &mig01,
            AUTHORIZED_MIG01_SHA256,
        )
        .expect_err("a hash mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("deterministic manifest")
        ));
    }

    #[test]
    fn a_wrong_manual_register_hash_is_rejected() {
        let deterministic = write_artifact(REAL_DETERMINISTIC_MANIFEST.as_bytes(), "d.json");
        let manual = write_artifact(REAL_MANUAL_REGISTER.as_bytes(), "m.json");
        let mig01 = write_artifact(REAL_MIG01_MANIFEST.as_bytes(), "p.json");
        let error = load_and_verify_inputs(
            &deterministic,
            AUTHORIZED_DETERMINISTIC_SHA256,
            &manual,
            AUTHORIZED_DETERMINISTIC_SHA256,
            &mig01,
            AUTHORIZED_MIG01_SHA256,
        )
        .expect_err("a hash mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("manual-resolution register")
        ));
    }

    #[test]
    fn a_wrong_mig01_manifest_hash_is_rejected() {
        let deterministic = write_artifact(REAL_DETERMINISTIC_MANIFEST.as_bytes(), "d.json");
        let manual = write_artifact(REAL_MANUAL_REGISTER.as_bytes(), "m.json");
        let mig01 = write_artifact(REAL_MIG01_MANIFEST.as_bytes(), "p.json");
        let error = load_and_verify_inputs(
            &deterministic,
            AUTHORIZED_DETERMINISTIC_SHA256,
            &manual,
            AUTHORIZED_MANUAL_SHA256,
            &mig01,
            AUTHORIZED_DETERMINISTIC_SHA256,
        )
        .expect_err("a hash mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("MIG-01 manifest")
        ));
    }

    #[test]
    fn hash_comparison_uses_the_exact_raw_bytes_not_a_normalized_form() {
        // Appending a single byte (a trailing newline) must change the
        // identity: artifact bytes are never normalized before hashing.
        let mut bytes = REAL_DETERMINISTIC_MANIFEST.as_bytes().to_vec();
        bytes.push(b'\n');
        assert_ne!(sha256_hex(&bytes), AUTHORIZED_DETERMINISTIC_SHA256);
    }

    // -----------------------------------------------------------------
    // Deterministic-manifest schema and invariant rejection
    // -----------------------------------------------------------------

    fn real_deterministic_value() -> serde_json::Value {
        serde_json::from_str(REAL_DETERMINISTIC_MANIFEST).unwrap()
    }

    fn reject_deterministic(value: &serde_json::Value) -> ThothError {
        parse_deterministic_manifest(&serde_json::to_vec(value).unwrap())
            .expect_err("the corrupted manifest must be rejected")
    }

    #[test]
    fn a_byte_order_mark_is_rejected() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(REAL_DETERMINISTIC_MANIFEST.as_bytes());
        assert!(matches!(
            parse_deterministic_manifest(&bytes),
            Err(ThothError::LicenceNormalizationInvalidInput(_))
        ));
    }

    #[test]
    fn malformed_json_is_rejected() {
        assert!(matches!(
            parse_deterministic_manifest(b"{not json"),
            Err(ThothError::LicenceNormalizationInvalidInput(_))
        ));
        assert!(matches!(
            parse_manual_register(b"{not json"),
            Err(ThothError::LicenceNormalizationInvalidInput(_))
        ));
    }

    #[test]
    fn an_unknown_manifest_field_is_rejected() {
        let mut value = real_deterministic_value();
        value["unexpectedInstruction"] = serde_json::json!("do something unreviewed");
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(_)
        ));
    }

    #[test]
    fn an_unsupported_schema_version_is_rejected() {
        let mut value = real_deterministic_value();
        value["schemaVersion"] = serde_json::json!(2);
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("schema version")
        ));
    }

    #[test]
    fn a_foreign_task_identity_is_rejected() {
        let mut value = real_deterministic_value();
        value["task"] = serde_json::json!("SOME-OTHER-TASK");
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("task")
        ));
    }

    #[test]
    fn a_duplicate_source_value_is_rejected() {
        let mut value = real_deterministic_value();
        let first = value["replacements"][0].clone();
        value["replacements"][1] = first;
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("duplicate deterministic source value")
        ));
    }

    #[test]
    fn a_from_equals_to_rule_is_rejected() {
        let mut value = real_deterministic_value();
        let target = value["replacements"][0]["to"].clone();
        value["replacements"][0]["from"] = target;
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("maps a value to itself")
        ));
    }

    #[test]
    fn a_replacement_chain_is_rejected() {
        // Rewrite rule 1 so its `from` is exactly rule 0's canonical target:
        // the target then appears as another rule's source, forming a chain.
        let mut value = real_deterministic_value();
        let target = value["replacements"][0]["to"].as_str().unwrap().to_string();
        value["replacements"][1]["from"] = serde_json::json!(target);
        value["replacements"][1]["to"] =
            serde_json::json!("https://creativecommons.org/licenses/by-sa/4.0/");
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("replacement chain")
        ));
    }

    #[test]
    fn a_noncanonical_target_is_rejected() {
        for bad_target in [
            "http://creativecommons.org/licenses/by/4.0/",
            "https://creativecommons.org/licenses/by/4.0",
            "https://creativecommons.org/licenses/by/4.0/extra/",
            "https://www.creativecommons.org/licenses/by/4.0/",
            "https://example.com/licenses/by/4.0/",
        ] {
            let mut value = real_deterministic_value();
            value["replacements"][0]["to"] = serde_json::json!(bad_target);
            value["replacements"][0]["from"] = serde_json::json!(format!("{bad_target}deed.en"));
            assert!(
                matches!(
                    reject_deterministic(&value),
                    ThothError::LicenceNormalizationInvalidInput(ref message)
                        if message.contains("not a canonical Creative Commons URL")
                ),
                "{bad_target} must be rejected as a canonical target"
            );
        }
    }

    #[test]
    fn a_source_not_prefixed_by_its_target_is_rejected() {
        let mut value = real_deterministic_value();
        value["replacements"][0]["from"] =
            serde_json::json!("https://creativecommons.org/licenses/by-sa/3.0/deed.en");
        value["replacements"][0]["to"] =
            serde_json::json!("https://creativecommons.org/licenses/by-sa/4.0/");
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("does not start with its target")
        ));
    }

    #[test]
    fn jurisdiction_stripping_is_rejected() {
        // `.../3.0/es/ -> .../3.0/` discards a jurisdiction path segment, not a
        // deed/legalcode token. It must never validate.
        let mut value = real_deterministic_value();
        value["replacements"][0]["from"] =
            serde_json::json!("https://creativecommons.org/licenses/by-nc-nd/3.0/es/");
        value["replacements"][0]["to"] =
            serde_json::json!("https://creativecommons.org/licenses/by-nc-nd/3.0/");
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("jurisdiction")
        ));
    }

    #[test]
    fn invalid_discarded_suffixes_are_rejected() {
        for bad_suffix in [
            "es/",
            "legalcode/extra",
            "deed.",
            "deed.9x",
            "index.html",
            "ec/",
        ] {
            let mut value = real_deterministic_value();
            value["replacements"][0]["from"] = serde_json::json!(format!(
                "https://creativecommons.org/licenses/by-nc-nd/3.0/{bad_suffix}"
            ));
            value["replacements"][0]["to"] =
                serde_json::json!("https://creativecommons.org/licenses/by-nc-nd/3.0/");
            assert!(
                matches!(
                    reject_deterministic(&value),
                    ThothError::LicenceNormalizationInvalidInput(_)
                ),
                "suffix {bad_suffix:?} must be rejected"
            );
        }
    }

    #[test]
    fn valid_deed_and_legalcode_suffixes_are_the_only_removable_tokens() {
        for good_suffix in [
            "deed",
            "deed.en",
            "deed.pt-BR",
            "legalcode",
            "legalcode.nl",
            "deed.i",
        ] {
            assert!(
                DISCARDED_SUFFIX.is_match(good_suffix),
                "reviewed suffix {good_suffix:?} must match"
            );
        }
    }

    #[test]
    fn a_wrong_rule_count_is_rejected() {
        let mut value = real_deterministic_value();
        let replacements = value["replacements"].as_array_mut().unwrap();
        replacements.pop();
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("23 rules")
        ));
    }

    #[test]
    fn a_wrong_distinct_target_count_is_rejected() {
        // Keep 24 rules but collapse to 8 targets: replace the single
        // by-nc-nd/2.0 rule with another by/4.0 suffix variant.
        let mut value = real_deterministic_value();
        let replacements = value["replacements"].as_array_mut().unwrap();
        let index = replacements
            .iter()
            .position(|rule| {
                rule["to"]
                    == serde_json::json!("https://creativecommons.org/licenses/by-nc-nd/2.0/")
            })
            .expect("the 2.0 rule exists");
        replacements[index] = serde_json::json!({
            "from": "https://creativecommons.org/licenses/by/4.0/deed.fr",
            "to": "https://creativecommons.org/licenses/by/4.0/",
            "reason": "test fixture",
            "observedPublishers": [],
        });
        assert!(matches!(
            reject_deterministic(&value),
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("8 distinct targets")
        ));
    }

    // -----------------------------------------------------------------
    // Manual-register rejection and non-executability
    // -----------------------------------------------------------------

    #[test]
    fn a_wrong_manual_value_count_is_rejected() {
        let mut value: serde_json::Value = serde_json::from_str(REAL_MANUAL_REGISTER).unwrap();
        value["values"].as_array_mut().unwrap().pop();
        assert!(matches!(
            parse_manual_register(&serde_json::to_vec(&value).unwrap()),
            Err(ThothError::LicenceNormalizationInvalidInput(ref message))
                if message.contains("27 values")
        ));
    }

    #[test]
    fn a_duplicate_manual_value_is_rejected() {
        let mut value: serde_json::Value = serde_json::from_str(REAL_MANUAL_REGISTER).unwrap();
        let first = value["values"][0].clone();
        value["values"][1] = first;
        assert!(matches!(
            parse_manual_register(&serde_json::to_vec(&value).unwrap()),
            Err(ThothError::LicenceNormalizationInvalidInput(ref message))
                if message.contains("duplicate manual-resolution value")
        ));
    }

    #[test]
    fn a_manual_value_overlapping_the_deterministic_sources_is_rejected() {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(REAL_MANUAL_REGISTER).unwrap();
        value["values"][0]["value"] = serde_json::json!(manifest.replacements[0].from.clone());
        let register = parse_manual_register(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(matches!(
            verify_no_overlap(&manifest, &register),
            Err(ThothError::LicenceNormalizationInvalidInput(ref message))
                if message.contains("never executable")
        ));
    }

    #[test]
    fn manual_categories_can_never_be_expressed_as_deterministic_rules() {
        // Each representative manual-register category, written as if it were
        // an automatic rule, must fail a mechanical invariant. Nothing may
        // promote, infer, repair or reinterpret a manual value.
        let cases: [(&str, &str); 8] = [
            // jurisdiction-specific URL (http scheme and jurisdiction path)
            (
                "http://creativecommons.org/licenses/by-nc-nd/3.0/es/",
                "https://creativecommons.org/licenses/by-nc-nd/3.0/",
            ),
            // jurisdiction port on https
            (
                "https://creativecommons.org/licenses/by-nc-nd/3.0/ec/",
                "https://creativecommons.org/licenses/by-nc-nd/3.0/",
            ),
            // path typo (licences) plus rights typo
            (
                "https://creativecommons.org/licences/by-nc-nc/4.0/",
                "https://creativecommons.org/licenses/by-nc/4.0/",
            ),
            // malformed composite URL
            (
                "https://creativecommons.org/licenses/by-nc-sa/4.0//by/4.0",
                "https://creativecommons.org/licenses/by-nc-sa/4.0/",
            ),
            // CC0 under the licenses path
            (
                "https://creativecommons.org/licenses/cc0/4.0/",
                "https://creativecommons.org/publicdomain/zero/1.0/",
            ),
            // jurisdictional deed not reachable by suffix removal
            (
                "https://creativecommons.org/licenses/by-nc/3.0/de/deed.en",
                "https://creativecommons.org/licenses/by-nc/3.0/",
            ),
            // non-Creative-Commons URL
            (
                "https://doi.org/10.7476/9786556308791",
                "https://creativecommons.org/licenses/by/4.0/",
            ),
            // multi-licence composite
            (
                "[https://creativecommons.org/licenses/by-nc/4.0/](https://creativecommons.org/licenses/by-nc/4.0/) (introduction, chapter introductions); [https://creativecommons.org/public-domain/cc0/](https://creativecommons.org/public-domain/cc0/) (other materials)",
                "https://creativecommons.org/licenses/by-nc/4.0/",
            ),
        ];
        for (index, (from, to)) in cases.iter().enumerate() {
            let rule = ReplacementRule {
                from: (*from).to_string(),
                to: (*to).to_string(),
                reason: None,
                observed_publishers: Vec::new(),
            };
            assert!(
                validate_rule(index, &rule).is_err(),
                "manual category {from:?} must never validate as an automatic rule"
            );
        }
    }

    #[test]
    fn public_domain_mark_is_rejected_by_the_canonical_parser_gate() {
        // The Public Domain Mark URL matches the canonical shape but is not a
        // cc-license licence; the parser gate must reject it as a target.
        let rule = ReplacementRule {
            from: "https://creativecommons.org/publicdomain/mark/1.0/deed.en".to_string(),
            to: "https://creativecommons.org/publicdomain/mark/1.0/".to_string(),
            reason: None,
            observed_publishers: Vec::new(),
        };
        assert!(matches!(
            validate_rule(0, &rule),
            Err(ThothError::LicenceNormalizationUnsupportedTarget(_))
        ));
    }

    // -----------------------------------------------------------------
    // MIG-01 SUPPORTED binding
    // -----------------------------------------------------------------

    #[test]
    fn a_missing_target_in_the_mig01_manifest_is_rejected() {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&fixture_mig01(&[])).unwrap();
        value["licenceDispositions"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        let mig01 = parse_mig01_manifest(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(matches!(
            verify_targets_supported(&manifest, &mig01),
            Err(ThothError::LicenceNormalizationUnsupportedTarget(ref message))
                if message.contains("absent")
        ));
    }

    #[test]
    fn a_non_supported_disposition_is_rejected() {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        for disposition in ["NORMALIZE", "DEFER", "REJECT"] {
            let mut value: serde_json::Value = serde_json::from_slice(&fixture_mig01(&[])).unwrap();
            value["licenceDispositions"][0]["disposition"] = serde_json::json!(disposition);
            let mig01 = parse_mig01_manifest(&serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(
                matches!(
                    verify_targets_supported(&manifest, &mig01),
                    Err(ThothError::LicenceNormalizationUnsupportedTarget(ref message))
                        if message.contains("not SUPPORTED")
                ),
                "disposition {disposition} must be rejected"
            );
        }
    }

    #[test]
    fn support_matching_is_exact_string_not_semantic() {
        // A trailing-slash-stripped variant of a target is a different string
        // and must not satisfy the SUPPORTED binding.
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&fixture_mig01(&[])).unwrap();
        value["licenceDispositions"][0]["value"] =
            serde_json::json!("https://creativecommons.org/licenses/by-nc-nd/2.0");
        let mig01 = parse_mig01_manifest(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(verify_targets_supported(&manifest, &mig01).is_err());
    }

    // -----------------------------------------------------------------
    // Canonical plan bytes, hash and timestamp determinism
    // -----------------------------------------------------------------

    fn sample_plan() -> NormalizationPlan {
        let manifest =
            parse_deterministic_manifest(REAL_DETERMINISTIC_MANIFEST.as_bytes()).unwrap();
        let rule = &manifest.replacements[0];
        NormalizationPlan {
            schema_version: PLAN_SCHEMA_VERSION,
            deterministic_manifest_sha256: AUTHORIZED_DETERMINISTIC_SHA256.to_string(),
            manual_resolution_sha256: AUTHORIZED_MANUAL_SHA256.to_string(),
            mig01_manifest_sha256: AUTHORIZED_MIG01_SHA256.to_string(),
            entries: vec![NormalizationPlanEntry {
                work_id: Uuid::from_u128(1),
                publisher_id: Uuid::from_u128(2),
                reviewed_updated_at: Timestamp::parse_from_rfc3339("2026-01-02T03:04:05.123456Z")
                    .unwrap(),
                from: rule.from.clone(),
                to: rule.to.clone(),
            }],
            expected: NormalizationPlanExpected {
                works_considered: 1,
                works_changing: 1,
                history_rows: 1,
                source_value_counts: vec![],
                target_value_counts: vec![],
                manual_unresolved_value_count: 0,
                manual_unresolved_work_count: 0,
            },
        }
    }

    #[test]
    fn plan_bytes_are_deterministic_and_hash_is_raw_byte_sha256() {
        let plan = sample_plan();
        let first = canonical_plan_bytes(&plan).unwrap();
        let second = canonical_plan_bytes(&plan).unwrap();
        assert_eq!(first, second);
        assert_eq!(sha256_hex(&first), sha256_hex(&second));
        // The hash is of exactly the raw bytes: flipping one byte changes it.
        let mut flipped = first.clone();
        *flipped.last_mut().unwrap() ^= 1;
        assert_ne!(sha256_hex(&first), sha256_hex(&flipped));
    }

    #[test]
    fn the_reviewed_timestamp_is_serialized_directly_in_utc_z_form() {
        let plan = sample_plan();
        let bytes = canonical_plan_bytes(&plan).unwrap();
        let rendered = String::from_utf8(bytes.clone()).unwrap();
        // Direct serde emission of the typed Timestamp: RFC 3339, literal Z,
        // full stored sub-second precision.
        assert!(
            rendered.contains("\"reviewedUpdatedAt\":\"2026-01-02T03:04:05.123456Z\""),
            "canonical bytes must carry the direct serde Z form: {rendered}"
        );
        // Neither `to_rfc3339()` (offset form) nor `Display` (space-separated,
        // second precision) representations may appear in canonical bytes.
        let timestamp = plan.entries[0].reviewed_updated_at;
        assert!(!rendered.contains(&timestamp.to_rfc3339()));
        assert!(!rendered.contains(&timestamp.to_string()));
    }

    #[test]
    fn the_typed_timestamp_round_trip_preserves_sub_second_precision() {
        let plan = sample_plan();
        let bytes = canonical_plan_bytes(&plan).unwrap();
        let parsed = parse_canonical_plan(&bytes).unwrap();
        assert_eq!(
            parsed.entries[0].reviewed_updated_at, plan.entries[0].reviewed_updated_at,
            "typed round trip must preserve the exact instant including microseconds"
        );
        assert_eq!(parsed, plan);
    }

    #[test]
    fn a_sub_second_difference_produces_a_different_typed_timestamp() {
        let a = Timestamp::parse_from_rfc3339("2026-01-02T03:04:05.123456Z").unwrap();
        let b = Timestamp::parse_from_rfc3339("2026-01-02T03:04:05.123457Z").unwrap();
        assert_ne!(a, b, "sub-second-only differences must be distinguishable");
    }

    #[test]
    fn a_noncanonical_plan_encoding_is_rejected() {
        let plan = sample_plan();
        let canonical = canonical_plan_bytes(&plan).unwrap();

        // Pretty-printed: semantically identical, byte-different.
        let pretty = serde_json::to_vec_pretty(&plan).unwrap();
        assert!(matches!(
            parse_canonical_plan(&pretty),
            Err(ThothError::LicenceNormalizationNoncanonicalPlan)
        ));

        // Equivalent timestamp in offset (+00:00) form: parses to the same
        // typed instant but re-serializes differently, so it is rejected.
        let offset_form = String::from_utf8(canonical.clone()).unwrap().replace(
            "2026-01-02T03:04:05.123456Z",
            "2026-01-02T03:04:05.123456+00:00",
        );
        assert!(matches!(
            parse_canonical_plan(offset_form.as_bytes()),
            Err(ThothError::LicenceNormalizationNoncanonicalPlan)
        ));

        // Byte-order mark.
        let mut with_bom = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice(&canonical);
        assert!(matches!(
            parse_canonical_plan(&with_bom),
            Err(ThothError::LicenceNormalizationNoncanonicalPlan)
        ));

        // The canonical bytes themselves parse.
        assert!(parse_canonical_plan(&canonical).is_ok());
    }

    #[test]
    fn an_unsupported_plan_schema_version_is_rejected() {
        let mut plan = sample_plan();
        plan.schema_version = 2;
        let bytes = canonical_plan_bytes(&plan).unwrap();
        assert!(matches!(
            parse_canonical_plan(&bytes),
            Err(ThothError::LicenceNormalizationInvalidInput(ref message))
                if message.contains("schema version")
        ));
    }

    #[test]
    fn plan_entries_out_of_work_id_order_are_rejected() {
        let mut plan = sample_plan();
        let mut second = plan.entries[0].clone();
        second.work_id = Uuid::from_u128(0); // below the first entry
        plan.entries.push(second);
        let bytes = canonical_plan_bytes(&plan).unwrap();
        assert!(matches!(
            parse_canonical_plan(&bytes),
            Err(ThothError::LicenceNormalizationNoncanonicalPlan)
        ));
        // A duplicated work id is equally non-canonical.
        let mut plan = sample_plan();
        let duplicate = plan.entries[0].clone();
        plan.entries.push(duplicate);
        let bytes = canonical_plan_bytes(&plan).unwrap();
        assert!(matches!(
            parse_canonical_plan(&bytes),
            Err(ThothError::LicenceNormalizationNoncanonicalPlan)
        ));
    }

    #[test]
    fn the_audit_actor_derives_from_the_plan_hash() {
        let actor = audit_actor("abc123");
        assert_eq!(actor, "MIG-01-LIC-NORM:abc123");
    }

    // -----------------------------------------------------------------
    // Dry run: determinism, discovery, scope and zero writes
    // -----------------------------------------------------------------

    #[test]
    fn dry_run_is_deterministic_and_performs_no_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_a = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.en",
        );
        let work_b = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc/4.0/legalcode",
        );
        // A manual value: reported, never planned.
        let manual_work = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/publicdomain/mark/1.0/",
        );
        // An unaffected supported value.
        let untouched = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/",
        );

        let files = input_files(&[publisher.publisher_id]);
        let (first, first_bytes, _, _) = run_dry_run(&pool, &files);
        let (second, second_bytes, _, _) = run_dry_run(&pool, &files);
        assert_eq!(
            first_bytes, second_bytes,
            "canonical bytes are deterministic"
        );
        assert_eq!(first.plan_sha256, second.plan_sha256);

        // Every affected work, ascending by work id, each PENDING-by-construction.
        let mut expected_ids = vec![work_a.work_id, work_b.work_id];
        expected_ids.sort();
        let planned_ids: Vec<Uuid> = first
            .plan
            .entries
            .iter()
            .map(|entry| entry.work_id)
            .collect();
        assert_eq!(planned_ids, expected_ids);
        assert_eq!(first.plan.expected.works_considered, 2);
        assert_eq!(first.plan.expected.works_changing, 2);
        assert_eq!(first.plan.expected.history_rows, 2);

        // Full-precision typed tokens and exact source/target values.
        for entry in &first.plan.entries {
            let current = Work::from_id(&pool, &entry.work_id).unwrap();
            assert_eq!(entry.reviewed_updated_at, current.updated_at);
            assert_eq!(current.license.as_deref(), Some(entry.from.as_str()));
        }

        // Source/target tallies cover the full reviewed sets, zeros included.
        assert_eq!(
            first.plan.expected.source_value_counts.len(),
            DETERMINISTIC_RULE_COUNT
        );
        assert_eq!(
            first.plan.expected.target_value_counts.len(),
            DETERMINISTIC_TARGET_COUNT
        );
        let by_deed: i64 = first
            .plan
            .expected
            .source_value_counts
            .iter()
            .find(|count| count.value == "https://creativecommons.org/licenses/by/4.0/deed.en")
            .unwrap()
            .works;
        assert_eq!(by_deed, 1);
        let by_target: i64 = first
            .plan
            .expected
            .target_value_counts
            .iter()
            .find(|count| count.value == "https://creativecommons.org/licenses/by/4.0/")
            .unwrap()
            .works;
        assert_eq!(by_target, 1);

        // Manual values are counted, never planned.
        assert_eq!(first.plan.expected.manual_unresolved_value_count, 1);
        assert_eq!(first.plan.expected.manual_unresolved_work_count, 1);
        assert!(first
            .plan
            .entries
            .iter()
            .all(|entry| entry.work_id != manual_work.work_id));
        assert_eq!(
            first.report.manual_unresolved_values.len(),
            MANUAL_VALUE_COUNT
        );

        // Zero writes: licences, tokens and history are untouched.
        for reference in [&work_a, &work_b, &manual_work, &untouched] {
            let now = Work::from_id(&pool, &reference.work_id).unwrap();
            assert_eq!(now.license, reference.license);
            assert_eq!(now.updated_at, reference.updated_at);
            assert_eq!(history_count(&pool, reference.work_id), 0);
        }

        // The report records the expected cache effect without acting on it.
        assert_eq!(first.report.mode, ReportMode::DryRun);
        assert_eq!(first.report.writes_performed, 0);
        assert_eq!(first.report.history_rows_written, 0);
        // Post-write reconciliation evidence exists only on an APPLY report:
        // a dry run performs no writes and claims no resulting state.
        assert_eq!(first.report.deterministic_source_works_remaining, None);
        assert_eq!(first.report.resulting_target_value_counts, None);
        // The dry-run manual aggregates are derived from the same per-value
        // vector the report embeds.
        assert_eq!(
            first.report.manual_unresolved_value_count,
            manual_distinct_value_count(&first.report.manual_unresolved_values)
        );
        assert_eq!(
            first.report.manual_unresolved_work_count,
            manual_work_count(&first.report.manual_unresolved_values)
        );
        assert_eq!(
            first
                .report
                .expected_export_cache_effect
                .affected_publishers,
            1
        );
        assert_eq!(first.report.expected_export_cache_effect.affected_works, 2);
        assert_eq!(
            first.report.expected_export_cache_effect.note,
            EXPORT_CACHE_EFFECT_NOTE
        );
        assert_eq!(
            first.report.affected_publisher_ids,
            vec![publisher.publisher_id]
        );
    }

    #[test]
    fn a_publisher_outside_the_mig01_scope_fails_closed() {
        let (_guard, pool) = setup_test_db();
        let in_scope = create_publisher(&pool);
        let out_of_scope = create_publisher(&pool);
        let imprint = create_imprint(&pool, &out_of_scope);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.en",
        );
        // The manifest scopes only the other publisher.
        let files = input_files(&[in_scope.publisher_id]);
        let error = try_dry_run(&pool, &files).expect_err("scope mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationScopeMismatch(ref message)
                if message.contains(&out_of_scope.publisher_id.to_string())
        ));
    }

    #[test]
    fn aliasing_artifact_paths_are_rejected_before_any_io() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let files = input_files(&[publisher.publisher_id]);
        let report_out = tmp_path("report.json");
        let request = DryRunRequest {
            deterministic_manifest_path: &files.deterministic,
            expected_deterministic_manifest_sha256: &files.deterministic_sha256,
            manual_register_path: &files.manual,
            expected_manual_register_sha256: &files.manual_sha256,
            mig01_manifest_path: &files.mig01,
            expected_mig01_manifest_sha256: &files.mig01_sha256,
            // The plan output aliases the reviewed deterministic manifest.
            plan_out_path: &files.deterministic,
            report_out_path: &report_out,
        };
        assert!(matches!(
            dry_run(&pool, &request),
            Err(ThothError::LicenceNormalizationArtifactAlias(_))
        ));
        // The reviewed input is untouched.
        assert_eq!(
            sha256_hex(&std::fs::read(&files.deterministic).unwrap()),
            files.deterministic_sha256
        );
    }

    // -----------------------------------------------------------------
    // Apply: writes, history, single-column scope and freshness
    // -----------------------------------------------------------------

    #[test]
    fn apply_writes_only_the_licence_with_one_history_row_per_work() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let source = "https://creativecommons.org/licenses/by-nc-sa/4.0/deed.en";
        let target = "https://creativecommons.org/licenses/by-nc-sa/4.0/";
        let work_row = create_licensed_work(&pool, &imprint, source);
        let before = Work::from_id(&pool, &work_row.work_id).unwrap();

        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        let applied = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("apply");
        assert_eq!(applied.written, 1);
        assert_eq!(applied.already_applied, 0);
        assert_eq!(applied.report.mode, ReportMode::Apply);
        assert_eq!(applied.report.writes_performed, 1);
        assert_eq!(applied.report.history_rows_written, 1);
        // Post-write reconciliation evidence: the deterministic-source
        // residual query ran and came back empty, and the resulting target
        // counts are current catalogue state for all 9 reviewed targets.
        assert_eq!(applied.report.deterministic_source_works_remaining, Some(0));
        let resulting = applied
            .report
            .resulting_target_value_counts
            .as_ref()
            .expect("a successful apply carries resulting target counts");
        assert_eq!(resulting.len(), DETERMINISTIC_TARGET_COUNT);
        assert_eq!(count_for(resulting, target), 1);

        let after = Work::from_id(&pool, &work_row.work_id).unwrap();
        assert_eq!(after.license.as_deref(), Some(target));
        // Normal database freshness effects: the triggers own the timestamps.
        assert!(after.updated_at > before.updated_at);
        assert!(after.updated_at_with_relations > before.updated_at_with_relations);
        // Nothing else changed.
        assert_eq!(after.work_type, before.work_type);
        assert_eq!(after.work_status, before.work_status);
        assert_eq!(after.reference, before.reference);
        assert_eq!(after.edition, before.edition);
        assert_eq!(after.imprint_id, before.imprint_id);
        assert_eq!(after.doi, before.doi);
        assert_eq!(after.publication_date, before.publication_date);
        assert_eq!(after.withdrawn_date, before.withdrawn_date);
        assert_eq!(after.place, before.place);
        assert_eq!(after.page_count, before.page_count);
        assert_eq!(after.copyright_holder, before.copyright_holder);
        assert_eq!(after.landing_page, before.landing_page);
        assert_eq!(after.created_at, before.created_at);

        // Exactly one history row, with the exact plan-derived actor, storing
        // the complete pre-update Work state.
        let actor = audit_actor(&outcome.plan_sha256);
        assert_eq!(applied.report.audit_actor, actor);
        let rows = history_rows(&pool, work_row.work_id, &actor);
        assert_eq!(rows.len(), 1);
        assert_eq!(history_count(&pool, work_row.work_id), 1);
        let (historic_licence, historic_token) =
            history_prestate(&rows[0]).expect("pre-state extractable");
        assert_eq!(historic_licence.as_deref(), Some(source));
        assert_eq!(historic_token, before.updated_at);
    }

    #[test]
    fn the_sql_write_surface_is_exactly_one_licence_column() {
        let (_guard, _pool) = setup_test_db();
        let probe = SqlCapture::install(&test_db_url());
        let publisher = create_publisher(&probe.pool);
        let imprint = create_imprint(&probe.pool, &publisher);
        create_licensed_work(
            &probe.pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/legalcode",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&probe.pool, &files);

        probe.start();
        apply_plan(
            &probe.pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("apply");
        let statements = probe.captured_statements();

        // Exactly one UPDATE touches "work", and its SET surface is only the
        // licence column: never a PatchWork-style full changeset.
        let work_updates: Vec<&String> = statements
            .iter()
            .filter(|sql| sql.starts_with("UPDATE") && sql.contains("\"work\""))
            .collect();
        assert_eq!(work_updates.len(), 1, "statements: {statements:#?}");
        let update = work_updates[0];
        assert!(update.contains("SET \"license\""), "{update}");
        for full_row_column in [
            "\"work_type\"",
            "\"work_status\"",
            "\"imprint_id\"",
            "\"doi\"",
            "\"publication_date\"",
            "\"updated_at\"",
            "\"copyright_holder\"",
        ] {
            assert!(
                !update.contains(full_row_column),
                "the licence update must not set {full_row_column}: {update}"
            );
        }

        // Exactly one history insert.
        let history_inserts = statements
            .iter()
            .filter(|sql| sql.starts_with("INSERT") && sql.contains("\"work_history\""))
            .count();
        assert_eq!(history_inserts, 1);

        // No package, platform, job or child-relation table is written.
        for table in [
            "\"publisher\"",
            "\"publisher_service_configuration_history\"",
            "\"publisher_distribution_platform\"",
            "\"distribution_job\"",
            "\"distribution_job_target\"",
            "\"distribution_job_attempt\"",
            "\"work_relation\"",
        ] {
            for sql in &statements {
                let writes = sql.starts_with("INSERT")
                    || sql.starts_with("UPDATE")
                    || sql.starts_with("DELETE");
                assert!(
                    !(writes && sql.contains(table)),
                    "no write may touch {table}: {sql}"
                );
            }
        }

        // The row was locked before the write: the transaction selects the
        // work FOR UPDATE.
        assert!(
            statements
                .iter()
                .any(|sql| sql.contains("\"work\"") && sql.contains("FOR UPDATE")),
            "the work row must be locked with FOR UPDATE: {statements:#?}"
        );
    }

    #[test]
    fn a_repeat_apply_is_an_idempotent_no_op() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_row = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nd/4.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);

        let first = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("first apply");
        assert_eq!(first.written, 1);
        let after_first = Work::from_id(&pool, &work_row.work_id).unwrap();

        let second = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("second apply");
        assert_eq!(second.written, 0);
        assert_eq!(second.already_applied, 1);
        let after_second = Work::from_id(&pool, &work_row.work_id).unwrap();
        assert_eq!(after_second.license, after_first.license);
        assert_eq!(
            after_second.updated_at, after_first.updated_at,
            "a no-op rerun must not touch the row at all"
        );
        assert_eq!(history_count(&pool, work_row.work_id), 1);
        // The rerun performed zero Work writes and zero history inserts, and
        // its post-write reconciliation reports the same catalogue state.
        assert_eq!(second.report.writes_performed, 0);
        assert_eq!(second.report.history_rows_written, 0);
        assert_eq!(second.report.deterministic_source_works_remaining, Some(0));
        assert_eq!(
            second.report.resulting_target_value_counts, first.report.resulting_target_value_counts,
            "the resulting catalogue state is unchanged by a no-op rerun"
        );
    }

    #[test]
    fn an_interrupted_apply_resumes_deterministically() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-sa/4.0/deed.de",
        );
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/publicdomain/zero/1.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        let actor = audit_actor(&outcome.plan_sha256);

        // Simulate an interruption: the first planned work was already
        // committed by a prior invocation of the same reviewed plan.
        let mut connection = pool.get().unwrap();
        apply_entry(&mut connection, &outcome.plan.entries[0], &actor).expect("partial apply");
        drop(connection);

        let resumed = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("resume");
        assert_eq!(resumed.already_applied, 1);
        assert_eq!(resumed.written, 1);
    }

    // -----------------------------------------------------------------
    // Classification: exact proof and drift
    // -----------------------------------------------------------------

    /// A pool, a pending planned entry and the plan actor, for classification
    /// tests. The plan is a genuine dry-run plan over one licensed work.
    fn classified_fixture(
        source: &str,
    ) -> (
        crate::model::tests::db::TestDbGuard,
        std::sync::Arc<PgPool>,
        NormalizationPlanEntry,
        String,
    ) {
        let (guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(&pool, &imprint, source);
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, _, _, _) = run_dry_run(&pool, &files);
        let actor = audit_actor(&outcome.plan_sha256);
        (guard, pool, outcome.plan.entries[0].clone(), actor)
    }

    #[test]
    fn an_untouched_planned_work_classifies_pending() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by/4.0/deed.es");
        assert_eq!(
            classify_entry(&pool, &entry, &actor).unwrap(),
            EntryClassification::Pending
        );
    }

    #[test]
    fn a_correctly_applied_work_classifies_already_applied_only_with_full_proof() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by/4.0/deed.it");
        let mut connection = pool.get().unwrap();
        apply_entry(&mut connection, &entry, &actor).expect("apply");
        drop(connection);
        assert_eq!(
            classify_entry(&pool, &entry, &actor).unwrap(),
            EntryClassification::AlreadyAppliedByThisPlan
        );
    }

    #[test]
    fn a_target_licence_without_any_plan_actor_history_is_drift() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by/4.0/deed.de");
        // The licence reaches the target by some other route; no plan history.
        set_licence(&pool, entry.work_id, Some(&entry.to));
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(
            matches!(class, EntryClassification::Drift(ref reason) if reason.contains("no history row")),
            "{class:?}"
        );
    }

    #[test]
    fn a_wrong_actor_history_row_is_not_proof() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by/4.0/legalcode");
        let before = Work::from_id(&pool, &entry.work_id).unwrap();
        // A different actor writes a history row with the right pre-state,
        // then the licence reaches the target by that other route.
        insert_history_row(
            &pool,
            entry.work_id,
            "someone-else",
            serde_json::Value::String(serde_json::to_string(&before).unwrap()),
        );
        set_licence(&pool, entry.work_id, Some(&entry.to));
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(matches!(class, EntryClassification::Drift(_)), "{class:?}");
    }

    #[test]
    fn a_plan_actor_row_with_the_wrong_historic_licence_is_drift() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc/4.0/deed.en");
        set_licence(&pool, entry.work_id, Some(&entry.to));
        insert_history_row(
            &pool,
            entry.work_id,
            &actor,
            serde_json::json!(format!(
                "{{\"license\":\"https://example.com/other\",\"updatedAt\":\"{}\"}}",
                "2026-01-02T03:04:05.123456Z"
            )),
        );
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(
            matches!(class, EntryClassification::Drift(ref reason) if reason.contains("proves")),
            "{class:?}"
        );
    }

    #[test]
    fn a_plan_actor_row_with_the_wrong_historic_timestamp_is_drift() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc/4.0/legalcode");
        set_licence(&pool, entry.work_id, Some(&entry.to));
        // Right licence, wrong (sub-second-shifted) historic token.
        insert_history_row(
            &pool,
            entry.work_id,
            &actor,
            serde_json::json!(format!(
                "{{\"license\":\"{}\",\"updatedAt\":\"2001-01-01T00:00:00.000001Z\"}}",
                entry.from
            )),
        );
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(matches!(class, EntryClassification::Drift(_)), "{class:?}");
    }

    #[test]
    fn unusable_plan_actor_history_evidence_fails_closed() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nd/4.0/legalcode");
        set_licence(&pool, entry.work_id, Some(&entry.to));
        // The actor row's payload carries no extractable license/updatedAt.
        insert_history_row(
            &pool,
            entry.work_id,
            &actor,
            serde_json::json!("{\"someLegacyShape\":true}"),
        );
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(
            matches!(class, EntryClassification::Drift(ref reason) if reason.contains("unusable")),
            "{class:?}"
        );
    }

    #[test]
    fn historic_payload_schema_drift_is_tolerated_for_the_required_scalars() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc-nd/4.0/deed.en");
        let reviewed = entry.reviewed_updated_at;
        set_licence(&pool, entry.work_id, Some(&entry.to));
        // A payload that does NOT deserialize into today's complete Work —
        // unknown legacy fields, missing modern fields — but carries the two
        // required scalars. Classification must still succeed.
        let rendered_token = serde_json::to_string(&reviewed).expect("timestamp serializes");
        let rendered_token = rendered_token.trim_matches('"');
        insert_history_row(
            &pool,
            entry.work_id,
            &actor,
            serde_json::json!(format!(
                "{{\"license\":\"{}\",\"updatedAt\":\"{rendered_token}\",\"aFieldTheCurrentWorkDoesNotHave\":42}}",
                entry.from
            )),
        );
        assert_eq!(
            classify_entry(&pool, &entry, &actor).unwrap(),
            EntryClassification::AlreadyAppliedByThisPlan
        );
        // Legacy rows from other actors with arbitrary payloads never break
        // classification: they are filtered out before parsing.
        insert_history_row(
            &pool,
            entry.work_id,
            "legacy-user",
            serde_json::json!(["not", "even", "an", "object"]),
        );
        assert_eq!(
            classify_entry(&pool, &entry, &actor).unwrap(),
            EntryClassification::AlreadyAppliedByThisPlan
        );
    }

    #[test]
    fn a_changed_source_licence_is_drift() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-sa/4.0/deed.it");
        set_licence(
            &pool,
            entry.work_id,
            Some("https://creativecommons.org/licenses/by/4.0/"),
        );
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(matches!(class, EntryClassification::Drift(_)), "{class:?}");
    }

    #[test]
    fn a_sub_second_token_difference_is_drift() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc-nd/3.0/legalcode");
        bump_updated_at_one_microsecond(&pool, entry.work_id);
        let class = classify_entry(&pool, &entry, &actor).unwrap();
        assert!(
            matches!(class, EntryClassification::Drift(ref reason) if reason.contains("updated_at")),
            "{class:?}"
        );
    }

    #[test]
    fn a_deleted_planned_work_is_drift_and_stops_the_apply() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_row = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        Work::from_id(&pool, &work_row.work_id)
            .unwrap()
            .delete(&pool)
            .expect("delete work");
        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("a deleted planned work stops the apply");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationDrift(ref message)
                if message.contains("no longer exists")
        ));
    }

    #[test]
    fn drift_stops_the_whole_apply_before_the_first_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_a = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.en",
        );
        let work_b = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.de",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        // Drift the second work only.
        bump_updated_at_one_microsecond(&pool, work_b.work_id);
        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("any drift stops the run");
        assert!(matches!(error, ThothError::LicenceNormalizationDrift(_)));
        // No write happened at all — including to the undrifted first work.
        let a_now = Work::from_id(&pool, &work_a.work_id).unwrap();
        assert_eq!(a_now.license, work_a.license);
        assert_eq!(a_now.updated_at, work_a.updated_at);
        assert_eq!(history_count(&pool, work_a.work_id), 0);
    }

    #[test]
    fn a_new_unplanned_work_with_a_source_value_stops_the_apply() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let planned = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc/4.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        // A new in-scope work with a deterministic source value appears after
        // the review.
        let unplanned = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc/4.0/legalcode",
        );
        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("an unplanned matching work stops the run");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationUnplannedWork(ref message)
                if message.contains(&unplanned.work_id.to_string())
        ));
        // Nothing was written, not even the planned work.
        let planned_now = Work::from_id(&pool, &planned.work_id).unwrap();
        assert_eq!(planned_now.license, planned.license);
        assert_eq!(history_count(&pool, planned.work_id), 0);
    }

    // -----------------------------------------------------------------
    // Transactionality, locking and concurrency
    // -----------------------------------------------------------------

    #[test]
    fn the_history_row_and_licence_update_roll_back_together() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc-nd/2.0/deed.es");
        let before = Work::from_id(&pool, &entry.work_id).unwrap();
        // Run the real single-work write inside an outer transaction that is
        // then aborted: both effects must disappear together, proving they
        // share one transaction.
        let mut connection = pool.get().unwrap();
        let result: Result<(), ThothError> = connection.transaction(|connection| {
            apply_entry(connection, &entry, &actor)?;
            Err(ThothError::InternalError("forced abort".to_string()))
        });
        assert!(result.is_err());
        drop(connection);
        let after = Work::from_id(&pool, &entry.work_id).unwrap();
        assert_eq!(
            after.license, before.license,
            "the licence write rolled back"
        );
        assert_eq!(after.updated_at, before.updated_at);
        assert_eq!(
            history_count(&pool, entry.work_id),
            0,
            "the history insert rolled back with it"
        );
    }

    #[test]
    fn late_drift_under_the_row_lock_fails_closed_without_writing() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc-sa/4.0/deed.it");
        // Classification said PENDING earlier; the row then changes before the
        // write transaction runs. The under-lock recheck must catch it.
        assert_eq!(
            classify_entry(&pool, &entry, &actor).unwrap(),
            EntryClassification::Pending
        );
        set_licence(
            &pool,
            entry.work_id,
            Some("https://creativecommons.org/licenses/by/4.0/"),
        );
        let mut connection = pool.get().unwrap();
        let error =
            apply_entry(&mut connection, &entry, &actor).expect_err("late drift fails closed");
        assert!(matches!(error, ThothError::LicenceNormalizationDrift(_)));
        drop(connection);
        assert_eq!(history_count(&pool, entry.work_id), 0);
        let now = Work::from_id(&pool, &entry.work_id).unwrap();
        assert_eq!(
            now.license.as_deref(),
            Some("https://creativecommons.org/licenses/by/4.0/")
        );
    }

    #[test]
    fn a_concurrent_change_holding_the_row_lock_cannot_be_overwritten() {
        let (_guard, pool, entry, actor) =
            classified_fixture("https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.nl");
        let (locked_tx, locked_rx) = std::sync::mpsc::channel::<()>();
        let concurrent_pool = pool.clone();
        let concurrent_work = entry.work_id;
        // A concurrent writer locks the row, holds the lock briefly, changes
        // the licence and commits.
        let concurrent = std::thread::spawn(move || {
            let mut connection = concurrent_pool.get().unwrap();
            connection
                .transaction::<_, ThothError, _>(|connection| {
                    let _locked: Work = work::table
                        .find(concurrent_work)
                        .for_update()
                        .first(connection)?;
                    locked_tx.send(()).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    diesel::update(work::table.find(concurrent_work))
                        .set(work::license.eq("https://example.com/concurrently-changed"))
                        .execute(connection)?;
                    Ok(())
                })
                .unwrap();
        });
        locked_rx.recv().unwrap();
        // The normalization write must wait on the lock, re-check, and refuse
        // to overwrite the concurrent change.
        let mut connection = pool.get().unwrap();
        let error = apply_entry(&mut connection, &entry, &actor)
            .expect_err("the concurrent change must not be overwritten");
        assert!(matches!(error, ThothError::LicenceNormalizationDrift(_)));
        drop(connection);
        concurrent.join().unwrap();
        let now = Work::from_id(&pool, &entry.work_id).unwrap();
        assert_eq!(
            now.license.as_deref(),
            Some("https://example.com/concurrently-changed")
        );
        assert_eq!(history_count(&pool, entry.work_id), 0);
    }

    // -----------------------------------------------------------------
    // Post-write reconciliation (SR-1 / SR-2)
    // -----------------------------------------------------------------

    /// A complete insertable Work row carrying the given licence, for
    /// injection inside another connection's open transaction.
    fn new_source_work_values(imprint_id: Uuid, licence: &str) -> crate::model::work::NewWork {
        use crate::model::work::{NewWork, WorkStatus, WorkType};
        NewWork {
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition: Some(1),
            imprint_id,
            doi: None,
            publication_date: None,
            withdrawn_date: None,
            place: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: Some(licence.to_string()),
            copyright_holder: None,
            landing_page: None,
            lccn: None,
            oclc: None,
            general_note: None,
            bibliography_note: None,
            toc: None,
            resources_description: None,
            cover_url: None,
            cover_caption: None,
            first_page: None,
            last_page: None,
            page_interval: None,
        }
    }

    /// The reported count for one exact value in a `ValueCount` vector.
    fn count_for(counts: &[ValueCount], value: &str) -> i64 {
        counts
            .iter()
            .find(|entry| entry.value == value)
            .unwrap_or_else(|| panic!("value {value:?} missing from the count vector"))
            .works
    }

    #[test]
    fn a_source_work_appearing_during_the_write_loop_prevents_successful_reconciliation() {
        // SR-1: the concurrency interval between the pre-write membership
        // check and the completion of the per-Work write transactions. The
        // ordering is enforced by locks, not sleeps: the injector holds the
        // second planned Work's row lock, waits until the first planned write
        // has committed (so the apply is provably past its pre-write checks),
        // then commits a brand-new deterministic-source Work while releasing
        // the lock. The apply finishes its writes and must then fail closed
        // on the post-write residual instead of reporting success.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.en",
        );
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.de",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        assert_eq!(outcome.plan.entries.len(), 2);
        let first = outcome.plan.entries[0].clone();
        let second = outcome.plan.entries[1].clone();

        let (locked_tx, locked_rx) = std::sync::mpsc::channel::<()>();
        let injector_pool = pool.clone();
        let injector_imprint = imprint.imprint_id;
        let injected_source = "https://creativecommons.org/licenses/by-nc/4.0/deed.en";
        let injector = std::thread::spawn(move || -> Uuid {
            let mut connection = injector_pool.get().unwrap();
            connection
                .transaction::<Uuid, ThothError, _>(|connection| {
                    let _locked: Work = work::table
                        .find(second.work_id)
                        .for_update()
                        .first(connection)?;
                    locked_tx.send(()).unwrap();
                    // READ COMMITTED sees fresh commits per statement: wait for
                    // the apply's first write transaction to commit, proving
                    // the write loop is past the pre-write membership check.
                    for _ in 0..3000 {
                        let current: Work = work::table.find(first.work_id).first(connection)?;
                        if current.license.as_deref() == Some(first.to.as_str()) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    let injected: Work = diesel::insert_into(work::table)
                        .values(&new_source_work_values(injector_imprint, injected_source))
                        .get_result(connection)?;
                    Ok(injected.work_id)
                })
                .unwrap()
        });
        locked_rx.recv().unwrap();

        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("a post-write deterministic-source residual must fail closed");
        let injected_work_id = injector.join().unwrap();
        assert!(
            matches!(
                error,
                ThothError::LicenceNormalizationDrift(ref message)
                    if message.contains("after the write loop")
                        && message.contains(&injected_work_id.to_string())
            ),
            "{error}"
        );

        // Previously committed planned Works remain committed, with their
        // history evidence; no cross-Work rollback happened.
        for entry in [&outcome.plan.entries[0], &outcome.plan.entries[1]] {
            let now = Work::from_id(&pool, &entry.work_id).unwrap();
            assert_eq!(now.license.as_deref(), Some(entry.to.as_str()));
            assert_eq!(history_count(&pool, entry.work_id), 1);
        }
        // The injected Work is untouched: detected, never normalized.
        let injected_now = Work::from_id(&pool, &injected_work_id).unwrap();
        assert_eq!(injected_now.license.as_deref(), Some(injected_source));
        assert_eq!(history_count(&pool, injected_work_id), 0);
    }

    #[test]
    fn resulting_target_counts_are_actual_catalogue_state_not_plan_deltas() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let target = "https://creativecommons.org/licenses/by-nc-sa/4.0/";
        // An already-canonical in-scope Work carrying the reviewed target...
        let already_canonical = create_licensed_work(&pool, &imprint, target);
        // ...and a planned Work that will normalize into the same target.
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc-sa/4.0/deed.en",
        );
        // A Work under a publisher outside the bound MIG-01 scope carrying the
        // same target: not a deterministic source, so it blocks nothing, but
        // it must not be counted in the scoped resulting evidence.
        let outside = create_publisher(&pool);
        let outside_imprint = create_imprint(&pool, &outside);
        create_licensed_work(&pool, &outside_imprint, target);

        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        let applied = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("apply");

        // The reviewed plan delta for the target is 1 (the one conversion)...
        assert_eq!(count_for(&applied.report.target_value_counts, target), 1);
        // ...while the resulting count is actual post-write catalogue state:
        // the converted Work AND the pre-existing canonical Work, in scope.
        let resulting = applied
            .report
            .resulting_target_value_counts
            .as_ref()
            .expect("a successful apply carries resulting target counts");
        assert_eq!(resulting.len(), DETERMINISTIC_TARGET_COUNT);
        assert_eq!(count_for(resulting, target), 2);
        assert_eq!(
            applied.report.deterministic_source_works_remaining,
            Some(0),
            "the post-write residual query must be reported as zero"
        );
        // The pre-existing canonical Work was never written.
        let untouched = Work::from_id(&pool, &already_canonical.work_id).unwrap();
        assert_eq!(untouched.updated_at, already_canonical.updated_at);
        assert_eq!(history_count(&pool, already_canonical.work_id), 0);
    }

    #[test]
    fn apply_manual_counts_are_one_current_snapshot_not_the_dry_run_aggregates() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nd/4.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        // At review time no manual value is present in the catalogue.
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);
        assert_eq!(outcome.plan.expected.manual_unresolved_value_count, 0);
        assert_eq!(outcome.plan.expected.manual_unresolved_work_count, 0);

        // Manual-resolution state then changes independently, without touching
        // any deterministic plan entry.
        let mark = "https://creativecommons.org/publicdomain/mark/1.0/";
        let typo = "https://creativecommons.org/licences/by-nc-nc/4.0/";
        let manual_one = create_licensed_work(&pool, &imprint, mark);
        let manual_two = create_licensed_work(&pool, &imprint, mark);
        let manual_three = create_licensed_work(&pool, &imprint, typo);
        // A manual value under an out-of-scope publisher is outside the bound
        // MIG-01 reporting scope and must not be counted.
        let outside = create_publisher(&pool);
        let outside_imprint = create_imprint(&pool, &outside);
        create_licensed_work(&pool, &outside_imprint, mark);

        let applied = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Disposable,
        )
        .expect("apply");
        let report = &applied.report;

        // Detailed per-value counts reflect the current post-write snapshot.
        assert_eq!(report.manual_unresolved_values.len(), MANUAL_VALUE_COUNT);
        assert_eq!(count_for(&report.manual_unresolved_values, mark), 2);
        assert_eq!(count_for(&report.manual_unresolved_values, typo), 1);
        // Aggregates agree with — and are derived from — that same vector.
        assert_eq!(report.manual_unresolved_value_count, 2);
        assert_eq!(report.manual_unresolved_work_count, 3);
        assert_eq!(
            report.manual_unresolved_value_count,
            manual_distinct_value_count(&report.manual_unresolved_values)
        );
        assert_eq!(
            report.manual_unresolved_work_count,
            manual_work_count(&report.manual_unresolved_values)
        );
        // They are decoupled from the stale dry-run aggregates.
        assert_ne!(
            report.manual_unresolved_work_count,
            outcome.plan.expected.manual_unresolved_work_count
        );
        // No manual value was written by LIC-NORM-01.
        for manual in [&manual_one, &manual_two, &manual_three] {
            let now = Work::from_id(&pool, &manual.work_id).unwrap();
            assert_eq!(now.license, manual.license);
            assert_eq!(now.updated_at, manual.updated_at);
            assert_eq!(history_count(&pool, manual.work_id), 0);
        }
    }

    #[test]
    fn an_apply_with_the_wrong_plan_hash_is_rejected() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/legalcode.de",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (_, plan_bytes, _, _) = run_dry_run(&pool, &files);
        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            "0000000000000000000000000000000000000000000000000000000000000000",
            ApplyExecutionMode::Disposable,
        )
        .expect_err("a plan hash mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("reviewed plan")
        ));
    }

    #[test]
    fn an_apply_whose_plan_binds_different_input_hashes_is_rejected() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by/4.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, _, _, _) = run_dry_run(&pool, &files);
        // A canonical plan whose recorded MIG-01 hash differs from the
        // supplied verified input.
        let mut tampered = outcome.plan.clone();
        tampered.mig01_manifest_sha256 = AUTHORIZED_MIG01_SHA256.to_string();
        let tampered_bytes = canonical_plan_bytes(&tampered).unwrap();
        let tampered_sha = sha256_hex(&tampered_bytes);
        let error = apply_plan(
            &pool,
            &files,
            &tampered_bytes,
            &tampered_sha,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("an input-hash binding mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("MIG-01 manifest hash")
        ));
    }

    #[test]
    fn a_plan_entry_outside_the_reviewed_rules_is_rejected_before_any_write() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let manual_work = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/publicdomain/mark/1.0/",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, _, _, _) = run_dry_run(&pool, &files);
        // A hand-crafted canonical plan that tries to make a manual value
        // executable. Its hash is self-consistent, so only the reviewed-rule
        // membership check can stop it — and it must.
        let manual_state = Work::from_id(&pool, &manual_work.work_id).unwrap();
        let mut forged = outcome.plan.clone();
        forged.entries = vec![NormalizationPlanEntry {
            work_id: manual_work.work_id,
            publisher_id: publisher.publisher_id,
            reviewed_updated_at: manual_state.updated_at,
            from: "https://creativecommons.org/publicdomain/mark/1.0/".to_string(),
            to: "https://creativecommons.org/publicdomain/zero/1.0/".to_string(),
        }];
        let forged_bytes = canonical_plan_bytes(&forged).unwrap();
        let forged_sha = sha256_hex(&forged_bytes);
        let error = apply_plan(
            &pool,
            &files,
            &forged_bytes,
            &forged_sha,
            ApplyExecutionMode::Disposable,
        )
        .expect_err("a manual value must never be executable");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("not one of the reviewed deterministic rules")
        ));
        let untouched = Work::from_id(&pool, &manual_work.work_id).unwrap();
        assert_eq!(
            untouched.license.as_deref(),
            Some("https://creativecommons.org/publicdomain/mark/1.0/")
        );
    }

    // -----------------------------------------------------------------
    // Production/disposable execution modes
    // -----------------------------------------------------------------

    #[test]
    fn production_mode_requires_the_exact_reviewed_dry_run_report() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc-nd/4.0/deed.it",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, report_path, report_sha) = run_dry_run(&pool, &files);

        // The wrong expected report hash fails closed before any write.
        let error = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Production {
                reviewed_report_path: &report_path,
                expected_reviewed_report_sha256:
                    "0000000000000000000000000000000000000000000000000000000000000000",
            },
        )
        .expect_err("a report hash mismatch must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("dry-run report")
        ));

        // The exact reviewed report verifies and the apply proceeds.
        let applied = apply_plan(
            &pool,
            &files,
            &plan_bytes,
            &outcome.plan_sha256,
            ApplyExecutionMode::Production {
                reviewed_report_path: &report_path,
                expected_reviewed_report_sha256: &report_sha,
            },
        )
        .expect("production-mode apply with exact reviewed evidence");
        assert_eq!(applied.written, 1);
    }

    #[test]
    fn an_apply_report_is_never_accepted_as_reviewed_dry_run_evidence() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_row = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nc-sa/4.0/deed.en",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (outcome, plan_bytes, _, _) = run_dry_run(&pool, &files);

        // Produce an APPLY report, then restore the work and try to use that
        // report as production evidence for a fresh plan.
        let plan_path = write_artifact(&plan_bytes, "plan.json");
        let apply_report_out = tmp_path("apply-report.json");
        let request = ApplyRequest {
            deterministic_manifest_path: &files.deterministic,
            expected_deterministic_manifest_sha256: &files.deterministic_sha256,
            manual_register_path: &files.manual,
            expected_manual_register_sha256: &files.manual_sha256,
            mig01_manifest_path: &files.mig01,
            expected_mig01_manifest_sha256: &files.mig01_sha256,
            plan_path: &plan_path,
            expected_plan_sha256: &outcome.plan_sha256,
            report_out_path: &apply_report_out,
            mode: ApplyExecutionMode::Disposable,
        };
        apply(&pool, &request).expect("disposable apply");
        let apply_report_sha = sha256_hex(&std::fs::read(&apply_report_out).unwrap());

        // Restore the pre-apply state so a fresh dry-run plan exists.
        set_licence(&pool, work_row.work_id, Some(&outcome.plan.entries[0].from));
        let fresh = {
            let (fresh_outcome, fresh_plan_bytes, _, _) = run_dry_run(&pool, &files);
            (fresh_outcome, fresh_plan_bytes)
        };
        let error = apply_plan(
            &pool,
            &files,
            &fresh.1,
            &fresh.0.plan_sha256,
            ApplyExecutionMode::Production {
                reviewed_report_path: &apply_report_out,
                expected_reviewed_report_sha256: &apply_report_sha,
            },
        )
        .expect_err("an APPLY report is not Gate-reviewed dry-run evidence");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationInvalidInput(ref message)
                if message.contains("DRY_RUN")
        ));
    }

    #[test]
    fn a_reviewed_report_bound_to_a_different_plan_is_rejected() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let work_row = create_licensed_work(
            &pool,
            &imprint,
            "https://creativecommons.org/licenses/by-nd/4.0/legalcode",
        );
        let files = input_files(&[publisher.publisher_id]);
        let (_, _, first_report_path, first_report_sha) = run_dry_run(&pool, &files);
        // Change the database state so a fresh dry run yields a different plan
        // (and therefore a different plan hash) from the first report.
        bump_updated_at_one_microsecond(&pool, work_row.work_id);
        let (second_outcome, second_plan_bytes, _, _) = run_dry_run(&pool, &files);
        let error = apply_plan(
            &pool,
            &files,
            &second_plan_bytes,
            &second_outcome.plan_sha256,
            ApplyExecutionMode::Production {
                reviewed_report_path: &first_report_path,
                expected_reviewed_report_sha256: &first_report_sha,
            },
        )
        .expect_err("stale reviewed evidence must fail closed");
        assert!(matches!(
            error,
            ThothError::LicenceNormalizationHashMismatch(ref message)
                if message.contains("different reviewed plan hash")
        ));
    }
}
