//! Independent verification of the derived coverage runs against raw
//! evidence (`MET-WP4-03C-B1`).
//!
//! This module is deliberately **structurally independent of the producer**.
//! The trigger installed by migration `20261006_v1.9.0` and the rebuild in
//! [`super::crud`] derive runs by a boundary sweep over clipped assertion
//! ranges; the expected state here is derived by **per-day expansion**: every
//! participating assertion is expanded over the days it covers, the winner
//! of each stream and day is chosen under the approved ordering — exactly
//! the formulation the current dashboard assertion path uses — and
//! consecutive days with an equal visible tuple are grouped into runs. It
//! shares no SQL text with the producer, never executes a producer
//! statement, never calls the rebuild, and never proves correctness by
//! rebuilding and then reading the rebuild. Two disagreeing implementations
//! of the same approved semantics are what make an exact match evidence.
//!
//! Everything is set-based PostgreSQL over the accounts of one page: the
//! expected runs are derived, the actual rows are read, the two sides are
//! compared by primary key and the four structural predicates are counted,
//! all inside one statement that returns only bounded per-account counts.
//! No run and no raw evidence row is loaded into Rust memory, and no row
//! identity, value or mismatch detail leaves the database except as a count.
//!
//! The callable page operation, [`verify_metric_coverage_runs`], runs on one
//! pooled connection in one `READ ONLY`, `REPEATABLE READ` transaction with
//! no row lock, so the complete source-account domain, the page's raw
//! evidence and the page's actual runs are one coherent snapshot. The
//! single-account form, [`verify_source_account`], is also what the rebuild
//! calls before and after replacing an account, under its own
//! `READ COMMITTED` transaction and account lock.

use diesel::pg::PgConnection;
use diesel::sql_types::{Array, BigInt, Text, Uuid as SqlUuid};
use diesel::RunQueryDsl;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    MaintenanceResult, MetricCoverageRunAccountVerification, MetricCoverageRunMaintenanceError,
    MetricCoverageRunVerificationPage, MAINTENANCE_LOCK_TIMEOUT_SQL,
    MAINTENANCE_STATEMENT_TIMEOUT_SQL, METRIC_COVERAGE_RUN_DOMAIN_FINGERPRINT_SCHEMA,
    METRIC_COVERAGE_RUN_VERIFY_DEFAULT_LIMIT, METRIC_COVERAGE_RUN_VERIFY_MAX_LIMIT,
    METRIC_COVERAGE_RUN_VERIFY_MIN_LIMIT,
};
use crate::db::PgPool;

// ---------------------------------------------------------------------------
// Domain
// ---------------------------------------------------------------------------

/// Every source account, in exact ascending UUID order. PostgreSQL orders
/// `uuid` bytewise, as `uuid::Uuid`'s `Ord` does, so the server order and
/// the page boundary comparison below agree.
pub(crate) const DOMAIN_SQL: &str =
    "SELECT source_account_id FROM public.metric_source_account ORDER BY source_account_id";

#[derive(diesel::QueryableByName)]
struct DomainRow {
    #[diesel(sql_type = SqlUuid)]
    source_account_id: Uuid,
}

/// Read the complete source-account domain in the current snapshot.
pub(crate) fn read_domain(connection: &mut PgConnection) -> MaintenanceResult<Vec<Uuid>> {
    let rows: Vec<DomainRow> = diesel::sql_query(DOMAIN_SQL).load(connection)?;
    Ok(rows.into_iter().map(|row| row.source_account_id).collect())
}

/// The byte-exact Amendment 5A domain fingerprint of an ascending UUID list:
/// lowercase hexadecimal SHA-256 of the schema line, one LF, and every UUID
/// as canonical lowercase hyphenated text followed by one LF. An empty
/// domain hashes the schema line and its LF alone.
pub(crate) fn domain_fingerprint(ascending_source_account_ids: &[Uuid]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(METRIC_COVERAGE_RUN_DOMAIN_FINGERPRINT_SCHEMA.as_bytes());
    hasher.update(b"\n");
    for id in ascending_source_account_ids {
        hasher.update(id.hyphenated().to_string().as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// Per-account comparison
// ---------------------------------------------------------------------------

/// The bounded verification counts of every account in `$1`, one row per
/// account in ascending order, including accounts with no evidence and no
/// run.
///
/// `evidence` applies the raw participation predicate and the historical
/// stream provenance (coverage platform and measure, **import** publisher).
/// `winning_days` expands each assertion over the days it covers and keeps
/// the first assertion per stream and day under the approved ordering.
/// `expected` groups consecutive days with an equal visible tuple into runs.
/// `compared` is a full outer join of expected and actual rows on the exact
/// primary key. `structural` evaluates V1–V4 over the actual rows of each
/// stream ordered by `run_start, run_end`; V2 resolves the four canonical
/// references only, never current eligibility.
pub(crate) const ACCOUNT_VERIFICATION_SQL: &str = "WITH accounts AS ( \
         SELECT a.source_account_id FROM unnest($1::uuid[]) AS a(source_account_id) \
     ), \
     evidence AS ( \
         SELECT c.source_account_id, c.platform_id, mi.publisher_id, c.measure_id, \
                c.period_start, c.period_end, \
                c.coverage_status, mi.status AS import_status, \
                c.country_coverage, c.institution_coverage, \
                mi.completed_at, mi.import_id, c.coverage_id \
           FROM public.metric_coverage c \
           JOIN public.metric_import mi \
             ON mi.import_id = c.import_id \
            AND mi.source_account_id = c.source_account_id \
          WHERE c.source_account_id = ANY($1) \
            AND mi.status IN ( \
                    'COMPLETED'::public.metric_import_status, \
                    'COMPLETED_WITH_ERRORS'::public.metric_import_status \
                ) \
            AND mi.completed_at IS NOT NULL \
            AND mi.publisher_id IS NOT NULL \
     ), \
     winning_days AS ( \
         SELECT DISTINCT ON (e.source_account_id, e.platform_id, e.publisher_id, \
                             e.measure_id, covered.day) \
                e.source_account_id, e.platform_id, e.publisher_id, e.measure_id, \
                covered.day::date AS day, \
                e.coverage_status, e.import_status, \
                e.country_coverage, e.institution_coverage \
           FROM evidence e \
          CROSS JOIN LATERAL generate_series( \
                    e.period_start, e.period_end - 1, interval '1 day' \
                ) AS covered(day) \
          ORDER BY e.source_account_id, e.platform_id, e.publisher_id, e.measure_id, \
                   covered.day, \
                   e.completed_at DESC, e.import_id DESC, \
                   e.coverage_status DESC, e.country_coverage ASC, \
                   e.institution_coverage ASC, e.coverage_id DESC \
     ), \
     marked_days AS ( \
         SELECT d.*, \
                CASE \
                    WHEN LAG(d.day) OVER stream = d.day - 1 \
                     AND LAG(d.coverage_status) OVER stream = d.coverage_status \
                     AND LAG(d.import_status) OVER stream = d.import_status \
                     AND LAG(d.country_coverage) OVER stream = d.country_coverage \
                     AND LAG(d.institution_coverage) OVER stream = d.institution_coverage \
                    THEN 0 \
                    ELSE 1 \
                END AS starts_run \
           FROM winning_days d \
         WINDOW stream AS ( \
             PARTITION BY d.source_account_id, d.platform_id, d.publisher_id, d.measure_id \
             ORDER BY d.day \
         ) \
     ), \
     numbered_days AS ( \
         SELECT m.*, \
                SUM(m.starts_run) OVER ( \
                    PARTITION BY m.source_account_id, m.platform_id, m.publisher_id, m.measure_id \
                    ORDER BY m.day \
                    ROWS UNBOUNDED PRECEDING \
                ) AS run_number \
           FROM marked_days m \
     ), \
     expected AS ( \
         SELECT n.source_account_id, n.platform_id, n.publisher_id, n.measure_id, \
                MIN(n.day) AS run_start, MAX(n.day) + 1 AS run_end, \
                n.coverage_status, n.import_status, \
                n.country_coverage, n.institution_coverage \
           FROM numbered_days n \
          GROUP BY n.source_account_id, n.platform_id, n.publisher_id, n.measure_id, \
                   n.run_number, n.coverage_status, n.import_status, \
                   n.country_coverage, n.institution_coverage \
     ), \
     actual AS ( \
         SELECT r.* FROM public.metric_coverage_run r WHERE r.source_account_id = ANY($1) \
     ), \
     compared AS ( \
         SELECT COALESCE(e.source_account_id, a.source_account_id) AS source_account_id, \
                (e.run_start IS NOT NULL) AS expected_present, \
                (a.run_start IS NOT NULL) AS actual_present, \
                (e.run_start IS NOT NULL AND a.run_start IS NOT NULL \
                 AND (e.run_end <> a.run_end \
                      OR e.coverage_status <> a.coverage_status \
                      OR e.import_status <> a.import_status \
                      OR e.country_coverage <> a.country_coverage \
                      OR e.institution_coverage <> a.institution_coverage)) AS mismatched \
           FROM expected e \
           FULL OUTER JOIN actual a \
             ON a.source_account_id = e.source_account_id \
            AND a.platform_id = e.platform_id \
            AND a.publisher_id = e.publisher_id \
            AND a.measure_id = e.measure_id \
            AND a.run_start = e.run_start \
     ), \
     comparison_counts AS ( \
         SELECT c.source_account_id, \
                COUNT(*) FILTER (WHERE c.expected_present) AS expected_rows, \
                COUNT(*) FILTER (WHERE c.actual_present) AS actual_rows, \
                COUNT(*) FILTER (WHERE c.expected_present AND NOT c.actual_present) AS missing_rows, \
                COUNT(*) FILTER (WHERE c.actual_present AND NOT c.expected_present) AS extra_rows, \
                COUNT(*) FILTER (WHERE c.mismatched) AS mismatched_rows \
           FROM compared c \
          GROUP BY c.source_account_id \
     ), \
     structural AS ( \
         SELECT a.source_account_id, \
                (a.run_end <= a.run_start)::int AS v1, \
                (sa.source_account_id IS NULL \
                 OR p.platform_id IS NULL \
                 OR pub.publisher_id IS NULL \
                 OR m.measure_id IS NULL)::int AS v2, \
                COALESCE((LAG(a.run_end) OVER stream > a.run_start)::int, 0) AS v3, \
                COALESCE((LAG(a.run_end) OVER stream = a.run_start \
                          AND LAG(a.coverage_status) OVER stream = a.coverage_status \
                          AND LAG(a.import_status) OVER stream = a.import_status \
                          AND LAG(a.country_coverage) OVER stream = a.country_coverage \
                          AND LAG(a.institution_coverage) OVER stream = a.institution_coverage)::int, \
                         0) AS v4 \
           FROM actual a \
           LEFT JOIN public.metric_source_account sa \
             ON sa.source_account_id = a.source_account_id \
           LEFT JOIN public.metric_platform p ON p.platform_id = a.platform_id \
           LEFT JOIN public.publisher pub ON pub.publisher_id = a.publisher_id \
           LEFT JOIN public.metric_measure m ON m.measure_id = a.measure_id \
         WINDOW stream AS ( \
             PARTITION BY a.source_account_id, a.platform_id, a.publisher_id, a.measure_id \
             ORDER BY a.run_start, a.run_end \
         ) \
     ), \
     structural_counts AS ( \
         SELECT s.source_account_id, SUM(s.v1 + s.v2 + s.v3 + s.v4)::bigint AS structural_violations \
           FROM structural s \
          GROUP BY s.source_account_id \
     ) \
     SELECT acc.source_account_id, \
            COALESCE(cc.expected_rows, 0)::bigint AS expected_rows, \
            COALESCE(cc.actual_rows, 0)::bigint AS actual_rows, \
            COALESCE(cc.missing_rows, 0)::bigint AS missing_rows, \
            COALESCE(cc.extra_rows, 0)::bigint AS extra_rows, \
            COALESCE(cc.mismatched_rows, 0)::bigint AS mismatched_rows, \
            COALESCE(sc.structural_violations, 0)::bigint AS structural_violations \
       FROM accounts acc \
       LEFT JOIN comparison_counts cc ON cc.source_account_id = acc.source_account_id \
       LEFT JOIN structural_counts sc ON sc.source_account_id = acc.source_account_id \
      ORDER BY acc.source_account_id";

#[derive(diesel::QueryableByName)]
struct AccountCountsRow {
    #[diesel(sql_type = SqlUuid)]
    source_account_id: Uuid,
    #[diesel(sql_type = BigInt)]
    expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    mismatched_rows: i64,
    #[diesel(sql_type = BigInt)]
    structural_violations: i64,
}

/// A count as a GraphQL `Int`, failing closed instead of wrapping,
/// saturating or truncating when it does not fit.
pub(crate) fn bounded_count(count: i64, what: &str) -> MaintenanceResult<i32> {
    i32::try_from(count).map_err(|_| {
        log::error!(
            "metric coverage-run verification count {what} = {count} does not fit a GraphQL Int"
        );
        MetricCoverageRunMaintenanceError::Internal
    })
}

impl TryFrom<AccountCountsRow> for MetricCoverageRunAccountVerification {
    type Error = MetricCoverageRunMaintenanceError;

    fn try_from(row: AccountCountsRow) -> MaintenanceResult<Self> {
        let missing_rows = bounded_count(row.missing_rows, "missingRows")?;
        let extra_rows = bounded_count(row.extra_rows, "extraRows")?;
        let mismatched_rows = bounded_count(row.mismatched_rows, "mismatchedRows")?;
        let structural_violations =
            bounded_count(row.structural_violations, "structuralViolations")?;
        Ok(Self {
            source_account_id: row.source_account_id,
            expected_rows: bounded_count(row.expected_rows, "expectedRows")?,
            actual_rows: bounded_count(row.actual_rows, "actualRows")?,
            missing_rows,
            extra_rows,
            mismatched_rows,
            structural_violations,
            exact: missing_rows == 0
                && extra_rows == 0
                && mismatched_rows == 0
                && structural_violations == 0,
        })
    }
}

/// Verify the given accounts in the current snapshot, returning one result
/// per requested account in ascending order. An empty request issues no
/// statement.
pub(crate) fn verify_source_accounts(
    connection: &mut PgConnection,
    source_account_ids: &[Uuid],
) -> MaintenanceResult<Vec<MetricCoverageRunAccountVerification>> {
    if source_account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<AccountCountsRow> = diesel::sql_query(ACCOUNT_VERIFICATION_SQL)
        .bind::<Array<SqlUuid>, _>(source_account_ids)
        .load(connection)?;
    if rows.len() != source_account_ids.len() {
        log::error!(
            "metric coverage-run verification returned {} rows for {} accounts",
            rows.len(),
            source_account_ids.len()
        );
        return Err(MetricCoverageRunMaintenanceError::Internal);
    }
    rows.into_iter()
        .map(MetricCoverageRunAccountVerification::try_from)
        .collect()
}

/// Verify one account in the current transaction. Used by the rebuild
/// before and after replacing an account's runs; it reads the account's raw
/// evidence and actual runs and writes nothing.
pub(crate) fn verify_source_account(
    connection: &mut PgConnection,
    source_account_id: Uuid,
) -> MaintenanceResult<MetricCoverageRunAccountVerification> {
    verify_source_accounts(connection, &[source_account_id])?
        .into_iter()
        .next()
        .ok_or_else(|| {
            log::error!("metric coverage-run verification returned no row for {source_account_id}");
            MetricCoverageRunMaintenanceError::Internal
        })
}

// ---------------------------------------------------------------------------
// verifyMetricCoverageRuns
// ---------------------------------------------------------------------------

/// The transaction mode the verification asserts before reading anything.
pub(crate) const TRANSACTION_MODE_SQL: &str =
    "SELECT current_setting('transaction_read_only') AS read_only, \
            current_setting('transaction_isolation') AS isolation";

#[derive(diesel::QueryableByName)]
struct TransactionModeRow {
    #[diesel(sql_type = Text)]
    read_only: String,
    #[diesel(sql_type = Text)]
    isolation: String,
}

/// Fail closed unless the current transaction is read-only at repeatable
/// read.
fn require_read_only_repeatable_read(connection: &mut PgConnection) -> MaintenanceResult<()> {
    let rows: Vec<TransactionModeRow> = diesel::sql_query(TRANSACTION_MODE_SQL).load(connection)?;
    match rows.into_iter().next() {
        Some(mode) if mode.read_only == "on" && mode.isolation == "repeatable read" => Ok(()),
        other => {
            log::error!(
                "metric coverage-run verification refused: transaction mode is {:?}",
                other.map(|mode| (mode.read_only, mode.isolation))
            );
            Err(MetricCoverageRunMaintenanceError::Internal)
        }
    }
}

/// The effective page size: the default when none is supplied, rejected —
/// not clamped — outside `1..=10`. Decided before a connection is taken.
pub(crate) fn effective_limit(limit: Option<i32>) -> MaintenanceResult<usize> {
    let limit = limit.unwrap_or(METRIC_COVERAGE_RUN_VERIFY_DEFAULT_LIMIT);
    if !(METRIC_COVERAGE_RUN_VERIFY_MIN_LIMIT..=METRIC_COVERAGE_RUN_VERIFY_MAX_LIMIT)
        .contains(&limit)
    {
        return Err(MetricCoverageRunMaintenanceError::VerifyLimitOutOfRange);
    }
    usize::try_from(limit).map_err(|_| MetricCoverageRunMaintenanceError::VerifyLimitOutOfRange)
}

/// Verify one keyset page of source accounts in one read-only snapshot.
///
/// One pooled connection, one transaction opened `READ ONLY` at
/// `REPEATABLE READ`, no `FOR UPDATE` and no database mutation of any kind:
/// the server refuses every write in a read-only transaction, and the
/// operation additionally asserts the mode it was granted before it reads
/// anything, so a misconfigured connection fails closed rather than
/// silently verifying under `READ COMMITTED`. The statement order is fixed:
/// the two local timeouts, the transaction-mode assertion (which freezes
/// the snapshot), the complete source-account domain, then the page's
/// comparison.
///
/// `after_source_account_id` is an exclusive ordering boundary that need not
/// name an existing account; `None` starts at the beginning of the domain.
/// The page is the next at most `limit` accounts strictly after it in
/// ascending UUID order, with no `OFFSET`. `next_after_source_account_id` is
/// the last returned account when at least one further account exists, and
/// `None` when the page exhausts the domain or is empty. The domain count
/// and fingerprint describe the complete domain of this snapshot.
pub(crate) fn verify_metric_coverage_runs(
    db: &PgPool,
    after_source_account_id: Option<Uuid>,
    limit: Option<i32>,
) -> MaintenanceResult<MetricCoverageRunVerificationPage> {
    let limit = effective_limit(limit)?;

    let mut connection = db.get()?;
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            diesel::sql_query(MAINTENANCE_LOCK_TIMEOUT_SQL).execute(connection)?;
            diesel::sql_query(MAINTENANCE_STATEMENT_TIMEOUT_SQL).execute(connection)?;
            require_read_only_repeatable_read(connection)?;

            let domain = read_domain(connection)?;
            let domain_account_count = bounded_count(
                i64::try_from(domain.len()).map_err(|_| {
                    log::error!(
                        "metric coverage-run domain of {} accounts cannot be counted",
                        domain.len()
                    );
                    MetricCoverageRunMaintenanceError::Internal
                })?,
                "domainAccountCount",
            )?;
            let domain_fingerprint = domain_fingerprint(&domain);

            let first_index = after_source_account_id.map_or(0, |after| {
                domain.iter().take_while(|id| **id <= after).count()
            });
            let page: &[Uuid] = &domain[first_index..domain.len().min(first_index + limit)];
            let next_after_source_account_id = if first_index + page.len() < domain.len() {
                page.last().copied()
            } else {
                None
            };

            let accounts = verify_source_accounts(connection, page)?;
            Ok(MetricCoverageRunVerificationPage {
                accounts,
                next_after_source_account_id,
                domain_account_count,
                domain_fingerprint,
            })
        })
}
