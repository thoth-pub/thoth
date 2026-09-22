//! Canonical Metrics ingestion coordinator (`MET-WP2-01B`).
//!
//! This module owns the repository-internal transaction that turns one bounded
//! batch of `thoth-normalized-metrics/1` observations and coverage assertions
//! into durable canonical Metrics state. It is the **MOM-1 managed `DRIVER`
//! slice** of `MET-WP2-01` (#898): one publisher-scoped managed DRIVER source
//! account, validated and resolved entirely inside Thoth, with first-arrival,
//! duplicate, managed-revision and conflict outcomes, deterministic canonical
//! hashing, transactional overlap prevention, durable batch idempotency,
//! provenance for every row, import accounting, explicit coverage evidence
//! and `PENDING` rollup-delta creation.
//!
//! It sits **below** GraphQL and service authorization: it exposes no API,
//! makes no authentication or entitlement decision about the *caller*, and
//! enforces only the *domain data* authority the approved design assigns to
//! Thoth. `PUBLISHER_UPLOAD`, `OPERAS` and `ADMIN_IMPORT` ingestion, retraction,
//! administrative repair, import lifecycle APIs, rollup-delta application and
//! metrics query serving are deliberately absent and fail closed where they
//! could otherwise be reached.
//!
//! # Entry point
//!
//! [`ingest_metric_batch`] is the single write entry point. A batch is
//! identified by `(import_id, batch_key)`; its payload is identified by the
//! deterministic request hash in [`hash`]. The persisted
//! [`metric_import_batch`](crate::model::metric_import_batch) row is the retry
//! boundary: replaying a committed batch returns its committed per-row
//! outcomes from provenance without repeating any write, and reusing a key
//! with a different payload is refused.
//!
//! # Transaction protocol
//!
//! Every first-time batch runs in one PostgreSQL transaction on one
//! connection, in this normative lock order (Amendments 1, 3 and 4):
//!
//! 1. `metric_import` `FOR UPDATE`, then the `(import_id, batch_key)`
//!    idempotency check;
//! 2. `metric_source_account`, `metric_source`, `metric_platform` and the
//!    expected `publisher`, each `FOR SHARE`;
//! 3. every referenced `metric_measure` `FOR SHARE`, ascending `measure_id`,
//!    then every referenced `metric_platform_measure` `FOR SHARE`, ascending
//!    `platform_measure_id`;
//! 4. provisional metadata discovery by ordinary reads only;
//! 5. `imprint`, `publication` and `institution` rows required by the
//!    provisionally accepted observations, each `FOR SHARE` in ascending id
//!    order, then the required `work` rows `FOR UPDATE` in ascending id order
//!    — parents strictly before works, because the existing publisher, imprint,
//!    publication and institution triggers write `work` rows;
//! 6. the locked re-resolution barrier: every observation is re-resolved under
//!    those locks and nothing is accepted using a row that was not locked in
//!    step 5. Drift rolls the whole attempt back; a call makes at most three
//!    attempts before failing as `CONCURRENT_AUTHORITY_CHANGE`;
//! 7. `pg_advisory_xact_lock` on every distinct dimensional cell, ascending
//!    signed key;
//! 8. existing `metric_record` / `metric_record_revision` rows `FOR UPDATE`,
//!    only after their cell lock, and the same-cell half-open overlap lookup;
//! 9. batch row, provenance, import errors, unresolved-DOI quarantine,
//!    canonical writes, rollup deltas, coverage and counters, then commit.
//!
//! Row-level validation failures do not fail the batch: they become
//! `REJECTED` provenance plus one sanitized `metric_import_error`. Request-level
//! failures, including any invalid coverage assertion and any database
//! failure, commit nothing.
//!
//! # Unresolved-DOI quarantine (`MET-WP7-PREREQ-02`)
//!
//! An observation that passed every input check, whose DOI therefore parsed,
//! but that resolved to no work at the locked re-resolution barrier is
//! `REJECTED` / `UNKNOWN_DOI` exactly as before. When the locked source's
//! `driver_key` is exactly
//! [`CLOUDFRONT_DRIVER_KEY`](crate::model::metric_source_account::CLOUDFRONT_DRIVER_KEY)
//! and the observation carries none of `publication_isbn`, `publication_type`,
//! `institution_ror`, `source_record_id` and `source_row_number`, the same
//! transaction also writes exactly one
//! [`metric_identifier_quarantine`](crate::model::metric_identifier_quarantine)
//! row linked to that rejected provenance. Quarantine is not acceptance: the
//! row's outcome, import error and invalid counter are unchanged, and nothing
//! canonical is written. Eligibility is decided only from the locked source
//! row and the observation itself; no caller flag, source or account code,
//! hostname or routing value participates. A replay writes nothing, so it
//! never creates a second quarantine row.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::{sql_query, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_import::{MetricImport, MetricImportStatus};
use crate::model::metric_import_batch::MetricImportBatch;
use crate::model::metric_import_error::MetricImportErrorSeverity;
use crate::model::metric_measure::MetricMeasure;
use crate::model::metric_platform::MetricPlatform;
use crate::model::metric_platform_measure::{MetricPlatformMeasure, MetricReportingGrain};
use crate::model::metric_record::MetricRecord;
use crate::model::metric_record_provenance::{
    MetricRecordProvenance, MetricRecordProvenanceClassification,
};
use crate::model::metric_record_revision::{MetricRecordRevision, MetricRecordRevisionStatus};
use crate::model::metric_source::{MetricSource, MetricSourceAcquisitionType};
use crate::model::metric_source_account::{MetricSourceAccount, CLOUDFRONT_DRIVER_KEY};
use crate::model::publication::PublicationType;
use crate::model::publisher::{PublisherCapability, ThothPackage};
use crate::model::{Doi, Isbn, Ror, Timestamp, ROR_DOMAIN};
use crate::schema::{
    imprint, institution, metric_coverage, metric_identifier_quarantine, metric_import,
    metric_import_batch, metric_import_error, metric_measure, metric_platform,
    metric_platform_measure, metric_record, metric_record_provenance, metric_record_revision,
    metric_rollup_delta, metric_source, metric_source_account, publication, publisher, work,
};

pub mod country;
pub mod error;
pub mod hash;
#[cfg(test)]
pub(crate) mod tests;

pub use error::{MetricIngestionError, MetricIngestionErrorCode};
use hash::{
    cell_lock_key, content_hash, enum_code, identity_hash, request_hash, CanonicalIdentity,
};
use MetricIngestionErrorCode as Code;

/// The only normalized schema version this coordinator accepts.
pub const SUPPORTED_SCHEMA_VERSION: &str = "thoth-normalized-metrics/1";
/// Maximum observations in one batch (Amendment 3 B1).
pub const MAX_OBSERVATIONS: usize = 500;
/// Maximum coverage assertions in one batch (Amendment 3 B1).
pub const MAX_COVERAGE_ASSERTIONS: usize = 100;
/// Maximum `batch_key` length in UTF-8 bytes (Amendment 3 B1).
pub const MAX_BATCH_KEY_BYTES: usize = 256;
/// Maximum first-time transaction attempts per call (Amendment 4 C6).
pub const MAX_ATTEMPTS: u8 = 3;
/// The fixed `metric_record_provenance.details` schema written for every row.
pub const PROVENANCE_DETAILS_SCHEMA: &str = "thoth-metric-provenance-details/1";
/// The rollup-delta status created by this coordinator. Application is WP4.
pub const ROLLUP_DELTA_PENDING: &str = "PENDING";

/// One bounded ingestion request.
///
/// `schema_version` appears exactly once, on this envelope. `observations`
/// and `coverage` are ordered: observation order fixes `batch_row_index` and
/// participates in the request hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricIngestionBatch {
    pub import_id: Uuid,
    pub batch_key: String,
    pub schema_version: String,
    pub observations: Vec<NormalizedMetricObservation>,
    pub coverage: Vec<NormalizedMetricCoverageAssertion>,
}

/// One `thoth-normalized-metrics/1` observation as presented to the
/// coordinator.
///
/// Stable codes and raw DOI/ISBN/ROR/country strings are carried exactly as
/// supplied; resolution to canonical Thoth identities happens inside the
/// transaction. `publication_type` reuses the repository's `PublicationType`.
/// `methodology_version` is required per observation because it participates
/// in canonical content hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedMetricObservation {
    pub source_account_code: String,
    pub platform_code: String,
    pub measure_code: String,
    pub work_doi: String,
    pub publication_isbn: Option<String>,
    pub publication_type: Option<PublicationType>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub reporting_grain: MetricReportingGrain,
    pub country_code: Option<String>,
    pub institution_ror: Option<String>,
    pub value: i64,
    pub source_record_id: Option<String>,
    pub methodology_version: String,
    pub source_row_number: Option<i64>,
}

/// One explicit coverage assertion.
///
/// The owning import supplies `source_account_id` and `import_id`; callers
/// never supply canonical UUID authority. Coverage is immutable import-scoped
/// evidence and is never inferred from accepted observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedMetricCoverageAssertion {
    pub platform_code: String,
    pub measure_code: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub status: MetricCoverageStatus,
    pub country_coverage: bool,
    pub institution_coverage: bool,
    pub notes: Option<String>,
}

/// The committed (or replayed) outcome of one batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricIngestionOutcome {
    pub import_batch_id: Uuid,
    pub request_hash: String,
    /// `true` when the outcome was read back from an already committed batch
    /// rather than produced by this call.
    pub replayed: bool,
    /// One entry per submitted observation, in `batch_row_index` order.
    pub rows: Vec<MetricIngestionRowOutcome>,
}

/// The durable classification of one observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricIngestionRowOutcome {
    pub batch_row_index: i64,
    pub classification: MetricRecordProvenanceClassification,
    /// `None` for `WINNER`, `DUPLICATE` and `REVISION`; the stable reason for
    /// `CONFLICT` and `REJECTED`.
    pub reason_code: Option<MetricIngestionErrorCode>,
    pub record_id: Option<Uuid>,
    pub identity_hash: Option<String>,
    pub content_hash: Option<String>,
}

/// Ingest one bounded batch under the protocol described in the module
/// documentation.
///
/// Request-shape failures are detected before any database access. Every
/// other request-level failure aborts the transaction and commits nothing. A
/// database failure is reduced to `INTERNAL_DATABASE_ERROR`.
pub fn ingest_metric_batch(
    db: &PgPool,
    batch: &MetricIngestionBatch,
) -> Result<MetricIngestionOutcome, MetricIngestionError> {
    validate_request_shape(batch)?;
    let request_hash = request_hash(batch);

    for _attempt in 0..MAX_ATTEMPTS {
        let mut connection = db.get().map_err(|error| {
            log::error!("metric ingestion could not obtain a database connection: {error}");
            MetricIngestionError::new(Code::InternalDatabaseError)
        })?;
        let attempt =
            connection.transaction(|connection| run_attempt(connection, batch, &request_hash));
        match attempt {
            Ok(outcome) => return Ok(outcome),
            Err(AttemptError::Drift) => continue,
            Err(AttemptError::Request(code)) => return Err(MetricIngestionError::new(code)),
            Err(AttemptError::Database(error)) => {
                log::error!("metric ingestion database failure: {error}");
                return Err(MetricIngestionError::new(Code::InternalDatabaseError));
            }
        }
    }
    Err(MetricIngestionError::new(Code::ConcurrentAuthorityChange))
}

/// Request-shape validation performed before any database access.
fn validate_request_shape(batch: &MetricIngestionBatch) -> Result<(), MetricIngestionError> {
    if batch.batch_key.trim().is_empty() || batch.batch_key.len() > MAX_BATCH_KEY_BYTES {
        return Err(MetricIngestionError::new(Code::InvalidBatchKey));
    }
    if batch.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(MetricIngestionError::new(Code::UnsupportedSchemaVersion));
    }
    if batch.observations.is_empty() && batch.coverage.is_empty() {
        return Err(MetricIngestionError::new(Code::EmptyBatch));
    }
    if batch.observations.len() > MAX_OBSERVATIONS || batch.coverage.len() > MAX_COVERAGE_ASSERTIONS
    {
        return Err(MetricIngestionError::new(Code::BatchLimitExceeded));
    }
    Ok(())
}

/// Why one transaction attempt did not commit.
#[derive(Debug)]
enum AttemptError {
    /// Locked re-resolution needed an authority row that was not locked in
    /// its prescribed phase: roll back and restart discovery.
    Drift,
    /// A request-level failure: roll back and return the code.
    Request(MetricIngestionErrorCode),
    /// A database failure: roll back and return `INTERNAL_DATABASE_ERROR`.
    Database(DieselError),
}

impl From<DieselError> for AttemptError {
    fn from(error: DieselError) -> Self {
        AttemptError::Database(error)
    }
}

impl From<MetricIngestionErrorCode> for AttemptError {
    fn from(code: MetricIngestionErrorCode) -> Self {
        AttemptError::Request(code)
    }
}

/// The request-level authority snapshot, held under row locks until commit.
struct RequestScope {
    import: MetricImport,
    account: MetricSourceAccount,
    /// The account's source, retained from its existing `FOR SHARE` lock so
    /// quarantine eligibility is decided from locked canonical authority
    /// without taking any further lock.
    source: MetricSource,
    platform: MetricPlatform,
    /// The expected publisher, equal to both `account.expected_publisher_id`
    /// and `import.publisher_id`.
    publisher_id: Uuid,
}

/// One referenced measure and, when one exists for the account's platform,
/// its mapping — both read under `FOR SHARE`.
struct LockedMeasure {
    measure: MetricMeasure,
    mapping: Option<MetricPlatformMeasure>,
}

/// Locked measures keyed by exact stable measure code.
type Registry = HashMap<String, LockedMeasure>;

/// One coverage assertion resolved and validated against the locked registry.
struct PlannedCoverage<'a> {
    assertion: &'a NormalizedMetricCoverageAssertion,
    measure_id: Uuid,
}

/// One observation that passed input validation, with its parsed identifiers
/// and locked registry references.
struct ValidatedInput<'a> {
    observation: &'a NormalizedMetricObservation,
    measure: &'a MetricMeasure,
    doi: Doi,
    /// The hyphenated ISBN-13 exactly as `publication.isbn` stores it.
    isbn: Option<String>,
    /// The canonical `https://ror.org/<id>` form `institution.ror` stores.
    ror: Option<String>,
}

/// The canonical Thoth identities one observation resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Resolution {
    work_id: Uuid,
    imprint_id: Uuid,
    publication_id: Option<Uuid>,
    institution_id: Option<Uuid>,
}

/// An observation that survived the locked re-resolution barrier.
struct Candidate<'a> {
    input: ValidatedInput<'a>,
    identity: CanonicalIdentity,
    identity_hash: String,
    content_hash: String,
    cell_key: i64,
}

/// The final plan for one observation before canonical processing.
enum RowPlan<'a> {
    Rejected {
        code: MetricIngestionErrorCode,
        identity_hash: Option<String>,
        content_hash: Option<String>,
        /// `Some` only for an eligible CloudFront `UNKNOWN_DOI` rejection.
        quarantine: Option<QuarantinePlan<'a>>,
    },
    Candidate(Box<Candidate<'a>>),
}

/// The canonical references one eligible unresolved-DOI rejection is
/// quarantined under, beyond what its observation and the request scope carry.
struct QuarantinePlan<'a> {
    /// The batch envelope's validated normalized schema version.
    schema_version: &'a str,
    /// The locked measure the observation validated against.
    measure_id: Uuid,
}

/// Import counter increments accumulated over one first-time batch.
#[derive(Default)]
struct Counters {
    received: i64,
    accepted: i64,
    duplicate: i64,
    revision: i64,
    conflict: i64,
    invalid: i64,
}

/// How the shared canonical application authority orders a same-source
/// correction. Ordinary ingestion is already in arrival order; quarantine
/// reconciliation carries historical evidence that may predate the current
/// revision and therefore supplies the original import timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalChronology {
    Arrival,
    Historical { import_created_at: Timestamp },
}

/// The source-independent facts required by the one canonical application
/// authority. Provenance and import counters are deliberately not included:
/// ordinary ingestion owns those ledgers, while quarantine reconciliation must
/// leave them byte-for-byte unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalApplicationCandidate {
    pub identity: CanonicalIdentity,
    pub identity_hash: String,
    pub content_hash: String,
    pub reporting_grain: MetricReportingGrain,
    pub source_account_id: Uuid,
    pub import_id: Uuid,
    pub value: i64,
}

/// Every decision the shared canonical authority can make after the
/// dimensional-cell lock has been acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalApplicationOutcome {
    Winner {
        record_id: Uuid,
        record_revision_id: Uuid,
    },
    Duplicate {
        record_id: Uuid,
        record_revision_id: Uuid,
    },
    Revision {
        record_id: Uuid,
        record_revision_id: Uuid,
    },
    Superseded {
        record_id: Uuid,
        record_revision_id: Uuid,
    },
    SourceConflict,
    OverlappingPeriod,
    SameImportOrder,
    ImportOrderAmbiguous,
    DeltaOverflow,
}

/// Failure of the shared canonical authority itself. It never carries a
/// database detail outside the model layer.
#[derive(Debug)]
pub(crate) enum CanonicalApplicationError {
    Database(DieselError),
    InternalStateInconsistency,
}

impl From<DieselError> for CanonicalApplicationError {
    fn from(error: DieselError) -> Self {
        Self::Database(error)
    }
}

/// One transaction attempt: idempotency check, then either replay or the
/// complete first-time protocol.
fn run_attempt(
    connection: &mut PgConnection,
    batch: &MetricIngestionBatch,
    request_hash: &str,
) -> Result<MetricIngestionOutcome, AttemptError> {
    // Lock 1: the owning import.
    let import: MetricImport = metric_import::table
        .find(batch.import_id)
        .for_update()
        .first(connection)
        .optional()?
        .ok_or(Code::ImportNotFound)?;

    // The durable idempotency boundary, checked on every attempt before any
    // fresh processing.
    let existing: Option<MetricImportBatch> = metric_import_batch::table
        .filter(metric_import_batch::import_id.eq(import.import_id))
        .filter(metric_import_batch::batch_key.eq(&batch.batch_key))
        .first(connection)
        .optional()?;
    if let Some(existing) = existing {
        if existing.request_hash == request_hash {
            return replay(connection, &existing, batch.observations.len());
        }
        return Err(Code::IdempotencyKeyReused.into());
    }

    if import.status != MetricImportStatus::Processing {
        return Err(Code::ImportNotProcessing.into());
    }

    // Locks 2-5: account, source, platform, expected publisher.
    let scope = lock_request_scope(connection, import)?;
    // Locks 6-7: measures and mappings.
    let registry = lock_registry(connection, &scope, batch)?;

    // Coverage has no per-row ledger, so any invalid assertion is a
    // request-level failure decided before observation work begins.
    let planned_coverage = plan_coverage(&scope, &registry, &batch.coverage)?;

    // Input validation needs no database access beyond the locked registry.
    let inputs: Vec<Result<ValidatedInput<'_>, MetricIngestionErrorCode>> = batch
        .observations
        .iter()
        .map(|observation| validate_input(&scope, &registry, observation))
        .collect();

    // Step 8: provisional discovery, ordinary reads only, never authoritative.
    let mut provisional_imprints = BTreeSet::new();
    let mut provisional_publications = BTreeSet::new();
    let mut provisional_institutions = BTreeSet::new();
    let mut provisional_works = BTreeSet::new();
    for input in inputs.iter().flatten() {
        if let Ok(resolution) = resolve(connection, input)? {
            provisional_imprints.insert(resolution.imprint_id);
            provisional_works.insert(resolution.work_id);
            provisional_publications.extend(resolution.publication_id);
            provisional_institutions.extend(resolution.institution_id);
        }
    }

    // Steps 9-12: parents before works, each in ascending id order.
    let locked_imprints = lock_imprints(connection, &provisional_imprints)?;
    let locked_publications = lock_publications(connection, &provisional_publications)?;
    let locked_institutions = lock_institutions(connection, &provisional_institutions)?;
    let locked_works = lock_works(connection, &provisional_works)?;

    // Step 13: the locked re-resolution barrier. No authority lock may be
    // taken from here on.
    let mut rows: Vec<RowPlan<'_>> = Vec::with_capacity(inputs.len());
    for input in inputs {
        let plan = match input {
            Err(code) => RowPlan::Rejected {
                code,
                identity_hash: None,
                content_hash: None,
                quarantine: None,
            },
            Ok(input) => match resolve(connection, &input)? {
                Err(code) => RowPlan::Rejected {
                    code,
                    identity_hash: None,
                    content_hash: None,
                    quarantine: quarantine_eligible(&scope, input.observation, code).then(|| {
                        QuarantinePlan {
                            schema_version: &batch.schema_version,
                            measure_id: input.measure.measure_id,
                        }
                    }),
                },
                Ok(resolution) => {
                    let imprint_publisher = locked_imprints.get(&resolution.imprint_id);
                    let fully_locked = locked_works.contains(&resolution.work_id)
                        && imprint_publisher.is_some()
                        && resolution
                            .publication_id
                            .is_none_or(|id| locked_publications.contains(&id))
                        && resolution
                            .institution_id
                            .is_none_or(|id| locked_institutions.contains(&id));
                    if !fully_locked {
                        return Err(AttemptError::Drift);
                    }
                    let identity = CanonicalIdentity {
                        platform_id: scope.platform.platform_id,
                        measure_id: input.measure.measure_id,
                        work_id: resolution.work_id,
                        publication_id: resolution.publication_id,
                        period_start: input.observation.period_start,
                        period_end: input.observation.period_end,
                        country_code: input.observation.country_code.clone(),
                        institution_id: resolution.institution_id,
                    };
                    let identity_hash = identity_hash(&identity);
                    let content_hash = content_hash(
                        &identity,
                        input.observation.value,
                        &input.observation.methodology_version,
                    );
                    if imprint_publisher != Some(&scope.publisher_id) {
                        RowPlan::Rejected {
                            code: Code::PublisherScopeMismatch,
                            identity_hash: Some(identity_hash),
                            content_hash: Some(content_hash),
                            quarantine: None,
                        }
                    } else {
                        let cell_key = cell_lock_key(&identity);
                        RowPlan::Candidate(Box::new(Candidate {
                            input,
                            identity,
                            identity_hash,
                            content_hash,
                            cell_key,
                        }))
                    }
                }
            },
        };
        rows.push(plan);
    }

    // Step 14: advisory dimensional-cell locks, ascending signed key.
    let cell_keys: BTreeSet<i64> = rows
        .iter()
        .filter_map(|plan| match plan {
            RowPlan::Candidate(candidate) => Some(candidate.cell_key),
            RowPlan::Rejected { .. } => None,
        })
        .collect();
    lock_cell_keys(connection, &cell_keys)?;

    // Step 15 onward: durable writes, in batch_row_index order.
    let import_batch_id: Uuid = diesel::insert_into(metric_import_batch::table)
        .values((
            metric_import_batch::import_id.eq(scope.import.import_id),
            metric_import_batch::batch_key.eq(&batch.batch_key),
            metric_import_batch::request_hash.eq(request_hash),
        ))
        .returning(metric_import_batch::import_batch_id)
        .get_result(connection)?;

    let mut counters = Counters::default();
    let mut outcomes = Vec::with_capacity(rows.len());
    for (index, plan) in rows.into_iter().enumerate() {
        let observation = &batch.observations[index];
        let row_index = index as i64;
        counters.received += 1;
        let outcome = match plan {
            RowPlan::Rejected {
                code,
                identity_hash,
                content_hash,
                quarantine,
            } => write_rejected(
                connection,
                &scope,
                import_batch_id,
                row_index,
                observation,
                code,
                identity_hash,
                content_hash,
                quarantine,
                &mut counters,
            )?,
            RowPlan::Candidate(candidate) => apply_candidate(
                connection,
                &scope,
                import_batch_id,
                row_index,
                &candidate,
                &mut counters,
            )?,
        };
        outcomes.push(outcome);
    }

    for planned in &planned_coverage {
        diesel::insert_into(metric_coverage::table)
            .values((
                metric_coverage::source_account_id.eq(scope.account.source_account_id),
                metric_coverage::import_id.eq(scope.import.import_id),
                metric_coverage::platform_id.eq(scope.platform.platform_id),
                metric_coverage::measure_id.eq(planned.measure_id),
                metric_coverage::period_start.eq(planned.assertion.period_start),
                metric_coverage::period_end.eq(planned.assertion.period_end),
                metric_coverage::coverage_status.eq(planned.assertion.status),
                metric_coverage::country_coverage.eq(planned.assertion.country_coverage),
                metric_coverage::institution_coverage.eq(planned.assertion.institution_coverage),
                metric_coverage::notes.eq(planned.assertion.notes.as_deref()),
            ))
            .execute(connection)?;
    }

    if counters.received > 0 {
        diesel::update(metric_import::table.find(scope.import.import_id))
            .set((
                metric_import::received_count.eq(metric_import::received_count + counters.received),
                metric_import::accepted_count.eq(metric_import::accepted_count + counters.accepted),
                metric_import::duplicate_count
                    .eq(metric_import::duplicate_count + counters.duplicate),
                metric_import::revision_count.eq(metric_import::revision_count + counters.revision),
                metric_import::conflict_count.eq(metric_import::conflict_count + counters.conflict),
                metric_import::invalid_count.eq(metric_import::invalid_count + counters.invalid),
            ))
            .execute(connection)?;
    }

    Ok(MetricIngestionOutcome {
        import_batch_id,
        request_hash: request_hash.to_owned(),
        replayed: false,
        rows: outcomes,
    })
}

/// Return an already committed batch's ordered outcome from its provenance.
///
/// No current configuration is revalidated, the import need not still be
/// `PROCESSING`, and nothing is written. Persisted state that contradicts the
/// submitted cardinality or order fails closed.
fn replay(
    connection: &mut PgConnection,
    existing: &MetricImportBatch,
    expected_rows: usize,
) -> Result<MetricIngestionOutcome, AttemptError> {
    let rows: Vec<MetricRecordProvenance> = metric_record_provenance::table
        .filter(metric_record_provenance::import_id.eq(existing.import_id))
        .filter(metric_record_provenance::import_batch_id.eq(existing.import_batch_id))
        .order(metric_record_provenance::batch_row_index.asc())
        .load(connection)?;
    if rows.len() != expected_rows {
        return Err(Code::InternalStateInconsistency.into());
    }
    let mut outcomes = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        if row.batch_row_index != Some(index as i64) {
            return Err(Code::InternalStateInconsistency.into());
        }
        let reason_code = match row.details.get("reason_code") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(code)) => Some(
                MetricIngestionErrorCode::from_str(code)
                    .map_err(|_| Code::InternalStateInconsistency)?,
            ),
            Some(_) => return Err(Code::InternalStateInconsistency.into()),
        };
        outcomes.push(MetricIngestionRowOutcome {
            batch_row_index: index as i64,
            classification: row.classification,
            reason_code,
            record_id: row.record_id,
            identity_hash: row.identity_hash,
            content_hash: row.content_hash,
        });
    }
    Ok(MetricIngestionOutcome {
        import_batch_id: existing.import_batch_id,
        request_hash: existing.request_hash.clone(),
        replayed: true,
        rows: outcomes,
    })
}

/// Locks 2-5 and the request-level MOM-1 DRIVER authority checks.
fn lock_request_scope(
    connection: &mut PgConnection,
    import: MetricImport,
) -> Result<RequestScope, AttemptError> {
    let account: MetricSourceAccount = metric_source_account::table
        .find(import.source_account_id)
        .for_share()
        .first(connection)
        .optional()?
        .ok_or(Code::InternalStateInconsistency)?;
    if !account.enabled {
        return Err(Code::SourceAccountDisabled.into());
    }

    let source: MetricSource = metric_source::table
        .find(account.source_id)
        .for_share()
        .first(connection)
        .optional()?
        .ok_or(Code::InternalStateInconsistency)?;
    if !source.enabled {
        return Err(Code::SourceDisabled.into());
    }
    if source.acquisition_type != MetricSourceAcquisitionType::Driver {
        return Err(Code::AcquisitionTypeDeferred.into());
    }

    let platform: MetricPlatform = metric_platform::table
        .find(account.platform_id)
        .for_share()
        .first(connection)
        .optional()?
        .ok_or(Code::InternalStateInconsistency)?;

    let publisher_id = match (account.expected_publisher_id, import.publisher_id) {
        (Some(expected), Some(scoped)) if expected == scoped => expected,
        _ => return Err(Code::PublisherScopeMismatch.into()),
    };
    let package: ThothPackage = publisher::table
        .find(publisher_id)
        .select(publisher::subscription_package)
        .for_share()
        .first(connection)
        .optional()?
        .ok_or(Code::InternalStateInconsistency)?;
    if !package.has_capability(PublisherCapability::MetricsCollect) {
        return Err(Code::MetricsCollectNotEntitled.into());
    }

    Ok(RequestScope {
        import,
        account,
        source,
        platform,
        publisher_id,
    })
}

/// Whether one row-level rejection is eligible for unresolved-DOI quarantine.
///
/// Called only for an observation that passed every input check, including
/// `Doi::from_str`, and then failed resolution at the locked barrier. The
/// policy is CloudFront-only and is proven solely by the locked source's exact
/// `driver_key`: no code, hostname, routing value or caller flag is consulted.
/// An observation carrying any of the five optional fields the reduced
/// quarantine representation cannot hold losslessly stays an ordinary
/// rejection.
fn quarantine_eligible(
    scope: &RequestScope,
    observation: &NormalizedMetricObservation,
    code: MetricIngestionErrorCode,
) -> bool {
    code == Code::UnknownDoi
        && scope.source.driver_key.as_deref() == Some(CLOUDFRONT_DRIVER_KEY)
        && observation.publication_isbn.is_none()
        && observation.publication_type.is_none()
        && observation.institution_ror.is_none()
        && observation.source_record_id.is_none()
        && observation.source_row_number.is_none()
}

/// Locks 6-7: every referenced measure and its mapping for the account's
/// platform, each `FOR SHARE` in ascending id order.
///
/// Codes are discovered by an ordinary read and then every row is re-read
/// under its lock, so the registry state used for acceptance is the locked
/// state.
fn lock_registry(
    connection: &mut PgConnection,
    scope: &RequestScope,
    batch: &MetricIngestionBatch,
) -> Result<Registry, AttemptError> {
    let codes: BTreeSet<&str> = batch
        .observations
        .iter()
        .map(|observation| observation.measure_code.as_str())
        .chain(
            batch
                .coverage
                .iter()
                .map(|coverage| coverage.measure_code.as_str()),
        )
        .collect();
    let codes: Vec<&str> = codes.into_iter().collect();
    let mut measure_ids: Vec<Uuid> = metric_measure::table
        .filter(metric_measure::code.eq_any(&codes))
        .select(metric_measure::measure_id)
        .load(connection)?;
    measure_ids.sort();
    measure_ids.dedup();

    let mut registry = Registry::new();
    let mut code_of_measure: HashMap<Uuid, String> = HashMap::new();
    for measure_id in &measure_ids {
        let locked: Option<MetricMeasure> = metric_measure::table
            .find(*measure_id)
            .for_share()
            .first(connection)
            .optional()?;
        if let Some(measure) = locked {
            code_of_measure.insert(measure.measure_id, measure.code.clone());
            registry.insert(
                measure.code.clone(),
                LockedMeasure {
                    measure,
                    mapping: None,
                },
            );
        }
    }

    let locked_measure_ids: Vec<Uuid> = code_of_measure.keys().copied().collect();
    let mut mapping_ids: Vec<Uuid> = metric_platform_measure::table
        .filter(metric_platform_measure::platform_id.eq(scope.platform.platform_id))
        .filter(metric_platform_measure::measure_id.eq_any(&locked_measure_ids))
        .select(metric_platform_measure::platform_measure_id)
        .load(connection)?;
    mapping_ids.sort();
    mapping_ids.dedup();
    for platform_measure_id in &mapping_ids {
        let locked: Option<MetricPlatformMeasure> = metric_platform_measure::table
            .find(*platform_measure_id)
            .for_share()
            .first(connection)
            .optional()?;
        if let Some(mapping) = locked {
            // Re-check under the lock: the mapping must still belong to the
            // account's authoritative platform and a locked measure.
            if mapping.platform_id != scope.platform.platform_id {
                continue;
            }
            if let Some(code) = code_of_measure.get(&mapping.measure_id) {
                if let Some(entry) = registry.get_mut(code) {
                    entry.mapping = Some(mapping);
                }
            }
        }
    }
    Ok(registry)
}

/// The account's authoritative platform must be named exactly and be enabled.
fn check_platform(
    scope: &RequestScope,
    platform_code: &str,
) -> Result<(), MetricIngestionErrorCode> {
    if platform_code != scope.platform.code {
        return Err(Code::PlatformMismatch);
    }
    if !scope.platform.enabled {
        return Err(Code::PlatformDisabled);
    }
    Ok(())
}

/// The exact measure code must resolve to an enabled measure with an enabled,
/// directly collected mapping for the account's platform.
fn resolve_measure<'r>(
    registry: &'r Registry,
    measure_code: &str,
) -> Result<(&'r MetricMeasure, &'r MetricPlatformMeasure), MetricIngestionErrorCode> {
    let locked = registry.get(measure_code).ok_or(Code::MeasureNotFound)?;
    if !locked.measure.enabled {
        return Err(Code::MeasureDisabled);
    }
    let mapping = locked
        .mapping
        .as_ref()
        .ok_or(Code::PlatformMeasureNotFound)?;
    if !mapping.enabled {
        return Err(Code::PlatformMeasureDisabled);
    }
    if !mapping.direct_collection {
        return Err(Code::NotDirectCollection);
    }
    Ok((&locked.measure, mapping))
}

/// Validate every coverage assertion against the locked authority. Any failure
/// is request-level.
fn plan_coverage<'a>(
    scope: &RequestScope,
    registry: &Registry,
    coverage: &'a [NormalizedMetricCoverageAssertion],
) -> Result<Vec<PlannedCoverage<'a>>, AttemptError> {
    let mut planned = Vec::with_capacity(coverage.len());
    for assertion in coverage {
        check_platform(scope, &assertion.platform_code)?;
        let (measure, mapping) = resolve_measure(registry, &assertion.measure_code)?;
        if assertion.period_end <= assertion.period_start {
            return Err(Code::InvalidCoveragePeriod.into());
        }
        if (assertion.country_coverage && !mapping.supports_country)
            || (assertion.institution_coverage && !mapping.supports_institution)
        {
            return Err(Code::InvalidCoverageDimension.into());
        }
        planned.push(PlannedCoverage {
            assertion,
            measure_id: measure.measure_id,
        });
    }
    Ok(planned)
}

/// Whether a half-open period satisfies the grain's period rule.
pub(crate) fn period_matches_grain(
    grain: MetricReportingGrain,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> bool {
    match grain {
        MetricReportingGrain::Day => period_start.succ_opt() == Some(period_end),
        MetricReportingGrain::Month => {
            period_start.day() == 1 && next_month_start(period_start) == Some(period_end)
        }
        MetricReportingGrain::ReportingPeriod => period_end > period_start,
    }
}

fn next_month_start(date: NaiveDate) -> Option<NaiveDate> {
    if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
    }
}

/// Per-observation input validation against the locked authority: everything
/// that needs no metadata resolution.
fn validate_input<'a>(
    scope: &RequestScope,
    registry: &'a Registry,
    observation: &'a NormalizedMetricObservation,
) -> Result<ValidatedInput<'a>, MetricIngestionErrorCode> {
    if observation.source_account_code != scope.account.code {
        return Err(Code::SourceAccountMismatch);
    }
    check_platform(scope, &observation.platform_code)?;
    let (measure, mapping) = resolve_measure(registry, &observation.measure_code)?;

    if !mapping
        .supported_grains
        .contains(&observation.reporting_grain)
    {
        return Err(Code::UnsupportedReportingGrain);
    }
    if !period_matches_grain(
        observation.reporting_grain,
        observation.period_start,
        observation.period_end,
    ) {
        return Err(Code::InvalidReportingGrainPeriod);
    }

    if (observation.publication_isbn.is_some() || observation.publication_type.is_some())
        && !mapping.supports_publication
    {
        return Err(Code::UnsupportedPublicationDimension);
    }
    if let Some(country) = &observation.country_code {
        if !mapping.supports_country {
            return Err(Code::UnsupportedCountryDimension);
        }
        if !country::is_assigned_alpha2(country) {
            return Err(Code::InvalidCountry);
        }
    }
    if observation.institution_ror.is_some() && !mapping.supports_institution {
        return Err(Code::UnsupportedInstitutionDimension);
    }

    if !measure.allow_negative && observation.value < 0 {
        return Err(Code::InvalidValue);
    }
    if observation.methodology_version.trim().is_empty() {
        return Err(Code::MethodologyRequired);
    }
    if let Some(fixed) = &measure.methodology_version {
        if *fixed != observation.methodology_version {
            return Err(Code::MethodologyMismatch);
        }
    }

    let doi = Doi::from_str(&observation.work_doi).map_err(|_| Code::InvalidDoi)?;
    let isbn = observation
        .publication_isbn
        .as_deref()
        .map(|isbn| Isbn::from_str(isbn).map(|isbn| isbn.to_string()))
        .transpose()
        .map_err(|_| Code::InvalidIsbn)?;
    let ror = observation
        .institution_ror
        .as_deref()
        .map(|ror| Ror::from_str(ror).map(|ror| format!("{ROR_DOMAIN}{ror}")))
        .transpose()
        .map_err(|_| Code::InvalidRor)?;

    Ok(ValidatedInput {
        observation,
        measure,
        doi,
        isbn,
        ror,
    })
}

/// Deterministic resolution classification: exactly one candidate resolves;
/// none is `unknown`; more than one is `ambiguous`.
///
/// The ambiguous branch is retained defensively even where a current database
/// uniqueness rule (`lower(work.doi)`, `(publication_type, work_id)`) makes it
/// unreachable through valid persisted state.
pub(crate) fn classify_unique<T: Copy>(
    candidates: &[T],
    unknown: MetricIngestionErrorCode,
    ambiguous: MetricIngestionErrorCode,
) -> Result<T, MetricIngestionErrorCode> {
    match candidates {
        [] => Err(unknown),
        [single] => Ok(*single),
        _ => Err(ambiguous),
    }
}

/// Resolve one canonical DOI to exactly one Work and its imprint.
///
/// This is the shared DOI authority used by both ordinary ingestion and
/// unresolved-DOI reconciliation. It performs ordinary discovery only; callers
/// must lock the returned parent/work rows and re-run it before canonical
/// application.
pub(crate) fn resolve_doi_work(
    connection: &mut PgConnection,
    doi: &Doi,
) -> Result<Result<(Uuid, Uuid), MetricIngestionErrorCode>, DieselError> {
    use diesel::dsl::sql;
    use diesel::sql_types::{Nullable, Text};

    let works: Vec<(Uuid, Uuid)> = work::table
        .filter(sql::<Nullable<Text>>("lower(doi)").eq(doi.to_lowercase_string()))
        .select((work::work_id, work::imprint_id))
        .load(connection)?;
    Ok(classify_unique(
        &works,
        Code::UnknownDoi,
        Code::AmbiguousDoi,
    ))
}

/// Resolve one validated observation's DOI, optional publication and optional
/// ROR to canonical identities by ordinary reads.
///
/// Used twice: for provisional discovery, and again at the locked
/// re-resolution barrier. The outer `Result` is a database failure; the inner
/// one is the row's resolution classification.
fn resolve(
    connection: &mut PgConnection,
    input: &ValidatedInput<'_>,
) -> Result<Result<Resolution, MetricIngestionErrorCode>, DieselError> {
    let (work_id, imprint_id) = match resolve_doi_work(connection, &input.doi)? {
        Ok(work) => work,
        Err(code) => return Ok(Err(code)),
    };

    let publication_id = if let Some(isbn) = &input.isbn {
        let candidates: Vec<Uuid> = publication::table
            .filter(publication::work_id.eq(work_id))
            .filter(publication::isbn.eq(isbn))
            .select(publication::publication_id)
            .load(connection)?;
        match classify_unique(
            &candidates,
            Code::UnknownPublication,
            Code::AmbiguousPublication,
        ) {
            Ok(publication_id) => Some(publication_id),
            Err(code) => return Ok(Err(code)),
        }
    } else if let Some(publication_type) = input.observation.publication_type {
        let candidates: Vec<Uuid> = publication::table
            .filter(publication::work_id.eq(work_id))
            .filter(publication::publication_type.eq(publication_type))
            .select(publication::publication_id)
            .load(connection)?;
        match classify_unique(
            &candidates,
            Code::UnknownPublication,
            Code::AmbiguousPublication,
        ) {
            Ok(publication_id) => Some(publication_id),
            Err(code) => return Ok(Err(code)),
        }
    } else {
        None
    };

    let institution_id = if let Some(ror) = &input.ror {
        let candidates: Vec<Uuid> = institution::table
            .filter(institution::ror.eq(ror))
            .select(institution::institution_id)
            .load(connection)?;
        match classify_unique(&candidates, Code::UnknownRor, Code::AmbiguousRor) {
            Ok(institution_id) => Some(institution_id),
            Err(code) => return Ok(Err(code)),
        }
    } else {
        None
    };

    Ok(Ok(Resolution {
        work_id,
        imprint_id,
        publication_id,
        institution_id,
    }))
}

/// Lock the required imprint rows `FOR SHARE` in ascending id order and
/// return each locked imprint's publisher.
pub(crate) fn lock_imprints(
    connection: &mut PgConnection,
    imprint_ids: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, Uuid>, DieselError> {
    let mut locked = BTreeMap::new();
    for imprint_id in imprint_ids {
        let row: Option<(Uuid, Uuid)> = imprint::table
            .filter(imprint::imprint_id.eq(imprint_id))
            .select((imprint::imprint_id, imprint::publisher_id))
            .for_share()
            .first(connection)
            .optional()?;
        if let Some((imprint_id, publisher_id)) = row {
            locked.insert(imprint_id, publisher_id);
        }
    }
    Ok(locked)
}

/// Lock the required publication rows `FOR SHARE` in ascending id order,
/// returning the ids that still existed at lock time.
fn lock_publications(
    connection: &mut PgConnection,
    publication_ids: &BTreeSet<Uuid>,
) -> Result<BTreeSet<Uuid>, DieselError> {
    let mut locked = BTreeSet::new();
    for publication_id in publication_ids {
        let row: Option<Uuid> = publication::table
            .filter(publication::publication_id.eq(publication_id))
            .select(publication::publication_id)
            .for_share()
            .first(connection)
            .optional()?;
        locked.extend(row);
    }
    Ok(locked)
}

/// Lock the required institution rows `FOR SHARE` in ascending id order,
/// returning the ids that still existed at lock time.
fn lock_institutions(
    connection: &mut PgConnection,
    institution_ids: &BTreeSet<Uuid>,
) -> Result<BTreeSet<Uuid>, DieselError> {
    let mut locked = BTreeSet::new();
    for institution_id in institution_ids {
        let row: Option<Uuid> = institution::table
            .filter(institution::institution_id.eq(institution_id))
            .select(institution::institution_id)
            .for_share()
            .first(connection)
            .optional()?;
        locked.extend(row);
    }
    Ok(locked)
}


/// Lock Work rows `FOR UPDATE` in ascending id order. Parent metadata must
/// already be locked by the caller.
pub(crate) fn lock_works(
    connection: &mut PgConnection,
    work_ids: &BTreeSet<Uuid>,
) -> Result<BTreeSet<Uuid>, DieselError> {
    let mut locked = BTreeSet::new();
    for work_id in work_ids {
        let row: Option<Uuid> = work::table
            .filter(work::work_id.eq(work_id))
            .select(work::work_id)
            .for_update()
            .first(connection)
            .optional()?;
        locked.extend(row);
    }
    Ok(locked)
}

/// Acquire the canonical dimensional-cell advisory locks in ascending signed
/// key order. A single-cell reconciliation uses the same helper and therefore
/// the same PostgreSQL lock namespace as ordinary ingestion.
pub(crate) fn lock_cell_keys(
    connection: &mut PgConnection,
    cell_keys: &BTreeSet<i64>,
) -> Result<(), DieselError> {
    for key in cell_keys {
        sql_query("SELECT pg_advisory_xact_lock($1)")
            .bind::<diesel::sql_types::BigInt, _>(*key)
            .execute(connection)?;
    }
    Ok(())
}

/// Whether a distinct accepted record in the same dimensional cell overlaps
/// the candidate's half-open period, using the approved same-cell lookup.
fn overlap_exists(
    connection: &mut PgConnection,
    identity: &CanonicalIdentity,
) -> Result<bool, DieselError> {
    let mut query = metric_record::table
        .into_boxed()
        .filter(metric_record::platform_id.eq(identity.platform_id))
        .filter(metric_record::measure_id.eq(identity.measure_id))
        .filter(metric_record::work_id.eq(identity.work_id))
        .filter(metric_record::period_start.lt(identity.period_end))
        .filter(metric_record::period_end.gt(identity.period_start));
    query = match identity.publication_id {
        Some(publication_id) => query.filter(metric_record::publication_id.eq(publication_id)),
        None => query.filter(metric_record::publication_id.is_null()),
    };
    query = match &identity.country_code {
        Some(country_code) => query.filter(metric_record::country_code.eq(country_code)),
        None => query.filter(metric_record::country_code.is_null()),
    };
    query = match identity.institution_id {
        Some(institution_id) => query.filter(metric_record::institution_id.eq(institution_id)),
        None => query.filter(metric_record::institution_id.is_null()),
    };
    let existing: Option<Uuid> = query
        .select(metric_record::record_id)
        .first(connection)
        .optional()?;
    Ok(existing.is_some())
}

/// Insert the provenance row for one processed observation, returning its
/// generated id.
#[allow(clippy::too_many_arguments)]
fn insert_provenance(
    connection: &mut PgConnection,
    scope: &RequestScope,
    import_batch_id: Uuid,
    row_index: i64,
    observation: &NormalizedMetricObservation,
    record_id: Option<Uuid>,
    identity_hash: Option<&str>,
    content_hash: Option<&str>,
    classification: MetricRecordProvenanceClassification,
    reason_code: Option<MetricIngestionErrorCode>,
) -> Result<Uuid, DieselError> {
    let details = json!({
        "schema": PROVENANCE_DETAILS_SCHEMA,
        "reason_code": reason_code.map(|code| code.to_string()),
        "reporting_grain": enum_code(&observation.reporting_grain),
    });
    diesel::insert_into(metric_record_provenance::table)
        .values((
            metric_record_provenance::record_id.eq(record_id),
            metric_record_provenance::import_id.eq(scope.import.import_id),
            metric_record_provenance::source_record_id.eq(observation.source_record_id.as_deref()),
            metric_record_provenance::source_row_number.eq(observation.source_row_number),
            metric_record_provenance::identity_hash.eq(identity_hash),
            metric_record_provenance::content_hash.eq(content_hash),
            metric_record_provenance::classification.eq(classification),
            metric_record_provenance::details.eq(details),
            metric_record_provenance::import_batch_id.eq(import_batch_id),
            metric_record_provenance::batch_row_index.eq(row_index),
        ))
        .returning(metric_record_provenance::record_provenance_id)
        .get_result(connection)
}

/// Insert one `PENDING` rollup delta for one canonical revision.
fn insert_delta(
    connection: &mut PgConnection,
    record_id: Uuid,
    revision_id: Uuid,
    delta_value: i64,
) -> Result<(), DieselError> {
    diesel::insert_into(metric_rollup_delta::table)
        .values((
            metric_rollup_delta::record_id.eq(record_id),
            metric_rollup_delta::revision_id.eq(revision_id),
            metric_rollup_delta::delta_value.eq(delta_value),
            metric_rollup_delta::status.eq(ROLLUP_DELTA_PENDING),
        ))
        .execute(connection)?;
    Ok(())
}

/// Record one `REJECTED` observation: provenance, exactly one sanitized
/// import error, the eligible unresolved-DOI quarantine row when planned, and
/// the invalid counter. The row's outcome is the same with or without
/// quarantine.
#[allow(clippy::too_many_arguments)]
fn write_rejected(
    connection: &mut PgConnection,
    scope: &RequestScope,
    import_batch_id: Uuid,
    row_index: i64,
    observation: &NormalizedMetricObservation,
    code: MetricIngestionErrorCode,
    identity_hash: Option<String>,
    content_hash: Option<String>,
    quarantine: Option<QuarantinePlan<'_>>,
    counters: &mut Counters,
) -> Result<MetricIngestionRowOutcome, DieselError> {
    let record_provenance_id = insert_provenance(
        connection,
        scope,
        import_batch_id,
        row_index,
        observation,
        None,
        identity_hash.as_deref(),
        content_hash.as_deref(),
        MetricRecordProvenanceClassification::Rejected,
        Some(code),
    )?;
    diesel::insert_into(metric_import_error::table)
        .values((
            metric_import_error::import_id.eq(scope.import.import_id),
            metric_import_error::row_number.eq(observation.source_row_number),
            metric_import_error::error_code.eq(code.to_string()),
            metric_import_error::severity.eq(MetricImportErrorSeverity::Error),
            metric_import_error::field_name
                .eq(code.field_name(observation.publication_isbn.is_some())),
            metric_import_error::message.eq(code.message()),
            metric_import_error::raw_value.eq(None::<String>),
        ))
        .execute(connection)?;
    if let Some(quarantine) = quarantine {
        // The DOI is stored exactly as supplied: it already passed
        // `Doi::from_str`, and it is not lowercased or re-prefixed here.
        diesel::insert_into(metric_identifier_quarantine::table)
            .values((
                metric_identifier_quarantine::record_provenance_id.eq(record_provenance_id),
                metric_identifier_quarantine::source_account_id.eq(scope.account.source_account_id),
                metric_identifier_quarantine::platform_id.eq(scope.platform.platform_id),
                metric_identifier_quarantine::measure_id.eq(quarantine.measure_id),
                metric_identifier_quarantine::schema_version.eq(quarantine.schema_version),
                metric_identifier_quarantine::work_doi.eq(&observation.work_doi),
                metric_identifier_quarantine::period_start.eq(observation.period_start),
                metric_identifier_quarantine::period_end.eq(observation.period_end),
                metric_identifier_quarantine::reporting_grain.eq(observation.reporting_grain),
                metric_identifier_quarantine::country_code.eq(observation.country_code.as_deref()),
                metric_identifier_quarantine::value.eq(observation.value),
                metric_identifier_quarantine::methodology_version
                    .eq(&observation.methodology_version),
            ))
            .execute(connection)?;
    }
    counters.invalid += 1;
    Ok(MetricIngestionRowOutcome {
        batch_row_index: row_index,
        classification: MetricRecordProvenanceClassification::Rejected,
        reason_code: Some(code),
        record_id: None,
        identity_hash,
        content_hash,
    })
}

/// Apply one canonical candidate after its dimensional-cell lock.
///
/// This is the single winner/duplicate/source-conflict/revision authority used
/// by ordinary ingestion and quarantine reconciliation. It intentionally does
/// not write provenance or import counters. The caller's surrounding
/// transaction owns those route-specific ledgers.
pub(crate) fn apply_canonical_application(
    connection: &mut PgConnection,
    candidate: &CanonicalApplicationCandidate,
    chronology: CanonicalChronology,
) -> Result<CanonicalApplicationOutcome, CanonicalApplicationError> {
    let existing: Option<MetricRecord> = metric_record::table
        .filter(metric_record::identity_hash.eq(&candidate.identity_hash))
        .for_update()
        .first(connection)
        .optional()?;

    let Some(record) = existing else {
        if overlap_exists(connection, &candidate.identity)? {
            return Ok(CanonicalApplicationOutcome::OverlappingPeriod);
        }

        let identity = &candidate.identity;
        let record_id: Uuid = diesel::insert_into(metric_record::table)
            .values((
                metric_record::identity_hash.eq(&candidate.identity_hash),
                metric_record::work_id.eq(identity.work_id),
                metric_record::publication_id.eq(identity.publication_id),
                metric_record::platform_id.eq(identity.platform_id),
                metric_record::measure_id.eq(identity.measure_id),
                metric_record::period_start.eq(identity.period_start),
                metric_record::period_end.eq(identity.period_end),
                metric_record::reporting_grain.eq(candidate.reporting_grain),
                metric_record::country_code.eq(identity.country_code.as_deref()),
                metric_record::institution_id.eq(identity.institution_id),
                metric_record::winning_source_account_id.eq(candidate.source_account_id),
            ))
            .returning(metric_record::record_id)
            .get_result(connection)?;
        let revision_id: Uuid = diesel::insert_into(metric_record_revision::table)
            .values((
                metric_record_revision::record_id.eq(record_id),
                metric_record_revision::revision_number.eq(1),
                metric_record_revision::import_id.eq(candidate.import_id),
                metric_record_revision::value.eq(candidate.value),
                metric_record_revision::content_hash.eq(&candidate.content_hash),
                metric_record_revision::status.eq(MetricRecordRevisionStatus::Current),
            ))
            .returning(metric_record_revision::record_revision_id)
            .get_result(connection)?;
        diesel::update(metric_record::table.find(record_id))
            .set(metric_record::current_revision_id.eq(revision_id))
            .execute(connection)?;
        insert_delta(connection, record_id, revision_id, candidate.value)?;
        return Ok(CanonicalApplicationOutcome::Winner {
            record_id,
            record_revision_id: revision_id,
        });
    };

    let current_revision_id = record
        .current_revision_id
        .ok_or(CanonicalApplicationError::InternalStateInconsistency)?;
    let current: MetricRecordRevision = metric_record_revision::table
        .find(current_revision_id)
        .for_update()
        .first(connection)
        .optional()?
        .ok_or(CanonicalApplicationError::InternalStateInconsistency)?;

    if current.content_hash == candidate.content_hash {
        return Ok(CanonicalApplicationOutcome::Duplicate {
            record_id: record.record_id,
            record_revision_id: current.record_revision_id,
        });
    }

    if record.winning_source_account_id != candidate.source_account_id {
        return Ok(CanonicalApplicationOutcome::SourceConflict);
    }

    if let CanonicalChronology::Historical { import_created_at } = chronology {
        if current.import_id == candidate.import_id {
            return Ok(CanonicalApplicationOutcome::SameImportOrder);
        }
        let current_import_created_at: Timestamp = metric_import::table
            .find(current.import_id)
            .select(metric_import::created_at)
            .first(connection)
            .optional()?
            .ok_or(CanonicalApplicationError::InternalStateInconsistency)?;
        if import_created_at < current_import_created_at {
            return Ok(CanonicalApplicationOutcome::Superseded {
                record_id: record.record_id,
                record_revision_id: current.record_revision_id,
            });
        }
        if import_created_at == current_import_created_at {
            return Ok(CanonicalApplicationOutcome::ImportOrderAmbiguous);
        }
    }

    let Some(delta_value) = candidate.value.checked_sub(current.value) else {
        return Ok(CanonicalApplicationOutcome::DeltaOverflow);
    };

    diesel::update(metric_record_revision::table.find(current.record_revision_id))
        .set(metric_record_revision::status.eq(MetricRecordRevisionStatus::Superseded))
        .execute(connection)?;
    let revision_id: Uuid = diesel::insert_into(metric_record_revision::table)
        .values((
            metric_record_revision::record_id.eq(record.record_id),
            metric_record_revision::revision_number.eq(current.revision_number + 1),
            metric_record_revision::import_id.eq(candidate.import_id),
            metric_record_revision::value.eq(candidate.value),
            metric_record_revision::content_hash.eq(&candidate.content_hash),
            metric_record_revision::status.eq(MetricRecordRevisionStatus::Current),
            metric_record_revision::supersedes_revision_id.eq(current.record_revision_id),
        ))
        .returning(metric_record_revision::record_revision_id)
        .get_result(connection)?;
    diesel::update(metric_record::table.find(record.record_id))
        .set(metric_record::current_revision_id.eq(revision_id))
        .execute(connection)?;
    insert_delta(connection, record.record_id, revision_id, delta_value)?;
    Ok(CanonicalApplicationOutcome::Revision {
        record_id: record.record_id,
        record_revision_id: revision_id,
    })
}

/// Apply one ordinary ingestion candidate through the shared canonical
/// authority, then write the ordinary-ingestion provenance/counters that
/// quarantine reconciliation is forbidden to alter.
fn apply_candidate(
    connection: &mut PgConnection,
    scope: &RequestScope,
    import_batch_id: Uuid,
    row_index: i64,
    candidate: &Candidate<'_>,
    counters: &mut Counters,
) -> Result<MetricIngestionRowOutcome, AttemptError> {
    let observation = candidate.input.observation;
    let shared = CanonicalApplicationCandidate {
        identity: candidate.identity.clone(),
        identity_hash: candidate.identity_hash.clone(),
        content_hash: candidate.content_hash.clone(),
        reporting_grain: observation.reporting_grain,
        source_account_id: scope.account.source_account_id,
        import_id: scope.import.import_id,
        value: observation.value,
    };
    let outcome = apply_canonical_application(connection, &shared, CanonicalChronology::Arrival)
        .map_err(|error| match error {
            CanonicalApplicationError::Database(error) => AttemptError::Database(error),
            CanonicalApplicationError::InternalStateInconsistency => {
                AttemptError::Request(Code::InternalStateInconsistency)
            }
        })?;

    let successful = |classification, record_id| MetricIngestionRowOutcome {
        batch_row_index: row_index,
        classification,
        reason_code: None,
        record_id: Some(record_id),
        identity_hash: Some(candidate.identity_hash.clone()),
        content_hash: Some(candidate.content_hash.clone()),
    };

    match outcome {
        CanonicalApplicationOutcome::Winner { record_id, .. } => {
            insert_provenance(
                connection,
                scope,
                import_batch_id,
                row_index,
                observation,
                Some(record_id),
                Some(&candidate.identity_hash),
                Some(&candidate.content_hash),
                MetricRecordProvenanceClassification::Winner,
                None,
            )?;
            counters.accepted += 1;
            Ok(successful(
                MetricRecordProvenanceClassification::Winner,
                record_id,
            ))
        }
        CanonicalApplicationOutcome::Duplicate { record_id, .. } => {
            insert_provenance(
                connection,
                scope,
                import_batch_id,
                row_index,
                observation,
                Some(record_id),
                Some(&candidate.identity_hash),
                Some(&candidate.content_hash),
                MetricRecordProvenanceClassification::Duplicate,
                None,
            )?;
            counters.duplicate += 1;
            Ok(successful(
                MetricRecordProvenanceClassification::Duplicate,
                record_id,
            ))
        }
        CanonicalApplicationOutcome::Revision { record_id, .. } => {
            insert_provenance(
                connection,
                scope,
                import_batch_id,
                row_index,
                observation,
                Some(record_id),
                Some(&candidate.identity_hash),
                Some(&candidate.content_hash),
                MetricRecordProvenanceClassification::Revision,
                None,
            )?;
            counters.revision += 1;
            Ok(successful(
                MetricRecordProvenanceClassification::Revision,
                record_id,
            ))
        }
        CanonicalApplicationOutcome::SourceConflict => {
            let record_id: Uuid = metric_record::table
                .filter(metric_record::identity_hash.eq(&candidate.identity_hash))
                .select(metric_record::record_id)
                .first(connection)?;
            insert_provenance(
                connection,
                scope,
                import_batch_id,
                row_index,
                observation,
                Some(record_id),
                Some(&candidate.identity_hash),
                Some(&candidate.content_hash),
                MetricRecordProvenanceClassification::Conflict,
                Some(Code::SourceConflict),
            )?;
            counters.conflict += 1;
            Ok(MetricIngestionRowOutcome {
                batch_row_index: row_index,
                classification: MetricRecordProvenanceClassification::Conflict,
                reason_code: Some(Code::SourceConflict),
                record_id: Some(record_id),
                identity_hash: Some(candidate.identity_hash.clone()),
                content_hash: Some(candidate.content_hash.clone()),
            })
        }
        CanonicalApplicationOutcome::OverlappingPeriod => Ok(write_rejected(
            connection,
            scope,
            import_batch_id,
            row_index,
            observation,
            Code::OverlappingPeriod,
            Some(candidate.identity_hash.clone()),
            Some(candidate.content_hash.clone()),
            None,
            counters,
        )?),
        CanonicalApplicationOutcome::DeltaOverflow => Ok(write_rejected(
            connection,
            scope,
            import_batch_id,
            row_index,
            observation,
            Code::InvalidValue,
            Some(candidate.identity_hash.clone()),
            Some(candidate.content_hash.clone()),
            None,
            counters,
        )?),
        CanonicalApplicationOutcome::Superseded { .. }
        | CanonicalApplicationOutcome::SameImportOrder
        | CanonicalApplicationOutcome::ImportOrderAmbiguous => {
            Err(Code::InternalStateInconsistency.into())
        }
    }
}
