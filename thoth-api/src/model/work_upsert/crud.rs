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
pub(crate) fn registered(
    platform: DistributionPlatform,
) -> ThothResult<&'static WorkLevelExecutionProfile> {
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
pub(crate) fn crossref_publisher_covered(
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
pub(crate) fn capture_enabled(
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
pub(crate) fn share_publisher(
    connection: &mut PgConnection,
    publisher_id: uuid::Uuid,
) -> QueryResult<()> {
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
pub(crate) fn share_work(connection: &mut PgConnection, work_id: uuid::Uuid) -> QueryResult<bool> {
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
pub(crate) fn current_publisher(
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
pub(crate) fn crossref_activation(
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
pub(crate) fn admission_row(
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
        // R52B T166 (#848 comment 5701572954): no admission while capture is disabled. Read before the activation,
        // any existing admission and the census, so the refusal writes nothing and runs no census.
        if !capture_enabled(connection, DistributionPlatform::Crossref)? {
            return Err(ThothError::WorkUpsertCaptureNotEnabled.into());
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
pub(crate) fn resolution(
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

/// `materializeWorkUpsertJob` as its GraphQL result needs it: the unit of
/// [`materialize_work_upsert_job`], and the job the unit created or found with
/// that job's targets and attempts, read by BE-06 statements inside the unit's
/// own transaction (Amendment 3 section 10.3, EB1 and X11). No child of the
/// returned job is left to the released request loaders.
pub fn materialize_work_upsert_job_with_payload(
    db: &PgPool,
    work_id: uuid::Uuid,
    platform: DistributionPlatform,
    force: bool,
) -> ThothResult<(
    super::WorkUpsertMaterialization,
    Option<crate::model::distribution_job::DistributionJobPayload>,
)> {
    use crate::model::distribution_job::{
        DistributionJobAttempt, DistributionJobPayload, DistributionJobTarget,
    };
    use crate::schema::{distribution_job_attempt, distribution_job_target};
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        let unit = materialization_unit(connection, work_id, profile, force)?;
        let payload = match &unit.job {
            Some(job) => {
                // The released job payload's orders: targets in canonical
                // platform order, attempts most recent first.
                let targets = distribution_job_target::table
                    .filter(
                        distribution_job_target::distribution_job_id.eq(job.distribution_job_id),
                    )
                    .order(distribution_job_target::platform.asc())
                    .load::<DistributionJobTarget>(connection)?;
                let attempts = distribution_job_attempt::table
                    .filter(
                        distribution_job_attempt::distribution_job_id.eq(job.distribution_job_id),
                    )
                    .order(distribution_job_attempt::attempt_number.desc())
                    .load::<DistributionJobAttempt>(connection)?;
                Some(DistributionJobPayload::preloaded(
                    job.clone(),
                    targets,
                    attempts,
                ))
            }
            None => None,
        };
        Ok((unit, payload))
    })
}

/// One row of the drain selector: a member of R52B section 9.2's candidate set
/// with the inputs of Amendment 3 section 9.4's filter.
#[derive(QueryableByName)]
pub(crate) struct CandidateRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub work_id: uuid::Uuid,
    #[diesel(sql_type = crate::schema::sql_types::DistributionPlatform)]
    pub execution_profile: DistributionPlatform,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub class: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub capture_enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub admitted: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub sql_eligible: bool,
    #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Text>)]
    pub abstracts: Vec<String>,
}

impl CandidateRow {
    /// Whether the abstract-normalisation clause holds.
    pub(crate) fn abstracts_normalise(&self) -> bool {
        super::policy::crossref_abstracts_normalise(&self.abstracts)
    }

    /// Membership of `D` (Amendment 3 section 9.4): capture enabled and either
    /// `STALE_PENDING`, or admitted and wholly eligible.
    pub(crate) fn drainable(&self) -> bool {
        self.capture_enabled
            && (self.class == "STALE_PENDING"
                || (self.admitted && self.sql_eligible && self.abstracts_normalise()))
    }
}

/// The drain selector's one statement: every member of the candidate set `C`
/// over the requested profiles, with no `LIMIT` and no row lock, ascending by
/// `(work_id, execution_profile)`.
pub(crate) fn candidate_rows(
    connection: &mut PgConnection,
    profiles: &[&'static WorkLevelExecutionProfile],
) -> QueryResult<Vec<CandidateRow>> {
    use super::policy;
    use diesel::sql_types::Array;
    debug_assert!(profiles
        .iter()
        .all(|profile| profile.key == DistributionPlatform::Crossref));
    let keys: Vec<DistributionPlatform> = profiles.iter().map(|profile| profile.key).collect();
    let eligible_route = policy::crossref_route_is_automatic_push();
    diesel::sql_query(format!(
        "WITH c AS ( \
             SELECT g.work_id, g.execution_profile, 'RESIDUE' AS class \
               FROM public.work_upsert_generation g \
              WHERE g.execution_profile = ANY($1) \
                AND NOT EXISTS (SELECT 1 FROM public.distribution_job j \
                                 WHERE j.kind = 'WORK_UPSERT' AND j.work_id = g.work_id \
                                   AND j.execution_profile = g.execution_profile \
                                   AND j.status IN ('PENDING', 'RUNNING')) \
                AND (g.source_generation = 0 \
                     OR g.source_generation > public.work_upsert_resolution(g.work_id, g.execution_profile)) \
             UNION \
             SELECT j.work_id, j.execution_profile, 'STALE_PENDING' \
               FROM public.distribution_job j \
              WHERE j.kind = 'WORK_UPSERT' AND j.execution_profile = ANY($1) \
                AND j.status = 'PENDING' AND j.work_id IS NOT NULL \
                AND ((SELECT i.publisher_id FROM public.work w JOIN public.imprint i USING (imprint_id) \
                       WHERE w.work_id = j.work_id) IS DISTINCT FROM j.publisher_id \
                     OR EXISTS (SELECT 1 FROM public.distribution_job_target t \
                                 WHERE t.distribution_job_id = j.distribution_job_id \
                                   AND NOT EXISTS (SELECT 1 FROM public.publisher_distribution_platform p \
                                                    WHERE p.publisher_id = j.publisher_id \
                                                      AND p.platform = t.platform \
                                                      AND p.enabled AND p.activation_id = j.activation_id)))) \
         SELECT c.work_id, c.execution_profile, c.class, \
                COALESCE((SELECT k.capture_enabled FROM public.work_upsert_control k \
                           WHERE k.execution_profile = c.execution_profile), false) AS capture_enabled, \
                EXISTS (SELECT 1 FROM public.work w \
                          JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                          JOIN public.publisher_distribution_platform a \
                            ON a.publisher_id = i.publisher_id AND a.platform = c.execution_profile AND a.enabled \
                          JOIN public.work_upsert_admission ad \
                            ON ad.execution_profile = c.execution_profile AND ad.publisher_id = a.publisher_id \
                           AND ad.activation_id = a.activation_id \
                         WHERE w.work_id = c.work_id) AS admitted, \
                ($2 AND COALESCE((SELECT {coverage} AND {eligibility} \
                                   FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                                  WHERE w.work_id = c.work_id), false)) AS sql_eligible, \
                COALESCE((SELECT {abstracts} FROM public.work w WHERE w.work_id = c.work_id), ARRAY[]::text[]) AS abstracts \
           FROM c \
          ORDER BY c.work_id, c.execution_profile",
        coverage = policy::CROSSREF_PUBLISHER_COVERAGE_SQL.replace("{publisher}", "i.publisher_id"),
        eligibility = policy::CROSSREF_SQL_ELIGIBILITY,
        abstracts = policy::CROSSREF_EVALUATED_ABSTRACTS_SQL,
    ))
    .bind::<Array<crate::schema::sql_types::DistributionPlatform>, _>(keys)
    .bind::<diesel::sql_types::Bool, _>(eligible_route)
    .load::<CandidateRow>(connection)
}

/// The drainable set `D` over the requested profiles, in unit order: the
/// selector's one statement, in its own transaction, then one pure filter with
/// no further database read. It uses the same eligibility expression and
/// abstract function as the unit's step 10 (M15): `CROSSREF_SQL_ELIGIBILITY`,
/// `CROSSREF_PUBLISHER_COVERAGE_SQL`, `CROSSREF_EVALUATED_ABSTRACTS_SQL` and
/// `crossref_abstracts_normalise` through [`CandidateRow::drainable`].
fn drainable_set(
    db: &PgPool,
    profiles: &[&'static WorkLevelExecutionProfile],
) -> ThothResult<Vec<(uuid::Uuid, DistributionPlatform)>> {
    let rows = work_upsert_transaction(db, |connection| Ok(candidate_rows(connection, profiles)?))?;
    Ok(rows
        .into_iter()
        .filter(CandidateRow::drainable)
        .map(|row| (row.work_id, row.execution_profile))
        .collect())
}

/// `materializeWorkUpsertJobs` (Amendment 3 section 9.4): evaluate `D`, run
/// R52B section 9.3's unit for its first `limit` members, one transaction each,
/// then evaluate `D` again for `remainingCandidates`.
pub fn materialize_work_upsert_jobs(
    db: &PgPool,
    platforms: &[DistributionPlatform],
    limit: Option<i32>,
) -> ThothResult<super::MaterializeWorkUpsertJobsResult> {
    use super::WorkUpsertMaterializationOutcome as Outcome;

    let profiles = super::policy::registered_profiles(platforms)?;
    let limit = super::policy::clamp_limit(limit, 100, 500);
    let members = drainable_set(db, &profiles)?;
    let mut result = super::MaterializeWorkUpsertJobsResult::default();
    let mut ran = false;
    for (work_id, platform) in members
        .iter()
        .take(usize::try_from(limit).unwrap_or_default())
    {
        let profile = registered(*platform)?;
        let unit = work_upsert_transaction(db, |connection| {
            materialization_unit(connection, *work_id, profile, false)
        })?;
        ran = true;
        result.examined += 1;
        if unit.rebound {
            result.rebound += 1;
        }
        match unit.outcome {
            Outcome::Created => result.created += 1,
            Outcome::Resolved => result.skipped_resolved += 1,
            Outcome::Ineligible => result.skipped_ineligible += 1,
            Outcome::ResidueNotAdmitted => result.skipped_not_admitted += 1,
            Outcome::PendingCurrent
            | Outcome::RunningInFlight
            | Outcome::CaptureDisabled
            | Outcome::BindingMovedRetryLater
            | Outcome::NoWork => {}
        }
    }
    let remaining = if ran {
        drainable_set(db, &profiles)?.len()
    } else {
        members.len()
    };
    result.remaining_candidates = as_int(remaining)?;
    Ok(result)
}

/// Fence clause 6 (R52B section 11.1): every target of the job is enabled in
/// `publisher_distribution_platform` under exactly the job's activation.
pub(crate) fn job_targets_enabled(
    connection: &mut PgConnection,
    distribution_job_id: uuid::Uuid,
    publisher_id: uuid::Uuid,
    activation_id: uuid::Uuid,
) -> QueryResult<bool> {
    use crate::schema::{distribution_job_target, publisher_distribution_platform as assignment};
    let targets = distribution_job_target::table
        .filter(distribution_job_target::distribution_job_id.eq(distribution_job_id))
        .select(distribution_job_target::platform)
        .load::<DistributionPlatform>(connection)?;
    let enabled = assignment::table
        .filter(assignment::publisher_id.eq(publisher_id))
        .filter(assignment::activation_id.eq(activation_id))
        .filter(assignment::enabled.eq(true))
        .select(assignment::platform)
        .load::<DistributionPlatform>(connection)?;
    Ok(targets.iter().all(|target| enabled.contains(target)))
}

/// `work_upsert_clear_fenced_abandonment` (R52B section 11.7): the internal,
/// profile-owned clearance of a fenced abandonment, on an attempt row the caller
/// already holds `FOR UPDATE`. It has no GraphQL exposure and exactly one
/// caller, `reconcileCrossrefWritePermit`, which calls it only after its own
/// permit write. It clears only when the attempt is an uncleared fenced
/// abandonment and no permit of the attempt still blocks.
pub(crate) fn clear_fenced_abandonment(
    connection: &mut PgConnection,
    attempt_id: uuid::Uuid,
    reference: &str,
) -> QueryResult<bool> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
    diesel::sql_query(
        "UPDATE public.distribution_job_attempt a \
            SET recovery_cleared_at = clock_timestamp(), recovery_clearance_reference = $2 \
          WHERE a.distribution_job_attempt_id = $1 \
            AND a.result = 'ABANDONED' AND a.fenced_at IS NOT NULL AND a.recovery_cleared_at IS NULL \
            AND NOT EXISTS (SELECT 1 FROM public.crossref_write_permit p \
                             WHERE p.attempt_identity = a.distribution_job_attempt_id \
                               AND public.crossref_is_blocking_write_permit(p.state, p.reconciliation_state))",
    )
    .bind::<SqlUuid, _>(attempt_id)
    .bind::<Text, _>(reference)
    .execute(connection)
    .map(|cleared| cleared == 1)
}

// ---------------------------------------------------------------------------
// R52B section 20 and Amendment 3 section 4.4: the work-level reports
// (entries 1-8; 9 and 10 are above). Superuser-only at the resolver.
// ---------------------------------------------------------------------------

/// Report 1, `workUpsertResolution`: every generation row of the profile,
/// ascending by `work_id`, with `D`, `H` and the resolution of R52B section 9.1.
pub fn work_upsert_resolutions(
    db: &PgPool,
    platform: DistributionPlatform,
    limit: i32,
    offset: i32,
) -> ThothResult<Vec<super::WorkUpsertResolutionRow>> {
    use super::{WorkUpsertResolutionRow, WorkUpsertResolutionState as State};
    use diesel::sql_types::{BigInt, Bool, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = SqlUuid)]
        work_id: uuid::Uuid,
        #[diesel(sql_type = BigInt)]
        source_generation: i64,
        #[diesel(sql_type = BigInt)]
        success: i64,
        #[diesel(sql_type = BigInt)]
        terminal: i64,
        #[diesel(sql_type = Bool)]
        actionable: bool,
    }
    let profile = registered(platform)?;
    let rows = work_upsert_transaction(db, |connection| {
        Ok(diesel::sql_query(
            "WITH jobs AS ( \
                 SELECT j.work_id, j.status, j.cancellation_reason, \
                        COALESCE((SELECT max(a.claimed_generation) FROM public.distribution_job_attempt a \
                                   WHERE a.distribution_job_id = j.distribution_job_id), j.created_generation) AS gen \
                   FROM public.distribution_job j \
                  WHERE j.kind = 'WORK_UPSERT' AND j.execution_profile = $1 AND j.work_id IS NOT NULL) \
             SELECT g.work_id, g.source_generation, \
                    COALESCE((SELECT max(gen) FROM jobs WHERE jobs.work_id = g.work_id AND status = 'SUCCEEDED'), 0) AS success, \
                    COALESCE((SELECT max(gen) FROM jobs WHERE jobs.work_id = g.work_id \
                               AND (status = 'FAILED' OR (status = 'CANCELLED' AND cancellation_reason = 'ADMINISTRATIVE'))), 0) AS terminal, \
                    EXISTS (SELECT 1 FROM jobs WHERE jobs.work_id = g.work_id AND status IN ('PENDING', 'RUNNING')) AS actionable \
               FROM public.work_upsert_generation g \
              WHERE g.execution_profile = $1 \
              ORDER BY g.work_id LIMIT $2 OFFSET $3",
        )
        .bind::<crate::schema::sql_types::DistributionPlatform, _>(profile.key)
        .bind::<BigInt, _>(i64::from(limit.max(0)))
        .bind::<BigInt, _>(i64::from(offset.max(0)))
        .load::<Row>(connection)?)
    })?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let resolution = row.success.max(row.terminal);
            let outstanding = row.source_generation > resolution;
            let state = if row.actionable {
                State::Actionable
            } else if row.source_generation == 0 || outstanding {
                State::Residue
            } else {
                State::Resolved
            };
            WorkUpsertResolutionRow {
                work_id: row.work_id,
                execution_profile: profile.key,
                current_source_generation: row.source_generation,
                success_resolution_generation: row.success,
                terminal_job_resolution_generation: row.terminal,
                resolution_generation: resolution,
                outstanding_residue: outstanding.then_some((resolution, row.source_generation)),
                state,
            }
        })
        .collect())
}

/// Report 2, `workUpsertResolutionCount`.
pub fn work_upsert_resolution_count(
    db: &PgPool,
    platform: DistributionPlatform,
) -> ThothResult<i32> {
    use crate::schema::work_upsert_generation as generation;
    let profile = registered(platform)?;
    let count = work_upsert_transaction(db, |connection| {
        Ok(generation::table
            .filter(generation::execution_profile.eq(profile.key))
            .count()
            .get_result::<i64>(connection)?)
    })?;
    as_int(count)
}

/// Report 3, `workUpsertResidue`: residue candidates classified as the drain
/// classifies them, with the first failing eligibility clause of an ineligible
/// one, ascending by `work_id`.
///
/// One statement reads the page: the residue half of R52B section 9.2's
/// candidate set for the profile, ordered and paged in SQL, with everything the
/// classification needs projected per row: whether the current binding is
/// admitted, the drain's own SQL-eligibility expression, the publisher coverage
/// of E1-E3, each SQL clause of E4-E6 and the evaluated abstracts. The page is
/// then classified in Rust with no further read, through the expression and the
/// abstract function the drain uses (Amendment 3 section 9.4, M15).
pub fn work_upsert_residue(
    db: &PgPool,
    platform: DistributionPlatform,
    limit: i32,
    offset: i32,
) -> ThothResult<Vec<super::WorkUpsertResidueRow>> {
    use super::policy::{self, CROSSREF_SQL_CLAUSES};
    use super::{
        WorkUpsertEligibilityClause as Clause, WorkUpsertResidueClass as Class,
        WorkUpsertResidueRow,
    };
    use diesel::sql_types::{Array, BigInt, Bool, Text};
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        work_id: uuid::Uuid,
        #[diesel(sql_type = crate::schema::sql_types::DistributionPlatform)]
        execution_profile: DistributionPlatform,
        #[diesel(sql_type = Bool)]
        admitted: bool,
        #[diesel(sql_type = Bool)]
        sql_eligible: bool,
        #[diesel(sql_type = Bool)]
        covered: bool,
        #[diesel(sql_type = Array<Bool>)]
        clauses: Vec<bool>,
        #[diesel(sql_type = Array<Text>)]
        abstracts: Vec<String>,
    }
    let profile = registered(platform)?;
    let clauses = CROSSREF_SQL_CLAUSES
        .iter()
        .map(|(_, clause)| {
            format!(
                "COALESCE((SELECT {clause} FROM public.work w WHERE w.work_id = r.work_id), false)"
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let rows = work_upsert_transaction(db, |connection| {
        Ok(diesel::sql_query(format!(
            "WITH r AS ( \
                 SELECT g.work_id, g.execution_profile \
                   FROM public.work_upsert_generation g \
                  WHERE g.execution_profile = $1 \
                    AND NOT EXISTS (SELECT 1 FROM public.distribution_job j \
                                     WHERE j.kind = 'WORK_UPSERT' AND j.work_id = g.work_id \
                                       AND j.execution_profile = g.execution_profile \
                                       AND j.status IN ('PENDING', 'RUNNING')) \
                    AND (g.source_generation = 0 \
                         OR g.source_generation > public.work_upsert_resolution(g.work_id, g.execution_profile)) \
                  ORDER BY g.work_id, g.execution_profile \
                  LIMIT $3 OFFSET $4) \
             SELECT r.work_id, r.execution_profile, \
                    EXISTS (SELECT 1 FROM public.work w \
                              JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                              JOIN public.publisher_distribution_platform a \
                                ON a.publisher_id = i.publisher_id AND a.platform = r.execution_profile AND a.enabled \
                              JOIN public.work_upsert_admission ad \
                                ON ad.execution_profile = r.execution_profile AND ad.publisher_id = a.publisher_id \
                               AND ad.activation_id = a.activation_id \
                             WHERE w.work_id = r.work_id) AS admitted, \
                    ($2 AND COALESCE((SELECT {coverage} AND {eligibility} \
                                       FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                                      WHERE w.work_id = r.work_id), false)) AS sql_eligible, \
                    ($2 AND COALESCE((SELECT {coverage} \
                                       FROM public.work w JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                                      WHERE w.work_id = r.work_id), false)) AS covered, \
                    ARRAY[{clauses}] AS clauses, \
                    COALESCE((SELECT {abstracts} FROM public.work w WHERE w.work_id = r.work_id), ARRAY[]::text[]) AS abstracts \
               FROM r \
              ORDER BY r.work_id, r.execution_profile",
            coverage = policy::CROSSREF_PUBLISHER_COVERAGE_SQL.replace("{publisher}", "i.publisher_id"),
            eligibility = policy::CROSSREF_SQL_ELIGIBILITY,
            abstracts = policy::CROSSREF_EVALUATED_ABSTRACTS_SQL,
        ))
        .bind::<crate::schema::sql_types::DistributionPlatform, _>(profile.key)
        .bind::<Bool, _>(policy::crossref_route_is_automatic_push())
        .bind::<BigInt, _>(i64::from(limit.max(0)))
        .bind::<BigInt, _>(i64::from(offset.max(0)))
        .load::<Row>(connection)?)
    })?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let eligible = row.sql_eligible && policy::crossref_abstracts_normalise(&row.abstracts);
            let (class, failing_clause) = if !eligible {
                let failing = if !row.covered {
                    Clause::PublisherCoverage
                } else {
                    CROSSREF_SQL_CLAUSES
                        .iter()
                        .zip(&row.clauses)
                        .find(|(_, holds)| !**holds)
                        .map_or(Clause::AbstractNormalisation, |((clause, _), _)| *clause)
                };
                (Class::Ineligible, Some(failing))
            } else if row.admitted {
                (Class::Materializable, None)
            } else {
                (Class::NotAdmitted, None)
            };
            WorkUpsertResidueRow {
                work_id: row.work_id,
                execution_profile: row.execution_profile,
                class,
                failing_clause,
            }
        })
        .collect())
}

/// Report 4, `workUpsertStaleBindings`: `PENDING` jobs whose binding is obsolete,
/// with the cancellation reason the drain will use.
///
/// One statement reads the page: the stale-binding half of R52B section 9.2's
/// candidate set for the profile, ordered and paged in SQL, with the job's
/// binding, the Work's current publisher, whether every target of the job is
/// enabled under the job's activation and whether the profile's assignment is
/// enabled at all. The reason is then decided in Rust with no further read, by
/// the rule of the unit's step 7 (R52B section 9.4).
pub fn work_upsert_stale_bindings(
    db: &PgPool,
    platform: DistributionPlatform,
    limit: i32,
    offset: i32,
) -> ThothResult<Vec<super::WorkUpsertStaleBindingRow>> {
    use crate::model::distribution_job::DistributionJobCancellationReason as Reason;
    use diesel::sql_types::{BigInt, Bool, Nullable, Uuid as SqlUuid};
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = SqlUuid)]
        distribution_job_id: uuid::Uuid,
        #[diesel(sql_type = SqlUuid)]
        work_id: uuid::Uuid,
        #[diesel(sql_type = crate::schema::sql_types::DistributionPlatform)]
        execution_profile: DistributionPlatform,
        #[diesel(sql_type = SqlUuid)]
        job_publisher_id: uuid::Uuid,
        #[diesel(sql_type = Nullable<SqlUuid>)]
        current_publisher_id: Option<uuid::Uuid>,
        #[diesel(sql_type = Bool)]
        targets_current: bool,
        #[diesel(sql_type = Bool)]
        profile_enabled: bool,
    }
    let profile = registered(platform)?;
    let rows = work_upsert_transaction(db, |connection| {
        Ok(diesel::sql_query(
            "SELECT s.distribution_job_id, s.work_id, s.execution_profile, s.publisher_id AS job_publisher_id, \
                    s.current_publisher_id, \
                    NOT EXISTS (SELECT 1 FROM public.distribution_job_target t \
                                 WHERE t.distribution_job_id = s.distribution_job_id \
                                   AND NOT EXISTS (SELECT 1 FROM public.publisher_distribution_platform p \
                                                    WHERE p.publisher_id = s.publisher_id AND p.platform = t.platform \
                                                      AND p.enabled AND p.activation_id = s.activation_id)) AS targets_current, \
                    EXISTS (SELECT 1 FROM public.publisher_distribution_platform p \
                             WHERE p.publisher_id = s.publisher_id AND p.platform = $1 AND p.enabled) AS profile_enabled \
               FROM (SELECT j.distribution_job_id, j.work_id, j.execution_profile, j.publisher_id, j.activation_id, \
                            (SELECT i.publisher_id FROM public.work w JOIN public.imprint i USING (imprint_id) \
                              WHERE w.work_id = j.work_id) AS current_publisher_id \
                       FROM public.distribution_job j \
                      WHERE j.kind = 'WORK_UPSERT' AND j.execution_profile = $1 \
                        AND j.status = 'PENDING' AND j.work_id IS NOT NULL \
                        AND ((SELECT i.publisher_id FROM public.work w JOIN public.imprint i USING (imprint_id) \
                               WHERE w.work_id = j.work_id) IS DISTINCT FROM j.publisher_id \
                             OR EXISTS (SELECT 1 FROM public.distribution_job_target t \
                                         WHERE t.distribution_job_id = j.distribution_job_id \
                                           AND NOT EXISTS (SELECT 1 FROM public.publisher_distribution_platform p \
                                                            WHERE p.publisher_id = j.publisher_id AND p.platform = t.platform \
                                                              AND p.enabled AND p.activation_id = j.activation_id))) \
                      ORDER BY j.work_id, j.execution_profile \
                      LIMIT $2 OFFSET $3) s \
              ORDER BY s.work_id, s.execution_profile",
        )
        .bind::<crate::schema::sql_types::DistributionPlatform, _>(profile.key)
        .bind::<BigInt, _>(i64::from(limit.max(0)))
        .bind::<BigInt, _>(i64::from(offset.max(0)))
        .load::<Row>(connection)?)
    })?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let current = row.current_publisher_id?;
            let reason = if row.job_publisher_id != current {
                Reason::BindingSuperseded
            } else if row.targets_current {
                return None;
            } else if row.profile_enabled {
                Reason::BindingSuperseded
            } else {
                Reason::AssignmentDisabled
            };
            Some(super::WorkUpsertStaleBindingRow {
                distribution_job_id: row.distribution_job_id,
                work_id: row.work_id,
                execution_profile: row.execution_profile,
                reason,
            })
        })
        .collect())
}

/// The released job payload of `jobs`, preloaded with the released target and
/// attempt helpers (Amendment 3 section 10.3, EB1's enumerated exception).
fn load_job_payloads(
    connection: &mut PgConnection,
    jobs: Vec<crate::model::distribution_job::DistributionJob>,
) -> Result<Vec<crate::model::distribution_job::DistributionJobPayload>, WorkUpsertTxError> {
    use crate::model::distribution_job::crud::{
        attempts_for_jobs, partition_by_job, targets_for_jobs,
    };
    use crate::model::distribution_job::DistributionJobPayload;
    let ids: Vec<uuid::Uuid> = jobs.iter().map(|job| job.distribution_job_id).collect();
    let mut targets = partition_by_job(targets_for_jobs(connection, &ids)?, |target| {
        target.distribution_job_id
    });
    let mut attempts = partition_by_job(attempts_for_jobs(connection, &ids)?, |attempt| {
        attempt.distribution_job_id
    });
    Ok(jobs
        .into_iter()
        .map(|job| {
            let id = job.distribution_job_id;
            DistributionJobPayload::preloaded(
                job,
                targets.remove(&id).unwrap_or_default(),
                attempts.remove(&id).unwrap_or_default(),
            )
        })
        .collect())
}

/// Report 5, `workUpsertJobs`: the profile's `WORK_UPSERT` jobs with lineage,
/// ascending by `(work_identity, job_ordinal)`.
pub fn work_upsert_jobs(
    db: &PgPool,
    platform: DistributionPlatform,
    limit: i32,
    offset: i32,
) -> ThothResult<Vec<crate::model::distribution_job::DistributionJobPayload>> {
    use crate::model::distribution_job::{DistributionJob, DistributionJobKind};
    use crate::schema::distribution_job;
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        let jobs = distribution_job::table
            .filter(distribution_job::kind.eq(DistributionJobKind::WorkUpsert))
            .filter(distribution_job::execution_profile.eq(profile.key))
            .order((
                distribution_job::work_identity.asc(),
                distribution_job::job_ordinal.asc(),
            ))
            .limit(i64::from(limit.max(0)))
            .offset(i64::from(offset.max(0)))
            .load::<DistributionJob>(connection)?;
        load_job_payloads(connection, jobs)
    })
}

/// Report 5, `workUpsertJob`: every `WORK_UPSERT` job of a Work identity, live or
/// deleted, ascending by `job_ordinal`.
pub fn work_upsert_job(
    db: &PgPool,
    work_identity: uuid::Uuid,
) -> ThothResult<Vec<crate::model::distribution_job::DistributionJobPayload>> {
    use crate::model::distribution_job::{DistributionJob, DistributionJobKind};
    use crate::schema::distribution_job;
    work_upsert_transaction(db, |connection| {
        let jobs = distribution_job::table
            .filter(distribution_job::kind.eq(DistributionJobKind::WorkUpsert))
            .filter(distribution_job::work_identity.eq(work_identity))
            .order((
                distribution_job::execution_profile.asc(),
                distribution_job::job_ordinal.asc(),
            ))
            .load::<DistributionJob>(connection)?;
        load_job_payloads(connection, jobs)
    })
}

/// Report 6, `workUpsertAttempts`: every attempt of every `WORK_UPSERT` job of a
/// Work identity.
pub fn work_upsert_attempts(
    db: &PgPool,
    work_identity: uuid::Uuid,
) -> ThothResult<Vec<crate::model::distribution_job::DistributionJobAttempt>> {
    let payloads = work_upsert_job(db, work_identity)?;
    Ok(payloads
        .into_iter()
        .flat_map(|payload| payload.preloaded_attempts.unwrap_or_default())
        .collect())
}

/// Report 7, `workUpsertBlockedByRecovery`: every uncleared fenced abandonment.
pub fn work_upsert_blocked_by_recovery(
    db: &PgPool,
    platform: DistributionPlatform,
) -> ThothResult<Vec<super::WorkUpsertBlockedByRecoveryRow>> {
    use crate::schema::{distribution_job, distribution_job_attempt};
    let profile = registered(platform)?;
    let rows = work_upsert_transaction(db, |connection| {
        Ok(distribution_job::table
            .inner_join(distribution_job_attempt::table)
            .filter(
                distribution_job::kind
                    .eq(crate::model::distribution_job::DistributionJobKind::WorkUpsert),
            )
            .filter(distribution_job::execution_profile.eq(profile.key))
            .filter(
                distribution_job_attempt::result
                    .eq(crate::model::distribution_job::DistributionJobAttemptResult::Abandoned),
            )
            .filter(distribution_job_attempt::fenced_at.is_not_null())
            .filter(distribution_job_attempt::recovery_cleared_at.is_null())
            .select((
                distribution_job::work_identity,
                distribution_job::distribution_job_id,
                distribution_job_attempt::distribution_job_attempt_id,
            ))
            .order((
                distribution_job::work_identity.asc(),
                distribution_job_attempt::distribution_job_attempt_id.asc(),
            ))
            .load::<(Option<uuid::Uuid>, uuid::Uuid, uuid::Uuid)>(connection)?)
    })?;
    Ok(rows
        .into_iter()
        .filter_map(|(work_identity, job, attempt)| {
            work_identity.map(|work_identity| super::WorkUpsertBlockedByRecoveryRow {
                work_identity,
                execution_profile: profile.key,
                distribution_job_id: job,
                distribution_job_attempt_id: attempt,
            })
        })
        .collect())
}

/// Report 8, `workUpsertCaptureLag`: `max(source_generation - resolution)` over
/// rows whose source generation exceeds their resolution; `0` when none does.
pub fn work_upsert_capture_lag(db: &PgPool, platform: DistributionPlatform) -> ThothResult<i64> {
    use diesel::sql_types::BigInt;
    #[derive(QueryableByName)]
    struct Lag {
        #[diesel(sql_type = BigInt)]
        lag: i64,
    }
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        Ok(diesel::sql_query(
            "SELECT COALESCE(max(g.source_generation - public.work_upsert_resolution(g.work_id, g.execution_profile)) \
                     FILTER (WHERE g.source_generation > public.work_upsert_resolution(g.work_id, g.execution_profile)), 0)::bigint AS lag \
               FROM public.work_upsert_generation g WHERE g.execution_profile = $1",
        )
        .bind::<crate::schema::sql_types::DistributionPlatform, _>(profile.key)
        .get_result::<Lag>(connection)?
        .lag)
    })
}
