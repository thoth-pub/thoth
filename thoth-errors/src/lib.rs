mod database_errors;

use core::convert::From;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// A specialised result type for returning Thoth data
pub type ThothResult<T> = Result<T, ThothError>;

#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Represents anything that can go wrong in Thoth
///
/// This type is not intended to be exhaustively matched, and new variants may
/// be added in the future without a major version bump.
pub enum ThothError {
    #[error("{input:?} is not a valid {subject_type:?} code")]
    InvalidSubjectCode { input: String, subject_type: String },
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("{0}")]
    DatabaseConstraintError(Cow<'static, str>),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid credentials.")]
    Unauthorised,
    #[error("Failed to validate token.")]
    InvalidToken,
    #[error("No record was found for the given ID.")]
    EntityNotFound,
    #[error("Issue's Work and Series cannot have different Imprints.")]
    IssueImprintsError,
    #[error("{0} is not a valid metadata specification")]
    InvalidMetadataSpecification(String),
    #[error("Invalid UUID supplied.")]
    InvalidUuid,
    #[error("Invalid timestamp supplied.")]
    InvalidTimestamp,
    #[error("CSV Error: {0}")]
    CsvError(String),
    #[error("MARC Error: {0}")]
    MarcError(String),
    #[error("Could not generate {0}: {1}")]
    IncompleteMetadataRecord(String, String),
    #[error("The metadata record has not yet been generated.")]
    MetadataRecordNotGenerated,
    #[error("{0} is not a validly formatted ORCID and will not be saved")]
    OrcidParseError(String),
    #[error("{0} is not a validly formatted DOI and will not be saved")]
    DoiParseError(String),
    #[error("{0} is not a validly formatted ISBN and will not be saved")]
    IsbnParseError(String),
    #[error("{0} is not a validly formatted ROR ID and will not be saved")]
    RorParseError(String),
    #[error("Cannot parse ORCID: no value provided")]
    OrcidEmptyError,
    #[error("Cannot parse DOI: no value provided")]
    DoiEmptyError,
    #[error("Cannot parse ISBN: no value provided")]
    IsbnEmptyError,
    #[error("Cannot parse ROR ID: no value provided")]
    RorEmptyError,
    #[error("Works of type Book Chapter cannot have ISBNs in their Publications.")]
    ChapterIsbnError,
    #[error("Works of type Book Chapter cannot have book-level metadata records.")]
    ChapterBookMetadataError,
    #[error(
        "Works of type Book Chapter cannot have Width, Height, Depth or Weight in their Publications."
    )]
    ChapterDimensionError,
    #[error("Each Publication must have exactly one canonical Location.")]
    CanonicalLocationError,
    #[error(
        "Canonical Locations for digital Publications must have both a Landing Page and a Full Text URL."
    )]
    LocationUrlError,
    #[error("When specifying Weight, both values (g and oz) must be supplied.")]
    WeightEmptyError,
    #[error("When specifying Width, both values (mm and in) must be supplied.")]
    WidthEmptyError,
    #[error("When specifying Height, both values (mm and in) must be supplied.")]
    HeightEmptyError,
    #[error("When specifying Depth, both values (mm and in) must be supplied.")]
    DepthEmptyError,
    #[error(
        "Width/Height/Depth/Weight are only applicable to physical (Paperback/Hardback) Publications."
    )]
    DimensionDigitalError,
    #[error(
        "Price values must be greater than zero. To indicate an unpriced Publication, omit all Prices."
    )]
    PriceZeroError,
    #[error("Publication Date is required for Active, Withdrawn, and Superseded Works.")]
    PublicationDateError,
    #[error("{0}")]
    RequestError(String),
    #[error("{0}")]
    GraphqlError(String),
    #[error("Withdrawn Date must be later than Publication Date.")]
    WithdrawnDateBeforePublicationDateError,
    #[error("Withdrawn Date can only be added to a Superseded or Withdrawn Work.")]
    WithdrawnDateError,
    #[error("A Superseded or Withdrawn Work must have a Withdrawn Date.")]
    NoWithdrawnDateError,
    #[error("Only superusers can create, edit, or delete Locations where the Location Platform is Thoth.")]
    ThothLocationError,
    #[error("Only superusers can update the canonical location when Thoth Location Platform is already set as canonical.")]
    ThothUpdateCanonicalError,
    #[error("Once a Work has been published, it cannot be unpublished. Please use the Withdrawn or Superseded status instead.")]
    ThothSetWorkStatusError,
    #[error("Once a Work has been published, it cannot be deleted.")]
    ThothDeleteWorkError,
    #[error("Publications belonging to a published Work cannot be deleted.")]
    ThothDeletePublicationError,
    #[error("If canonical abstract already exists, other abstract can't be set as canonical.")]
    CanonicalAbstractExistsError,
    #[error("Short abstract must be less than 350 characters.")]
    ShortAbstractLimitExceedError,
    #[error("If canonical biography already exists, other biography can't be set as canonical.")]
    CanonicalBiographyExistsError,
    #[error("If canonical title already exists, other title can't be set as canonical.")]
    CanonicalTitleExistsError,
    #[error("If file extension is not found, the file format is not supported.")]
    NoFileExtensionFound,
    #[error("Unsupported file format")]
    UnsupportedFileFormatError,
    #[error("Content tag not found")]
    TagNotFoundError,
    #[error("Title content cannot contain multiple top-level elements.")]
    TitleMultipleTopLevelElementsError,
    #[error("Title content cannot contain list item elements.")]
    TitleListItemError,
    #[error("Markup format was not provided.")]
    MissingMarkupFormat,
    #[error("Invalid file extension")]
    InvalidFileExtension,
    #[error("Invalid file MIME type")]
    InvalidFileMimeType,
    #[error("File size is below the minimum allowed")]
    FileTooSmall,
    #[error("File size exceeds the maximum allowed")]
    FileTooLarge,
    #[error("File uploads not supported for publication type")]
    UnsupportedPublicationTypeForFileUpload,
    #[error("File uploads not supported for this additional resource type")]
    UnsupportedResourceTypeForFileUpload,
    #[error("Publication type required for publication file validation")]
    PublicationTypeRequiredForFileValidation,
    #[error("File must reference exactly one scope: work_id, publication_id, additional_resource_id, or work_featured_video_id")]
    FileMissingWorkOrPublicationId,
    #[error("FileUpload must reference exactly one scope: work_id, publication_id, additional_resource_id, or work_featured_video_id")]
    FileUploadMissingWorkOrPublicationId,
    #[error("Work must have a DOI to upload files")]
    WorkMissingDoiForFileUpload,
    #[error("Publication file upload missing publication_id")]
    PublicationFileUploadMissingPublicationId,
    #[error("Frontcover file upload missing work_id")]
    FrontcoverFileUploadMissingWorkId,
    #[error("Additional resource file upload missing additional_resource_id")]
    AdditionalResourceFileUploadMissingAdditionalResourceId,
    #[error("Work featured video file upload missing work_featured_video_id")]
    WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId,
    #[error("Only superusers can add a Location Checksum.")]
    CreateLocationChecksumError,
    #[error("Only superusers can update or delete an existing Location Checksum.")]
    UpdateLocationChecksumError,
    #[error("{0} is not currently available for publisher distribution assignment.")]
    DistributionPlatformNotAssignable(String),
    /// The caller's `expectedUpdatedAt` did not match the stored publisher
    /// service-configuration version.
    ///
    /// The message deliberately carries no SQL, table name, column name, driver
    /// text or the current stored token: disclosing the current version to a
    /// caller that has just failed a version check would let it blind-write over
    /// a change it never read. The caller re-reads the configuration instead.
    #[error(
        "The publisher service configuration changed since it was read. Reload it and try again."
    )]
    StalePublisherServiceConfiguration,
    /// The presented distribution-job claim token is not the job's current
    /// token, or the job is not `RUNNING`.
    ///
    /// The message deliberately discloses neither the current token, nor the
    /// current holder, nor whether one exists. "Not claimed" and "held by
    /// another worker" report identically on purpose: telling a caller that
    /// another worker currently holds the job is exactly the information that
    /// makes a stale caller retry aggressively.
    #[error("The distribution job claim is no longer valid.")]
    StaleDistributionJobClaim,
    /// The distribution job is `SUCCEEDED`, `FAILED` or `CANCELLED`.
    ///
    /// The payload carries the current status code, which the caller is
    /// entitled to know: an automated worker must be able to distinguish "this
    /// job is finished, stop" from "someone else owns this now, stop".
    #[error("The distribution job is already in the terminal state {0}.")]
    DistributionJobAlreadyTerminal(String),
    /// A configuration transaction would have produced a new activation
    /// requiring a durable onboarding job while automatic creation is `OFF`.
    ///
    /// The message states the operational fact and nothing more: no SQL, table
    /// name, column name, driver text, environment-variable value, environment
    /// name, deployment identifier or platform credential. The switch's *name*
    /// is the operator-facing control and belongs in the runbook, not in this
    /// payload.
    #[error(
        "Automatic distribution job creation is disabled, so this platform activation cannot be saved."
    )]
    DistributionJobCreationDisabled,
    /// A worker supplied a classification code outside the published shape.
    ///
    /// The message is a fixed string with no interpolation: it must not echo
    /// the submitted value, quote any part of it, report its length, or restate
    /// the pattern with the offending characters. The malformed value is the
    /// caller's, is unbounded, and is never reflected.
    #[error("The supplied distribution job error code is not a valid classification code.")]
    InvalidDistributionJobErrorCode,
    /// The `MIG-01` input manifest, or the reviewed plan, could not be read,
    /// parsed or version-validated. These administrative inputs are operator-
    /// supplied files, not GraphQL/API input, so this variant never reaches a
    /// public API surface.
    #[error("MIG-01 manifest/plan is invalid: {0}")]
    MigrationBackfillManifestInvalid(String),
    /// A `MIG-01` manifest maps the same canonical publisher more than once, so a
    /// unique desired state cannot be resolved. Surfaced rather than guessed.
    #[error("MIG-01 mapping is ambiguous: {0}")]
    MigrationBackfillAmbiguousMapping(String),
    /// A `MIG-01` manifest names a publisher that does not resolve to a canonical
    /// Thoth publisher identity.
    #[error("MIG-01 manifest references an unmatched publisher: {0}")]
    MigrationBackfillUnmatchedPublisher(String),
    /// The reviewed `MIG-01` plan's raw-byte SHA-256 did not equal the expected
    /// reviewed plan SHA-256. The apply stops before parsing for execution.
    #[error("The MIG-01 reviewed plan hash does not match the expected reviewed plan hash.")]
    MigrationBackfillPlanHashMismatch,
    /// The raw `MIG-01` manifest SHA-256 did not equal the value recorded in the
    /// reviewed plan.
    #[error("The MIG-01 manifest hash does not match the hash recorded in the reviewed plan.")]
    MigrationBackfillManifestHashMismatch,
    /// The supplied `MIG-01` plan is semantically parseable but not in canonical
    /// bytes (byte-order mark, insignificant whitespace or a noncanonical
    /// equivalent encoding). It is rejected before any write.
    #[error("The MIG-01 reviewed plan is not in canonical byte form and was rejected.")]
    MigrationBackfillNoncanonicalPlan,
    /// A reviewed `MIG-01` plan entry classified as drift during resume
    /// classification. The invocation stops before any new write; recovery is
    /// forward repair under a fresh reviewed and authorized plan.
    #[error("MIG-01 apply stopped on drift: {0}")]
    MigrationBackfillDrift(String),
    /// The strict production job-state precondition failed: automatic
    /// distribution-job creation is not effectively `OFF`, or a distribution-job
    /// table is non-empty. The apply stops before any write.
    #[error("MIG-01 production precondition failed: {0}")]
    MigrationBackfillProductionPrecondition(String),
    /// A pending publisher's current work count exceeds the approved per-publisher
    /// lock envelope. The apply stops before writing that publisher.
    #[error("MIG-01 lock envelope exceeded: {0}")]
    MigrationBackfillLockEnvelopeExceeded(String),
    /// A production `MIG-01` apply observed a catalogue licence value that is not
    /// reviewed as supported (unreviewed, or carrying a disposition that requires
    /// a separate normalization/repair action). The apply stops before any write;
    /// it never rewrites a licence value.
    #[error("MIG-01 production apply blocked by unresolved licence state: {0}")]
    MigrationBackfillUnresolvedLicence(String),
    /// Two `MIG-01` input/output artifact paths resolve to the same filesystem
    /// location, which could destroy a reviewed manifest or plan needed for
    /// deterministic recovery. The invocation is rejected before any read or
    /// write, leaving inputs untouched.
    #[error("MIG-01 artifact paths alias: {0}")]
    MigrationBackfillArtifactAlias(String),
    /// The exact Gate-D-reviewed `MIG-01` dry-run reconciliation report supplied
    /// to a production apply did not match the expected reviewed evidence: a
    /// raw-byte hash mismatch, an unparseable or wrong-mode report, or a
    /// manifest/plan identity mismatch. The apply stops before any write.
    #[error("MIG-01 reviewed dry-run report does not match the expected evidence: {0}")]
    MigrationBackfillReviewedReportMismatch(String),
    /// The current publisher-omission evidence differs from the exact
    /// Gate-D-reviewed dry-run report, so the reviewed production snapshot
    /// changed between review and apply. The apply stops before any write;
    /// recovery is a fresh dry run, Gate-D review and Gate-E authorization.
    #[error("MIG-01 production apply blocked by changed omission evidence: {0}")]
    MigrationBackfillOmissionMismatch(String),
    // ---- BE-06: work-level upsert and Crossref write permits (#848; R52B section 25.1
    // as amended by Amendment 3 section 10). Every message is fixed and echoes no
    // caller value; each code has its own `IntoFieldError` arm.
    #[error("The work-level execution profile is not implemented.")]
    WorkUpsertProfileNotImplemented,
    #[error("At least one work-level execution profile is required.")]
    WorkUpsertExecutionProfilesRequired,
    #[error("The work-level execution profile is not admitted for this binding.")]
    WorkUpsertProfileNotAdmitted,
    #[error("Work-level execution is not permitted for this work.")]
    WorkUpsertExecutionNotPermitted,
    #[error("The work-level job cannot be completed before its attempt passed the fence.")]
    WorkUpsertCompletionRequiresFence,
    #[error(
        "The work-level job cannot be completed without an accepted write permit for its attempt."
    )]
    WorkUpsertCompletionRequiresAcceptedPermit,
    #[error("The work-level job cannot be cancelled while its current attempt is fenced.")]
    WorkUpsertCancellationRefusedFencedAttempt,
    #[error("Work-level execution is blocked by an uncleared fenced abandonment.")]
    WorkUpsertRecoveryBlocked,
    #[error("Work-level capture cannot be disabled once enabled.")]
    WorkUpsertCaptureIsMonotone,
    #[error("A work-level control row cannot be removed.")]
    WorkUpsertControlRowIsPermanent,
    #[error("A work-level control row's profile cannot be changed.")]
    WorkUpsertControlKeyImmutable,
    #[error("A work-level admission cannot be changed.")]
    WorkUpsertAdmissionImmutable,
    #[error("A work-level admission is removed only with its publisher.")]
    WorkUpsertAdmissionDeleteOnlyByPublisherCascade,
    #[error("The work-level admission census is not empty.")]
    WorkUpsertAdmissionCensusNotEmpty,
    #[error("The work-level source generation cannot be advanced further.")]
    WorkUpsertGenerationOverflow,
    #[error("A requested distribution job kind cannot be claimed by this operation.")]
    DistributionJobKindNotClaimable,
    #[error("A distribution job's work identity cannot be changed.")]
    DistributionJobWorkIdentityImmutable,
    #[error("A distribution job's cleared work reference cannot be restored.")]
    DistributionJobWorkReferenceNotRestorable,
    #[error("The work cannot be deleted while a work-level attempt for it is fenced.")]
    WorkDeleteBlockedByFencedAttempt,
    #[error("The work's binding changed during deletion.")]
    WorkDeleteBindingDrift,
    #[error("The work's binding kept changing during deletion; retry later.")]
    WorkDeleteBindingDriftUnresolved,
    #[error("The Crossref root work does not exist.")]
    CrossrefRootWorkNotFound,
    #[error("The work's publisher has no enabled Crossref assignment.")]
    CrossrefPublisherNotCovered,
    #[error("The work's binding changed; retry later.")]
    CrossrefBindingMovedRetry,
    #[error("A blocking Crossref write permit overlaps this deposit.")]
    CrossrefPermitBlocked,
    #[error("The deposit has no DOI to register.")]
    CrossrefPermitEmptyDoiSet,
    #[error("A stored DOI of the deposit cannot be canonicalised.")]
    CrossrefDoiNotCanonicalisable,
    #[error("The work was already deposited within this job.")]
    CrossrefUnitAlreadyDepositedInJob,
    #[error("A manual recovery reservation requires an operator authorization reference.")]
    CrossrefManualRecoveryRequiresReference,
    #[error("The claim for this Crossref write is no longer valid.")]
    CrossrefPermitClaimStale,
    #[error("A Crossref write permit must be created reserved.")]
    CrossrefPermitInitialStateInvalid,
    #[error("Crossref write permit evidence cannot be changed.")]
    CrossrefPermitEvidenceImmutable,
    #[error("A Crossref write permit link can only be cleared.")]
    CrossrefPermitLinkNotRestorable,
    #[error("A write-once Crossref write permit field cannot be written here.")]
    CrossrefPermitWriteOnceField,
    #[error("The Crossref write permit transition is not allowed.")]
    CrossrefPermitIllegalTransition,
    #[error("Authorizing a Crossref write requires its payload manifest.")]
    CrossrefPermitAuthorizationRequiresManifest,
    #[error("Authorizing a work-level Crossref write requires a fenced open attempt.")]
    CrossrefPermitAuthorizationRequiresFence,
    #[error("The work changed after the Crossref deposit was prepared.")]
    CrossrefArtifactSourceChanged,
    #[error("Crossref write permits cannot be deleted.")]
    CrossrefPermitDeleteRefused,
    #[error("Crossref write permit membership cannot be changed.")]
    CrossrefPermitMembershipImmutable,
    #[error("Crossref write permit membership does not agree with its digest.")]
    CrossrefPermitMembershipCardinalityMismatch,
    #[error("The Crossref write permit does not exist.")]
    CrossrefPermitNotFound,
    #[error("The Crossref write permit reservation token does not match.")]
    CrossrefPermitRequiresReservationToken,
    #[error("Only a reserved Crossref write permit can be voided.")]
    CrossrefPermitVoidRequiresReserved,
    #[error("Voiding a Crossref write reservation requires a detail.")]
    CrossrefPermitVoidRequiresDetail,
    #[error(
        "Voiding a Crossref write reservation as superuser requires an authorization reference."
    )]
    CrossrefVoidRequiresAuthorizationReference,
    #[error("Reconciling a Crossref write permit requires an authorization reference.")]
    CrossrefReconciliationRequiresReference,
    #[error("The Crossref deposit timestamp is not increasing.")]
    CrossrefTimestampNotIncreasing,
    #[error("The Crossref deposit timestamp is not a valid encoded instant.")]
    CrossrefTimestampNotDecodable,
    #[error("The Crossref deposit timestamp is outside its domain.")]
    CrossrefTimestampOverflow,
    #[error("The Crossref version floor cannot be lowered.")]
    CrossrefVersionFloorNotDecreasing,
    #[error("The Crossref version floor value is outside its domain.")]
    CrossrefVersionFloorDomain,
    #[error("The version floor cannot advance while a blocking Crossref write permit exists.")]
    CrossrefVersionFloorNotDrained,
    #[error("This G-6 attempt has already advanced the version floor.")]
    CrossrefVersionFloorAlreadyAdvanced,
    #[error("The Crossref version floor cannot be removed.")]
    CrossrefVersionFloorPermanent,
    #[error("The Crossref version floor audit is append-only.")]
    CrossrefVersionFloorAuditAppendOnly,
    #[error(
        "The attempt holds an authorized Crossref write permit whose outcome is not reported."
    )]
    AttemptHasAuthorizedPermit,
    #[error("The attempt holds an open Crossref write reservation.")]
    AttemptHasOpenReservation,
    #[error("The attempt holds unresolved Crossref write permits.")]
    OuterAttemptHasOpenPermits,
    #[error("The requested version floor target is not the approved target.")]
    CrossrefVersionFloorTargetInvalid,
    #[error("A version floor advance requires a non-blank authorization reference.")]
    CrossrefVersionFloorRequiresAuthorizationReference,
    #[error("The authorization register digest must be 64 lowercase hexadecimal characters.")]
    CrossrefVersionFloorRegisterDigestInvalid,
    #[error("The version floor is not at the value this advance is bound to.")]
    CrossrefVersionFloorBindingMismatch,
    #[error("Work-level capture is not enabled.")]
    WorkUpsertCaptureNotEnabled,
    #[error("A work-level admission requires an evidence reference.")]
    WorkUpsertAdmissionRequiresEvidenceReference,
    #[error("The claimed job is not of this reservation route's kind.")]
    CrossrefReservationJobKindMismatch,
    #[error("The work does not belong to the job's publisher.")]
    CrossrefUnitPublisherMismatch,
    #[error("A Crossref write permit already exists for this attempt.")]
    CrossrefPermitAttemptAlreadyReserved,
    #[error("The payload digest is not a lower-case SHA-256 hex digest.")]
    CrossrefPayloadDigestInvalid,
    /// A BE-06 database operation failed in a way no exact code describes.
    ///
    /// Deliberately has no `IntoFieldError` arm: it reaches GraphQL as
    /// `INTERNAL_ERROR` with this fixed message, and nothing about the SQL, the
    /// constraint, the driver or the pool reaches the caller (Amendment 3
    /// section 10.3). The underlying error is recorded server-side instead.
    #[error("A work-level distribution database operation failed.")]
    WorkUpsertDatabaseFailure,
}

impl ThothError {
    /// Serialise to JSON
    pub fn to_json(&self) -> ThothResult<String> {
        serde_json::to_string(&self).map_err(Into::into)
    }

    /// Deserialise from JSON
    pub fn from_json(s: &str) -> ThothResult<ThothError> {
        serde_json::from_str(s).map_err(Into::into)
    }
}

/// The codes BE-06 Migration 2 raises from its triggers and functions that are
/// BE-06 error codes (Amendment 3 section 10.3). The target-set invariant
/// message `WORK_UPSERT_TARGET_SET_MISMATCH` is deliberately not one of them.
pub const WORK_UPSERT_TRIGGER_CODES: [&str; 28] = [
    "CROSSREF_ARTIFACT_SOURCE_CHANGED",
    "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE",
    "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_MANIFEST",
    "CROSSREF_PERMIT_BLOCKED",
    "CROSSREF_PERMIT_DELETE_REFUSED",
    "CROSSREF_PERMIT_EVIDENCE_IMMUTABLE",
    "CROSSREF_PERMIT_ILLEGAL_TRANSITION",
    "CROSSREF_PERMIT_INITIAL_STATE_INVALID",
    "CROSSREF_PERMIT_LINK_NOT_RESTORABLE",
    "CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH",
    "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE",
    "CROSSREF_PERMIT_WRITE_ONCE_FIELD",
    "CROSSREF_RECONCILIATION_REQUIRES_REFERENCE",
    "CROSSREF_TIMESTAMP_NOT_DECODABLE",
    "CROSSREF_TIMESTAMP_NOT_INCREASING",
    "CROSSREF_TIMESTAMP_OVERFLOW",
    "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY",
    "CROSSREF_VERSION_FLOOR_DOMAIN",
    "CROSSREF_VERSION_FLOOR_NOT_DECREASING",
    "CROSSREF_VERSION_FLOOR_PERMANENT",
    "DISTRIBUTION_JOB_WORK_IDENTITY_IMMUTABLE",
    "DISTRIBUTION_JOB_WORK_REFERENCE_NOT_RESTORABLE",
    "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE",
    "WORK_UPSERT_ADMISSION_IMMUTABLE",
    "WORK_UPSERT_CAPTURE_IS_MONOTONE",
    "WORK_UPSERT_CONTROL_KEY_IMMUTABLE",
    "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT",
    "WORK_UPSERT_PROFILE_NOT_IMPLEMENTED",
];

/// The trigger codes Migration 2 also raises with a value appended after `": "`.
pub const WORK_UPSERT_SUFFIXED_TRIGGER_CODES: [&str; 2] = [
    "CROSSREF_TIMESTAMP_NOT_DECODABLE",
    "CROSSREF_TIMESTAMP_OVERFLOW",
];

impl ThothError {
    /// The scoped conversion of a database error raised inside a BE-06
    /// operation (Amendment 3 section 10.3).
    ///
    /// It is deliberately not a `From` impl, so `?` never selects it. An exact
    /// single-purpose CHECK or unique index maps to its code; a trigger-raised
    /// code with no constraint name maps to its variant, whose fixed message
    /// never carries an appended value; everything else is
    /// [`ThothError::WorkUpsertDatabaseFailure`].
    pub fn from_work_upsert_database_error(error: diesel::result::Error) -> ThothError {
        use diesel::result::{DatabaseErrorKind, Error};

        let Error::DatabaseError(kind, info) = error else {
            return ThothError::WorkUpsertDatabaseFailure;
        };
        match (kind, info.constraint_name()) {
            (DatabaseErrorKind::CheckViolation, Some(constraint)) => match constraint {
                "work_upsert_control_execution_requires_capture_check" => {
                    ThothError::WorkUpsertCaptureNotEnabled
                }
                "work_upsert_admission_evidence_reference_check" => {
                    ThothError::WorkUpsertAdmissionRequiresEvidenceReference
                }
                "crossref_write_permit_payload_digest_check" => {
                    ThothError::CrossrefPayloadDigestInvalid
                }
                "work_crossref_version_floor_domain_check" => {
                    ThothError::CrossrefVersionFloorDomain
                }
                _ => ThothError::WorkUpsertDatabaseFailure,
            },
            (DatabaseErrorKind::UniqueViolation, Some(constraint)) => match constraint {
                "crossref_write_permit_one_per_work_upsert_attempt_idx" => {
                    ThothError::CrossrefPermitAttemptAlreadyReserved
                }
                "crossref_version_floor_audit_one_advance_per_g6_attempt_idx" => {
                    ThothError::CrossrefVersionFloorAlreadyAdvanced
                }
                _ => ThothError::WorkUpsertDatabaseFailure,
            },
            (_, Some(_)) => ThothError::WorkUpsertDatabaseFailure,
            (_, None) => {
                let message = info.message();
                let code = if WORK_UPSERT_TRIGGER_CODES.contains(&message) {
                    message
                } else {
                    match message.split_once(": ") {
                        Some((prefix, _))
                            if WORK_UPSERT_SUFFIXED_TRIGGER_CODES.contains(&prefix) =>
                        {
                            prefix
                        }
                        _ => return ThothError::WorkUpsertDatabaseFailure,
                    }
                };
                Self::work_upsert_trigger_error(code)
            }
        }
    }

    /// The variant of one member of [`WORK_UPSERT_TRIGGER_CODES`].
    fn work_upsert_trigger_error(code: &str) -> ThothError {
        match code {
            "CROSSREF_ARTIFACT_SOURCE_CHANGED" => ThothError::CrossrefArtifactSourceChanged,
            "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE" => {
                ThothError::CrossrefPermitAuthorizationRequiresFence
            }
            "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_MANIFEST" => {
                ThothError::CrossrefPermitAuthorizationRequiresManifest
            }
            "CROSSREF_PERMIT_BLOCKED" => ThothError::CrossrefPermitBlocked,
            "CROSSREF_PERMIT_DELETE_REFUSED" => ThothError::CrossrefPermitDeleteRefused,
            "CROSSREF_PERMIT_EVIDENCE_IMMUTABLE" => ThothError::CrossrefPermitEvidenceImmutable,
            "CROSSREF_PERMIT_ILLEGAL_TRANSITION" => ThothError::CrossrefPermitIllegalTransition,
            "CROSSREF_PERMIT_INITIAL_STATE_INVALID" => {
                ThothError::CrossrefPermitInitialStateInvalid
            }
            "CROSSREF_PERMIT_LINK_NOT_RESTORABLE" => ThothError::CrossrefPermitLinkNotRestorable,
            "CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH" => {
                ThothError::CrossrefPermitMembershipCardinalityMismatch
            }
            "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE" => ThothError::CrossrefPermitMembershipImmutable,
            "CROSSREF_PERMIT_WRITE_ONCE_FIELD" => ThothError::CrossrefPermitWriteOnceField,
            "CROSSREF_RECONCILIATION_REQUIRES_REFERENCE" => {
                ThothError::CrossrefReconciliationRequiresReference
            }
            "CROSSREF_TIMESTAMP_NOT_DECODABLE" => ThothError::CrossrefTimestampNotDecodable,
            "CROSSREF_TIMESTAMP_NOT_INCREASING" => ThothError::CrossrefTimestampNotIncreasing,
            "CROSSREF_TIMESTAMP_OVERFLOW" => ThothError::CrossrefTimestampOverflow,
            "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY" => {
                ThothError::CrossrefVersionFloorAuditAppendOnly
            }
            "CROSSREF_VERSION_FLOOR_DOMAIN" => ThothError::CrossrefVersionFloorDomain,
            "CROSSREF_VERSION_FLOOR_NOT_DECREASING" => {
                ThothError::CrossrefVersionFloorNotDecreasing
            }
            "CROSSREF_VERSION_FLOOR_PERMANENT" => ThothError::CrossrefVersionFloorPermanent,
            "DISTRIBUTION_JOB_WORK_IDENTITY_IMMUTABLE" => {
                ThothError::DistributionJobWorkIdentityImmutable
            }
            "DISTRIBUTION_JOB_WORK_REFERENCE_NOT_RESTORABLE" => {
                ThothError::DistributionJobWorkReferenceNotRestorable
            }
            "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE" => {
                ThothError::WorkUpsertAdmissionDeleteOnlyByPublisherCascade
            }
            "WORK_UPSERT_ADMISSION_IMMUTABLE" => ThothError::WorkUpsertAdmissionImmutable,
            "WORK_UPSERT_CAPTURE_IS_MONOTONE" => ThothError::WorkUpsertCaptureIsMonotone,
            "WORK_UPSERT_CONTROL_KEY_IMMUTABLE" => ThothError::WorkUpsertControlKeyImmutable,
            "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT" => ThothError::WorkUpsertControlRowIsPermanent,
            "WORK_UPSERT_PROFILE_NOT_IMPLEMENTED" => ThothError::WorkUpsertProfileNotImplemented,
            _ => ThothError::WorkUpsertDatabaseFailure,
        }
    }
}

impl juniper::IntoFieldError for ThothError {
    fn into_field_error(self) -> juniper::FieldError {
        use juniper::graphql_value;
        match self {
            ThothError::InvalidSubjectCode { .. } => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "INVALID_SUBJECT_CODE"
                }),
            ),
            ThothError::Unauthorised => juniper::FieldError::new(
                "Unauthorized",
                graphql_value!({
                    "type": "NO_ACCESS"
                }),
            ),
            ThothError::StalePublisherServiceConfiguration => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "STALE_SERVICE_CONFIGURATION"
                }),
            ),
            ThothError::StaleDistributionJobClaim => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "STALE_DISTRIBUTION_JOB_CLAIM"
                }),
            ),
            ThothError::DistributionJobAlreadyTerminal(_) => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "DISTRIBUTION_JOB_TERMINAL"
                }),
            ),
            ThothError::DistributionJobCreationDisabled => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "DISTRIBUTION_JOB_CREATION_DISABLED"
                }),
            ),
            ThothError::InvalidDistributionJobErrorCode => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "INVALID_DISTRIBUTION_JOB_ERROR_CODE"
                }),
            ),
            ThothError::WorkUpsertProfileNotImplemented => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_PROFILE_NOT_IMPLEMENTED"
                }),
            ),
            ThothError::WorkUpsertExecutionProfilesRequired => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_EXECUTION_PROFILES_REQUIRED"
                }),
            ),
            ThothError::WorkUpsertProfileNotAdmitted => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_PROFILE_NOT_ADMITTED"
                }),
            ),
            ThothError::WorkUpsertExecutionNotPermitted => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_EXECUTION_NOT_PERMITTED"
                }),
            ),
            ThothError::WorkUpsertCompletionRequiresFence => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_COMPLETION_REQUIRES_FENCE"
                }),
            ),
            ThothError::WorkUpsertCompletionRequiresAcceptedPermit => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_COMPLETION_REQUIRES_ACCEPTED_PERMIT"
                }),
            ),
            ThothError::WorkUpsertCancellationRefusedFencedAttempt => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_CANCELLATION_REFUSED_FENCED_ATTEMPT"
                }),
            ),
            ThothError::WorkUpsertRecoveryBlocked => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_RECOVERY_BLOCKED"
                }),
            ),
            ThothError::WorkUpsertCaptureIsMonotone => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_CAPTURE_IS_MONOTONE"
                }),
            ),
            ThothError::WorkUpsertControlRowIsPermanent => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT"
                }),
            ),
            ThothError::WorkUpsertControlKeyImmutable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_CONTROL_KEY_IMMUTABLE"
                }),
            ),
            ThothError::WorkUpsertAdmissionImmutable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_ADMISSION_IMMUTABLE"
                }),
            ),
            ThothError::WorkUpsertAdmissionDeleteOnlyByPublisherCascade => {
                juniper::FieldError::new(
                    self.to_string(),
                    graphql_value!({
                        "type": "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE"
                    }),
                )
            }
            ThothError::WorkUpsertAdmissionCensusNotEmpty => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_ADMISSION_CENSUS_NOT_EMPTY"
                }),
            ),
            ThothError::WorkUpsertGenerationOverflow => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_GENERATION_OVERFLOW"
                }),
            ),
            ThothError::DistributionJobKindNotClaimable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "DISTRIBUTION_JOB_KIND_NOT_CLAIMABLE"
                }),
            ),
            ThothError::DistributionJobWorkIdentityImmutable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "DISTRIBUTION_JOB_WORK_IDENTITY_IMMUTABLE"
                }),
            ),
            ThothError::DistributionJobWorkReferenceNotRestorable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "DISTRIBUTION_JOB_WORK_REFERENCE_NOT_RESTORABLE"
                }),
            ),
            ThothError::WorkDeleteBlockedByFencedAttempt => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_DELETE_BLOCKED_BY_FENCED_ATTEMPT"
                }),
            ),
            ThothError::WorkDeleteBindingDrift => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_DELETE_BINDING_DRIFT"
                }),
            ),
            ThothError::WorkDeleteBindingDriftUnresolved => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_DELETE_BINDING_DRIFT_UNRESOLVED"
                }),
            ),
            ThothError::CrossrefRootWorkNotFound => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_ROOT_WORK_NOT_FOUND"
                }),
            ),
            ThothError::CrossrefPublisherNotCovered => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PUBLISHER_NOT_COVERED"
                }),
            ),
            ThothError::CrossrefBindingMovedRetry => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_BINDING_MOVED_RETRY"
                }),
            ),
            ThothError::CrossrefPermitBlocked => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_BLOCKED"
                }),
            ),
            ThothError::CrossrefPermitEmptyDoiSet => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_EMPTY_DOI_SET"
                }),
            ),
            ThothError::CrossrefDoiNotCanonicalisable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_DOI_NOT_CANONICALISABLE"
                }),
            ),
            ThothError::CrossrefUnitAlreadyDepositedInJob => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB"
                }),
            ),
            ThothError::CrossrefManualRecoveryRequiresReference => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_MANUAL_RECOVERY_REQUIRES_REFERENCE"
                }),
            ),
            ThothError::CrossrefPermitClaimStale => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_CLAIM_STALE"
                }),
            ),
            ThothError::CrossrefPermitInitialStateInvalid => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_INITIAL_STATE_INVALID"
                }),
            ),
            ThothError::CrossrefPermitEvidenceImmutable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_EVIDENCE_IMMUTABLE"
                }),
            ),
            ThothError::CrossrefPermitLinkNotRestorable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_LINK_NOT_RESTORABLE"
                }),
            ),
            ThothError::CrossrefPermitWriteOnceField => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_WRITE_ONCE_FIELD"
                }),
            ),
            ThothError::CrossrefPermitIllegalTransition => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_ILLEGAL_TRANSITION"
                }),
            ),
            ThothError::CrossrefPermitAuthorizationRequiresManifest => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_MANIFEST"
                }),
            ),
            ThothError::CrossrefPermitAuthorizationRequiresFence => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE"
                }),
            ),
            ThothError::CrossrefArtifactSourceChanged => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_ARTIFACT_SOURCE_CHANGED"
                }),
            ),
            ThothError::CrossrefPermitDeleteRefused => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_DELETE_REFUSED"
                }),
            ),
            ThothError::CrossrefPermitMembershipImmutable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE"
                }),
            ),
            ThothError::CrossrefPermitMembershipCardinalityMismatch => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH"
                }),
            ),
            ThothError::CrossrefPermitNotFound => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_NOT_FOUND"
                }),
            ),
            ThothError::CrossrefPermitRequiresReservationToken => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_REQUIRES_RESERVATION_TOKEN"
                }),
            ),
            ThothError::CrossrefPermitVoidRequiresReserved => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_VOID_REQUIRES_RESERVED"
                }),
            ),
            ThothError::CrossrefPermitVoidRequiresDetail => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_VOID_REQUIRES_DETAIL"
                }),
            ),
            ThothError::CrossrefVoidRequiresAuthorizationReference => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VOID_REQUIRES_AUTHORIZATION_REFERENCE"
                }),
            ),
            ThothError::CrossrefReconciliationRequiresReference => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_RECONCILIATION_REQUIRES_REFERENCE"
                }),
            ),
            ThothError::CrossrefTimestampNotIncreasing => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_TIMESTAMP_NOT_INCREASING"
                }),
            ),
            ThothError::CrossrefTimestampNotDecodable => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_TIMESTAMP_NOT_DECODABLE"
                }),
            ),
            ThothError::CrossrefTimestampOverflow => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_TIMESTAMP_OVERFLOW"
                }),
            ),
            ThothError::CrossrefVersionFloorNotDecreasing => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_NOT_DECREASING"
                }),
            ),
            ThothError::CrossrefVersionFloorDomain => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_DOMAIN"
                }),
            ),
            ThothError::CrossrefVersionFloorNotDrained => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_NOT_DRAINED"
                }),
            ),
            ThothError::CrossrefVersionFloorAlreadyAdvanced => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_ALREADY_ADVANCED"
                }),
            ),
            ThothError::CrossrefVersionFloorPermanent => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_PERMANENT"
                }),
            ),
            ThothError::CrossrefVersionFloorAuditAppendOnly => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY"
                }),
            ),
            ThothError::AttemptHasAuthorizedPermit => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "ATTEMPT_HAS_AUTHORIZED_PERMIT"
                }),
            ),
            ThothError::AttemptHasOpenReservation => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "ATTEMPT_HAS_OPEN_RESERVATION"
                }),
            ),
            ThothError::OuterAttemptHasOpenPermits => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "OUTER_ATTEMPT_HAS_OPEN_PERMITS"
                }),
            ),
            ThothError::CrossrefVersionFloorTargetInvalid => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_TARGET_INVALID"
                }),
            ),
            ThothError::CrossrefVersionFloorRequiresAuthorizationReference => {
                juniper::FieldError::new(
                    self.to_string(),
                    graphql_value!({
                        "type": "CROSSREF_VERSION_FLOOR_REQUIRES_AUTHORIZATION_REFERENCE"
                    }),
                )
            }
            ThothError::CrossrefVersionFloorRegisterDigestInvalid => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_REGISTER_DIGEST_INVALID"
                }),
            ),
            ThothError::CrossrefVersionFloorBindingMismatch => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_VERSION_FLOOR_BINDING_MISMATCH"
                }),
            ),
            ThothError::WorkUpsertCaptureNotEnabled => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_CAPTURE_NOT_ENABLED"
                }),
            ),
            ThothError::WorkUpsertAdmissionRequiresEvidenceReference => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "WORK_UPSERT_ADMISSION_REQUIRES_EVIDENCE_REFERENCE"
                }),
            ),
            ThothError::CrossrefReservationJobKindMismatch => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_RESERVATION_JOB_KIND_MISMATCH"
                }),
            ),
            ThothError::CrossrefUnitPublisherMismatch => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_UNIT_PUBLISHER_MISMATCH"
                }),
            ),
            ThothError::CrossrefPermitAttemptAlreadyReserved => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED"
                }),
            ),
            ThothError::CrossrefPayloadDigestInvalid => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "CROSSREF_PAYLOAD_DIGEST_INVALID"
                }),
            ),
            _ => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "INTERNAL_ERROR"
                }),
            ),
        }
    }
}

impl actix_web::error::ResponseError for ThothError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::HttpResponse;
        match self {
            ThothError::Unauthorised | ThothError::InvalidToken => {
                HttpResponse::Unauthorized().json(self.to_string())
            }
            ThothError::EntityNotFound => HttpResponse::NotFound().json(self.to_string()),
            ThothError::InvalidMetadataSpecification(_) | ThothError::InvalidUuid => {
                HttpResponse::BadRequest().json(self.to_string())
            }
            ThothError::DatabaseError { .. } => {
                HttpResponse::InternalServerError().json("DB error")
            }
            ThothError::RedisError { .. } => {
                HttpResponse::InternalServerError().json("Redis error")
            }
            ThothError::IncompleteMetadataRecord(_, _) => {
                HttpResponse::NotFound().json(self.to_string())
            }
            _ => HttpResponse::InternalServerError().json(self.to_string()),
        }
    }
}

impl From<csv::Error> for ThothError {
    fn from(e: csv::Error) -> Self {
        ThothError::CsvError(e.to_string())
    }
}

impl From<std::io::Error> for ThothError {
    fn from(error: std::io::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<&std::io::Error> for ThothError {
    fn from(error: &std::io::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<reqwest::Error> for ThothError {
    fn from(error: reqwest::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<reqwest_middleware::Error> for ThothError {
    fn from(error: reqwest_middleware::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<xml::writer::Error> for ThothError {
    fn from(error: xml::writer::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<uuid::Error> for ThothError {
    fn from(_: uuid::Error) -> ThothError {
        ThothError::InvalidUuid
    }
}

impl From<marc::Error> for ThothError {
    fn from(e: marc::Error) -> Self {
        ThothError::MarcError(e.to_string())
    }
}

impl From<dialoguer::Error> for ThothError {
    fn from(e: dialoguer::Error) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<chrono::ParseError> for ThothError {
    fn from(_: chrono::ParseError) -> Self {
        ThothError::InvalidTimestamp
    }
}

impl From<deadpool_redis::redis::RedisError> for ThothError {
    fn from(e: deadpool_redis::redis::RedisError) -> Self {
        ThothError::RedisError(e.to_string())
    }
}

impl From<deadpool_redis::PoolError> for ThothError {
    fn from(e: deadpool_redis::PoolError) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ThothError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ThothError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<tonic::Status> for ThothError {
    fn from(e: tonic::Status) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<serde_json::Error> for ThothError {
    fn from(e: serde_json::Error) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_error() {
        // We are just testing that _a_ `csv::error` is converted to `ThothError::CsvError`.
        // The test instantiation is copied from the library: https://github.com/BurntSushi/rust-csv/blob/40ea4c49d7467d2b607a6396424f8e0e101adae1/src/writer.rs#L1268
        let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
        wtr.write_record(&csv::ByteRecord::from(vec!["a", "b", "c"]))
            .unwrap();
        let err = wtr
            .write_record(&csv::ByteRecord::from(vec!["a"]))
            .unwrap_err();
        assert!(matches!(ThothError::from(err), ThothError::CsvError { .. }));
    }

    #[test]
    fn test_uuid_error() {
        assert_eq!(
            ThothError::from(uuid::Uuid::parse_str("not-a-uuid").unwrap_err()),
            ThothError::InvalidUuid
        );
    }

    #[test]
    fn test_round_trip_serialisation() {
        let original_error = ThothError::InvalidSubjectCode {
            input: "002".to_string(),
            subject_type: "BIC".to_string(),
        };
        let json = original_error.to_json().unwrap();
        let deserialised_error = ThothError::from_json(&json).unwrap();
        assert_eq!(original_error, deserialised_error);
    }

    #[test]
    fn test_to_json_valid_error() {
        let error = ThothError::InvalidSubjectCode {
            input: "001".to_string(),
            subject_type: "BIC".to_string(),
        };
        let json = error.to_json().unwrap();

        assert!(json.contains("\"InvalidSubjectCode\""));
        assert!(json.contains("\"001\""));
        assert!(json.contains("\"BIC\""));
    }

    #[test]
    fn test_invalid_json_deserialisation() {
        let invalid_json = r#"{"UnknownError":"Unexpected field"}"#;
        let error = ThothError::from_json(invalid_json);
        assert!(error.is_err());
    }
}
