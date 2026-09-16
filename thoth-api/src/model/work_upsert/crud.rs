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
