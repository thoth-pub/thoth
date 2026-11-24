//! The judgement of an import's coverage evidence for checkpoint progress
//! (`MET-HIST-01-CKPT`).
//!
//! [`update_metric_source_checkpoint`](super::update_metric_source_checkpoint)
//! reads the coverage rows of the terminal import it records, grouped, and
//! hands them to [`judge_coverage`], a pure function with no database access,
//! so every case of the rule below is a table test.
//!
//! The import's envelope is the one-day period `[e0, e1)` that
//! `beginMetricImport` enforces. A coverage row has a platform code, a measure
//! code, a half-open period `[s, t)` and a status. A row is **accepted** when
//! rule A or rule B holds.
//!
//! - **Rule A** (judged from the row alone, never from the registry): the
//!   status is `COMPLETE` and the period is exactly the envelope.
//! - **Rule B**, the MONTH rule, evaluated only when the caller enables it
//!   (the locked source's driver key is not `cloudfront`): the registry lists
//!   `MONTH` for the row's platform-measure pair, the row starts on the
//!   envelope day, its period is the calendar month that starts on that day
//!   (the period rule that observations of the `MONTH` grain obey), and either
//!   the status is `COMPLETE`, or the status is `PARTIAL` and the platform
//!   code, measure code and month start are one of [`PARTIAL_ALLOWLIST`].
//!
//! The verdict has two parts. `accepted` is true when the import has at least
//! one coverage row and every row is accepted. `strict_day` is true when it has
//! at least one coverage row and every row satisfies rule A; it is what the
//! CloudFront quarantine-only exception uses, so that rule B never reaches it.
//! Both are false for zero rows.
//!
//! The evidence is also checked against itself: the group counts must add up to
//! the plain row count, every group needs a platform code and a measure code
//! and every status must be one of the three known values. Anything else is
//! [`Inconsistent`], which the caller turns into
//! `INTERNAL_STATE_INCONSISTENCY` with no write.

use std::str::FromStr;

use chrono::NaiveDate;
use diesel::sql_types::{BigInt, Bool, Date as SqlDate, Nullable, Text};

use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_ingestion::period_matches_grain;
use crate::model::metric_platform_measure::MetricReportingGrain;

/// One month of one platform-measure pair for which a `PARTIAL` coverage row is
/// accepted as checkpoint progress.
///
/// The month's end is the next 1st, computed and never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PartialMonth {
    /// The stable platform code, compared exactly.
    pub(super) platform_code: &'static str,
    /// The stable measure code, compared exactly.
    pub(super) measure_code: &'static str,
    /// The year of the month's first day.
    pub(super) year: i32,
    /// The month of the year, `1` to `12`.
    pub(super) month: u32,
}

impl PartialMonth {
    /// Whether a coverage row for `platform_code` and `measure_code` over
    /// `[period_start, period_end)` is exactly this entry's calendar month.
    fn matches(
        &self,
        platform_code: &str,
        measure_code: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> bool {
        let Some(start) = NaiveDate::from_ymd_opt(self.year, self.month, 1) else {
            return false;
        };
        let end = if self.month == 12 {
            NaiveDate::from_ymd_opt(self.year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(self.year, self.month + 1, 1)
        };
        self.platform_code == platform_code
            && self.measure_code == measure_code
            && start == period_start
            && end == Some(period_end)
    }
}

/// The only `PARTIAL` coverage a checkpoint accepts: OAPEN downloads in the
/// months 2026-01 and 2026-08, whose figures are estimated
/// (`MET-HIST-02`, CTO decisions of 2026-11-23 and 2026-11-24).
///
/// Codes compare exactly: no case folding, no trimming. Every account on the
/// platform matches, because the driver key is deliberately not part of the
/// key. Extending the list is a code change under its own release.
pub(super) const PARTIAL_ALLOWLIST: [PartialMonth; 2] = [
    PartialMonth {
        platform_code: "oapen",
        measure_code: "downloads",
        year: 2026,
        month: 1,
    },
    PartialMonth {
        platform_code: "oapen",
        measure_code: "downloads",
        year: 2026,
        month: 8,
    },
];

/// Whether `PARTIAL` coverage of `[period_start, period_end)` for the pair is
/// on the allowlist.
fn partial_is_allowed(
    platform_code: &str,
    measure_code: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> bool {
    PARTIAL_ALLOWLIST
        .iter()
        .any(|entry| entry.matches(platform_code, measure_code, period_start, period_end))
}

/// The coverage rows of one import that share a platform, a measure, a period
/// and a status, with the registry's verdict on the pair's `MONTH` grain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CoverageGroup {
    /// The platform code; `None` only if the platform row is missing.
    pub(super) platform_code: Option<String>,
    /// The measure code; `None` only if the measure row is missing.
    pub(super) measure_code: Option<String>,
    /// The first day of the covered period.
    pub(super) period_start: NaiveDate,
    /// The exclusive end of the covered period.
    pub(super) period_end: NaiveDate,
    /// The stored status as text, so an unknown value cannot surface as a
    /// database decode failure.
    pub(super) status: String,
    /// Whether the registry pair row exists and lists `MONTH`; a missing pair
    /// row is `false`.
    pub(super) month_supported: bool,
    /// How many coverage rows the group holds.
    pub(super) rows: i64,
}

/// One row of the grouped coverage query: a [`CoverageGroup`] and the plain
/// number of the import's coverage rows, read in the same statement.
#[derive(diesel::QueryableByName)]
pub(super) struct CoverageGroupRow {
    #[diesel(sql_type = Nullable<Text>)]
    platform_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    measure_code: Option<String>,
    #[diesel(sql_type = SqlDate)]
    period_start: NaiveDate,
    #[diesel(sql_type = SqlDate)]
    period_end: NaiveDate,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Bool)]
    month_supported: bool,
    #[diesel(sql_type = BigInt)]
    rows: i64,
    #[diesel(sql_type = BigInt)]
    total: i64,
}

impl CoverageGroupRow {
    /// The plain row count the statement reported, which is the same on every
    /// row of one statement.
    pub(super) fn total(&self) -> i64 {
        self.total
    }
}

impl From<CoverageGroupRow> for CoverageGroup {
    fn from(row: CoverageGroupRow) -> Self {
        CoverageGroup {
            platform_code: row.platform_code,
            measure_code: row.measure_code,
            period_start: row.period_start,
            period_end: row.period_end,
            status: row.status,
            month_supported: row.month_supported,
            rows: row.rows,
        }
    }
}

/// The verdict on one import's coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CheckpointCoverage {
    /// At least one row, and every row satisfies rule A or, when it is
    /// enabled, rule B.
    pub(super) accepted: bool,
    /// At least one row, and every row satisfies rule A.
    pub(super) strict_day: bool,
}

/// The evidence contradicts itself: the group counts do not add up to the plain
/// row count, a group has no platform or measure code, or a status is not one
/// of the three known values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Inconsistent;

/// Judge `groups`, the grouped coverage of one import of `total` rows, against
/// the import's one-day `envelope` `[e0, e1)`.
///
/// `rule_b_enabled` is true when the locked source's driver key is not
/// `cloudfront`; with it false the verdict is rule A alone and `accepted`
/// equals `strict_day`. Every group is checked before the verdict is given, so
/// inconsistent evidence is never masked by a group that already failed.
pub(super) fn judge_coverage(
    groups: &[CoverageGroup],
    total: i64,
    envelope: (NaiveDate, NaiveDate),
    rule_b_enabled: bool,
) -> Result<CheckpointCoverage, Inconsistent> {
    let (envelope_start, envelope_end) = envelope;
    let mut counted: i64 = 0;
    let mut accepted = true;
    let mut strict_day = true;
    for group in groups {
        if group.rows < 1 {
            return Err(Inconsistent);
        }
        counted = counted.checked_add(group.rows).ok_or(Inconsistent)?;
        let (Some(platform_code), Some(measure_code)) = (
            group.platform_code.as_deref(),
            group.measure_code.as_deref(),
        ) else {
            return Err(Inconsistent);
        };
        let status = MetricCoverageStatus::from_str(&group.status).map_err(|_| Inconsistent)?;
        let (start, end) = (group.period_start, group.period_end);

        let rule_a = status == MetricCoverageStatus::Complete
            && start == envelope_start
            && end == envelope_end;
        let rule_b = rule_b_enabled
            && group.month_supported
            && start == envelope_start
            && period_matches_grain(MetricReportingGrain::Month, start, end)
            && match status {
                MetricCoverageStatus::Complete => true,
                MetricCoverageStatus::Partial => {
                    partial_is_allowed(platform_code, measure_code, start, end)
                }
                MetricCoverageStatus::Unknown => false,
            };
        accepted &= rule_a || rule_b;
        strict_day &= rule_a;
    }
    if counted != total {
        return Err(Inconsistent);
    }
    if total == 0 {
        return Ok(CheckpointCoverage {
            accepted: false,
            strict_day: false,
        });
    }
    Ok(CheckpointCoverage {
        accepted,
        strict_day,
    })
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    /// The one-day envelope that starts on `start`.
    fn envelope(start: NaiveDate) -> (NaiveDate, NaiveDate) {
        (start, start.succ_opt().unwrap())
    }

    /// The first day of the month after `start`'s month.
    fn next_month(start: NaiveDate) -> NaiveDate {
        if start.month() == 12 {
            date(start.year() + 1, 1, 1)
        } else {
            date(start.year(), start.month() + 1, 1)
        }
    }

    fn group(
        platform: &str,
        measure: &str,
        start: NaiveDate,
        end: NaiveDate,
        status: &str,
        month_supported: bool,
        rows: i64,
    ) -> CoverageGroup {
        CoverageGroup {
            platform_code: Some(platform.into()),
            measure_code: Some(measure.into()),
            period_start: start,
            period_end: end,
            status: status.into(),
            month_supported,
            rows,
        }
    }

    /// A group of one row on a non-allowlisted pair that the registry lists
    /// as `MONTH`-capable.
    fn other(start: NaiveDate, end: NaiveDate, status: &str) -> CoverageGroup {
        group("archive", "usage", start, end, status, true, 1)
    }

    /// A group of one `oapen`/`downloads` row on a `MONTH`-capable pair.
    fn oapen(start: NaiveDate, end: NaiveDate, status: &str) -> CoverageGroup {
        group("oapen", "downloads", start, end, status, true, 1)
    }

    fn verdict(
        groups: &[CoverageGroup],
        envelope_start: NaiveDate,
        rule_b_enabled: bool,
    ) -> (bool, bool) {
        let total = groups.iter().map(|group| group.rows).sum();
        let verdict = judge_coverage(groups, total, envelope(envelope_start), rule_b_enabled)
            .expect("consistent evidence");
        (verdict.accepted, verdict.strict_day)
    }

    #[test]
    fn the_partial_allowlist_is_exactly_the_two_estimated_oapen_download_months() {
        assert_eq!(
            PARTIAL_ALLOWLIST,
            [
                PartialMonth {
                    platform_code: "oapen",
                    measure_code: "downloads",
                    year: 2026,
                    month: 1,
                },
                PartialMonth {
                    platform_code: "oapen",
                    measure_code: "downloads",
                    year: 2026,
                    month: 8,
                },
            ]
        );
        // The entries select exactly their own calendar months.
        for (month, next) in [
            (date(2026, 1, 1), date(2026, 2, 1)),
            (date(2026, 8, 1), date(2026, 9, 1)),
        ] {
            assert!(partial_is_allowed("oapen", "downloads", month, next));
        }
        for (month, next) in [
            (date(2025, 1, 1), date(2025, 2, 1)),
            (date(2026, 2, 1), date(2026, 3, 1)),
            (date(2026, 7, 1), date(2026, 8, 1)),
            (date(2026, 9, 1), date(2026, 10, 1)),
            (date(2027, 1, 1), date(2027, 2, 1)),
            (date(2027, 8, 1), date(2027, 9, 1)),
        ] {
            assert!(!partial_is_allowed("oapen", "downloads", month, next));
        }
        // A period that is not exactly the month is not the entry, either.
        assert!(!partial_is_allowed(
            "oapen",
            "downloads",
            date(2026, 1, 1),
            date(2026, 1, 2)
        ));
        assert!(!partial_is_allowed(
            "oapen",
            "downloads",
            date(2026, 1, 1),
            date(2026, 3, 1)
        ));
        assert!(!partial_is_allowed(
            "oapen",
            "downloads",
            date(2026, 1, 2),
            date(2026, 2, 1)
        ));
        // Codes compare exactly.
        for (platform, measure) in [
            ("OAPEN", "downloads"),
            ("oapen", "Downloads"),
            ("oapen ", "downloads"),
            (" oapen", "downloads"),
            ("oapen", "downloads "),
            ("oapen", "download"),
            ("oapen", "title_sessions"),
            ("cf", "downloads"),
            ("", ""),
        ] {
            assert!(
                !partial_is_allowed(platform, measure, date(2026, 1, 1), date(2026, 2, 1)),
                "{platform:?} {measure:?}"
            );
        }
    }

    #[test]
    fn rule_a_is_the_unchanged_day_rule_and_never_reads_the_registry() {
        let day = date(2026, 3, 10);
        let next = day.succ_opt().unwrap();
        for month_supported in [true, false] {
            for rule_b_enabled in [true, false] {
                let exact = group(
                    "cf",
                    "title_sessions",
                    day,
                    next,
                    "COMPLETE",
                    month_supported,
                    3,
                );
                assert_eq!(
                    verdict(std::slice::from_ref(&exact), day, rule_b_enabled),
                    (true, true)
                );
                assert_eq!(
                    verdict(
                        &[
                            exact.clone(),
                            group("cf", "other", day, next, "COMPLETE", month_supported, 1)
                        ],
                        day,
                        rule_b_enabled
                    ),
                    (true, true)
                );
            }
        }
        // Each deviation from the exact COMPLETE day row fails, and fails
        // both parts of the verdict (no MONTH-capable pair is involved).
        let two_days = next.succ_opt().unwrap();
        let cases = [
            ("PARTIAL", group("cf", "m", day, next, "PARTIAL", false, 1)),
            ("UNKNOWN", group("cf", "m", day, next, "UNKNOWN", false, 1)),
            (
                "longer period",
                group("cf", "m", day, two_days, "COMPLETE", false, 1),
            ),
            (
                "previous day",
                group(
                    "cf",
                    "m",
                    day.pred_opt().unwrap(),
                    day,
                    "COMPLETE",
                    false,
                    1,
                ),
            ),
            (
                "next day",
                group("cf", "m", next, two_days, "COMPLETE", false, 1),
            ),
        ];
        for (label, bad) in cases {
            for rule_b_enabled in [true, false] {
                assert_eq!(
                    verdict(std::slice::from_ref(&bad), day, rule_b_enabled),
                    (false, false),
                    "{label}"
                );
                let exact = group("cf", "title_sessions", day, next, "COMPLETE", false, 1);
                assert_eq!(
                    verdict(&[exact, bad.clone()], day, rule_b_enabled),
                    (false, false),
                    "{label} beside an exact row"
                );
            }
        }
    }

    #[test]
    fn rule_b_accepts_each_complete_calendar_month_that_starts_on_the_envelope_day() {
        for (start, label) in [
            (date(2026, 1, 1), "31 days"),
            (date(2026, 2, 1), "28 days"),
            (date(2028, 2, 1), "29 days, a leap year"),
            (date(2026, 4, 1), "30 days"),
            (date(2026, 12, 1), "December to January"),
            (date(2023, 12, 1), "December to January, another year"),
        ] {
            let month = other(start, next_month(start), "COMPLETE");
            assert_eq!(
                verdict(std::slice::from_ref(&month), start, true),
                (true, false),
                "{label}"
            );
            // The CloudFront case: rule B is never evaluated.
            assert_eq!(verdict(&[month], start, false), (false, false), "{label}");
        }
        // Several measures and months' worth of rows in one unit.
        let start = date(2026, 5, 1);
        let end = next_month(start);
        let rows = [
            group("archive", "usage", start, end, "COMPLETE", true, 4),
            group("archive", "views", start, end, "COMPLETE", true, 2),
        ];
        assert_eq!(verdict(&rows, start, true), (true, false));
    }

    #[test]
    fn rule_b_refuses_every_row_that_is_not_that_exact_month() {
        let start = date(2026, 3, 1);
        let end = date(2026, 4, 1);
        let cases = [
            ("the previous month", date(2026, 2, 1), start),
            ("the next month", end, date(2026, 5, 1)),
            ("two months wide", start, date(2026, 5, 1)),
            (
                "one day wide, the day after the envelope",
                date(2026, 3, 2),
                date(2026, 3, 3),
            ),
            (
                "a month wide but shifted one day later",
                date(2026, 3, 2),
                date(2026, 4, 2),
            ),
            (
                "a month wide but shifted one day earlier",
                date(2026, 2, 28),
                date(2026, 3, 31),
            ),
            ("ending a day early", start, date(2026, 3, 31)),
            ("ending a day late", start, date(2026, 4, 2)),
            ("a year wide", date(2026, 1, 1), date(2027, 1, 1)),
        ];
        for (label, row_start, row_end) in cases {
            let row = other(row_start, row_end, "COMPLETE");
            assert_eq!(
                verdict(std::slice::from_ref(&row), start, true),
                (false, false),
                "{label}"
            );
            // Not even beside a row that is accepted.
            let exact = other(start, end, "COMPLETE");
            assert_eq!(
                verdict(&[exact, row], start, true),
                (false, false),
                "{label} beside an exact month"
            );
        }
        // A one-day COMPLETE row on the envelope day is rule A, also in a
        // MONTH-capable pair, and it is not a PARTIAL exception.
        assert_eq!(
            verdict(&[other(start, date(2026, 3, 2), "COMPLETE")], start, true),
            (true, true)
        );
        assert_eq!(
            verdict(&[other(start, date(2026, 3, 2), "PARTIAL")], start, true),
            (false, false)
        );
        // A calendar month, but the envelope starts on another day.
        for envelope_start in [date(2026, 3, 2), date(2026, 3, 31), date(2026, 2, 28)] {
            assert_eq!(
                verdict(&[other(start, end, "COMPLETE")], envelope_start, true),
                (false, false),
                "envelope {envelope_start}"
            );
        }
        // An envelope on the 1st of another month.
        assert_eq!(
            verdict(&[other(start, end, "COMPLETE")], date(2026, 4, 1), true),
            (false, false)
        );
        // A month-start envelope but a row that is not a month starting on the
        // 1st (a month's worth of days from the 2nd, the envelope on the 2nd).
        assert_eq!(
            verdict(
                &[other(date(2026, 3, 2), date(2026, 4, 2), "COMPLETE")],
                date(2026, 3, 2),
                true
            ),
            (false, false)
        );
    }

    #[test]
    fn rule_b_needs_a_month_capable_registry_pair() {
        let start = date(2026, 3, 1);
        let end = date(2026, 4, 1);
        let unsupported = group("archive", "usage", start, end, "COMPLETE", false, 1);
        assert_eq!(
            verdict(std::slice::from_ref(&unsupported), start, true),
            (false, false)
        );
        // A missing pair row reads as unsupported (the caller's `COALESCE`),
        // and it does not change the verdict of a rule A row.
        let exact_day = group(
            "archive",
            "usage",
            start,
            start.succ_opt().unwrap(),
            "COMPLETE",
            false,
            1,
        );
        assert_eq!(
            verdict(std::slice::from_ref(&exact_day), start, true),
            (true, true)
        );
        assert_eq!(
            verdict(&[exact_day, unsupported], start, true),
            (false, false)
        );
        // One unsupported pair beside a supported one.
        let supported = group("archive", "views", start, end, "COMPLETE", true, 1);
        let unsupported = group("archive", "usage", start, end, "COMPLETE", false, 1);
        assert_eq!(
            verdict(&[supported, unsupported], start, true),
            (false, false)
        );
    }

    #[test]
    fn a_partial_row_is_accepted_only_for_the_allowlisted_pair_and_month() {
        for (month, next) in [
            (date(2026, 1, 1), date(2026, 2, 1)),
            (date(2026, 8, 1), date(2026, 9, 1)),
        ] {
            assert_eq!(
                verdict(&[oapen(month, next, "PARTIAL")], month, true),
                (true, false)
            );
            // Beside a COMPLETE row of another measure.
            let beside = [
                oapen(month, next, "PARTIAL"),
                group("oapen", "views", month, next, "COMPLETE", true, 1),
            ];
            assert_eq!(verdict(&beside, month, true), (true, false));
            // CloudFront: rule B never runs.
            assert_eq!(
                verdict(&[oapen(month, next, "PARTIAL")], month, false),
                (false, false)
            );
            // Not MONTH-capable in the registry.
            assert_eq!(
                verdict(
                    &[group(
                        "oapen",
                        "downloads",
                        month,
                        next,
                        "PARTIAL",
                        false,
                        1
                    )],
                    month,
                    true
                ),
                (false, false)
            );
            // A one-day PARTIAL inside an allowlisted month.
            assert_eq!(
                verdict(
                    &[oapen(month, month.succ_opt().unwrap(), "PARTIAL")],
                    month,
                    true
                ),
                (false, false)
            );
            // Beside an UNKNOWN row.
            assert_eq!(
                verdict(
                    &[
                        oapen(month, next, "PARTIAL"),
                        group("oapen", "views", month, next, "UNKNOWN", true, 1)
                    ],
                    month,
                    true
                ),
                (false, false)
            );
            // Beside another, non-allowlisted PARTIAL row.
            assert_eq!(
                verdict(
                    &[
                        oapen(month, next, "PARTIAL"),
                        group("oapen", "views", month, next, "PARTIAL", true, 1)
                    ],
                    month,
                    true
                ),
                (false, false)
            );
            // UNKNOWN is never accepted, on the allowlist or not.
            assert_eq!(
                verdict(&[oapen(month, next, "UNKNOWN")], month, true),
                (false, false)
            );
        }
        // Other months, other measures and other platforms.
        for (platform, measure, month) in [
            ("oapen", "downloads", date(2026, 2, 1)),
            ("oapen", "downloads", date(2026, 7, 1)),
            ("oapen", "downloads", date(2026, 9, 1)),
            ("oapen", "downloads", date(2025, 1, 1)),
            ("oapen", "downloads", date(2027, 1, 1)),
            ("oapen", "views", date(2026, 1, 1)),
            ("oapen", "title_sessions", date(2026, 8, 1)),
            ("ojs", "downloads", date(2026, 1, 1)),
            ("cf", "downloads", date(2026, 8, 1)),
            ("OAPEN", "downloads", date(2026, 1, 1)),
            ("oapen", "DOWNLOADS", date(2026, 1, 1)),
            ("oapen ", "downloads", date(2026, 1, 1)),
            ("oapen", " downloads", date(2026, 1, 1)),
        ] {
            let row = group(
                platform,
                measure,
                month,
                next_month(month),
                "PARTIAL",
                true,
                1,
            );
            assert_eq!(
                verdict(&[row], month, true),
                (false, false),
                "{platform:?} {measure:?} {month}"
            );
        }
    }

    #[test]
    fn unknown_never_advances_and_is_not_masked_by_other_rows() {
        let start = date(2026, 3, 1);
        let end = date(2026, 4, 1);
        assert_eq!(
            verdict(&[other(start, end, "UNKNOWN")], start, true),
            (false, false)
        );
        let day_end = start.succ_opt().unwrap();
        assert_eq!(
            verdict(&[other(start, day_end, "UNKNOWN")], start, true),
            (false, false)
        );
        assert_eq!(
            verdict(
                &[other(start, end, "COMPLETE"), other(start, end, "UNKNOWN")],
                start,
                true
            ),
            (false, false)
        );
    }

    #[test]
    fn zero_rows_are_neither_accepted_nor_strict() {
        for rule_b_enabled in [true, false] {
            let verdict = judge_coverage(&[], 0, envelope(date(2026, 3, 1)), rule_b_enabled);
            assert_eq!(
                verdict,
                Ok(CheckpointCoverage {
                    accepted: false,
                    strict_day: false
                })
            );
        }
    }

    #[test]
    fn strict_day_is_rule_a_alone() {
        let start = date(2026, 3, 1);
        let day_end = start.succ_opt().unwrap();
        let month_end = date(2026, 4, 1);
        let day = group("archive", "usage", start, day_end, "COMPLETE", true, 1);
        let month = group("archive", "views", start, month_end, "COMPLETE", true, 1);
        let partial = group(
            "oapen",
            "downloads",
            date(2026, 1, 1),
            date(2026, 2, 1),
            "PARTIAL",
            true,
            1,
        );
        // Day rows only: both.
        assert_eq!(
            verdict(std::slice::from_ref(&day), start, true),
            (true, true)
        );
        // A month row, alone or beside a day row: accepted, not strict.
        assert_eq!(
            verdict(std::slice::from_ref(&month), start, true),
            (true, false)
        );
        assert_eq!(verdict(&[day, month], start, true), (true, false));
        // An allowlisted PARTIAL row: accepted, not strict.
        assert_eq!(verdict(&[partial], date(2026, 1, 1), true), (true, false));
        // With rule B disabled the two parts are the same, always.
        for groups in [
            vec![group("cf", "m", start, day_end, "COMPLETE", true, 2)],
            vec![group("cf", "m", start, month_end, "COMPLETE", true, 2)],
            vec![
                group("cf", "m", start, day_end, "COMPLETE", true, 2),
                group("cf", "n", start, month_end, "COMPLETE", true, 1),
            ],
            vec![group("cf", "m", start, day_end, "PARTIAL", true, 1)],
        ] {
            let (accepted, strict) = verdict(&groups, start, false);
            assert_eq!(accepted, strict);
        }
    }

    #[test]
    fn evidence_that_contradicts_itself_is_inconsistent_whatever_else_it_says() {
        let start = date(2026, 3, 1);
        let day_end = start.succ_opt().unwrap();
        let env = envelope(start);
        let exact = group("cf", "m", start, day_end, "COMPLETE", false, 2);

        // The group counts add up to the total.
        assert!(judge_coverage(std::slice::from_ref(&exact), 2, env, false).is_ok());
        // A row the groups do not account for, or too many.
        assert_eq!(
            judge_coverage(std::slice::from_ref(&exact), 3, env, false),
            Err(Inconsistent)
        );
        assert_eq!(
            judge_coverage(std::slice::from_ref(&exact), 1, env, false),
            Err(Inconsistent)
        );
        assert_eq!(judge_coverage(&[], 1, env, false), Err(Inconsistent));
        assert_eq!(
            judge_coverage(std::slice::from_ref(&exact), 0, env, false),
            Err(Inconsistent)
        );
        assert_eq!(
            judge_coverage(std::slice::from_ref(&exact), -2, env, false),
            Err(Inconsistent)
        );
        // Several groups: the counts are summed.
        let second = group("cf", "n", start, day_end, "COMPLETE", false, 5);
        assert!(judge_coverage(&[exact.clone(), second.clone()], 7, env, false).is_ok());
        assert_eq!(
            judge_coverage(&[exact.clone(), second.clone()], 6, env, false),
            Err(Inconsistent)
        );
        // A group that holds no row, or a negative count.
        for rows in [0, -1] {
            let empty = CoverageGroup {
                rows,
                ..exact.clone()
            };
            assert_eq!(
                judge_coverage(&[empty], rows, env, false),
                Err(Inconsistent),
                "{rows}"
            );
        }
        let overflow = CoverageGroup {
            rows: i64::MAX,
            ..exact.clone()
        };
        assert_eq!(
            judge_coverage(&[overflow, exact.clone()], i64::MAX, env, false),
            Err(Inconsistent)
        );
        // A status that is none of the three values, in any spelling.
        for status in [
            "",
            "complete",
            "Complete",
            "COMPLETE ",
            " COMPLETE",
            "DONE",
            "NULL",
        ] {
            let bad = CoverageGroup {
                status: status.into(),
                ..exact.clone()
            };
            assert_eq!(
                judge_coverage(&[bad], 2, env, false),
                Err(Inconsistent),
                "{status:?}"
            );
        }
        // A missing platform or measure code.
        let no_platform = CoverageGroup {
            platform_code: None,
            ..exact.clone()
        };
        let no_measure = CoverageGroup {
            measure_code: None,
            ..exact.clone()
        };
        for bad in [no_platform, no_measure] {
            assert_eq!(judge_coverage(&[bad], 2, env, false), Err(Inconsistent));
        }
        // Inconsistency wins over a verdict that was already false.
        let failing = group("cf", "m", start, day_end, "UNKNOWN", false, 1);
        let bad_status = group("cf", "n", start, day_end, "DONE", false, 1);
        assert_eq!(
            judge_coverage(&[failing, bad_status], 2, env, false),
            Err(Inconsistent)
        );
    }
}
