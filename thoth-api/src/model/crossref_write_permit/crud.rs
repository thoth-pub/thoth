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

use super::{CrossrefWriteReservation, CrossrefWriteRoute};
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
