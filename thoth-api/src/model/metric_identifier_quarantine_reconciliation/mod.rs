//! Automatic unresolved-DOI quarantine reconciliation (`MET-WP7-PREREQ-03`).
//!
//! Historical quarantine/provenance/import evidence is immutable. This module
//! owns only the additive reconciliation state and the bounded protected sweep
//! that retries current DOI -> Work authority. Canonical application is
//! delegated to `metric_ingestion`'s shared lock/hash/application authority;
//! no second winner/duplicate/revision implementation exists here.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::str::FromStr;

use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::sql_types::{Bool, Nullable, Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::metric_identifier_quarantine::MetricIdentifierQuarantine;
use crate::model::metric_import::{MetricImport, MetricImportStatus};
use crate::model::metric_ingestion::country;
use crate::model::metric_ingestion::hash::{
    cell_lock_key, content_hash, identity_hash, CanonicalIdentity,
};
use crate::model::metric_ingestion::{
    apply_canonical_application, lock_cell_keys, lock_imprints, lock_works, period_matches_grain,
    resolve_doi_work, CanonicalApplicationCandidate, CanonicalApplicationError,
    CanonicalApplicationOutcome, CanonicalChronology, MetricIngestionErrorCode,
    PROVENANCE_DETAILS_SCHEMA, SUPPORTED_SCHEMA_VERSION,
};
use crate::model::metric_measure::MetricMeasure;
use crate::model::metric_platform::MetricPlatform;
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::{
    MetricRecordProvenance, MetricRecordProvenanceClassification,
};
use crate::model::metric_source::{MetricSource, MetricSourceAcquisitionType};
use crate::model::metric_source_account::{MetricSourceAccount, CLOUDFRONT_DRIVER_KEY};
use crate::model::{Doi, Timestamp};
use crate::schema::{
    metric_identifier_quarantine, metric_import, metric_measure, metric_platform,
    metric_record_provenance, metric_source, metric_source_account,
};

/// Smallest accepted reconciliation sweep.
pub const RECONCILIATION_MIN_LIMIT: i32 = 1;
/// Largest accepted reconciliation sweep.
pub const RECONCILIATION_MAX_LIMIT: i32 = 50;
/// Maximum authority-drift restarts before the call fails closed.
const MAX_AUTHORITY_DRIFT_RETRIES: u8 = 3;

/// The closed durable reconciliation state vocabulary.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricIdentifierQuarantineReconciliationState"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricIdentifierQuarantineReconciliationState {
    #[cfg_attr(feature = "backend", db_rename = "PENDING_UNKNOWN_DOI")]
    PendingUnknownDoi,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_AMBIGUOUS_DOI")]
    BlockedAmbiguousDoi,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_PUBLISHER_SCOPE_MISMATCH")]
    BlockedPublisherScopeMismatch,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_SOURCE_CONFLICT")]
    BlockedSourceConflict,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_OVERLAPPING_PERIOD")]
    BlockedOverlappingPeriod,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_SAME_IMPORT_ORDER")]
    BlockedSameImportOrder,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_IMPORT_ORDER_AMBIGUOUS")]
    BlockedImportOrderAmbiguous,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_DELTA_OVERFLOW")]
    BlockedDeltaOverflow,
    #[cfg_attr(feature = "backend", db_rename = "BLOCKED_INCONSISTENT_EVIDENCE")]
    BlockedInconsistentEvidence,
    #[cfg_attr(feature = "backend", db_rename = "RESOLVED_WINNER")]
    ResolvedWinner,
    #[cfg_attr(feature = "backend", db_rename = "RESOLVED_DUPLICATE")]
    ResolvedDuplicate,
    #[cfg_attr(feature = "backend", db_rename = "RESOLVED_REVISION")]
    ResolvedRevision,
    #[cfg_attr(feature = "backend", db_rename = "RESOLVED_SUPERSEDED")]
    ResolvedSuperseded,
}

impl MetricIdentifierQuarantineReconciliationState {
    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::ResolvedWinner
                | Self::ResolvedDuplicate
                | Self::ResolvedRevision
                | Self::ResolvedSuperseded
        )
    }

    fn bucket(self) -> AttemptBucket {
        match self {
            Self::PendingUnknownDoi => AttemptBucket::Pending,
            Self::ResolvedWinner
            | Self::ResolvedDuplicate
            | Self::ResolvedRevision
            | Self::ResolvedSuperseded => AttemptBucket::Resolved,
            _ => AttemptBucket::Blocked,
        }
    }
}

/// One persisted reconciliation row.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricIdentifierQuarantineReconciliation {
    pub identifier_quarantine_id: Uuid,
    pub state: MetricIdentifierQuarantineReconciliationState,
    pub attempt_count: i32,
    pub last_attempted_by: String,
    pub first_attempt_at: Timestamp,
    pub last_attempt_at: Timestamp,
    pub next_attempt_at: Option<Timestamp>,
    pub resolved_at: Option<Timestamp>,
    pub record_id: Option<Uuid>,
    pub record_revision_id: Option<Uuid>,
}

/// Aggregate-only result of one bounded reconciliation invocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MetricIdentifierQuarantineReconciliationBatch {
    pub attempted: i32,
    pub resolved: i32,
    pub pending: i32,
    pub blocked: i32,
}

impl MetricIdentifierQuarantineReconciliationBatch {
    fn record(&mut self, bucket: AttemptBucket) {
        self.attempted += 1;
        match bucket {
            AttemptBucket::Resolved => self.resolved += 1,
            AttemptBucket::Pending => self.pending += 1,
            AttemptBucket::Blocked => self.blocked += 1,
        }
        debug_assert_eq!(self.attempted, self.resolved + self.pending + self.blocked);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttemptBucket {
    Resolved,
    Pending,
    Blocked,
}

#[derive(Debug)]
enum RowAttemptError {
    Database(DieselError),
    Drift,
    InternalState,
}

impl From<DieselError> for RowAttemptError {
    fn from(error: DieselError) -> Self {
        Self::Database(error)
    }
}

#[derive(diesel::QueryableByName)]
struct CandidateId {
    #[diesel(sql_type = SqlUuid)]
    identifier_quarantine_id: Uuid,
}

fn rejected_limit() -> ThothError {
    ThothError::DatabaseConstraintError(Cow::Borrowed(
        "A metric identifier quarantine reconciliation limit must be between 1 and 50 inclusive.",
    ))
}

fn internal_error() -> ThothError {
    ThothError::InternalError(
        "Metric identifier quarantine reconciliation failed safely.".to_string(),
    )
}

/// Reconcile up to `limit` due quarantine rows.
///
/// Every selected quarantine row owns one database transaction. Expected
/// pending/blocked/resolved outcomes commit independently; if a later
/// unexpected failure occurs, earlier rows remain committed and the call
/// returns one fixed redacted error.
pub(crate) fn reconcile_metric_identifier_quarantine(
    db: &PgPool,
    actor: &str,
    limit: i32,
) -> ThothResult<MetricIdentifierQuarantineReconciliationBatch> {
    if !(RECONCILIATION_MIN_LIMIT..=RECONCILIATION_MAX_LIMIT).contains(&limit) {
        return Err(rejected_limit());
    }
    if actor.trim().is_empty() {
        return Err(internal_error());
    }

    let mut batch = MetricIdentifierQuarantineReconciliationBatch::default();
    let mut drift_retries = 0_u8;

    while batch.attempted < limit {
        let mut connection = db.get().map_err(|error| {
            log::error!(
                "quarantine reconciliation could not obtain a database connection: {error}"
            );
            internal_error()
        })?;

        let attempt =
            connection.transaction::<Option<AttemptBucket>, RowAttemptError, _>(|connection| {
                process_one(connection, actor)
            });

        match attempt {
            Ok(Some(bucket)) => {
                batch.record(bucket);
                drift_retries = 0;
            }
            Ok(None) => break,
            Err(RowAttemptError::Drift) if drift_retries + 1 < MAX_AUTHORITY_DRIFT_RETRIES => {
                drift_retries += 1;
            }
            Err(RowAttemptError::Drift) => {
                log::warn!(
                    "quarantine reconciliation exhausted bounded metadata authority drift retries"
                );
                return Err(internal_error());
            }
            Err(RowAttemptError::Database(error)) => {
                log::error!("quarantine reconciliation database failure: {error}");
                return Err(internal_error());
            }
            Err(RowAttemptError::InternalState) => {
                log::error!("quarantine reconciliation encountered inconsistent canonical state");
                return Err(internal_error());
            }
        }
    }

    Ok(batch)
}

fn process_one(
    connection: &mut PgConnection,
    actor: &str,
) -> Result<Option<AttemptBucket>, RowAttemptError> {
    let candidate: Option<CandidateId> = sql_query(
        "SELECT q.identifier_quarantine_id \
         FROM public.metric_identifier_quarantine q \
         LEFT JOIN public.metric_identifier_quarantine_reconciliation r \
           ON r.identifier_quarantine_id = q.identifier_quarantine_id \
         WHERE r.identifier_quarantine_id IS NULL \
            OR (r.resolved_at IS NULL \
                AND r.next_attempt_at <= transaction_timestamp()) \
         ORDER BY (r.identifier_quarantine_id IS NOT NULL) ASC, \
                  r.next_attempt_at ASC NULLS FIRST, \
                  q.created_at ASC, \
                  q.identifier_quarantine_id ASC \
         FOR UPDATE OF q SKIP LOCKED \
         LIMIT 1",
    )
    .get_result(connection)
    .optional()?;

    let Some(candidate) = candidate else {
        return Ok(None);
    };

    let quarantine: MetricIdentifierQuarantine = metric_identifier_quarantine::table
        .find(candidate.identifier_quarantine_id)
        .first(connection)?;

    let provenance: Option<MetricRecordProvenance> = metric_record_provenance::table
        .find(quarantine.record_provenance_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(provenance) = provenance else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    let import: Option<MetricImport> = metric_import::table
        .find(provenance.import_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(import) = import else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    let account: Option<MetricSourceAccount> = metric_source_account::table
        .find(quarantine.source_account_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(account) = account else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    let source: Option<MetricSource> = metric_source::table
        .find(account.source_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(source) = source else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    let platform: Option<MetricPlatform> = metric_platform::table
        .find(quarantine.platform_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(platform) = platform else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    let measure: Option<MetricMeasure> = metric_measure::table
        .find(quarantine.measure_id)
        .for_share()
        .first(connection)
        .optional()?;
    let Some(measure) = measure else {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    };

    if !historical_evidence_consistent(
        &quarantine,
        &provenance,
        &import,
        &account,
        &source,
        &platform,
        &measure,
    ) {
        return persist_inconsistent(connection, actor, quarantine.identifier_quarantine_id);
    }

    let doi = Doi::from_str(&quarantine.work_doi).map_err(|_| RowAttemptError::InternalState)?;

    let provisional = match resolve_doi_work(connection, &doi)? {
        Err(MetricIngestionErrorCode::UnknownDoi) => {
            persist_state(
                connection,
                actor,
                quarantine.identifier_quarantine_id,
                MetricIdentifierQuarantineReconciliationState::PendingUnknownDoi,
                None,
                None,
            )?;
            return Ok(Some(AttemptBucket::Pending));
        }
        Err(MetricIngestionErrorCode::AmbiguousDoi) => {
            persist_state(
                connection,
                actor,
                quarantine.identifier_quarantine_id,
                MetricIdentifierQuarantineReconciliationState::BlockedAmbiguousDoi,
                None,
                None,
            )?;
            return Ok(Some(AttemptBucket::Blocked));
        }
        Err(_) => return Err(RowAttemptError::InternalState),
        Ok(resolution) => resolution,
    };

    let (work_id, imprint_id) = provisional;
    let mut imprint_ids = BTreeSet::new();
    imprint_ids.insert(imprint_id);
    let locked_imprints = lock_imprints(connection, &imprint_ids)?;

    let mut work_ids = BTreeSet::new();
    work_ids.insert(work_id);
    let locked_works = lock_works(connection, &work_ids)?;

    if !locked_works.contains(&work_id) || !locked_imprints.contains_key(&imprint_id) {
        return Err(RowAttemptError::Drift);
    }

    let locked_resolution = match resolve_doi_work(connection, &doi)? {
        Ok(resolution) => resolution,
        Err(_) => return Err(RowAttemptError::Drift),
    };
    if locked_resolution != provisional {
        return Err(RowAttemptError::Drift);
    }

    let current_publisher_id = locked_imprints
        .get(&imprint_id)
        .copied()
        .ok_or(RowAttemptError::Drift)?;
    if import.publisher_id != Some(current_publisher_id) {
        persist_state(
            connection,
            actor,
            quarantine.identifier_quarantine_id,
            MetricIdentifierQuarantineReconciliationState::BlockedPublisherScopeMismatch,
            None,
            None,
        )?;
        return Ok(Some(AttemptBucket::Blocked));
    }

    let identity = CanonicalIdentity {
        platform_id: quarantine.platform_id,
        measure_id: quarantine.measure_id,
        work_id,
        publication_id: None,
        period_start: quarantine.period_start,
        period_end: quarantine.period_end,
        country_code: quarantine.country_code.clone(),
        institution_id: None,
    };
    let identity_hash = identity_hash(&identity);
    let content_hash = content_hash(&identity, quarantine.value, &quarantine.methodology_version);
    let cell_key = cell_lock_key(&identity);
    let mut cell_keys = BTreeSet::new();
    cell_keys.insert(cell_key);
    lock_cell_keys(connection, &cell_keys)?;

    let canonical = CanonicalApplicationCandidate {
        identity,
        identity_hash,
        content_hash,
        reporting_grain: quarantine.reporting_grain,
        source_account_id: quarantine.source_account_id,
        import_id: import.import_id,
        value: quarantine.value,
    };

    let outcome = apply_canonical_application(
        connection,
        &canonical,
        CanonicalChronology::Historical {
            import_created_at: import.created_at,
        },
    )
    .map_err(|error| match error {
        CanonicalApplicationError::Database(error) => RowAttemptError::Database(error),
        CanonicalApplicationError::InternalStateInconsistency => RowAttemptError::InternalState,
    })?;

    let (state, record_id, revision_id) = match outcome {
        CanonicalApplicationOutcome::Winner {
            record_id,
            record_revision_id,
        } => (
            MetricIdentifierQuarantineReconciliationState::ResolvedWinner,
            Some(record_id),
            Some(record_revision_id),
        ),
        CanonicalApplicationOutcome::Duplicate {
            record_id,
            record_revision_id,
        } => (
            MetricIdentifierQuarantineReconciliationState::ResolvedDuplicate,
            Some(record_id),
            Some(record_revision_id),
        ),
        CanonicalApplicationOutcome::Revision {
            record_id,
            record_revision_id,
        } => (
            MetricIdentifierQuarantineReconciliationState::ResolvedRevision,
            Some(record_id),
            Some(record_revision_id),
        ),
        CanonicalApplicationOutcome::Superseded {
            record_id,
            record_revision_id,
        } => (
            MetricIdentifierQuarantineReconciliationState::ResolvedSuperseded,
            Some(record_id),
            Some(record_revision_id),
        ),
        CanonicalApplicationOutcome::SourceConflict => (
            MetricIdentifierQuarantineReconciliationState::BlockedSourceConflict,
            None,
            None,
        ),
        CanonicalApplicationOutcome::OverlappingPeriod => (
            MetricIdentifierQuarantineReconciliationState::BlockedOverlappingPeriod,
            None,
            None,
        ),
        CanonicalApplicationOutcome::SameImportOrder => (
            MetricIdentifierQuarantineReconciliationState::BlockedSameImportOrder,
            None,
            None,
        ),
        CanonicalApplicationOutcome::ImportOrderAmbiguous => (
            MetricIdentifierQuarantineReconciliationState::BlockedImportOrderAmbiguous,
            None,
            None,
        ),
        CanonicalApplicationOutcome::DeltaOverflow => (
            MetricIdentifierQuarantineReconciliationState::BlockedDeltaOverflow,
            None,
            None,
        ),
    };

    persist_state(
        connection,
        actor,
        quarantine.identifier_quarantine_id,
        state,
        record_id,
        revision_id,
    )?;
    Ok(Some(state.bucket()))
}

fn historical_evidence_consistent(
    quarantine: &MetricIdentifierQuarantine,
    provenance: &MetricRecordProvenance,
    import: &MetricImport,
    account: &MetricSourceAccount,
    source: &MetricSource,
    platform: &MetricPlatform,
    measure: &MetricMeasure,
) -> bool {
    let Some(details) = provenance.details.as_object() else {
        return false;
    };
    let schema_is_supported = details.get("schema").and_then(serde_json::Value::as_str)
        == Some(PROVENANCE_DETAILS_SCHEMA);
    let reason_is_unknown_doi = details
        .get("reason_code")
        .and_then(serde_json::Value::as_str)
        .and_then(|code| MetricIngestionErrorCode::from_str(code).ok())
        == Some(MetricIngestionErrorCode::UnknownDoi);
    let reporting_grain_matches = details
        .get("reporting_grain")
        .and_then(serde_json::Value::as_str)
        .and_then(|grain| MetricReportingGrain::from_str(grain).ok())
        == Some(quarantine.reporting_grain);

    provenance.classification == MetricRecordProvenanceClassification::Rejected
        && schema_is_supported
        && reason_is_unknown_doi
        && reporting_grain_matches
        && provenance.import_id == import.import_id
        && provenance.record_id.is_none()
        && provenance.identity_hash.is_none()
        && provenance.content_hash.is_none()
        && provenance.source_record_id.is_none()
        && provenance.source_row_number.is_none()
        && provenance.import_batch_id.is_some()
        && provenance.batch_row_index.is_some()
        && import.source_account_id == quarantine.source_account_id
        && import.status == MetricImportStatus::CompletedWithErrors
        && import.publisher_id.is_some()
        && account.source_account_id == quarantine.source_account_id
        && account.platform_id == quarantine.platform_id
        && source.source_id == account.source_id
        && source.acquisition_type == MetricSourceAcquisitionType::Driver
        && source.driver_key.as_deref() == Some(CLOUDFRONT_DRIVER_KEY)
        && platform.platform_id == quarantine.platform_id
        && measure.measure_id == quarantine.measure_id
        && quarantine.schema_version == SUPPORTED_SCHEMA_VERSION
        && !quarantine.methodology_version.trim().is_empty()
        && period_matches_grain(
            quarantine.reporting_grain,
            quarantine.period_start,
            quarantine.period_end,
        )
        && quarantine
            .country_code
            .as_deref()
            .is_none_or(country::is_assigned_alpha2)
        && (measure.allow_negative || quarantine.value >= 0)
        && Doi::from_str(&quarantine.work_doi).is_ok()
}

fn persist_inconsistent(
    connection: &mut PgConnection,
    actor: &str,
    identifier_quarantine_id: Uuid,
) -> Result<Option<AttemptBucket>, RowAttemptError> {
    persist_state(
        connection,
        actor,
        identifier_quarantine_id,
        MetricIdentifierQuarantineReconciliationState::BlockedInconsistentEvidence,
        None,
        None,
    )?;
    Ok(Some(AttemptBucket::Blocked))
}

fn persist_state(
    connection: &mut PgConnection,
    actor: &str,
    identifier_quarantine_id: Uuid,
    state: MetricIdentifierQuarantineReconciliationState,
    record_id: Option<Uuid>,
    record_revision_id: Option<Uuid>,
) -> Result<(), DieselError> {
    let terminal = state.is_terminal();
    sql_query(
        "INSERT INTO public.metric_identifier_quarantine_reconciliation AS reconciliation ( \
             identifier_quarantine_id, state, attempt_count, last_attempted_by, \
             first_attempt_at, last_attempt_at, next_attempt_at, resolved_at, \
             record_id, record_revision_id \
         ) VALUES ( \
             $1, $2::public.metric_identifier_quarantine_reconciliation_state, 1, $3, \
             transaction_timestamp(), transaction_timestamp(), \
             CASE WHEN $4 THEN NULL ELSE transaction_timestamp() + INTERVAL '1 hour' END, \
             CASE WHEN $4 THEN transaction_timestamp() ELSE NULL END, \
             $5, $6 \
         ) \
         ON CONFLICT (identifier_quarantine_id) DO UPDATE SET \
             state = EXCLUDED.state, \
             attempt_count = reconciliation.attempt_count + 1, \
             last_attempted_by = EXCLUDED.last_attempted_by, \
             last_attempt_at = transaction_timestamp(), \
             next_attempt_at = CASE WHEN $4 THEN NULL ELSE \
                 transaction_timestamp() + CASE reconciliation.attempt_count + 1 \
                     WHEN 2 THEN INTERVAL '2 hours' \
                     WHEN 3 THEN INTERVAL '4 hours' \
                     WHEN 4 THEN INTERVAL '8 hours' \
                     WHEN 5 THEN INTERVAL '16 hours' \
                     ELSE INTERVAL '24 hours' \
                 END \
             END, \
             resolved_at = CASE WHEN $4 THEN transaction_timestamp() ELSE NULL END, \
             record_id = EXCLUDED.record_id, \
             record_revision_id = EXCLUDED.record_revision_id",
    )
    .bind::<SqlUuid, _>(identifier_quarantine_id)
    .bind::<Text, _>(state.to_string())
    .bind::<Text, _>(actor)
    .bind::<Bool, _>(terminal)
    .bind::<Nullable<SqlUuid>, _>(record_id)
    .bind::<Nullable<SqlUuid>, _>(record_revision_id)
    .execute(connection)?;
    Ok(())
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
