//! The protected `metric_platform_measure` administration coordinator
//! (`MET-WP1-12`).
//!
//! [`Crud`] is deliberately **not** implemented for the platform/measure
//! mapping registry: there is no generic create/update/delete surface for it,
//! no `all`/`count` listing and no delete at all. The two supported writes are
//! [`create_metric_platform_measure`] and [`update_metric_platform_measure`],
//! and they are the only places in the repository that commit a change to a
//! `metric_platform_measure` row.
//!
//! Neither function makes an authorization decision. Authorization is the
//! caller's responsibility and happens before any of this module is reached;
//! the caller supplies the already-authorized `actor` explicitly.
//!
//! **Lock topology.** Platform and measure codes are resolved with ordinary
//! non-locking reads. The single application-requested `FOR UPDATE` lock is
//! taken on the mapping row alone. The referenced `metric_platform` and
//! `metric_measure` rows are never locked by this module, and no joined
//! multi-table `FOR UPDATE` exists anywhere in it: locking the parents to
//! update a child is exactly the multi-row coordinator lock ordering that could
//! create a cycle against the platform and measure coordinators.
//!
//! [`Crud`]: crate::model::Crud

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

use super::{MetricPlatformMeasure, NewMetricPlatformMeasure, PatchMetricPlatformMeasure};
use crate::db::PgPool;
use crate::model::metric_measure::crud::by_code as measure_by_code;
use crate::model::metric_platform::crud::by_code as platform_by_code;
use crate::model::metric_registry_history::{
    record_create, record_update, MetricRegistryHistoryEntity,
};
use crate::schema::metric_platform_measure;

/// Resolve the `(platform_id, measure_id)` pair named by two exact codes.
///
/// Both reads are ordinary non-locking `SELECT`s using PostgreSQL `TEXT`
/// equality, with exactly the same semantics as the platform and measure
/// lookups themselves. An unknown code fails here, before the mapping is
/// touched, and surfaces as `EntityNotFound` rather than as a foreign-key
/// violation.
fn resolve_pair(
    connection: &mut PgConnection,
    platform_code: &str,
    measure_code: &str,
) -> ThothResult<(Uuid, Uuid)> {
    let platform = platform_by_code(connection, platform_code)?;
    let measure = measure_by_code(connection, measure_code)?;
    Ok((platform.platform_id, measure.measure_id))
}

/// Look one platform/measure mapping up by its exact stable code pair.
pub(crate) fn metric_platform_measure_by_codes(
    db: &PgPool,
    platform_code: &str,
    measure_code: &str,
) -> ThothResult<MetricPlatformMeasure> {
    let mut connection = db.get()?;
    let (platform_id, measure_id) = resolve_pair(&mut connection, platform_code, measure_code)?;
    metric_platform_measure::table
        .filter(metric_platform_measure::platform_id.eq(platform_id))
        .filter(metric_platform_measure::measure_id.eq(measure_id))
        .first::<MetricPlatformMeasure>(&mut connection)
        .map_err(Into::into)
}

/// Create one platform/measure mapping and its `CREATE` audit row atomically.
///
/// One transaction on one connection: resolve the two stable codes to their
/// persisted UUID foreign keys with ordinary reads, insert the canonical row
/// returning what PostgreSQL actually persisted, then append exactly one audit
/// row with `before_state = NULL`.
///
/// PostgreSQL remains the authority on pair uniqueness, foreign-key validity
/// and the `supported_grains` CHECK — non-empty, no NULL element, no duplicate
/// grain. No application-requested `FOR UPDATE` lock is taken, so a concurrent
/// duplicate pair loses at the unique index rather than at an application
/// pre-check.
pub(crate) fn create_metric_platform_measure(
    db: &PgPool,
    actor: &str,
    data: &NewMetricPlatformMeasure,
) -> ThothResult<MetricPlatformMeasure> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let (platform_id, measure_id) =
            resolve_pair(connection, &data.platform_code, &data.measure_code)?;

        let created: MetricPlatformMeasure = diesel::insert_into(metric_platform_measure::table)
            .values((
                metric_platform_measure::platform_id.eq(platform_id),
                metric_platform_measure::measure_id.eq(measure_id),
                metric_platform_measure::supported_grains.eq(&data.supported_grains),
                metric_platform_measure::supports_country.eq(data.supports_country),
                metric_platform_measure::supports_institution.eq(data.supports_institution),
                metric_platform_measure::supports_publication.eq(data.supports_publication),
                metric_platform_measure::direct_collection.eq(data.direct_collection),
                metric_platform_measure::enabled.eq(data.enabled),
            ))
            .returning(metric_platform_measure::all_columns)
            .get_result(connection)?;

        record_create(
            connection,
            MetricRegistryHistoryEntity::PlatformMeasure,
            created.platform_measure_id,
            actor,
            &created,
        )?;

        Ok(created)
    })
}

/// Replace one platform/measure mapping's mutable fields and audit the change
/// atomically.
///
/// One transaction on one connection, under `serialized last-write-wins`:
///
/// 1. resolve `platform_code` and `measure_code` through **ordinary
///    non-locking reads**;
/// 2. lock **exactly one** row — the `metric_platform_measure` mapping for that
///    pair — with `FOR UPDATE`. The platform and measure rows are deliberately
///    left unlocked;
/// 3. read the current persisted state under that lock;
/// 4. if the requested replacement equals the current mutable state, return the
///    current row unchanged and write no audit row. The table carries no
///    timestamp column, so there is no timestamp to move either way;
/// 5. otherwise update **only** `supported_grains`, `supports_country`,
///    `supports_institution`, `supports_publication`, `direct_collection` and
///    `enabled`. `platform_id` and `measure_id` are absent from the `SET`
///    clause, so the mapping's identity cannot move to another pair;
/// 6. append exactly one audit row carrying the exact before and after states.
///
/// `supported_grains` is compared and stored as an ordered array, so a
/// reordering of the same grains is a real change rather than a no-op: the
/// persisted order is part of the canonical row and the audit records it.
pub(crate) fn update_metric_platform_measure(
    db: &PgPool,
    actor: &str,
    data: &PatchMetricPlatformMeasure,
) -> ThothResult<MetricPlatformMeasure> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let (platform_id, measure_id) =
            resolve_pair(connection, &data.platform_code, &data.measure_code)?;

        let current: MetricPlatformMeasure = metric_platform_measure::table
            .filter(metric_platform_measure::platform_id.eq(platform_id))
            .filter(metric_platform_measure::measure_id.eq(measure_id))
            .for_update()
            .first::<MetricPlatformMeasure>(connection)?;

        // A genuine no-op: every approved mutable field already holds the
        // requested value.
        if current.supported_grains == data.supported_grains
            && current.supports_country == data.supports_country
            && current.supports_institution == data.supports_institution
            && current.supports_publication == data.supports_publication
            && current.direct_collection == data.direct_collection
            && current.enabled == data.enabled
        {
            return Ok(current);
        }

        let updated: MetricPlatformMeasure =
            diesel::update(metric_platform_measure::table.find(current.platform_measure_id))
                .set((
                    metric_platform_measure::supported_grains.eq(&data.supported_grains),
                    metric_platform_measure::supports_country.eq(data.supports_country),
                    metric_platform_measure::supports_institution.eq(data.supports_institution),
                    metric_platform_measure::supports_publication.eq(data.supports_publication),
                    metric_platform_measure::direct_collection.eq(data.direct_collection),
                    metric_platform_measure::enabled.eq(data.enabled),
                ))
                .returning(metric_platform_measure::all_columns)
                .get_result(connection)?;

        record_update(
            connection,
            MetricRegistryHistoryEntity::PlatformMeasure,
            updated.platform_measure_id,
            actor,
            &current,
            &updated,
        )?;

        Ok(updated)
    })
}
