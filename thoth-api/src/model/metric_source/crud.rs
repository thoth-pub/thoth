//! The protected `metric_source` administration coordinator (`MET-WP1-13`).
//!
//! [`Crud`] is deliberately **not** implemented for the metric source
//! registry: there is no generic create/update/delete surface for it, no
//! `all`/`count` listing and no delete at all. The two supported writes are
//! [`create_metric_source`] and [`update_metric_source`], and they are the only
//! places in the repository that commit a change to a `metric_source` row.
//!
//! Neither function makes an authorization decision. Authorization is the
//! caller's responsibility and happens before any of this module is reached;
//! the caller supplies the already-authorized `actor` explicitly.
//!
//! [`Crud`]: crate::model::Crud

use std::borrow::Cow;

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};

use super::{MetricSource, NewMetricSource, PatchMetricSource};
use crate::db::PgPool;
use crate::model::metric_source_registry_history::{
    record_create, record_update, MetricSourceRegistryHistoryEntity,
};
use crate::schema::metric_source;

/// The bounded message for a driver-key invariant violation.
///
/// It is the same string whether the coordinator's own pre-check or
/// PostgreSQL's `metric_source_driver_key_check` rejects the row, so the two
/// boundaries are indistinguishable to a client and neither exposes a
/// constraint name, SQL or driver text.
pub(crate) const DRIVER_KEY_INVARIANT_MESSAGE: &str =
    "A DRIVER metric source requires a non-blank driver key, and a non-DRIVER metric source must not carry one.";

/// Look one metric source up by its exact stable code.
///
/// The comparison is PostgreSQL `TEXT` equality on the `UNIQUE(code)` column:
/// no trimming, case folding, `ILIKE`, Unicode or whitespace normalization or
/// aliasing is applied to either side. A code that differs by case or by
/// surrounding whitespace is a different code and is not found.
///
/// This takes no row lock: it is an ordinary read used by the administrative
/// lookup and by source-account code resolution.
pub(crate) fn metric_source_by_code(db: &PgPool, code: &str) -> ThothResult<MetricSource> {
    let mut connection = db.get()?;
    by_code(&mut connection, code)
}

/// The same exact-equality read, on a caller-supplied connection.
pub(crate) fn by_code(connection: &mut PgConnection, code: &str) -> ThothResult<MetricSource> {
    metric_source::table
        .filter(metric_source::code.eq(code))
        .first::<MetricSource>(connection)
        .map_err(Into::into)
}

/// An ordinary non-locking read of one source by its persisted identifier.
///
/// Used by the source-account coordinator to enforce configuration
/// compatibility against the account's immutable source without locking it.
pub(crate) fn by_id(
    connection: &mut PgConnection,
    source_id: uuid::Uuid,
) -> ThothResult<MetricSource> {
    metric_source::table
        .find(source_id)
        .first::<MetricSource>(connection)
        .map_err(Into::into)
}

/// Enforce the driver-key invariant at the application boundary.
fn check_driver_key(data: &NewMetricSource) -> ThothResult<()> {
    if data
        .acquisition_type
        .accepts_driver_key(data.driver_key.as_deref())
    {
        Ok(())
    } else {
        Err(ThothError::DatabaseConstraintError(Cow::Borrowed(
            DRIVER_KEY_INVARIANT_MESSAGE,
        )))
    }
}

/// Create one metric source and its `CREATE` audit row atomically.
///
/// One transaction on one connection:
///
/// 1. reject a row that violates the driver-key invariant before touching the
///    database, with the same bounded message the CHECK constraint produces;
/// 2. insert the canonical row, returning what PostgreSQL actually persisted,
///    including the generated `source_id`;
/// 3. append exactly one audit row with `before_state = NULL` and
///    `after_state` equal to that persisted row.
///
/// Any failure in either write rolls the whole transaction back. `code` and
/// `driver_key` are persisted exactly as supplied; PostgreSQL's `UNIQUE(code)`,
/// nonblank, non-negative-day and driver-key CHECK constraints remain the
/// authority, and a concurrent duplicate insert loses at the constraint rather
/// than at an application pre-check.
///
/// No application-requested `FOR UPDATE` lock is taken.
pub(crate) fn create_metric_source(
    db: &PgPool,
    actor: &str,
    data: &NewMetricSource,
) -> ThothResult<MetricSource> {
    check_driver_key(data)?;
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let created: MetricSource = diesel::insert_into(metric_source::table)
            .values((
                metric_source::code.eq(&data.code),
                metric_source::acquisition_type.eq(data.acquisition_type),
                metric_source::driver_key.eq(&data.driver_key),
                metric_source::enabled.eq(data.enabled),
                metric_source::default_lookback_days.eq(data.default_lookback_days),
                metric_source::default_finalization_delay_days
                    .eq(data.default_finalization_delay_days),
            ))
            .returning(metric_source::all_columns)
            .get_result(connection)?;

        record_create(
            connection,
            MetricSourceRegistryHistoryEntity::Source,
            created.source_id,
            actor,
            &created,
        )?;

        Ok(created)
    })
}

/// Replace one metric source's mutable fields and audit the change atomically.
///
/// One transaction on one connection, under `serialized last-write-wins`:
///
/// 1. resolve and lock **exactly one** row — the `metric_source` row named by
///    the exact `code` — with `FOR UPDATE`. This is the coordinator's only
///    application-requested lock; no other row is locked and there is no
///    joined multi-table `FOR UPDATE`;
/// 2. read the current persisted state under that lock;
/// 3. if the requested replacement equals the current mutable state, return the
///    current row unchanged: no `UPDATE` statement runs and no audit row is
///    written;
/// 4. otherwise update **only** `enabled`, `default_lookback_days` and
///    `default_finalization_delay_days`. `code`, `acquisition_type` and
///    `driver_key` are absent from the `SET` clause and are not expressible in
///    [`PatchMetricSource`];
/// 5. append exactly one audit row carrying the exact before and after states.
pub(crate) fn update_metric_source(
    db: &PgPool,
    actor: &str,
    data: &PatchMetricSource,
) -> ThothResult<MetricSource> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let current: MetricSource = metric_source::table
            .filter(metric_source::code.eq(&data.code))
            .for_update()
            .first::<MetricSource>(connection)?;

        if current.enabled == data.enabled
            && current.default_lookback_days == data.default_lookback_days
            && current.default_finalization_delay_days == data.default_finalization_delay_days
        {
            return Ok(current);
        }

        let updated: MetricSource = diesel::update(metric_source::table.find(current.source_id))
            .set((
                metric_source::enabled.eq(data.enabled),
                metric_source::default_lookback_days.eq(data.default_lookback_days),
                metric_source::default_finalization_delay_days
                    .eq(data.default_finalization_delay_days),
            ))
            .returning(metric_source::all_columns)
            .get_result(connection)?;

        record_update(
            connection,
            MetricSourceRegistryHistoryEntity::Source,
            updated.source_id,
            actor,
            &current,
            &updated,
        )?;

        Ok(updated)
    })
}
