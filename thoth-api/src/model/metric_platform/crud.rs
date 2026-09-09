//! The protected `metric_platform` administration coordinator (`MET-WP1-12`).
//!
//! [`Crud`] is deliberately **not** implemented for the metric platform
//! registry: there is no generic create/update/delete surface for it, no
//! `all`/`count` listing and no delete at all. The two supported writes are
//! [`create_metric_platform`] and [`update_metric_platform`], and they are the
//! only places in the repository that commit a change to a `metric_platform`
//! row.
//!
//! Neither function makes an authorization decision. Authorization is the
//! caller's responsibility and happens before any of this module is reached;
//! the caller supplies the already-authorized `actor` explicitly.
//!
//! [`Crud`]: crate::model::Crud

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;

use super::{MetricPlatform, NewMetricPlatform, PatchMetricPlatform};
use crate::db::PgPool;
use crate::model::metric_registry_history::{
    record_create, record_update, MetricRegistryHistoryEntity,
};
use crate::schema::metric_platform;

/// Look one metric platform up by its exact stable code.
///
/// The comparison is PostgreSQL `TEXT` equality on the `UNIQUE(code)` column:
/// no trimming, case folding, `ILIKE`, Unicode or whitespace normalization or
/// aliasing is applied to either side. A code that differs by case or by
/// surrounding whitespace is a different code and is not found.
///
/// This takes no row lock: it is an ordinary read used by the administrative
/// lookup and by mapping-code resolution.
pub(crate) fn metric_platform_by_code(db: &PgPool, code: &str) -> ThothResult<MetricPlatform> {
    let mut connection = db.get()?;
    by_code(&mut connection, code)
}

/// The same exact-equality read, on a caller-supplied connection.
pub(crate) fn by_code(connection: &mut PgConnection, code: &str) -> ThothResult<MetricPlatform> {
    metric_platform::table
        .filter(metric_platform::code.eq(code))
        .first::<MetricPlatform>(connection)
        .map_err(Into::into)
}

/// Create one metric platform and its `CREATE` audit row atomically.
///
/// One transaction on one connection:
///
/// 1. insert the canonical row, returning what PostgreSQL actually persisted,
///    including the generated `platform_id` and the database-authored
///    `created_at`/`updated_at`;
/// 2. append exactly one audit row with `before_state = NULL` and
///    `after_state` equal to that persisted row.
///
/// Any failure in either step rolls the whole transaction back, so a platform
/// can never exist without its audit evidence and vice versa. `code` is
/// persisted exactly as supplied; PostgreSQL's `UNIQUE(code)` and nonblank
/// CHECK constraints remain the authority on whether it is acceptable, and a
/// concurrent duplicate insert loses at the constraint rather than at an
/// application pre-check.
///
/// No application-requested `FOR UPDATE` lock is taken: creation is serialized
/// by PostgreSQL's own unique-index enforcement, not by hand-written locking.
pub(crate) fn create_metric_platform(
    db: &PgPool,
    actor: &str,
    data: &NewMetricPlatform,
) -> ThothResult<MetricPlatform> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let created: MetricPlatform = diesel::insert_into(metric_platform::table)
            .values((
                metric_platform::code.eq(&data.code),
                metric_platform::display_name.eq(&data.display_name),
                metric_platform::ownership_class.eq(data.ownership_class),
                metric_platform::enabled.eq(data.enabled),
                metric_platform::public_description.eq(&data.public_description),
            ))
            .returning(metric_platform::all_columns)
            .get_result(connection)?;

        record_create(
            connection,
            MetricRegistryHistoryEntity::Platform,
            created.platform_id,
            actor,
            &created,
        )?;

        Ok(created)
    })
}

/// Replace one metric platform's mutable fields and audit the change
/// atomically.
///
/// One transaction on one connection, under `serialized last-write-wins`:
///
/// 1. resolve and lock **exactly one** row — the `metric_platform` row named by
///    the exact `code` — with `FOR UPDATE`. This is the coordinator's only
///    application-requested lock; no related registry row is locked, and there
///    is no joined multi-table `FOR UPDATE`;
/// 2. read the current persisted state under that lock;
/// 3. if the requested replacement equals the current mutable state, return the
///    current row unchanged: no `UPDATE` statement runs, so the
///    `diesel_manage_updated_at` trigger does not fire, `updated_at` does not
///    move, and no audit row is written;
/// 4. otherwise update **only** `display_name`, `enabled` and
///    `public_description`. `code` and `ownership_class` are absent from the
///    `SET` clause and are not expressible in [`PatchMetricPlatform`];
/// 5. append exactly one audit row carrying the exact before and after states.
pub(crate) fn update_metric_platform(
    db: &PgPool,
    actor: &str,
    data: &PatchMetricPlatform,
) -> ThothResult<MetricPlatform> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let current: MetricPlatform = metric_platform::table
            .filter(metric_platform::code.eq(&data.code))
            .for_update()
            .first::<MetricPlatform>(connection)?;

        // A genuine no-op: every approved mutable field already holds the
        // requested value. Returning here is what keeps `updated_at` still and
        // the audit history free of entries that record no change.
        if current.display_name == data.display_name
            && current.enabled == data.enabled
            && current.public_description == data.public_description
        {
            return Ok(current);
        }

        let updated: MetricPlatform =
            diesel::update(metric_platform::table.find(current.platform_id))
                .set((
                    metric_platform::display_name.eq(&data.display_name),
                    metric_platform::enabled.eq(data.enabled),
                    metric_platform::public_description.eq(&data.public_description),
                ))
                .returning(metric_platform::all_columns)
                .get_result(connection)?;

        record_update(
            connection,
            MetricRegistryHistoryEntity::Platform,
            updated.platform_id,
            actor,
            &current,
            &updated,
        )?;

        Ok(updated)
    })
}
