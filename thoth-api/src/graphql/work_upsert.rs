//! The GraphQL surface of `BE-06` (#848): the work-level worker API, the
//! Crossref permit operations, the control and operator mutations, and the
//! work-level and Crossref reports.
//!
//! The contract is Amendment 3 section 4 (the complete freeze) with the
//! authorization order of section 10.1: the required role, then for the three
//! route-derived permit operations the permit's route and its role, then the
//! operation's own input validation, then protected state. Request
//! authorization is evaluated on every invocation and is never carried by a
//! permit, a token or an earlier call.

use juniper::{FieldResult, IntoFieldError};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::graphql::Context;
use crate::model::crossref_write_permit::crud::{
    self as permit_crud, AdvanceCrossrefVersionFloor, CrossrefWritePermitFilter,
    FinaliseCrossrefWrite,
};
use crate::model::crossref_write_permit::{
    CrossrefFinalisationOutcome, CrossrefFinalisationResult, CrossrefReconciliationState,
    CrossrefVersionFloorAdvance, CrossrefVersionFloorReport, CrossrefVoidReason,
    CrossrefWriteOutcome, CrossrefWritePermitState, CrossrefWritePermitWithDois,
    CrossrefWriteReservation, CrossrefWriteRoute, CrossrefWriteScope,
};
use crate::model::distribution_job::crud::claim_work_upsert_jobs;
use crate::model::distribution_job::{
    ClaimedDistributionJob, DistributionJobCancellationReason, DistributionJobPayload,
};
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::work_upsert::crud as work_upsert_crud;
use crate::model::work_upsert::{
    MaterializeWorkUpsertJobsResult, SeedCrossrefWorkUpsertResult, WorkUpsertAdmission,
    WorkUpsertBlockedByRecoveryRow, WorkUpsertControl, WorkUpsertEligibilityClause,
    WorkUpsertMaterializationOutcome, WorkUpsertResidueClass, WorkUpsertResidueRow,
    WorkUpsertResolutionRow, WorkUpsertResolutionState, WorkUpsertStaleBindingRow,
};
use crate::model::{Generation, Timestamp};
use crate::policy::PolicyContext;

fn generation(value: i64) -> Generation {
    // Every value here comes from a non-negative database column.
    Generation::from_i64(value).unwrap_or_else(|| Generation::from_i64(0).expect("0"))
}

// ---------------------------------------------------------------------------
// Inputs (Amendment 3 section 4.3)
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "Which work-level execution profiles to drain, and how many units to run")]
pub struct MaterializeWorkUpsertJobsInput {
    #[graphql(description = "The work-level execution profiles to drain. Required and non-empty")]
    pub execution_profiles: Vec<DistributionPlatform>,
    #[graphql(
        default = 100,
        description = "Maximum units to run. Values above 500 are clamped to 500; values at or below 0 run none"
    )]
    pub limit: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(
    description = "Which work-level execution profiles to claim jobs for, how many and for how long"
)]
pub struct ClaimWorkUpsertJobsInput {
    #[graphql(description = "The work-level execution profiles to claim. Required and non-empty")]
    pub execution_profiles: Vec<DistributionPlatform>,
    #[graphql(
        default = 10,
        description = "Maximum jobs to claim. Values above 50 are clamped to 50; values at or below 0 claim nothing"
    )]
    pub limit: Option<i32>,
    #[graphql(
        default = 900,
        description = "Requested lease duration in seconds, clamped to the range 60 to 3600"
    )]
    pub lease_seconds: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "The claimed WORK_UPSERT job to reserve a Crossref write for")]
pub struct ReserveWorkUpsertCrossrefWriteInput {
    pub distribution_job_id: Uuid,
    pub claim_token: Uuid,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(
    description = "The claimed back-catalogue job and the unit Work to reserve a Crossref write for"
)]
pub struct ReserveBackCatalogueCrossrefWriteInput {
    pub distribution_job_id: Uuid,
    pub claim_token: Uuid,
    pub root_work_id: Uuid,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "The root Work to reserve a legacy scheduled Crossref write for")]
pub struct ReserveLegacyScheduledCrossrefWriteInput {
    pub root_work_id: Uuid,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "The root Work to reserve a manual recovery Crossref write for")]
pub struct ReserveManualRecoveryCrossrefWriteInput {
    pub root_work_id: Uuid,
    #[graphql(description = "Non-blank operator authorization reference, recorded on the permit")]
    pub operator_authorization_reference: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "A finalisation presentation of a reserved Crossref write")]
pub struct FinaliseCrossrefWriteInput {
    pub permit_id: Uuid,
    pub reservation_token: Uuid,
    #[graphql(description = "The job claim token; required exactly when the permit has a job")]
    pub claim_token: Option<Uuid>,
    #[graphql(description = "The registration DOIs extracted from the exact artifact bytes")]
    pub observed_dois: Vec<String>,
    pub observed_doi_batch_id: String,
    pub observed_crossref_timestamp: Generation,
    #[graphql(description = "Lower-case hexadecimal SHA-256 of the exact artifact bytes")]
    pub payload_digest: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "The provider outcome of an authorised Crossref write")]
pub struct ReportCrossrefWriteInput {
    pub permit_id: Uuid,
    pub reservation_token: Uuid,
    pub outcome: CrossrefWriteOutcome,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "The route owner's release of a reserved Crossref write")]
pub struct VoidCrossrefWriteReservationInput {
    pub permit_id: Uuid,
    pub reservation_token: Uuid,
    pub detail: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "A superuser's release of a reserved Crossref write")]
pub struct VoidCrossrefWriteReservationAsSuperuserInput {
    pub permit_id: Uuid,
    pub detail: String,
    pub authorization_reference: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "A superuser's reconciliation of a Crossref write permit")]
pub struct ReconcileCrossrefWritePermitInput {
    pub permit_id: Uuid,
    pub outcome: CrossrefWriteOutcome,
    pub reconciliation_state: CrossrefReconciliationState,
    pub authorization_reference: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "One bounded batch of the Crossref bootstrap seed for a publisher")]
pub struct SeedCrossrefWorkUpsertInput {
    pub publisher_id: Uuid,
    #[graphql(
        default = 100,
        description = "Maximum units to run. Values above 500 are clamped to 500; values at or below 0 run none"
    )]
    pub limit: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "Admission of a publisher's current Crossref binding")]
pub struct AdmitCrossrefWorkUpsertInput {
    pub publisher_id: Uuid,
    #[graphql(description = "Non-blank pointer to the durable bootstrap and census proof")]
    pub evidence_reference: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "A version floor advance bound to a separately validated authorization")]
pub struct AdvanceCrossrefVersionFloorInput {
    pub target_value: Generation,
    pub g6_attempt_id: Uuid,
    pub observation_id: Uuid,
    pub authorization_reference: String,
    pub authorization_register_digest: String,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
#[graphql(description = "One Work to materialize under one work-level execution profile")]
pub struct MaterializeWorkUpsertJobInput {
    pub work_id: Uuid,
    pub execution_profile: DistributionPlatform,
    #[graphql(default = false, description = "Skip the resolution test only")]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// Objects (Amendment 3 section 4.3)
// ---------------------------------------------------------------------------

#[juniper::graphql_object(Context = Context, description = "The counts of one work-level drain call")]
impl MaterializeWorkUpsertJobsResult {
    #[graphql(description = "Units run")]
    fn examined(&self) -> i32 {
        self.examined
    }
    #[graphql(description = "Units that created a job")]
    fn created(&self) -> i32 {
        self.created
    }
    #[graphql(description = "Units that retired a stale PENDING job, whatever their end")]
    fn rebound(&self) -> i32 {
        self.rebound
    }
    #[graphql(description = "Units that found the generation resolved")]
    fn skipped_resolved(&self) -> i32 {
        self.skipped_resolved
    }
    #[graphql(description = "Units that found the Work ineligible")]
    fn skipped_ineligible(&self) -> i32 {
        self.skipped_ineligible
    }
    #[graphql(description = "Units that found the binding not admitted")]
    fn skipped_not_admitted(&self) -> i32 {
        self.skipped_not_admitted
    }
    #[graphql(description = "Drainable candidates remaining after the call's last unit")]
    fn remaining_candidates(&self) -> i32 {
        self.remaining_candidates
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "A reserved Crossref write: the permit handle and its token, the deposit identity and membership, and the binding the reservation checked"
)]
impl CrossrefWriteReservation {
    fn permit_id(&self) -> Uuid {
        self.permit_id
    }
    #[graphql(description = "Per-permit bearer token; returned only here")]
    fn reservation_token(&self) -> Uuid {
        self.reservation_token
    }
    #[graphql(description = "The 17-digit deposit timestamp the artifact must carry")]
    fn crossref_timestamp(&self) -> Generation {
        generation(self.crossref_timestamp)
    }
    fn doi_batch_id(&self) -> &String {
        &self.doi_batch_id
    }
    #[graphql(description = "Canonical registration DOIs, ascending by code point")]
    fn dois(&self) -> &Vec<String> {
        &self.dois
    }
    fn publisher_identity(&self) -> Uuid {
        self.publisher_identity
    }
    fn root_work_identity(&self) -> Uuid {
        self.root_work_identity
    }
}

#[juniper::graphql_object(Context = Context, description = "The committed decision of a finalisation")]
impl CrossrefFinalisationResult {
    fn outcome(&self) -> CrossrefFinalisationOutcome {
        self.outcome
    }
    #[graphql(description = "Null exactly when the outcome is AUTHORIZED")]
    fn void_reason(&self) -> Option<CrossrefVoidReason> {
        self.void_reason
    }
    fn permit(&self) -> &CrossrefWritePermitWithDois {
        &self.permit
    }
}

#[juniper::graphql_object(
    Context = Context,
    name = "CrossrefWritePermit",
    description = "A Crossref write permit: the durable record of what may have reached the provider"
)]
impl CrossrefWritePermitWithDois {
    fn permit_id(&self) -> Uuid {
        self.permit.permit_id
    }
    fn route(&self) -> CrossrefWriteRoute {
        self.permit.route
    }
    fn scope(&self) -> CrossrefWriteScope {
        self.permit.scope
    }
    fn state(&self) -> CrossrefWritePermitState {
        self.permit.state
    }
    fn reconciliation_state(&self) -> Option<CrossrefReconciliationState> {
        self.permit.reconciliation_state
    }
    #[graphql(description = "Operational link to the publisher; null after its deletion")]
    fn publisher_id(&self) -> Option<Uuid> {
        self.permit.publisher_id
    }
    fn publisher_identity(&self) -> Uuid {
        self.permit.publisher_identity
    }
    fn root_work_identity(&self) -> Uuid {
        self.permit.root_work_identity
    }
    fn distribution_job_id(&self) -> Option<Uuid> {
        self.permit.distribution_job_id
    }
    fn distribution_job_attempt_id(&self) -> Option<Uuid> {
        self.permit.distribution_job_attempt_id
    }
    fn job_identity(&self) -> Option<Uuid> {
        self.permit.job_identity
    }
    fn attempt_identity(&self) -> Option<Uuid> {
        self.permit.attempt_identity
    }
    fn permit_generation(&self) -> Option<Generation> {
        self.permit.permit_generation.map(generation)
    }
    fn source_generation_witness(&self) -> Generation {
        generation(self.permit.source_generation_witness)
    }
    #[graphql(description = "Canonical membership, ascending by code point")]
    fn dois(&self) -> &Vec<String> {
        &self.dois
    }
    fn doi_set_digest(&self) -> &String {
        &self.permit.doi_set_digest
    }
    fn doi_set_cardinality(&self) -> i32 {
        self.permit.doi_set_cardinality
    }
    fn crossref_timestamp(&self) -> Generation {
        generation(self.permit.crossref_timestamp)
    }
    fn doi_batch_id(&self) -> &String {
        &self.permit.doi_batch_id
    }
    fn payload_digest(&self) -> Option<&String> {
        self.permit.payload_digest.as_ref()
    }
    fn operator_authorization_reference(&self) -> Option<&String> {
        self.permit.operator_authorization_reference.as_ref()
    }
    fn void_reason(&self) -> Option<CrossrefVoidReason> {
        self.permit.void_reason
    }
    fn void_detail(&self) -> Option<&String> {
        self.permit.void_detail.as_ref()
    }
    fn void_authorization_reference(&self) -> Option<&String> {
        self.permit.void_authorization_reference.as_ref()
    }
    fn reconciliation_annotation_reference(&self) -> Option<&String> {
        self.permit.reconciliation_annotation_reference.as_ref()
    }
    fn reconciliation_annotated_at(&self) -> Option<Timestamp> {
        self.permit.reconciliation_annotated_at
    }
    fn reconciliation_authorization_reference(&self) -> Option<&String> {
        self.permit.reconciliation_authorization_reference.as_ref()
    }
    fn reconciled_at(&self) -> Option<Timestamp> {
        self.permit.reconciled_at
    }
    fn issued_at(&self) -> Timestamp {
        self.permit.issued_at
    }
    fn authorized_at(&self) -> Option<Timestamp> {
        self.permit.authorized_at
    }
    fn provider_reported_at(&self) -> Option<Timestamp> {
        self.permit.provider_reported_at
    }
    fn closed_at(&self) -> Option<Timestamp> {
        self.permit.closed_at
    }
}

#[juniper::graphql_object(Context = Context, description = "One work-level execution key's control flags")]
impl WorkUpsertControl {
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    fn capture_enabled(&self) -> bool {
        self.capture_enabled
    }
    fn execution_enabled(&self) -> bool {
        self.execution_enabled
    }
}

#[juniper::graphql_object(Context = Context, description = "The counts of one bootstrap seed batch")]
impl SeedCrossrefWorkUpsertResult {
    fn examined(&self) -> i32 {
        self.examined
    }
    fn seeded(&self) -> i32 {
        self.seeded
    }
    fn observed(&self) -> i32 {
        self.observed
    }
    fn binding_moved_retry_later(&self) -> i32 {
        self.binding_moved_retry_later
    }
    fn no_work(&self) -> i32 {
        self.no_work
    }
    #[graphql(
        description = "Advisory: eligible Works of the publisher still uncovered after the batch"
    )]
    fn remaining_uncovered(&self) -> i32 {
        self.remaining_uncovered
    }
}

#[juniper::graphql_object(Context = Context, description = "A publisher's admission for work-level execution")]
impl WorkUpsertAdmission {
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }
    fn evidence_reference(&self) -> &String {
        &self.evidence_reference
    }
    fn actor(&self) -> &String {
        &self.actor
    }
    fn admitted_at(&self) -> Timestamp {
        self.admitted_at
    }
}

#[juniper::graphql_object(Context = Context, description = "One version floor advance, as audited")]
impl CrossrefVersionFloorAdvance {
    fn audit_id(&self) -> Uuid {
        self.audit_id
    }
    fn before_value(&self) -> Generation {
        generation(self.before_value)
    }
    fn after_value(&self) -> Generation {
        generation(self.after_value)
    }
    fn g6_attempt_id(&self) -> Uuid {
        self.g6_attempt_id
    }
    fn observation_id(&self) -> Uuid {
        self.observation_id
    }
    fn authorization_reference(&self) -> &String {
        &self.authorization_reference
    }
    fn authorization_register_digest(&self) -> &String {
        &self.authorization_register_digest
    }
    fn actor(&self) -> &String {
        &self.actor
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// The result of `materializeWorkUpsertJob` (GraphQL `WorkUpsertMaterialization`):
/// the unit's end, and the job it created or found with that job's targets and
/// attempts already read inside the unit's transaction, so that no child of the
/// job resolves through the released request loaders (Amendment 3 section 10.3).
pub struct MaterializedWorkUpsertJob {
    outcome: WorkUpsertMaterializationOutcome,
    rebound: bool,
    job: Option<DistributionJobPayload>,
}

#[juniper::graphql_object(
    Context = Context,
    name = "WorkUpsertMaterialization",
    description = "How one work-level materialization unit ended"
)]
impl MaterializedWorkUpsertJob {
    fn outcome(&self) -> WorkUpsertMaterializationOutcome {
        self.outcome
    }
    #[graphql(description = "Whether the unit retired a stale PENDING job")]
    fn rebound(&self) -> bool {
        self.rebound
    }
    #[graphql(description = "The created job, or the actionable job the unit found")]
    fn job(&self) -> Option<DistributionJobPayload> {
        self.job.clone()
    }
}

// ---------------------------------------------------------------------------
// Report objects (Amendment 3 section 4.4)
// ---------------------------------------------------------------------------

#[juniper::graphql_object(Context = Context, description = "The resolution state of one Work under one execution key")]
impl WorkUpsertResolutionRow {
    fn work_id(&self) -> Uuid {
        self.work_id
    }
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    fn current_source_generation(&self) -> Generation {
        generation(self.current_source_generation)
    }
    #[graphql(description = "D: the highest generation a SUCCEEDED job resolved")]
    fn success_resolution_generation(&self) -> Generation {
        generation(self.success_resolution_generation)
    }
    #[graphql(
        description = "H: the highest generation a FAILED or administratively cancelled job resolved"
    )]
    fn terminal_job_resolution_generation(&self) -> Generation {
        generation(self.terminal_job_resolution_generation)
    }
    fn resolution_generation(&self) -> Generation {
        generation(self.resolution_generation)
    }
    #[graphql(description = "Exclusive lower bound of the outstanding residue interval, if any")]
    fn outstanding_residue_from(&self) -> Option<Generation> {
        self.outstanding_residue.map(|(from, _)| generation(from))
    }
    #[graphql(description = "Inclusive upper bound of the outstanding residue interval, if any")]
    fn outstanding_residue_to(&self) -> Option<Generation> {
        self.outstanding_residue.map(|(_, to)| generation(to))
    }
    fn state(&self) -> WorkUpsertResolutionState {
        self.state
    }
}

#[juniper::graphql_object(Context = Context, description = "One work-level residue candidate")]
impl WorkUpsertResidueRow {
    fn work_id(&self) -> Uuid {
        self.work_id
    }
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    fn class(&self) -> WorkUpsertResidueClass {
        self.class
    }
    #[graphql(description = "The first eligibility clause an ineligible candidate fails")]
    fn failing_clause(&self) -> Option<WorkUpsertEligibilityClause> {
        self.failing_clause
    }
}

#[juniper::graphql_object(Context = Context, description = "A PENDING work-level job whose binding is obsolete")]
impl WorkUpsertStaleBindingRow {
    fn distribution_job_id(&self) -> Uuid {
        self.distribution_job_id
    }
    fn work_id(&self) -> Uuid {
        self.work_id
    }
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    #[graphql(description = "The cancellation reason the drain will use")]
    fn reason(&self) -> DistributionJobCancellationReason {
        self.reason
    }
}

#[juniper::graphql_object(Context = Context, description = "An uncleared fenced abandonment")]
impl WorkUpsertBlockedByRecoveryRow {
    fn work_identity(&self) -> Uuid {
        self.work_identity
    }
    fn execution_profile(&self) -> DistributionPlatform {
        self.execution_profile
    }
    fn distribution_job_id(&self) -> Uuid {
        self.distribution_job_id
    }
    fn distribution_job_attempt_id(&self) -> Uuid {
        self.distribution_job_attempt_id
    }
}

#[juniper::graphql_object(Context = Context, name = "CrossrefVersionFloor", description = "The Crossref version floor and its audit history")]
impl CrossrefVersionFloorReport {
    fn floor_value(&self) -> Generation {
        generation(self.floor_value)
    }
    fn advances(&self) -> &Vec<CrossrefVersionFloorAdvance> {
        &self.advances
    }
}

// ---------------------------------------------------------------------------
// Resolver bodies (Amendment 3 sections 9 and 10.1)
// ---------------------------------------------------------------------------

/// Map a `ThothResult` to the field result every BE-06 field returns.
pub(crate) fn field<T>(result: ThothResult<T>) -> FieldResult<T> {
    result.map_err(IntoFieldError::into_field_error)
}

/// Tier 1 of the three route-derived permit operations: a principal holding
/// neither `DISSEMINATION_WORKER` nor `SUPERUSER` is refused before any read.
fn require_permit_operator(context: &Context) -> ThothResult<()> {
    if context.require_dissemination_worker().is_ok() || context.require_superuser().is_ok() {
        Ok(())
    } else {
        Err(ThothError::Unauthorised)
    }
}

/// C0: the role derived from a permit's persisted route (R52B section 19.4).
fn require_route_role(context: &Context, route: CrossrefWriteRoute) -> ThothResult<()> {
    match route {
        CrossrefWriteRoute::WorkUpsert
        | CrossrefWriteRoute::PublisherBackCatalogue
        | CrossrefWriteRoute::LegacyScheduled => context.require_dissemination_worker().map(|_| ()),
        CrossrefWriteRoute::ManualRecovery => context.require_superuser().map(|_| ()),
    }
}

pub(crate) fn materialize_work_upsert_jobs(
    context: &Context,
    data: &MaterializeWorkUpsertJobsInput,
) -> ThothResult<MaterializeWorkUpsertJobsResult> {
    context.require_dissemination_worker()?;
    work_upsert_crud::materialize_work_upsert_jobs(
        &context.db,
        &data.execution_profiles,
        data.limit,
    )
}

pub(crate) fn claim_work_upsert(
    context: &Context,
    data: &ClaimWorkUpsertJobsInput,
) -> ThothResult<Vec<ClaimedDistributionJob>> {
    context.require_dissemination_worker()?;
    let worker = context.user_id()?;
    claim_work_upsert_jobs(
        &context.db,
        worker,
        &data.execution_profiles,
        data.limit.unwrap_or(10),
        data.lease_seconds.unwrap_or(900),
    )
}

pub(crate) fn reserve_work_upsert(
    context: &Context,
    data: &ReserveWorkUpsertCrossrefWriteInput,
) -> ThothResult<CrossrefWriteReservation> {
    context.require_dissemination_worker()?;
    permit_crud::reserve_work_upsert_crossref_write(
        &context.db,
        data.distribution_job_id,
        data.claim_token,
    )
}

pub(crate) fn reserve_back_catalogue(
    context: &Context,
    data: &ReserveBackCatalogueCrossrefWriteInput,
) -> ThothResult<CrossrefWriteReservation> {
    context.require_dissemination_worker()?;
    permit_crud::reserve_back_catalogue_crossref_write(
        &context.db,
        data.distribution_job_id,
        data.claim_token,
        data.root_work_id,
    )
}

pub(crate) fn reserve_legacy_scheduled(
    context: &Context,
    data: &ReserveLegacyScheduledCrossrefWriteInput,
) -> ThothResult<CrossrefWriteReservation> {
    context.require_dissemination_worker()?;
    permit_crud::reserve_legacy_scheduled_crossref_write(&context.db, data.root_work_id)
}

pub(crate) fn reserve_manual_recovery(
    context: &Context,
    data: &ReserveManualRecoveryCrossrefWriteInput,
) -> ThothResult<CrossrefWriteReservation> {
    context.require_superuser()?;
    permit_crud::reserve_manual_recovery_crossref_write(
        &context.db,
        data.root_work_id,
        &data.operator_authorization_reference,
    )
}

pub(crate) fn finalise(
    context: &Context,
    data: &FinaliseCrossrefWriteInput,
) -> ThothResult<CrossrefFinalisationResult> {
    require_permit_operator(context)?;
    let input = FinaliseCrossrefWrite {
        permit_id: data.permit_id,
        reservation_token: data.reservation_token,
        claim_token: data.claim_token,
        observed_dois: data.observed_dois.clone(),
        observed_doi_batch_id: data.observed_doi_batch_id.clone(),
        observed_crossref_timestamp: data.observed_crossref_timestamp.value(),
        payload_digest: data.payload_digest.clone(),
    };
    let authorize = |route| require_route_role(context, route);
    permit_crud::finalise_crossref_write(&context.db, &input, &authorize)
}

pub(crate) fn report(
    context: &Context,
    data: &ReportCrossrefWriteInput,
) -> ThothResult<CrossrefWritePermitWithDois> {
    require_permit_operator(context)?;
    let authorize = |route| require_route_role(context, route);
    permit_crud::report_crossref_write(
        &context.db,
        data.permit_id,
        data.reservation_token,
        data.outcome,
        &authorize,
    )
}

pub(crate) fn void_reservation(
    context: &Context,
    data: &VoidCrossrefWriteReservationInput,
) -> ThothResult<CrossrefWritePermitWithDois> {
    require_permit_operator(context)?;
    let authorize = |route| require_route_role(context, route);
    permit_crud::void_crossref_write_reservation(
        &context.db,
        data.permit_id,
        data.reservation_token,
        &data.detail,
        &authorize,
    )
}

pub(crate) fn void_reservation_as_superuser(
    context: &Context,
    data: &VoidCrossrefWriteReservationAsSuperuserInput,
) -> ThothResult<CrossrefWritePermitWithDois> {
    context.require_superuser()?;
    permit_crud::void_crossref_write_reservation_as_superuser(
        &context.db,
        data.permit_id,
        &data.detail,
        &data.authorization_reference,
    )
}

pub(crate) fn reconcile(
    context: &Context,
    data: &ReconcileCrossrefWritePermitInput,
) -> ThothResult<CrossrefWritePermitWithDois> {
    context.require_superuser()?;
    permit_crud::reconcile_crossref_write_permit(
        &context.db,
        data.permit_id,
        data.outcome,
        data.reconciliation_state,
        &data.authorization_reference,
    )
}

pub(crate) fn enable_capture(
    context: &Context,
    execution_profile: DistributionPlatform,
) -> ThothResult<WorkUpsertControl> {
    context.require_superuser()?;
    work_upsert_crud::enable_work_upsert_capture(&context.db, execution_profile)
}

pub(crate) fn set_execution(
    context: &Context,
    execution_profile: DistributionPlatform,
    enabled: bool,
) -> ThothResult<WorkUpsertControl> {
    context.require_superuser()?;
    work_upsert_crud::set_work_upsert_execution(&context.db, execution_profile, enabled)
}

pub(crate) fn seed(
    context: &Context,
    data: &SeedCrossrefWorkUpsertInput,
) -> ThothResult<SeedCrossrefWorkUpsertResult> {
    context.require_superuser()?;
    work_upsert_crud::seed_crossref_work_upsert(&context.db, data.publisher_id, data.limit)
}

pub(crate) fn admit(
    context: &Context,
    data: &AdmitCrossrefWorkUpsertInput,
) -> ThothResult<WorkUpsertAdmission> {
    context.require_superuser()?;
    let actor = context.user_id()?;
    work_upsert_crud::admit_crossref_work_upsert(
        &context.db,
        data.publisher_id,
        &data.evidence_reference,
        actor,
    )
}

pub(crate) fn advance_floor(
    context: &Context,
    data: &AdvanceCrossrefVersionFloorInput,
) -> ThothResult<CrossrefVersionFloorAdvance> {
    context.require_superuser()?;
    let actor = context.user_id()?;
    let input = AdvanceCrossrefVersionFloor {
        target_value: data.target_value.value(),
        g6_attempt_id: data.g6_attempt_id,
        observation_id: data.observation_id,
        authorization_reference: data.authorization_reference.clone(),
        authorization_register_digest: data.authorization_register_digest.clone(),
    };
    permit_crud::advance_crossref_version_floor(&context.db, &input, actor)
}

pub(crate) fn materialize_one(
    context: &Context,
    data: &MaterializeWorkUpsertJobInput,
) -> ThothResult<MaterializedWorkUpsertJob> {
    context.require_superuser()?;
    let (unit, job) = work_upsert_crud::materialize_work_upsert_job_with_payload(
        &context.db,
        data.work_id,
        data.execution_profile,
        data.force,
    )?;
    Ok(MaterializedWorkUpsertJob {
        outcome: unit.outcome,
        rebound: unit.rebound,
        job,
    })
}

pub(crate) fn permit_filter(
    job_identity: Option<Uuid>,
    attempt_identity: Option<Uuid>,
    root_work_identity: Option<Uuid>,
    publisher_identity: Option<Uuid>,
    states: Option<Vec<CrossrefWritePermitState>>,
) -> CrossrefWritePermitFilter {
    CrossrefWritePermitFilter {
        job_identity,
        attempt_identity,
        root_work_identity,
        publisher_identity,
        states: states.unwrap_or_default(),
    }
}
