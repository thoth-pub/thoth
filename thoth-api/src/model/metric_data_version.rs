//! The Metrics data version (`METRIC-DATA-VERSION-01`, thoth#1071).
//!
//! `metricDataVersion` is a protected, read-only, constant-cost read of the
//! singleton rollup frontier. It lets a service that caches
//! `metricDashboard` answers learn that the data behind them may have changed
//! without paying for a dashboard read: the version changes when the work-day
//! frontier moves (rollups applied, whether from the Sphinx drain, the
//! quarantine reconciliation or an import), and not otherwise.
//!
//! It takes no input, reads no publisher, work or value, holds no lock and
//! needs no concurrent-read slot. It reads one row of
//! `metric_rollup_work_day_state` in its own statement. It is not a freshness
//! or coverage claim: coverage and import status are read live by the
//! dashboard, and the exceptional monthly rebuild does not move the frontier,
//! so a cache keyed by this version also needs a maximum age.

use diesel::sql_types::{BigInt as SqlBigInt, Timestamptz};
use diesel::RunQueryDsl;

use crate::db::PgPool;
use crate::model::metric_dashboard::MetricReadError;
use crate::model::Timestamp;

/// The rollup frontier, as an opaque version.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "The version of the Metrics data a dashboard read is served from: it changes when the rollup frontier moves"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDataVersion {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "An opaque string that is equal for two reads exactly when the rollup frontier is the same. Compare it for equality only"
        )
    )]
    pub version: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "When the applied work-day frontier was established")
    )]
    pub rollup_watermark: Timestamp,
}

#[derive(diesel::QueryableByName)]
struct FrontierRow {
    #[diesel(sql_type = SqlBigInt)]
    applied_through_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    watermark_at: Timestamp,
}

const FRONTIER_SQL: &str = "SELECT applied_through_sequence, watermark_at \
     FROM public.metric_rollup_work_day_state WHERE state_id = 1";

/// Reads the frontier. A missing singleton row, a pool failure or any
/// database error is `Unavailable`, with no detail.
pub(crate) fn read_metric_data_version(db: &PgPool) -> Result<MetricDataVersion, MetricReadError> {
    let mut connection = db.get().map_err(|_| MetricReadError::Unavailable)?;
    let row = diesel::sql_query(FRONTIER_SQL)
        .get_result::<FrontierRow>(&mut connection)
        .map_err(|_| MetricReadError::Unavailable)?;
    Ok(MetricDataVersion {
        version: format!(
            "{}:{}",
            row.applied_through_sequence,
            row.watermark_at.to_rfc3339()
        ),
        rollup_watermark: row.watermark_at,
    })
}
