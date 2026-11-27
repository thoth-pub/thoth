//! The native-month projection (`MET-WP4-MONTH-01`, thoth#1042).
//!
//! Four derived tables hold, per work, platform, measure and month, the
//! resolved total, country and institution values of the canonical `MONTH`
//! records, and every non-`DAY` period that must refuse a Metrics read:
//!
//! - `metric_native_month`: one row per key holding a served month, with its
//!   resolved total (`NULL` when the cell is total-ambiguous), whether it is
//!   *live* (some served record's value is not 0), the two dependency flags of
//!   the selected mask and the two section-ambiguity flags;
//! - `metric_native_country_month` and `metric_native_institution_month`: the
//!   rows of the cell's unique least target-bearing mask, summed per country
//!   or institution, with the cross-dependency flag;
//! - `metric_native_refusal`: one row per period of a key holding an active
//!   `REPORTING_PERIOD` record or an inconsistent record of either grain.
//!
//! **Keys.** A key is `(work, platform, measure, period_start)`: every
//! non-`DAY` record that starts on that day, whatever its end or grain (the
//! columns of `metric_record_native_grain_idx`). So a write to one period never
//! removes another period's rows.
//!
//! **Maintenance.** The rows are a function of the current records of their
//! own key, so recomputing them is idempotent and needs no order: no position,
//! frontier, claim or lag. The one canonical writer,
//! [`apply_canonical_application`], adds the key of every `Winner` and
//! `Revision` outcome of a non-`DAY` record to a [`NativeMonthKeys`] set, and
//! each of its callers passes its set to [`recompute`] after its last
//! canonical write and before it commits: ingestion once per request,
//! quarantine reconciliation once per row. Every canonical write holds its
//! work row `FOR UPDATE`, so writers of one key are serialized and each
//! recompute sees every committed record of its key; records and rows commit
//! together. Any later path that changes a non-`DAY` record or its revisions
//! must recompute the key in the same transaction, under the work row lock.
//! The tables are maintained whatever `THOTH_METRIC_MONTH_SERVING` says, and
//! hold no frontier or watermark.
//!
//! **Classes.** Each record is classified by its revision pointer, as the
//! dashboard's native-grain check does:
//!
//! - *withdrawn*: the pointer names a `RETRACTED` revision and the record has
//!   no `CURRENT` one. It is ignored;
//! - *active reporting period*: a `REPORTING_PERIOD` record whose pointer
//!   names a `CURRENT` revision;
//! - *served period* (`MET-QUARTER-01`): a `MONTH` record spanning exactly
//!   one calendar month (`span_months` 1), or a `QUARTER` record spanning
//!   exactly one calendar quarter (`span_months` 3), whose pointer names a
//!   `CURRENT` revision of that record; its value is that revision's. The
//!   columns called `month_start` hold the period's first day whatever its
//!   span. A key whose served records carry two spans, or a served period that
//!   overlaps a served period starting on another day of the same work,
//!   platform and measure, is *conflicted*: neither is served and each is
//!   inconsistent. The rule is symmetric, so a recompute widens its keys to
//!   every key whose records overlap theirs;
//! - *inconsistent*: anything else of any non-`DAY` grain, a `MONTH` or
//!   `QUARTER` record of any other period included.
//!
//! **Resolution.** A key's served months form one cell, resolved exactly as
//! `MET-WP4-03A` resolves a day (`metric_rollup_delta/crud.rs`, register
//! Amendment 6): the undimensioned row if present, else the single
//! represented mask, summed, else total-ambiguous; sections by the unique
//! least target-bearing mask, incomparable minimal masks ambiguous. Sums are
//! signed `bigint`; an overflow, or any failure, fails the caller's
//! transaction, so nothing commits.
//!
//! **Repair.** [`recompute_metric_native_months`] deletes and re-derives the
//! rows of every work under a publisher's imprints, in chunks of at most
//! [`REPAIR_CHUNK_WORKS`] works, each chunk one transaction under the work
//! row locks. It is the remedy for a non-zero V-NATIVE (the contract
//! register's check), idempotent, and safe beside ingestion and reads.
//!
//! [`apply_canonical_application`]:
//!     crate::model::metric_ingestion::apply_canonical_application

use std::collections::BTreeSet;

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::sql_types::{Array, Date, Uuid as SqlUuid};
use diesel::{sql_query, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::metric_ingestion::lock_works;
use crate::schema::{imprint, publisher, work};

/// The most works one repair transaction recomputes.
pub(crate) const REPAIR_CHUNK_WORKS: usize = 50;

/// One projection key: `(work_id, platform_id, measure_id, period_start)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NativeMonthKey {
    pub work_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub period_start: NaiveDate,
}

/// The keys one transaction's canonical writes touched.
///
/// [`apply_canonical_application`] takes it by `&mut`, so no caller can apply
/// a candidate without one, and adds the key of every `Winner` and `Revision`
/// outcome of a non-`DAY` record.
///
/// [`apply_canonical_application`]:
///     crate::model::metric_ingestion::apply_canonical_application
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NativeMonthKeys {
    keys: BTreeSet<NativeMonthKey>,
}

impl NativeMonthKeys {
    /// Add one key; a key already present is kept once.
    pub(crate) fn insert(&mut self, key: NativeMonthKey) {
        self.keys.insert(key);
    }

    /// Whether no key was touched, so [`recompute`] runs no statement.
    pub(crate) fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The number of distinct keys.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.keys.len()
    }

    fn columns(&self) -> (Vec<Uuid>, Vec<Uuid>, Vec<Uuid>, Vec<NaiveDate>) {
        let mut works = Vec::with_capacity(self.keys.len());
        let mut platforms = Vec::with_capacity(self.keys.len());
        let mut measures = Vec::with_capacity(self.keys.len());
        let mut starts = Vec::with_capacity(self.keys.len());
        for key in &self.keys {
            works.push(key.work_id);
            platforms.push(key.platform_id);
            measures.push(key.measure_id);
            starts.push(key.period_start);
        }
        (works, platforms, measures, starts)
    }
}

impl FromIterator<NativeMonthKey> for NativeMonthKeys {
    fn from_iter<I: IntoIterator<Item = NativeMonthKey>>(iter: I) -> Self {
        Self {
            keys: iter.into_iter().collect(),
        }
    }
}

/// The keys of a recompute, from four parallel arrays, widened by every key
/// of the same work, platform and measure whose non-`DAY` records overlap a
/// record of a given key (`MET-QUARTER-01`: a served month and a served
/// quarter that overlap refuse each other, so a write to one must re-derive
/// the other). The widening reads `metric_record_native_grain_idx`.
macro_rules! key_scope_sql {
    () => {
        "keys AS ( \
             SELECT DISTINCT x.work_id, x.platform_id, x.measure_id, x.period_start \
             FROM ( \
                 SELECT k.work_id, k.platform_id, k.measure_id, k.period_start \
                 FROM unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
                      AS k(work_id, platform_id, measure_id, period_start) \
                 UNION \
                 SELECT r2.work_id, r2.platform_id, r2.measure_id, r2.period_start \
                 FROM unnest($1::uuid[], $2::uuid[], $3::uuid[], $4::date[]) \
                      AS k(work_id, platform_id, measure_id, period_start) \
                 JOIN public.metric_record r1 \
                   ON r1.work_id = k.work_id \
                  AND r1.platform_id = k.platform_id \
                  AND r1.measure_id = k.measure_id \
                  AND r1.period_start = k.period_start \
                  AND r1.reporting_grain <> 'DAY' \
                 JOIN public.metric_record r2 \
                   ON r2.work_id = r1.work_id \
                  AND r2.platform_id = r1.platform_id \
                  AND r2.measure_id = r1.measure_id \
                  AND r2.reporting_grain <> 'DAY' \
                  AND r2.period_start < r1.period_end \
                  AND r2.period_end > r1.period_start \
             ) x \
         )"
    };
}

/// The keys of a repair: every non-`DAY` key of the works `$1`.
macro_rules! work_scope_sql {
    () => {
        "keys AS ( \
             SELECT DISTINCT r.work_id, r.platform_id, r.measure_id, r.period_start \
             FROM public.metric_record r \
             WHERE r.work_id = ANY($1) \
               AND r.reporting_grain <> 'DAY' \
         )"
    };
}

/// Every non-`DAY` record of the keys and of every key whose records overlap
/// theirs (so a key is classified against all of its overlaps; only the keys' own
/// records, `in_keys`, are written), read through
/// `metric_record_native_grain_idx` (its predicate restated so the partial
/// index applies), with the revision its pointer names, by primary key.
macro_rules! key_records_sql {
    () => {
        "wide AS ( \
             SELECT k.work_id, k.platform_id, k.measure_id, k.period_start \
             FROM keys k \
             UNION \
             SELECT r2.work_id, r2.platform_id, r2.measure_id, r2.period_start \
             FROM keys k \
             JOIN public.metric_record r1 \
               ON r1.work_id = k.work_id \
              AND r1.platform_id = k.platform_id \
              AND r1.measure_id = k.measure_id \
              AND r1.period_start = k.period_start \
              AND r1.reporting_grain <> 'DAY' \
             JOIN public.metric_record r2 \
               ON r2.work_id = r1.work_id \
              AND r2.platform_id = r1.platform_id \
              AND r2.measure_id = r1.measure_id \
              AND r2.reporting_grain <> 'DAY' \
              AND r2.period_start < r1.period_end \
              AND r2.period_end > r1.period_start \
         ), \
         records AS ( \
             SELECT r.record_id, r.work_id, r.platform_id, r.measure_id, r.period_start, \
                    r.period_end, r.reporting_grain, r.publication_id, r.country_code, \
                    r.institution_id, v.status, v.value, \
                    EXISTS (SELECT 1 FROM keys q \
                            WHERE q.work_id = r.work_id AND q.platform_id = r.platform_id \
                              AND q.measure_id = r.measure_id \
                              AND q.period_start = r.period_start) AS in_keys \
             FROM wide k \
             JOIN public.metric_record r \
               ON r.work_id = k.work_id \
              AND r.platform_id = k.platform_id \
              AND r.measure_id = k.measure_id \
              AND r.period_start = k.period_start \
              AND r.reporting_grain <> 'DAY' \
             LEFT JOIN public.metric_record_revision v \
               ON v.record_revision_id = r.current_revision_id \
              AND v.record_id = r.record_id \
         )"
    };
}

/// The served periods of the keys: a `CURRENT` `MONTH` record of one calendar
/// month (`span` 1) or `QUARTER` record of one calendar quarter (`span` 3),
/// then the keys that must not be served. A key is *conflicted* when its served
/// records carry two spans, or when a served period of the same work, platform
/// and measure that starts on another day overlaps it. Both keys of an overlap
/// are conflicted, so the rule is symmetric and whichever is written second is
/// not the one refused.
macro_rules! cand_sql {
    () => {
        "cand AS ( \
             SELECT d.*, \
                    CASE d.reporting_grain WHEN 'QUARTER' THEN 3 ELSE 1 END AS span \
             FROM records d \
             WHERE d.status = 'CURRENT' \
               AND EXTRACT(DAY FROM d.period_start) = 1 \
               AND ((d.reporting_grain = 'MONTH' \
                     AND d.period_end = (d.period_start + interval '1 month')::date) \
                 OR (d.reporting_grain = 'QUARTER' \
                     AND EXTRACT(MONTH FROM d.period_start) IN (1, 4, 7, 10) \
                     AND d.period_end = (d.period_start + interval '3 months')::date)) \
         ), \
         conflicted AS ( \
             SELECT DISTINCT a.work_id, a.platform_id, a.measure_id, a.period_start \
             FROM cand a \
             JOIN cand b \
               ON b.work_id = a.work_id \
              AND b.platform_id = a.platform_id \
              AND b.measure_id = a.measure_id \
              AND ((b.period_start = a.period_start AND b.span <> a.span) \
                OR (b.period_start <> a.period_start \
                    AND b.period_start < a.period_end \
                    AND b.period_end > a.period_start)) \
         )"
    };
}

/// The served months of the keys, each with its dimension mask (publication
/// 4, country 2, institution 1), its cell's represented masks and the
/// resolution of its cell, exactly as `MET-WP4-03A` resolves a day.
macro_rules! served_sql {
    () => {
        "served AS ( \
             SELECT x.*, \
                    CASE WHEN x.represented & 1 <> 0 THEN 0 \
                         WHEN x.represented = 2 THEN 1 \
                         WHEN x.represented = 4 THEN 2 \
                         WHEN x.represented = 8 THEN 3 \
                         WHEN x.represented = 16 THEN 4 \
                         WHEN x.represented = 32 THEN 5 \
                         WHEN x.represented = 64 THEN 6 \
                         WHEN x.represented = 128 THEN 7 \
                    END AS total_mask, \
                    CASE WHEN x.represented & 4 <> 0 THEN 2 \
                         WHEN x.represented & 8 <> 0 AND x.represented & 64 <> 0 THEN NULL \
                         WHEN x.represented & 8 <> 0 THEN 3 \
                         WHEN x.represented & 64 <> 0 THEN 6 \
                         WHEN x.represented & 128 <> 0 THEN 7 \
                    END AS country_mask, \
                    CASE WHEN x.represented & 2 <> 0 THEN 1 \
                         WHEN x.represented & 8 <> 0 AND x.represented & 32 <> 0 THEN NULL \
                         WHEN x.represented & 8 <> 0 THEN 3 \
                         WHEN x.represented & 32 <> 0 THEN 5 \
                         WHEN x.represented & 128 <> 0 THEN 7 \
                    END AS institution_mask \
             FROM ( \
                 SELECT m.*, \
                        bit_or(1 << m.mask) OVER (PARTITION BY m.work_id, m.platform_id, \
                                                               m.measure_id, m.month_start) \
                            AS represented \
                 FROM ( \
                     SELECT d.work_id, d.platform_id, d.measure_id, d.period_start AS month_start, \
                            d.span, d.country_code, d.institution_id, d.value, \
                            (d.publication_id IS NOT NULL)::int * 4 \
                                + (d.country_code IS NOT NULL)::int * 2 \
                                + (d.institution_id IS NOT NULL)::int AS mask \
                     FROM cand d \
                     WHERE d.in_keys \
                       AND NOT EXISTS (SELECT 1 FROM conflicted c \
                                       WHERE c.work_id = d.work_id \
                                         AND c.platform_id = d.platform_id \
                                         AND c.measure_id = d.measure_id \
                                         AND c.period_start = d.period_start) \
                 ) m \
             ) x \
         )"
    };
}

/// The month rows: one per cell holding a served month.
macro_rules! month_insert_sql {
    () => {
        "INSERT INTO public.metric_native_month \
             (work_id, platform_id, measure_id, month_start, span_months, value, live, \
              requires_country_coverage, requires_institution_coverage, \
              country_ambiguous, institution_ambiguous) \
         SELECT s.work_id, s.platform_id, s.measure_id, s.month_start, CASE WHEN MIN(s.span) = MAX(s.span) THEN MIN(s.span) END, \
                (SUM(s.value) FILTER (WHERE s.mask = s.total_mask))::bigint, \
                bool_or(s.value <> 0), \
                COALESCE(bool_or(s.mask & 2 <> 0) FILTER (WHERE s.mask = s.total_mask), FALSE), \
                COALESCE(bool_or(s.mask & 1 <> 0) FILTER (WHERE s.mask = s.total_mask), FALSE), \
                bool_or(s.represented & 4 = 0 AND s.represented & 8 <> 0 \
                        AND s.represented & 64 <> 0), \
                bool_or(s.represented & 2 = 0 AND s.represented & 8 <> 0 \
                        AND s.represented & 32 <> 0) \
         FROM served s \
         GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start"
    };
}

/// The country rows of each cell's unique least country-bearing mask.
macro_rules! country_insert_sql {
    () => {
        "INSERT INTO public.metric_native_country_month \
             (work_id, platform_id, measure_id, month_start, span_months, country_code, value, \
              requires_institution_coverage) \
         SELECT s.work_id, s.platform_id, s.measure_id, s.month_start, CASE WHEN MIN(s.span) = MAX(s.span) THEN MIN(s.span) END, s.country_code, \
                SUM(s.value)::bigint, \
                bool_or(s.mask & 1 <> 0) \
         FROM served s \
         WHERE s.mask = s.country_mask \
         GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start, s.country_code"
    };
}

/// The institution rows of each cell's unique least institution-bearing mask.
macro_rules! institution_insert_sql {
    () => {
        "INSERT INTO public.metric_native_institution_month \
             (work_id, platform_id, measure_id, month_start, span_months, institution_id, value, \
              requires_country_coverage) \
         SELECT s.work_id, s.platform_id, s.measure_id, s.month_start, CASE WHEN MIN(s.span) = MAX(s.span) THEN MIN(s.span) END, s.institution_id, \
                SUM(s.value)::bigint, \
                bool_or(s.mask & 2 <> 0) \
         FROM served s \
         WHERE s.mask = s.institution_mask \
         GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start, s.institution_id"
    };
}

/// The refusal rows: one per period holding an active reporting period or an
/// inconsistent record, `inconsistent` when it holds an inconsistent one.
/// Served months and withdrawn records are not refusals.
macro_rules! refusal_insert_sql {
    () => {
        "INSERT INTO public.metric_native_refusal \
             (work_id, platform_id, measure_id, period_start, period_end, inconsistent) \
         SELECT c.work_id, c.platform_id, c.measure_id, c.period_start, c.period_end, \
                bool_or(NOT c.active_reporting_period) \
         FROM ( \
             SELECT d.work_id, d.platform_id, d.measure_id, d.period_start, d.period_end, \
                    (d.reporting_grain = 'REPORTING_PERIOD' \
                     AND COALESCE(d.status = 'CURRENT', FALSE)) AS active_reporting_period, \
                    (EXISTS (SELECT 1 FROM cand s WHERE s.record_id = d.record_id) \
                     AND NOT EXISTS (SELECT 1 FROM conflicted c \
                                     WHERE c.work_id = d.work_id \
                                       AND c.platform_id = d.platform_id \
                                       AND c.measure_id = d.measure_id \
                                       AND c.period_start = d.period_start)) \
                        AS served_month, \
                    (COALESCE(d.status = 'RETRACTED', FALSE) \
                     AND NOT EXISTS (SELECT 1 FROM public.metric_record_revision o \
                                     WHERE o.record_id = d.record_id \
                                       AND o.status = 'CURRENT')) AS withdrawn \
             FROM records d \
             WHERE d.in_keys \
         ) c \
         WHERE NOT c.served_month \
           AND NOT c.withdrawn \
         GROUP BY c.work_id, c.platform_id, c.measure_id, c.period_start, c.period_end"
    };
}

/// One delete of the keys' rows from one table, whose period column is
/// `$column`.
macro_rules! key_delete_sql {
    ($table:literal, $column:literal) => {
        concat!(
            "WITH ",
            key_scope_sql!(),
            " DELETE FROM public.",
            $table,
            " n \
             USING keys k \
             WHERE n.work_id = k.work_id \
               AND n.platform_id = k.platform_id \
               AND n.measure_id = k.measure_id \
               AND n.",
            $column,
            " = k.period_start"
        )
    };
}

/// One delete of the works' rows from one table.
macro_rules! work_delete_sql {
    ($table:literal) => {
        concat!(
            "DELETE FROM public.",
            $table,
            " n WHERE n.work_id = ANY($1)"
        )
    };
}

/// Recompute statements 1 to 4 of 8: delete every row of the keys.
pub(crate) const NATIVE_MONTH_DELETE_SQL: &str =
    key_delete_sql!("metric_native_month", "month_start");
pub(crate) const NATIVE_COUNTRY_MONTH_DELETE_SQL: &str =
    key_delete_sql!("metric_native_country_month", "month_start");
pub(crate) const NATIVE_INSTITUTION_MONTH_DELETE_SQL: &str =
    key_delete_sql!("metric_native_institution_month", "month_start");
pub(crate) const NATIVE_REFUSAL_DELETE_SQL: &str =
    key_delete_sql!("metric_native_refusal", "period_start");

/// Recompute statements 5 to 8 of 8: the keys' rows again, from their records.
pub(crate) const NATIVE_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    key_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    month_insert_sql!()
);
pub(crate) const NATIVE_COUNTRY_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    key_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    country_insert_sql!()
);
pub(crate) const NATIVE_INSTITUTION_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    key_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    institution_insert_sql!()
);
pub(crate) const NATIVE_REFUSAL_INSERT_SQL: &str = concat!(
    "WITH ",
    key_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    " ",
    refusal_insert_sql!()
);

/// The eight recompute statements, in execution order.
pub(crate) const RECOMPUTE_STATEMENTS: [&str; 8] = [
    NATIVE_MONTH_DELETE_SQL,
    NATIVE_COUNTRY_MONTH_DELETE_SQL,
    NATIVE_INSTITUTION_MONTH_DELETE_SQL,
    NATIVE_REFUSAL_DELETE_SQL,
    NATIVE_MONTH_INSERT_SQL,
    NATIVE_COUNTRY_MONTH_INSERT_SQL,
    NATIVE_INSTITUTION_MONTH_INSERT_SQL,
    NATIVE_REFUSAL_INSERT_SQL,
];

/// Repair statements 1 to 4 of 8: delete every row of the works `$1`.
pub(crate) const REPAIR_MONTH_DELETE_SQL: &str = work_delete_sql!("metric_native_month");
pub(crate) const REPAIR_COUNTRY_MONTH_DELETE_SQL: &str =
    work_delete_sql!("metric_native_country_month");
pub(crate) const REPAIR_INSTITUTION_MONTH_DELETE_SQL: &str =
    work_delete_sql!("metric_native_institution_month");
pub(crate) const REPAIR_REFUSAL_DELETE_SQL: &str = work_delete_sql!("metric_native_refusal");

/// Repair statements 5 to 8 of 8: the works' rows again, by the same rule
/// over every non-`DAY` key of the works.
pub(crate) const REPAIR_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    work_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    month_insert_sql!()
);
pub(crate) const REPAIR_COUNTRY_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    work_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    country_insert_sql!()
);
pub(crate) const REPAIR_INSTITUTION_MONTH_INSERT_SQL: &str = concat!(
    "WITH ",
    work_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    ", ",
    served_sql!(),
    " ",
    institution_insert_sql!()
);
pub(crate) const REPAIR_REFUSAL_INSERT_SQL: &str = concat!(
    "WITH ",
    work_scope_sql!(),
    ", ",
    key_records_sql!(),
    ", ",
    cand_sql!(),
    " ",
    refusal_insert_sql!()
);

/// The eight repair statements, in execution order.
pub(crate) const REPAIR_STATEMENTS: [&str; 8] = [
    REPAIR_MONTH_DELETE_SQL,
    REPAIR_COUNTRY_MONTH_DELETE_SQL,
    REPAIR_INSTITUTION_MONTH_DELETE_SQL,
    REPAIR_REFUSAL_DELETE_SQL,
    REPAIR_MONTH_INSERT_SQL,
    REPAIR_COUNTRY_MONTH_INSERT_SQL,
    REPAIR_INSTITUTION_MONTH_INSERT_SQL,
    REPAIR_REFUSAL_INSERT_SQL,
];

/// Recompute the projection rows of exactly `keys`, set-wise, in the caller's
/// transaction, and return how many rows the four inserts wrote.
///
/// Runs the eight [`RECOMPUTE_STATEMENTS`] in order, each over the whole key
/// set at once, so the count is eight for one key or fifty. An empty set runs
/// no statement. It takes no lock beyond its own rows: the caller already
/// holds every work row of `keys` `FOR UPDATE`. Any failure, an overflowing
/// sum included, is returned so the caller's transaction rolls back.
pub(crate) fn recompute(
    connection: &mut PgConnection,
    keys: &NativeMonthKeys,
) -> Result<usize, DieselError> {
    if keys.is_empty() {
        return Ok(0);
    }
    let (works, platforms, measures, starts) = keys.columns();
    let mut written = 0;
    for (index, statement) in RECOMPUTE_STATEMENTS.iter().enumerate() {
        let rows = sql_query(*statement)
            .bind::<Array<SqlUuid>, _>(&works)
            .bind::<Array<SqlUuid>, _>(&platforms)
            .bind::<Array<SqlUuid>, _>(&measures)
            .bind::<Array<Date>, _>(&starts)
            .execute(connection)?;
        if index >= 4 {
            written += rows;
        }
    }
    Ok(written)
}

/// Delete and re-derive every projection row of `work_ids` in the caller's
/// transaction, and return how many rows the four inserts wrote.
fn rebuild_works(connection: &mut PgConnection, work_ids: &[Uuid]) -> Result<usize, DieselError> {
    let mut written = 0;
    for (index, statement) in REPAIR_STATEMENTS.iter().enumerate() {
        let rows = sql_query(*statement)
            .bind::<Array<SqlUuid>, _>(work_ids)
            .execute(connection)?;
        if index >= 4 {
            written += rows;
        }
    }
    Ok(written)
}

/// What one `recomputeMetricNativeMonths` call did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, juniper::GraphQLObject)]
#[graphql(description = "What one recomputation of a publisher's native-month projection did")]
pub struct MetricNativeMonthRecomputeResult {
    /// The works whose rows were recomputed.
    #[graphql(description = "How many works under the publisher's imprints were recomputed")]
    pub works: i32,
    /// The rows the recompute wrote to the four tables.
    #[graphql(description = "How many native-month projection rows the recomputation wrote")]
    pub rows: i32,
}

/// Repair the projection of every work now under `publisher_id`'s imprints
/// (`recomputeMetricNativeMonths`).
///
/// The works are read once, without a lock, and processed in ascending id, in
/// chunks of at most [`REPAIR_CHUNK_WORKS`]. Each chunk is one transaction:
/// it locks its works `FOR UPDATE` in ascending id ([`lock_works`]), deletes
/// their rows from the four tables, and inserts them again by the projection
/// rule. A work deleted since the read is skipped. A failed chunk (a deadlock
/// with a publisher or imprint change included) leaves the earlier chunks
/// committed and exact, and is retried by calling this again: the result is
/// idempotent. It runs beside ingestion and reads, and changes no canonical
/// table. An unknown publisher fails as not found, before any write.
///
/// Authorization is the caller's: the resolver's superuser guard runs first.
pub(crate) fn recompute_metric_native_months(
    db: &PgPool,
    publisher_id: Uuid,
) -> ThothResult<MetricNativeMonthRecomputeResult> {
    let mut connection = db.get()?;
    publisher::table
        .find(publisher_id)
        .select(publisher::publisher_id)
        .first::<Uuid>(&mut connection)
        .optional()?
        .ok_or(ThothError::EntityNotFound)?;
    let work_ids: Vec<Uuid> = work::table
        .inner_join(imprint::table)
        .filter(imprint::publisher_id.eq(publisher_id))
        .select(work::work_id)
        .order(work::work_id.asc())
        .load(&mut connection)?;

    let (mut works, mut rows) = (0_usize, 0_usize);
    for chunk in work_ids.chunks(REPAIR_CHUNK_WORKS) {
        let (chunk_works, chunk_rows) = connection.transaction(|connection| {
            let requested: BTreeSet<Uuid> = chunk.iter().copied().collect();
            let locked: Vec<Uuid> = lock_works(connection, &requested)?.into_iter().collect();
            let written = rebuild_works(connection, &locked)?;
            Ok::<_, DieselError>((locked.len(), written))
        })?;
        works += chunk_works;
        rows += chunk_rows;
    }
    let count = |n: usize| {
        i32::try_from(n)
            .map_err(|_| ThothError::InternalError("a recompute count does not fit an Int".into()))
    };
    Ok(MetricNativeMonthRecomputeResult {
        works: count(works)?,
        rows: count(rows)?,
    })
}

#[cfg(test)]
pub(crate) mod tests;
