//! Derived Metrics coverage runs: the H1 foundation of `MET-WP4-03C-B1`
//! (issue #952, Specification Amendments 5, 5A and 5B).
//!
//! This module owns the persisted `metric_coverage_run` model and the two
//! protected maintenance operations over it. A coverage run is the
//! externally observable winning coverage assertion of one **stream** —
//! `(source_account_id, platform_id, publisher_id, measure_id)` — over one
//! maximal half-open range of days `[run_start, run_end)`. It is derived,
//! rebuildable and independently verifiable state whose only authority is
//! the raw evidence `metric_coverage` joined to its terminal `metric_import`:
//!
//! - the stream's `source_account_id`, `platform_id` and `measure_id` are the
//!   coverage row's, and its `publisher_id` is the **import's**
//!   `publisher_id` (Amendment 5B). The current `metric_source_account`
//!   platform and expected publisher are request-time serving eligibility
//!   (Model A) and are never written into a run;
//! - a coverage assertion participates only while its import is terminal
//!   (`COMPLETED` or `COMPLETED_WITH_ERRORS`), has a recorded `completed_at`
//!   and names a publisher; a terminal import with a null publisher
//!   contributes no run;
//! - per day the winner is the first assertion covering that day under
//!   `completed_at DESC, import_id DESC, coverage_status DESC,
//!   country_coverage ASC, institution_coverage ASC, coverage_id DESC`, and
//!   a run is a maximal range of consecutive covered days whose four
//!   visible values (`coverage_status`, `import_status`, `country_coverage`,
//!   `institution_coverage`) are equal. A change of winning import or
//!   coverage row alone never splits a run, and no per-day row is stored.
//!
//! Incremental maintenance is a PostgreSQL trigger installed by migration
//! `20261006_v1.9.0` on `metric_import`, firing inside the transaction that
//! terminalizes an import (or later changes a terminal import's `status`,
//! `completed_at` or `publisher_id`). It requires `READ COMMITTED`, takes the
//! account row `FOR UPDATE` — the single serialization point it shares with
//! the rebuild below — and replaces every stream of the account over the
//! import's coverage hull from raw evidence, preserving unaffected outer
//! fragments and coalescing maximally. No application code gates it, and no
//! account, source or coverage writer maintains a run.
//!
//! [`verification`] is the independent verifier: for each source account of
//! a strictly ascending keyset page it derives the expected runs from raw
//! evidence by **per-day expansion** — the formulation the current dashboard
//! assertion path uses, and deliberately not the boundary sweep the trigger
//! and the rebuild use — compares them with the durable rows by primary key,
//! and counts structural violations, all in one `READ ONLY`,
//! `REPEATABLE READ` snapshot. [`crud`] is the one-account rebuild: in one
//! `READ COMMITTED` transaction it locks the account row, verifies
//! independently, and only if the account is not exact replaces its runs
//! from raw evidence and verifies independently again before commit.
//!
//! What is deliberately absent: any reader of this table (the
//! `metricDashboard` cutover is Unit B2, separately specified from the
//! exact merged B1 base), any historical backfill in the migration, any
//! scheduled or automatic rebuild, any winner identity, timestamp,
//! generation or watermark column, and any public error code beyond the
//! three existing classifications mapped in
//! [`MetricCoverageRunMaintenanceError`].

use chrono::NaiveDate;
use diesel::r2d2::PoolError;
use diesel::result::Error as DieselError;
use juniper::{graphql_value, FieldError, IntoFieldError};
use uuid::Uuid;

use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_import::MetricImportStatus;

/// One persisted coverage run: the exact approved ten-column v1 row.
///
/// `[run_start, run_end)` is half-open and the database enforces
/// `run_end > run_start`; the primary key is the stream identity plus
/// `run_start`. The four visible values are the winning assertion's declared
/// coverage status, its import's raw terminal status, and whether the
/// assertion includes the country and institution dimensions. The four
/// foreign keys are plain non-cascading references, so a referenced account,
/// platform, publisher or measure cannot be deleted while a run names it.
#[derive(diesel::Queryable, Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverageRun {
    pub source_account_id: Uuid,
    pub platform_id: Uuid,
    pub publisher_id: Uuid,
    pub measure_id: Uuid,
    pub run_start: NaiveDate,
    pub run_end: NaiveDate,
    pub coverage_status: MetricCoverageStatus,
    pub import_status: MetricImportStatus,
    pub country_coverage: bool,
    pub institution_coverage: bool,
}

/// The `verifyMetricCoverageRuns` limit applied when none is supplied.
pub const METRIC_COVERAGE_RUN_VERIFY_DEFAULT_LIMIT: i32 = 10;
/// The smallest accepted `verifyMetricCoverageRuns` limit.
pub const METRIC_COVERAGE_RUN_VERIFY_MIN_LIMIT: i32 = 1;
/// The largest accepted `verifyMetricCoverageRuns` limit.
///
/// A limit outside `1..=10` is **rejected**, not clamped: a verification
/// page's size is part of the full-domain traversal protocol, and silently
/// granting a different span than the caller asked for would make the
/// caller's own idea of where its traversal stands wrong.
pub const METRIC_COVERAGE_RUN_VERIFY_MAX_LIMIT: i32 = 10;

/// The `lock_timeout` a verification or rebuild transaction sets locally,
/// in seconds.
pub const METRIC_COVERAGE_RUN_MAINTENANCE_LOCK_TIMEOUT_SECONDS: u64 = 5;
/// The `statement_timeout` a verification or rebuild transaction sets
/// locally, in seconds.
pub const METRIC_COVERAGE_RUN_MAINTENANCE_STATEMENT_TIMEOUT_SECONDS: u64 = 30;

/// The local lock timeout of a maintenance transaction:
/// [`METRIC_COVERAGE_RUN_MAINTENANCE_LOCK_TIMEOUT_SECONDS`].
pub(crate) const MAINTENANCE_LOCK_TIMEOUT_SQL: &str = "SET LOCAL lock_timeout = '5s'";
/// The local statement timeout of a maintenance transaction:
/// [`METRIC_COVERAGE_RUN_MAINTENANCE_STATEMENT_TIMEOUT_SECONDS`].
pub(crate) const MAINTENANCE_STATEMENT_TIMEOUT_SQL: &str = "SET LOCAL statement_timeout = '30s'";

/// The first line of the domain fingerprint byte stream (Amendment 5A
/// section 10): this literal, one LF, then every source-account UUID in
/// canonical lowercase hyphenated form, ascending, each followed by one LF.
pub const METRIC_COVERAGE_RUN_DOMAIN_FINGERPRINT_SCHEMA: &str =
    "thoth-metric-coverage-run-domain/1";

/// The independent verification of one source account's coverage runs.
///
/// Only bounded counts leave Thoth: no run, stream identity, raw evidence
/// row or mismatch detail is carried. `expected_rows` are the maximally
/// coalesced runs independently derived from raw evidence, `actual_rows` the
/// durable rows. `missing_rows` are expected primary keys with no actual
/// row, `extra_rows` actual primary keys with no expected row, and
/// `mismatched_rows` primary keys present on both sides whose `run_end`,
/// `coverage_status`, `import_status`, `country_coverage` or
/// `institution_coverage` differs; one key is never counted both ways.
/// `structural_violations` is the additive sum of the four deterministic
/// predicates over the actual rows (Amendment 5A section 12, 5B section 7):
/// V1 an invalid interval, V2 a stream identity that no longer resolves to
/// its canonical account, platform, publisher or measure row, V3 an overlap
/// between adjacent runs of one stream, and V4 two adjacent runs of one
/// stream that should have been coalesced. Current account enablement,
/// platform or expected publisher is never a structural criterion.
///
/// Every count is a GraphQL `Int`. A count that does not fit fails the whole
/// verification closed as a sanitized internal failure instead of wrapping,
/// saturating or truncating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverageRunAccountVerification {
    pub source_account_id: Uuid,
    pub expected_rows: i32,
    pub actual_rows: i32,
    pub missing_rows: i32,
    pub extra_rows: i32,
    pub mismatched_rows: i32,
    pub structural_violations: i32,
    /// True exactly when every mismatch and structural count is zero.
    pub exact: bool,
}

/// One page of the full-domain verification traversal.
///
/// `accounts` are the next at most `limit` source accounts whose IDs are
/// strictly greater than the exclusive boundary, in exact ascending UUID
/// order; `next_after_source_account_id` is the last returned ID when at
/// least one further account exists, and `None` when the page exhausts the
/// domain or is empty. `domain_account_count` and `domain_fingerprint`
/// describe the **complete** `metric_source_account` domain visible in this
/// page's own snapshot, so every page of one coherent traversal reports the
/// same two values and a change between pages invalidates the traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverageRunVerificationPage {
    pub accounts: Vec<MetricCoverageRunAccountVerification>,
    pub next_after_source_account_id: Option<Uuid>,
    pub domain_account_count: i32,
    pub domain_fingerprint: String,
}

/// The receipt of one `rebuildMetricCoverageRuns` request.
///
/// `rebuilt = false` means the account was already exact and no run was
/// written; `rebuilt = true` means its runs were replaced from raw evidence
/// and independently verified exact inside the same committed transaction.
/// In both cases `verification` is the exact state the transaction committed
/// with: a rebuild whose post-verification is not exact rolls back and
/// returns a failure, never a non-exact receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverageRunRebuildResult {
    pub source_account_id: Uuid,
    pub rebuilt: bool,
    pub verification: MetricCoverageRunAccountVerification,
}

/// The H1-local internal error boundary of the two maintenance operations.
///
/// Deliberately not `metric_dashboard::MetricReadError` or any other
/// reader-owned type: the maintenance API is coupled to no reader. Each
/// variant maps to exactly one existing stable public classification and
/// carries nothing a caller could read back. Authorization failures never
/// reach this type; the resolver denies them before any coverage-run read.
///
/// Every database, pool, isolation, verifier or rebuild failure becomes
/// [`Self::Internal`]: the detail is logged server-side and the client sees
/// only the fixed message and `INTERNAL_ERROR`. No SQL, constraint name,
/// raw evidence row, source configuration or connection detail leaves Thoth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricCoverageRunMaintenanceError {
    /// The verification limit is outside `1..=10`:
    /// `METRIC_QUERY_LIMIT_EXCEEDED`.
    VerifyLimitOutOfRange,
    /// The source account to rebuild does not exist: `METRIC_QUERY_INVALID`.
    UnknownSourceAccount,
    /// An unexpected database, isolation, verification or rebuild failure,
    /// deliberately without detail: `INTERNAL_ERROR`.
    Internal,
}

impl MetricCoverageRunMaintenanceError {
    /// The stable machine-readable code, returned as `extensions.type`.
    pub fn code(self) -> &'static str {
        match self {
            Self::VerifyLimitOutOfRange => "METRIC_QUERY_LIMIT_EXCEEDED",
            Self::UnknownSourceAccount => "METRIC_QUERY_INVALID",
            Self::Internal => "INTERNAL_ERROR",
        }
    }

    /// The fixed client-facing message. Never derived from input or state.
    pub fn message(self) -> &'static str {
        match self {
            Self::VerifyLimitOutOfRange => {
                "verifyMetricCoverageRuns limit must be between 1 and 10 inclusive."
            }
            Self::UnknownSourceAccount => "The metric source account was not found.",
            Self::Internal => "The coverage-run maintenance operation could not be completed.",
        }
    }
}

impl std::fmt::Display for MetricCoverageRunMaintenanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for MetricCoverageRunMaintenanceError {}

impl From<DieselError> for MetricCoverageRunMaintenanceError {
    fn from(error: DieselError) -> Self {
        log::error!("metric coverage-run maintenance database failure: {error}");
        Self::Internal
    }
}

impl From<PoolError> for MetricCoverageRunMaintenanceError {
    fn from(error: PoolError) -> Self {
        log::error!(
            "metric coverage-run maintenance could not obtain a database connection: {error}"
        );
        Self::Internal
    }
}

impl IntoFieldError for MetricCoverageRunMaintenanceError {
    fn into_field_error(self) -> FieldError {
        let code = self.code();
        FieldError::new(self.message(), graphql_value!({ "type": code }))
    }
}

/// The result of one maintenance operation.
pub(crate) type MaintenanceResult<T> = Result<T, MetricCoverageRunMaintenanceError>;

pub mod crud;
#[cfg(test)]
pub(crate) mod tests;
pub mod verification;
