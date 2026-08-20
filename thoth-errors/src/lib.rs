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
    /// A `MIG-01-LIC-NORM-01` immutable input artifact (deterministic
    /// normalization manifest, manual-resolution register, bound MIG-01
    /// manifest, reviewed plan or reviewed dry-run report) could not be read,
    /// parsed or validated against the reviewed mechanical invariants. These
    /// administrative inputs are operator-supplied files, not GraphQL/API
    /// input, so this variant never reaches a public API surface.
    #[error("MIG-01-LIC-NORM input is invalid: {0}")]
    LicenceNormalizationInvalidInput(String),
    /// A `MIG-01-LIC-NORM-01` artifact's exact raw-byte SHA-256 did not equal
    /// the expected reviewed hash. The message names only the artifact role;
    /// it never echoes artifact content.
    #[error("MIG-01-LIC-NORM artifact hash mismatch: {0}")]
    LicenceNormalizationHashMismatch(String),
    /// The supplied `MIG-01-LIC-NORM-01` plan is semantically parseable but not
    /// in canonical bytes (byte-order mark, insignificant whitespace, entry
    /// disorder or a noncanonical equivalent encoding, including a differing
    /// timestamp representation). It is rejected before any write.
    #[error("The MIG-01-LIC-NORM reviewed plan is not in canonical byte form and was rejected.")]
    LicenceNormalizationNoncanonicalPlan,
    /// An affected Work's publisher is absent from the bound MIG-01 production
    /// manifest. This is a blocking scope mismatch: remediation is an approved
    /// MIG-01 manifest/programme amendment with a newly frozen hash and fresh
    /// review, never a local omission or waiver.
    #[error("MIG-01-LIC-NORM publisher scope mismatch: {0}")]
    LicenceNormalizationScopeMismatch(String),
    /// A deterministic normalization target failed canonical `cc-license`
    /// parsing or is not exact-string `SUPPORTED` in the bound MIG-01
    /// production manifest. The run stops before plan emission or writes.
    #[error("MIG-01-LIC-NORM target is not reviewed as supported: {0}")]
    LicenceNormalizationUnsupportedTarget(String),
    /// A reviewed `MIG-01-LIC-NORM-01` plan entry classified as `DRIFT`. The
    /// invocation stops before any new write; recovery is deterministic resume
    /// under a fresh reviewed plan or separately authorized forward repair.
    #[error("MIG-01-LIC-NORM stopped on drift: {0}")]
    LicenceNormalizationDrift(String),
    /// A Work absent from the reviewed plan currently carries a deterministic
    /// source value, so the reviewed plan no longer covers the exact current
    /// source-value membership. The apply stops before any write.
    #[error("MIG-01-LIC-NORM apply stopped on an unplanned Work: {0}")]
    LicenceNormalizationUnplannedWork(String),
    /// Two `MIG-01-LIC-NORM-01` input/output artifact paths resolve to the same
    /// filesystem location, which could destroy a reviewed input needed for
    /// deterministic recovery. Rejected before any read or write.
    #[error("MIG-01-LIC-NORM artifact paths alias: {0}")]
    LicenceNormalizationArtifactAlias(String),
    /// The post-update in-transaction re-read did not observe exactly the
    /// reviewed target licence. The surrounding transaction rolls back the
    /// history row and the licence update together.
    #[error("MIG-01-LIC-NORM write verification failed: {0}")]
    LicenceNormalizationWriteVerification(String),
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
