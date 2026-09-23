//! The two named domain operations of the MOM-1 rollup application path
//! (`MET-WP4-01`) and the derived monthly serving layer maintained beneath
//! the second of them (`MET-WP4-03A`).
//!
//! `Crud` is deliberately **not** implemented for `metric_rollup_delta`,
//! `metric_rollup_work_day`, `metric_rollup_work_day_state` or any of the
//! four monthly tables. There is no generic create/update/delete surface for
//! rollup state: the two functions here are the only supported writes, and
//! each one implements exactly one transition of the approved protocol. The
//! monthly projections are written only by [`complete_metric_rollup_deltas`],
//! inside its transaction and beneath its state-row lock; the test-only
//! rebuild at the end of this file is the reviewed rebuild procedure, not a
//! callable surface.
//!
//! Every mechanism here is programme-local. There is no generic job
//! framework, no reusable lease abstraction and no cross-programme claim
//! protocol (`ADR-0008` sections 3.4 and 3.5). Nothing in this module
//! authorizes a caller: authorization happens at the resolver boundary,
//! before any function here is called and therefore before any
//! rollup-specific read, row lock or write.
//!
//! # Why the frontier is strict
//!
//! A work-day delta is an accounting *difference*, so the projection is only
//! correct if every delta below a position has already been applied. MOM-1
//! therefore serializes application at one global frontier: a claim starts at
//! exactly `applied_through_sequence + 1`, takes a contiguous run, and never
//! steps over a live claim or a hole. That is why the durable watermark can
//! be read as "everything up to here is applied" without a per-row gap scan,
//! and why an inconsistent frontier **blocks** rather than being skipped.

use std::borrow::Cow;
use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::sql_types::{
    Array, BigInt, Bool, Date, Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::{Connection, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::{
    MetricRollupDeltaClaim, MetricRollupWatermark, METRIC_ROLLUP_CLAIM_MAX_BATCH,
    METRIC_ROLLUP_CLAIM_MIN_BATCH, METRIC_ROLLUP_DELTA_APPLIED, METRIC_ROLLUP_DELTA_CLAIMED,
    METRIC_ROLLUP_DELTA_PENDING, METRIC_ROLLUP_LEASE_SECONDS,
};
use crate::db::PgPool;
use crate::model::Timestamp;

/// The one canonical grain the MOM-1 progress stream covers.
const WORK_DAY_GRAIN: &str = "DAY";

/// A rejection the caller can act on, rendered verbatim.
///
/// `thoth-errors` is outside this task's approved write budget, so no
/// dedicated stable `extensions.type` is introduced for rollup rejections and
/// these surface as `INTERNAL_ERROR` with an exact message. The messages are
/// fixed and carry no token, principal, canonical value or database detail.
fn rejected(message: &'static str) -> ThothError {
    ThothError::DatabaseConstraintError(Cow::Borrowed(message))
}

/// An invariant the database is supposed to make impossible.
///
/// Reached only if a row violates the closed lifecycle constraint, so it
/// fails the transaction closed instead of unwrapping.
fn broken_invariant(message: &'static str) -> ThothError {
    ThothError::InternalError(message.to_string())
}

// ---------------------------------------------------------------------------
// Row shapes
// ---------------------------------------------------------------------------

/// The singleton progress row, read under `FOR UPDATE`.
#[derive(diesel::QueryableByName)]
struct StateRow {
    #[diesel(sql_type = BigInt)]
    next_sequence: i64,
    #[diesel(sql_type = BigInt)]
    applied_through_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    watermark_at: Timestamp,
}

/// One candidate row of the claim scan.
#[derive(diesel::QueryableByName)]
struct FrontierRow {
    #[diesel(sql_type = SqlUuid)]
    delta_id: Uuid,
    #[diesel(sql_type = Nullable<BigInt>)]
    work_day_sequence: Option<i64>,
    #[diesel(sql_type = Text)]
    status: String,
    /// Computed in SQL against `transaction_timestamp()` rather than compared
    /// in Rust, so "expired" is decided by the same clock and the same
    /// transaction snapshot that the claim update writes under.
    #[diesel(sql_type = Bool)]
    lease_is_live: bool,
}

/// One row returned by the claim update.
#[derive(diesel::QueryableByName)]
struct ClaimedRow {
    #[diesel(sql_type = SqlUuid)]
    delta_id: Uuid,
    #[diesel(sql_type = Nullable<BigInt>)]
    work_day_sequence: Option<i64>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    claim_token: Option<Uuid>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    lease_expires_at: Option<Timestamp>,
}

/// One claimed delta joined to the canonical record that supplies its
/// projection dimensions.
#[derive(diesel::QueryableByName)]
struct BatchRow {
    #[diesel(sql_type = Nullable<BigInt>)]
    work_day_sequence: Option<i64>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<Text>)]
    claimed_by: Option<String>,
    #[diesel(sql_type = Bool)]
    lease_is_live: bool,
    #[diesel(sql_type = BigInt)]
    delta_value: i64,
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    publication_id: Option<Uuid>,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    period_start: NaiveDate,
    #[diesel(sql_type = Date)]
    period_end: NaiveDate,
    #[diesel(sql_type = Text)]
    reporting_grain: String,
    #[diesel(sql_type = Nullable<Text>)]
    country_code: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    institution_id: Option<Uuid>,
}

/// The current value of one projection row, read under `FOR UPDATE`.
#[derive(diesel::QueryableByName)]
struct ProjectionValueRow {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

/// The state row as it stands after a completion.
#[derive(diesel::QueryableByName)]
struct WatermarkRow {
    #[diesel(sql_type = BigInt)]
    applied_through_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    watermark_at: Timestamp,
}

/// The greatest work-day watermark among the day rows that feed a monthly
/// recomputation, or `None` when no such row exists.
#[derive(diesel::QueryableByName)]
struct SourceWatermarkRow {
    #[diesel(sql_type = Nullable<BigInt>)]
    watermark: Option<i64>,
}

// ---------------------------------------------------------------------------
// claimMetricRollupDeltas
// ---------------------------------------------------------------------------

/// Claim the contiguous run of work-day rollup deltas that begins at the
/// durable frontier.
///
/// One transaction on one connection performs, in order:
///
/// 1. lock the singleton progress row `FOR UPDATE`, which both fixes the
///    frontier for this transaction and excludes every concurrent claim,
///    completion and work-day sequence allocation;
/// 2. read at most `limit` rows starting at `applied_through_sequence + 1`,
///    in ascending sequence, `FOR UPDATE`;
/// 3. update the eligible contiguous prefix to `CLAIMED` under one fresh
///    batch token and one common lease expiry.
///
/// The scan stops — it never skips — at the first hole, at the first live
/// foreign claim, and at any row whose durable state is inconsistent with the
/// frontier. Stopping at position `F` itself returns an empty result, which
/// is the correct answer for "another worker currently holds the frontier",
/// not an error.
///
/// `claimant` is the authenticated machine identity derived by the resolver,
/// never a caller-supplied value: it is what makes a foreign completion
/// detectable. `limit` outside `1..=50` is rejected before the transaction
/// opens, so a malformed request performs no database work at all.
pub(crate) fn claim_metric_rollup_deltas(
    db: &PgPool,
    claimant: &str,
    limit: i32,
) -> ThothResult<Vec<MetricRollupDeltaClaim>> {
    if !(METRIC_ROLLUP_CLAIM_MIN_BATCH..=METRIC_ROLLUP_CLAIM_MAX_BATCH).contains(&limit) {
        return Err(rejected(
            "A rollup delta claim limit must be between 1 and 50 inclusive.",
        ));
    }
    if claimant.trim().is_empty() {
        return Err(broken_invariant(
            "an authenticated rollup claimant has no identity",
        ));
    }
    let claimant = claimant.to_string();

    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let state = lock_state(connection)?;
        let frontier = state.applied_through_sequence + 1;

        // The scan window is the frontier run the caller asked for. A row
        // outside it is irrelevant: MOM-1 never applies out of order, so a
        // later sequence cannot become eligible by skipping an earlier one.
        let candidates: Vec<FrontierRow> = diesel::sql_query(
            "SELECT delta_id, work_day_sequence, status, \
                    (lease_expires_at IS NOT NULL \
                     AND lease_expires_at > transaction_timestamp()) AS lease_is_live \
             FROM public.metric_rollup_delta \
             WHERE work_day_sequence >= $1 \
               AND work_day_sequence <= $2 \
             ORDER BY work_day_sequence ASC \
             FOR UPDATE",
        )
        .bind::<BigInt, _>(frontier)
        .bind::<BigInt, _>(frontier + i64::from(limit) - 1)
        .load(connection)?;

        // Zipped against the ascending run the frontier demands, so "the
        // sequence we are standing on" is derived from the frontier rather
        // than tracked separately and able to drift from it.
        let mut eligible: Vec<Uuid> = Vec::new();
        for (expected, row) in (frontier..).zip(candidates.iter()) {
            let Some(sequence) = row.work_day_sequence else {
                return Err(broken_invariant(
                    "a work-day rollup delta was selected without a sequence",
                ));
            };
            if sequence != expected {
                // A hole in a gap-free ordering means the frontier cannot be
                // proven complete. Stop; do not reach across it.
                break;
            }
            match row.status.as_str() {
                METRIC_ROLLUP_DELTA_PENDING => {}
                METRIC_ROLLUP_DELTA_CLAIMED if !row.lease_is_live => {
                    // An expired lease is reclaimable. The update below
                    // overwrites the stale token, which is what stops the
                    // previous claimant from completing it.
                }
                // A live claim, or an APPLIED row at or above the frontier,
                // both mean this position is not ours to take. The APPLIED
                // case is durable evidence of an inconsistency, because the
                // frontier should already have moved past it; MOM-1 blocks on
                // it rather than advancing over data whose effect cannot be
                // accounted for.
                _ => break,
            }
            eligible.push(row.delta_id);
        }

        if eligible.is_empty() {
            return Ok(Vec::new());
        }

        let claim_token = Uuid::new_v4();
        let claimed: Vec<ClaimedRow> = diesel::sql_query(
            "UPDATE public.metric_rollup_delta \
             SET status = 'CLAIMED', \
                 claim_token = $1, \
                 claimed_by = $2, \
                 claimed_at = transaction_timestamp(), \
                 lease_expires_at = transaction_timestamp() \
                     + ($3 * interval '1 second') \
             WHERE delta_id = ANY($4) \
             RETURNING delta_id, work_day_sequence, claim_token, lease_expires_at",
        )
        .bind::<SqlUuid, _>(claim_token)
        .bind::<Text, _>(&claimant)
        .bind::<Integer, _>(METRIC_ROLLUP_LEASE_SECONDS)
        .bind::<Array<SqlUuid>, _>(&eligible)
        .load(connection)?;

        if claimed.len() != eligible.len() {
            return Err(broken_invariant(
                "a locked rollup delta disappeared during its own claim",
            ));
        }

        let mut claims: Vec<MetricRollupDeltaClaim> = claimed
            .into_iter()
            .map(|row| {
                Ok(MetricRollupDeltaClaim {
                    delta_id: row.delta_id,
                    work_day_sequence: row.work_day_sequence.ok_or_else(|| {
                        broken_invariant("a claimed rollup delta has no sequence")
                    })?,
                    claim_token: row.claim_token.ok_or_else(|| {
                        broken_invariant("a claimed rollup delta has no claim token")
                    })?,
                    lease_expires_at: row
                        .lease_expires_at
                        .ok_or_else(|| broken_invariant("a claimed rollup delta has no lease"))?,
                })
            })
            .collect::<ThothResult<Vec<_>>>()?;
        claims.sort_by_key(|claim| claim.work_day_sequence);
        Ok(claims)
    })
}

// ---------------------------------------------------------------------------
// completeMetricRollupDeltas
// ---------------------------------------------------------------------------

/// Apply one whole claimed batch and advance the durable watermark.
///
/// One transaction on one connection performs, in order:
///
/// 1. lock the singleton progress row `FOR UPDATE`;
/// 2. lock every delta carrying the supplied token, in ascending sequence;
/// 3. validate claimant identity, live lease, and that the batch begins at
///    exactly `applied_through_sequence + 1` and is consecutive;
/// 4. apply each delta's signed value to `metric_rollup_work_day`, in
///    ascending sequence, using dimensions read from the canonical record;
/// 5. derive the distinct `(work, platform, measure, month)` keys the batch
///    touched and recompute the four derived monthly datasets for exactly
///    those keys, set-wise, in a fixed number of statements (nine: one
///    source watermark bound, four deletes, four inserts) that does not grow
///    with the number of keys;
/// 6. terminalize every batch row as `APPLIED`, closing the lease while
///    retaining its token, owner and claim time as evidence;
/// 7. set `applied_through_sequence` to the batch's last sequence and stamp
///    the watermark.
///
/// It is **all rows or none**: any validation, arithmetic, constraint or
/// statement failure — in the day application, in the monthly
/// recomputation, in terminalization or in the frontier advance — rolls the
/// whole transaction back, leaving the work-day projection, the monthly
/// projections, every delta's status and the watermark exactly as they were,
/// with the batch still claimed until its lease expires.
///
/// A repeat of an already-applied token is read-only and returns the current
/// durable watermark, which is what makes a completion that timed out *after*
/// committing safe to retry. A token that names nothing, a batch a different
/// principal claimed, a batch that has since been reclaimed, and a live batch
/// that no longer begins at the frontier all fail without mutation.
pub(crate) fn complete_metric_rollup_deltas(
    db: &PgPool,
    claimant: &str,
    claim_token: Uuid,
) -> ThothResult<MetricRollupWatermark> {
    if claimant.trim().is_empty() {
        return Err(broken_invariant(
            "an authenticated rollup claimant has no identity",
        ));
    }

    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let state = lock_state(connection)?;

        let batch: Vec<BatchRow> = diesel::sql_query(
            "SELECT d.work_day_sequence, d.status, d.claimed_by, \
                    (d.lease_expires_at IS NOT NULL \
                     AND d.lease_expires_at > transaction_timestamp()) AS lease_is_live, \
                    d.delta_value, \
                    r.work_id, r.publication_id, r.platform_id, r.measure_id, \
                    r.period_start, r.period_end, \
                    r.reporting_grain::text AS reporting_grain, \
                    r.country_code::text AS country_code, r.institution_id \
             FROM public.metric_rollup_delta d \
             JOIN public.metric_record r ON r.record_id = d.record_id \
             WHERE d.claim_token = $1 \
             ORDER BY d.work_day_sequence ASC \
             FOR UPDATE OF d",
        )
        .bind::<SqlUuid, _>(claim_token)
        .load(connection)?;

        if batch.is_empty() {
            return Err(rejected(
                "This rollup claim token names no claimed delta. It is stale, \
                 already superseded by a reclaim, or was never issued.",
            ));
        }
        // Ownership is checked before anything else about the batch, so a
        // foreign principal learns nothing about its state.
        if batch
            .iter()
            .any(|row| row.claimed_by.as_deref() != Some(claimant))
        {
            return Err(rejected(
                "This rollup claim token belongs to a different claimant.",
            ));
        }

        let applied = batch
            .iter()
            .filter(|row| row.status == METRIC_ROLLUP_DELTA_APPLIED)
            .count();
        if applied == batch.len() {
            return replay(&state, &batch);
        }
        if applied > 0
            || batch
                .iter()
                .any(|row| row.status != METRIC_ROLLUP_DELTA_CLAIMED)
        {
            // A batch that is neither wholly live nor wholly applied has been
            // partially superseded. Applying the remainder would double count
            // or leave a hole, so it fails and waits for repair.
            return Err(rejected(
                "This rollup claim batch is no longer in one consistent state \
                 and cannot be completed.",
            ));
        }
        if batch.iter().any(|row| !row.lease_is_live) {
            return Err(rejected(
                "This rollup claim lease has expired. The batch is reclaimable \
                 by a fresh claim, and this token can no longer apply it.",
            ));
        }

        let sequences = batch_sequences(&batch)?;
        let first = sequences[0];
        let last = sequences[sequences.len() - 1];
        if first != state.applied_through_sequence + 1 {
            return Err(rejected(
                "This rollup claim batch no longer begins at the durable \
                 frontier and cannot be applied.",
            ));
        }
        if last - first + 1 != sequences.len() as i64 {
            return Err(broken_invariant(
                "a claimed rollup batch is not consecutive",
            ));
        }
        if last >= state.next_sequence {
            return Err(broken_invariant(
                "a claimed rollup delta holds a sequence that was never allocated",
            ));
        }

        // Step 4. Every delta, in ascending sequence, one at a time. The batch
        // is bounded to 50, so this is a bounded number of statements and not
        // an unbounded loop; deltas are deliberately not pre-aggregated,
        // because each row's own sequence is what the projection records.
        for (row, sequence) in batch.iter().zip(sequences.iter().copied()) {
            apply_delta(connection, row, sequence)?;
        }

        // Step 5. The derived monthly serving layer (MET-WP4-03A). Every
        // distinct month key the batch touched is recomputed from the
        // work-day projection as it now stands, for the whole key set at
        // once. A failure here fails the transaction, so the day updates
        // above, the terminalization and the frontier advance below all roll
        // back together; nothing is partially completed.
        let affected = affected_month_keys(&batch)?;
        recompute_month_projections(connection, &affected, last)?;

        // Step 6. Terminalize. `status = 'CLAIMED'` in the predicate is
        // redundant under the locks already held and is kept as a fail-closed
        // assertion: a mismatch in the affected count aborts the batch.
        let terminalized = diesel::sql_query(
            "UPDATE public.metric_rollup_delta \
             SET status = 'APPLIED', \
                 applied_at = transaction_timestamp(), \
                 lease_expires_at = NULL \
             WHERE claim_token = $1 \
               AND status = 'CLAIMED'",
        )
        .bind::<SqlUuid, _>(claim_token)
        .execute(connection)?;
        if terminalized != batch.len() {
            return Err(broken_invariant(
                "a claimed rollup delta changed state during its own completion",
            ));
        }

        // Step 7. The frontier moves to exactly the batch's last sequence and
        // no further, so it can never cross a delta this transaction did not
        // apply.
        let watermark: Vec<WatermarkRow> = diesel::sql_query(
            "UPDATE public.metric_rollup_work_day_state \
             SET applied_through_sequence = $1, \
                 watermark_at = transaction_timestamp(), \
                 updated_at = transaction_timestamp() \
             WHERE state_id = 1 \
             RETURNING applied_through_sequence, watermark_at",
        )
        .bind::<BigInt, _>(last)
        .load(connection)?;
        let Some(watermark) = watermark.into_iter().next() else {
            return Err(broken_invariant(
                "the rollup progress state row disappeared during completion",
            ));
        };

        Ok(MetricRollupWatermark {
            applied_through_sequence: watermark.applied_through_sequence,
            watermark_at: watermark.watermark_at,
        })
    })
}

// ---------------------------------------------------------------------------
// Shared steps
// ---------------------------------------------------------------------------

/// Lock the singleton progress row for the rest of the transaction.
///
/// This is the serialization point of the whole protocol. Holding it excludes
/// every other claim, every other completion and — because the allocation
/// trigger updates the same row — every concurrent work-day sequence
/// allocation, which is what lets the frontier be read and advanced without a
/// separate consistency scan.
fn lock_state(connection: &mut PgConnection) -> ThothResult<StateRow> {
    let rows: Vec<StateRow> = diesel::sql_query(
        "SELECT next_sequence, applied_through_sequence, watermark_at \
         FROM public.metric_rollup_work_day_state \
         WHERE state_id = 1 \
         FOR UPDATE",
    )
    .load(connection)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| broken_invariant("the rollup progress state row is missing"))
}

/// The sequence of every row of a claimed batch, in ascending order.
fn batch_sequences(batch: &[BatchRow]) -> ThothResult<Vec<i64>> {
    batch
        .iter()
        .map(|row| {
            row.work_day_sequence
                .ok_or_else(|| broken_invariant("a claimed rollup delta has no sequence"))
        })
        .collect()
}

/// Add one signed delta to its logical aggregate row.
///
/// The aggregate is located by the exact approved identity, with
/// `IS NOT DISTINCT FROM` on the three optional dimensions so an absent
/// publication, country or institution matches the one row that also has it
/// absent, rather than matching nothing and splitting the total across
/// duplicates. That mirrors the `UNIQUE NULLS NOT DISTINCT` constraint the
/// upsert resolves against.
///
/// Arithmetic is checked twice and fails closed both times: `checked_add`
/// rejects an overflowing total in Rust with an exact message, and
/// PostgreSQL's own `BIGINT` arithmetic remains the backstop that no code
/// path can bypass. Neither can wrap, so an overflowing delta aborts the
/// whole batch instead of silently recording a sign-flipped total.
fn apply_delta(connection: &mut PgConnection, row: &BatchRow, sequence: i64) -> ThothResult<()> {
    // The dimensions come from the canonical record, never from the request.
    // A delta whose record is not a valid one-day DAY record must not be in
    // the work-day stream at all, so its presence is an inconsistency that
    // blocks the frontier.
    if row.reporting_grain != WORK_DAY_GRAIN || row.period_start.succ_opt() != Some(row.period_end)
    {
        return Err(rejected(
            "A claimed rollup delta does not describe exactly one calendar \
             day. The work-day frontier is blocked pending repair.",
        ));
    }
    let day = row.period_start;

    let existing: Vec<ProjectionValueRow> = diesel::sql_query(
        "SELECT value \
         FROM public.metric_rollup_work_day \
         WHERE work_id = $1 \
           AND publication_id IS NOT DISTINCT FROM $2 \
           AND platform_id = $3 \
           AND measure_id = $4 \
           AND day = $5 \
           AND country_code IS NOT DISTINCT FROM $6 \
           AND institution_id IS NOT DISTINCT FROM $7 \
         FOR UPDATE",
    )
    .bind::<SqlUuid, _>(row.work_id)
    .bind::<Nullable<SqlUuid>, _>(row.publication_id)
    .bind::<SqlUuid, _>(row.platform_id)
    .bind::<SqlUuid, _>(row.measure_id)
    .bind::<Date, _>(day)
    .bind::<Nullable<Text>, _>(row.country_code.as_deref())
    .bind::<Nullable<SqlUuid>, _>(row.institution_id)
    .load(connection)?;

    if let Some(current) = existing.into_iter().next() {
        current.value.checked_add(row.delta_value).ok_or_else(|| {
            rejected(
                "Applying this rollup delta would overflow the projected \
                 total. The work-day frontier is blocked pending repair.",
            )
        })?;
    }

    diesel::sql_query(
        "INSERT INTO public.metric_rollup_work_day \
             (work_id, publication_id, platform_id, measure_id, day, \
              country_code, institution_id, value, watermark) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT ON CONSTRAINT metric_rollup_work_day_identity_key \
         DO UPDATE SET value = public.metric_rollup_work_day.value \
                             + EXCLUDED.value, \
                       watermark = EXCLUDED.watermark",
    )
    .bind::<SqlUuid, _>(row.work_id)
    .bind::<Nullable<SqlUuid>, _>(row.publication_id)
    .bind::<SqlUuid, _>(row.platform_id)
    .bind::<SqlUuid, _>(row.measure_id)
    .bind::<Date, _>(day)
    .bind::<Nullable<Text>, _>(row.country_code.as_deref())
    .bind::<Nullable<SqlUuid>, _>(row.institution_id)
    .bind::<BigInt, _>(row.delta_value)
    .bind::<BigInt, _>(sequence)
    .execute(connection)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// MET-WP4-03A: derived monthly serving projections
// ---------------------------------------------------------------------------
//
// The four monthly tables are derived exclusively from
// `metric_rollup_work_day`. Each statement below starts from a relation
// `affected` of `(work_id, platform_id, measure_id, month_start)` keys and
// resolves every daily base cell of those months FIRST — the `MET-WP4-02`
// Amendment 6 total rule for totals, the unique-least target-dimension rule
// for country and institution — and only then sums the resolved daily
// contributions into the month. Raw day rows are never compacted into a
// month and re-resolved there.
//
// A day row's presence mask is the `MET-WP4-02` mask: publication = 4,
// country = 2, institution = 1, so an undimensioned row is 0. A window
// aggregate `bit_or(1 << mask) OVER (PARTITION BY base cell)` gives every
// row the bitmap of masks its own cell represents, so no row ever has to be
// joined back to its cell (a self-join over statistics-less CTE scans was
// measured at 300 ms per statement for fifty keys and is exactly what this
// shape avoids), and one CASE per target derives the selected mask (or NULL)
// from that bitmap:
//
//   total       0 if represented, else the single represented mask, else
//               NULL = total_ambiguous;
//   country     the least of {2, 3, 6, 7} under inclusion: 2 if present,
//               else NULL when both 3 and 6 (incomparable minima) are present
//               = country_ambiguous, else 3, else 6, else 7, else NULL = no
//               country contribution;
//   institution the mirror over {1, 3, 5, 7}: 1, else NULL when both 3 and 5
//               are present = institution_ambiguous, else 3, else 5, else 7,
//               else NULL.
//
// The rows carrying the selected mask are then kept and summed with
// `publication_id` as the row holds it: the selected representation
// either carries a publication on every row or on none, so retaining
// `r.publication_id` is exactly "publication only if the representation
// contains it", and rows of the same month with and without a publication
// are additive contributions from different days. `SUM(bigint)` is exact
// `numeric`; the cast back to `bigint` is what fails closed on overflow.
//
// The statements receive the affected keys as four parallel arrays through
// `unnest`, so one statement serves one key or fifty, and the statement count
// is fixed at nine whatever the key count. The test-only rebuild at the end
// of this file replays exactly these statements over every represented month
// key, in chunks bounded like a claim batch.

/// The distinct affected month keys of one completion, from four parallel
/// arrays.
macro_rules! affected_month_keys_sql {
    () => {
        "SELECT DISTINCT k.work_id, k.platform_id, k.measure_id, k.month_start \
         FROM unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
              AS k(work_id, platform_id, measure_id, month_start)"
    };
}

/// The per-cell resolution shared by every monthly statement, as the CTEs
/// that follow an `affected` relation.
macro_rules! month_resolution_ctes_sql {
    () => {
        "day_rows AS ( \
             SELECT x.*, \
                    bit_or(1 << x.mask) OVER (PARTITION BY x.work_id, x.platform_id, \
                                                           x.measure_id, x.day) AS represented \
             FROM ( \
                 SELECT r.work_id, r.publication_id, r.platform_id, r.measure_id, r.day, \
                        a.month_start, r.country_code, r.institution_id, r.value, r.watermark, \
                        (r.publication_id IS NOT NULL)::int * 4 \
                            + (r.country_code IS NOT NULL)::int * 2 \
                            + (r.institution_id IS NOT NULL)::int AS mask \
                 FROM affected a \
                 JOIN public.metric_rollup_work_day r \
                   ON r.work_id = a.work_id \
                  AND r.platform_id = a.platform_id \
                  AND r.measure_id = a.measure_id \
                  AND r.day >= a.month_start \
                  AND r.day < (a.month_start + interval '1 month')::date \
             ) x \
         ), \
         resolved AS ( \
             SELECT day_rows.*, \
                    CASE WHEN represented & 1 <> 0 THEN 0 \
                         WHEN represented = 2 THEN 1 \
                         WHEN represented = 4 THEN 2 \
                         WHEN represented = 8 THEN 3 \
                         WHEN represented = 16 THEN 4 \
                         WHEN represented = 32 THEN 5 \
                         WHEN represented = 64 THEN 6 \
                         WHEN represented = 128 THEN 7 \
                    END AS total_mask, \
                    CASE WHEN represented & 4 <> 0 THEN 2 \
                         WHEN represented & 8 <> 0 AND represented & 64 <> 0 THEN NULL \
                         WHEN represented & 8 <> 0 THEN 3 \
                         WHEN represented & 64 <> 0 THEN 6 \
                         WHEN represented & 128 <> 0 THEN 7 \
                    END AS country_mask, \
                    (represented & 4 = 0 AND represented & 8 <> 0 AND represented & 64 <> 0) \
                        AS country_ambiguous, \
                    CASE WHEN represented & 2 <> 0 THEN 1 \
                         WHEN represented & 8 <> 0 AND represented & 32 <> 0 THEN NULL \
                         WHEN represented & 8 <> 0 THEN 3 \
                         WHEN represented & 32 <> 0 THEN 5 \
                         WHEN represented & 128 <> 0 THEN 7 \
                    END AS institution_mask, \
                    (represented & 2 = 0 AND represented & 8 <> 0 AND represented & 32 <> 0) \
                        AS institution_ambiguous \
             FROM day_rows \
         )"
    };
}

/// The greatest day watermark feeding the affected months.
macro_rules! source_watermark_sql {
    () => {
        "SELECT MAX(r.watermark) AS watermark \
         FROM affected a \
         JOIN public.metric_rollup_work_day r \
           ON r.work_id = a.work_id \
          AND r.platform_id = a.platform_id \
          AND r.measure_id = a.measure_id \
          AND r.day >= a.month_start \
          AND r.day < (a.month_start + interval '1 month')::date"
    };
}

macro_rules! month_total_insert_sql {
    () => {
        "INSERT INTO public.metric_rollup_work_month \
             (work_id, publication_id, platform_id, measure_id, month_start, value, \
              requires_country_coverage, requires_institution_coverage, watermark) \
         SELECT r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start, \
                SUM(r.value)::bigint, \
                bool_or(r.mask & 2 <> 0), \
                bool_or(r.mask & 1 <> 0), \
                MAX(r.watermark) \
         FROM resolved r \
         WHERE r.mask = r.total_mask \
         GROUP BY r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start"
    };
}

macro_rules! month_country_insert_sql {
    () => {
        "INSERT INTO public.metric_rollup_work_country_month \
             (work_id, publication_id, platform_id, measure_id, month_start, country_code, \
              value, requires_institution_coverage, watermark) \
         SELECT r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start, \
                r.country_code, \
                SUM(r.value)::bigint, \
                bool_or(r.mask & 1 <> 0), \
                MAX(r.watermark) \
         FROM resolved r \
         WHERE r.mask = r.country_mask \
         GROUP BY r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start, \
                  r.country_code"
    };
}

macro_rules! month_institution_insert_sql {
    () => {
        "INSERT INTO public.metric_rollup_work_institution_month \
             (work_id, publication_id, platform_id, measure_id, month_start, institution_id, \
              value, requires_country_coverage, watermark) \
         SELECT r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start, \
                r.institution_id, \
                SUM(r.value)::bigint, \
                bool_or(r.mask & 2 <> 0), \
                MAX(r.watermark) \
         FROM resolved r \
         WHERE r.mask = r.institution_mask \
         GROUP BY r.work_id, r.publication_id, r.platform_id, r.measure_id, r.month_start, \
                  r.institution_id"
    };
}

/// Sparse ambiguity: one row per month key with at least one true flag. The
/// watermark is taken over exactly the rows that establish a true flag: every
/// row of a total-ambiguous cell, and the rows carrying the incomparable
/// minimal masks (3 and 6 for country, 3 and 5 for institution) of a
/// country- or institution-ambiguous cell.
macro_rules! month_ambiguity_insert_sql {
    () => {
        "INSERT INTO public.metric_rollup_work_month_ambiguity \
             (work_id, platform_id, measure_id, month_start, total_ambiguous, \
              country_ambiguous, institution_ambiguous, watermark) \
         SELECT r.work_id, r.platform_id, r.measure_id, r.month_start, \
                bool_or(r.total_mask IS NULL), \
                bool_or(r.country_ambiguous), \
                bool_or(r.institution_ambiguous), \
                MAX(r.watermark) FILTER (WHERE r.total_mask IS NULL \
                    OR (r.country_ambiguous AND r.mask IN (3, 6)) \
                    OR (r.institution_ambiguous AND r.mask IN (3, 5))) \
         FROM resolved r \
         GROUP BY r.work_id, r.platform_id, r.measure_id, r.month_start \
         HAVING bool_or(r.total_mask IS NULL OR r.country_ambiguous OR r.institution_ambiguous)"
    };
}

/// Statement 1 of 9: the greatest work-day watermark feeding the affected
/// months, checked against the frontier the completion is about to
/// establish.
pub(crate) const MONTH_SOURCE_WATERMARK_SQL: &str = concat!(
    "WITH affected AS (",
    affected_month_keys_sql!(),
    ") ",
    source_watermark_sql!()
);

/// Statements 2 to 5 of 9: clear every row of the affected keys, whatever
/// its publication, country or institution.
pub(crate) const MONTH_TOTAL_DELETE_SQL: &str = "DELETE FROM public.metric_rollup_work_month m \
     USING unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
           AS k(work_id, platform_id, measure_id, month_start) \
     WHERE m.work_id = k.work_id \
       AND m.platform_id = k.platform_id \
       AND m.measure_id = k.measure_id \
       AND m.month_start = k.month_start";
pub(crate) const MONTH_COUNTRY_DELETE_SQL: &str =
    "DELETE FROM public.metric_rollup_work_country_month m \
     USING unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
           AS k(work_id, platform_id, measure_id, month_start) \
     WHERE m.work_id = k.work_id \
       AND m.platform_id = k.platform_id \
       AND m.measure_id = k.measure_id \
       AND m.month_start = k.month_start";
pub(crate) const MONTH_INSTITUTION_DELETE_SQL: &str =
    "DELETE FROM public.metric_rollup_work_institution_month m \
     USING unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
           AS k(work_id, platform_id, measure_id, month_start) \
     WHERE m.work_id = k.work_id \
       AND m.platform_id = k.platform_id \
       AND m.measure_id = k.measure_id \
       AND m.month_start = k.month_start";
pub(crate) const MONTH_AMBIGUITY_DELETE_SQL: &str =
    "DELETE FROM public.metric_rollup_work_month_ambiguity m \
     USING unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
           AS k(work_id, platform_id, measure_id, month_start) \
     WHERE m.work_id = k.work_id \
       AND m.platform_id = k.platform_id \
       AND m.measure_id = k.measure_id \
       AND m.month_start = k.month_start";

/// Statements 6 to 9 of 9: the recomputed rows of the affected keys.
pub(crate) const MONTH_TOTAL_INSERT_SQL: &str = concat!(
    "WITH affected AS (",
    affected_month_keys_sql!(),
    "), ",
    month_resolution_ctes_sql!(),
    " ",
    month_total_insert_sql!()
);
pub(crate) const MONTH_COUNTRY_INSERT_SQL: &str = concat!(
    "WITH affected AS (",
    affected_month_keys_sql!(),
    "), ",
    month_resolution_ctes_sql!(),
    " ",
    month_country_insert_sql!()
);
pub(crate) const MONTH_INSTITUTION_INSERT_SQL: &str = concat!(
    "WITH affected AS (",
    affected_month_keys_sql!(),
    "), ",
    month_resolution_ctes_sql!(),
    " ",
    month_institution_insert_sql!()
);
pub(crate) const MONTH_AMBIGUITY_INSERT_SQL: &str = concat!(
    "WITH affected AS (",
    affected_month_keys_sql!(),
    "), ",
    month_resolution_ctes_sql!(),
    " ",
    month_ambiguity_insert_sql!()
);

/// The four deletes, in execution order.
pub(crate) const MONTH_DELETE_STATEMENTS: [&str; 4] = [
    MONTH_TOTAL_DELETE_SQL,
    MONTH_COUNTRY_DELETE_SQL,
    MONTH_INSTITUTION_DELETE_SQL,
    MONTH_AMBIGUITY_DELETE_SQL,
];

/// The four inserts, in execution order.
pub(crate) const MONTH_INSERT_STATEMENTS: [&str; 4] = [
    MONTH_TOTAL_INSERT_SQL,
    MONTH_COUNTRY_INSERT_SQL,
    MONTH_INSTITUTION_INSERT_SQL,
    MONTH_AMBIGUITY_INSERT_SQL,
];

/// Every statement the monthly maintenance executes, in order: the source
/// watermark bound, the four deletes and the four inserts. The count is a
/// property of the design, not of the batch: it is the same for one affected
/// key and for fifty, which the tests prove by capturing the statements one
/// completion actually issues.
#[cfg(test)]
pub(crate) const MONTH_MAINTENANCE_STATEMENTS: [&str; 9] = [
    MONTH_SOURCE_WATERMARK_SQL,
    MONTH_TOTAL_DELETE_SQL,
    MONTH_COUNTRY_DELETE_SQL,
    MONTH_INSTITUTION_DELETE_SQL,
    MONTH_AMBIGUITY_DELETE_SQL,
    MONTH_TOTAL_INSERT_SQL,
    MONTH_COUNTRY_INSERT_SQL,
    MONTH_INSTITUTION_INSERT_SQL,
    MONTH_AMBIGUITY_INSERT_SQL,
];

/// The fixed number of SQL statements the monthly maintenance adds to one
/// completion: nine.
#[cfg(test)]
pub(crate) const MONTH_MAINTENANCE_STATEMENT_COUNT: usize = MONTH_MAINTENANCE_STATEMENTS.len();

/// PostgreSQL's fixed message for a `numeric` value that does not fit a
/// `bigint` (SQLSTATE `22003`). Diesel exposes no SQLSTATE, so the message
/// is the discriminator; any other database failure propagates unchanged.
const BIGINT_OUT_OF_RANGE: &str = "bigint out of range";

/// One month key: `(work_id, platform_id, measure_id, month_start)`.
pub(crate) type MonthKey = (Uuid, Uuid, Uuid, NaiveDate);

/// The distinct month keys a validated batch touches.
///
/// Every row has already been checked by [`apply_delta`] to describe exactly
/// one calendar day, so the month is the day's own month.
fn affected_month_keys(batch: &[BatchRow]) -> ThothResult<BTreeSet<MonthKey>> {
    batch
        .iter()
        .map(|row| {
            let month_start = row.period_start.with_day(1).ok_or_else(|| {
                broken_invariant("a work-day rollup delta names a day outside any calendar month")
            })?;
            Ok((row.work_id, row.platform_id, row.measure_id, month_start))
        })
        .collect()
}

/// Recompute the four monthly datasets for exactly `keys`, set-wise.
///
/// Executes the nine [`MONTH_MAINTENANCE_STATEMENTS`] in order, each over the
/// whole key set at once: the source watermark bound, then delete and
/// reinsert per projection. It is not a loop over keys. `frontier` is the
/// `applied_through_sequence` the enclosing completion is about to
/// establish; a day row watermarked above it is evidence of out-of-band
/// damage and fails the transaction closed, because a derived watermark must
/// never exceed the frontier.
pub(crate) fn recompute_month_projections(
    connection: &mut PgConnection,
    keys: &BTreeSet<MonthKey>,
    frontier: i64,
) -> ThothResult<()> {
    let work_ids: Vec<Uuid> = keys.iter().map(|key| key.0).collect();
    let platform_ids: Vec<Uuid> = keys.iter().map(|key| key.1).collect();
    let measure_ids: Vec<Uuid> = keys.iter().map(|key| key.2).collect();
    let month_starts: Vec<NaiveDate> = keys.iter().map(|key| key.3).collect();

    let source: Vec<SourceWatermarkRow> = diesel::sql_query(MONTH_SOURCE_WATERMARK_SQL)
        .bind::<Array<SqlUuid>, _>(&work_ids)
        .bind::<Array<SqlUuid>, _>(&platform_ids)
        .bind::<Array<SqlUuid>, _>(&measure_ids)
        .bind::<Array<Date>, _>(&month_starts)
        .load(connection)?;
    if source
        .into_iter()
        .next()
        .and_then(|row| row.watermark)
        .is_some_and(|watermark| watermark > frontier)
    {
        return Err(broken_invariant(
            "a work-day projection row is watermarked above the frontier this completion establishes",
        ));
    }

    for statement in MONTH_DELETE_STATEMENTS {
        diesel::sql_query(statement)
            .bind::<Array<SqlUuid>, _>(&work_ids)
            .bind::<Array<SqlUuid>, _>(&platform_ids)
            .bind::<Array<SqlUuid>, _>(&measure_ids)
            .bind::<Array<Date>, _>(&month_starts)
            .execute(connection)?;
    }
    for statement in MONTH_INSERT_STATEMENTS {
        diesel::sql_query(statement)
            .bind::<Array<SqlUuid>, _>(&work_ids)
            .bind::<Array<SqlUuid>, _>(&platform_ids)
            .bind::<Array<SqlUuid>, _>(&measure_ids)
            .bind::<Array<Date>, _>(&month_starts)
            .execute(connection)
            .map_err(month_arithmetic)?;
    }
    Ok(())
}

/// Render a monthly `bigint` overflow as the bounded rollup rejection.
///
/// `SUM(bigint)` is exact `numeric`, so the overflow surfaces at the cast back
/// to `bigint`, from PostgreSQL itself: nothing can wrap or narrow. The
/// transaction has already failed by the time this runs; the mapping only
/// gives the caller the same fixed, detail-free message the day path uses.
fn month_arithmetic(error: DieselError) -> ThothError {
    if let DieselError::DatabaseError(_, info) = &error {
        if info.message() == BIGINT_OUT_OF_RANGE {
            return rejected(
                "Recomputing the monthly projections for this batch would overflow \
                 a projected total. The work-day frontier is blocked pending repair.",
            );
        }
    }
    error.into()
}

// ---------------------------------------------------------------------------
// Full rebuild (reviewed procedure; test-only, no callable surface)
// ---------------------------------------------------------------------------

/// The rebuild's source watermark bound: the whole work-day projection.
#[cfg(test)]
pub(crate) const REBUILD_SOURCE_WATERMARK_SQL: &str =
    "SELECT MAX(r.watermark) AS watermark FROM public.metric_rollup_work_day r";

/// Fresh derived state for the rebuild: one `TRUNCATE`, not keyed deletes,
/// so a production-shaped rebuild leaves no dead-tuple bloat.
#[cfg(test)]
pub(crate) const REBUILD_TRUNCATE_SQL: &str = "TRUNCATE TABLE \
         public.metric_rollup_work_month, \
         public.metric_rollup_work_country_month, \
         public.metric_rollup_work_institution_month, \
         public.metric_rollup_work_month_ambiguity";

/// Every month key represented in the work-day projection, in a
/// deterministic order.
#[cfg(test)]
pub(crate) const REBUILD_MONTH_KEYS_SQL: &str =
    "SELECT DISTINCT r.work_id, r.platform_id, r.measure_id, \
            date_trunc('month', r.day)::date AS month_start \
     FROM public.metric_rollup_work_day r \
     ORDER BY 1, 2, 3, 4";

/// One represented month key.
#[cfg(test)]
#[derive(diesel::QueryableByName)]
struct MonthKeyRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    month_start: NaiveDate,
}

/// Rebuild all four monthly datasets from `metric_rollup_work_day` alone.
///
/// This is the reviewed deterministic rebuild procedure `MET-WP4-03A`
/// requires as evidence, compiled only for tests: the task adds no callable
/// rebuild operation, and a historical production rebuild is a separately
/// authorized operational action. One transaction on one connection locks
/// the singleton state row `FOR UPDATE` — the same serialization point every
/// completion takes, so no completion can change the work-day projection
/// under the rebuild — checks that no day row is watermarked above the
/// durable frontier, truncates the four tables, reads every represented
/// month key, and then replays [`recompute_month_projections`] over those
/// keys in chunks of at most [`METRIC_ROLLUP_CLAIM_MAX_BATCH`]: exactly the
/// statements, resolution text and index paths a completion uses, so the
/// rebuilt state is by construction what incremental maintenance produces.
/// A rebuild therefore costs a bounded number of keyed recomputations rather
/// than one whole-table plan, and reads no canonical record, revision or
/// delta. Returns the `applied_through_sequence` the rebuilt state
/// corresponds to.
#[cfg(test)]
pub(crate) fn rebuild_month_projections(db: &PgPool) -> ThothResult<i64> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let state = lock_state(connection)?;
        let source: Vec<SourceWatermarkRow> =
            diesel::sql_query(REBUILD_SOURCE_WATERMARK_SQL).load(connection)?;
        if source
            .into_iter()
            .next()
            .and_then(|row| row.watermark)
            .is_some_and(|watermark| watermark > state.applied_through_sequence)
        {
            return Err(broken_invariant(
                "a work-day projection row is watermarked above the durable frontier",
            ));
        }
        diesel::sql_query(REBUILD_TRUNCATE_SQL).execute(connection)?;
        let keys: Vec<MonthKeyRow> = diesel::sql_query(REBUILD_MONTH_KEYS_SQL).load(connection)?;
        for chunk in keys.chunks(METRIC_ROLLUP_CLAIM_MAX_BATCH as usize) {
            let keys: BTreeSet<MonthKey> = chunk
                .iter()
                .map(|key| {
                    (
                        key.work_id,
                        key.platform_id,
                        key.measure_id,
                        key.month_start,
                    )
                })
                .collect();
            recompute_month_projections(connection, &keys, state.applied_through_sequence)?;
        }
        Ok(state.applied_through_sequence)
    })
}

/// The read-only answer to a repeat of an already-applied batch.
///
/// This is the timeout-after-commit case: the completion committed, the
/// response never arrived, and the claimant retried with the same token.
/// Returning the current durable watermark without writing is what makes that
/// retry safe: no day row, monthly row, delta or frontier is touched. It is
/// permitted only when the whole batch is applied, was
/// applied by this same principal, and sits at or below the durable frontier
/// — the last condition being what distinguishes a genuine replay from a
/// token whose effect is not actually reflected in the watermark.
fn replay(state: &StateRow, batch: &[BatchRow]) -> ThothResult<MetricRollupWatermark> {
    let sequences = batch_sequences(batch)?;
    if sequences
        .iter()
        .any(|sequence| *sequence > state.applied_through_sequence)
    {
        return Err(rejected(
            "This rollup claim token is applied above the durable frontier \
             and cannot be replayed.",
        ));
    }
    Ok(MetricRollupWatermark {
        applied_through_sequence: state.applied_through_sequence,
        watermark_at: state.watermark_at,
    })
}
