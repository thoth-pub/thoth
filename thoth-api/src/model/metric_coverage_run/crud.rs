//! The one-account coverage-run rebuild (`MET-WP4-03C-B1`).
//!
//! `rebuildMetricCoverageRuns` is exceptional repair or initial-population
//! maintenance for one source account. It is **not** the incremental
//! producer: that is the `metric_import_maintain_coverage_runs` trigger
//! installed by migration `20261006_v1.9.0`, which recomputes the affected
//! hull of one import inside the terminalizing transaction. The rebuild
//! recomputes the whole account from the same raw authority with the same
//! approved semantics, serialized on the same account row lock, so the two
//! can never interleave: a terminalization waits behind a rebuild of its
//! account and a rebuild waits behind an in-flight terminalization.
//!
//! The recomputation here is the **boundary sweep** also used by the
//! trigger: every participating assertion's clipped start and end are the
//! boundaries of the stream's elementary segments, each segment's winner is
//! the first covering assertion under the approved ordering, and adjacent
//! segments with an equal visible tuple are coalesced. The independent
//! verifier in [`super::verification`] uses a per-day expansion instead;
//! the rebuild never consults its own output as an oracle, and commits only
//! when that independent verifier is exact.

use diesel::pg::PgConnection;
use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;

use super::verification::verify_source_account;
use super::{
    MaintenanceResult, MetricCoverageRunMaintenanceError, MetricCoverageRunRebuildResult,
    MAINTENANCE_LOCK_TIMEOUT_SQL, MAINTENANCE_STATEMENT_TIMEOUT_SQL,
};
use crate::db::PgPool;

/// The isolation level as the server reports it for this transaction.
pub(crate) const TRANSACTION_ISOLATION_SQL: &str =
    "SELECT current_setting('transaction_isolation') AS isolation";

#[derive(diesel::QueryableByName)]
struct IsolationRow {
    #[diesel(sql_type = Text)]
    isolation: String,
}

/// The canonical serialization row of one account's coverage runs, in the
/// frozen lock mode. Doubles as the existence check: no row means an
/// unknown source account.
pub(crate) const LOCK_SOURCE_ACCOUNT_SQL: &str = "SELECT sa.source_account_id \
       FROM public.metric_source_account sa \
      WHERE sa.source_account_id = $1 \
        FOR UPDATE";

#[derive(diesel::QueryableByName)]
struct LockedAccountRow {
    #[diesel(sql_type = SqlUuid)]
    #[allow(dead_code)]
    source_account_id: Uuid,
}

/// Remove every run of one account before its replacement. A plain
/// `DELETE`, not `TRUNCATE`: it is MVCC-safe, so a concurrent read-only
/// verification snapshot sees either the old or the new runs, never an
/// empty table.
pub(crate) const DELETE_ACCOUNT_RUNS_SQL: &str =
    "DELETE FROM public.metric_coverage_run WHERE source_account_id = $1";

/// Recompute and insert every run of one account from raw evidence alone.
///
/// The whole-account form of the trigger's hull recomputation: the same
/// participation predicate, historical stream provenance (coverage platform
/// and measure, import publisher), winner ordering, boundary sweep and
/// maximal coalescing, with no hull clipping and no preserved fragments
/// because every run of the account is being replaced. It reads
/// `metric_coverage` and `metric_import` only.
pub(crate) const REBUILD_ACCOUNT_RUNS_SQL: &str = "INSERT INTO public.metric_coverage_run ( \
         source_account_id, platform_id, publisher_id, measure_id, \
         run_start, run_end, \
         coverage_status, import_status, country_coverage, institution_coverage \
     ) \
     WITH evidence AS ( \
         SELECT c.platform_id, mi.publisher_id, c.measure_id, \
                c.period_start AS covered_start, c.period_end AS covered_end, \
                c.coverage_status, mi.status AS import_status, \
                c.country_coverage, c.institution_coverage, \
                mi.completed_at, mi.import_id, c.coverage_id \
           FROM public.metric_coverage c \
           JOIN public.metric_import mi \
             ON mi.import_id = c.import_id \
            AND mi.source_account_id = c.source_account_id \
          WHERE c.source_account_id = $1 \
            AND mi.status IN ( \
                    'COMPLETED'::public.metric_import_status, \
                    'COMPLETED_WITH_ERRORS'::public.metric_import_status \
                ) \
            AND mi.completed_at IS NOT NULL \
            AND mi.publisher_id IS NOT NULL \
     ), \
     boundaries AS ( \
         SELECT DISTINCT e.platform_id, e.publisher_id, e.measure_id, b.boundary \
           FROM evidence e \
          CROSS JOIN LATERAL (VALUES (e.covered_start), (e.covered_end)) AS b(boundary) \
     ), \
     segments AS ( \
         SELECT b.platform_id, b.publisher_id, b.measure_id, \
                b.boundary AS segment_start, \
                LEAD(b.boundary) OVER ( \
                    PARTITION BY b.platform_id, b.publisher_id, b.measure_id \
                    ORDER BY b.boundary \
                ) AS segment_end \
           FROM boundaries b \
     ), \
     winners AS ( \
         SELECT DISTINCT ON (s.platform_id, s.publisher_id, s.measure_id, s.segment_start) \
                s.platform_id, s.publisher_id, s.measure_id, \
                s.segment_start AS run_start, s.segment_end AS run_end, \
                e.coverage_status, e.import_status, \
                e.country_coverage, e.institution_coverage \
           FROM segments s \
           JOIN evidence e \
             ON e.platform_id = s.platform_id \
            AND e.publisher_id = s.publisher_id \
            AND e.measure_id = s.measure_id \
            AND e.covered_start <= s.segment_start \
            AND e.covered_end > s.segment_start \
          WHERE s.segment_end IS NOT NULL \
          ORDER BY s.platform_id, s.publisher_id, s.measure_id, s.segment_start, \
                   e.completed_at DESC, e.import_id DESC, \
                   e.coverage_status DESC, e.country_coverage ASC, \
                   e.institution_coverage ASC, e.coverage_id DESC \
     ), \
     marked AS ( \
         SELECT w.*, \
                CASE \
                    WHEN LAG(w.run_end) OVER stream = w.run_start \
                     AND LAG(w.coverage_status) OVER stream = w.coverage_status \
                     AND LAG(w.import_status) OVER stream = w.import_status \
                     AND LAG(w.country_coverage) OVER stream = w.country_coverage \
                     AND LAG(w.institution_coverage) OVER stream = w.institution_coverage \
                    THEN 0 \
                    ELSE 1 \
                END AS starts_run \
           FROM winners w \
         WINDOW stream AS ( \
             PARTITION BY w.platform_id, w.publisher_id, w.measure_id \
             ORDER BY w.run_start \
         ) \
     ), \
     numbered AS ( \
         SELECT m.*, \
                SUM(m.starts_run) OVER ( \
                    PARTITION BY m.platform_id, m.publisher_id, m.measure_id \
                    ORDER BY m.run_start \
                    ROWS UNBOUNDED PRECEDING \
                ) AS run_number \
           FROM marked m \
     ) \
     SELECT $1, n.platform_id, n.publisher_id, n.measure_id, \
            MIN(n.run_start), MAX(n.run_end), \
            n.coverage_status, n.import_status, \
            n.country_coverage, n.institution_coverage \
       FROM numbered n \
      GROUP BY n.platform_id, n.publisher_id, n.measure_id, n.run_number, \
               n.coverage_status, n.import_status, \
               n.country_coverage, n.institution_coverage";

/// Fail closed unless the current transaction is `READ COMMITTED`.
///
/// After the account lock is granted, every later statement must see the
/// state committed by the previous holder of that lock; only
/// `READ COMMITTED`, which takes a fresh snapshot per statement, guarantees
/// that. Under `REPEATABLE READ` or `SERIALIZABLE` the recomputation could
/// read a stale snapshot and commit an inconsistent replacement, so the
/// rebuild refuses before reading or writing any run.
pub(crate) fn require_read_committed(connection: &mut PgConnection) -> MaintenanceResult<()> {
    let rows: Vec<IsolationRow> = diesel::sql_query(TRANSACTION_ISOLATION_SQL).load(connection)?;
    match rows.into_iter().next() {
        Some(row) if row.isolation == "read committed" => Ok(()),
        other => {
            log::error!(
                "metric coverage-run rebuild refused: transaction isolation is {:?}, not read committed",
                other.map(|row| row.isolation)
            );
            Err(MetricCoverageRunMaintenanceError::Internal)
        }
    }
}

/// Lock one account's serialization row `FOR UPDATE`, classifying a missing
/// account as the invalid-query failure.
pub(crate) fn lock_source_account(
    connection: &mut PgConnection,
    source_account_id: Uuid,
) -> MaintenanceResult<()> {
    let rows: Vec<LockedAccountRow> = diesel::sql_query(LOCK_SOURCE_ACCOUNT_SQL)
        .bind::<SqlUuid, _>(source_account_id)
        .load(connection)?;
    if rows.is_empty() {
        return Err(MetricCoverageRunMaintenanceError::UnknownSourceAccount);
    }
    Ok(())
}

/// Replace every run of one account from raw evidence: delete, then insert
/// the recomputation. Returns the number of runs inserted.
pub(crate) fn replace_source_account_runs(
    connection: &mut PgConnection,
    source_account_id: Uuid,
) -> MaintenanceResult<usize> {
    diesel::sql_query(DELETE_ACCOUNT_RUNS_SQL)
        .bind::<SqlUuid, _>(source_account_id)
        .execute(connection)?;
    Ok(diesel::sql_query(REBUILD_ACCOUNT_RUNS_SQL)
        .bind::<SqlUuid, _>(source_account_id)
        .execute(connection)?)
}

/// Rebuild one source account's coverage runs, only if they are not already
/// exact, and commit only if the rebuilt state is independently verified
/// exact.
///
/// One transaction on one pooled connection performs, in order:
///
/// 1. `SET LOCAL lock_timeout = '5s'` and `SET LOCAL statement_timeout =
///    '30s'`;
/// 2. require `READ COMMITTED` ([`require_read_committed`]);
/// 3. lock `metric_source_account(source_account_id) FOR UPDATE`
///    ([`lock_source_account`]) — the same row every terminalization of
///    that account locks in its trigger, so nothing changes the account's
///    raw evidence or runs underneath the rebuild and every such
///    terminalization waits until it commits; a missing account is
///    `METRIC_QUERY_INVALID` and nothing further is read;
/// 4. verify the account independently ([`verify_source_account`]); if
///    exact, return `rebuilt = false` with that verification and **no**
///    `DELETE` or `INSERT` of any run;
/// 5. otherwise replace the account's runs from raw evidence
///    ([`replace_source_account_runs`]);
/// 6. verify independently again, in this same transaction, and fail —
///    rolling the whole rebuild back — unless it is exact;
/// 7. return `rebuilt = true` with that exact verification.
///
/// It is all or nothing: any rejection, isolation refusal, statement
/// failure, lock or statement timeout or inexact post-rebuild verification
/// rolls the complete transaction back, so no partially rebuilt account can
/// commit. The operation writes only `metric_coverage_run` rows of the one
/// account; it never reads another account's runs and never touches a
/// coverage, import, checkpoint or account row. There is no scheduled or
/// automatic invocation.
pub(crate) fn rebuild_metric_coverage_runs(
    db: &PgPool,
    source_account_id: Uuid,
) -> MaintenanceResult<MetricCoverageRunRebuildResult> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        diesel::sql_query(MAINTENANCE_LOCK_TIMEOUT_SQL).execute(connection)?;
        diesel::sql_query(MAINTENANCE_STATEMENT_TIMEOUT_SQL).execute(connection)?;
        require_read_committed(connection)?;
        lock_source_account(connection, source_account_id)?;

        let before = verify_source_account(connection, source_account_id)?;
        if before.exact {
            return Ok(MetricCoverageRunRebuildResult {
                source_account_id,
                rebuilt: false,
                verification: before,
            });
        }

        replace_source_account_runs(connection, source_account_id)?;

        let after = verify_source_account(connection, source_account_id)?;
        if !after.exact {
            log::error!(
                "metric coverage-run rebuild of source account {source_account_id} was rolled back: \
                 the rebuilt runs do not match their independently derived expected state"
            );
            return Err(MetricCoverageRunMaintenanceError::Internal);
        }
        Ok(MetricCoverageRunRebuildResult {
            source_account_id,
            rebuilt: true,
            verification: after,
        })
    })
}
