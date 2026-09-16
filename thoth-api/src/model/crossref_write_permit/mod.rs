//! The Crossref execution profile of `BE-06` (#848): universal write permits,
//! canonical DOI membership, deposit timestamps and the version floor.
//!
//! The specification is `docs/publisher-services/specifications/BE-06-R52B.md`
//! sections 14-17 as amended by #848 Amendments 1-3. Nothing here contacts
//! Crossref: provider writes are performed by the separately specified
//! downstream dissemination task (`thoth-pub/thoth-dissemination#106`).

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::model::Timestamp;

macro_rules! crossref_db_enum {
    (
        $(#[$meta:meta])*
        $name:ident, $sql:literal, $description:literal,
        [$($variant:ident = $label:literal : $doc:literal),+ $(,)?]
    ) => {
        $(#[$meta])*
        #[cfg_attr(
            feature = "backend",
            derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
            graphql(description = $description),
            ExistingTypePath = $sql
        )]
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
        pub enum $name {
            $(
                #[cfg_attr(feature = "backend", db_rename = $label, graphql(description = $doc))]
                $variant,
            )+
        }
    };
}

crossref_db_enum!(
    /// The route that issued a Crossref write permit (R52B section 16.1).
    CrossrefWriteRoute,
    "crate::schema::sql_types::CrossrefWriteRoute",
    "The route that issued a Crossref write permit",
    [
        WorkUpsert = "WORK_UPSERT": "A work-level WORK_UPSERT job attempt",
        PublisherBackCatalogue = "PUBLISHER_BACK_CATALOGUE": "One unit of a publisher back-catalogue job attempt",
        LegacyScheduled = "LEGACY_SCHEDULED": "The legacy scheduled selector",
        ManualRecovery = "MANUAL_RECOVERY": "A superuser's manual recovery",
    ]
);

crossref_db_enum!(
    /// The state of a Crossref write permit (R52B section 16.2).
    CrossrefWritePermitState,
    "crate::schema::sql_types::CrossrefWritePermitState",
    "The state of a Crossref write permit",
    [
        Reserved = "RESERVED": "Membership, timestamp and batch id are fixed; nothing is authorised",
        Authorized = "AUTHORIZED": "Exactly the bytes named by the payload digest may be submitted",
        Indeterminate = "INDETERMINATE": "A submission was attempted and its outcome is unknown",
        Accepted = "ACCEPTED": "The provider acknowledged receipt of the submission",
        NoneAttempted = "NONE_ATTEMPTED": "Proven that no submission reached the provider",
        Voided = "VOIDED": "The reservation was released before it authorised anything",
    ]
);

crossref_db_enum!(
    /// A superuser's reconciliation state of a permit (R52B section 16.8).
    CrossrefReconciliationState,
    "crate::schema::sql_types::CrossrefReconciliationState",
    "A superuser's reconciliation state of a Crossref write permit",
    [
        Reconciled = "RECONCILED": "Reconciliation truth is recorded",
        ReconciliationRequired = "RECONCILIATION_REQUIRED": "The outcome is unknown and must be established",
        ReconciliationImpossible = "RECONCILIATION_IMPOSSIBLE": "The evidence available when recorded cannot establish the outcome",
    ]
);

crossref_db_enum!(
    /// The scope of a Crossref write permit.
    CrossrefWriteScope,
    "crate::schema::sql_types::CrossrefWriteScope",
    "The scope of a Crossref write permit",
    [SingleRootWork = "SINGLE_ROOT_WORK": "One root Work and its registered DOIs"]
);

crossref_db_enum!(
    /// Why a reservation was voided (R52B sections 16.6 and 16.7).
    CrossrefVoidReason,
    "crate::schema::sql_types::CrossrefVoidReason",
    "Why a Crossref write reservation was voided",
    [
        SourceChangedDuringPreparation = "SOURCE_CHANGED_DURING_PREPARATION": "The root's source generation changed after the reservation",
        DoiMembershipChanged = "DOI_MEMBERSHIP_CHANGED": "The re-derived DOI membership differs from the reservation",
        ArtifactDoiSetMismatch = "ARTIFACT_DOI_SET_MISMATCH": "The artifact's registration DOIs differ from the reservation",
        ArtifactBatchIdMismatch = "ARTIFACT_BATCH_ID_MISMATCH": "The artifact's batch id differs from the reservation",
        ArtifactTimestampMismatch = "ARTIFACT_TIMESTAMP_MISMATCH": "The artifact's timestamp differs from the reservation",
        ExecutionNotPermitted = "EXECUTION_NOT_PERMITTED": "Work-level execution was not permitted at finalisation",
        ProfileNotAdmitted = "PROFILE_NOT_ADMITTED": "The binding was not admitted at finalisation",
        Ineligible = "INELIGIBLE": "The Work was not eligible at finalisation",
        BindingSuperseded = "BINDING_SUPERSEDED": "The Work's binding changed before finalisation",
        AssignmentDisabled = "ASSIGNMENT_DISABLED": "The Crossref assignment was not enabled at finalisation",
        NoWork = "NO_WORK": "The root Work no longer exists",
        OwnerAbandoned = "OWNER_ABANDONED": "The route owner released the reservation",
        OperatorCleanup = "OPERATOR_CLEANUP": "A superuser released the reservation",
    ]
);

/// One Crossref write permit row (R52B sections 16 and 18.6).
///
/// `reservation_token` is a per-permit bearer capability. It is returned only
/// by a reservation, as [`CrossrefWriteReservation::reservation_token`], and no
/// GraphQL type exposes this field.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Queryable, diesel::QueryableByName),
    diesel(table_name = crate::schema::crossref_write_permit)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossrefWritePermit {
    pub permit_id: Uuid,
    pub route: CrossrefWriteRoute,
    pub scope: CrossrefWriteScope,
    pub state: CrossrefWritePermitState,
    pub reconciliation_state: Option<CrossrefReconciliationState>,
    pub reservation_token: Uuid,
    pub publisher_id: Option<Uuid>,
    pub publisher_identity: Uuid,
    pub root_work_identity: Uuid,
    pub distribution_job_id: Option<Uuid>,
    pub distribution_job_attempt_id: Option<Uuid>,
    pub job_identity: Option<Uuid>,
    pub attempt_identity: Option<Uuid>,
    pub permit_generation: Option<i64>,
    pub source_generation_witness: i64,
    pub doi_set_digest: String,
    pub doi_set_cardinality: i32,
    pub crossref_timestamp: i64,
    pub doi_batch_id: String,
    pub payload_digest: Option<String>,
    pub operator_authorization_reference: Option<String>,
    pub reconciliation_annotation_reference: Option<String>,
    pub reconciliation_authorization_reference: Option<String>,
    pub void_reason: Option<CrossrefVoidReason>,
    pub void_detail: Option<String>,
    pub void_authorization_reference: Option<String>,
    pub issued_at: Timestamp,
    pub authorized_at: Option<Timestamp>,
    pub provider_reported_at: Option<Timestamp>,
    pub reconciliation_annotated_at: Option<Timestamp>,
    pub reconciled_at: Option<Timestamp>,
    pub closed_at: Option<Timestamp>,
}

/// A permit together with its canonical membership, ascending by code point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossrefWritePermitWithDois {
    pub permit: CrossrefWritePermit,
    pub dois: Vec<String>,
}

/// The result of any of the four reservations (Amendment 3 sections 4.3 and 6):
/// the permit handle and token, the allocated deposit identity, the derived
/// membership, and the public projection of the binding the reservation checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossrefWriteReservation {
    pub permit_id: Uuid,
    pub reservation_token: Uuid,
    pub crossref_timestamp: i64,
    pub doi_batch_id: String,
    pub dois: Vec<String>,
    pub publisher_identity: Uuid,
    pub root_work_identity: Uuid,
}

/// A reported or reconciled provider outcome (R52B section 16.7).
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "A Crossref provider outcome")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossrefWriteOutcome {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The provider acknowledged receipt of the submission")
    )]
    Accepted,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "A submission was attempted and its outcome is unknown")
    )]
    Indeterminate,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Proven that no submission reached the provider")
    )]
    NoneAttempted,
}

/// The outcome of a finalisation (R52B section 16.6).
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "The outcome of a Crossref write finalisation")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossrefFinalisationOutcome {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The write is authorised for exactly the bound bytes")
    )]
    Authorized,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The reservation was voided; the job may be retried")
    )]
    VoidedRetryable,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The reservation was voided and the job was retired")
    )]
    VoidedJobRetired,
}

/// The result of a finalisation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossrefFinalisationResult {
    pub outcome: CrossrefFinalisationOutcome,
    pub void_reason: Option<CrossrefVoidReason>,
    pub permit: CrossrefWritePermitWithDois,
}

/// One version-floor advance: the audit row it inserted (Amendment 3 section
/// 9.9). `mutation_kind` is not carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossrefVersionFloorAdvance {
    pub audit_id: Uuid,
    pub before_value: i64,
    pub after_value: i64,
    pub g6_attempt_id: Uuid,
    pub observation_id: Uuid,
    pub authorization_reference: String,
    pub authorization_register_digest: String,
    pub actor: String,
    pub occurred_at: Timestamp,
}

#[cfg(feature = "backend")]
pub mod crud;

#[cfg(all(test, feature = "backend"))]
mod tests;
