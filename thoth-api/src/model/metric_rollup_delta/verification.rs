//! Independent consistency verification of the four derived monthly and the
//! two derived yearly projections against `metric_rollup_work_day`
//! (`MET-WP4-03A-OPS-02`, extended by `MET-WP4-03C`).
//!
//! This module is the production counterpart of the reviewed per-day test
//! oracle in `tests.rs`, and it is deliberately **structurally independent of
//! the monthly and yearly writers** in `crud.rs`. It shares no SQL constant,
//! macro or statement with the writers, never calls
//! `recompute_month_projections` or `recompute_year_projections`, never
//! executes a writer-side maintenance statement, never reads a monthly row
//! to derive an expected yearly row, and never proves correctness by
//! rebuilding and then reading the rebuild. Where the writer resolves each
//! daily base cell through a fixed case table over a bitmap of represented
//! masks, the verifier computes the generic rule the oracle computes: the
//! distinct set of represented masks per cell, the Amendment 6 total rule
//! over that set, and for country and institution the unique least
//! represented mask containing the target dimension under set inclusion,
//! found by an anti-join that removes every candidate with a strict subset
//! also represented. The expected yearly families regroup those same
//! resolved daily contributions by calendar year directly from the work-day
//! projection, so the yearly writer's monthly-to-yearly regrouping is checked
//! against a derivation that never saw a monthly row. Two disagreeing
//! implementations of the same approved semantics are what make an exact
//! match evidence.
//!
//! Everything is set-based PostgreSQL over the whole work-day projection: the
//! expected state is derived, the actual tables are read, and the two sides
//! are compared by logical identity inside one statement — one work-day
//! scan, one resolution, six comparisons — that returns only bounded counts.
//! No monthly, yearly or work-day row is ever loaded into Rust memory, and
//! no row identity, value or mismatch detail leaves the database except as a
//! count.
//!
//! The callable operation, [`verify_metric_rollup_months`], runs on one
//! pooled connection in one `READ ONLY`, `REPEATABLE READ` transaction with
//! no row lock, so it sees one coherent snapshot of the frontier, the
//! work-day projection and all six derived tables, and cannot write. Before
//! that snapshot is frozen it takes one table-level `ACCESS SHARE` lock on
//! exactly the four monthly and the two yearly tables, for one reason only:
//! PostgreSQL's `TRUNCATE` is not MVCC-safe, so a snapshot taken before a
//! concurrent rebuild's truncate would otherwise see the tables it had not
//! yet touched as empty. `ACCESS SHARE` conflicts only with
//! `ACCESS EXCLUSIVE`, which is what the rebuild's `TRUNCATE` takes, and not
//! with the `ROW EXCLUSIVE` locks of normal incremental completion, so
//! routine maintenance stays concurrent with verification while an
//! exceptional truncate cannot cross a verifier's snapshot in either
//! direction. The same comparison, [`verify_projections`], is also run twice
//! inside the rebuild transaction in `crud.rs`: before deciding whether
//! anything must be rebuilt, and after rebuilding, where an inexact result
//! rolls the whole rebuild back.

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::sql_types::{BigInt, Date, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::{QueryableByName, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

use super::crud::{broken_invariant, rejected};
use super::{MetricRollupMonthProjectionVerification, MetricRollupMonthVerification};
use crate::db::PgPool;
use crate::model::Timestamp;

// ---------------------------------------------------------------------------
// Frontier and work-day facts
// ---------------------------------------------------------------------------

/// The durable frontier a verification is taken at.
///
/// Read without `FOR UPDATE` by the verification operation, and converted
/// from the locked state row by the rebuild.
#[derive(Debug, Clone, Copy, diesel::QueryableByName)]
pub(crate) struct FrontierSnapshot {
    #[diesel(sql_type = BigInt)]
    pub(crate) next_sequence: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) applied_through_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) watermark_at: Timestamp,
}

/// The state row, read in the current snapshot without locking it.
pub(crate) const FRONTIER_SQL: &str =
    "SELECT next_sequence, applied_through_sequence, watermark_at \
     FROM public.metric_rollup_work_day_state \
     WHERE state_id = 1";

/// Read the singleton frontier row without taking any lock.
pub(crate) fn read_frontier(connection: &mut PgConnection) -> ThothResult<FrontierSnapshot> {
    let rows: Vec<FrontierSnapshot> = diesel::sql_query(FRONTIER_SQL).load(connection)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| broken_invariant("the rollup progress state row is missing"))
}

/// Bounded facts about the whole work-day projection.
#[derive(Debug, Clone, Copy, diesel::QueryableByName)]
pub(crate) struct WorkDayStats {
    /// `None` when the projection is empty.
    #[diesel(sql_type = Nullable<BigInt>)]
    pub(crate) max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    pub(crate) row_count: i64,
    /// The number of distinct `(work, platform, measure, month)` keys
    /// represented by at least one day row.
    #[diesel(sql_type = BigInt)]
    pub(crate) month_key_count: i64,
}

/// The greatest work-day watermark, the row count and the represented
/// month-key count, in one statement.
pub(crate) const WORK_DAY_STATS_SQL: &str =
    "SELECT s.max_watermark, s.row_count, k.month_key_count \
     FROM (SELECT MAX(watermark) AS max_watermark, COUNT(*) AS row_count \
           FROM public.metric_rollup_work_day) s \
     CROSS JOIN (SELECT COUNT(*) AS month_key_count \
                 FROM (SELECT DISTINCT work_id, platform_id, measure_id, \
                              date_trunc('month', day) \
                       FROM public.metric_rollup_work_day) d) k";

/// Read the work-day facts in the current snapshot.
pub(crate) fn work_day_stats(connection: &mut PgConnection) -> ThothResult<WorkDayStats> {
    let rows: Vec<WorkDayStats> = diesel::sql_query(WORK_DAY_STATS_SQL).load(connection)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| broken_invariant("the work-day projection facts could not be read"))
}

/// Fail closed when any work-day row is watermarked above the frontier.
///
/// A derived monthly watermark can never exceed the frontier it was derived
/// under, so a day row above `W` is out-of-band damage that no verification
/// result could describe truthfully; both operations refuse instead.
pub(crate) fn require_work_day_within_frontier(
    frontier: &FrontierSnapshot,
    stats: &WorkDayStats,
) -> ThothResult<()> {
    if stats
        .max_watermark
        .is_some_and(|watermark| watermark > frontier.applied_through_sequence)
    {
        return Err(broken_invariant(
            "a work-day projection row is watermarked above the durable frontier",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The independent expected-state derivation and comparison
// ---------------------------------------------------------------------------

/// The fragments of the one comparison statement, [`VERIFICATION_SQL`], and
/// of its range form, [`RANGE_VERIFICATION_SQL`] (`MET-WP4-03A-OPS-03`).
///
/// The statement is split only where the range form adds a predicate or an
/// output, so [`VERIFICATION_SQL`] is exactly the fragments in order with
/// nothing between them, and its rendered text is the text the OPS-02
/// specification approved; a hash test pins it. The fragments are
/// verifier-only: no writer statement, constant or macro uses them.
///
/// Structure, top to bottom:
///
/// 1. `membership`: every pair of a possible represented set — an integer
///    whose bit `m` records that presence mask `m` is represented in a daily
///    base cell — and one of its member masks. There are 256 possible sets
///    over the eight `MET-WP4-02` masks (publication = 4, country = 2,
///    institution = 1), so the approved rule is derived for every one of
///    them, generically, rather than enumerated by hand.
/// 2. `country_minimal` / `institution_minimal`: for every set, the minimal
///    member masks containing the target dimension under set inclusion, by
///    anti-join: a candidate survives only if no other candidate of the same
///    set is a strict subset of it. This is the generic minimality rule of
///    the test oracle, not the writer's fixed case table.
/// 3. `resolution`: one row per set holding the Amendment 6 total rule (an
///    undimensioned member is authoritative, otherwise a single member,
///    otherwise no total and the cell is total-ambiguous), the unique
///    minimal country and institution masks or the ambiguity flag where two
///    or more minimal masks are incomparable, and the bitmap of minimal
///    masks whose rows establish an ambiguity watermark.
/// 4. `day_rows`: every work-day row with its month and its calendar year,
///    its own mask and its cell's represented set, gathered by one window
///    aggregate over the base cell.
/// 5. `resolved`: every day row joined to its set's resolution: a 256-row
///    lookup, so the plan is one scan of the work-day projection, one sort
///    by base cell and one small hash join, whatever the row count.
/// 6. `expected_*`: the four expected monthly datasets, summing only the
///    rows whose own mask is the selected one, with `publication_id`
///    retained exactly as the row holds it, dependency flags as "any
///    contributing row was broken down by that dimension", and the greatest
///    contributing watermark. The ambiguity watermark ranges over the rows
///    that establish a true flag: every row of a total-ambiguous cell and the
///    minimal rows of a country- or institution-ambiguous cell. The two
///    expected yearly datasets (`expected_country_year`,
///    `expected_institution_year`) sum the same selected daily rows grouped
///    by calendar year instead of month, straight from the resolved work-day
///    rows: the writer's monthly rows take no part in them.
/// 7. `*_compare`: expected and actual rows of one family are tagged and
///    unioned, grouped by logical identity — `GROUP BY` treats `NULL`
///    publication ids as one identity, which is exactly the tables'
///    `UNIQUE NULLS NOT DISTINCT` contract — and counted as missing (expected
///    only), extra (actual only) or mismatched (both, with any semantic field
///    differing, detected as `MIN <> MAX` or `bool_and <> bool_or` within the
///    identity group). The greatest actual watermark per family is returned
///    too, so a monthly or yearly row above the frontier is caught
///    explicitly.
///
/// `SUM(bigint)` is exact `numeric`; the cast back to `bigint` is what fails
/// closed, from PostgreSQL itself, when an expected monthly or yearly total
/// would overflow.
macro_rules! verification_head_sql {
    () => {
        "WITH membership AS ( \
         SELECT s.s, m.m \
         FROM generate_series(0, 255) AS s(s) \
         CROSS JOIN generate_series(0, 7) AS m(m) \
         WHERE s.s & (1 << m.m) <> 0 \
     ), \
     country_minimal AS ( \
         SELECT c.s, c.m \
         FROM membership c \
         WHERE c.m & 2 <> 0 \
           AND NOT EXISTS ( \
               SELECT 1 FROM membership o \
               WHERE o.s = c.s AND o.m & 2 <> 0 AND o.m <> c.m AND (o.m & c.m) = o.m) \
     ), \
     institution_minimal AS ( \
         SELECT c.s, c.m \
         FROM membership c \
         WHERE c.m & 1 <> 0 \
           AND NOT EXISTS ( \
               SELECT 1 FROM membership o \
               WHERE o.s = c.s AND o.m & 1 <> 0 AND o.m <> c.m AND (o.m & c.m) = o.m) \
     ), \
     resolution AS ( \
         SELECT t.s, \
                t.total_mask, \
                (t.total_mask IS NULL) AS total_ambiguous, \
                c.selected_mask AS country_mask, \
                COALESCE(c.ambiguous, false) AS country_ambiguous, \
                COALESCE(c.minimal_bits, 0) AS country_minimal_bits, \
                i.selected_mask AS institution_mask, \
                COALESCE(i.ambiguous, false) AS institution_ambiguous, \
                COALESCE(i.minimal_bits, 0) AS institution_minimal_bits \
         FROM ( \
             SELECT s, \
                    CASE WHEN bool_or(m = 0) THEN 0 \
                         WHEN count(*) = 1 THEN min(m) \
                    END AS total_mask \
             FROM membership \
             GROUP BY s \
         ) t \
         LEFT JOIN ( \
             SELECT s, \
                    CASE WHEN count(*) = 1 THEN min(m) END AS selected_mask, \
                    count(*) > 1 AS ambiguous, \
                    bit_or(1 << m) AS minimal_bits \
             FROM country_minimal \
             GROUP BY s \
         ) c ON c.s = t.s \
         LEFT JOIN ( \
             SELECT s, \
                    CASE WHEN count(*) = 1 THEN min(m) END AS selected_mask, \
                    count(*) > 1 AS ambiguous, \
                    bit_or(1 << m) AS minimal_bits \
             FROM institution_minimal \
             GROUP BY s \
         ) i ON i.s = t.s \
     ), \
     day_rows AS ( \
         SELECT r.work_id, r.publication_id, r.platform_id, r.measure_id, r.day, \
                date_trunc('month', r.day)::date AS month_start, \
                date_trunc('year', r.day)::date AS year_start, \
                r.country_code, r.institution_id, r.value, r.watermark, \
                x.mask, \
                bit_or(1 << x.mask) OVER (PARTITION BY r.work_id, r.platform_id, \
                                                       r.measure_id, r.day) AS represented \
         FROM public.metric_rollup_work_day r \
         CROSS JOIN LATERAL ( \
             SELECT CASE WHEN r.publication_id IS NULL THEN 0 ELSE 4 END \
                  + CASE WHEN r.country_code IS NULL THEN 0 ELSE 2 END \
                  + CASE WHEN r.institution_id IS NULL THEN 0 ELSE 1 END AS mask \
         ) x "
    };
}

/// From the end of `day_rows` through the actual side of `total_compare`.
macro_rules! verification_total_sql {
    () => {
        "), \
     resolved AS ( \
         SELECT d.work_id, d.publication_id, d.platform_id, d.measure_id, d.month_start, \
                d.year_start, d.country_code, d.institution_id, d.value, d.watermark, d.mask, \
                x.total_mask, x.total_ambiguous, \
                x.country_mask, x.country_ambiguous, \
                (x.country_minimal_bits & (1 << d.mask)) <> 0 AS country_minimal, \
                x.institution_mask, x.institution_ambiguous, \
                (x.institution_minimal_bits & (1 << d.mask)) <> 0 AS institution_minimal \
         FROM day_rows d \
         JOIN resolution x ON x.s = d.represented \
     ), \
     expected_total AS ( \
         SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                SUM(value)::bigint AS value, \
                bool_or(country_code IS NOT NULL) AS requires_country_coverage, \
                bool_or(institution_id IS NOT NULL) AS requires_institution_coverage, \
                MAX(watermark) AS watermark \
         FROM resolved \
         WHERE mask = total_mask \
         GROUP BY work_id, publication_id, platform_id, measure_id, month_start \
     ), \
     expected_country AS ( \
         SELECT work_id, publication_id, platform_id, measure_id, month_start, country_code, \
                SUM(value)::bigint AS value, \
                bool_or(institution_id IS NOT NULL) AS requires_institution_coverage, \
                MAX(watermark) AS watermark \
         FROM resolved \
         WHERE mask = country_mask \
         GROUP BY work_id, publication_id, platform_id, measure_id, month_start, country_code \
     ), \
     expected_institution AS ( \
         SELECT work_id, publication_id, platform_id, measure_id, month_start, institution_id, \
                SUM(value)::bigint AS value, \
                bool_or(country_code IS NOT NULL) AS requires_country_coverage, \
                MAX(watermark) AS watermark \
         FROM resolved \
         WHERE mask = institution_mask \
         GROUP BY work_id, publication_id, platform_id, measure_id, month_start, institution_id \
     ), \
     expected_ambiguity AS ( \
         SELECT work_id, platform_id, measure_id, month_start, \
                bool_or(total_ambiguous) AS total_ambiguous, \
                bool_or(country_ambiguous) AS country_ambiguous, \
                bool_or(institution_ambiguous) AS institution_ambiguous, \
                MAX(watermark) FILTER (WHERE total_ambiguous \
                    OR (country_ambiguous AND country_minimal) \
                    OR (institution_ambiguous AND institution_minimal)) AS watermark \
         FROM resolved \
         GROUP BY work_id, platform_id, measure_id, month_start \
         HAVING bool_or(total_ambiguous OR country_ambiguous OR institution_ambiguous) \
     ), \
     expected_country_year AS ( \
         SELECT work_id, publication_id, platform_id, measure_id, year_start, country_code, \
                SUM(value)::bigint AS value, \
                bool_or(institution_id IS NOT NULL) AS requires_institution_coverage, \
                MAX(watermark) AS watermark \
         FROM resolved \
         WHERE mask = country_mask \
         GROUP BY work_id, publication_id, platform_id, measure_id, year_start, country_code \
     ), \
     expected_institution_year AS ( \
         SELECT work_id, publication_id, platform_id, measure_id, year_start, institution_id, \
                SUM(value)::bigint AS value, \
                bool_or(country_code IS NOT NULL) AS requires_country_coverage, \
                MAX(watermark) AS watermark \
         FROM resolved \
         WHERE mask = institution_mask \
         GROUP BY work_id, publication_id, platform_id, measure_id, year_start, institution_id \
     ), \
     total_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (MIN(value) <> MAX(value) \
                     OR bool_and(requires_country_coverage) <> bool_or(requires_country_coverage) \
                     OR bool_and(requires_institution_coverage) \
                        <> bool_or(requires_institution_coverage) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, value, \
                        requires_country_coverage, requires_institution_coverage, watermark, \
                        1 AS e, 0 AS a \
                 FROM expected_total \
                 UNION ALL \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, value, \
                        requires_country_coverage, requires_institution_coverage, watermark, \
                        0, 1 \
                 FROM public.metric_rollup_work_month "
    };
}

/// From the end of `total_compare` through the actual side of
/// `country_compare`.
macro_rules! verification_country_sql {
    () => {
        ") u \
             GROUP BY work_id, publication_id, platform_id, measure_id, month_start \
         ) g \
     ), \
     country_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (MIN(value) <> MAX(value) \
                     OR bool_and(requires_institution_coverage) \
                        <> bool_or(requires_institution_coverage) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                        country_code, value, requires_institution_coverage, watermark, \
                        1 AS e, 0 AS a \
                 FROM expected_country \
                 UNION ALL \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                        country_code, value, requires_institution_coverage, watermark, \
                        0, 1 \
                 FROM public.metric_rollup_work_country_month "
    };
}

/// From the end of `country_compare` through the actual side of
/// `institution_compare`.
macro_rules! verification_institution_sql {
    () => {
        ") u \
             GROUP BY work_id, publication_id, platform_id, measure_id, month_start, country_code \
         ) g \
     ), \
     institution_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (MIN(value) <> MAX(value) \
                     OR bool_and(requires_country_coverage) <> bool_or(requires_country_coverage) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                        institution_id, value, requires_country_coverage, watermark, \
                        1 AS e, 0 AS a \
                 FROM expected_institution \
                 UNION ALL \
                 SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                        institution_id, value, requires_country_coverage, watermark, \
                        0, 1 \
                 FROM public.metric_rollup_work_institution_month "
    };
}

/// From the end of `institution_compare` through the actual side of
/// `ambiguity_compare`.
macro_rules! verification_ambiguity_sql {
    () => {
        ") u \
             GROUP BY work_id, publication_id, platform_id, measure_id, month_start, \
                      institution_id \
         ) g \
     ), \
     ambiguity_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (bool_and(total_ambiguous) <> bool_or(total_ambiguous) \
                     OR bool_and(country_ambiguous) <> bool_or(country_ambiguous) \
                     OR bool_and(institution_ambiguous) <> bool_or(institution_ambiguous) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, platform_id, measure_id, month_start, total_ambiguous, \
                        country_ambiguous, institution_ambiguous, watermark, 1 AS e, 0 AS a \
                 FROM expected_ambiguity \
                 UNION ALL \
                 SELECT work_id, platform_id, measure_id, month_start, total_ambiguous, \
                        country_ambiguous, institution_ambiguous, watermark, 0, 1 \
                 FROM public.metric_rollup_work_month_ambiguity "
    };
}

/// From the end of `ambiguity_compare` through the actual side of
/// `country_year_compare`.
macro_rules! verification_country_year_sql {
    () => {
        ") u \
             GROUP BY work_id, platform_id, measure_id, month_start \
         ) g \
     ), \
     country_year_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (MIN(value) <> MAX(value) \
                     OR bool_and(requires_institution_coverage) \
                        <> bool_or(requires_institution_coverage) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, publication_id, platform_id, measure_id, year_start, \
                        country_code, value, requires_institution_coverage, watermark, \
                        1 AS e, 0 AS a \
                 FROM expected_country_year \
                 UNION ALL \
                 SELECT work_id, publication_id, platform_id, measure_id, year_start, \
                        country_code, value, requires_institution_coverage, watermark, \
                        0, 1 \
                 FROM public.metric_rollup_work_country_year "
    };
}

/// From the end of `country_year_compare` through the actual side of
/// `institution_year_compare`.
macro_rules! verification_institution_year_sql {
    () => {
        ") u \
             GROUP BY work_id, publication_id, platform_id, measure_id, year_start, country_code \
         ) g \
     ), \
     institution_year_compare AS ( \
         SELECT count(*) FILTER (WHERE e = 1) AS expected_rows, \
                count(*) FILTER (WHERE a = 1) AS actual_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 0) AS missing_rows, \
                count(*) FILTER (WHERE e = 0 AND a = 1) AS extra_rows, \
                count(*) FILTER (WHERE e = 1 AND a = 1 AND differs) AS mismatched_rows, \
                MAX(actual_watermark) AS actual_max_watermark \
         FROM ( \
             SELECT SUM(e) AS e, SUM(a) AS a, \
                    (MIN(value) <> MAX(value) \
                     OR bool_and(requires_country_coverage) <> bool_or(requires_country_coverage) \
                     OR MIN(watermark) <> MAX(watermark)) AS differs, \
                    MAX(watermark) FILTER (WHERE a = 1) AS actual_watermark \
             FROM ( \
                 SELECT work_id, publication_id, platform_id, measure_id, year_start, \
                        institution_id, value, requires_country_coverage, watermark, \
                        1 AS e, 0 AS a \
                 FROM expected_institution_year \
                 UNION ALL \
                 SELECT work_id, publication_id, platform_id, measure_id, year_start, \
                        institution_id, value, requires_country_coverage, watermark, \
                        0, 1 \
                 FROM public.metric_rollup_work_institution_year "
    };
}

/// The end of `institution_year_compare`, which closes the `WITH` list.
macro_rules! verification_compare_end_sql {
    () => {
        ") u \
             GROUP BY work_id, publication_id, platform_id, measure_id, year_start, \
                      institution_id \
         ) g \
     )"
    };
}

/// The output columns: the five counts and the greatest actual watermark of
/// each family.
macro_rules! verification_columns_sql {
    () => {
        " SELECT t.expected_rows AS total_expected_rows, t.actual_rows AS total_actual_rows, \
            t.missing_rows AS total_missing_rows, t.extra_rows AS total_extra_rows, \
            t.mismatched_rows AS total_mismatched_rows, \
            t.actual_max_watermark AS total_actual_max_watermark, \
            c.expected_rows AS country_expected_rows, c.actual_rows AS country_actual_rows, \
            c.missing_rows AS country_missing_rows, c.extra_rows AS country_extra_rows, \
            c.mismatched_rows AS country_mismatched_rows, \
            c.actual_max_watermark AS country_actual_max_watermark, \
            i.expected_rows AS institution_expected_rows, \
            i.actual_rows AS institution_actual_rows, \
            i.missing_rows AS institution_missing_rows, \
            i.extra_rows AS institution_extra_rows, \
            i.mismatched_rows AS institution_mismatched_rows, \
            i.actual_max_watermark AS institution_actual_max_watermark, \
            m.expected_rows AS ambiguity_expected_rows, m.actual_rows AS ambiguity_actual_rows, \
            m.missing_rows AS ambiguity_missing_rows, m.extra_rows AS ambiguity_extra_rows, \
            m.mismatched_rows AS ambiguity_mismatched_rows, \
            m.actual_max_watermark AS ambiguity_actual_max_watermark, \
            cy.expected_rows AS country_year_expected_rows, \
            cy.actual_rows AS country_year_actual_rows, \
            cy.missing_rows AS country_year_missing_rows, \
            cy.extra_rows AS country_year_extra_rows, \
            cy.mismatched_rows AS country_year_mismatched_rows, \
            cy.actual_max_watermark AS country_year_actual_max_watermark, \
            iy.expected_rows AS institution_year_expected_rows, \
            iy.actual_rows AS institution_year_actual_rows, \
            iy.missing_rows AS institution_year_missing_rows, \
            iy.extra_rows AS institution_year_extra_rows, \
            iy.mismatched_rows AS institution_year_mismatched_rows, \
            iy.actual_max_watermark AS institution_year_actual_max_watermark"
    };
}

/// The six one-row comparisons, crossed into the one output row.
macro_rules! verification_from_sql {
    () => {
        " FROM total_compare t \
     CROSS JOIN country_compare c \
     CROSS JOIN institution_compare i \
     CROSS JOIN ambiguity_compare m \
     CROSS JOIN country_year_compare cy \
     CROSS JOIN institution_year_compare iy"
    };
}

/// The one comparison statement over the whole work-day projection and the
/// whole of every derived table: the fragments above, in order, with nothing
/// between them.
pub(crate) const VERIFICATION_SQL: &str = concat!(
    verification_head_sql!(),
    verification_total_sql!(),
    verification_country_sql!(),
    verification_institution_sql!(),
    verification_ambiguity_sql!(),
    verification_country_year_sql!(),
    verification_institution_year_sql!(),
    verification_compare_end_sql!(),
    verification_columns_sql!(),
    verification_from_sql!(),
);

// ---------------------------------------------------------------------------
// MET-WP4-03A-OPS-03: the range verifier
// ---------------------------------------------------------------------------
//
// The range form applies the same generic rules to the work-day rows and the
// derived rows of one range `(lo, hi]` of units `(work_id, year_start)`, in
// unit order. Its bounds are four bind parameters: `$1` and `$2` are the
// lower bound's `work_id` and the 31 December of its year (`NULL` for minus
// infinity, with `$1` the nil uuid), and `$3` and `$4` the upper bound's
// (`NULL` for plus infinity, with `$3` the greatest uuid). A work-day row is
// in the range when `(work_id, day)` lies in `((lo, 31 Dec), (hi, 31 Dec)]`:
// the `(work_id, day)` index range. A derived row is in the range when its
// `(work_id, month_start)` or `(work_id, year_start)` lies in the same
// interval, which places it by its own year, so orphan rows are covered; the
// explicit `work_id` bounds are what the identity indexes lead with.

/// The work-day predicate of the range form, closing `day_rows`.
macro_rules! range_work_day_filter_sql {
    () => {
        "WHERE r.work_id >= $1 AND r.work_id <= $3 \
           AND (r.work_id, r.day) > ($1, COALESCE($2, '-infinity'::date)) \
           AND (r.work_id, r.day) <= ($3, COALESCE($4, 'infinity'::date)) "
    };
}

/// The predicate on a monthly table's actual rows.
macro_rules! range_month_filter_sql {
    () => {
        "WHERE work_id >= $1 AND work_id <= $3 \
           AND (work_id, month_start) > ($1, COALESCE($2, '-infinity'::date)) \
           AND (work_id, month_start) <= ($3, COALESCE($4, 'infinity'::date)) "
    };
}

/// The predicate on a yearly table's actual rows.
macro_rules! range_year_filter_sql {
    () => {
        "WHERE work_id >= $1 AND work_id <= $3 \
           AND (work_id, year_start) > ($1, COALESCE($2, '-infinity'::date)) \
           AND (work_id, year_start) <= ($3, COALESCE($4, 'infinity'::date)) "
    };
}

/// The comparison statement of one range: [`VERIFICATION_SQL`] with the
/// range predicate on the work-day rows and on every actual table, plus the
/// range's work-day row count, represented month-key count and greatest
/// work-day watermark, from the resolved rows of the range.
pub(crate) const RANGE_VERIFICATION_SQL: &str = concat!(
    verification_head_sql!(),
    range_work_day_filter_sql!(),
    verification_total_sql!(),
    range_month_filter_sql!(),
    verification_country_sql!(),
    range_month_filter_sql!(),
    verification_institution_sql!(),
    range_month_filter_sql!(),
    verification_ambiguity_sql!(),
    range_month_filter_sql!(),
    verification_country_year_sql!(),
    range_year_filter_sql!(),
    verification_institution_year_sql!(),
    range_year_filter_sql!(),
    verification_compare_end_sql!(),
    ", range_stats AS ( \
         SELECT count(*) AS work_day_rows, \
                count(DISTINCT (work_id, platform_id, measure_id, month_start)) AS month_keys, \
                MAX(watermark) AS max_work_day_watermark \
         FROM resolved \
     )",
    verification_columns_sql!(),
    ", s.work_day_rows, s.month_keys, s.max_work_day_watermark",
    verification_from_sql!(),
    " CROSS JOIN range_stats s",
);

/// The bounds of one range `(lo, hi]` of units, as the range verifier binds
/// them. A bound's date is the 31 December of its unit's year; `None` is
/// minus infinity for the lower bound (with the nil uuid) and plus infinity
/// for the upper bound (with the greatest uuid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RangeBounds {
    pub(crate) lo_work_id: Uuid,
    pub(crate) lo_end: Option<NaiveDate>,
    pub(crate) hi_work_id: Uuid,
    pub(crate) hi_end: Option<NaiveDate>,
}

/// The single row the range comparison statement returns.
#[derive(diesel::QueryableByName)]
struct RangeComparisonRow {
    #[diesel(embed)]
    comparison: ComparisonRow,
    #[diesel(sql_type = BigInt)]
    work_day_rows: i64,
    #[diesel(sql_type = BigInt)]
    month_keys: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    max_work_day_watermark: Option<i64>,
}

/// The bounded result of one range verification: the five counts of each of
/// the six families, the greatest actual watermark of each family (in the
/// family order `total`, `country`, `institution`, `ambiguity`,
/// `country_year`, `institution_year`), and the range's work-day facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RangeVerification {
    pub(crate) total: MetricRollupMonthProjectionVerification,
    pub(crate) country: MetricRollupMonthProjectionVerification,
    pub(crate) institution: MetricRollupMonthProjectionVerification,
    pub(crate) ambiguity: MetricRollupMonthProjectionVerification,
    pub(crate) country_year: MetricRollupMonthProjectionVerification,
    pub(crate) institution_year: MetricRollupMonthProjectionVerification,
    pub(crate) actual_max_watermarks: [Option<i64>; 6],
    pub(crate) work_day_rows: i64,
    pub(crate) month_keys: i64,
    pub(crate) max_work_day_watermark: Option<i64>,
}

impl RangeVerification {
    /// The six families, in the fixed family order.
    pub(crate) fn families(&self) -> [&MetricRollupMonthProjectionVerification; 6] {
        [
            &self.total,
            &self.country,
            &self.institution,
            &self.ambiguity,
            &self.country_year,
            &self.institution_year,
        ]
    }

    /// The missing rows summed over the six families.
    pub(crate) fn missing_rows(&self) -> i64 {
        self.families().iter().map(|f| f.missing_rows).sum()
    }

    /// The extra rows summed over the six families.
    pub(crate) fn extra_rows(&self) -> i64 {
        self.families().iter().map(|f| f.extra_rows).sum()
    }

    /// The mismatched rows summed over the six families.
    pub(crate) fn mismatched_rows(&self) -> i64 {
        self.families().iter().map(|f| f.mismatched_rows).sum()
    }

    /// Whether some work-day row of the range is watermarked above
    /// `frontier`: out-of-band damage no range result can describe.
    pub(crate) fn work_day_above(&self, frontier: i64) -> bool {
        self.max_work_day_watermark
            .is_some_and(|watermark| watermark > frontier)
    }

    /// Whether the range is exact at `frontier`: every family has zero
    /// missing, extra and mismatched rows, and no derived row of the range
    /// is watermarked above `frontier`.
    pub(crate) fn exact_at(&self, frontier: i64) -> bool {
        self.families().iter().all(|family| exact(family))
            && self
                .actual_max_watermarks
                .iter()
                .all(|watermark| watermark.is_none_or(|watermark| watermark <= frontier))
    }
}

/// Compare the six families against their independently derived expected
/// state over one range of units, in the caller's transaction and snapshot,
/// with one statement.
///
/// Reads only, and returns the raw database error so the caller can classify
/// it before any conversion; an overflow of an expected sum surfaces from
/// PostgreSQL's own cast as `bigint out of range`.
pub(crate) fn verify_range(
    connection: &mut PgConnection,
    bounds: &RangeBounds,
) -> Result<RangeVerification, DieselError> {
    let row: RangeComparisonRow = diesel::sql_query(RANGE_VERIFICATION_SQL)
        .bind::<SqlUuid, _>(bounds.lo_work_id)
        .bind::<Nullable<Date>, _>(bounds.lo_end)
        .bind::<SqlUuid, _>(bounds.hi_work_id)
        .bind::<Nullable<Date>, _>(bounds.hi_end)
        .get_result(connection)?;
    let c = row.comparison;
    Ok(RangeVerification {
        total: family(
            c.total_expected_rows,
            c.total_actual_rows,
            c.total_missing_rows,
            c.total_extra_rows,
            c.total_mismatched_rows,
        ),
        country: family(
            c.country_expected_rows,
            c.country_actual_rows,
            c.country_missing_rows,
            c.country_extra_rows,
            c.country_mismatched_rows,
        ),
        institution: family(
            c.institution_expected_rows,
            c.institution_actual_rows,
            c.institution_missing_rows,
            c.institution_extra_rows,
            c.institution_mismatched_rows,
        ),
        ambiguity: family(
            c.ambiguity_expected_rows,
            c.ambiguity_actual_rows,
            c.ambiguity_missing_rows,
            c.ambiguity_extra_rows,
            c.ambiguity_mismatched_rows,
        ),
        country_year: family(
            c.country_year_expected_rows,
            c.country_year_actual_rows,
            c.country_year_missing_rows,
            c.country_year_extra_rows,
            c.country_year_mismatched_rows,
        ),
        institution_year: family(
            c.institution_year_expected_rows,
            c.institution_year_actual_rows,
            c.institution_year_missing_rows,
            c.institution_year_extra_rows,
            c.institution_year_mismatched_rows,
        ),
        actual_max_watermarks: [
            c.total_actual_max_watermark,
            c.country_actual_max_watermark,
            c.institution_actual_max_watermark,
            c.ambiguity_actual_max_watermark,
            c.country_year_actual_max_watermark,
            c.institution_year_actual_max_watermark,
        ],
        work_day_rows: row.work_day_rows,
        month_keys: row.month_keys,
        max_work_day_watermark: row.max_work_day_watermark,
    })
}

/// The single row the comparison statement returns.
#[derive(diesel::QueryableByName)]
struct ComparisonRow {
    #[diesel(sql_type = BigInt)]
    total_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    total_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    total_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    total_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    total_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    total_actual_max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    country_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    country_actual_max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    institution_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    institution_actual_max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    ambiguity_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    ambiguity_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    ambiguity_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    ambiguity_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    ambiguity_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    ambiguity_actual_max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    country_year_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_year_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_year_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_year_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    country_year_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    country_year_actual_max_watermark: Option<i64>,
    #[diesel(sql_type = BigInt)]
    institution_year_expected_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_year_actual_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_year_missing_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_year_extra_rows: i64,
    #[diesel(sql_type = BigInt)]
    institution_year_mismatched_rows: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    institution_year_actual_max_watermark: Option<i64>,
}

/// PostgreSQL's fixed message for a `numeric` value that does not fit a
/// `bigint` (SQLSTATE `22003`), raised by the cast of an expected monthly or
/// yearly sum. Diesel exposes no SQLSTATE, so the message is the
/// discriminator.
const BIGINT_OUT_OF_RANGE: &str = "bigint out of range";

/// Render an overflow of the expected-state derivation as the bounded
/// rollup rejection; every other database failure propagates unchanged. A
/// yearly sum can overflow while every monthly sum of the year fits, so the
/// message names both layers.
fn verification_arithmetic(error: DieselError) -> thoth_errors::ThothError {
    if let DieselError::DatabaseError(_, info) = &error {
        if info.message() == BIGINT_OUT_OF_RANGE {
            return rejected(
                "Deriving the expected monthly or yearly projections would overflow \
                 a projected total. The derived state cannot be verified or \
                 rebuilt until the work-day projection is repaired.",
            );
        }
    }
    error.into()
}

fn family(
    expected_rows: i64,
    actual_rows: i64,
    missing_rows: i64,
    extra_rows: i64,
    mismatched_rows: i64,
) -> MetricRollupMonthProjectionVerification {
    MetricRollupMonthProjectionVerification {
        expected_rows,
        actual_rows,
        missing_rows,
        extra_rows,
        mismatched_rows,
    }
}

fn exact(family: &MetricRollupMonthProjectionVerification) -> bool {
    family.missing_rows == 0 && family.extra_rows == 0 && family.mismatched_rows == 0
}

/// Compare all four monthly and both yearly projections against their
/// independently derived expected state, in the caller's transaction and
/// snapshot, with one statement.
///
/// `frontier` and `stats` must have been read in the same transaction. The
/// work-day watermark bound is re-checked here so that no caller can obtain
/// a verification describing a projection whose source is above the
/// frontier. `matches` is true only when every one of the six families is
/// exact and no actual monthly or yearly row is watermarked above the
/// frontier.
pub(crate) fn verify_projections(
    connection: &mut PgConnection,
    frontier: &FrontierSnapshot,
    stats: &WorkDayStats,
) -> ThothResult<MetricRollupMonthVerification> {
    require_work_day_within_frontier(frontier, stats)?;

    let rows: Vec<ComparisonRow> = diesel::sql_query(VERIFICATION_SQL)
        .load(connection)
        .map_err(verification_arithmetic)?;
    let Some(row) = rows.into_iter().next() else {
        return Err(broken_invariant(
            "the monthly verification comparison returned no row",
        ));
    };

    let total = family(
        row.total_expected_rows,
        row.total_actual_rows,
        row.total_missing_rows,
        row.total_extra_rows,
        row.total_mismatched_rows,
    );
    let country = family(
        row.country_expected_rows,
        row.country_actual_rows,
        row.country_missing_rows,
        row.country_extra_rows,
        row.country_mismatched_rows,
    );
    let institution = family(
        row.institution_expected_rows,
        row.institution_actual_rows,
        row.institution_missing_rows,
        row.institution_extra_rows,
        row.institution_mismatched_rows,
    );
    let ambiguity = family(
        row.ambiguity_expected_rows,
        row.ambiguity_actual_rows,
        row.ambiguity_missing_rows,
        row.ambiguity_extra_rows,
        row.ambiguity_mismatched_rows,
    );
    let country_year = family(
        row.country_year_expected_rows,
        row.country_year_actual_rows,
        row.country_year_missing_rows,
        row.country_year_extra_rows,
        row.country_year_mismatched_rows,
    );
    let institution_year = family(
        row.institution_year_expected_rows,
        row.institution_year_actual_rows,
        row.institution_year_missing_rows,
        row.institution_year_extra_rows,
        row.institution_year_mismatched_rows,
    );

    let frontier_w = frontier.applied_through_sequence;
    let derived_within_frontier = [
        row.total_actual_max_watermark,
        row.country_actual_max_watermark,
        row.institution_actual_max_watermark,
        row.ambiguity_actual_max_watermark,
        row.country_year_actual_max_watermark,
        row.institution_year_actual_max_watermark,
    ]
    .into_iter()
    .all(|watermark| watermark.is_none_or(|watermark| watermark <= frontier_w));

    let matches = exact(&total)
        && exact(&country)
        && exact(&institution)
        && exact(&ambiguity)
        && exact(&country_year)
        && exact(&institution_year)
        && derived_within_frontier;

    Ok(MetricRollupMonthVerification {
        applied_through_sequence: frontier.applied_through_sequence,
        next_sequence: frontier.next_sequence,
        watermark_at: frontier.watermark_at,
        max_work_day_watermark: stats.max_watermark,
        work_day_row_count: stats.row_count,
        represented_month_key_count: stats.month_key_count,
        total,
        country,
        institution,
        ambiguity,
        country_year,
        institution_year,
        matches,
    })
}

// ---------------------------------------------------------------------------
// verifyMetricRollupMonths
// ---------------------------------------------------------------------------

/// The statement timeout the verification transaction sets locally:
/// [`super::METRIC_ROLLUP_MAINTENANCE_STATEMENT_TIMEOUT_SECONDS`].
pub(crate) const VERIFICATION_STATEMENT_TIMEOUT_SQL: &str = "SET LOCAL statement_timeout = '30s'";

/// The one table-level lock the verification takes, before its first query
/// and therefore before its `REPEATABLE READ` snapshot is frozen.
///
/// Exactly the four monthly and the two yearly projection tables, in
/// `ACCESS SHARE` mode: the weakest table lock, granted to every ordinary
/// reader, conflicting only with `ACCESS EXCLUSIVE`. It exists solely to
/// close PostgreSQL's non-MVCC-safe `TRUNCATE` anomaly: with the lock held
/// from before the snapshot until commit, the rebuild's six-table `TRUNCATE`
/// (which needs `ACCESS EXCLUSIVE`) either finished and committed before the
/// lock was granted — and the snapshot then sees the rebuilt state — or
/// waits until the verification commits. Normal completion writes the
/// monthly and yearly tables under `ROW EXCLUSIVE`, which does not conflict,
/// so incremental maintenance is never held up by a verification. It is not
/// a row lock, takes no `FOR UPDATE`/`FOR SHARE`, and the transaction
/// remains read-only.
pub(crate) const VERIFICATION_LOCK_SQL: &str = "LOCK TABLE \
         public.metric_rollup_work_month, \
         public.metric_rollup_work_country_month, \
         public.metric_rollup_work_institution_month, \
         public.metric_rollup_work_month_ambiguity, \
         public.metric_rollup_work_country_year, \
         public.metric_rollup_work_institution_year \
     IN ACCESS SHARE MODE";

/// The transaction mode the verification asserts before reading anything.
#[derive(diesel::QueryableByName)]
struct TransactionModeRow {
    #[diesel(sql_type = Text)]
    read_only: String,
    #[diesel(sql_type = Text)]
    isolation: String,
}

/// The transaction mode as the server reports it for this transaction.
pub(crate) const TRANSACTION_MODE_SQL: &str =
    "SELECT current_setting('transaction_read_only') AS read_only, \
            current_setting('transaction_isolation') AS isolation";

/// Verify the four monthly and the two yearly projections in one read-only
/// snapshot.
///
/// One pooled connection, one transaction opened `READ ONLY` at
/// `REPEATABLE READ`, no `FOR UPDATE` and no database mutation of any kind:
/// the server refuses every write in a read-only transaction, and the
/// operation additionally asserts the mode it was granted before it reads
/// anything, so a misconfigured connection fails closed rather than
/// silently verifying under `READ COMMITTED`. The statement order is fixed:
/// the local statement timeout, then [`VERIFICATION_LOCK_SQL`] — which must
/// precede the first query, because a `REPEATABLE READ` snapshot is frozen
/// by the first query and a snapshot older than a concurrent `TRUNCATE`
/// would see the truncated tables as empty — then the transaction-mode
/// assertion, which freezes the snapshot, then the frontier, the work-day
/// facts and the comparison. The statement timeout also bounds how long the
/// lock may wait behind an in-flight rebuild.
///
/// `next_sequence - 1 > applied_through_sequence` is ordinary pending rollup
/// lag and is returned as such, never reported as derived-state corruption.
/// A work-day row watermarked above the frontier is an invariant failure and
/// fails the operation closed.
pub(crate) fn verify_metric_rollup_months(
    db: &PgPool,
) -> ThothResult<MetricRollupMonthVerification> {
    let mut connection = db.get()?;
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            diesel::sql_query(VERIFICATION_STATEMENT_TIMEOUT_SQL).execute(connection)?;
            diesel::sql_query(VERIFICATION_LOCK_SQL).execute(connection)?;
            require_read_only_repeatable_read(connection)?;
            let frontier = read_frontier(connection)?;
            let stats = work_day_stats(connection)?;
            require_work_day_within_frontier(&frontier, &stats)?;
            verify_projections(connection, &frontier, &stats)
        })
}

/// Fail closed unless the current transaction is read-only at repeatable
/// read.
fn require_read_only_repeatable_read(connection: &mut PgConnection) -> ThothResult<()> {
    let rows: Vec<TransactionModeRow> = diesel::sql_query(TRANSACTION_MODE_SQL).load(connection)?;
    let mode = rows.into_iter().next();
    if mode
        .as_ref()
        .is_some_and(|mode| mode.read_only == "on" && mode.isolation == "repeatable read")
    {
        Ok(())
    } else {
        Err(broken_invariant(
            "the derived-state verification transaction is not read-only at repeatable read",
        ))
    }
}
