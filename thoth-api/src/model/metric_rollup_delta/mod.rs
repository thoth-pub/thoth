//! Durable Metrics rollup deltas, the MOM-1 work-day projection, the
//! derived monthly serving projections and the derived yearly projection
//! foundation (`MET-WP1-07`, `MET-WP4-01`, `MET-WP4-03A`, `MET-WP4-03C-A`).
//!
//! This module owns the persisted `metric_rollup_delta` model: the durable
//! accounting bridge between one canonical metric-record revision and the
//! rebuildable work-level rollup projections. The approved Metrics design
//! commits a canonical revision transactionally together with its delta —
//! applying a new record adds its value, a revision contributes `new - old`,
//! and a retraction subtracts the old value — so a delta is canonical
//! accounting evidence rather than a derived cache.
//!
//! `MET-WP1-07` established delta storage only. `MET-WP4-01` adds the MOM-1
//! application path on top of it: a gap-free work-day progress ordering, a
//! strict-frontier claim/lease protocol, whole-batch atomic application into
//! the work-day projection `metric_rollup_work_day`, and a durable
//! contiguous watermark. `MET-WP4-03A` adds the derived monthly serving
//! layer beneath that same completion transaction: three resolved monthly
//! value projections (`metric_rollup_work_month`,
//! `metric_rollup_work_country_month`, `metric_rollup_work_institution_month`)
//! and the sparse `metric_rollup_work_month_ambiguity` state, all derived
//! exclusively from `metric_rollup_work_day` after each daily base cell has
//! been resolved. See [`crud`] for the two protected operations and their
//! exact transaction shapes.
//!
//! `MET-WP4-03A-OPS-02` adds the two protected maintenance operations the
//! monthly layer needs before it may be served from: a database-read-only
//! consistency verification that independently derives the expected monthly
//! state from `metric_rollup_work_day` and compares all four monthly
//! datasets by bounded counts (see [`verification`]), and an exceptional,
//! self-verifying full rebuild at a caller-pinned frontier that replaces the
//! four monthly datasets only when they are not already exact (see
//! [`crud`]). Neither reads or writes canonical state, rollup deltas, the
//! work-day projection or the durable frontier.
//!
//! `MET-WP4-03C-A` (Unit A of `MET-WP4-03C`, #952, Specification
//! Amendments 4, 4A and 4B) adds the yearly producer, verification and
//! rebuild foundation beneath the same completion transaction: two resolved
//! yearly section projections (`metric_rollup_work_country_year`,
//! `metric_rollup_work_institution_year`) that regroup, by calendar year,
//! exactly the resolved daily contributions the monthly section rows already
//! hold. They are maintained from the monthly rows by four more fixed
//! statements inside every completion, verified by the same independent
//! work-day derivation as the monthly families, and replaced together with
//! the monthly datasets by the same all-or-nothing rebuild. The two yearly
//! families are already part of the approved six-family maintenance API:
//! `verifyMetricRollupMonths`, and the `verification` returned by
//! `rebuildMetricRollupMonths`, report them as the additive bounded-count
//! fields `countryYear` and `institutionYear`.
//!
//! Unit A adds no reader of the yearly tables. No `metricDashboard` source
//! reads them, and nothing serves complete calendar years from them. A
//! `metricDashboard` reader of the yearly layer is Unit B of `MET-WP4-03C`,
//! which is separately gated and on HOLD; anything said in this module about
//! such a reader is prospective and does not describe existing behaviour.
//!
//! What is still deliberately absent: any reader of the yearly projections,
//! any reader of the monthly projections other than the separately specified
//! `MET-WP4-03B` dashboard, any retry/backoff or poison-skipping behaviour,
//! any scheduled or automatic rebuild, and any progress stream for a
//! non-`DAY` grain. A poison frontier blocks and waits for separately
//! authorized repair rather than being stepped over. A verification mismatch
//! never invokes a rebuild by itself. The monthly and yearly tables are
//! created empty by their migrations and may not be served from until a
//! separately authorized rebuild has populated them and an independent
//! verification is exact at a recorded work-day frontier; any serving from
//! the yearly tables additionally requires the separately authorized Unit B
//! activation.
//!
//! `MET-WP4-03A-OPS-03` (#1012) adds a resumable, caller-driven rebuild and
//! a paged verification of the same six families for corpora of any size
//! (see [`rebuild_run`]). A run is one row of
//! `metric_rollup_month_rebuild_run`: a durable cursor over units
//! `(work_id, year_start)` and the counters of its build steps and verify
//! pages. Each step or page is one short transaction under fixed
//! server-side budgets; a build step replaces only a range that its own
//! independent verification finds inexact, by re-executing the completion's
//! keyed statements, and never truncates. The caller drives every step:
//! Thoth runs nothing on its own, and a mismatch never starts a rebuild.
//! The OPS-02 pair is unchanged.

use std::time::Duration;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted durable rollup delta row.
///
/// The database enforces `UNIQUE(revision_id)`, so at most one durable delta
/// exists per canonical revision and later rollup application cannot double
/// count a duplicated row. A composite foreign key over
/// `(record_id, revision_id)` against the MET-WP1-04
/// `metric_record_revision (record_id, record_revision_id)` unique key
/// guarantees the named revision genuinely belongs to the named record. That
/// foreign key is non-cascading, so deleting a canonical record or revision
/// that still has a delta fails rather than silently erasing accounting
/// evidence.
///
/// `delta_value` is a signed `i64` with deliberately **no** non-negative
/// rule: a revision contributes the signed difference `new - old` and a
/// retraction subtracts the previously applied value, so positive, zero and
/// negative values are all valid.
///
/// `status` is a plain `String`, not an enum. The approved design names the
/// field but defines no closed status vocabulary, transition model, claim
/// ownership, lease or recovery protocol, so this foundation deliberately
/// declares no closed runtime state machine — in the database or in Rust —
/// and offers no claim or apply method. `applied_at` is correspondingly a
/// plain nullable timestamp: nothing in this slice ties it to any particular
/// `status` value.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupDelta {
    pub delta_id: Uuid,
    pub record_id: Uuid,
    pub revision_id: Uuid,
    pub delta_value: i64,
    pub status: String,
    pub created_at: Timestamp,
    pub applied_at: Option<Timestamp>,
    pub work_day_sequence: Option<i64>,
    pub claim_token: Option<Uuid>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<Timestamp>,
    pub lease_expires_at: Option<Timestamp>,
}

/// One row of the single MOM-1 rollup projection.
///
/// Derived, rebuildable state. `value` is a signed `i64` because a revision
/// applies `new - old`, which is negative whenever a source corrects a figure
/// downwards, and a zero-valued row is retained rather than deleted so the
/// difference between "counted, and the answer is zero" and "never counted"
/// survives.
///
/// `watermark` is an **ordered work-day progress position**, not a wall-clock
/// time: it is the greatest `work_day_sequence` that has actually changed this
/// row. It never overrides the global serving boundary in
/// [`MetricRollupWorkDayState::applied_through_sequence`], which is the only
/// value a read path may treat as "everything up to here is applied".
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkDay {
    pub rollup_work_day_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub day: NaiveDate,
    pub country_code: Option<String>,
    pub institution_id: Option<Uuid>,
    pub value: i64,
    pub watermark: i64,
}

/// One row of the resolved monthly total projection (`MET-WP4-03A`).
///
/// Derived, rebuildable state fed only by `metric_rollup_work_day`. Each
/// daily base cell `(work, platform, measure, day)` is resolved first under
/// the `MET-WP4-02` Amendment 6 total rule — an undimensioned row is
/// authoritative, otherwise exactly one represented optional-dimension mask
/// is summed, otherwise the cell is ambiguous and contributes nothing here —
/// and only those resolved contributions are summed into the month.
/// `publication_id` is retained exactly when the selected daily
/// representation carries it, so a month may hold both a `NULL` row and
/// publication-specific rows from different days; they are additive and are
/// never re-ranked against each other.
///
/// `requires_country_coverage` and `requires_institution_coverage` are the
/// `MET-WP4-02` range-wide dependency flags, OR-ed over the resolved daily
/// contributions grouped into this row: true when at least one of them was
/// taken from rows broken down by that dimension.
///
/// `watermark` is the greatest [`MetricRollupWorkDay::watermark`] over
/// exactly the current daily source rows that contribute to this row. It is
/// local derived-state evidence, never a serving boundary, and it never
/// exceeds [`MetricRollupWorkDayState::applied_through_sequence`].
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkMonth {
    pub rollup_work_month_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub month_start: NaiveDate,
    pub value: i64,
    pub requires_country_coverage: bool,
    pub requires_institution_coverage: bool,
    pub watermark: i64,
}

/// One row of the resolved monthly per-country projection (`MET-WP4-03A`).
///
/// For each daily base cell the represented masks containing country are
/// considered and the unique least one under set inclusion is selected; the
/// non-target dimensions of that representation are summed away, and
/// `publication_id` is retained only if the representation carries it. A
/// cell with no country-bearing mask contributes nothing, and a cell whose
/// minimal country masks are incomparable contributes nothing and is
/// recorded as `country_ambiguous` instead. `requires_institution_coverage`
/// is OR-ed over the contributing resolved days. `watermark` follows the
/// same rule as [`MetricRollupWorkMonth::watermark`].
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkCountryMonth {
    pub rollup_work_country_month_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub month_start: NaiveDate,
    pub country_code: String,
    pub value: i64,
    pub requires_institution_coverage: bool,
    pub watermark: i64,
}

/// One row of the resolved monthly per-institution projection
/// (`MET-WP4-03A`): the mirror of [`MetricRollupWorkCountryMonth`] with
/// institution as the target dimension and country as the dependency.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkInstitutionMonth {
    pub rollup_work_institution_month_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub month_start: NaiveDate,
    pub institution_id: Uuid,
    pub value: i64,
    pub requires_country_coverage: bool,
    pub watermark: i64,
}

/// One row of the resolved yearly per-country projection (`MET-WP4-03C-A`).
///
/// Unit A producer state: maintained, verified and rebuilt here, and read by
/// no `metricDashboard` source (the yearly reader, Unit B, is separately
/// gated and on HOLD).
///
/// Derived, rebuildable state fed only by the reviewed monthly per-country
/// projection: the regrouping, by calendar year, of exactly the resolved
/// daily contributions the monthly rows of that year already hold. `value`
/// is the sum of the monthly values, `requires_institution_coverage` the OR
/// of the monthly flags and `watermark` the greatest monthly watermark; a
/// monthly row is itself `SUM` / OR / `MAX` over resolved daily contributions,
/// so dropping the month key regroups the same contributions and no daily
/// base cell is ever re-resolved at the year. `publication_id` is retained
/// exactly as the monthly rows hold it, so a year may hold both a `NULL` row
/// and publication-specific rows; they are additive and never re-ranked.
/// `year_start` is the 1 January of the year. Ambiguity is not aggregated:
/// it stays sparse and monthly. `watermark` follows the same rule as
/// [`MetricRollupWorkMonth::watermark`].
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkCountryYear {
    pub rollup_work_country_year_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub year_start: NaiveDate,
    pub country_code: String,
    pub value: i64,
    pub requires_institution_coverage: bool,
    pub watermark: i64,
}

/// One row of the resolved yearly per-institution projection
/// (`MET-WP4-03C-A`): the mirror of [`MetricRollupWorkCountryYear`] over the
/// monthly per-institution projection, with country as the dependency. Like
/// it, Unit A producer state that no `metricDashboard` source reads.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkInstitutionYear {
    pub rollup_work_institution_year_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub year_start: NaiveDate,
    pub institution_id: Uuid,
    pub value: i64,
    pub requires_country_coverage: bool,
    pub watermark: i64,
}

/// One sparse row of monthly ambiguity state (`MET-WP4-03A`).
///
/// Exists only while at least one flag is true, and may exist for a month
/// that has no value row at all in the ambiguous section. Ambiguity never
/// blocks completion or the frontier: it is fail-closed serving evidence the
/// later reader must honour. `watermark` is the greatest
/// [`MetricRollupWorkDay::watermark`] over the union of the daily source
/// rows whose current represented state establishes any currently-true
/// flag; when every flag clears, the row is removed rather than retained for
/// its old watermark.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkMonthAmbiguity {
    pub rollup_work_month_ambiguity_id: Uuid,
    pub work_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub month_start: NaiveDate,
    pub total_ambiguous: bool,
    pub country_ambiguous: bool,
    pub institution_ambiguous: bool,
    pub watermark: i64,
}

/// The singleton work-day progress and watermark row.
///
/// Exactly one row exists, with `state_id = 1`. It owns the next work-day
/// position to allocate and the contiguous applied-through frontier, and the
/// database enforces `applied_through_sequence < next_sequence` so the
/// frontier can never claim a position that was never allocated.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkDayState {
    pub state_id: i16,
    pub next_sequence: i64,
    pub applied_through_sequence: i64,
    pub watermark_at: Timestamp,
    pub updated_at: Timestamp,
}

/// The durable `PENDING` status written by canonical ingestion.
pub const METRIC_ROLLUP_DELTA_PENDING: &str = "PENDING";
/// The durable `CLAIMED` status written by a successful claim.
pub const METRIC_ROLLUP_DELTA_CLAIMED: &str = "CLAIMED";
/// The durable terminal `APPLIED` status written by a successful completion.
pub const METRIC_ROLLUP_DELTA_APPLIED: &str = "APPLIED";

/// The smallest accepted `claimMetricRollupDeltas` limit.
pub const METRIC_ROLLUP_CLAIM_MIN_BATCH: i32 = 1;
/// The largest accepted `claimMetricRollupDeltas` limit.
///
/// A limit outside `1..=50` is **rejected**, not clamped. Unlike the
/// `BE-04` worker claim, where clamping keeps a long-running process making
/// bounded progress, a rollup claim's batch size determines exactly how far
/// the single global frontier advances in one transaction; silently granting
/// a different span than the caller asked for would make the caller's own
/// idea of what it holds wrong.
pub const METRIC_ROLLUP_CLAIM_MAX_BATCH: i32 = 50;

/// The server-fixed rollup lease, in seconds.
///
/// Fixed rather than caller-supplied: the lease is the only thing that makes
/// a crashed claimant's hold on the global frontier recoverable, so its
/// duration is a property of the protocol rather than a request parameter.
pub const METRIC_ROLLUP_LEASE_SECONDS: i32 = 900;

/// One delta granted by a successful claim.
///
/// Every row of one claim carries the **same** `claim_token` and the same
/// `lease_expires_at`: the token identifies the whole contiguous batch, and
/// completion addresses the batch by that token alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupDeltaClaim {
    pub delta_id: Uuid,
    pub work_day_sequence: i64,
    pub claim_token: Uuid,
    pub lease_expires_at: Timestamp,
}

/// The durable global rollup serving boundary.
///
/// `applied_through_sequence = W` means every committed work-day delta
/// position `1..=W` is `APPLIED`. Because allocation is gap-free and
/// application is strict-frontier, no projection row can hold the effect of a
/// work-day delta above `W` after a committed transaction.
///
/// `watermark_at` is the wall-clock time at which the current `W` was
/// atomically established. It is a freshness fact about the frontier, not a
/// claim that every canonical row is projected: a reader detects outstanding
/// rollup lag by comparing `W` with `next_sequence - 1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWatermark {
    pub applied_through_sequence: i64,
    pub watermark_at: Timestamp,
}

/// Which claimed rollup batch to apply.
///
/// This carries the batch claim token and **nothing else**. No delta id,
/// arithmetic value, projection dimension or watermark value is accepted from
/// the caller: Thoth derives every one of them from the durable claimed batch
/// and the canonical records it names, so a claimant cannot choose what its
/// completion adds, to which aggregate, or how far the watermark moves.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which claimed rollup delta batch to apply")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteMetricRollupDeltasInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The batch token returned by every row of one successful claim. It is the only accepted input: all values, dimensions and the resulting watermark are derived from durable state"
        )
    )]
    pub claim_token: Uuid,
}

/// The largest number of represented month keys one full rebuild of the
/// derived monthly and yearly projections supports (`MET-WP4-03A-OPS-02`).
///
/// A work-day projection representing more keys than this is rejected before
/// any monthly or yearly row is touched. It is a safety envelope for the one
/// all-or-nothing rebuild transaction, not a production SLO, and it is not
/// raised silently: a larger corpus needs a separately approved envelope.
pub const METRIC_ROLLUP_REBUILD_MAX_MONTH_KEYS: i64 = 100_000;

/// The largest number of month keys one rebuild chunk recomputes at once:
/// the same bound as a claim batch, so a rebuild replays exactly the keyed
/// recomputation shape a completion uses.
pub const METRIC_ROLLUP_REBUILD_CHUNK_KEYS: usize = METRIC_ROLLUP_CLAIM_MAX_BATCH as usize;

/// The server-side elapsed ceiling of one whole rebuild operation, in
/// seconds, covering its initial verification, every rebuild chunk, the
/// yearly derivation, its post-rebuild verification and the moment before
/// commit.
///
/// Exceeding it at any checkpoint fails the operation and rolls the whole
/// transaction back, so no partially rebuilt monthly or yearly state can
/// commit.
pub const METRIC_ROLLUP_REBUILD_ELAPSED_CEILING_SECONDS: u64 = 120;

/// The `lock_timeout` a rebuild transaction sets locally, in seconds.
pub const METRIC_ROLLUP_REBUILD_LOCK_TIMEOUT_SECONDS: u64 = 5;

/// The `statement_timeout` both a verification and a rebuild transaction set
/// locally, in seconds.
pub const METRIC_ROLLUP_MAINTENANCE_STATEMENT_TIMEOUT_SECONDS: u64 = 30;

/// Bounded verification counts for one derived projection family
/// (`MET-WP4-03A-OPS-02`, extended to the yearly families by `MET-WP4-03C-A`).
///
/// Only counts leave Thoth: no row identity, value or mismatch detail is
/// carried. `missing_rows` counts expected logical identities absent from the
/// actual table, `extra_rows` actual identities absent from the expected
/// state, and `mismatched_rows` identities present on both sides whose
/// value, dependency flag, ambiguity flag or watermark differs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricRollupMonthProjectionVerification {
    pub expected_rows: i64,
    pub actual_rows: i64,
    pub missing_rows: i64,
    pub extra_rows: i64,
    pub mismatched_rows: i64,
}

/// One coherent verification of the four monthly and the two yearly derived
/// projections against the state independently derived from
/// `metric_rollup_work_day` (`MET-WP4-03A-OPS-02`, `MET-WP4-03C-A`).
///
/// `applied_through_sequence` is the frontier `W` the verification was taken
/// at, and `next_sequence - 1 > W` is ordinary pending rollup lag, never
/// derived-state corruption. `max_work_day_watermark` is `None` when the
/// work-day projection is empty; it is never above `W` in a returned value,
/// because a work-day row above the frontier fails the operation closed
/// instead. `matches` is true only when every one of the six families has
/// zero missing, extra and mismatched rows and no monthly or yearly row is
/// watermarked above `W`.
///
/// `country_year` and `institution_year` are the `MET-WP4-03C-A` yearly
/// families. They are part of `matches` and of every rebuild decision, and
/// they are already part of the approved six-family maintenance API
/// (Specification Amendments 4 and 4A): the GraphQL
/// `verifyMetricRollupMonths` result, and the `verification` of
/// `rebuildMetricRollupMonths`, expose them as the two additive fields
/// `countryYear` and `institutionYear`, each with the unchanged five-count
/// per-family shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupMonthVerification {
    pub applied_through_sequence: i64,
    pub next_sequence: i64,
    pub watermark_at: Timestamp,
    pub max_work_day_watermark: Option<i64>,
    pub work_day_row_count: i64,
    pub represented_month_key_count: i64,
    pub total: MetricRollupMonthProjectionVerification,
    pub country: MetricRollupMonthProjectionVerification,
    pub institution: MetricRollupMonthProjectionVerification,
    pub ambiguity: MetricRollupMonthProjectionVerification,
    pub country_year: MetricRollupMonthProjectionVerification,
    pub institution_year: MetricRollupMonthProjectionVerification,
    pub matches: bool,
}

/// The receipt of one rebuild request (`MET-WP4-03A-OPS-02`).
///
/// `rebuilt = false` means the monthly and yearly state was already exact at
/// the pinned frontier and nothing was truncated or written; `rebuilt = true`
/// means the four monthly and the two yearly datasets were replaced and
/// independently verified exact inside the same committed transaction. In
/// both cases `verification` is the exact state the transaction committed
/// with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildResult {
    pub rebuilt: bool,
    pub verification: MetricRollupMonthVerification,
}

/// Which frontier a rebuild is pinned to.
///
/// This carries the caller's expected `applied_through_sequence` and
/// **nothing else**: no work, month, platform or measure identity, value,
/// dimension, dependency or ambiguity flag, watermark, SQL or repair
/// instruction is accepted. Thoth derives every monthly row from the durable
/// work-day projection alone and every yearly row from those monthly rows,
/// so a caller can only say which frontier it believes the derived state
/// should be rebuilt at, and a stale frontier is rejected before any monthly
/// or yearly row is touched.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which durable rollup frontier the monthly rebuild is pinned to")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuildMetricRollupMonthsInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The current appliedThroughSequence the caller observed, as a decimal string holding a non-negative 64-bit integer. It is the only accepted input: the rebuild is rejected before any monthly row is touched if the durable frontier differs, and every rebuilt value is derived from durable work-day state"
        )
    )]
    pub expected_applied_through_sequence: String,
}

// ---------------------------------------------------------------------------
// MET-WP4-03A-OPS-03: resumable rebuild and paged verification
// ---------------------------------------------------------------------------
//
// The server-fixed bounds of the run operations (specification section 10).
// The caller supplies no size, range, key, value or SQL, and changing any of
// these needs an amendment with evidence. Register section 3.8 quotes the
// step budgets beside the run-grant aggregate, and a test pins the quotation
// to these values.

/// Pre-sizing a build step, before any row lock is taken.
pub const METRIC_ROLLUP_RUN_STEP_PRESIZE_BUDGET: Duration = Duration::from_secs(5);

/// A build step's lock phase: from the statement that locks the step's
/// `work` rows until the state row is held.
pub const METRIC_ROLLUP_RUN_STEP_LOCK_BUDGET: Duration = Duration::from_secs(2);

/// A build step's hold of the state row: from acquiring it to the end of the
/// transaction, enforced before and during every statement.
pub const METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET: Duration = Duration::from_secs(5);

/// The month keys one build step takes, unless one unit alone exceeds it.
pub const METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS: i64 = 1_000;

/// The work-day rows one build step takes, unless one unit alone exceeds it.
pub const METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS: i64 = 50_000;

/// The replacing-read figure one build step takes, unless one unit alone
/// exceeds it: over the step's work-months, the month keys of the
/// work-month times the work's work-day rows in that month, plus, over the
/// step's year keys, the work's work-day rows across its whole history.
/// Provisional: the section 17 F4 evidence confirms it.
pub const METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ: i64 = 4_000_000;

/// The month keys, and separately the year keys, one recompute chunk of a
/// replacing build step takes: the same bound as a rebuild chunk and a claim
/// batch.
pub const METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS: usize = METRIC_ROLLUP_REBUILD_CHUNK_KEYS;

/// The works one sizing statement walks at most.
pub const METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS: i64 = 256;

/// One verify page: from its `LOCK TABLE` to the end of its transaction,
/// enforced before and during every statement.
pub const METRIC_ROLLUP_RUN_PAGE_BUDGET: Duration = Duration::from_secs(15);

/// The month keys one verify page takes, unless one unit alone exceeds it.
pub const METRIC_ROLLUP_RUN_PAGE_MAX_MONTH_KEYS: i64 = 10_000;

/// The work-day rows one verify page takes, unless one unit alone exceeds it.
pub const METRIC_ROLLUP_RUN_PAGE_MAX_WORK_DAY_ROWS: i64 = 500_000;

/// The `lock_timeout` of every run transaction except the dispatch and
/// status reads: the run row, `LOCK TABLE`, start and abandon. A step's
/// lock waits are also cut by its budgets.
pub const METRIC_ROLLUP_RUN_LOCK_TIMEOUT: Duration = Duration::from_secs(5);

/// The largest `statement_timeout` any run statement is given.
pub const METRIC_ROLLUP_RUN_STATEMENT_TIMEOUT_CEILING: Duration = Duration::from_secs(30);

/// The `lock_timeout` and `statement_timeout` of the dispatch read of an
/// advance and of the status read.
pub const METRIC_ROLLUP_RUN_READ_TIMEOUT: Duration = Duration::from_secs(1);

/// The longest wait for the one pooled connection every run operation takes.
pub const METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT: Duration = Duration::from_secs(2);

/// What a run does.
///
/// A `REBUILD` run first builds (replacing only ranges its own verification
/// finds inexact) and then verifies; a `VERIFY` run only verifies and never
/// writes a projection row.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "What a resumable monthly rebuild run does")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricRollupMonthRebuildMode {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Build, replacing only the ranges an independent verification finds inexact, then verify every range"
        )
    )]
    Rebuild,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Verify every range only. Never writes a projection row")
    )]
    Verify,
}

impl MetricRollupMonthRebuildMode {
    /// The stored value of the closed `mode` vocabulary.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rebuild => "REBUILD",
            Self::Verify => "VERIFY",
        }
    }

    /// The mode a stored value names, if any.
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "REBUILD" => Some(Self::Rebuild),
            "VERIFY" => Some(Self::Verify),
            _ => None,
        }
    }
}

/// Where a run is in its state machine.
///
/// `BUILDING` and `VERIFYING` are open; at most one run is open at a time.
/// `RECONCILED` means every verify page was exact at one and the same
/// frontier. `VERIFIED` means every page was exact but the frontier moved
/// between pages: health evidence, never activation evidence. `MISMATCHED`
/// means some page found a missing, extra or mismatched row. `ABANDONED`
/// means the caller stopped the run.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Where a resumable monthly rebuild run is in its state machine")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricRollupMonthRebuildStatus {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Open: a REBUILD run is taking build steps")
    )]
    Building,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Open: the run is taking verify pages")
    )]
    Verifying,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Terminal: every verify page was exact, all at one and the same recorded frontier"
        )
    )]
    Reconciled,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Terminal: every verify page was exact, but the frontier moved between pages. Health evidence only, never activation evidence"
        )
    )]
    Verified,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Terminal: some verify page found a missing, extra or mismatched row"
        )
    )]
    Mismatched,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Terminal: the caller abandoned the run")
    )]
    Abandoned,
}

impl MetricRollupMonthRebuildStatus {
    /// The stored value of the closed `status` vocabulary.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Building => "BUILDING",
            Self::Verifying => "VERIFYING",
            Self::Reconciled => "RECONCILED",
            Self::Verified => "VERIFIED",
            Self::Mismatched => "MISMATCHED",
            Self::Abandoned => "ABANDONED",
        }
    }

    /// The status a stored value names, if any.
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "BUILDING" => Some(Self::Building),
            "VERIFYING" => Some(Self::Verifying),
            "RECONCILED" => Some(Self::Reconciled),
            "VERIFIED" => Some(Self::Verified),
            "MISMATCHED" => Some(Self::Mismatched),
            "ABANDONED" => Some(Self::Abandoned),
            _ => None,
        }
    }

    /// Whether the run is still open.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Building | Self::Verifying)
    }
}

/// The per-family counts a run accumulates over its verify pages: the five
/// bounded counts of [`MetricRollupMonthProjectionVerification`] for each of
/// the six derived families, each summed over pages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildFamilies {
    pub total: MetricRollupMonthProjectionVerification,
    pub country: MetricRollupMonthProjectionVerification,
    pub institution: MetricRollupMonthProjectionVerification,
    pub ambiguity: MetricRollupMonthProjectionVerification,
    pub country_year: MetricRollupMonthProjectionVerification,
    pub institution_year: MetricRollupMonthProjectionVerification,
}

/// The progress of a `REBUILD` run's build phase. Counts only: the
/// frontiers are the first and the last `applied_through_sequence` a
/// committed build step saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildBuildProgress {
    pub steps: i64,
    pub replaced_steps: i64,
    pub units: i64,
    pub month_keys: i64,
    pub work_day_rows: i64,
    pub found_missing_rows: i64,
    pub found_extra_rows: i64,
    pub found_mismatched_rows: i64,
    pub first_applied_through_sequence: Option<i64>,
    pub last_applied_through_sequence: Option<i64>,
}

/// The progress of a run's verify phase. Counts only: the frontiers are the
/// first and the last `applied_through_sequence` a committed verify page
/// saw, and the per-family counts are sums over pages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildVerifyProgress {
    pub pages: i64,
    pub units: i64,
    pub month_keys: i64,
    pub work_day_rows: i64,
    pub first_applied_through_sequence: Option<i64>,
    pub last_applied_through_sequence: Option<i64>,
    pub counts: MetricRollupMonthRebuildFamilies,
}

impl MetricRollupMonthRebuildVerifyProgress {
    /// True when the verify frontiers are set and equal.
    pub fn has_single_frontier(&self) -> bool {
        self.first_applied_through_sequence.is_some()
            && self.first_applied_through_sequence == self.last_applied_through_sequence
    }
}

/// One resumable monthly rebuild or verification run, as the run operations
/// return it.
///
/// Only the run's own id, counts, frontiers and timestamps leave Thoth: the
/// cursor, every work, platform and measure identity and the principal that
/// started the run are never part of this type. `position_permille`
/// approximates the current phase's cursor position from the cursor's
/// leading 32 bits; it is an estimate, never a count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildRun {
    pub run_id: Uuid,
    pub mode: MetricRollupMonthRebuildMode,
    pub status: MetricRollupMonthRebuildStatus,
    pub step_count: i64,
    pub position_permille: i32,
    pub started_at: Timestamp,
    pub updated_at: Timestamp,
    pub build_completed_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub start_applied_through_sequence: i64,
    /// `None` for a `VERIFY` run.
    pub build: Option<MetricRollupMonthRebuildBuildProgress>,
    pub verify: MetricRollupMonthRebuildVerifyProgress,
}

/// The answer to one advance request: whether this request committed a step
/// or a page, and the run as it now stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupMonthRebuildAdvance {
    pub advanced: bool,
    pub run: MetricRollupMonthRebuildRun,
}

/// Which kind of run to start. It is the only accepted input: every bound is
/// server-fixed and every range is derived from durable state.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which kind of resumable monthly rebuild run to start")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartMetricRollupMonthRebuildInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "REBUILD to build and then verify, or VERIFY to verify only. No size, range, key, value or SQL is accepted"
        )
    )]
    pub mode: MetricRollupMonthRebuildMode,
}

/// Which run to advance, and the step count the caller last saw.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which resumable monthly rebuild run to advance by one step or page")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceMetricRollupMonthRebuildInput {
    #[cfg_attr(feature = "backend", graphql(description = "The run to advance"))]
    pub run_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The run's stepCount the caller last saw, as a decimal string holding a non-negative 64-bit integer. The advance commits a step or page only while the run still has exactly this count, so a retry after a lost response is safe"
        )
    )]
    pub expected_step_count: String,
}

/// Which run to abandon.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which resumable monthly rebuild run to abandon")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbandonMetricRollupMonthRebuildInput {
    #[cfg_attr(feature = "backend", graphql(description = "The run to abandon"))]
    pub run_id: Uuid,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
pub mod rebuild_run;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod rebuild_run_tests;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
#[cfg(feature = "backend")]
pub mod verification;
