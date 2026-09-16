//! The work-level substrate operations of `BE-06`: control, seed, admission,
//! materialization, the drain and the work-level reports.
//!
//! Every function here is a model function behind a new BE-06 entry point and
//! opens every transaction it runs through
//! [`work_upsert_transaction`](super::work_upsert_transaction) (Amendment 3
//! section 10.3, EB1). Request authorization is the resolver's, and always
//! precedes the call.

use diesel::prelude::*;
use thoth_errors::{ThothError, ThothResult};

use super::registry::{self, WorkLevelExecutionProfile};
use super::{take_execution_gate, work_upsert_transaction, WorkUpsertControl, WorkUpsertTxError};
use crate::db::PgPool;
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::schema::work_upsert_control;

/// The registered profile of `platform`, or `WORK_UPSERT_PROFILE_NOT_IMPLEMENTED`
/// before any database access.
fn registered(platform: DistributionPlatform) -> ThothResult<&'static WorkLevelExecutionProfile> {
    registry::execution_profile(platform).ok_or(ThothError::WorkUpsertProfileNotImplemented)
}

/// Read one control row `FOR UPDATE`; its absence is the released
/// `EntityNotFound`.
fn lock_control(
    connection: &mut PgConnection,
    profile: DistributionPlatform,
) -> Result<WorkUpsertControl, WorkUpsertTxError> {
    work_upsert_control::table
        .filter(work_upsert_control::execution_profile.eq(profile))
        .select((
            work_upsert_control::execution_profile,
            work_upsert_control::capture_enabled,
            work_upsert_control::execution_enabled,
        ))
        .for_update()
        .first::<WorkUpsertControl>(connection)
        .optional()?
        .ok_or(WorkUpsertTxError::Thoth(ThothError::EntityNotFound))
}

/// `enableWorkUpsertCapture` (Amendment 3 section 9.1): `(false, false)` to
/// `(true, false)`; idempotent; takes no execution gate.
pub fn enable_work_upsert_capture(
    db: &PgPool,
    platform: DistributionPlatform,
) -> ThothResult<WorkUpsertControl> {
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        let control = lock_control(connection, profile.key)?;
        if control.capture_enabled {
            return Ok(control);
        }
        diesel::update(work_upsert_control::table)
            .filter(work_upsert_control::execution_profile.eq(profile.key))
            .set((
                work_upsert_control::capture_enabled.eq(true),
                work_upsert_control::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection)?;
        Ok(WorkUpsertControl {
            capture_enabled: true,
            ..control
        })
    })
}

/// `setWorkUpsertExecution` (Amendment 3 section 9.1): takes the profile's
/// execution gate `Q` EXCLUSIVE first and holds it to commit, so it returns
/// only after every consumer that read the previous value has committed.
pub fn set_work_upsert_execution(
    db: &PgPool,
    platform: DistributionPlatform,
    enabled: bool,
) -> ThothResult<WorkUpsertControl> {
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        take_execution_gate(connection, profile.key, true)?;
        let control = lock_control(connection, profile.key)?;
        if enabled && !control.capture_enabled {
            return Err(ThothError::WorkUpsertCaptureNotEnabled.into());
        }
        if control.execution_enabled == enabled {
            return Ok(control);
        }
        diesel::update(work_upsert_control::table)
            .filter(work_upsert_control::execution_profile.eq(profile.key))
            .set((
                work_upsert_control::execution_enabled.eq(enabled),
                work_upsert_control::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection)?;
        Ok(WorkUpsertControl {
            execution_enabled: enabled,
            ..control
        })
    })
}

/// Report 10, `workUpsertControl`: the control row of every profile, in
/// canonical platform order.
pub fn work_upsert_controls(db: &PgPool) -> ThothResult<Vec<WorkUpsertControl>> {
    let mut rows = work_upsert_transaction(db, |connection| {
        Ok(work_upsert_control::table
            .select((
                work_upsert_control::execution_profile,
                work_upsert_control::capture_enabled,
                work_upsert_control::execution_enabled,
            ))
            .load::<WorkUpsertControl>(connection)?)
    })?;
    rows.sort_by_key(|row| {
        DistributionPlatform::ALL
            .iter()
            .position(|platform| *platform == row.execution_profile)
    });
    Ok(rows)
}

/// A count as a GraphQL `Int`; an unrepresentable count is `INTERNAL_ERROR`
/// with an empty message (Amendment 3 section 7.3).
fn as_int(value: impl TryInto<i32>) -> ThothResult<i32> {
    value
        .try_into()
        .map_err(|_| ThothError::InternalError(String::new()))
}

#[derive(QueryableByName)]
struct UuidRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    work_id: uuid::Uuid,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// Whether the publisher exists, by MVCC.
fn publisher_exists(connection: &mut PgConnection, publisher_id: uuid::Uuid) -> QueryResult<bool> {
    use crate::schema::publisher;
    publisher::table
        .filter(publisher::publisher_id.eq(publisher_id))
        .select(publisher::publisher_id)
        .first::<uuid::Uuid>(connection)
        .optional()
        .map(|row| row.is_some())
}

/// Whether the publisher has an enabled `CROSSREF` assignment (E1-E3), by MVCC.
fn crossref_publisher_covered(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
) -> QueryResult<bool> {
    use diesel::sql_types::{Bool, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Covered {
        #[diesel(sql_type = Bool)]
        covered: bool,
    }
    if !super::policy::crossref_route_is_automatic_push() {
        return Ok(false);
    }
    diesel::sql_query(format!(
        "SELECT {} AS covered",
        super::policy::CROSSREF_PUBLISHER_COVERAGE_SQL.replace("{publisher}", "$1")
    ))
    .bind::<SqlUuid, _>(publisher_id)
    .get_result::<Covered>(connection)
    .map(|row| row.covered)
}

/// The control row's `capture_enabled`, by MVCC.
fn capture_enabled(
    connection: &mut PgConnection,
    profile: DistributionPlatform,
) -> Result<bool, WorkUpsertTxError> {
    work_upsert_control::table
        .filter(work_upsert_control::execution_profile.eq(profile))
        .select(work_upsert_control::capture_enabled)
        .first::<bool>(connection)
        .optional()?
        .ok_or(WorkUpsertTxError::Thoth(ThothError::EntityNotFound))
}

/// The Crossref-eligible population of a publisher on current ownership (E1-E6
/// of R52B section 14.2), ascending by `work_id`: one statement for the
/// SQL-evaluable clauses and the evaluated abstracts, then the abstract clause
/// in Rust. The seed's census, admission and `remainingUncovered` use it.
pub(crate) fn crossref_eligible_population(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
) -> QueryResult<Vec<uuid::Uuid>> {
    use diesel::sql_types::{Array, Text, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Candidate {
        #[diesel(sql_type = SqlUuid)]
        work_id: uuid::Uuid,
        #[diesel(sql_type = Array<Text>)]
        abstracts: Vec<String>,
    }
    if !super::policy::crossref_route_is_automatic_push() {
        return Ok(Vec::new());
    }
    let rows = diesel::sql_query(format!(
        "SELECT w.work_id, {abstracts} AS abstracts \
           FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
          WHERE i.publisher_id = $1 AND {coverage} AND {eligibility} \
          ORDER BY w.work_id",
        abstracts = super::policy::CROSSREF_EVALUATED_ABSTRACTS_SQL,
        coverage =
            super::policy::CROSSREF_PUBLISHER_COVERAGE_SQL.replace("{publisher}", "i.publisher_id"),
        eligibility = super::policy::CROSSREF_SQL_ELIGIBILITY,
    ))
    .bind::<SqlUuid, _>(publisher_id)
    .load::<Candidate>(connection)?;
    Ok(rows
        .into_iter()
        .filter(|row| super::policy::crossref_abstracts_normalise(&row.abstracts))
        .map(|row| row.work_id)
        .collect())
}

/// R52B section 21.3's census statement, exactly: the eligible Works of the
/// population, on current ownership, with no `CROSSREF` generation row or a
/// `0` row.
pub(crate) fn crossref_census_uncovered(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
    population: &[uuid::Uuid],
) -> QueryResult<i64> {
    use diesel::sql_types::{Array, Uuid as SqlUuid};
    diesel::sql_query(
        "SELECT count(*) AS count \
           FROM unnest($2::uuid[]) AS e(work_id) \
           JOIN public.work w    ON w.work_id = e.work_id \
           JOIN public.imprint i ON i.imprint_id = w.imprint_id AND i.publisher_id = $1 \
           LEFT JOIN public.work_upsert_generation g \
                  ON g.work_id = w.work_id AND g.execution_profile = 'CROSSREF' \
          WHERE g.work_id IS NULL \
             OR g.source_generation = 0",
    )
    .bind::<SqlUuid, _>(publisher_id)
    .bind::<Array<SqlUuid>, _>(population)
    .get_result::<CountRow>(connection)
    .map(|row| row.count)
}

/// `remainingUncovered`: the census over the current eligible population.
fn crossref_remaining_uncovered(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
) -> QueryResult<i64> {
    let population = crossref_eligible_population(connection, publisher_id)?;
    crossref_census_uncovered(connection, publisher_id, &population)
}

/// Lock the binding publisher `FOR SHARE` (LO-5's first class).
fn share_publisher(connection: &mut PgConnection, publisher_id: uuid::Uuid) -> QueryResult<()> {
    use crate::schema::publisher;
    publisher::table
        .filter(publisher::publisher_id.eq(publisher_id))
        .select(publisher::publisher_id)
        .for_share()
        .first::<uuid::Uuid>(connection)
        .optional()
        .map(|_| ())
}

/// Lock the Work `FOR SHARE`; `false` when it does not exist.
fn share_work(connection: &mut PgConnection, work_id: uuid::Uuid) -> QueryResult<bool> {
    use crate::schema::work;
    work::table
        .filter(work::work_id.eq(work_id))
        .select(work::work_id)
        .for_share()
        .first::<uuid::Uuid>(connection)
        .optional()
        .map(|row| row.is_some())
}

/// The Work's current publisher, `work -> imprint -> publisher`, in a fresh
/// statement.
fn current_publisher(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
) -> QueryResult<Option<uuid::Uuid>> {
    use crate::schema::{imprint, work};
    work::table
        .inner_join(imprint::table)
        .filter(work::work_id.eq(work_id))
        .select(imprint::publisher_id)
        .first::<uuid::Uuid>(connection)
        .optional()
}

/// Insert the `(work, profile)` generation row at `0` if absent, then lock it
/// `FOR UPDATE` and return its `source_generation`.
pub(crate) fn lock_generation(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
    profile: DistributionPlatform,
) -> QueryResult<i64> {
    use crate::schema::work_upsert_generation as generation;
    diesel::insert_into(generation::table)
        .values((
            generation::work_id.eq(work_id),
            generation::execution_profile.eq(profile),
            generation::source_generation.eq(0_i64),
        ))
        .on_conflict((generation::work_id, generation::execution_profile))
        .do_nothing()
        .execute(connection)?;
    generation::table
        .filter(generation::work_id.eq(work_id))
        .filter(generation::execution_profile.eq(profile))
        .select(generation::source_generation)
        .for_update()
        .first::<i64>(connection)
}

/// Raise a locked `0` generation row to `1`: the seed event, exactly once.
fn raise_zero_generation(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
    profile: DistributionPlatform,
) -> QueryResult<()> {
    use crate::schema::work_upsert_generation as generation;
    diesel::update(generation::table)
        .filter(generation::work_id.eq(work_id))
        .filter(generation::execution_profile.eq(profile))
        .filter(generation::source_generation.eq(0_i64))
        .set((
            generation::source_generation.eq(1_i64),
            generation::updated_at.eq(diesel::dsl::now),
        ))
        .execute(connection)
        .map(|_| ())
}

/// One seed unit (Amendment 3 section 9.2), in its own transaction, under
/// LO-5's prefix. It writes generation state only.
pub(crate) fn seed_unit(
    db: &PgPool,
    publisher_id: uuid::Uuid,
    work_id: uuid::Uuid,
) -> ThothResult<super::SeedUnitOutcome> {
    use super::SeedUnitOutcome;
    let profile = DistributionPlatform::Crossref;
    work_upsert_transaction(db, |connection| {
        share_publisher(connection, publisher_id)?;
        if !share_work(connection, work_id)? {
            return Ok(SeedUnitOutcome::NoWork);
        }
        if current_publisher(connection, work_id)? != Some(publisher_id) {
            return Ok(SeedUnitOutcome::BindingMovedRetryLater);
        }
        if lock_generation(connection, work_id, profile)? == 0 {
            raise_zero_generation(connection, work_id, profile)?;
            Ok(SeedUnitOutcome::Seeded)
        } else {
            Ok(SeedUnitOutcome::Observed)
        }
    })
}

/// `seedCrossrefWorkUpsert` (Amendment 3 section 9.2): one bounded batch of the
/// lock-and-inspect seed for a publisher.
pub fn seed_crossref_work_upsert(
    db: &PgPool,
    publisher_id: uuid::Uuid,
    limit: Option<i32>,
) -> ThothResult<super::SeedCrossrefWorkUpsertResult> {
    use super::{policy, SeedUnitOutcome};
    use diesel::sql_types::{BigInt, Uuid as SqlUuid};

    let limit = policy::clamp_limit(limit, 100, 500);
    let (candidates, remaining) = work_upsert_transaction(db, |connection| {
        if !publisher_exists(connection, publisher_id)? {
            return Err(ThothError::EntityNotFound.into());
        }
        if !capture_enabled(connection, DistributionPlatform::Crossref)? {
            return Err(ThothError::WorkUpsertCaptureNotEnabled.into());
        }
        if !crossref_publisher_covered(connection, publisher_id)? {
            return Err(ThothError::CrossrefPublisherNotCovered.into());
        }
        if limit == 0 {
            let remaining = crossref_remaining_uncovered(connection, publisher_id)?;
            return Ok((Vec::new(), Some(remaining)));
        }
        let candidates = diesel::sql_query(format!(
            "SELECT w.work_id FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
              WHERE i.publisher_id = $1 \
                AND NOT EXISTS (SELECT 1 FROM public.work_upsert_generation g \
                                 WHERE g.work_id = w.work_id AND g.execution_profile = 'CROSSREF' \
                                   AND g.source_generation > 0) \
                AND {} \
              ORDER BY w.work_id LIMIT $2",
            policy::CROSSREF_SQL_ELIGIBILITY
        ))
        .bind::<SqlUuid, _>(publisher_id)
        .bind::<BigInt, _>(limit)
        .load::<UuidRow>(connection)?;
        let candidates: Vec<uuid::Uuid> = candidates.into_iter().map(|row| row.work_id).collect();
        let remaining = if candidates.is_empty() {
            Some(crossref_remaining_uncovered(connection, publisher_id)?)
        } else {
            None
        };
        Ok((candidates, remaining))
    })?;

    let mut result = super::SeedCrossrefWorkUpsertResult::default();
    for work_id in candidates {
        let outcome = seed_unit(db, publisher_id, work_id)?;
        result.examined += 1;
        match outcome {
            SeedUnitOutcome::Seeded => result.seeded += 1,
            SeedUnitOutcome::Observed => result.observed += 1,
            SeedUnitOutcome::BindingMovedRetryLater => result.binding_moved_retry_later += 1,
            SeedUnitOutcome::NoWork => result.no_work += 1,
        }
    }
    let remaining = match remaining {
        Some(remaining) => remaining,
        None => work_upsert_transaction(db, |connection| {
            Ok(crossref_remaining_uncovered(connection, publisher_id)?)
        })?,
    };
    result.remaining_uncovered = as_int(remaining)?;
    Ok(result)
}

/// The activation of the publisher's enabled `CROSSREF` assignment, by MVCC.
fn crossref_activation(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
) -> QueryResult<Option<uuid::Uuid>> {
    use crate::schema::publisher_distribution_platform as assignment;
    assignment::table
        .filter(assignment::publisher_id.eq(publisher_id))
        .filter(assignment::platform.eq(DistributionPlatform::Crossref))
        .filter(assignment::enabled.eq(true))
        .select(assignment::activation_id)
        .first::<uuid::Uuid>(connection)
        .optional()
}

/// The admission row of an exact binding, if any.
fn admission_row(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
    activation_id: uuid::Uuid,
) -> QueryResult<Option<super::WorkUpsertAdmission>> {
    use crate::schema::work_upsert_admission as admission;
    admission::table
        .filter(admission::execution_profile.eq(DistributionPlatform::Crossref))
        .filter(admission::publisher_id.eq(publisher_id))
        .filter(admission::activation_id.eq(activation_id))
        .select((
            admission::execution_profile,
            admission::publisher_id,
            admission::activation_id,
            admission::evidence_reference,
            admission::actor,
            admission::admitted_at,
        ))
        .first::<super::WorkUpsertAdmission>(connection)
        .optional()
}

/// `admitCrossrefWorkUpsert` (Amendment 3 section 9.3): census and admission of
/// the exact `('CROSSREF', publisher, activation)` binding in one `READ
/// COMMITTED` transaction. `actor` is the request principal's user id. No job,
/// target or attempt is created.
pub fn admit_crossref_work_upsert(
    db: &PgPool,
    publisher_id: uuid::Uuid,
    evidence_reference: &str,
    actor: &str,
) -> ThothResult<super::WorkUpsertAdmission> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};

    if super::policy::is_blank(evidence_reference) {
        return Err(ThothError::WorkUpsertAdmissionRequiresEvidenceReference);
    }
    work_upsert_transaction(db, |connection| {
        {
            use crate::schema::publisher;
            let held = publisher::table
                .filter(publisher::publisher_id.eq(publisher_id))
                .select(publisher::publisher_id)
                .for_share()
                .first::<uuid::Uuid>(connection)
                .optional()?;
            if held.is_none() {
                return Err(ThothError::EntityNotFound.into());
            }
        }
        let activation = match crossref_activation(connection, publisher_id)? {
            Some(activation) if super::policy::crossref_route_is_automatic_push() => activation,
            _ => return Err(ThothError::CrossrefPublisherNotCovered.into()),
        };
        if let Some(existing) = admission_row(connection, publisher_id, activation)? {
            return Ok(existing);
        }
        let population = crossref_eligible_population(connection, publisher_id)?;
        if crossref_census_uncovered(connection, publisher_id, &population)? > 0 {
            return Err(ThothError::WorkUpsertAdmissionCensusNotEmpty.into());
        }
        let inserted = diesel::sql_query(
            "INSERT INTO public.work_upsert_admission (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
             VALUES ('CROSSREF', $1, $2, $3, $4) \
             ON CONFLICT (execution_profile, publisher_id, activation_id) DO NOTHING \
             RETURNING execution_profile, publisher_id, activation_id, evidence_reference, actor, admitted_at",
        )
        .bind::<SqlUuid, _>(publisher_id)
        .bind::<SqlUuid, _>(activation)
        .bind::<Text, _>(evidence_reference)
        .bind::<Text, _>(actor)
        .get_result::<super::WorkUpsertAdmission>(connection)
        .optional()?;
        if let Some(inserted) = inserted {
            return Ok(inserted);
        }
        admission_row(connection, publisher_id, activation)?.ok_or(WorkUpsertTxError::Thoth(
            ThothError::WorkUpsertDatabaseFailure,
        ))
    })
}

/// Report 9, `workUpsertAdmissions` (Amendment 3 section 4.4): the admission
/// rows of a profile whose activation is the publisher's currently enabled
/// assignment's, ascending by publisher.
pub fn work_upsert_admissions(
    db: &PgPool,
    platform: DistributionPlatform,
) -> ThothResult<Vec<super::WorkUpsertAdmission>> {
    use crate::schema::publisher_distribution_platform as assignment;
    use crate::schema::work_upsert_admission as admission;
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        Ok(admission::table
            .inner_join(
                assignment::table.on(assignment::publisher_id
                    .eq(admission::publisher_id)
                    .and(assignment::platform.eq(admission::execution_profile))
                    .and(assignment::activation_id.eq(admission::activation_id))
                    .and(assignment::enabled.eq(true))),
            )
            .filter(admission::execution_profile.eq(profile.key))
            .select((
                admission::execution_profile,
                admission::publisher_id,
                admission::activation_id,
                admission::evidence_reference,
                admission::actor,
                admission::admitted_at,
            ))
            .order(admission::publisher_id.asc())
            .load::<super::WorkUpsertAdmission>(connection)?)
    })
}

/// R52B section 9.1's `resolution(w, p)`, through the migration's function.
fn resolution(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
    profile: DistributionPlatform,
) -> QueryResult<i64> {
    use diesel::sql_types::{BigInt, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Resolution {
        #[diesel(sql_type = BigInt)]
        resolution: i64,
    }
    diesel::sql_query("SELECT public.work_upsert_resolution($1, $2) AS resolution")
        .bind::<SqlUuid, _>(work_id)
        .bind::<crate::schema::sql_types::DistributionPlatform, _>(profile)
        .get_result::<Resolution>(connection)
        .map(|row| row.resolution)
}

/// The profile's whole eligibility predicate for one Work on current ownership
/// (R52B section 14.2, E1-E6): the one SQL expression and the one abstract
/// function the drain selector also uses (Amendment 3 section 9.4, M15).
pub(crate) fn profile_eligible(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
    profile: &WorkLevelExecutionProfile,
) -> QueryResult<bool> {
    use diesel::sql_types::{Array, Bool, Text, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Eligibility {
        #[diesel(sql_type = Bool)]
        sql_eligible: bool,
        #[diesel(sql_type = Array<Text>)]
        abstracts: Vec<String>,
    }
    debug_assert_eq!(profile.key, DistributionPlatform::Crossref);
    if !super::policy::crossref_route_is_automatic_push() {
        return Ok(false);
    }
    let row = diesel::sql_query(format!(
        "SELECT ({coverage} AND {eligibility}) AS sql_eligible, {abstracts} AS abstracts \
           FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
          WHERE w.work_id = $1",
        coverage =
            super::policy::CROSSREF_PUBLISHER_COVERAGE_SQL.replace("{publisher}", "i.publisher_id"),
        eligibility = super::policy::CROSSREF_SQL_ELIGIBILITY,
        abstracts = super::policy::CROSSREF_EVALUATED_ABSTRACTS_SQL,
    ))
    .bind::<SqlUuid, _>(work_id)
    .get_result::<Eligibility>(connection)
    .optional()?;
    Ok(row.is_some_and(|row| {
        row.sql_eligible && super::policy::crossref_abstracts_normalise(&row.abstracts)
    }))
}

/// Why a `PENDING` job's binding is obsolete under the held publisher, if it is
/// (R52B section 9.4).
fn stale_binding_reason(
    connection: &mut PgConnection,
    job: &crate::model::distribution_job::DistributionJob,
    held_publisher: uuid::Uuid,
    profile: &WorkLevelExecutionProfile,
) -> QueryResult<Option<crate::model::distribution_job::DistributionJobCancellationReason>> {
    use crate::model::distribution_job::DistributionJobCancellationReason as Reason;
    use crate::schema::{distribution_job_target, publisher_distribution_platform as assignment};

    if job.publisher_id != held_publisher {
        return Ok(Some(Reason::BindingSuperseded));
    }
    let targets = distribution_job_target::table
        .filter(distribution_job_target::distribution_job_id.eq(job.distribution_job_id))
        .select(distribution_job_target::platform)
        .load::<DistributionPlatform>(connection)?;
    let enabled: Vec<(DistributionPlatform, uuid::Uuid)> = assignment::table
        .filter(assignment::publisher_id.eq(job.publisher_id))
        .filter(assignment::enabled.eq(true))
        .select((assignment::platform, assignment::activation_id))
        .load(connection)?;
    let current = targets
        .iter()
        .all(|target| enabled.contains(&(*target, job.activation_id)));
    if current {
        return Ok(None);
    }
    if enabled.iter().any(|(platform, _)| *platform == profile.key) {
        Ok(Some(Reason::BindingSuperseded))
    } else {
        Ok(Some(Reason::AssignmentDisabled))
    }
}

/// The single creation helper of R52B section 11.2: the only place a
/// `WORK_UPSERT` job is created.
///
/// The caller holds `P FOR SHARE`, `W FOR SHARE` and `G FOR UPDATE`, which is
/// what makes `job_ordinal` race-free. Every statement is converted by the
/// scoped conversion, so it serves both the EB1 and EB2 creation sites.
pub(crate) fn work_upsert_create_job(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
    work_id: uuid::Uuid,
    activation_id: uuid::Uuid,
    platform: DistributionPlatform,
    generation: i64,
    predecessor_job_id: Option<uuid::Uuid>,
) -> ThothResult<crate::model::distribution_job::DistributionJob> {
    use super::WorkUpsertQueryResultExt;
    use crate::model::distribution_job::{
        DistributionJob, DistributionJobKind, DistributionJobStatus,
    };
    use crate::schema::{distribution_job, distribution_job_target};

    let profile = registered(platform)?;
    let existing = distribution_job::table
        .filter(distribution_job::kind.eq(DistributionJobKind::WorkUpsert))
        .filter(distribution_job::work_identity.eq(work_id))
        .filter(distribution_job::execution_profile.eq(profile.key))
        .count()
        .get_result::<i64>(connection)
        .work_upsert()?;
    let job_ordinal =
        i32::try_from(existing + 1).map_err(|_| ThothError::WorkUpsertDatabaseFailure)?;
    let deduplication_key = format!(
        "WORK_UPSERT:{publisher_id}:{work_id}:{}:{activation_id}:{generation}:{job_ordinal}",
        profile.key
    );
    let job = diesel::insert_into(distribution_job::table)
        .values((
            distribution_job::kind.eq(DistributionJobKind::WorkUpsert),
            distribution_job::publisher_id.eq(publisher_id),
            distribution_job::work_id.eq(work_id),
            distribution_job::activation_id.eq(activation_id),
            distribution_job::status.eq(DistributionJobStatus::Pending),
            distribution_job::deduplication_key.eq(&deduplication_key),
            distribution_job::attempt_count.eq(0),
            distribution_job::available_at.eq(diesel::dsl::now),
            distribution_job::execution_profile.eq(profile.key),
            distribution_job::work_identity.eq(work_id),
            distribution_job::created_generation.eq(generation),
            distribution_job::job_ordinal.eq(job_ordinal),
            distribution_job::predecessor_job_id.eq(predecessor_job_id),
        ))
        .get_result::<DistributionJob>(connection)
        .work_upsert()?;
    if let Some(predecessor) = predecessor_job_id {
        diesel::update(distribution_job::table)
            .filter(distribution_job::distribution_job_id.eq(predecessor))
            .set(distribution_job::superseded_by_job_id.eq(job.distribution_job_id))
            .execute(connection)
            .work_upsert()?;
    }
    let targets: Vec<_> = profile
        .targets
        .iter()
        .map(|platform| {
            (
                distribution_job_target::distribution_job_id.eq(job.distribution_job_id),
                distribution_job_target::platform.eq(*platform),
            )
        })
        .collect();
    diesel::insert_into(distribution_job_target::table)
        .values(&targets)
        .execute(connection)
        .work_upsert()?;
    Ok(job)
}

/// R52B section 9.3's materialization unit for one `(work, profile)`, on the
/// caller's transaction, under LO-5. `force` skips step 9 only.
pub(crate) fn materialization_unit(
    connection: &mut PgConnection,
    work_id: uuid::Uuid,
    profile: &'static WorkLevelExecutionProfile,
    force: bool,
) -> Result<super::WorkUpsertMaterialization, WorkUpsertTxError> {
    use super::{WorkUpsertMaterialization, WorkUpsertMaterializationOutcome as Outcome};
    use crate::model::distribution_job::{
        DistributionJob, DistributionJobKind, DistributionJobStatus,
    };
    use crate::schema::distribution_job;

    let end = |outcome, rebound, job| {
        Ok(WorkUpsertMaterialization {
            outcome,
            rebound,
            job,
        })
    };

    // 1-4: the binding, P FOR SHARE, W FOR SHARE, the binding re-read under W.
    let Some(publisher_id) = current_publisher(connection, work_id)? else {
        return end(Outcome::NoWork, false, None);
    };
    share_publisher(connection, publisher_id)?;
    if !share_work(connection, work_id)? {
        return end(Outcome::NoWork, false, None);
    }
    if current_publisher(connection, work_id)? != Some(publisher_id) {
        return end(Outcome::BindingMovedRetryLater, false, None);
    }
    // 5: capture.
    if !capture_enabled(connection, profile.key)? {
        return end(Outcome::CaptureDisabled, false, None);
    }
    // 6: G FOR UPDATE.
    let mut generation = lock_generation(connection, work_id, profile.key)?;
    // 7: the actionable row, and section 9.4.
    let actionable = distribution_job::table
        .filter(distribution_job::kind.eq(DistributionJobKind::WorkUpsert))
        .filter(distribution_job::work_id.eq(work_id))
        .filter(distribution_job::execution_profile.eq(profile.key))
        .filter(distribution_job::status.eq_any([
            DistributionJobStatus::Pending,
            DistributionJobStatus::Running,
        ]))
        .for_update()
        .first::<DistributionJob>(connection)
        .optional()?;
    let mut rebound = false;
    let mut predecessor = None;
    if let Some(job) = actionable {
        if job.status == DistributionJobStatus::Running {
            return end(Outcome::RunningInFlight, false, Some(job));
        }
        match stale_binding_reason(connection, &job, publisher_id, profile)? {
            None => return end(Outcome::PendingCurrent, false, Some(job)),
            Some(reason) => {
                diesel::update(distribution_job::table)
                    .filter(distribution_job::distribution_job_id.eq(job.distribution_job_id))
                    .filter(distribution_job::status.eq(DistributionJobStatus::Pending))
                    .set((
                        distribution_job::status.eq(DistributionJobStatus::Cancelled),
                        distribution_job::cancellation_reason.eq(reason),
                        distribution_job::completed_at.eq(diesel::dsl::now),
                    ))
                    .execute(connection)?;
                rebound = true;
                predecessor = Some(job.distribution_job_id);
            }
        }
    }
    // 8: the seed event, exactly once.
    if generation == 0 {
        raise_zero_generation(connection, work_id, profile.key)?;
        generation = 1;
    }
    // 9: resolution.
    if !force && generation <= resolution(connection, work_id, profile.key)? {
        return end(Outcome::Resolved, rebound, None);
    }
    // 10: eligibility, then admission of the current binding.
    if !profile_eligible(connection, work_id, profile)? {
        return end(Outcome::Ineligible, rebound, None);
    }
    let Some(activation_id) = crossref_activation(connection, publisher_id)? else {
        return end(Outcome::Ineligible, rebound, None);
    };
    if admission_row(connection, publisher_id, activation_id)?.is_none() {
        return end(Outcome::ResidueNotAdmitted, rebound, None);
    }
    // 11: creation site 1.
    let job = work_upsert_create_job(
        connection,
        publisher_id,
        work_id,
        activation_id,
        profile.key,
        generation,
        predecessor,
    )?;
    end(Outcome::Created, rebound, Some(job))
}

/// `materializeWorkUpsertJob` (Amendment 3 section 9.5): R52B section 9.3's
/// unit for one Work, whatever the drainable set says, in one transaction.
pub fn materialize_work_upsert_job(
    db: &PgPool,
    work_id: uuid::Uuid,
    platform: DistributionPlatform,
    force: bool,
) -> ThothResult<super::WorkUpsertMaterialization> {
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        materialization_unit(connection, work_id, profile, force)
    })
}
