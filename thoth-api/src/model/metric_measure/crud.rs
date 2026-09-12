//! The protected `metric_measure` administration coordinator (`MET-WP1-12`).
//!
//! [`Crud`] is deliberately **not** implemented for the metric measure
//! registry: there is no generic create/update/delete surface for it, no
//! generic `all`/`count` listing and no delete at all. `MET-WP4-02` adds one
//! bounded, unfiltered, read-only list, [`list_metric_measures`], for protected
//! service registry discovery. The two supported writes are
//! [`create_metric_measure`] and [`update_metric_measure`], and they are the
//! only places in the repository that commit a change to a `metric_measure`
//! row.
//!
//! Neither function makes an authorization decision. Authorization is the
//! caller's responsibility and happens before any of this module is reached;
//! the caller supplies the already-authorized `actor` explicitly.
//!
//! The migration-owned `title_sessions` and `net_units` seeds are ordinary
//! rows to this module: they are addressable by their stable codes and are
//! neither re-seeded nor rewritten by anything here.
//!
//! [`Crud`]: crate::model::Crud

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;

use super::{MetricMeasure, NewMetricMeasure, PatchMetricMeasure};
use crate::db::PgPool;
use crate::model::metric_dashboard::{MetricReadError, METRIC_REGISTRY_READ_MAX};
use crate::model::metric_registry_history::{
    record_create, record_update, MetricRegistryHistoryEntity,
};
use crate::schema::metric_measure;

/// Look one metric measure up by its exact stable code.
///
/// The comparison is PostgreSQL `TEXT` equality on the `UNIQUE(code)` column:
/// no trimming, case folding, `ILIKE`, Unicode or whitespace normalization or
/// aliasing is applied to either side.
///
/// This takes no row lock: it is an ordinary read used by the administrative
/// lookup and by mapping-code resolution.
pub(crate) fn metric_measure_by_code(db: &PgPool, code: &str) -> ThothResult<MetricMeasure> {
    let mut connection = db.get()?;
    by_code(&mut connection, code)
}

/// The same exact-equality read, on a caller-supplied connection.
pub(crate) fn by_code(connection: &mut PgConnection, code: &str) -> ThothResult<MetricMeasure> {
    metric_measure::table
        .filter(metric_measure::code.eq(code))
        .first::<MetricMeasure>(connection)
        .map_err(Into::into)
}

/// Create one metric measure and its `CREATE` audit row atomically.
///
/// One transaction on one connection: insert the canonical row returning what
/// PostgreSQL actually persisted, then append exactly one audit row with
/// `before_state = NULL` and that persisted row as `after_state`. Any failure
/// rolls the whole transaction back.
///
/// `code` is persisted exactly as supplied. PostgreSQL's `UNIQUE(code)` and the
/// nonblank `code`, `display_name` and `definition` CHECK constraints remain
/// the authority, so a concurrent duplicate insert loses at the constraint
/// rather than at an application pre-check.
///
/// No application-requested `FOR UPDATE` lock is taken.
pub(crate) fn create_metric_measure(
    db: &PgPool,
    actor: &str,
    data: &NewMetricMeasure,
) -> ThothResult<MetricMeasure> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let created: MetricMeasure = diesel::insert_into(metric_measure::table)
            .values((
                metric_measure::code.eq(&data.code),
                metric_measure::display_name.eq(&data.display_name),
                metric_measure::category.eq(data.category),
                metric_measure::unit.eq(data.unit),
                metric_measure::allow_negative.eq(data.allow_negative),
                metric_measure::public_visibility.eq(data.public_visibility),
                metric_measure::additive_across_time.eq(data.additive_across_time),
                metric_measure::additive_across_works.eq(data.additive_across_works),
                metric_measure::definition.eq(&data.definition),
                metric_measure::methodology_version.eq(&data.methodology_version),
                metric_measure::enabled.eq(data.enabled),
            ))
            .returning(metric_measure::all_columns)
            .get_result(connection)?;

        record_create(
            connection,
            MetricRegistryHistoryEntity::Measure,
            created.measure_id,
            actor,
            &created,
        )?;

        Ok(created)
    })
}

/// Replace one metric measure's mutable fields and audit the change
/// atomically.
///
/// One transaction on one connection, under `serialized last-write-wins`:
///
/// 1. resolve and lock **exactly one** row — the `metric_measure` row named by
///    the exact `code` — with `FOR UPDATE`. This is the coordinator's only
///    application-requested lock; no related registry row is locked, and there
///    is no joined multi-table `FOR UPDATE`;
/// 2. read the current persisted state under that lock;
/// 3. if the requested replacement equals the current mutable state, return the
///    current row unchanged: no `UPDATE` statement runs, `updated_at` does not
///    move, and no audit row is written;
/// 4. otherwise update **only** `display_name`, `public_visibility`,
///    `definition`, `methodology_version` and `enabled`. `code` and the
///    canonical semantic fields `category`, `unit`, `allow_negative`,
///    `additive_across_time` and `additive_across_works` are absent from the
///    `SET` clause and are not expressible in [`PatchMetricMeasure`];
/// 5. append exactly one audit row carrying the exact before and after states.
pub(crate) fn update_metric_measure(
    db: &PgPool,
    actor: &str,
    data: &PatchMetricMeasure,
) -> ThothResult<MetricMeasure> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let current: MetricMeasure = metric_measure::table
            .filter(metric_measure::code.eq(&data.code))
            .for_update()
            .first::<MetricMeasure>(connection)?;

        // A genuine no-op: every approved mutable field already holds the
        // requested value.
        if current.display_name == data.display_name
            && current.public_visibility == data.public_visibility
            && current.definition == data.definition
            && current.methodology_version == data.methodology_version
            && current.enabled == data.enabled
        {
            return Ok(current);
        }

        let updated: MetricMeasure = diesel::update(metric_measure::table.find(current.measure_id))
            .set((
                metric_measure::display_name.eq(&data.display_name),
                metric_measure::public_visibility.eq(data.public_visibility),
                metric_measure::definition.eq(&data.definition),
                metric_measure::methodology_version.eq(&data.methodology_version),
                metric_measure::enabled.eq(data.enabled),
            ))
            .returning(metric_measure::all_columns)
            .get_result(connection)?;

        record_update(
            connection,
            MetricRegistryHistoryEntity::Measure,
            updated.measure_id,
            actor,
            &current,
            &updated,
        )?;

        Ok(updated)
    })
}

/// Every metric measure, for protected service registry discovery
/// (`MET-WP4-02`).
///
/// This is the read behind `metricMeasures`, which lets a
/// `METRICS_READ_SERVICE` client learn the measure UUIDs a dashboard request
/// filters by without any out-of-band mapping. It makes no authorization
/// decision: the resolver authorizes before calling it.
///
/// Enabled and disabled rows are both returned, because disabling a registry
/// row retires it without deleting the canonical history that still refers
/// to its UUID.
///
/// The result is bounded to [`METRIC_REGISTRY_READ_MAX`] rows. One more than
/// that is requested, so an oversized registry is detected and refused rather
/// than silently truncated; when the registry is within the bound the query
/// has returned every row. Ordering is by exact `code`, compared byte-wise,
/// so it does not depend on the database collation. It takes no row lock and
/// touches no other table.
pub(crate) fn list_metric_measures(db: &PgPool) -> Result<Vec<MetricMeasure>, MetricReadError> {
    let mut connection = db.get()?;
    let mut rows: Vec<MetricMeasure> = metric_measure::table
        .limit(METRIC_REGISTRY_READ_MAX as i64 + 1)
        .load::<MetricMeasure>(&mut connection)?;
    if rows.len() > METRIC_REGISTRY_READ_MAX {
        return Err(MetricReadError::RegistryLimitExceeded);
    }
    rows.sort_by(|left, right| left.code.as_bytes().cmp(right.code.as_bytes()));
    Ok(rows)
}
