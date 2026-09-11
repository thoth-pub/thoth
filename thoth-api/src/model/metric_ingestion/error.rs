//! The closed internal error and reason vocabulary of the canonical ingestion
//! coordinator (`MET-WP2-01B`).
//!
//! Every code is a stable SCREAMING_SNAKE_CASE identifier. The same vocabulary
//! is used for request-level failures (which abort the whole batch and commit
//! nothing), for per-observation `REJECTED` reasons stored in provenance
//! details and `metric_import_error.error_code`, and for the `CONFLICT` reason.
//! Which scope a code has is fixed by the specification and documented on each
//! variant; the vocabulary itself is shared so that no second competing code
//! set exists.
//!
//! Nothing here carries SQL, a constraint name, a connection string, a secret,
//! an IP address, a user agent or raw upstream data: a database failure is
//! reduced to [`MetricIngestionErrorCode::InternalDatabaseError`] before it
//! leaves the coordinator. Public GraphQL error mapping belongs to
//! `MET-WP2-02`, not to this module.

use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// The closed inventory of coordinator error and reason codes.
///
/// There is deliberately no `OTHER` or `UNKNOWN` variant and no `Default`: an
/// unrecognised persisted reason code must fail to parse rather than resolve to
/// a nearest classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricIngestionErrorCode {
    // ---------------------------------------------------------------------
    // Request shape. Request-level: no durable consequence.
    // ---------------------------------------------------------------------
    /// Both the observation list and the coverage list were empty.
    EmptyBatch,
    /// `batch_key` was blank or exceeded 256 UTF-8 bytes.
    InvalidBatchKey,
    /// More than 500 observations or more than 100 coverage assertions.
    BatchLimitExceeded,
    /// `schema_version` was not exactly `thoth-normalized-metrics/1`.
    UnsupportedSchemaVersion,

    // ---------------------------------------------------------------------
    // Import and idempotency. Request-level: no durable consequence.
    // ---------------------------------------------------------------------
    /// No `metric_import` row exists for the supplied `import_id`.
    ImportNotFound,
    /// A first-time batch requires `metric_import.status = PROCESSING`.
    ImportNotProcessing,
    /// Reserved. This code has **no reachable emission condition** in the
    /// `MET-WP2-01B` DRIVER coordinator (Amendment 5 D1): the import's
    /// `source_account_id` is the authoritative batch source account, a
    /// differing per-observation `source_account_code` is
    /// [`SourceAccountMismatch`](Self::SourceAccountMismatch), and an
    /// import/account publisher disagreement is
    /// [`PublisherScopeMismatch`](Self::PublisherScopeMismatch). A later
    /// transport that introduces an independent batch-level source-scope
    /// assertion may assign semantics to it only through separately approved
    /// specification work.
    ImportSourceScopeMismatch,
    /// The same `(import_id, batch_key)` already committed under a different
    /// request hash.
    IdempotencyKeyReused,
    /// Persisted state contradicted a persistence invariant: a replayed batch
    /// whose provenance cardinality or order does not match the submitted
    /// observations, an import whose referenced source account, source,
    /// platform or publisher cannot be read, or a canonical record with no
    /// current revision.
    InternalStateInconsistency,
    /// A database or connection failure. The underlying detail is never
    /// surfaced.
    InternalDatabaseError,
    /// Three first-time transaction attempts each observed metadata-authority
    /// drift between provisional discovery and the locked re-resolution
    /// barrier (Amendment 4 C6). No batch row exists, so the same request may
    /// be retried.
    ConcurrentAuthorityChange,

    // ---------------------------------------------------------------------
    // Source authority. Request-level unless stated.
    // ---------------------------------------------------------------------
    /// The import's source is not a `DRIVER` source. `PUBLISHER_UPLOAD`,
    /// `OPERAS` and `ADMIN_IMPORT` ingestion is deferred beyond MOM-1 and fails
    /// closed here.
    AcquisitionTypeDeferred,
    /// Per-observation: `source_account_code` is not exactly the import source
    /// account's stable `code`.
    SourceAccountMismatch,
    /// The import's source account is disabled.
    SourceAccountDisabled,
    /// The import's source is disabled.
    SourceDisabled,

    // ---------------------------------------------------------------------
    // Platform, measure and mapping. Per-observation `REJECTED`; request-level
    // when raised by a coverage assertion.
    // ---------------------------------------------------------------------
    /// `platform_code` is not exactly the import account's platform code.
    PlatformMismatch,
    /// The account's authoritative platform is disabled.
    PlatformDisabled,
    /// No `metric_measure` row carries the supplied `measure_code`.
    MeasureNotFound,
    /// The measure is disabled.
    MeasureDisabled,
    /// No `metric_platform_measure` mapping exists for the account's platform
    /// and the measure.
    PlatformMeasureNotFound,
    /// The mapping is disabled.
    PlatformMeasureDisabled,
    /// The mapping's `direct_collection` is false, so the DRIVER route may not
    /// ingest it.
    NotDirectCollection,

    // ---------------------------------------------------------------------
    // Grain, dimensions, identifiers and values. Per-observation `REJECTED`.
    // ---------------------------------------------------------------------
    /// The reporting grain is not in the mapping's `supported_grains`.
    UnsupportedReportingGrain,
    /// The half-open period does not satisfy the grain's period rule.
    InvalidReportingGrainPeriod,
    /// An ISBN or publication type was supplied for a mapping without
    /// `supports_publication`.
    UnsupportedPublicationDimension,
    /// A country was supplied for a mapping without `supports_country`.
    UnsupportedCountryDimension,
    /// A ROR was supplied for a mapping without `supports_institution`.
    UnsupportedInstitutionDimension,
    /// `work_doi` is not a syntactically valid DOI.
    InvalidDoi,
    /// No work carries the DOI.
    UnknownDoi,
    /// More than one work carries the DOI. Unreachable under the current
    /// `lower(work.doi)` uniqueness; retained as a defensive classification.
    AmbiguousDoi,
    /// `publication_isbn` is not a syntactically valid ISBN-13.
    InvalidIsbn,
    /// No publication of the resolved work matches the ISBN or type.
    UnknownPublication,
    /// More than one publication of the resolved work matches.
    AmbiguousPublication,
    /// `institution_ror` is not a syntactically valid ROR identifier.
    InvalidRor,
    /// No institution carries the ROR.
    UnknownRor,
    /// More than one institution carries the ROR.
    AmbiguousRor,
    /// `country_code` is not an exactly uppercase, officially assigned
    /// ISO 3166-1 alpha-2 code.
    InvalidCountry,
    /// A negative value for a measure without `allow_negative`, or a managed
    /// revision whose `new - old` delta cannot be represented as `i64`.
    InvalidValue,
    /// `methodology_version` was blank.
    MethodologyRequired,
    /// The measure fixes a `methodology_version` and the observation's differs.
    MethodologyMismatch,

    // ---------------------------------------------------------------------
    // Publisher scope. Request-level for the import/account scope; per
    // observation when the resolved work belongs to another publisher.
    // ---------------------------------------------------------------------
    /// The account has no expected publisher, the import has no publisher, the
    /// two differ, or (per observation) the resolved work's current publisher
    /// is not the expected publisher.
    PublisherScopeMismatch,
    /// The expected publisher's current package lacks `METRICS_COLLECT`.
    MetricsCollectNotEntitled,

    // ---------------------------------------------------------------------
    // Canonical outcomes.
    // ---------------------------------------------------------------------
    /// Per-observation `REJECTED`: a distinct accepted record in the same
    /// dimensional cell already covers an overlapping half-open period.
    OverlappingPeriod,
    /// The `CONFLICT` reason: same identity, different content, from a DRIVER
    /// account that is not the canonical winner.
    SourceConflict,
    /// Reserved for the deferred publisher-final path. Not emitted by the
    /// MOM-1 DRIVER slice.
    ConflictingFinalRecord,

    // ---------------------------------------------------------------------
    // Coverage. Always request-level.
    // ---------------------------------------------------------------------
    /// A coverage assertion's `period_end` is not after its `period_start`.
    InvalidCoveragePeriod,
    /// A coverage assertion claims a dimension the mapping does not support.
    InvalidCoverageDimension,
}

impl MetricIngestionErrorCode {
    /// The static bounded `metric_import_error.field_name` for a
    /// per-observation rejection, where one applies.
    ///
    /// `publication_supplied_isbn` selects between the two publication input
    /// fields for the codes that can arise from either.
    pub(crate) fn field_name(self, publication_supplied_isbn: bool) -> Option<&'static str> {
        use MetricIngestionErrorCode::*;
        let publication_field = if publication_supplied_isbn {
            "publication_isbn"
        } else {
            "publication_type"
        };
        match self {
            SourceAccountMismatch => Some("source_account_code"),
            PlatformMismatch | PlatformDisabled => Some("platform_code"),
            MeasureNotFound
            | MeasureDisabled
            | PlatformMeasureNotFound
            | PlatformMeasureDisabled
            | NotDirectCollection => Some("measure_code"),
            UnsupportedReportingGrain => Some("reporting_grain"),
            InvalidReportingGrainPeriod | OverlappingPeriod => Some("period_start"),
            UnsupportedPublicationDimension | UnknownPublication | AmbiguousPublication => {
                Some(publication_field)
            }
            InvalidIsbn => Some("publication_isbn"),
            UnsupportedCountryDimension | InvalidCountry => Some("country_code"),
            UnsupportedInstitutionDimension | InvalidRor | UnknownRor | AmbiguousRor => {
                Some("institution_ror")
            }
            InvalidDoi | UnknownDoi | AmbiguousDoi | PublisherScopeMismatch => Some("work_doi"),
            InvalidValue => Some("value"),
            MethodologyRequired | MethodologyMismatch => Some("methodology_version"),
            _ => None,
        }
    }

    /// The static sanitized `metric_import_error.message` for a
    /// per-observation rejection. Never derived from caller input.
    pub(crate) fn message(self) -> &'static str {
        use MetricIngestionErrorCode::*;
        match self {
            SourceAccountMismatch => "The source account code is not the import's source account",
            PlatformMismatch => "The platform code is not the source account's platform",
            PlatformDisabled => "The platform is disabled",
            MeasureNotFound => "The measure code is not registered",
            MeasureDisabled => "The measure is disabled",
            PlatformMeasureNotFound => "The platform does not report this measure",
            PlatformMeasureDisabled => "The platform/measure mapping is disabled",
            NotDirectCollection => "The platform/measure mapping is not directly collected",
            UnsupportedReportingGrain => "The reporting grain is not supported for this mapping",
            InvalidReportingGrainPeriod => "The period does not match the reporting grain",
            UnsupportedPublicationDimension => {
                "The mapping does not support the publication dimension"
            }
            UnsupportedCountryDimension => "The mapping does not support the country dimension",
            UnsupportedInstitutionDimension => {
                "The mapping does not support the institution dimension"
            }
            InvalidDoi => "The work DOI is not a valid DOI",
            UnknownDoi => "The work DOI does not resolve to a work",
            AmbiguousDoi => "The work DOI resolves to more than one work",
            InvalidIsbn => "The publication ISBN is not a valid ISBN-13",
            UnknownPublication => "The publication does not resolve within the work",
            AmbiguousPublication => "The publication resolves to more than one publication",
            InvalidRor => "The institution ROR is not a valid ROR identifier",
            UnknownRor => "The institution ROR does not resolve to an institution",
            AmbiguousRor => "The institution ROR resolves to more than one institution",
            InvalidCountry => "The country code is not an assigned ISO 3166-1 alpha-2 code",
            InvalidValue => "The value is not valid for this measure",
            MethodologyRequired => "A methodology version is required",
            MethodologyMismatch => "The methodology version does not match the measure",
            PublisherScopeMismatch => "The work does not belong to the expected publisher",
            OverlappingPeriod => "The period overlaps an existing canonical record",
            _ => "The observation was rejected",
        }
    }
}

/// A request-level coordinator failure.
///
/// A request-level failure aborts the batch: no `metric_import_batch`,
/// provenance, import-error, coverage, counter, canonical record/revision or
/// rollup-delta state is committed. The error carries only its stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricIngestionError {
    pub code: MetricIngestionErrorCode,
}

impl MetricIngestionError {
    pub(crate) fn new(code: MetricIngestionErrorCode) -> Self {
        MetricIngestionError { code }
    }
}

impl fmt::Display for MetricIngestionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "metric ingestion request failed: {}", self.code)
    }
}

impl std::error::Error for MetricIngestionError {}

impl From<MetricIngestionErrorCode> for MetricIngestionError {
    fn from(code: MetricIngestionErrorCode) -> Self {
        MetricIngestionError::new(code)
    }
}
