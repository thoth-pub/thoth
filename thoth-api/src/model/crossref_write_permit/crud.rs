//! The Crossref profile operations of `BE-06`: reservations, finalisation,
//! outcome reports, voids, reconciliation and the version-floor advance.
//!
//! Every function here is a model function behind a new BE-06 entry point and
//! opens its transaction only through
//! [`work_upsert_transaction`](crate::model::work_upsert::work_upsert_transaction)
//! (Amendment 3 section 10.3, EB1). Request authorization, including the
//! route-derived role, is the resolver's and always precedes the call.

use diesel::prelude::*;
use diesel::sql_types::{Array, BigInt, Integer, Nullable, Text, Uuid as SqlUuid};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::{CrossrefWritePermitState, CrossrefWriteReservation, CrossrefWriteRoute};
use crate::db::PgPool;
use crate::model::distribution_job::{DistributionJobKind, DistributionJobStatus};
use crate::model::work_upsert::crud as substrate;
use crate::model::work_upsert::registry::CROSSREF_PROFILE;
use crate::model::work_upsert::{
    policy, take_execution_gate, work_upsert_transaction, WorkUpsertTxError,
};

/// The advisory-lock namespace of the Crossref DOI keys `K` (R52B section 16.5).
const DOI_KEY_NAMESPACE: i32 = 1_948_572_001;

type Tx<T> = Result<T, WorkUpsertTxError>;

fn refuse<T>(error: ThothError) -> Tx<T> {
    Err(WorkUpsertTxError::Thoth(error))
}

/// The claim-relevant columns of a job, read by MVCC.
#[derive(Queryable)]
struct JobClaim {
    kind: DistributionJobKind,
    status: DistributionJobStatus,
    claim_token: Option<Uuid>,
    publisher_id: Uuid,
    work_id: Option<Uuid>,
    activation_id: Uuid,
}

fn job_claim(connection: &mut PgConnection, job_id: Uuid) -> QueryResult<Option<JobClaim>> {
    use crate::schema::distribution_job as job;
    job::table
        .filter(job::distribution_job_id.eq(job_id))
        .select((
            job::kind,
            job::status,
            job::claim_token,
            job::publisher_id,
            job::work_id,
            job::activation_id,
        ))
        .first::<JobClaim>(connection)
        .optional()
}

/// Whether a claim, read by MVCC, is current: the job exists, is `RUNNING` and
/// holds `claim_token`.
fn claim_is_current(claim: &Option<JobClaim>, claim_token: Uuid) -> bool {
    claim.as_ref().is_some_and(|claim| {
        claim.status == DistributionJobStatus::Running && claim.claim_token == Some(claim_token)
    })
}

/// `J FOR UPDATE` then `A FOR UPDATE`, re-checking the claim under the locks.
/// Returns the open attempt bound to the token.
fn lock_job_and_attempt(
    connection: &mut PgConnection,
    job_id: Uuid,
    claim_token: Uuid,
) -> Tx<Uuid> {
    use crate::schema::{distribution_job as job, distribution_job_attempt as attempt};
    let locked = job::table
        .filter(job::distribution_job_id.eq(job_id))
        .select((job::status, job::claim_token))
        .for_update()
        .first::<(DistributionJobStatus, Option<Uuid>)>(connection)
        .optional()?;
    if locked != Some((DistributionJobStatus::Running, Some(claim_token))) {
        return refuse(ThothError::CrossrefPermitClaimStale);
    }
    let open = attempt::table
        .filter(attempt::distribution_job_id.eq(job_id))
        .filter(attempt::claim_token.eq(claim_token))
        .filter(attempt::finished_at.is_null())
        .select(attempt::distribution_job_attempt_id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()?;
    match open {
        Some(attempt_id) => Ok(attempt_id),
        None => refuse(ThothError::CrossrefPermitClaimStale),
    }
}

/// R52B section 14.3's membership of a deposit, ascending by code point.
fn derived_membership(connection: &mut PgConnection, root: Uuid) -> Tx<Vec<String>> {
    #[derive(QueryableByName)]
    struct Membership {
        #[diesel(sql_type = Array<Text>)]
        dois: Vec<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        non_canonical: bool,
    }
    let row = diesel::sql_query(
        "SELECT public.crossref_deposit_membership($1) AS dois, \
                (EXISTS (SELECT 1 FROM public.work w \
                          WHERE w.work_id = $1 AND w.doi IS NOT NULL AND w.landing_page IS NOT NULL \
                            AND public.crossref_canonical_doi(w.doi) IS NULL) \
                 OR EXISTS (SELECT 1 FROM public.work_relation r \
                              JOIN public.work c ON c.work_id = r.related_work_id \
                             WHERE r.relator_work_id = $1 AND r.relation_type = 'has-child' \
                               AND c.doi IS NOT NULL AND public.crossref_canonical_doi(c.doi) IS NULL)) AS non_canonical",
    )
    .bind::<SqlUuid, _>(root)
    .get_result::<Membership>(connection)?;
    if row.dois.is_empty() {
        return refuse(ThothError::CrossrefPermitEmptyDoiSet);
    }
    if row.non_canonical {
        return refuse(ThothError::CrossrefDoiNotCanonicalisable);
    }
    Ok(row.dois)
}

/// What the common suffix of a reservation inserts.
struct Issue<'a> {
    route: CrossrefWriteRoute,
    publisher_id: Uuid,
    root: Uuid,
    witness: i64,
    job: Option<(Uuid, Uuid)>,
    permit_generation: Option<i64>,
    operator_authorization_reference: Option<&'a str>,
}

/// LO-C after the route prefix (R52B sections 16.4 and 16.5): `K` ascending,
/// `F FOR SHARE`, the blocking check, history, the one clock read and
/// allocation, then `X` and `M`.
fn issue(
    connection: &mut PgConnection,
    issue: Issue<'_>,
    membership: Vec<String>,
) -> Tx<CrossrefWriteReservation> {
    #[derive(QueryableByName)]
    struct Key {
        #[diesel(sql_type = Integer)]
        key: i32,
    }
    #[derive(QueryableByName)]
    struct Value {
        #[diesel(sql_type = Nullable<BigInt>)]
        value: Option<i64>,
    }
    #[derive(QueryableByName)]
    struct Issued {
        #[diesel(sql_type = SqlUuid)]
        permit_id: Uuid,
        #[diesel(sql_type = SqlUuid)]
        reservation_token: Uuid,
    }

    // K: DOI advisory keys, deduplicated and ascending as signed integer.
    let mut keys: Vec<i32> = diesel::sql_query(
        "SELECT DISTINCT hashtext('be06:crossref:doi:' || d) AS key FROM unnest($1::text[]) AS d",
    )
    .bind::<Array<Text>, _>(&membership)
    .load::<Key>(connection)?
    .into_iter()
    .map(|row| row.key)
    .collect();
    keys.sort_unstable();
    keys.dedup();
    for key in keys {
        diesel::sql_query("SELECT pg_advisory_xact_lock($1, $2)::text")
            .bind::<Integer, _>(DOI_KEY_NAMESPACE)
            .bind::<Integer, _>(key)
            .execute(connection)?;
    }
    // F: the floor, FOR SHARE.
    let floor = diesel::sql_query(
        "SELECT floor_value AS value FROM public.work_crossref_version_floor WHERE floor_id FOR SHARE",
    )
    .get_result::<Value>(connection)?
    .value;
    // No blocking permit overlaps the membership.
    let blocked = diesel::sql_query(
        "SELECT CASE WHEN EXISTS ( \
             SELECT 1 FROM public.crossref_write_permit_doi d \
               JOIN public.crossref_write_permit p ON p.permit_id = d.permit_id \
              WHERE d.doi = ANY($1::text[]) \
                AND public.crossref_is_blocking_write_permit(p.state, p.reconciliation_state)) \
         THEN 1::bigint END AS value",
    )
    .bind::<Array<Text>, _>(&membership)
    .get_result::<Value>(connection)?
    .value
    .is_some();
    if blocked {
        return refuse(ThothError::CrossrefPermitBlocked);
    }
    // History: every non-VOIDED overlapping permit.
    let history = diesel::sql_query(
        "SELECT max(p.crossref_timestamp) AS value FROM public.crossref_write_permit_doi d \
           JOIN public.crossref_write_permit p ON p.permit_id = d.permit_id \
          WHERE d.doi = ANY($1::text[]) AND p.state <> 'VOIDED'",
    )
    .bind::<Array<Text>, _>(&membership)
    .get_result::<Value>(connection)?
    .value;
    // The one clock read, and the pure allocation rule.
    let timestamp = diesel::sql_query(
        "SELECT public.crossref_allocate_timestamp($1, $2, public.crossref_ts_now()) AS value",
    )
    .bind::<Nullable<BigInt>, _>(history)
    .bind::<Nullable<BigInt>, _>(floor)
    .get_result::<Value>(connection)?
    .value
    .ok_or(WorkUpsertTxError::Thoth(
        ThothError::WorkUpsertDatabaseFailure,
    ))?;
    let doi_batch_id = format!("{}_{timestamp}", issue.root);

    // X, then M.
    let (job_id, attempt_id) = issue
        .job
        .map(|(job, attempt)| (Some(job), Some(attempt)))
        .unwrap_or_default();
    let issued = diesel::sql_query(
        "INSERT INTO public.crossref_write_permit \
             (route, scope, publisher_id, publisher_identity, root_work_identity, \
              distribution_job_id, distribution_job_attempt_id, job_identity, attempt_identity, \
              permit_generation, source_generation_witness, doi_set_digest, doi_set_cardinality, \
              crossref_timestamp, doi_batch_id, operator_authorization_reference) \
         VALUES ($1, 'SINGLE_ROOT_WORK', $2, $2, $3, $4, $5, $4, $5, $6, $7, \
                 public.crossref_doi_set_digest($8::text[]), cardinality($8::text[]), $9, $10, $11) \
         RETURNING permit_id, reservation_token",
    )
    .bind::<crate::schema::sql_types::CrossrefWriteRoute, _>(issue.route)
    .bind::<SqlUuid, _>(issue.publisher_id)
    .bind::<SqlUuid, _>(issue.root)
    .bind::<Nullable<SqlUuid>, _>(job_id)
    .bind::<Nullable<SqlUuid>, _>(attempt_id)
    .bind::<Nullable<BigInt>, _>(issue.permit_generation)
    .bind::<BigInt, _>(issue.witness)
    .bind::<Array<Text>, _>(&membership)
    .bind::<BigInt, _>(timestamp)
    .bind::<Text, _>(&doi_batch_id)
    .bind::<Nullable<Text>, _>(issue.operator_authorization_reference)
    .get_result::<Issued>(connection)?;
    diesel::sql_query(
        "INSERT INTO public.crossref_write_permit_doi (permit_id, doi) SELECT $1, d FROM unnest($2::text[]) AS d",
    )
    .bind::<SqlUuid, _>(issued.permit_id)
    .bind::<Array<Text>, _>(&membership)
    .execute(connection)?;

    Ok(CrossrefWriteReservation {
        permit_id: issued.permit_id,
        reservation_token: issued.reservation_token,
        crossref_timestamp: timestamp,
        doi_batch_id,
        dois: membership,
        publisher_identity: issue.publisher_id,
        root_work_identity: issue.root,
    })
}

/// `reserveWorkUpsertCrossrefWrite` (Amendment 3 section 9.6), steps 2-11.
pub fn reserve_work_upsert_crossref_write(
    db: &PgPool,
    distribution_job_id: Uuid,
    claim_token: Uuid,
) -> ThothResult<CrossrefWriteReservation> {
    work_upsert_transaction(db, |connection| {
        // 2-3: the claim and the kind, by MVCC, before any lock.
        let claim = job_claim(connection, distribution_job_id)?;
        if !claim_is_current(&claim, claim_token) {
            return refuse(ThothError::CrossrefPermitClaimStale);
        }
        let claim = claim.expect("a current claim exists");
        if claim.kind != DistributionJobKind::WorkUpsert {
            return refuse(ThothError::CrossrefReservationJobKindMismatch);
        }
        // 4: P, then W; an absent Work proves a retired claim.
        let Some(work_id) = claim.work_id else {
            return refuse(ThothError::CrossrefPermitClaimStale);
        };
        substrate::share_publisher(connection, claim.publisher_id)?;
        if !substrate::share_work(connection, work_id)? {
            return refuse(ThothError::CrossrefPermitClaimStale);
        }
        // 5: G, read unchanged as the witness.
        let witness = substrate::lock_generation(connection, work_id, CROSSREF_PROFILE.key)?;
        // 6: J then A, the claim re-checked under the locks.
        let attempt_id = lock_job_and_attempt(connection, distribution_job_id, claim_token)?;
        // 7: fence clauses 5 and 6.
        if substrate::current_publisher(connection, work_id)? != Some(claim.publisher_id)
            || !substrate::job_targets_enabled(
                connection,
                distribution_job_id,
                claim.publisher_id,
                claim.activation_id,
            )?
        {
            return refuse(ThothError::CrossrefBindingMovedRetry);
        }
        // 8: one permit per attempt, ever.
        {
            use crate::schema::crossref_write_permit as permit;
            let existing = permit::table
                .filter(permit::route.eq(CrossrefWriteRoute::WorkUpsert))
                .filter(permit::attempt_identity.eq(attempt_id))
                .select(permit::permit_id)
                .first::<Uuid>(connection)
                .optional()?;
            if existing.is_some() {
                return refuse(ThothError::CrossrefPermitAttemptAlreadyReserved);
            }
        }
        // 9: Q SHARE, then fence clauses 7, 8 and 9.
        take_execution_gate(connection, CROSSREF_PROFILE.key, false)?;
        if substrate::admission_row(connection, claim.publisher_id, claim.activation_id)?.is_none()
        {
            return refuse(ThothError::WorkUpsertProfileNotAdmitted);
        }
        let execution_enabled = {
            use crate::schema::work_upsert_control as control;
            control::table
                .filter(control::execution_profile.eq(CROSSREF_PROFILE.key))
                .select(control::execution_enabled)
                .first::<bool>(connection)
                .optional()?
                .unwrap_or(false)
        };
        if !execution_enabled
            || !substrate::profile_eligible(connection, work_id, &CROSSREF_PROFILE)?
        {
            return refuse(ThothError::WorkUpsertExecutionNotPermitted);
        }
        // The attempt's claimed generation.
        let permit_generation = {
            use crate::schema::distribution_job_attempt as attempt;
            attempt::table
                .filter(attempt::distribution_job_attempt_id.eq(attempt_id))
                .select(attempt::claimed_generation)
                .first::<Option<i64>>(connection)?
        };
        // 10-11.
        let membership = derived_membership(connection, work_id)?;
        issue(
            connection,
            Issue {
                route: CrossrefWriteRoute::WorkUpsert,
                publisher_id: claim.publisher_id,
                root: work_id,
                witness,
                job: Some((distribution_job_id, attempt_id)),
                permit_generation,
                operator_authorization_reference: None,
            },
            membership,
        )
    })
}

/// `reserveBackCatalogueCrossrefWrite` (Amendment 3 section 9.6), steps 2-11.
pub fn reserve_back_catalogue_crossref_write(
    db: &PgPool,
    distribution_job_id: Uuid,
    claim_token: Uuid,
    root_work_id: Uuid,
) -> ThothResult<CrossrefWriteReservation> {
    work_upsert_transaction(db, |connection| {
        let claim = job_claim(connection, distribution_job_id)?;
        if !claim_is_current(&claim, claim_token) {
            return refuse(ThothError::CrossrefPermitClaimStale);
        }
        let claim = claim.expect("a current claim exists");
        if claim.kind != DistributionJobKind::PublisherBackCatalogue {
            return refuse(ThothError::CrossrefReservationJobKindMismatch);
        }
        substrate::share_publisher(connection, claim.publisher_id)?;
        if !substrate::share_work(connection, root_work_id)? {
            return refuse(ThothError::CrossrefRootWorkNotFound);
        }
        if substrate::current_publisher(connection, root_work_id)? != Some(claim.publisher_id) {
            return refuse(ThothError::CrossrefUnitPublisherMismatch);
        }
        if !substrate::crossref_publisher_covered(connection, claim.publisher_id)? {
            return refuse(ThothError::CrossrefPublisherNotCovered);
        }
        let witness = substrate::lock_generation(connection, root_work_id, CROSSREF_PROFILE.key)?;
        let attempt_id = lock_job_and_attempt(connection, distribution_job_id, claim_token)?;
        {
            use crate::model::crossref_write_permit::CrossrefWritePermitState;
            use crate::schema::crossref_write_permit as permit;
            let deposited = permit::table
                .filter(permit::route.eq(CrossrefWriteRoute::PublisherBackCatalogue))
                .filter(permit::job_identity.eq(distribution_job_id))
                .filter(permit::root_work_identity.eq(root_work_id))
                .filter(permit::state.eq(CrossrefWritePermitState::Accepted))
                .select(permit::permit_id)
                .first::<Uuid>(connection)
                .optional()?;
            if deposited.is_some() {
                return refuse(ThothError::CrossrefUnitAlreadyDepositedInJob);
            }
        }
        let membership = derived_membership(connection, root_work_id)?;
        issue(
            connection,
            Issue {
                route: CrossrefWriteRoute::PublisherBackCatalogue,
                publisher_id: claim.publisher_id,
                root: root_work_id,
                witness,
                job: Some((distribution_job_id, attempt_id)),
                permit_generation: None,
                operator_authorization_reference: None,
            },
            membership,
        )
    })
}

/// The jobless reservation routes (Amendment 3 section 9.6), steps 3-9.
fn reserve_jobless(
    db: &PgPool,
    route: CrossrefWriteRoute,
    root_work_id: Uuid,
    operator_authorization_reference: Option<&str>,
) -> ThothResult<CrossrefWriteReservation> {
    work_upsert_transaction(db, |connection| {
        let Some(publisher_id) = substrate::current_publisher(connection, root_work_id)? else {
            return refuse(ThothError::CrossrefRootWorkNotFound);
        };
        substrate::share_publisher(connection, publisher_id)?;
        if !substrate::share_work(connection, root_work_id)? {
            return refuse(ThothError::CrossrefRootWorkNotFound);
        }
        if substrate::current_publisher(connection, root_work_id)? != Some(publisher_id) {
            return refuse(ThothError::CrossrefBindingMovedRetry);
        }
        if !substrate::crossref_publisher_covered(connection, publisher_id)? {
            return refuse(ThothError::CrossrefPublisherNotCovered);
        }
        let witness = substrate::lock_generation(connection, root_work_id, CROSSREF_PROFILE.key)?;
        let membership = derived_membership(connection, root_work_id)?;
        issue(
            connection,
            Issue {
                route,
                publisher_id,
                root: root_work_id,
                witness,
                job: None,
                permit_generation: None,
                operator_authorization_reference,
            },
            membership,
        )
    })
}

/// `reserveLegacyScheduledCrossrefWrite` (Amendment 3 section 9.6).
pub fn reserve_legacy_scheduled_crossref_write(
    db: &PgPool,
    root_work_id: Uuid,
) -> ThothResult<CrossrefWriteReservation> {
    reserve_jobless(db, CrossrefWriteRoute::LegacyScheduled, root_work_id, None)
}

/// `reserveManualRecoveryCrossrefWrite` (Amendment 3 section 9.6): a blank
/// operator authorization reference is refused before any database access, and
/// the reference is recorded verbatim.
pub fn reserve_manual_recovery_crossref_write(
    db: &PgPool,
    root_work_id: Uuid,
    operator_authorization_reference: &str,
) -> ThothResult<CrossrefWriteReservation> {
    if policy::is_blank(operator_authorization_reference) {
        return Err(ThothError::CrossrefManualRecoveryRequiresReference);
    }
    reserve_jobless(
        db,
        CrossrefWriteRoute::ManualRecovery,
        root_work_id,
        Some(operator_authorization_reference),
    )
}

/// One `finaliseCrossrefWrite` presentation (R52B section 16.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinaliseCrossrefWrite {
    pub permit_id: Uuid,
    pub reservation_token: Uuid,
    pub claim_token: Option<Uuid>,
    pub observed_dois: Vec<String>,
    pub observed_doi_batch_id: String,
    pub observed_crossref_timestamp: i64,
    pub payload_digest: String,
}

/// A permit's immutable routing metadata, read by MVCC before any lock.
#[derive(Queryable)]
struct PermitMetadata {
    route: CrossrefWriteRoute,
    publisher_identity: Uuid,
    root_work_identity: Uuid,
    job_identity: Option<Uuid>,
    attempt_identity: Option<Uuid>,
}

fn permit_metadata(
    connection: &mut PgConnection,
    permit_id: Uuid,
) -> QueryResult<Option<PermitMetadata>> {
    use crate::schema::crossref_write_permit as permit;
    permit::table
        .filter(permit::permit_id.eq(permit_id))
        .select((
            permit::route,
            permit::publisher_identity,
            permit::root_work_identity,
            permit::job_identity,
            permit::attempt_identity,
        ))
        .first::<PermitMetadata>(connection)
        .optional()
}

/// The permit row and its canonical membership, ascending by code point.
pub(crate) fn permit_with_dois(
    connection: &mut PgConnection,
    permit_id: Uuid,
    lock: bool,
) -> QueryResult<Option<super::CrossrefWritePermitWithDois>> {
    use crate::schema::crossref_write_permit as permit;
    let query = permit::table.filter(permit::permit_id.eq(permit_id));
    let row = if lock {
        query
            .for_update()
            .first::<super::CrossrefWritePermit>(connection)
            .optional()?
    } else {
        query
            .first::<super::CrossrefWritePermit>(connection)
            .optional()?
    };
    let Some(row) = row else {
        return Ok(None);
    };
    let dois = membership_of(connection, permit_id)?;
    Ok(Some(super::CrossrefWritePermitWithDois {
        permit: row,
        dois,
    }))
}

/// A permit's persisted membership, ascending by code point.
fn membership_of(connection: &mut PgConnection, permit_id: Uuid) -> QueryResult<Vec<String>> {
    #[derive(QueryableByName)]
    struct Doi {
        #[diesel(sql_type = Text)]
        doi: String,
    }
    Ok(diesel::sql_query(
        "SELECT doi FROM public.crossref_write_permit_doi WHERE permit_id = $1 ORDER BY doi COLLATE \"C\"",
    )
    .bind::<SqlUuid, _>(permit_id)
    .load::<Doi>(connection)?
    .into_iter()
    .map(|row| row.doi)
    .collect())
}

/// The finalisation void reasons of groups A and B (R52B section 16.7).
fn is_finalisation_void(reason: super::CrossrefVoidReason) -> bool {
    !matches!(
        reason,
        super::CrossrefVoidReason::OwnerAbandoned | super::CrossrefVoidReason::OperatorCleanup
    )
}

/// The outcome a finalisation void recorded, recomputed from the route and the
/// persisted reason for a replay (R52B section 16.6 C2).
fn voided_outcome(
    route: CrossrefWriteRoute,
    reason: super::CrossrefVoidReason,
) -> super::CrossrefFinalisationOutcome {
    use super::CrossrefVoidReason as Reason;
    match (route, reason) {
        (
            CrossrefWriteRoute::WorkUpsert,
            Reason::BindingSuperseded | Reason::AssignmentDisabled,
        ) => super::CrossrefFinalisationOutcome::VoidedJobRetired,
        _ => super::CrossrefFinalisationOutcome::VoidedRetryable,
    }
}

/// A finalisation voiding its own reservation (R52B section 16.7).
fn void_own_reservation(
    connection: &mut PgConnection,
    permit_id: Uuid,
    reason: super::CrossrefVoidReason,
) -> QueryResult<()> {
    use crate::schema::crossref_write_permit as permit;
    diesel::update(permit::table.filter(permit::permit_id.eq(permit_id)))
        .set((
            permit::state.eq(super::CrossrefWritePermitState::Voided),
            permit::void_reason.eq(reason),
            permit::closed_at.eq(
                diesel::dsl::sql::<Nullable<diesel::sql_types::Timestamptz>>("clock_timestamp()"),
            ),
        ))
        .execute(connection)
        .map(|_| ())
}

/// Group B's binding refusal on the `WORK_UPSERT` route (R52B section 11.5):
/// the open attempt closes `CANCELLED`, the job becomes `CANCELLED` with the
/// binding reason, and — only when the binding re-resolved under `W` is the held
/// publisher and is eligible and admitted with residue above resolution — a
/// replacement is created through the single creation helper.
fn finalise_binding_refusal_replacement(
    connection: &mut PgConnection,
    job_id: Uuid,
    attempt_id: Uuid,
    held_publisher: Uuid,
    root: Uuid,
    generation: Option<i64>,
    reason: crate::model::distribution_job::DistributionJobCancellationReason,
) -> Tx<()> {
    use crate::schema::{distribution_job as job, distribution_job_attempt as attempt};
    diesel::update(attempt::table.filter(attempt::distribution_job_attempt_id.eq(attempt_id)))
        .filter(attempt::finished_at.is_null())
        .set((
            attempt::finished_at.eq(diesel::dsl::now),
            attempt::result
                .eq(crate::model::distribution_job::DistributionJobAttemptResult::Cancelled),
        ))
        .execute(connection)?;
    diesel::update(job::table.filter(job::distribution_job_id.eq(job_id)))
        .set((
            job::status.eq(DistributionJobStatus::Cancelled),
            job::cancellation_reason.eq(reason),
            job::completed_at.eq(diesel::dsl::now),
            job::claim_token.eq(None::<Uuid>),
            job::claimed_by.eq(None::<String>),
            job::claimed_at.eq(None::<crate::model::Timestamp>),
            job::lease_expires_at.eq(None::<crate::model::Timestamp>),
        ))
        .execute(connection)?;

    let Some(generation) = generation else {
        return Ok(());
    };
    if substrate::current_publisher(connection, root)? != Some(held_publisher) {
        return Ok(());
    }
    if !substrate::profile_eligible(connection, root, &CROSSREF_PROFILE)? {
        return Ok(());
    }
    let Some(activation) = substrate::crossref_activation(connection, held_publisher)? else {
        return Ok(());
    };
    if substrate::admission_row(connection, held_publisher, activation)?.is_none() {
        return Ok(());
    }
    if generation <= substrate::resolution(connection, root, CROSSREF_PROFILE.key)? {
        return Ok(());
    }
    substrate::work_upsert_create_job(
        connection,
        held_publisher,
        root,
        activation,
        CROSSREF_PROFILE.key,
        generation,
        Some(job_id),
    )?;
    Ok(())
}

/// `finaliseCrossrefWrite` (R52B section 16.6; Amendment 3 section 9.7).
///
/// `authorize_route` is the request's route-derived role check (C0). It runs
/// after the MVCC read of the permit's route and before the digest rule, any
/// lock and any credential, state or replay check.
pub fn finalise_crossref_write(
    db: &PgPool,
    input: &FinaliseCrossrefWrite,
    authorize_route: &dyn Fn(CrossrefWriteRoute) -> ThothResult<()>,
) -> ThothResult<super::CrossrefFinalisationResult> {
    use super::{
        CrossrefFinalisationOutcome as Outcome, CrossrefVoidReason as Reason,
        CrossrefWritePermitState as State,
    };
    use crate::model::distribution_job::DistributionJobCancellationReason as Cancel;

    work_upsert_transaction(db, |connection| {
        // 1-2: the route, by MVCC; C0; then the operation's own input rule.
        let Some(metadata) = permit_metadata(connection, input.permit_id)? else {
            return refuse(ThothError::CrossrefPermitNotFound);
        };
        authorize_route(metadata.route)?;
        if !policy::is_sha256_lower_hex(&input.payload_digest) {
            return refuse(ThothError::CrossrefPayloadDigestInvalid);
        }
        let job_linked = matches!(
            metadata.route,
            CrossrefWriteRoute::WorkUpsert | CrossrefWriteRoute::PublisherBackCatalogue
        );

        // 3: locks, in TLO order, from the permit's own publisher.
        let root = metadata.root_work_identity;
        substrate::share_publisher(connection, metadata.publisher_identity)?;
        let work_exists = substrate::share_work(connection, root)?;
        let bound_publisher = if work_exists {
            substrate::current_publisher(connection, root)?
        } else {
            None
        };
        let generation = if work_exists {
            Some(substrate::lock_generation(
                connection,
                root,
                CROSSREF_PROFILE.key,
            )?)
        } else {
            None
        };
        if job_linked {
            use crate::schema::{distribution_job as job, distribution_job_attempt as attempt};
            if let Some(job_id) = metadata.job_identity {
                job::table
                    .filter(job::distribution_job_id.eq(job_id))
                    .select(job::distribution_job_id)
                    .for_update()
                    .first::<Uuid>(connection)
                    .optional()?;
            }
            if let Some(attempt_id) = metadata.attempt_identity {
                attempt::table
                    .filter(attempt::distribution_job_attempt_id.eq(attempt_id))
                    .select(attempt::distribution_job_attempt_id)
                    .for_update()
                    .first::<Uuid>(connection)
                    .optional()?;
            }
        }
        if metadata.route == CrossrefWriteRoute::WorkUpsert {
            take_execution_gate(connection, CROSSREF_PROFILE.key, false)?;
        }
        let Some(current) = permit_with_dois(connection, input.permit_id, true)? else {
            return refuse(ThothError::CrossrefPermitNotFound);
        };
        let permit = &current.permit;

        // Group C.
        if permit.reservation_token != input.reservation_token {
            return refuse(ThothError::CrossrefPermitRequiresReservationToken);
        }
        match (permit.state, permit.void_reason) {
            (State::Authorized, _)
                if permit.payload_digest.as_deref() == Some(input.payload_digest.as_str()) =>
            {
                return Ok(super::CrossrefFinalisationResult {
                    outcome: Outcome::Authorized,
                    void_reason: None,
                    permit: current,
                });
            }
            (State::Voided, Some(reason)) if is_finalisation_void(reason) => {
                return Ok(super::CrossrefFinalisationResult {
                    outcome: voided_outcome(metadata.route, reason),
                    void_reason: Some(reason),
                    permit: current,
                });
            }
            (State::Reserved, _) => {}
            _ => return refuse(ThothError::CrossrefPermitIllegalTransition),
        }
        #[derive(Queryable)]
        struct Claim {
            status: DistributionJobStatus,
            claim_token: Option<Uuid>,
            publisher_id: Uuid,
            activation_id: Uuid,
        }
        let mut claim: Option<(Claim, Uuid, Uuid)> = None;
        if job_linked {
            use crate::schema::{distribution_job as job, distribution_job_attempt as attempt};
            let (Some(job_id), Some(attempt_id), Some(claim_token)) = (
                permit.distribution_job_id,
                permit.distribution_job_attempt_id,
                input.claim_token,
            ) else {
                return refuse(ThothError::CrossrefPermitClaimStale);
            };
            let row = job::table
                .filter(job::distribution_job_id.eq(job_id))
                .select((
                    job::status,
                    job::claim_token,
                    job::publisher_id,
                    job::activation_id,
                ))
                .first::<Claim>(connection)
                .optional()?;
            let open_attempt = attempt::table
                .filter(attempt::distribution_job_attempt_id.eq(attempt_id))
                .filter(attempt::claim_token.eq(claim_token))
                .filter(attempt::finished_at.is_null())
                .select(attempt::distribution_job_attempt_id)
                .first::<Uuid>(connection)
                .optional()?;
            match (row, open_attempt) {
                (Some(row), Some(_))
                    if row.status == DistributionJobStatus::Running
                        && row.claim_token == Some(claim_token) =>
                {
                    claim = Some((row, job_id, attempt_id));
                }
                _ => return refuse(ThothError::CrossrefPermitClaimStale),
            }
        } else if input.claim_token.is_some() {
            return refuse(ThothError::CrossrefPermitClaimStale);
        }

        // Group B: binding, against the held publisher.
        let binding_refusal = if !work_exists {
            Some(Reason::NoWork)
        } else if bound_publisher != Some(metadata.publisher_identity) {
            Some(Reason::BindingSuperseded)
        } else {
            let enabled_activation =
                substrate::crossref_activation(connection, metadata.publisher_identity)?;
            match (&claim, metadata.route) {
                (Some((row, job_id, _)), CrossrefWriteRoute::WorkUpsert) => {
                    if substrate::job_targets_enabled(
                        connection,
                        *job_id,
                        row.publisher_id,
                        row.activation_id,
                    )? {
                        None
                    } else if enabled_activation.is_some() {
                        Some(Reason::BindingSuperseded)
                    } else {
                        Some(Reason::AssignmentDisabled)
                    }
                }
                _ if enabled_activation.is_none() => Some(Reason::AssignmentDisabled),
                _ => None,
            }
        };
        if let Some(reason) = binding_refusal {
            let mut outcome = Outcome::VoidedRetryable;
            if let (
                Some((_, job_id, attempt_id)),
                CrossrefWriteRoute::WorkUpsert,
                Reason::BindingSuperseded | Reason::AssignmentDisabled,
            ) = (&claim, metadata.route, reason)
            {
                let cancel = if reason == Reason::AssignmentDisabled {
                    Cancel::AssignmentDisabled
                } else {
                    Cancel::BindingSuperseded
                };
                finalise_binding_refusal_replacement(
                    connection,
                    *job_id,
                    *attempt_id,
                    metadata.publisher_identity,
                    root,
                    generation,
                    cancel,
                )?;
                outcome = Outcome::VoidedJobRetired;
            }
            void_own_reservation(connection, input.permit_id, reason)?;
            let permit = permit_with_dois(connection, input.permit_id, false)?.ok_or(
                WorkUpsertTxError::Thoth(ThothError::WorkUpsertDatabaseFailure),
            )?;
            return Ok(super::CrossrefFinalisationResult {
                outcome,
                void_reason: Some(reason),
                permit,
            });
        }

        // Group A: retryable invalidation.
        let mut invalidation = None;
        if let Some((row, _, _)) = &claim {
            if metadata.route == CrossrefWriteRoute::WorkUpsert {
                let execution_enabled = {
                    use crate::schema::work_upsert_control as control;
                    control::table
                        .filter(control::execution_profile.eq(CROSSREF_PROFILE.key))
                        .select(control::execution_enabled)
                        .first::<bool>(connection)
                        .optional()?
                        .unwrap_or(false)
                };
                invalidation =
                    if substrate::admission_row(connection, row.publisher_id, row.activation_id)?
                        .is_none()
                    {
                        Some(Reason::ProfileNotAdmitted)
                    } else if !execution_enabled {
                        Some(Reason::ExecutionNotPermitted)
                    } else if !substrate::profile_eligible(connection, root, &CROSSREF_PROFILE)? {
                        Some(Reason::Ineligible)
                    } else {
                        None
                    };
            }
        }
        if invalidation.is_none() && generation != Some(permit.source_generation_witness) {
            invalidation = Some(Reason::SourceChangedDuringPreparation);
        }
        if invalidation.is_none() {
            #[derive(QueryableByName)]
            struct Dois {
                #[diesel(sql_type = Array<Text>)]
                derived: Vec<String>,
                #[diesel(sql_type = Array<Nullable<Text>>)]
                observed: Vec<Option<String>>,
            }
            let dois = diesel::sql_query(
                "SELECT public.crossref_deposit_membership($1) AS derived, \
                        coalesce((SELECT array_agg(DISTINCT public.crossref_canonical_doi(o)) \
                                    FROM unnest($2::text[]) AS o), ARRAY[]::text[]) AS observed",
            )
            .bind::<SqlUuid, _>(root)
            .bind::<Array<Text>, _>(&input.observed_dois)
            .get_result::<Dois>(connection)?;
            let reserved: std::collections::BTreeSet<&str> =
                current.dois.iter().map(String::as_str).collect();
            let derived: std::collections::BTreeSet<&str> =
                dois.derived.iter().map(String::as_str).collect();
            let observed: Option<std::collections::BTreeSet<&str>> =
                dois.observed.iter().map(|doi| doi.as_deref()).collect();
            invalidation = if derived != reserved {
                Some(Reason::DoiMembershipChanged)
            } else if observed.as_ref() != Some(&reserved) {
                Some(Reason::ArtifactDoiSetMismatch)
            } else if input.observed_doi_batch_id != permit.doi_batch_id {
                Some(Reason::ArtifactBatchIdMismatch)
            } else if input.observed_crossref_timestamp != permit.crossref_timestamp {
                Some(Reason::ArtifactTimestampMismatch)
            } else {
                None
            };
        }
        if let Some(reason) = invalidation {
            void_own_reservation(connection, input.permit_id, reason)?;
            let permit = permit_with_dois(connection, input.permit_id, false)?.ok_or(
                WorkUpsertTxError::Thoth(ThothError::WorkUpsertDatabaseFailure),
            )?;
            return Ok(super::CrossrefFinalisationResult {
                outcome: Outcome::VoidedRetryable,
                void_reason: Some(reason),
                permit,
            });
        }

        // Success: the fence, then the authorization, in one transaction.
        if let (Some((_, _, attempt_id)), CrossrefWriteRoute::WorkUpsert) = (&claim, metadata.route)
        {
            use crate::schema::distribution_job_attempt as attempt;
            diesel::update(
                attempt::table.filter(attempt::distribution_job_attempt_id.eq(*attempt_id)),
            )
            .set(attempt::fenced_at.eq(
                diesel::dsl::sql::<Nullable<diesel::sql_types::Timestamptz>>("clock_timestamp()"),
            ))
            .execute(connection)?;
        }
        {
            use crate::schema::crossref_write_permit as permit_table;
            diesel::update(permit_table::table.filter(permit_table::permit_id.eq(input.permit_id)))
                .set((
                    permit_table::state.eq(CrossrefWritePermitState::Authorized),
                    permit_table::payload_digest.eq(&input.payload_digest),
                    permit_table::authorized_at.eq(diesel::dsl::sql::<
                        Nullable<diesel::sql_types::Timestamptz>,
                    >("clock_timestamp()")),
                ))
                .execute(connection)?;
        }
        let permit = permit_with_dois(connection, input.permit_id, false)?.ok_or(
            WorkUpsertTxError::Thoth(ThothError::WorkUpsertDatabaseFailure),
        )?;
        Ok(super::CrossrefFinalisationResult {
            outcome: Outcome::Authorized,
            void_reason: None,
            permit,
        })
    })
}

/// Lock the permit row, check the reservation token, and return the row.
fn lock_permit_with_token(
    connection: &mut PgConnection,
    permit_id: Uuid,
    reservation_token: Option<Uuid>,
) -> Tx<super::CrossrefWritePermitWithDois> {
    let Some(current) = permit_with_dois(connection, permit_id, true)? else {
        return refuse(ThothError::CrossrefPermitNotFound);
    };
    if let Some(token) = reservation_token {
        if current.permit.reservation_token != token {
            return refuse(ThothError::CrossrefPermitRequiresReservationToken);
        }
    }
    Ok(current)
}

fn reload(
    connection: &mut PgConnection,
    permit_id: Uuid,
) -> Tx<super::CrossrefWritePermitWithDois> {
    permit_with_dois(connection, permit_id, false)?.ok_or(WorkUpsertTxError::Thoth(
        ThothError::WorkUpsertDatabaseFailure,
    ))
}

fn now() -> diesel::expression::SqlLiteral<Nullable<diesel::sql_types::Timestamptz>> {
    diesel::dsl::sql::<Nullable<diesel::sql_types::Timestamptz>>("clock_timestamp()")
}

/// `reportCrossrefWrite` (R52B section 16.7; Amendment 3 section 9.8): moves
/// `AUTHORIZED` to the reported outcome, writing `provider_reported_at` from the
/// server clock and closing a terminal outcome. Takes `X` only.
pub fn report_crossref_write(
    db: &PgPool,
    permit_id: Uuid,
    reservation_token: Uuid,
    outcome: super::CrossrefWriteOutcome,
    authorize_route: &dyn Fn(CrossrefWriteRoute) -> ThothResult<()>,
) -> ThothResult<super::CrossrefWritePermitWithDois> {
    use super::CrossrefWriteOutcome as Outcome;
    use crate::schema::crossref_write_permit as permit;
    work_upsert_transaction(db, |connection| {
        let Some(metadata) = permit_metadata(connection, permit_id)? else {
            return refuse(ThothError::CrossrefPermitNotFound);
        };
        authorize_route(metadata.route)?;
        let current = lock_permit_with_token(connection, permit_id, Some(reservation_token))?;
        if current.permit.state != CrossrefWritePermitState::Authorized {
            return refuse(ThothError::CrossrefPermitIllegalTransition);
        }
        let (state, closes) = match outcome {
            Outcome::Accepted => (CrossrefWritePermitState::Accepted, true),
            Outcome::Indeterminate => (CrossrefWritePermitState::Indeterminate, false),
            Outcome::NoneAttempted => (CrossrefWritePermitState::NoneAttempted, true),
        };
        let target = permit::table.filter(permit::permit_id.eq(permit_id));
        if closes {
            diesel::update(target)
                .set((
                    permit::state.eq(state),
                    permit::provider_reported_at.eq(now()),
                    permit::closed_at.eq(now()),
                ))
                .execute(connection)?;
        } else {
            diesel::update(target)
                .set((
                    permit::state.eq(state),
                    permit::provider_reported_at.eq(now()),
                ))
                .execute(connection)?;
        }
        reload(connection, permit_id)
    })
}

/// `voidCrossrefWriteReservation` (R52B section 16.7; Amendment 3 section 9.8):
/// the route owner's void, `RESERVED` to `VOIDED` with `OWNER_ABANDONED`.
pub fn void_crossref_write_reservation(
    db: &PgPool,
    permit_id: Uuid,
    reservation_token: Uuid,
    detail: &str,
    authorize_route: &dyn Fn(CrossrefWriteRoute) -> ThothResult<()>,
) -> ThothResult<super::CrossrefWritePermitWithDois> {
    work_upsert_transaction(db, |connection| {
        let Some(metadata) = permit_metadata(connection, permit_id)? else {
            return refuse(ThothError::CrossrefPermitNotFound);
        };
        authorize_route(metadata.route)?;
        if policy::is_blank(detail) {
            return refuse(ThothError::CrossrefPermitVoidRequiresDetail);
        }
        let current = lock_permit_with_token(connection, permit_id, Some(reservation_token))?;
        explicit_void(
            connection,
            current,
            super::CrossrefVoidReason::OwnerAbandoned,
            detail,
            None,
        )
    })
}

/// `voidCrossrefWriteReservationAsSuperuser` (R52B section 16.7; Amendment 3
/// section 9.8): `RESERVED` to `VOIDED` with `OPERATOR_CLEANUP`, on any route,
/// with no reservation token.
pub fn void_crossref_write_reservation_as_superuser(
    db: &PgPool,
    permit_id: Uuid,
    detail: &str,
    authorization_reference: &str,
) -> ThothResult<super::CrossrefWritePermitWithDois> {
    if policy::is_blank(detail) {
        return Err(ThothError::CrossrefPermitVoidRequiresDetail);
    }
    if policy::is_blank(authorization_reference) {
        return Err(ThothError::CrossrefVoidRequiresAuthorizationReference);
    }
    work_upsert_transaction(db, |connection| {
        if permit_metadata(connection, permit_id)?.is_none() {
            return refuse(ThothError::CrossrefPermitNotFound);
        }
        let current = lock_permit_with_token(connection, permit_id, None)?;
        explicit_void(
            connection,
            current,
            super::CrossrefVoidReason::OperatorCleanup,
            detail,
            Some(authorization_reference),
        )
    })
}

fn explicit_void(
    connection: &mut PgConnection,
    current: super::CrossrefWritePermitWithDois,
    reason: super::CrossrefVoidReason,
    detail: &str,
    authorization_reference: Option<&str>,
) -> Tx<super::CrossrefWritePermitWithDois> {
    use crate::schema::crossref_write_permit as permit;
    if current.permit.state != CrossrefWritePermitState::Reserved {
        return refuse(ThothError::CrossrefPermitVoidRequiresReserved);
    }
    let permit_id = current.permit.permit_id;
    diesel::update(permit::table.filter(permit::permit_id.eq(permit_id)))
        .set((
            permit::state.eq(CrossrefWritePermitState::Voided),
            permit::void_reason.eq(reason),
            permit::void_detail.eq(detail),
            permit::void_authorization_reference.eq(authorization_reference),
            permit::closed_at.eq(now()),
        ))
        .execute(connection)?;
    reload(connection, permit_id)
}

/// `reconcileCrossrefWritePermit` (R52B section 16.8; Amendment 3 section 9.8):
/// `A` (when the permit has an attempt), then `X`; the transition table of
/// section 16.8; then the internal clearance of the attempt.
pub fn reconcile_crossref_write_permit(
    db: &PgPool,
    permit_id: Uuid,
    outcome: super::CrossrefWriteOutcome,
    reconciliation_state: super::CrossrefReconciliationState,
    authorization_reference: &str,
) -> ThothResult<super::CrossrefWritePermitWithDois> {
    use super::{CrossrefReconciliationState as Recon, CrossrefWriteOutcome as Outcome};
    use crate::schema::crossref_write_permit as permit;
    use CrossrefWritePermitState as State;

    if policy::is_blank(authorization_reference) {
        return Err(ThothError::CrossrefReconciliationRequiresReference);
    }
    work_upsert_transaction(db, |connection| {
        let Some(metadata) = permit_metadata(connection, permit_id)? else {
            return refuse(ThothError::CrossrefPermitNotFound);
        };
        let held_attempt = match metadata.attempt_identity {
            Some(attempt_id) => {
                use crate::schema::distribution_job_attempt as attempt;
                attempt::table
                    .filter(attempt::distribution_job_attempt_id.eq(attempt_id))
                    .select(attempt::distribution_job_attempt_id)
                    .for_update()
                    .first::<Uuid>(connection)
                    .optional()?
            }
            None => None,
        };
        let current = lock_permit_with_token(connection, permit_id, None)?;
        let (from, recorded) = (current.permit.state, current.permit.reconciliation_state);
        let target = permit::table.filter(permit::permit_id.eq(permit_id));
        let resolved_state = match outcome {
            Outcome::Accepted => Some(State::Accepted),
            Outcome::NoneAttempted => Some(State::NoneAttempted),
            Outcome::Indeterminate => None,
        };
        match (from, resolved_state, reconciliation_state) {
            // A resolution of AUTHORIZED or INDETERMINATE.
            (State::Authorized | State::Indeterminate, Some(to), Recon::Reconciled) => {
                diesel::update(target)
                    .set((
                        permit::state.eq(to),
                        permit::reconciliation_state.eq(Some(Recon::Reconciled)),
                        permit::reconciliation_authorization_reference.eq(authorization_reference),
                        permit::reconciled_at.eq(now()),
                        permit::closed_at.eq(now()),
                    ))
                    .execute(connection)?;
            }
            // The annotation act, from AUTHORIZED or an unannotated INDETERMINATE.
            (
                State::Authorized,
                None,
                Recon::ReconciliationRequired | Recon::ReconciliationImpossible,
            )
            | (
                State::Indeterminate,
                None,
                Recon::ReconciliationRequired | Recon::ReconciliationImpossible,
            ) if recorded.is_none() => {
                diesel::update(target)
                    .set((
                        permit::state.eq(State::Indeterminate),
                        permit::reconciliation_state.eq(Some(reconciliation_state)),
                        permit::reconciliation_annotation_reference.eq(authorization_reference),
                        permit::reconciliation_annotated_at.eq(now()),
                    ))
                    .execute(connection)?;
            }
            // Confirmation of a terminal truth.
            (State::Accepted | State::NoneAttempted, Some(to), Recon::Reconciled) if to == from => {
                if recorded.is_none() {
                    diesel::update(target)
                        .set((
                            permit::reconciliation_state.eq(Some(Recon::Reconciled)),
                            permit::reconciliation_authorization_reference
                                .eq(authorization_reference),
                            permit::reconciled_at.eq(now()),
                        ))
                        .execute(connection)?;
                }
            }
            _ => return refuse(ThothError::CrossrefPermitIllegalTransition),
        }
        if let Some(attempt_id) = held_attempt {
            substrate::clear_fenced_abandonment(connection, attempt_id, authorization_reference)?;
        }
        reload(connection, permit_id)
    })
}
