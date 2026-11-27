//! Tests of the native-month projection (`MET-WP4-MONTH-01`): T15, the
//! projection's exactness through the real coordinator; T3, the migration;
//! and the register's checks, run as the register gives them.
//!
//! The oracle reads `metric_record` and its revisions with plain reads and
//! resolves each cell in Rust by the register's generic rule, sharing no SQL
//! with the writer or with V-NATIVE.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Datelike, Days, Months, NaiveDate};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{Array, BigInt as SqlBigInt, Bool, Date, Nullable, Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use thoth_errors::ThothError;
use uuid::Uuid;

use super::{
    recompute, recompute_metric_native_months, NativeMonthKey, NativeMonthKeys,
    RECOMPUTE_STATEMENTS, REPAIR_CHUNK_WORKS, REPAIR_STATEMENTS,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_dashboard::tests::{
    apply_all, coverage_sql, exec as dashboard_exec, import_sql, oracle_buckets,
};
use crate::model::metric_dashboard::{
    metric_dashboard_with_month_serving, MetricDashboard, MetricDashboardInput, MetricMonthServing,
    MetricReadError, MetricSelectorInput, MetricTimelineGrain, MetricWarningCode,
    NATIVE_MONTHS_SQL,
};
use crate::model::metric_identifier_quarantine_reconciliation::reconcile_metric_identifier_quarantine;
use crate::model::metric_ingestion::{
    ingest_metric_batch, MetricIngestionBatch, MetricIngestionOutcome, NormalizedMetricObservation,
    SUPPORTED_SCHEMA_VERSION,
};
use crate::model::metric_platform::tests::setup_registry_db;
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification as Class;
use crate::model::metric_rollup_delta::crud::claim_metric_rollup_deltas;
use crate::model::metric_rollup_delta::tests::{
    captured, logging_pool, month_state, oracle_month_state, oracle_year_state, projection,
    verification, year_state,
};
use crate::model::metric_widget::{
    metric_widget_with_month_serving, MetricWidget, MetricWidgetError, MetricWidgetGrain,
    MetricWidgetInput, MetricWidgetSwitch,
};
use crate::model::tests::db::{test_db_url, TestDbGuard};
use crate::model::Doi;

const REGISTER: &str = include_str!("../../../../docs/metrics/contract-register.md");
const MUTATION_SOURCE: &str = include_str!("../../graphql/mutation.rs");
const MIGRATION: &str = "20261116";
const PLATFORM: &str = "cf";
const ACCOUNT_A: &str = "acct-a";
const ACCOUNT_B: &str = "acct-b";
const ISBNS: [&str; 3] = [
    "978-3-16-148410-0",
    "978-0-306-40615-7",
    "978-1-4028-9462-6",
];
const RORS: [&str; 2] = ["https://ror.org/02mhbdp94", "https://ror.org/03yrm5c26"];
const COUNTRIES: [&str; 2] = ["GB", "US"];
const NATIVE_TABLES: [&str; 4] = [
    "metric_native_month",
    "metric_native_country_month",
    "metric_native_institution_month",
    "metric_native_refusal",
];

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("a valid date")
}

// ---------------------------------------------------------------------------
// Plain reads
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
struct Line {
    #[diesel(sql_type = Text)]
    line: String,
}

fn lines_on(connection: &mut PgConnection, statement: &str) -> Vec<String> {
    sql_query(statement)
        .load::<Line>(connection)
        .unwrap_or_else(|error| panic!("{statement}: {error}"))
        .into_iter()
        .map(|row| row.line)
        .collect()
}

fn lines(pool: &PgPool, statement: &str) -> Vec<String> {
    let mut connection = pool.get().expect("a connection");
    lines_on(&mut connection, statement)
}

fn exec(pool: &PgPool, statement: &str) {
    let mut connection = pool.get().expect("a connection");
    sql_query(statement)
        .execute(&mut connection)
        .unwrap_or_else(|error| panic!("{statement}: {error}"));
}

/// A small deterministic generator (xorshift64*).
struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

/// A publisher with two accounts on one CloudFront platform, three works
/// (each with a DOI and a PDF ISBN), two institutions with a ROR, and two
/// measures mapped with every native grain and every dimension:
/// `title_sessions` (non-negative) and `net_units` (signed).
struct Fx {
    pool: Arc<PgPool>,
    publisher_id: Uuid,
    imprint_id: Uuid,
    platform_id: Uuid,
    works: Vec<Uuid>,
    dois: Vec<String>,
    title_sessions: Uuid,
    import_a: Uuid,
    import_b: Uuid,
}

#[derive(Debug, Clone, Copy)]
struct Obs {
    work: usize,
    net: bool,
    start: NaiveDate,
    end: NaiveDate,
    grain: MetricReportingGrain,
    mask: u8,
    country: usize,
    institution: usize,
    value: i64,
}

fn month_obs(work: usize, net: bool, start: NaiveDate, mask: u8, value: i64) -> Obs {
    Obs {
        work,
        net,
        start,
        end: start + Months::new(1),
        grain: MetricReportingGrain::Month,
        mask,
        country: 0,
        institution: 0,
        value,
    }
}

fn setup() -> (TestDbGuard, Fx) {
    let (guard, pool) = setup_registry_db();
    let [source, platform, account_a, account_b, publisher, imprint, import_a, import_b]: [Uuid;
        8] = std::array::from_fn(|_| Uuid::new_v4());
    let p = pool.as_ref();
    exec(p, &format!("INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) VALUES ('{source}', 'cloudfront-driver', 'DRIVER', 'cloudfront', TRUE)"));
    exec(p, &format!("INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ('{platform}', '{PLATFORM}', 'CloudFront', 'THOTH_MANAGED', TRUE)"));
    exec(p, &format!("INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ('{publisher}', 'Native publisher', 'OBELISK')"));
    for (account, code, key) in [
        (account_a, ACCOUNT_A, "dist-a"),
        (account_b, ACCOUNT_B, "dist-b"),
    ] {
        exec(p, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, enabled) VALUES ('{account}', '{code}', '{source}', '{platform}', '{key}', '{publisher}', TRUE)"));
    }
    exec(p, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint}', '{publisher}', 'Native imprint')"));
    let (mut works, mut dois) = (Vec::new(), Vec::new());
    for (index, isbn) in ISBNS.iter().enumerate() {
        let work = Uuid::new_v4();
        let doi = format!("https://doi.org/10.12345/native-{index}");
        exec(p, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{work}', 'monograph', 'forthcoming', '{imprint}', 1, '{doi}')"));
        exec(p, &format!("INSERT INTO publication (publication_id, publication_type, work_id, isbn) VALUES ('{}', 'PDF', '{work}', '{isbn}')", Uuid::new_v4()));
        works.push(work);
        dois.push(doi);
    }
    for (index, ror) in RORS.iter().enumerate() {
        exec(p, &format!("INSERT INTO institution (institution_id, institution_name, ror) VALUES ('{}', 'Institution {index}', '{ror}')", Uuid::new_v4()));
    }
    let measure = |code: &str| -> Uuid {
        lines(
            p,
            &format!("SELECT measure_id::text AS line FROM metric_measure WHERE code = '{code}'"),
        )[0]
        .parse()
        .expect("a measure id")
    };
    let (title_sessions, net_units) = (measure("title_sessions"), measure("net_units"));
    for (measure, grains) in [
        (title_sessions, "'DAY','MONTH','REPORTING_PERIOD'"),
        (net_units, "'MONTH','REPORTING_PERIOD'"),
    ] {
        exec(p, &format!("INSERT INTO metric_platform_measure (platform_measure_id, platform_id, measure_id, supported_grains, supports_country, supports_institution, supports_publication, direct_collection, enabled) VALUES ('{}', '{platform}', '{measure}', ARRAY[{grains}]::metric_reporting_grain[], TRUE, TRUE, TRUE, TRUE, TRUE)", Uuid::new_v4()));
    }
    for (import, account) in [(import_a, account_a), (import_b, account_b)] {
        exec(p, &format!("INSERT INTO metric_import (import_id, source_account_id, publisher_id, format_code, format_version, status, normalizer_version, created_by) VALUES ('{import}', '{account}', '{publisher}', 'cloudfront', '1', 'PROCESSING', 'normalizer/1', 'test')"));
    }
    (
        guard,
        Fx {
            pool,
            publisher_id: publisher,
            imprint_id: imprint,
            platform_id: platform,
            works,
            dois,
            title_sessions,
            import_a,
            import_b,
        },
    )
}

impl Fx {
    fn observation(&self, obs: &Obs, account_b: bool) -> NormalizedMetricObservation {
        NormalizedMetricObservation {
            source_account_code: if account_b { ACCOUNT_B } else { ACCOUNT_A }.into(),
            platform_code: PLATFORM.into(),
            measure_code: if obs.net {
                "net_units"
            } else {
                "title_sessions"
            }
            .into(),
            work_doi: self.dois[obs.work].clone(),
            publication_isbn: (obs.mask & 4 != 0).then(|| ISBNS[obs.work].to_string()),
            publication_type: None,
            period_start: obs.start,
            period_end: obs.end,
            reporting_grain: obs.grain,
            country_code: (obs.mask & 2 != 0).then(|| COUNTRIES[obs.country].to_string()),
            institution_ror: (obs.mask & 1 != 0).then(|| RORS[obs.institution].to_string()),
            value: obs.value,
            source_record_id: None,
            methodology_version: if obs.net {
                "sales/1"
            } else {
                "cloudfront-title-session/2"
            }
            .into(),
            source_row_number: None,
        }
    }

    fn batch(&self, key: &str, account_b: bool, observations: &[Obs]) -> MetricIngestionBatch {
        self.raw_batch(
            key,
            account_b,
            observations
                .iter()
                .map(|obs| self.observation(obs, account_b))
                .collect(),
        )
    }

    fn raw_batch(
        &self,
        key: &str,
        account_b: bool,
        observations: Vec<NormalizedMetricObservation>,
    ) -> MetricIngestionBatch {
        MetricIngestionBatch {
            import_id: if account_b {
                self.import_b
            } else {
                self.import_a
            },
            batch_key: key.into(),
            schema_version: SUPPORTED_SCHEMA_VERSION.into(),
            observations,
            coverage: vec![],
        }
    }

    fn ingest(
        &self,
        key: &str,
        account_b: bool,
        observations: &[Obs],
    ) -> Option<MetricIngestionOutcome> {
        ingest_metric_batch(&self.pool, &self.batch(key, account_b, observations)).ok()
    }

    /// The `WHERE` clause selecting the one record `obs` names.
    fn record_where(&self, obs: &Obs) -> String {
        let nullness = |bit: u8, column: &str| {
            if obs.mask & bit != 0 {
                format!("{column} IS NOT NULL")
            } else {
                format!("{column} IS NULL")
            }
        };
        format!(
            "work_id = '{}' AND measure_id = (SELECT measure_id FROM metric_measure WHERE code = '{}') \
             AND period_start = '{}' AND period_end = '{}' AND {} AND {} AND {}",
            self.works[obs.work],
            if obs.net { "net_units" } else { "title_sessions" },
            obs.start,
            obs.end,
            nullness(4, "publication_id"),
            nullness(2, "country_code"),
            nullness(1, "institution_id"),
        )
    }
}

// ---------------------------------------------------------------------------
// The oracle
// ---------------------------------------------------------------------------

/// The four native tables, one text line per row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Projection {
    months: BTreeSet<String>,
    countries: BTreeSet<String>,
    institutions: BTreeSet<String>,
    refusals: BTreeSet<String>,
}

impl Projection {
    fn rows(&self) -> usize {
        self.months.len() + self.countries.len() + self.institutions.len() + self.refusals.len()
    }
}

fn flag(value: bool) -> &'static str {
    if value {
        "t"
    } else {
        "f"
    }
}

fn actual(pool: &PgPool) -> Projection {
    let set = |statement: &str| lines(pool, statement).into_iter().collect();
    Projection {
        months: set("SELECT concat_ws('|', work_id, platform_id, measure_id, month_start, coalesce(value::text, 'null'), live, requires_country_coverage, requires_institution_coverage, country_ambiguous, institution_ambiguous) AS line FROM metric_native_month"),
        countries: set("SELECT concat_ws('|', work_id, platform_id, measure_id, month_start, country_code, value, requires_institution_coverage) AS line FROM metric_native_country_month"),
        institutions: set("SELECT concat_ws('|', work_id, platform_id, measure_id, month_start, institution_id, value, requires_country_coverage) AS line FROM metric_native_institution_month"),
        refusals: set("SELECT concat_ws('|', work_id, platform_id, measure_id, period_start, period_end, inconsistent) AS line FROM metric_native_refusal"),
    }
}

/// One non-`DAY` record as plain reads give it.
struct Record {
    key: [String; 3],
    start: NaiveDate,
    end: NaiveDate,
    grain: String,
    mask: u8,
    country: String,
    institution: String,
    status: String,
    value: i64,
    currents: i64,
    revisions: i64,
}

fn records(pool: &PgPool) -> Vec<Record> {
    lines(
        pool,
        "SELECT concat_ws('|', r.work_id, r.platform_id, r.measure_id, r.period_start, \
           r.period_end, r.reporting_grain, coalesce(r.publication_id::text, '-'), \
           coalesce(r.country_code::text, '-'), coalesce(r.institution_id::text, '-'), \
           coalesce(p.status::text, '-'), coalesce(p.value, 0), \
           (SELECT count(*) FROM metric_record_revision c \
             WHERE c.record_id = r.record_id AND c.status = 'CURRENT'), \
           (SELECT count(*) FROM metric_record_revision a WHERE a.record_id = r.record_id)) AS line \
         FROM metric_record r \
         LEFT JOIN metric_record_revision p \
           ON p.record_revision_id = r.current_revision_id AND p.record_id = r.record_id \
         WHERE r.reporting_grain <> 'DAY'",
    )
    .into_iter()
    .map(|line| {
        let f: Vec<&str> = line.split('|').collect();
        assert_eq!(f.len(), 13, "{line}");
        let parse_date = |text: &str| NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("a date");
        Record {
            key: [f[0].into(), f[1].into(), f[2].into()],
            start: parse_date(f[3]),
            end: parse_date(f[4]),
            grain: f[5].into(),
            mask: u8::from(f[6] != "-") * 4 + u8::from(f[7] != "-") * 2 + u8::from(f[8] != "-"),
            country: f[7].into(),
            institution: f[8].into(),
            status: f[9].into(),
            value: f[10].parse().expect("a value"),
            currents: f[11].parse().expect("a count"),
            revisions: f[12].parse().expect("a count"),
        }
    })
    .collect()
}

enum Kind {
    Withdrawn,
    ActiveReportingPeriod,
    ServedMonth,
    Inconsistent,
}

fn kind(record: &Record) -> Kind {
    let calendar = record.start.day() == 1 && record.start + Months::new(1) == record.end;
    if record.status == "RETRACTED" && record.currents == 0 {
        Kind::Withdrawn
    } else if record.grain == "REPORTING_PERIOD" && record.status == "CURRENT" {
        Kind::ActiveReportingPeriod
    } else if record.grain == "MONTH" && record.status == "CURRENT" && calendar {
        Kind::ServedMonth
    } else {
        Kind::Inconsistent
    }
}

/// The least mask (a subset of every other) among the masks bearing `bit`:
/// `(None, false)` when none bears it, `(None, true)` when no least exists.
fn least_bearing(masks: &BTreeSet<u8>, bit: u8) -> (Option<u8>, bool) {
    let bearing: Vec<u8> = masks.iter().copied().filter(|m| m & bit != 0).collect();
    if bearing.is_empty() {
        return (None, false);
    }
    let least: Vec<u8> = bearing
        .iter()
        .copied()
        .filter(|m| bearing.iter().all(|n| n & m == *m))
        .collect();
    match least.as_slice() {
        [m] => (Some(*m), false),
        _ => (None, true),
    }
}

fn oracle(pool: &PgPool) -> Projection {
    let records = records(pool);
    let mut cells: BTreeMap<(String, String, String, NaiveDate), Vec<&Record>> = BTreeMap::new();
    let mut refusals: BTreeMap<String, bool> = BTreeMap::new();
    for record in &records {
        match kind(record) {
            Kind::Withdrawn => {}
            Kind::ServedMonth => cells
                .entry((
                    record.key[0].clone(),
                    record.key[1].clone(),
                    record.key[2].clone(),
                    record.start,
                ))
                .or_default()
                .push(record),
            other => {
                let line = format!(
                    "{}|{}|{}|{}|{}",
                    record.key[0], record.key[1], record.key[2], record.start, record.end
                );
                let inconsistent = refusals.entry(line).or_insert(false);
                *inconsistent |= matches!(other, Kind::Inconsistent);
            }
        }
    }
    let mut projection = Projection {
        refusals: refusals
            .into_iter()
            .map(|(line, inconsistent)| format!("{line}|{}", flag(inconsistent)))
            .collect(),
        ..Projection::default()
    };
    for ((work, platform, measure, month), cell) in cells {
        let head = format!("{work}|{platform}|{measure}|{month}");
        let masks: BTreeSet<u8> = cell.iter().map(|record| record.mask).collect();
        let total = if masks.contains(&0) {
            Some(0)
        } else if masks.len() == 1 {
            masks.first().copied()
        } else {
            None
        };
        let value = total.map(|mask| {
            cell.iter()
                .filter(|record| record.mask == mask)
                .map(|record| record.value)
                .sum::<i64>()
        });
        let (country, country_ambiguous) = least_bearing(&masks, 2);
        let (institution, institution_ambiguous) = least_bearing(&masks, 1);
        projection.months.insert(format!(
            "{head}|{}|{}|{}|{}|{}|{}",
            value.map_or("null".to_string(), |v| v.to_string()),
            flag(cell.iter().any(|record| record.value != 0)),
            flag(total.is_some_and(|mask| mask & 2 != 0)),
            flag(total.is_some_and(|mask| mask & 1 != 0)),
            flag(country_ambiguous),
            flag(institution_ambiguous),
        ));
        if let Some(mask) = country {
            let mut sums: BTreeMap<&str, i64> = BTreeMap::new();
            for record in cell.iter().filter(|record| record.mask == mask) {
                *sums.entry(record.country.as_str()).or_default() += record.value;
            }
            for (code, sum) in sums {
                projection
                    .countries
                    .insert(format!("{head}|{code}|{sum}|{}", flag(mask & 1 != 0)));
            }
        }
        if let Some(mask) = institution {
            let mut sums: BTreeMap<&str, i64> = BTreeMap::new();
            for record in cell.iter().filter(|record| record.mask == mask) {
                *sums.entry(record.institution.as_str()).or_default() += record.value;
            }
            for (id, sum) in sums {
                projection
                    .institutions
                    .insert(format!("{head}|{id}|{sum}|{}", flag(mask & 2 != 0)));
            }
        }
    }
    projection
}

// ---------------------------------------------------------------------------
// The register's checks, extracted as written
// ---------------------------------------------------------------------------

/// The first `sql` fence after `marker` in the register.
fn register_sql(marker: &str) -> String {
    let at = REGISTER
        .find(marker)
        .unwrap_or_else(|| panic!("the register must name `{marker}`"));
    let rest = &REGISTER[at..];
    let start = rest.find("```sql\n").expect("a sql fence") + "```sql\n".len();
    let end = rest[start..].find("\n```").expect("the fence closes");
    rest[start..start + end].to_string()
}

fn v_native_sql() -> String {
    let checks = register_sql("**V-NATIVE**, after each deploy");
    let projection = register_sql("#### Native-month projection (`MET-WP4-MONTH-01`)");
    assert_eq!(
        checks, projection,
        "sections 2.2 and 3.6 must give V-NATIVE as one text"
    );
    checks
}

fn json_rows(pool: &PgPool, sql: &str) -> Vec<serde_json::Value> {
    let body = sql.trim_end().trim_end_matches(';');
    lines(
        pool,
        &format!("SELECT row_to_json(v)::text AS line FROM ({body}\n) v"),
    )
    .iter()
    .map(|line| serde_json::from_str(line).expect("a JSON row"))
    .collect()
}

pub(crate) fn v_native(pool: &PgPool) -> BTreeMap<String, i64> {
    let rows = json_rows(pool, &v_native_sql());
    assert_eq!(rows.len(), 1);
    let counts: BTreeMap<String, i64> = rows[0]
        .as_object()
        .expect("one row")
        .iter()
        .map(|(name, value)| (name.clone(), value.as_i64().expect("a count")))
        .collect();
    assert_eq!(counts.len(), 12, "V-NATIVE returns twelve counts");
    counts
}

pub(crate) fn v_native_total(pool: &PgPool) -> i64 {
    v_native(pool).values().sum()
}

fn v_native_keys(pool: &PgPool) -> Vec<serde_json::Value> {
    let full = v_native_sql();
    let at = full.rfind("\nSELECT\n").expect("V-NATIVE's final SELECT");
    let keys = register_sql("V-NATIVE's key form");
    json_rows(pool, &format!("{}\n{keys}", &full[..at]))
}

pub(crate) fn v_month(pool: &PgPool) -> BTreeMap<String, serde_json::Value> {
    json_rows(pool, &register_sql("**V-MONTH (PRE-1)**"))
        .into_iter()
        .map(|row| {
            (
                row["reporting_grain"]
                    .as_str()
                    .expect("a grain")
                    .to_string(),
                row,
            )
        })
        .collect()
}

pub(crate) fn v_conflict(pool: &PgPool) -> i64 {
    json_rows(pool, &register_sql("**V-CONFLICT**, once rollups"))[0]["count"]
        .as_i64()
        .expect("a count")
}

pub(crate) fn v_plan(
    pool: &PgPool,
    publisher: Uuid,
    platform: Uuid,
    from: NaiveDate,
    through: NaiveDate,
) -> i64 {
    let sql = register_sql("**V-PLAN**, with no");
    let body = sql.trim_end().trim_end_matches(';');
    let mut connection = pool.get().expect("a connection");
    let rows: Vec<Line> = sql_query(format!(
        "SELECT row_to_json(v)::text AS line FROM ({body}\n) v"
    ))
    .bind::<SqlUuid, _>(publisher)
    .bind::<SqlUuid, _>(platform)
    .bind::<Date, _>(from)
    .bind::<Date, _>(through)
    .load(&mut connection)
    .expect("V-PLAN");
    let row: serde_json::Value = serde_json::from_str(&rows[0].line).expect("a JSON row");
    row["count"].as_i64().expect("a count")
}

/// The four tables equal the oracle, and V-NATIVE counts nothing.
fn check(fx: &Fx, label: &str) {
    assert_eq!(actual(&fx.pool), oracle(&fx.pool), "{label}");
    let counts = v_native(&fx.pool);
    assert!(
        counts.values().all(|count| *count == 0),
        "{label}: {counts:?}"
    );
}

/// Digests of the canonical tables, to prove a path leaves them as they were.
fn canonical(pool: &PgPool) -> Vec<String> {
    [
        "metric_record",
        "metric_record_revision",
        "metric_record_provenance",
        "metric_rollup_delta",
    ]
    .iter()
    .map(|table| {
        lines(
            pool,
            &format!("SELECT coalesce(md5(string_agg(t::text, ',' ORDER BY t::text)), '-') AS line FROM {table} t"),
        )
        .remove(0)
    })
    .collect()
}

fn repair(pool: &PgPool, publisher: Uuid) -> super::MetricNativeMonthRecomputeResult {
    recompute_metric_native_months(pool, publisher).expect("the repair succeeds")
}

// ---------------------------------------------------------------------------
// T15: projection exactness
// ---------------------------------------------------------------------------

#[test]
fn the_projection_equals_the_oracle_after_every_request_and_v_native_finds_every_drift() {
    let (_guard, fx) = setup();
    let pool = fx.pool.as_ref();
    let mut counters: BTreeMap<&str, usize> = BTreeMap::new();
    let mut bump = |name: &'static str| *counters.entry(name).or_default() += 1;

    // An anchor month outside the random months, and designed cells: total
    // ambiguity (masks 2 and 1), country and total ambiguity (6 and 3),
    // institution ambiguity (5 and 3), and a signed month netting to 0.
    let anchor = month_obs(0, false, date(2026, 5, 1), 0, 7);
    assert!(fx.ingest("anchor", false, &[anchor]).is_some());
    check(&fx, "anchor");
    let april = date(2026, 4, 1);
    let designed = [
        month_obs(1, false, april, 2, 3),
        month_obs(1, false, april, 1, 4),
        month_obs(2, false, april, 6, 2),
        month_obs(2, false, april, 3, 5),
        month_obs(0, true, april, 5, 2),
        month_obs(0, true, april, 3, -2),
    ];
    assert!(fx.ingest("designed", false, &designed).is_some());
    check(&fx, "designed");

    let mut generator = Generator(0x4d45_545f_4d4f_4e54);
    let months = [date(2026, 1, 1), date(2026, 2, 1), date(2026, 3, 1)];
    let mut history: Vec<Obs> = Vec::new();
    for step in 0..90 {
        let account_b = generator.below(6) == 0;
        let mut batch = Vec::new();
        for _ in 0..1 + generator.below(4) {
            let obs = if !history.is_empty() && generator.below(3) == 0 {
                let mut again = history[generator.below(history.len() as u64) as usize];
                match generator.below(4) {
                    0 => {}
                    1 => again.value += 1 + generator.below(3) as i64,
                    2 => again.value -= 1 + generator.below(3) as i64,
                    _ => again.value = 0,
                }
                if !again.net {
                    again.value = again.value.max(0);
                }
                again
            } else {
                let start = months[generator.below(3) as usize];
                let reporting_period = generator.below(8) == 0;
                let net = generator.below(2) == 0;
                Obs {
                    work: generator.below(3) as usize,
                    net,
                    start,
                    end: if reporting_period {
                        start + Days::new(10 + generator.below(50))
                    } else {
                        start + Months::new(1)
                    },
                    grain: if reporting_period {
                        MetricReportingGrain::ReportingPeriod
                    } else {
                        MetricReportingGrain::Month
                    },
                    mask: generator.below(8) as u8,
                    country: generator.below(2) as usize,
                    institution: generator.below(2) as usize,
                    value: if net {
                        generator.below(11) as i64 - 5
                    } else {
                        generator.below(6) as i64
                    },
                }
            };
            batch.push(obs);
        }
        let label = format!("step {step}");
        match fx.ingest(&label, account_b, &batch) {
            Some(outcome) => {
                for row in &outcome.rows {
                    match row.classification {
                        Class::Winner => bump("winner"),
                        Class::Duplicate => bump("duplicate"),
                        Class::Revision => bump("revision"),
                        Class::Conflict => bump("conflict"),
                        Class::Rejected => bump("rejected"),
                    }
                }
                let keys: BTreeSet<(usize, bool, NaiveDate)> =
                    batch.iter().map(|o| (o.work, o.net, o.start)).collect();
                if keys.len() > 1 {
                    bump("several keys in one request");
                }
                history.extend(batch.iter().copied());
            }
            None => bump("failed request"),
        }
        check(&fx, &label);
    }

    // The sequence reached every case it is meant to.
    let records = records(pool);
    let expected = oracle(pool);
    for record in &records {
        if record.grain == "REPORTING_PERIOD" {
            bump("reporting period record");
        }
        if record.value == 0 && record.revisions > 1 {
            bump("revised to zero");
        }
    }
    for line in &expected.months {
        let fields: Vec<&str> = line.split('|').collect();
        if fields[4] == "null" {
            bump("total-ambiguous cell");
        }
        if fields[4].starts_with('-') {
            bump("negative month");
        }
        if fields[8] == "t" {
            bump("country-ambiguous cell");
        }
        if fields[9] == "t" {
            bump("institution-ambiguous cell");
        }
    }
    for name in [
        "winner",
        "duplicate",
        "revision",
        "conflict",
        "several keys in one request",
        "reporting period record",
        "revised to zero",
        "total-ambiguous cell",
        "negative month",
        "country-ambiguous cell",
        "institution-ambiguous cell",
    ] {
        assert!(
            counters.get(name).copied().unwrap_or(0) > 0,
            "{name}: {counters:?}"
        );
    }
    assert!(!expected.countries.is_empty() && !expected.institutions.is_empty());

    // V-MONTH, V-CONFLICT and V-PLAN over this database.
    for (grain, row) in v_month(pool) {
        assert_eq!(row["invalid_periods"], 0, "{grain}");
        assert_eq!(row["inconsistent_pointers"], 0, "{grain}");
        assert_eq!(row["non_additive"], 0, "{grain}");
    }
    assert_eq!(v_conflict(pool), 0);
    let (publisher, platform) = (fx.publisher_id, fx.platform_id);
    assert_eq!(
        v_plan(
            pool,
            publisher,
            platform,
            date(2026, 5, 31),
            date(2026, 6, 10)
        ),
        1
    );
    // Its last day lies in the anchor's month, and it covers April's three
    // live designed cells.
    assert_eq!(
        v_plan(
            pool,
            publisher,
            platform,
            date(2026, 4, 20),
            date(2026, 5, 1)
        ),
        4
    );
    assert_eq!(
        v_plan(
            pool,
            publisher,
            platform,
            date(2026, 6, 1),
            date(2026, 6, 30)
        ),
        0
    );
    assert_eq!(
        v_plan(
            pool,
            Uuid::new_v4(),
            platform,
            date(2026, 5, 1),
            date(2026, 5, 31)
        ),
        0
    );
    let anchor_key = format!(
        "'{}', '{}', '{}', '2026-05-01'",
        fx.works[0], fx.platform_id, fx.title_sessions
    );
    exec(pool, &format!("INSERT INTO metric_rollup_work_month (work_id, platform_id, measure_id, month_start, value, requires_country_coverage, requires_institution_coverage, watermark) VALUES ({anchor_key}, 3, FALSE, FALSE, 1)"));
    assert_eq!(v_conflict(pool), 1, "a live day side beside a live month");
    exec(pool, "UPDATE metric_rollup_work_month SET value = 0");
    assert_eq!(v_conflict(pool), 0, "a zero day side is not live");
    exec(pool, "DELETE FROM metric_rollup_work_month");

    // Direct SQL drift: each corruption makes V-NATIVE non-zero, and the
    // repair restores the oracle's rows and 0 without touching a canonical
    // table; undoing it is drift too, repaired the same way.
    let anchor_record = format!(
        "(SELECT record_id FROM metric_record WHERE {})",
        fx.record_where(&anchor)
    );
    let anchor_revision = format!(
        "(SELECT current_revision_id FROM metric_record WHERE record_id = {anchor_record})"
    );
    let corruptions = [
        (
            "a contradictory pointer",
            format!("UPDATE metric_record_revision SET status = 'SUPERSEDED' WHERE record_revision_id = {anchor_revision}"),
            format!("UPDATE metric_record_revision SET status = 'CURRENT' WHERE record_revision_id = {anchor_revision}"),
        ),
        (
            "a non-calendar month",
            format!("UPDATE metric_record SET period_end = '2026-06-02' WHERE record_id = {anchor_record}"),
            "UPDATE metric_record SET period_end = '2026-06-01' WHERE period_end = '2026-06-02'".to_string(),
        ),
        (
            "a withdrawn record",
            format!("UPDATE metric_record_revision SET status = 'RETRACTED' WHERE record_revision_id = {anchor_revision}"),
            format!("UPDATE metric_record_revision SET status = 'CURRENT' WHERE record_revision_id = {anchor_revision}"),
        ),
        (
            "a changed value",
            format!("UPDATE metric_record_revision SET value = value + 1 WHERE record_revision_id = {anchor_revision}"),
            format!("UPDATE metric_record_revision SET value = value - 1 WHERE record_revision_id = {anchor_revision}"),
        ),
    ];
    for (name, corrupt, undo) in &corruptions {
        exec(pool, corrupt);
        assert!(v_native_total(pool) > 0, "{name} must be found");
        let months = v_month(pool);
        let month_row = &months["MONTH"];
        let inconsistent = month_row["inconsistent_pointers"]
            .as_i64()
            .expect("a count");
        let invalid = month_row["invalid_periods"].as_i64().expect("a count");
        match *name {
            "a contradictory pointer" => assert_eq!((inconsistent, invalid), (1, 0)),
            "a non-calendar month" => assert_eq!((inconsistent, invalid), (0, 1)),
            _ => assert_eq!((inconsistent, invalid), (0, 0), "{name}"),
        }
        let before = canonical(pool);
        let result = repair(pool, fx.publisher_id);
        assert_eq!(result.works, 3);
        assert_eq!(result.rows as usize, actual(pool).rows());
        check(&fx, name);
        assert_eq!(
            canonical(pool),
            before,
            "{name}: the repair writes no canonical row"
        );
        if *name == "a contradictory pointer" {
            assert!(actual(pool).refusals.contains(&format!(
                "{}|{}|{}|2026-05-01|2026-06-01|t",
                fx.works[0], fx.platform_id, fx.title_sessions
            )));
        }
        exec(pool, undo);
        assert!(v_native_total(pool) > 0, "undoing {name} is drift too");
        repair(pool, fx.publisher_id);
        check(&fx, &format!("undoing {name}"));
    }
    exec(
        pool,
        &format!(
            "DELETE FROM metric_native_month WHERE work_id = '{}' AND month_start = '2026-05-01'",
            fx.works[0]
        ),
    );
    assert_eq!(v_native(pool)["month_missing"], 1);
    let keys = v_native_keys(pool);
    assert_eq!(keys.len(), 1, "{keys:?}");
    assert_eq!(keys[0]["relation"], "metric_native_month");
    assert_eq!(keys[0]["kind"], "missing");
    assert_eq!(keys[0]["work_id"], fx.works[0].to_string());
    assert_eq!(keys[0]["period_start"], "2026-05-01");
    let first = repair(pool, fx.publisher_id);
    check(&fx, "a deleted native row");
    let (rows, digest) = (actual(pool), canonical(pool));
    assert_eq!(
        repair(pool, fx.publisher_id),
        first,
        "a second run changes nothing"
    );
    assert_eq!((actual(pool), canonical(pool)), (rows, digest));

    // Neutral changes are no drift.
    exec(pool, "UPDATE metric_record SET updated_at = now()");
    assert_eq!(v_native_total(pool), 0);
    exec(
        pool,
        "UPDATE metric_record_revision SET value = value + 100 WHERE status = 'SUPERSEDED'",
    );
    assert_eq!(v_native_total(pool), 0, "a superseded value is no drift");
    exec(
        pool,
        "UPDATE metric_record_revision SET value = value - 100 WHERE status = 'SUPERSEDED'",
    );
}

#[test]
fn a_reporting_period_beside_a_served_month_adds_or_removes_only_its_own_refusal_row() {
    let (_guard, fx) = setup();
    let march = date(2026, 3, 1);
    let mut month = month_obs(0, false, march, 0, 5);
    assert!(fx.ingest("month", false, &[month]).is_some());
    check(&fx, "month");
    let months = actual(&fx.pool).months;
    let mut period = Obs {
        end: date(2026, 4, 15),
        grain: MetricReportingGrain::ReportingPeriod,
        mask: 2,
        value: 3,
        ..month
    };
    let refusal = |inconsistent: &str| {
        BTreeSet::from([format!(
            "{}|{}|{}|2026-03-01|2026-04-15|{inconsistent}",
            fx.works[0], fx.platform_id, fx.title_sessions
        )])
    };
    for (label, value) in [("active", 3), ("active zero", 0)] {
        period.value = value;
        assert!(fx.ingest(label, false, &[period]).is_some());
        check(&fx, label);
        let rows = actual(&fx.pool);
        assert_eq!(rows.months, months, "{label}: the month's rows stay");
        assert_eq!(rows.refusals, refusal("f"), "{label}");
    }
    exec(&fx.pool, &format!("UPDATE metric_record_revision SET status = 'RETRACTED' WHERE record_revision_id = (SELECT current_revision_id FROM metric_record WHERE {})", fx.record_where(&period)));
    assert!(
        v_native_total(&fx.pool) > 0,
        "stale until its key is recomputed"
    );
    month.value = 6;
    assert!(fx.ingest("recompute the key", false, &[month]).is_some());
    check(&fx, "withdrawn");
    assert!(actual(&fx.pool).refusals.is_empty());
}

#[test]
fn only_a_request_writing_a_native_key_runs_the_eight_statements() {
    let (_guard, fx) = setup();
    let (logging, log) = logging_pool();
    let native = || -> Vec<String> {
        captured(&log)
            .into_iter()
            .filter(|statement| statement.contains("metric_native_"))
            .collect()
    };
    let eight: Vec<String> = RECOMPUTE_STATEMENTS.iter().map(|s| s.to_string()).collect();
    let day = Obs {
        end: date(2026, 3, 2),
        grain: MetricReportingGrain::Day,
        ..month_obs(0, false, date(2026, 3, 1), 2, 4)
    };
    assert!(ingest_metric_batch(&logging, &fx.batch("day", false, &[day])).is_ok());
    assert!(captured(&log).len() > 3, "the request is captured");
    assert!(
        native().is_empty(),
        "a DAY-only request runs no native statement"
    );
    log.lock().expect("log").clear();
    let two_keys = [
        month_obs(0, false, date(2026, 1, 1), 2, 4),
        month_obs(1, true, date(2026, 2, 1), 0, -3),
    ];
    assert!(ingest_metric_batch(&logging, &fx.batch("months", false, &two_keys)).is_ok());
    assert_eq!(native(), eight, "two keys, one recompute");
    check(&fx, "months");

    let mut connection = logging.get().expect("a connection");
    for count in [1_u32, 50] {
        let keys: NativeMonthKeys = (0..count)
            .map(|offset| NativeMonthKey {
                work_id: fx.works[0],
                platform_id: fx.platform_id,
                measure_id: fx.title_sessions,
                period_start: date(2026, 1, 1) + Months::new(offset),
            })
            .collect();
        assert_eq!(keys.len(), count as usize);
        log.lock().expect("log").clear();
        connection
            .transaction(|connection| recompute(connection, &keys))
            .expect("the recompute succeeds");
        assert_eq!(native(), eight, "{count} keys");
    }
    log.lock().expect("log").clear();
    assert_eq!(
        recompute(&mut connection, &NativeMonthKeys::default()).expect("empty"),
        0
    );
    assert!(captured(&log).is_empty(), "an empty set runs no statement");
    drop(connection);
    check(&fx, "direct recomputes");
}

/// A trigger that fails every insert into one native table, dropped on exit.
struct FailingInsert<'a> {
    pool: &'a PgPool,
    table: &'static str,
}

impl<'a> FailingInsert<'a> {
    fn on(pool: &'a PgPool, table: &'static str) -> Self {
        exec(pool, "CREATE OR REPLACE FUNCTION native_test_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected native failure'; END $$");
        exec(pool, &format!("CREATE TRIGGER native_test_failure BEFORE INSERT ON {table} FOR EACH ROW EXECUTE FUNCTION native_test_failure()"));
        Self { pool, table }
    }
}

impl Drop for FailingInsert<'_> {
    fn drop(&mut self) {
        if let Ok(mut connection) = self.pool.get() {
            let _ = sql_query(format!(
                "DROP TRIGGER IF EXISTS native_test_failure ON {}",
                self.table
            ))
            .execute(&mut connection);
            let _ =
                sql_query("DROP FUNCTION IF EXISTS native_test_failure()").execute(&mut connection);
        }
    }
}

#[test]
fn a_failure_in_any_native_table_or_an_overflow_rolls_the_request_back() {
    let (_guard, fx) = setup();
    let january = date(2026, 1, 1);
    let mut seed = vec![
        month_obs(0, false, january, 2, 3),
        month_obs(0, false, date(2026, 2, 1), 1, 4),
        month_obs(2, false, january, 0, 5),
        Obs {
            end: january + Days::new(40),
            grain: MetricReportingGrain::ReportingPeriod,
            ..month_obs(1, false, january, 0, 2)
        },
    ];
    assert!(fx.ingest("seed", false, &seed).is_some());
    check(&fx, "seed");
    for obs in &mut seed {
        obs.value += 1;
    }
    for table in NATIVE_TABLES {
        let before = (canonical(&fx.pool), actual(&fx.pool));
        {
            let _failing = FailingInsert::on(&fx.pool, table);
            assert!(
                fx.ingest(&format!("fail {table}"), false, &seed).is_none(),
                "{table}"
            );
        }
        assert_eq!((canonical(&fx.pool), actual(&fx.pool)), before, "{table}");
    }
    let mut huge = month_obs(1, false, date(2026, 3, 1), 2, i64::MAX / 2 + 1);
    assert!(fx.ingest("huge", false, &[huge]).is_some());
    check(&fx, "huge");
    let before = (canonical(&fx.pool), actual(&fx.pool));
    huge.country = 1;
    assert!(
        fx.ingest("overflow", false, &[huge]).is_none(),
        "an overflowing sum fails the request"
    );
    assert_eq!((canonical(&fx.pool), actual(&fx.pool)), before);
    assert!(fx.ingest("after", false, &seed).is_some());
    check(&fx, "after");
}

#[test]
fn concurrent_requests_on_one_cell_wait_on_the_work_row_and_leave_the_oracle_rows() {
    let (_guard, fx) = setup();
    let pool: Arc<PgPool> = Arc::new(
        Pool::builder()
            .max_size(3)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("a pool"),
    );
    let mut holder = PgConnection::establish(&test_db_url()).expect("a connection");
    sql_query("BEGIN").execute(&mut holder).expect("begin");
    sql_query(format!(
        "SELECT work_id FROM work WHERE work_id = '{}' FOR UPDATE",
        fx.works[0]
    ))
    .execute(&mut holder)
    .expect("lock the work row");
    let holder_pid = lines_on(&mut holder, "SELECT pg_backend_pid()::text AS line").remove(0);
    let spawn = |account_b: bool, country: usize| {
        let pool = Arc::clone(&pool);
        let obs = Obs {
            country,
            ..month_obs(0, false, date(2026, 1, 1), 2, 3 + country as i64)
        };
        let batch = fx.batch(&format!("concurrent {country}"), account_b, &[obs]);
        thread::spawn(move || ingest_metric_batch(&pool, &batch).is_ok())
    };
    let first = spawn(false, 0);
    let second = spawn(true, 1);
    // The first waits on the holder's work row, and the second behind it on
    // the same row's lock.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let waiting = lines(
            &fx.pool,
            &format!(
                "SELECT (count(*) FILTER (WHERE cardinality(pg_blocking_pids(pid)) > 0))::text \
                   || '|' || (count(*) FILTER (WHERE {holder_pid} = ANY(pg_blocking_pids(pid))))::text AS line \
                 FROM pg_stat_activity"
            ),
        )
        .remove(0);
        if waiting == "2|1" || waiting == "2|2" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "both requests wait on the work row: {waiting}"
        );
        thread::sleep(Duration::from_millis(50));
    }
    sql_query("COMMIT").execute(&mut holder).expect("commit");
    assert!(first.join().expect("first"), "the first request commits");
    assert!(second.join().expect("second"), "the second request commits");
    check(&fx, "concurrent");
    let countries = actual(&fx.pool).countries;
    assert_eq!(countries.len(), 2, "{countries:?}");
}

#[test]
fn a_month_resolved_by_quarantine_reconciliation_is_projected() {
    let (_guard, fx) = setup();
    let doi = "https://doi.org/10.12345/native-later";
    let mut observation = fx.observation(&month_obs(0, false, date(2026, 2, 1), 2, 4), false);
    observation.work_doi = doi.into();
    let outcome = ingest_metric_batch(
        &fx.pool,
        &fx.raw_batch("unresolved", false, vec![observation]),
    )
    .expect("the request commits");
    assert!(outcome.rows.iter().all(|row| row.record_id.is_none()));
    check(&fx, "quarantined");
    assert!(actual(&fx.pool).months.is_empty());
    // Reconciliation takes rows of a terminal import only.
    exec(&fx.pool, &format!("UPDATE metric_import SET status = 'COMPLETED_WITH_ERRORS', completed_at = now() WHERE import_id = '{}'", fx.import_a));
    let work = Uuid::new_v4();
    exec(&fx.pool, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{work}', 'monograph', 'forthcoming', '{}', 1, '{doi}')", fx.imprint_id));
    let result =
        reconcile_metric_identifier_quarantine(&fx.pool, "native-test", 10).expect("reconcile");
    assert_eq!(result.resolved, 1, "{result:?}");
    check(&fx, "reconciled");
    let rows = actual(&fx.pool);
    assert!(rows
        .months
        .iter()
        .any(|line| line.starts_with(&work.to_string())));
    assert!(rows
        .countries
        .iter()
        .any(|line| line.starts_with(&work.to_string())));
}

#[test]
fn the_repair_rebuilds_every_work_of_the_publisher_in_chunks_and_only_for_a_superuser() {
    let (_guard, fx) = setup();
    assert_eq!(REPAIR_CHUNK_WORKS, 50);
    let mut observations = Vec::new();
    for index in 0..52 {
        let doi = format!("https://doi.org/10.12345/native-extra-{index}");
        exec(&fx.pool, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{}', 'monograph', 'forthcoming', '{}', 1, '{doi}')", Uuid::new_v4(), fx.imprint_id));
        let mut observation = fx.observation(
            &month_obs(0, index % 2 == 0, date(2026, 1, 1), 0, index),
            false,
        );
        observation.work_doi = doi;
        observations.push(observation);
    }
    for obs in [
        month_obs(0, false, date(2026, 1, 1), 6, 2),
        month_obs(1, true, date(2026, 2, 1), 1, -2),
    ] {
        observations.push(fx.observation(&obs, false));
    }
    ingest_metric_batch(&fx.pool, &fx.raw_batch("many works", false, observations))
        .expect("the request commits");
    check(&fx, "many works");
    let (expected, digest) = (actual(&fx.pool), canonical(&fx.pool));
    for table in NATIVE_TABLES {
        exec(&fx.pool, &format!("DELETE FROM {table}"));
    }
    assert!(v_native_total(&fx.pool) > 0);
    let (logging, log) = logging_pool();
    let result = repair(&logging, fx.publisher_id);
    assert_eq!(result.works, 55);
    assert_eq!(result.rows as usize, expected.rows());
    let chunks = captured(&log)
        .iter()
        .filter(|statement| statement.as_str() == REPAIR_STATEMENTS[0])
        .count();
    assert_eq!(chunks, 2, "55 works in chunks of at most 50");
    check(&fx, "repaired");
    assert_eq!(actual(&fx.pool), expected);
    assert_eq!(canonical(&fx.pool), digest);
    assert_eq!(repair(&fx.pool, fx.publisher_id), result, "idempotent");
    assert_eq!((actual(&fx.pool), canonical(&fx.pool)), (expected, digest));
    assert!(matches!(
        recompute_metric_native_months(&fx.pool, Uuid::new_v4()),
        Err(ThothError::EntityNotFound)
    ));
    let guarded: String = MUTATION_SOURCE
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        guarded
            .matches("authorize_metric_registry_admin(context) .and_then(|_| recompute_metric_native_months(&context.db, publisher_id))")
            .count(),
        1,
        "the repair is reached only through the superuser guard"
    );
    assert_eq!(
        guarded
            .matches("recompute_metric_native_months(&context.db")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// T3: the migration
// ---------------------------------------------------------------------------

fn catalog(connection: &mut PgConnection) -> BTreeSet<String> {
    [
        "SELECT 'relation ' || relname || ' ' || relkind::text AS line FROM pg_class WHERE relnamespace = 'public'::regnamespace",
        "SELECT 'index ' || indexname || ' ' || tablename AS line FROM pg_indexes WHERE schemaname = 'public'",
        "SELECT 'constraint ' || conrelid::regclass::text || ' ' || conname || ' ' || contype::text || CASE WHEN contype IN ('p', 'f', 'u') THEN ' ' || pg_get_constraintdef(oid) ELSE '' END AS line FROM pg_constraint WHERE connamespace = 'public'::regnamespace AND contype <> 'n'",
        "SELECT 'trigger ' || tgrelid::regclass::text || ' ' || tgname AS line FROM pg_trigger WHERE NOT tgisinternal",
        "SELECT 'function ' || oid::regprocedure::text AS line FROM pg_proc WHERE pronamespace = 'public'::regnamespace",
        "SELECT 'column ' || table_name || '.' || column_name || ' ' || data_type || ' ' || is_nullable AS line FROM information_schema.columns WHERE table_schema = 'public'",
    ]
    .iter()
    .flat_map(|statement| lines_on(connection, statement))
    // `revert` also reverts migrations that landed after this stage; the
    // schema of PUB-DASH-URL-01 is not this stage's and is left out of every
    // comparison. The citation tables (CITATIONS-01 C1) are likewise a later
    // migration's and are left out.
    .filter(|line| !line.contains("metrics_dashboard_url") && !line.contains("metric_citation"))
    .collect()
}

/// Exactly the catalog lines section 4.1 adds.
fn migration_lines() -> BTreeSet<String> {
    let key = ["work_id uuid", "platform_id uuid", "measure_id uuid"];
    let tables: [(&str, Vec<&str>, &str); 4] = [
        (
            "metric_native_month",
            vec![
                "month_start date",
                "live boolean",
                "requires_country_coverage boolean",
                "requires_institution_coverage boolean",
                "country_ambiguous boolean",
                "institution_ambiguous boolean",
            ],
            "PRIMARY KEY (work_id, platform_id, measure_id, month_start)",
        ),
        (
            "metric_native_country_month",
            vec![
                "month_start date",
                "country_code character",
                "value bigint",
                "requires_institution_coverage boolean",
            ],
            "PRIMARY KEY (work_id, platform_id, measure_id, month_start, country_code)",
        ),
        (
            "metric_native_institution_month",
            vec![
                "month_start date",
                "institution_id uuid",
                "value bigint",
                "requires_country_coverage boolean",
            ],
            "PRIMARY KEY (work_id, platform_id, measure_id, month_start, institution_id)",
        ),
        (
            "metric_native_refusal",
            vec![
                "period_start date",
                "period_end date",
                "inconsistent boolean",
            ],
            "PRIMARY KEY (work_id, platform_id, measure_id, period_start, period_end)",
        ),
    ];
    let mut expected = BTreeSet::new();
    for (table, columns, primary_key) in tables {
        expected.insert(format!("relation {table} r"));
        expected.insert(format!("relation {table}_pkey i"));
        expected.insert(format!("index {table}_pkey {table}"));
        expected.insert(format!("constraint {table} {table}_pkey p {primary_key}"));
        if table != "metric_native_refusal" {
            expected.insert(format!("constraint {table} {table}_month_start_check c"));
            // MET-QUARTER-01 (migration 20261129) adds the span to the three month tables.
            expected.insert(format!("constraint {table} {table}_span_months_check c"));
            expected.insert(format!("column {table}.span_months smallint NO"));
        }
        for column in key.iter().chain(columns.iter()) {
            expected.insert(format!("column {table}.{column} NO"));
        }
    }
    expected.insert("column metric_native_month.value bigint YES".into());
    expected.insert(
        "constraint metric_native_country_month metric_native_country_month_country_code_check c"
            .into(),
    );
    for (column, kind) in [
        ("window_start", "date"),
        ("window_end", "date"),
        ("succession", "text"),
    ] {
        expected.insert(format!("column metric_source_account.{column} {kind} YES"));
    }
    for check in ["window", "succession"] {
        expected.insert(format!(
            "constraint metric_source_account metric_source_account_{check}_check c"
        ));
    }
    expected
}

/// Revert this stage's migration. Migrations that landed after it are reverted
/// first (newest first), because `down.sql` of a newer migration never depends
/// on this stage and this stage's `down.sql` must run against the schema it
/// was written for.
fn revert(connection: &mut PgConnection) -> Result<(), String> {
    loop {
        let latest = connection
            .applied_migrations()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|version| version.to_string())
            .max()
            .expect("migrations are applied");
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .map_err(|error| error.to_string())?;
        assert_eq!(reverted.to_string(), latest);
        if latest.as_str() <= MIGRATION {
            assert_eq!(latest, MIGRATION, "this stage's migration is reached");
            return Ok(());
        }
    }
}

fn apply(connection: &mut PgConnection) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the migration applies");
}

#[test]
fn the_migration_adds_exactly_its_schema_populates_exactly_and_rolls_back_only_when_clear() {
    let (_guard, fx) = setup();
    let pool = fx.pool.as_ref();
    let mut connection = PgConnection::establish(&test_db_url()).expect("a connection");
    let with = catalog(&mut connection);

    // Records of every class and mask, then the tables dropped.
    let (january, february) = (date(2026, 1, 1), date(2026, 2, 1));
    let mut observations: Vec<Obs> = (0..8_u8)
        .flat_map(|mask| {
            [
                month_obs(
                    usize::from(mask % 3),
                    false,
                    january,
                    mask,
                    1 + i64::from(mask),
                ),
                month_obs(0, true, february, mask, i64::from(mask) - 3),
            ]
        })
        .collect();
    observations.extend([
        month_obs(2, false, february, 2, 3),
        month_obs(2, false, february, 1, 4),
        month_obs(1, true, january, 6, 5),
        month_obs(1, true, january, 3, 6),
        Obs {
            end: february + Days::new(40),
            grain: MetricReportingGrain::ReportingPeriod,
            ..month_obs(1, false, february, 2, 4)
        },
    ]);
    assert!(fx.ingest("every class", false, &observations).is_some());
    let mut zero = observations[0];
    zero.value = 0;
    assert!(fx.ingest("to zero", false, &[zero]).is_some());
    check(&fx, "before the rollback");

    revert(&mut connection).expect("down.sql runs with no window or succession");
    let without = catalog(&mut connection);
    let added: BTreeSet<String> = with.difference(&without).cloned().collect();
    assert_eq!(added, migration_lines(), "exactly section 4.1's schema");
    assert!(without.is_subset(&with), "the migration removes nothing");

    // Withdrawn and inconsistent records, written while no table maintains.
    let record = |obs: &Obs| {
        format!(
            "(SELECT record_id FROM metric_record WHERE {})",
            fx.record_where(obs)
        )
    };
    let pointer = |obs: &Obs| {
        format!(
            "(SELECT current_revision_id FROM metric_record WHERE record_id = {})",
            record(obs)
        )
    };
    exec(
        pool,
        &format!(
            "UPDATE metric_record_revision SET status = 'RETRACTED' WHERE record_revision_id = {}",
            pointer(&month_obs(1, false, january, 4, 5))
        ),
    );
    exec(
        pool,
        &format!(
            "UPDATE metric_record SET current_revision_id = NULL WHERE record_id = {}",
            record(&month_obs(1, false, january, 7, 8))
        ),
    );
    exec(
        pool,
        &format!(
            "UPDATE metric_record SET period_end = '2026-02-02' WHERE record_id = {}",
            record(&month_obs(2, false, january, 2, 3))
        ),
    );

    apply(&mut connection);
    assert_eq!(catalog(&mut connection), with);
    check(&fx, "populated by up.sql");
    let populated = actual(pool);
    assert!(populated.refusals.iter().any(|line| line.ends_with("|t")));
    assert!(populated.months.iter().any(|line| line.contains("|null|")));
    for (statement, name) in [
        (format!("INSERT INTO metric_native_month VALUES ('{0}', '{0}', '{0}', '2026-01-02', 1, TRUE, FALSE, FALSE, FALSE, FALSE)", Uuid::new_v4()), "month_start"),
        (format!("INSERT INTO metric_native_country_month VALUES ('{0}', '{0}', '{0}', '2026-01-01', 'gb', 1, FALSE)", Uuid::new_v4()), "country_code"),
    ] {
        let error = sql_query(&statement).execute(&mut connection).expect_err(name);
        assert!(error.to_string().contains(name), "{error}");
    }

    for setting in [
        "window_start = '2026-01-01'",
        "window_end = '2026-01-01'",
        "succession = 'site-a'",
    ] {
        exec(
            pool,
            &format!("UPDATE metric_source_account SET {setting} WHERE code = '{ACCOUNT_A}'"),
        );
        let error = revert(&mut connection).expect_err(setting);
        assert!(
            error.contains("MET-WP4-MONTH-01 rollback refused"),
            "{error}"
        );
        // `revert` first reverted the newer, clear migrations (MET-QUARTER-01);
        // re-applying them restores the schema, so only this stage's own
        // refused revert could leave a difference.
        apply(&mut connection);
        assert_eq!(
            catalog(&mut connection),
            with,
            "{setting}: nothing is dropped"
        );
        exec(pool, "UPDATE metric_source_account SET window_start = NULL, window_end = NULL, succession = NULL");
    }
    revert(&mut connection).expect("down.sql runs once every account is clear");
    assert_eq!(catalog(&mut connection), without);
    apply(&mut connection);
    assert_eq!(catalog(&mut connection), with);
    assert_eq!(actual(pool), populated, "up.sql re-runs to the same rows");
    check(&fx, "re-applied");
}

/// T1, stream untouched (annex): ingested `MONTH` records leave their deltas
/// `PENDING` with no position and the stream's sequence, frontier and
/// watermark unchanged, and a claim takes only the positioned `DAY` delta.
#[test]
fn month_records_leave_the_rollup_stream_untouched_and_a_claim_skips_them() {
    let (_guard, fx) = setup();
    let stream = |pool: &PgPool| {
        lines(
            pool,
            "SELECT next_sequence::text || ' ' || applied_through_sequence::text || ' ' \
                    || watermark_at::text AS line \
               FROM metric_rollup_work_day_state",
        )
    };
    let month_deltas = |pool: &PgPool| {
        lines(
            pool,
            "SELECT d.delta_id::text || ' ' || d.status || ' ' \
                    || COALESCE(d.work_day_sequence::text, 'unpositioned') AS line \
               FROM metric_rollup_delta d \
               JOIN metric_record r ON r.record_id = d.record_id \
              WHERE r.reporting_grain = 'MONTH' \
              ORDER BY d.delta_id",
        )
    };
    let before = stream(&fx.pool);
    fx.ingest(
        "t1-months",
        false,
        &[
            month_obs(0, false, date(2024, 1, 1), 0, 7),
            month_obs(1, true, date(2024, 2, 1), 2, -3),
        ],
    )
    .expect("the months are ingested");
    let months = month_deltas(&fx.pool);
    assert_eq!(months.len(), 2, "{months:?}");
    assert!(
        months
            .iter()
            .all(|line| line.ends_with(" PENDING unpositioned")),
        "{months:?}"
    );
    assert_eq!(stream(&fx.pool), before, "the stream does not move");
    check(&fx, "months");

    let day = Obs {
        end: date(2024, 3, 2),
        grain: MetricReportingGrain::Day,
        ..month_obs(0, false, date(2024, 3, 1), 0, 5)
    };
    fx.ingest("t1-day", false, &[day])
        .expect("the day is ingested");
    let claims = claim_metric_rollup_deltas(&fx.pool, "t1-claimant", 50).expect("a claim");
    assert_eq!(
        claims.len(),
        1,
        "only the DAY delta is positioned and claimed"
    );
    let claimed = claims[0].delta_id.to_string();
    assert!(
        months.iter().all(|line| !line.starts_with(&claimed)),
        "a claim never takes a MONTH delta"
    );
    assert_eq!(
        month_deltas(&fx.pool),
        months,
        "the claim leaves the MONTH deltas"
    );
}

// ---------------------------------------------------------------------------
// MET-WP4-MONTH-01 T1 (its `ON` part), T2 and T14: the reads over ingested
// months and rolled-up days
// ---------------------------------------------------------------------------

fn next_month(month: NaiveDate) -> NaiveDate {
    month + Months::new(1)
}

fn known(text: &str) -> Option<String> {
    Some(text.to_string())
}

/// The id of the seeded measure `code`.
fn measure_id(fx: &Fx, code: &str) -> Uuid {
    lines(
        &fx.pool,
        &format!("SELECT measure_id::text AS line FROM metric_measure WHERE code = '{code}'"),
    )[0]
    .parse()
    .expect("a measure id")
}

/// `net_units` reported by day too, so a signed day side can net to 0.
fn allow_daily_net_units(fx: &Fx) {
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_platform_measure \
                SET supported_grains = \
                    ARRAY['DAY','MONTH','REPORTING_PERIOD']::metric_reporting_grain[] \
              WHERE measure_id = '{}'",
            measure_id(fx, "net_units")
        ),
    );
}

/// Give the publisher the dashboard and widget capabilities, and both
/// accounts `COMPLETE` coverage of both measures over `[start, end)`, with
/// countries and institutions, each from its own completed import (the
/// coverage runs derived as the trigger would).
fn entitle_and_cover(fx: &Fx, start: NaiveDate, end: NaiveDate) {
    exec(
        &fx.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'SPHINX' WHERE publisher_id = '{}'",
            fx.publisher_id
        ),
    );
    let net_units = measure_id(fx, "net_units");
    for code in [ACCOUNT_A, ACCOUNT_B] {
        let account: Uuid = lines(
            &fx.pool,
            &format!(
                "SELECT source_account_id::text AS line FROM metric_source_account \
                 WHERE code = '{code}'"
            ),
        )[0]
        .parse()
        .expect("an account id");
        let import = Uuid::new_v4();
        let mut statements = import_sql(import, account, "COMPLETED", "'2025-02-01T00:00:00Z'");
        for measure in [fx.title_sessions, net_units] {
            statements.push_str(&coverage_sql(
                account,
                import,
                fx.platform_id,
                measure,
                start,
                end,
                "COMPLETE",
                true,
                true,
            ));
        }
        dashboard_exec(&fx.pool, &statements);
    }
}

/// A `metricDashboard` request of the publisher, or of `works` of it, on
/// the fixture platform and `measures`, over `[start, end)` (`start`
/// omitted for all time) at `grain`, with the given sections.
fn native_request(
    fx: &Fx,
    works: &[Uuid],
    measures: &[Uuid],
    (start, end): (Option<NaiveDate>, NaiveDate),
    grain: MetricTimelineGrain,
    (countries, institutions): (bool, bool),
) -> MetricDashboardInput {
    MetricDashboardInput {
        selector: MetricSelectorInput {
            publisher_ids: Some(vec![fx.publisher_id]),
            work_ids: (!works.is_empty()).then(|| works.to_vec()),
            ..MetricSelectorInput::default()
        },
        start_date: start,
        end_date: end,
        measures: Some(measures.to_vec()),
        platforms: Some(vec![fx.platform_id]),
        timeline_grain: Some(grain),
        include_countries: Some(countries),
        include_institutions: Some(institutions),
    }
}

/// A widget read, under `ON`, of the fixture work `work`.
fn native_widget(
    fx: &Fx,
    work: usize,
    (start, end): (Option<NaiveDate>, NaiveDate),
    grain: MetricWidgetGrain,
) -> Result<MetricWidget, MetricWidgetError> {
    let doi = fx.dois[work]
        .trim_start_matches("https://doi.org/")
        .to_string();
    metric_widget_with_month_serving(
        &fx.pool,
        MetricWidgetSwitch::On,
        MetricMonthServing::On,
        &AtomicUsize::new(0),
        &MetricWidgetInput {
            doi: Doi(doi),
            chapter_dois: None,
            start_date: start,
            end_date: end,
            timeline_grain: Some(grain),
        },
    )
}

/// T1 (annex), its `ON` part: `MONTH` records just ingested, with no rollup
/// run since, are served at once by `metricDashboard` and by the widget,
/// with no `ROLLUP_LAG` and the `rollupWatermark` of a read made before
/// them; `OFF` refuses the same read as today.
#[test]
fn under_on_ingested_months_are_served_at_once_with_no_lag_and_the_same_watermark() {
    let (_guard, fx) = setup();
    let (start, end) = (date(2024, 1, 1), date(2024, 4, 1));
    entitle_and_cover(&fx, start, end);
    let net_units = measure_id(&fx, "net_units");
    let input = native_request(
        &fx,
        &[],
        &[fx.title_sessions, net_units],
        (Some(start), end),
        MetricTimelineGrain::Month,
        (true, true),
    );
    let read = |serving| metric_dashboard_with_month_serving(&fx.pool, &input, serving);
    let values = |dashboard: &MetricDashboard, measure: Uuid| {
        let total = dashboard
            .totals
            .iter()
            .find(|total| total.measure_id == measure)
            .and_then(|total| total.value)
            .map(|value| value.to_string());
        let buckets: Vec<Option<String>> = dashboard
            .timeline
            .iter()
            .filter(|bucket| bucket.measure_id == measure)
            .map(|bucket| bucket.value.map(|value| value.to_string()))
            .collect();
        (total, buckets)
    };
    let before = read(MetricMonthServing::On).expect("served before the months");
    assert_eq!(
        values(&before, fx.title_sessions),
        (known("0"), vec![known("0"); 3])
    );
    fx.ingest(
        "t1-on",
        false,
        &[
            month_obs(0, false, date(2024, 1, 1), 0, 7),
            month_obs(1, true, date(2024, 2, 1), 2, -3),
        ],
    )
    .expect("the months are ingested");

    let after = read(MetricMonthServing::On).expect("served at once");
    assert_eq!(after.rollup_watermark, before.rollup_watermark);
    assert!(
        after
            .warnings
            .iter()
            .all(|warning| warning.code != MetricWarningCode::RollupLag),
        "{:?}",
        after.warnings
    );
    assert_eq!(
        values(&after, fx.title_sessions),
        (known("7"), vec![known("7"), known("0"), known("0")])
    );
    assert_eq!(
        values(&after, net_units),
        (known("-3"), vec![known("0"), known("-3"), known("0")])
    );
    assert_eq!(
        after
            .countries
            .iter()
            .map(|row| (
                row.measure_id,
                row.country_code.clone(),
                row.value.to_string()
            ))
            .collect::<Vec<_>>(),
        vec![(net_units, "GB".to_string(), "-3".to_string())]
    );
    assert_eq!(after.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(
        read(MetricMonthServing::Off),
        Err(MetricReadError::UnsupportedSourceGrain),
        "OFF refuses as today"
    );

    let widget = native_widget(&fx, 0, (Some(start), end), MetricWidgetGrain::Month)
        .expect("the widget serves the month at once");
    assert_eq!(widget.rollup_watermark, before.rollup_watermark);
    assert_eq!(
        widget
            .work
            .timeline
            .iter()
            .map(|bucket| (
                bucket.measure_id,
                bucket.value.map(|value| value.to_string())
            ))
            .collect::<Vec<_>>(),
        vec![
            (fx.title_sessions, known("7")),
            (fx.title_sessions, known("0")),
            (fx.title_sessions, known("0")),
        ]
    );
    assert_eq!(
        widget.work.coverage[0].status,
        MetricCoverageStatus::Complete
    );
}

/// What one generated (work, measure, month) cell of T2 and T14 holds on
/// its native side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeSide {
    Absent,
    Live,
    /// Every record written or revised to 0.
    Zero,
    /// Masks 2 and 1 alone.
    TotalAmbiguous,
    /// Masks 0, 3 and 6.
    CountryAmbiguous,
}

/// What one generated cell holds on its day side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DaySide {
    Absent,
    Live,
    /// Rows on one to three days, every value 0.
    Zero,
    /// An undimensioned 0 beside a non-zero country row on one day.
    ZeroTotalBesideSection,
    /// One row +x on the 3rd and -x on the 21st (a signed measure).
    SignedNetZero,
    /// Two countries, +x and -x, on one day (a signed measure).
    CountriesNettingToZero,
    /// Masks 2 and 1 alone on the 10th.
    TotalAmbiguous,
    /// Masks 0, 3 and 5 on the 5th.
    InstitutionAmbiguous,
}

/// Every mask set whose total and both sections resolve; all eight masks
/// are among them.
const RESOLVING: [&[u8]; 20] = [
    &[0],
    &[0, 2],
    &[0, 1],
    &[0, 4],
    &[0, 2, 1],
    &[0, 3],
    &[0, 6],
    &[0, 5],
    &[0, 7],
    &[2],
    &[1],
    &[3],
    &[4],
    &[5],
    &[6],
    &[7],
    &[0, 2, 3],
    &[0, 1, 3],
    &[0, 2, 6],
    &[0, 1, 5],
];

/// The (country, institution) indices of one mask's rows: one or more of
/// its combinations.
fn combinations(generator: &mut Generator, mask: u8) -> Vec<(usize, usize)> {
    let countries: &[usize] = if mask & 2 != 0 { &[0, 1] } else { &[0] };
    let institutions: &[usize] = if mask & 1 != 0 { &[0, 1] } else { &[0] };
    let all: Vec<(usize, usize)> = countries
        .iter()
        .flat_map(|country| {
            institutions
                .iter()
                .map(move |institution| (*country, *institution))
        })
        .collect();
    let mut chosen: Vec<(usize, usize)> = all
        .iter()
        .copied()
        .filter(|_| generator.below(2) == 0)
        .collect();
    if chosen.is_empty() {
        chosen.push(all[generator.below(all.len() as u64) as usize]);
    }
    chosen
}

/// A non-zero value, signed for `net_units`.
fn nonzero(generator: &mut Generator, net: bool) -> i64 {
    let value = 1 + generator.below(20) as i64;
    if net && generator.below(2) == 0 {
        -value
    } else {
        value
    }
}

/// One `DAY` observation.
fn day_obs(
    work: usize,
    net: bool,
    day: NaiveDate,
    mask: u8,
    (country, institution): (usize, usize),
    value: i64,
) -> Obs {
    Obs {
        end: day + Days::new(1),
        grain: MetricReportingGrain::Day,
        country,
        institution,
        ..month_obs(work, net, day, mask, value)
    }
}

/// The `MONTH` records of one native side.
fn native_side(
    generator: &mut Generator,
    work: usize,
    net: bool,
    month: NaiveDate,
    side: NativeSide,
) -> Vec<Obs> {
    let masks: &[u8] = match side {
        NativeSide::Absent => return Vec::new(),
        NativeSide::Live | NativeSide::Zero => {
            RESOLVING[generator.below(RESOLVING.len() as u64) as usize]
        }
        NativeSide::TotalAmbiguous => &[2, 1],
        NativeSide::CountryAmbiguous => &[0, 3, 6],
    };
    let mut records = Vec::new();
    for mask in masks {
        for (country, institution) in combinations(generator, *mask) {
            let value = if side == NativeSide::Zero {
                0
            } else {
                nonzero(generator, net)
            };
            records.push(Obs {
                country,
                institution,
                ..month_obs(work, net, month, *mask, value)
            });
        }
    }
    records
}

/// The `DAY` records of one day side.
fn day_side(
    generator: &mut Generator,
    work: usize,
    net: bool,
    month: NaiveDate,
    side: DaySide,
) -> Vec<Obs> {
    let last = next_month(month) - Days::new(1);
    let some_day = |generator: &mut Generator| match generator.below(4) {
        0 => month,
        1 => last,
        _ => month + Days::new(1 + generator.below(27)),
    };
    let mut rows = Vec::new();
    match side {
        DaySide::Absent => {}
        DaySide::Live | DaySide::Zero => {
            let mut days = BTreeSet::new();
            for _ in 0..1 + generator.below(3) {
                days.insert(some_day(generator));
            }
            for day in days {
                let masks = RESOLVING[generator.below(RESOLVING.len() as u64) as usize];
                for mask in masks {
                    for dims in combinations(generator, *mask) {
                        let value = if side == DaySide::Zero {
                            0
                        } else {
                            nonzero(generator, net)
                        };
                        rows.push(day_obs(work, net, day, *mask, dims, value));
                    }
                }
            }
        }
        DaySide::ZeroTotalBesideSection => {
            let day = some_day(generator);
            rows.push(day_obs(work, net, day, 0, (0, 0), 0));
            let country = generator.below(2) as usize;
            rows.push(day_obs(
                work,
                net,
                day,
                2,
                (country, 0),
                nonzero(generator, false),
            ));
        }
        DaySide::SignedNetZero => {
            assert!(net, "a signed measure");
            let mask = [0, 2, 3, 6][generator.below(4) as usize];
            let dims = (generator.below(2) as usize, generator.below(2) as usize);
            let value = 1 + generator.below(20) as i64;
            rows.push(day_obs(work, net, month + Days::new(2), mask, dims, value));
            rows.push(day_obs(
                work,
                net,
                month + Days::new(20),
                mask,
                dims,
                -value,
            ));
        }
        DaySide::CountriesNettingToZero => {
            assert!(net, "a signed measure");
            let day = some_day(generator);
            let value = 1 + generator.below(20) as i64;
            rows.push(day_obs(work, net, day, 2, (0, 0), value));
            rows.push(day_obs(work, net, day, 2, (1, 0), -value));
        }
        DaySide::TotalAmbiguous => {
            let day = month + Days::new(9);
            rows.push(day_obs(work, net, day, 2, (0, 0), nonzero(generator, net)));
            rows.push(day_obs(work, net, day, 1, (0, 0), nonzero(generator, net)));
        }
        DaySide::InstitutionAmbiguous => {
            let day = month + Days::new(4);
            rows.push(day_obs(work, net, day, 0, (0, 0), nonzero(generator, net)));
            rows.push(day_obs(work, net, day, 3, (0, 0), nonzero(generator, net)));
            rows.push(day_obs(work, net, day, 5, (0, 1), nonzero(generator, net)));
        }
    }
    rows
}

/// The dimension tuple of `obs` within its (work, measure): which of the
/// publication, country and institution it names.
fn dimension_tuple(obs: &Obs) -> (bool, Option<usize>, Option<usize>) {
    (
        obs.mask & 4 != 0,
        (obs.mask & 2 != 0).then_some(obs.country),
        (obs.mask & 1 != 0).then_some(obs.institution),
    )
}

/// The two request rounds of T2 and T14 over 2024: the designed cells, and a
/// seeded draw for every other (work, measure, month). A third of the keys
/// are first written in round one with another value and revised to theirs
/// in round two: up, down or to 0.
fn mixed_plan(seed: u64) -> [Vec<Obs>; 2] {
    use DaySide as D;
    use NativeSide as N;
    let designed: BTreeMap<(usize, bool, u32), (NativeSide, DaySide)> = BTreeMap::from([
        ((0, false, 2), (N::Live, D::Live)),
        ((1, true, 3), (N::Live, D::SignedNetZero)),
        ((1, true, 4), (N::Live, D::CountriesNettingToZero)),
        ((2, false, 5), (N::Live, D::ZeroTotalBesideSection)),
        ((0, true, 6), (N::Zero, D::Live)),
        ((2, true, 2), (N::Live, D::Zero)),
        ((0, false, 7), (N::TotalAmbiguous, D::Live)),
        ((0, true, 9), (N::Absent, D::InstitutionAmbiguous)),
        ((1, false, 10), (N::CountryAmbiguous, D::Absent)),
        ((2, true, 11), (N::TotalAmbiguous, D::Absent)),
        ((2, false, 12), (N::Absent, D::TotalAmbiguous)),
    ]);
    let mut generator = Generator(seed);
    let [mut first, mut second] = [Vec::new(), Vec::new()];
    for number in 1..=12 {
        let month = date(2024, number, 1);
        for work in 0..3 {
            for net in [false, true] {
                let (native, day) = match designed.get(&(work, net, number)) {
                    Some(sides) => *sides,
                    None => match generator.below(12) {
                        0 | 1 => (N::Absent, D::Absent),
                        2..=4 => (N::Live, D::Absent),
                        5..=7 => (N::Absent, D::Live),
                        8 | 9 => (N::Live, D::Live),
                        10 => (N::Zero, D::Live),
                        _ => (N::Live, D::Zero),
                    },
                };
                // Ingestion refuses a record whose exact dimensional cell
                // already holds an overlapping period, so the two sides are
                // redrawn until they share no dimension tuple.
                let observations = loop {
                    let mut observations = native_side(&mut generator, work, net, month, native);
                    let tuples: BTreeSet<_> = observations.iter().map(dimension_tuple).collect();
                    let days = day_side(&mut generator, work, net, month, day);
                    if days
                        .iter()
                        .all(|obs| !tuples.contains(&dimension_tuple(obs)))
                    {
                        observations.extend(days);
                        break observations;
                    }
                };
                for obs in observations {
                    if generator.below(3) == 0 {
                        let earlier = loop {
                            let value = if net {
                                generator.below(41) as i64 - 20
                            } else {
                                generator.below(21) as i64
                            };
                            if value != obs.value {
                                break value;
                            }
                        };
                        first.push(Obs {
                            value: earlier,
                            ..obs
                        });
                    }
                    second.push(obs);
                }
            }
        }
    }
    [first, second]
}

/// Ingest the plan of `seed` through the real coordinator on account A, in
/// two rounds of requests, each followed by the real rollup claim and
/// completion; the rows' classifications, counted.
fn ingest_mixed(fx: &Fx, seed: u64) -> BTreeMap<String, usize> {
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    for (round, observations) in mixed_plan(seed).iter().enumerate() {
        for (chunk, part) in observations.chunks(400).enumerate() {
            let key = format!("mixed-{seed}-{round}-{chunk}");
            let outcome = ingest_metric_batch(&fx.pool, &fx.batch(&key, false, part))
                .unwrap_or_else(|error| panic!("{key}: {error:?}"));
            for row in &outcome.rows {
                *classes
                    .entry(format!("{:?}", row.classification))
                    .or_default() += 1;
            }
        }
        apply_all(&fx.pool);
    }
    classes
}

/// Section rows of the 03A tables whose cell `(work, platform, measure,
/// month)` has neither a total nor an ambiguity row.
fn orphan_section_rows(pool: &PgPool) -> usize {
    lines(
        pool,
        "SELECT count(*)::text AS line FROM ( \
             SELECT work_id, platform_id, measure_id, month_start \
               FROM metric_rollup_work_country_month \
             UNION \
             SELECT work_id, platform_id, measure_id, month_start \
               FROM metric_rollup_work_institution_month \
         ) s \
         WHERE NOT EXISTS (SELECT 1 FROM metric_rollup_work_month t \
                            WHERE t.work_id = s.work_id AND t.platform_id = s.platform_id \
                              AND t.measure_id = s.measure_id \
                              AND t.month_start = s.month_start) \
           AND NOT EXISTS (SELECT 1 FROM metric_rollup_work_month_ambiguity a \
                            WHERE a.work_id = s.work_id AND a.platform_id = s.platform_id \
                              AND a.measure_id = s.measure_id \
                              AND a.month_start = s.month_start)",
    )[0]
    .parse()
    .expect("a count")
}

/// T2 (annex), 03A untouched: with `MONTH` and `DAY` records mixed (every
/// mask, both measures, three works, revisions up, down and to 0), rolled
/// up through the real claim and completion, the work-day rows are exactly
/// the `DAY` records' current values, the 03A and yearly tables equal the
/// existing per-day oracle over work-day rows only, every 03A section row
/// has a total or ambiguity row of its cell, and `verifyMetricRollupMonths`
/// reports exact.
#[test]
fn mixed_month_and_day_records_leave_03a_equal_to_the_per_day_oracle() {
    let (_guard, fx) = setup();
    let pool = fx.pool.as_ref();
    allow_daily_net_units(&fx);
    let classes = ingest_mixed(&fx, 20_261_121);
    assert_eq!(
        classes.keys().map(String::as_str).collect::<Vec<_>>(),
        ["Revision", "Winner"],
        "{classes:?}"
    );
    let months = lines(
        pool,
        "SELECT count(*)::text AS line FROM metric_record WHERE reporting_grain = 'MONTH'",
    );
    assert_ne!(months, ["0"], "MONTH records are mixed in");
    let set = |statement: &str| lines(pool, statement).into_iter().collect::<BTreeSet<_>>();
    let from_records = set(
        "SELECT concat_ws('|', r.work_id, coalesce(r.publication_id::text, '-'), r.platform_id, \
                r.measure_id, r.period_start, coalesce(r.country_code::text, '-'), \
                coalesce(r.institution_id::text, '-'), sum(v.value)) AS line \
           FROM metric_record r \
           JOIN metric_record_revision v \
             ON v.record_revision_id = r.current_revision_id AND v.record_id = r.record_id \
          WHERE r.reporting_grain = 'DAY' AND v.status = 'CURRENT' \
          GROUP BY r.work_id, r.publication_id, r.platform_id, r.measure_id, r.period_start, \
                   r.country_code, r.institution_id",
    );
    let work_days = set(
        "SELECT concat_ws('|', work_id, coalesce(publication_id::text, '-'), platform_id, \
                measure_id, day, coalesce(country_code::text, '-'), \
                coalesce(institution_id::text, '-'), value) AS line \
           FROM metric_rollup_work_day",
    );
    assert!(!work_days.is_empty());
    assert_eq!(
        work_days, from_records,
        "work-day rows are the DAY records alone"
    );
    let days = projection(pool);
    assert_eq!(month_state(pool), oracle_month_state(&days));
    assert_eq!(year_state(pool), oracle_year_state(&days));
    assert_eq!(orphan_section_rows(pool), 0);
    let verified = verification(pool);
    assert!(verified.matches, "{verified:?}");
}

/// One row of a cell as plain reads give it: its mask, publication,
/// country and institution (`-` when absent) and value.
#[derive(Debug, Clone)]
struct CellRow {
    mask: u8,
    publication: String,
    country: String,
    institution: String,
    value: i64,
}

/// A cell resolved by the register's generic rule (Amendment 6 for the
/// total; per section, the unique least mask bearing it). Totals and
/// section rows are kept per publication, as the 03A identity keys keep
/// them.
#[derive(Debug, Default)]
struct Resolved {
    /// `None` when the total is ambiguous.
    total: Option<i128>,
    totals: BTreeMap<String, i128>,
    countries: BTreeMap<(String, String), i128>,
    institutions: BTreeMap<(String, String), i128>,
    country_ambiguous: bool,
    institution_ambiguous: bool,
    /// Any row is not 0.
    live: bool,
}

fn resolve(rows: &[CellRow]) -> Resolved {
    let masks: BTreeSet<u8> = rows.iter().map(|row| row.mask).collect();
    let total_mask = if masks.contains(&0) {
        Some(0)
    } else if masks.len() == 1 {
        masks.first().copied()
    } else {
        None
    };
    let (country, country_ambiguous) = least_bearing(&masks, 2);
    let (institution, institution_ambiguous) = least_bearing(&masks, 1);
    let mut resolved = Resolved {
        country_ambiguous,
        institution_ambiguous,
        live: rows.iter().any(|row| row.value != 0),
        ..Resolved::default()
    };
    for row in rows {
        let value = i128::from(row.value);
        if Some(row.mask) == total_mask {
            resolved.total = Some(resolved.total.unwrap_or(0) + value);
            *resolved.totals.entry(row.publication.clone()).or_default() += value;
        }
        if Some(row.mask) == country {
            *resolved
                .countries
                .entry((row.publication.clone(), row.country.clone()))
                .or_default() += value;
        }
        if Some(row.mask) == institution {
            *resolved
                .institutions
                .entry((row.publication.clone(), row.institution.clone()))
                .or_default() += value;
        }
    }
    if total_mask.is_some() && resolved.total.is_none() {
        resolved.total = Some(0);
    }
    resolved
}

/// The expected answer for one served pair.
#[derive(Debug, PartialEq, Eq)]
struct PairAnswer {
    total: Option<String>,
    buckets: Vec<Option<String>>,
    status: MetricCoverageStatus,
    data_through: Option<NaiveDate>,
}

/// The expected answer of one read, its pairs by measure.
#[derive(Debug, PartialEq, Eq)]
struct ReadAnswer {
    pairs: BTreeMap<Uuid, PairAnswer>,
    countries: Vec<(Uuid, String, String)>,
    institutions: Vec<(Uuid, Uuid, String)>,
}

/// T14's oracle: plain reads of `metric_record`, its current revisions and
/// `metric_rollup_work_day` only, never of a native table, under complete
/// coverage of every day it is asked about.
struct ReadOracle {
    /// The served `MONTH` cells, by (work, measure, month).
    natives: BTreeMap<(Uuid, Uuid, NaiveDate), Vec<CellRow>>,
    /// The work-day cells, by (work, measure, day).
    days: BTreeMap<(Uuid, Uuid, NaiveDate), Vec<CellRow>>,
}

impl ReadOracle {
    fn read(fx: &Fx) -> ReadOracle {
        let cells = |statement: String| {
            let mut cells: BTreeMap<(Uuid, Uuid, NaiveDate), Vec<CellRow>> = BTreeMap::new();
            for line in lines(&fx.pool, &statement) {
                let f: Vec<&str> = line.split('|').collect();
                assert_eq!(f.len(), 7, "{line}");
                let key = (
                    f[0].parse().expect("a work"),
                    f[1].parse().expect("a measure"),
                    NaiveDate::parse_from_str(f[2], "%Y-%m-%d").expect("a date"),
                );
                cells.entry(key).or_default().push(CellRow {
                    mask: u8::from(f[3] != "-") * 4
                        + u8::from(f[4] != "-") * 2
                        + u8::from(f[5] != "-"),
                    publication: f[3].into(),
                    country: f[4].into(),
                    institution: f[5].into(),
                    value: f[6].parse().expect("a value"),
                });
            }
            cells
        };
        let platform = fx.platform_id;
        ReadOracle {
            natives: cells(format!(
                "SELECT concat_ws('|', r.work_id, r.measure_id, r.period_start, \
                        coalesce(r.publication_id::text, '-'), \
                        coalesce(r.country_code::text, '-'), \
                        coalesce(r.institution_id::text, '-'), v.value) AS line \
                   FROM metric_record r \
                   JOIN metric_record_revision v \
                     ON v.record_revision_id = r.current_revision_id \
                    AND v.record_id = r.record_id \
                  WHERE r.reporting_grain = 'MONTH' AND v.status = 'CURRENT' \
                    AND r.platform_id = '{platform}' \
                    AND r.period_end = (r.period_start + interval '1 month')::date \
                    AND extract(day FROM r.period_start) = 1"
            )),
            days: cells(format!(
                "SELECT concat_ws('|', work_id, measure_id, day, \
                        coalesce(publication_id::text, '-'), \
                        coalesce(country_code::text, '-'), \
                        coalesce(institution_id::text, '-'), value) AS line \
                   FROM metric_rollup_work_day WHERE platform_id = '{platform}'"
            )),
        }
    }

    /// The day side of `(work, measure, month)` as the 03A month rows the
    /// per-day rule gives it: (it has rows, it is live — a total, country
    /// or institution row not 0, or an ambiguity row — every total row is
    /// 0, some section row is not 0).
    fn day_month(&self, work: Uuid, measure: Uuid, month: NaiveDate) -> (bool, bool, bool, bool) {
        let mut totals: BTreeMap<String, i128> = BTreeMap::new();
        let mut sections: BTreeMap<(u8, String, String), i128> = BTreeMap::new();
        let (mut rows, mut ambiguity) = (false, false);
        for (_, cell) in self
            .days
            .range((work, measure, month)..(work, measure, next_month(month)))
        {
            rows = true;
            let resolved = resolve(cell);
            ambiguity |= resolved.total.is_none()
                || resolved.country_ambiguous
                || resolved.institution_ambiguous;
            for (publication, value) in resolved.totals {
                *totals.entry(publication).or_default() += value;
            }
            for ((publication, code), value) in resolved.countries {
                *sections.entry((2, publication, code)).or_default() += value;
            }
            for ((publication, id), value) in resolved.institutions {
                *sections.entry((1, publication, id)).or_default() += value;
            }
        }
        let sections_live = sections.values().any(|value| *value != 0);
        let live = ambiguity || totals.values().any(|value| *value != 0) || sections_live;
        let zero_totals = !totals.is_empty() && totals.values().all(|value| *value == 0);
        (rows, live, zero_totals, sections_live)
    }

    fn conflicted(&self, work: Uuid, measure: Uuid, month: NaiveDate) -> bool {
        self.natives
            .get(&(work, measure, month))
            .is_some_and(|cell| resolve(cell).live)
            && self.day_month(work, measure, month).1
    }

    /// Every conflicted cell.
    fn conflicts(&self) -> BTreeSet<(Uuid, Uuid, NaiveDate)> {
        self.natives
            .keys()
            .copied()
            .filter(|(work, measure, month)| self.conflicted(*work, *measure, *month))
            .collect()
    }

    /// The refusal of a read of `works` and `measures` over `[start, end)`:
    /// an ambiguous day cell in the range (total, or a requested section),
    /// then an unconflicted native cell wholly inside it that is.
    fn refusal(
        &self,
        works: &[Uuid],
        measures: &[Uuid],
        (start, end): (NaiveDate, NaiveDate),
        (countries, institutions): (bool, bool),
    ) -> Option<MetricReadError> {
        let refuses = |resolved: &Resolved| {
            resolved.total.is_none()
                || (countries && resolved.country_ambiguous)
                || (institutions && resolved.institution_ambiguous)
        };
        let day_refuses = self.days.iter().any(|((work, measure, day), cell)| {
            works.contains(work)
                && measures.contains(measure)
                && start <= *day
                && *day < end
                && refuses(&resolve(cell))
        });
        let native_refuses = self.natives.iter().any(|((work, measure, month), cell)| {
            works.contains(work)
                && measures.contains(measure)
                && start <= *month
                && next_month(*month) <= end
                && !self.conflicted(*work, *measure, *month)
                && refuses(&resolve(cell))
        });
        (day_refuses || native_refuses).then_some(MetricReadError::DimensionScopeAmbiguous)
    }

    /// The `TOTAL` rows `NATIVE_MONTHS_SQL` must give for `works` and
    /// `measures` over `[start, end)` (`start` omitted for all time):
    /// `measure|month|total|conflict|wholly_inside`.
    fn native_totals(
        &self,
        works: &[Uuid],
        measures: &[Uuid],
        (start, end): (Option<NaiveDate>, NaiveDate),
    ) -> BTreeSet<String> {
        let mut months: BTreeMap<(Uuid, NaiveDate), (Option<i128>, bool)> = BTreeMap::new();
        for ((work, measure, month), cell) in &self.natives {
            if !works.contains(work)
                || !measures.contains(measure)
                || *month >= end
                || start.is_some_and(|start| next_month(*month) <= start)
            {
                continue;
            }
            let entry = months.entry((*measure, *month)).or_insert((None, false));
            if self.conflicted(*work, *measure, *month) {
                entry.1 = true;
            } else if let Some(total) = resolve(cell).total {
                entry.0 = Some(entry.0.unwrap_or(0) + total);
            }
        }
        months
            .into_iter()
            .map(|((measure, month), (total, conflict))| {
                let whole = start.is_none_or(|start| month >= start) && next_month(month) <= end;
                format!(
                    "{measure}|{month}|{}|{}|{}",
                    total.map_or("null".to_string(), |total| total.to_string()),
                    flag(conflict),
                    flag(whole)
                )
            })
            .collect()
    }

    /// The answer of a read of `works` and `measures` over `[start, end)`
    /// at the served `grain`, every day completely covered.
    fn answer(
        &self,
        works: &[Uuid],
        measures: &[Uuid],
        (start, end): (NaiveDate, NaiveDate),
        grain: MetricTimelineGrain,
        sections: (bool, bool),
    ) -> Result<ReadAnswer, MetricReadError> {
        if let Some(error) = self.refusal(works, measures, (start, end), sections) {
            return Err(error);
        }
        let buckets = oracle_buckets(start, end, grain);
        let mut countries: BTreeMap<(Uuid, String), i128> = BTreeMap::new();
        let mut institutions: BTreeMap<(Uuid, Uuid), i128> = BTreeMap::new();
        let mut pairs = BTreeMap::new();
        for measure in measures {
            let mut add_sections = |resolved: &Resolved| {
                for ((_, code), value) in &resolved.countries {
                    *countries.entry((*measure, code.clone())).or_default() += value;
                }
                for ((_, id), value) in &resolved.institutions {
                    let id: Uuid = id.parse().expect("an institution");
                    *institutions.entry((*measure, id)).or_default() += value;
                }
            };
            // The day side: each day's total over the works.
            let mut day_totals: BTreeMap<NaiveDate, i128> = BTreeMap::new();
            for ((_, _, day), cell) in self.days.iter().filter(|((work, cell_measure, day), _)| {
                cell_measure == measure && works.contains(work) && start <= *day && *day < end
            }) {
                let resolved = resolve(cell);
                *day_totals.entry(*day).or_default() += resolved.total.expect("not refused");
                add_sections(&resolved);
            }
            // The native side: per month overlapping the range, the
            // unconflicted cells' total and whether one is conflicted.
            let mut months: BTreeMap<NaiveDate, (Option<i128>, bool)> = BTreeMap::new();
            for ((work, cell_measure, month), cell) in &self.natives {
                if cell_measure != measure
                    || !works.contains(work)
                    || *month >= end
                    || next_month(*month) <= start
                {
                    continue;
                }
                let entry = months.entry(*month).or_insert((None, false));
                if self.conflicted(*work, *measure, *month) {
                    entry.1 = true;
                    continue;
                }
                let resolved = resolve(cell);
                if let Some(total) = resolved.total {
                    entry.0 = Some(entry.0.unwrap_or(0) + total);
                }
                if start <= *month && next_month(*month) <= end {
                    add_sections(&resolved);
                }
            }
            // The cut set: conflicted months, and those not wholly inside
            // the range and the bucket of their first day.
            let cut: Vec<(NaiveDate, NaiveDate)> = months
                .iter()
                .filter(|(month, (_, conflict))| {
                    let month_end = next_month(**month);
                    let whole = start <= **month
                        && month_end <= end
                        && buckets
                            .iter()
                            .find(|(from, to)| *from <= **month && **month < *to)
                            .is_some_and(|(_, to)| month_end <= *to);
                    *conflict || !whole
                })
                .map(|(month, _)| ((*month).max(start), next_month(*month).min(end)))
                .collect();
            let value = |from: NaiveDate, to: NaiveDate| -> Option<String> {
                let mut sum: Option<i128> = None;
                for total in day_totals.range(from..to).map(|(_, total)| *total) {
                    sum = Some(sum.unwrap_or(0) + total);
                }
                for (month, (total, _)) in months.range(from..to) {
                    if let (true, Some(total)) = (next_month(*month) <= to, total) {
                        sum = Some(sum.unwrap_or(0) + total);
                    }
                }
                match sum {
                    Some(sum) => Some(sum.to_string()),
                    None if cut
                        .iter()
                        .all(|(cut_from, cut_to)| *cut_to <= from || *cut_from >= to) =>
                    {
                        known("0")
                    }
                    None => None,
                }
            };
            let first_break = cut.iter().map(|(from, _)| *from).min();
            pairs.insert(
                *measure,
                PairAnswer {
                    total: value(start, end),
                    buckets: buckets.iter().map(|(from, to)| value(*from, *to)).collect(),
                    status: if cut.is_empty() {
                        MetricCoverageStatus::Complete
                    } else {
                        MetricCoverageStatus::Partial
                    },
                    data_through: match first_break {
                        Some(day) if day <= start => None,
                        Some(day) => day.pred_opt(),
                        None => end.pred_opt(),
                    },
                },
            );
        }
        let (with_countries, with_institutions) = sections;
        Ok(ReadAnswer {
            pairs,
            countries: if with_countries {
                countries
                    .into_iter()
                    .map(|((measure, code), value)| (measure, code, value.to_string()))
                    .collect()
            } else {
                Vec::new()
            },
            institutions: if with_institutions {
                institutions
                    .into_iter()
                    .map(|((measure, id), value)| (measure, id, value.to_string()))
                    .collect()
            } else {
                Vec::new()
            },
        })
    }

    /// The measures the widget serves for `work` over the months `[from,
    /// to)`: those with a work-day row or a native cell there.
    fn widget_measures(
        &self,
        work: Uuid,
        measures: &[Uuid],
        (from, to): (NaiveDate, NaiveDate),
    ) -> Vec<Uuid> {
        measures
            .iter()
            .copied()
            .filter(|measure| {
                self.days
                    .range((work, *measure, from)..(work, *measure, to))
                    .next()
                    .is_some()
                    || self
                        .natives
                        .range((work, *measure, from)..(work, *measure, to))
                        .next()
                        .is_some()
            })
            .collect()
    }

    /// The first day of the earliest month with a work-day row or a native
    /// cell of `work` before `end`.
    fn first_month(&self, work: Uuid, measures: &[Uuid], end: NaiveDate) -> Option<NaiveDate> {
        measures
            .iter()
            .flat_map(|measure| {
                let day = self
                    .days
                    .range((work, *measure, NaiveDate::MIN)..(work, *measure, end))
                    .next()
                    .map(|((_, _, day), _)| *day);
                let month = self
                    .natives
                    .range((work, *measure, NaiveDate::MIN)..(work, *measure, end))
                    .next()
                    .map(|((_, _, month), _)| *month);
                [day, month]
            })
            .flatten()
            .min()
            .map(|day| day.with_day(1).expect("a first day"))
    }
}

/// One served pair: its total, its buckets and their values.
type ServedPair = (
    Option<String>,
    Vec<(NaiveDate, NaiveDate)>,
    Vec<Option<String>>,
);

/// The total, the buckets and the bucket values of `measure` in one served
/// answer.
fn served_pair(
    totals: &[crate::model::metric_dashboard::MetricTotal],
    timeline: &[crate::model::metric_dashboard::MetricTimeBucket],
    measure: Uuid,
) -> ServedPair {
    let total = totals
        .iter()
        .find(|total| total.measure_id == measure)
        .and_then(|total| total.value)
        .map(|value| value.to_string());
    let buckets = timeline
        .iter()
        .filter(|bucket| bucket.measure_id == measure)
        .map(|bucket| (bucket.start_date, bucket.end_date))
        .collect();
    let values = timeline
        .iter()
        .filter(|bucket| bucket.measure_id == measure)
        .map(|bucket| bucket.value.map(|value| value.to_string()))
        .collect();
    (total, buckets, values)
}

/// T14 (`MET-WP4-MONTH-01`), with a fixed seed: `MONTH` records over all
/// eight dimension masks, three works and twelve months, ingested through
/// the real coordinator and revised up, down and to 0, mixed with `DAY`
/// records rolled up through the real claim and completion, with designed
/// conflicts, zero sides, zero totals beside non-zero section rows, signed
/// months netting to 0 and ambiguity on either side. No 03A section row
/// lacks a total or ambiguity row of its cell. Under `ON`,
/// `metricDashboard` (every grain, aligned and clipped ranges, filtered
/// works, explicit and omitted start) and the widget in pair mode
/// (explicit and all-history) give the totals, buckets, sections, statuses,
/// `dataThrough` and ambiguity refusals of an oracle computed from plain
/// reads of `metric_record`, its current revisions and
/// `metric_rollup_work_day` only, and `NATIVE_MONTHS_SQL`'s `TOTAL` rows
/// carry its conflict flags. The register's checks over this database:
/// V-NATIVE 0, V-CONFLICT the oracle's conflicted cells, V-MONTH clear and
/// V-PLAN the oracle's live native cells of each plan.
#[test]
fn ingested_months_and_rolled_up_days_read_equal_to_an_independent_oracle() {
    let (_guard, fx) = setup();
    let pool = fx.pool.as_ref();
    allow_daily_net_units(&fx);
    let net_units = measure_id(&fx, "net_units");
    let measures = [fx.title_sessions, net_units];
    let (first_day, end_of_data) = (date(2024, 1, 1), date(2025, 1, 1));
    entitle_and_cover(&fx, first_day, end_of_data);
    let classes = ingest_mixed(&fx, 20_261_120);
    assert_eq!(
        classes.keys().map(String::as_str).collect::<Vec<_>>(),
        ["Revision", "Winner"],
        "{classes:?}"
    );
    assert_eq!(orphan_section_rows(pool), 0);
    let oracle = ReadOracle::read(&fx);

    // The fixture holds what the stage names.
    let mut counters: BTreeMap<&str, usize> = BTreeMap::new();
    let mut bump = |name: &'static str| *counters.entry(name).or_default() += 1;
    let masks: BTreeSet<u8> = oracle
        .natives
        .values()
        .flatten()
        .map(|row| row.mask)
        .collect();
    assert_eq!(masks, (0..8).collect(), "every mask among the months");
    for (work, measure, month) in oracle.natives.keys() {
        let native = resolve(&oracle.natives[&(*work, *measure, *month)]);
        let (rows, live, zero_totals, sections_live) = oracle.day_month(*work, *measure, *month);
        match (rows, native.live, live) {
            (true, true, true) => bump("conflicted cells"),
            (true, false, true) => bump("zero native sides beside a live day side"),
            (true, true, false) => bump("zero day sides beside a live native side"),
            _ => {}
        }
        if rows && live && zero_totals && sections_live {
            bump("zero totals beside a non-zero section row");
        }
        let day_values = oracle
            .days
            .range((*work, *measure, *month)..(*work, *measure, next_month(*month)))
            .flat_map(|(_, cell)| cell)
            .any(|row| row.value != 0);
        if rows && !live && day_values {
            bump("signed day months netting to 0, both sides added");
        }
        if native.total.is_none() {
            bump("total-ambiguous native cells");
        }
        if native.country_ambiguous {
            bump("country-ambiguous native cells");
        }
    }
    for cell in oracle.days.values() {
        let resolved = resolve(cell);
        if resolved.total.is_none() {
            bump("total-ambiguous day cells");
        }
        if resolved.institution_ambiguous {
            bump("institution-ambiguous day cells");
        }
    }
    for line in lines(
        pool,
        "SELECT concat_ws('|', r.reporting_grain, \
                CASE WHEN n.value = 0 THEN 'zero' WHEN n.value > o.value THEN 'up' \
                     ELSE 'down' END) AS line \
           FROM metric_record r \
           JOIN metric_record_revision o ON o.record_id = r.record_id AND o.revision_number = 1 \
           JOIN metric_record_revision n ON n.record_id = r.record_id AND n.revision_number = 2",
    ) {
        match line.as_str() {
            "MONTH|up" => bump("MONTH revisions up"),
            "MONTH|down" => bump("MONTH revisions down"),
            "MONTH|zero" => bump("MONTH revisions to 0"),
            "DAY|up" => bump("DAY revisions up"),
            "DAY|down" => bump("DAY revisions down"),
            _ => bump("DAY revisions to 0"),
        }
    }

    // The conflict flags of NATIVE_MONTHS_SQL's TOTAL rows.
    let native_totals = |works: &[Uuid], range: (Option<NaiveDate>, NaiveDate)| {
        let mut connection = pool.get().expect("a connection");
        sql_query(format!(
            "SELECT concat_ws('|', n.measure_id, n.month_start, coalesce(n.total, 'null'), \
                    n.conflict, n.wholly_inside) AS line \
               FROM ({NATIVE_MONTHS_SQL}) n WHERE n.kind = 'TOTAL'"
        ))
        .bind::<Array<SqlUuid>, _>(works.to_vec())
        .bind::<Array<SqlUuid>, _>(vec![fx.platform_id])
        .bind::<Array<SqlUuid>, _>(measures.to_vec())
        .bind::<Nullable<Date>, _>(range.0)
        .bind::<Date, _>(range.1)
        .bind::<Bool, _>(true)
        .bind::<Bool, _>(true)
        .bind::<SqlBigInt, _>(100_001_i64)
        .load::<Line>(&mut connection)
        .expect("NATIVE_MONTHS_SQL")
        .into_iter()
        .map(|row| row.line)
        .collect::<BTreeSet<String>>()
    };
    for (works, range) in [
        (fx.works.clone(), (Some(first_day), end_of_data)),
        (
            fx.works.clone(),
            (Some(date(2024, 2, 10)), date(2024, 8, 20)),
        ),
        (vec![fx.works[0], fx.works[2]], (None, date(2024, 10, 1))),
        (
            vec![fx.works[1]],
            (Some(date(2024, 3, 1)), date(2024, 5, 15)),
        ),
    ] {
        let expected = oracle.native_totals(&works, &measures, range);
        assert!(!expected.is_empty());
        assert_eq!(native_totals(&works, range), expected, "{range:?}");
    }

    // metricDashboard under ON.
    let mut read_dashboard = |label: &str,
                              works: &[usize],
                              (start, end): (Option<NaiveDate>, NaiveDate),
                              grain: MetricTimelineGrain,
                              sections: (bool, bool)| {
        let selected: Vec<Uuid> = works.iter().map(|work| fx.works[*work]).collect();
        let scope = if selected.is_empty() {
            fx.works.clone()
        } else {
            selected.clone()
        };
        let input = native_request(&fx, &selected, &measures, (start, end), grain, sections);
        let actual = metric_dashboard_with_month_serving(pool, &input, MetricMonthServing::On);
        // All time starts at the earliest evidence: the coverage's first day.
        let from = start.unwrap_or(first_day);
        let refusal = oracle.refusal(&scope, &measures, (from, end), sections);
        let dashboard = match (actual, refusal) {
            (Err(error), Some(expected)) => {
                assert_eq!(error, expected, "{label}");
                bump("refused reads");
                return;
            }
            (Ok(dashboard), None) => dashboard,
            (actual, refusal) => panic!("{label}: {actual:?}, the oracle's refusal {refusal:?}"),
        };
        assert_eq!(
            (dashboard.start_date, dashboard.end_date),
            (from, end),
            "{label}"
        );
        let served = dashboard.timeline_grain;
        assert_ne!(served, MetricTimelineGrain::Auto, "{label}");
        if grain != MetricTimelineGrain::Auto {
            assert_eq!(served, grain, "{label}");
        }
        let answer = oracle
            .answer(&scope, &measures, (from, end), served, sections)
            .expect("not refused");
        for measure in &measures {
            let expected = &answer.pairs[measure];
            let (total, buckets, values) =
                served_pair(&dashboard.totals, &dashboard.timeline, *measure);
            assert_eq!(buckets, oracle_buckets(from, end, served), "{label}");
            assert_eq!(
                (&total, &values),
                (&expected.total, &expected.buckets),
                "{label} {measure}"
            );
            let item = dashboard
                .coverage
                .items
                .iter()
                .find(|item| item.measure_id == *measure)
                .expect("an item");
            assert_eq!(
                (item.status, item.data_through),
                (expected.status, expected.data_through),
                "{label} {measure}"
            );
            match expected.status {
                MetricCoverageStatus::Partial => bump("served pairs PARTIAL from a cut"),
                MetricCoverageStatus::Complete => bump("served pairs COMPLETE"),
                MetricCoverageStatus::Unknown => {}
            }
            if expected.buckets.iter().any(Option::is_none) {
                bump("served pairs with a null bucket");
            }
        }
        assert_eq!(
            dashboard
                .countries
                .iter()
                .map(|row| (
                    row.measure_id,
                    row.country_code.clone(),
                    row.value.to_string()
                ))
                .collect::<Vec<_>>(),
            answer.countries,
            "{label}"
        );
        assert_eq!(
            dashboard
                .institutions
                .iter()
                .map(|row| (row.measure_id, row.institution_id, row.value.to_string()))
                .collect::<Vec<_>>(),
            answer.institutions,
            "{label}"
        );
        bump("served reads");
    };
    use MetricTimelineGrain::{Auto, Day, Month, Year};
    let all: &[usize] = &[];
    for (label, works, start, end, grain, sections) in [
        ("a year", all, first_day, end_of_data, Year, (true, true)),
        (
            "six months",
            all,
            first_day,
            date(2024, 7, 1),
            Month,
            (true, true),
        ),
        (
            "six months at YEAR",
            all,
            first_day,
            date(2024, 7, 1),
            Year,
            (false, false),
        ),
        (
            "two months of days",
            all,
            date(2024, 2, 1),
            date(2024, 4, 1),
            Day,
            (true, true),
        ),
        (
            "clipped months",
            all,
            date(2024, 1, 15),
            date(2024, 6, 20),
            Month,
            (true, false),
        ),
        (
            "a clipped year",
            all,
            date(2024, 3, 10),
            date(2024, 8, 25),
            Year,
            (false, true),
        ),
        (
            "clipped days",
            all,
            date(2024, 4, 3),
            date(2024, 5, 27),
            Day,
            (true, true),
        ),
        (
            "AUTO",
            all,
            date(2024, 6, 15),
            date(2024, 8, 15),
            Auto,
            (true, true),
        ),
        (
            "autumn",
            all,
            date(2024, 9, 1),
            date(2024, 12, 1),
            Month,
            (false, false),
        ),
        (
            "autumn of two works",
            &[0, 1],
            date(2024, 9, 1),
            date(2024, 12, 1),
            Month,
            (false, false),
        ),
        (
            "autumn countries",
            &[1],
            date(2024, 9, 1),
            date(2024, 11, 1),
            Month,
            (true, false),
        ),
        (
            "autumn institutions",
            &[0],
            date(2024, 9, 1),
            date(2024, 11, 1),
            Month,
            (false, true),
        ),
        (
            "a cut ambiguous month",
            &[2],
            date(2024, 11, 10),
            date(2024, 12, 5),
            Day,
            (false, false),
        ),
        (
            "an ambiguous day",
            &[2],
            date(2024, 11, 10),
            date(2024, 12, 20),
            Day,
            (false, false),
        ),
    ] {
        read_dashboard(label, works, (Some(start), end), grain, sections);
    }
    for (label, works, end, grain, sections) in [
        (
            "all time to September",
            &[1, 2][..],
            date(2024, 9, 1),
            Year,
            (true, true),
        ),
        (
            "all time to July, AUTO",
            all,
            date(2024, 7, 1),
            Auto,
            (false, false),
        ),
        (
            "all time of two works",
            &[0, 1],
            end_of_data,
            Year,
            (false, false),
        ),
        (
            "all time to mid-June",
            all,
            date(2024, 6, 15),
            Month,
            (true, true),
        ),
        ("all time", all, end_of_data, Year, (true, true)),
    ] {
        read_dashboard(label, works, (None, end), grain, sections);
    }
    let mut generator = Generator(20_261_122);
    let selections: [&[usize]; 5] = [&[], &[0], &[1], &[2], &[0, 2]];
    for draw in 0..24 {
        let start = first_day + Days::new(generator.below(330));
        let end = start + Days::new(1 + generator.below((end_of_data - start).num_days() as u64));
        let grain = [Day, Month, Year, Auto][generator.below(4) as usize];
        let sections = (generator.below(2) == 0, generator.below(2) == 0);
        let works = selections[generator.below(5) as usize];
        read_dashboard(
            &format!("draw {draw}: {works:?} [{start}, {end}) {grain:?} {sections:?}"),
            works,
            (Some(start), end),
            grain,
            sections,
        );
    }

    // The widget in pair mode under ON.
    for work in 0..3 {
        let work_id = fx.works[work];
        for (label, (start, end), grain) in [
            (
                "six months",
                (Some(first_day), date(2024, 7, 1)),
                MetricWidgetGrain::Month,
            ),
            (
                "summer to autumn",
                (Some(date(2024, 6, 1)), date(2024, 12, 1)),
                MetricWidgetGrain::Month,
            ),
            (
                "all history to September",
                (None, date(2024, 9, 1)),
                MetricWidgetGrain::Year,
            ),
            ("all history", (None, end_of_data), MetricWidgetGrain::Year),
        ] {
            let label = format!("widget, work {work}, {label}");
            let actual = native_widget(&fx, work, (start, end), grain);
            let from = start
                .or_else(|| oracle.first_month(work_id, &measures, end))
                .expect("every work has a served month");
            let served = oracle.widget_measures(work_id, &measures, (from, end));
            let timeline = match grain {
                MetricWidgetGrain::Year => Year,
                _ => Month,
            };
            let expected = oracle.answer(&[work_id], &served, (from, end), timeline, (true, false));
            let widget = match (actual, expected) {
                (Err(error), Err(_)) => {
                    assert_eq!(error, MetricWidgetError::Unavailable, "{label}");
                    bump("refused widget reads");
                    continue;
                }
                (Ok(widget), Ok(answer)) => (widget, answer),
                (actual, expected) => panic!("{label}: {actual:?} against {expected:?}"),
            };
            let (widget, answer) = widget;
            assert_eq!((widget.start_date, widget.end_date), (from, end), "{label}");
            let section = &widget.work;
            assert_eq!(
                section
                    .coverage
                    .iter()
                    .map(|item| item.measure_id)
                    .collect::<BTreeSet<_>>(),
                served.iter().copied().collect::<BTreeSet<_>>(),
                "{label}"
            );
            for item in &section.coverage {
                let expected = &answer.pairs[&item.measure_id];
                let (total, buckets, values) =
                    served_pair(&section.totals, &section.timeline, item.measure_id);
                assert_eq!(buckets, oracle_buckets(from, end, timeline), "{label}");
                assert_eq!(
                    (&total, &values),
                    (&expected.total, &expected.buckets),
                    "{label}"
                );
                assert_eq!(
                    (item.status, item.data_through),
                    (expected.status, expected.data_through),
                    "{label}"
                );
            }
            assert_eq!(
                section
                    .countries
                    .iter()
                    .map(|row| (
                        row.measure_id,
                        row.country_code.clone(),
                        row.value.to_string()
                    ))
                    .collect::<Vec<_>>(),
                answer.countries,
                "{label}"
            );
            bump("served widget reads");
        }
    }

    // The register's checks over this database.
    let conflicts = oracle.conflicts();
    assert_eq!(v_native_total(pool), 0, "V-NATIVE");
    assert_eq!(
        v_conflict(pool),
        i64::try_from(conflicts.len()).expect("a count"),
        "V-CONFLICT"
    );
    let grains = v_month(pool);
    let month_row = &grains["MONTH"];
    let records = oracle.natives.values().map(Vec::len).sum::<usize>();
    assert_eq!(
        (
            month_row["active"].as_i64(),
            month_row["invalid_periods"].as_i64(),
            month_row["inconsistent_pointers"].as_i64(),
            month_row["non_additive"].as_i64(),
        ),
        (
            Some(i64::try_from(records).expect("a count")),
            Some(0),
            Some(0),
            Some(0)
        ),
        "V-MONTH"
    );
    for (from, through) in [
        (date(2023, 12, 1), date(2023, 12, 31)),
        (date(2024, 2, 10), date(2024, 2, 20)),
        (date(2024, 3, 31), date(2024, 6, 1)),
        (first_day, date(2024, 12, 31)),
    ] {
        let live = oracle
            .natives
            .iter()
            .filter(|((_, _, month), cell)| {
                *month <= through && next_month(*month) > from && resolve(cell).live
            })
            .count();
        assert_eq!(
            v_plan(pool, fx.publisher_id, fx.platform_id, from, through),
            i64::try_from(live).expect("a count"),
            "V-PLAN [{from}, {through}]"
        );
    }

    println!("T14 counters: {counters:?}");
    for name in [
        "conflicted cells",
        "zero native sides beside a live day side",
        "zero day sides beside a live native side",
        "zero totals beside a non-zero section row",
        "signed day months netting to 0, both sides added",
        "total-ambiguous native cells",
        "country-ambiguous native cells",
        "total-ambiguous day cells",
        "institution-ambiguous day cells",
        "MONTH revisions up",
        "MONTH revisions down",
        "MONTH revisions to 0",
        "DAY revisions up",
        "DAY revisions down",
        "DAY revisions to 0",
        "refused reads",
        "served reads",
        "served pairs PARTIAL from a cut",
        "served pairs COMPLETE",
        "served pairs with a null bucket",
        "refused widget reads",
        "served widget reads",
    ] {
        assert!(
            counters.get(name).is_some_and(|count| *count > 0),
            "{name}: {counters:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// MET-QUARTER-01: quarters in the projection
// ---------------------------------------------------------------------------

fn quarter_obs(work: usize, start: NaiveDate, mask: u8, value: i64) -> Obs {
    Obs {
        work,
        net: true,
        start,
        end: start + Months::new(3),
        grain: MetricReportingGrain::Quarter,
        mask,
        country: 0,
        institution: 0,
        value,
    }
}

/// `net_units` lists exactly `grains` (set directly, so the registry guard is
/// bypassed to build the states the guard prevents).
fn net_units_grains(fx: &Fx, grains: &str) {
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_platform_measure SET supported_grains = ARRAY[{grains}]::metric_reporting_grain[] \
              WHERE measure_id = '{}'",
            measure_id(fx, "net_units")
        ),
    );
}

fn spans(pool: &PgPool) -> Vec<String> {
    lines(pool, "SELECT concat_ws('|', month_start, span_months, coalesce(value::text, 'null')) AS line FROM metric_native_month ORDER BY month_start")
}

#[test]
fn a_quarter_is_validated_projected_with_its_span_and_matches_v_native() {
    let (_guard, fx) = setup();
    net_units_grains(&fx, "'QUARTER','REPORTING_PERIOD'");
    let q1 = date(2026, 1, 1);
    assert!(fx
        .ingest("q1", false, &[quarter_obs(0, q1, 0, 7)])
        .is_some());
    assert_eq!(spans(&fx.pool), ["2026-01-01|3|7"]);
    // A country row of the same quarter joins the same key and carries the span.
    let mut country = quarter_obs(0, q1, 2, 4);
    country.country = 1;
    assert!(fx.ingest("q1-country", false, &[country]).is_some());
    assert_eq!(
        lines(
            &fx.pool,
            "SELECT concat_ws('|', month_start, span_months, country_code, value) AS line FROM metric_native_country_month"
        ),
        ["2026-01-01|3|US|4"]
    );
    // A misaligned or wrong-length quarter is rejected and projects nothing.
    for (label, start, end) in [
        ("shifted", date(2026, 2, 1), date(2026, 5, 1)),
        ("two months", date(2026, 4, 1), date(2026, 6, 1)),
        ("half year", date(2026, 7, 1), date(2027, 1, 1)),
    ] {
        let mut bad = quarter_obs(1, start, 0, 3);
        bad.end = end;
        assert!(fx.ingest(label, false, &[bad]).is_some(), "{label}");
        assert_eq!(spans(&fx.pool), ["2026-01-01|3|7"], "{label}");
    }
    assert_eq!(
        v_native_total(&fx.pool),
        0,
        "V-NATIVE agrees with the writer"
    );
    let before = actual(&fx.pool);
    repair(&fx.pool, fx.publisher_id);
    assert_eq!(actual(&fx.pool), before, "repair equals ingestion");
}

#[test]
fn a_month_and_an_overlapping_quarter_refuse_each_other_and_clear_together() {
    let (_guard, fx) = setup();
    let feb = date(2026, 2, 1);
    net_units_grains(&fx, "'MONTH','REPORTING_PERIOD'");
    let month = month_obs(0, true, feb, 0, 5);
    assert!(fx.ingest("month", false, &[month]).is_some());
    assert_eq!(spans(&fx.pool), ["2026-02-01|1|5"]);
    // Build the state the registry guard prevents: both grains, different cells.
    net_units_grains(&fx, "'QUARTER','REPORTING_PERIOD'");
    let mut country = quarter_obs(0, date(2026, 1, 1), 2, 4);
    country.country = 1;
    assert!(fx.ingest("quarter", false, &[country]).is_some());
    assert!(spans(&fx.pool).is_empty(), "neither period is served");
    assert_eq!(
        lines(
            &fx.pool,
            "SELECT concat_ws('|', period_start, period_end, inconsistent) AS line FROM metric_native_refusal ORDER BY period_start"
        ),
        ["2026-01-01|2026-04-01|t", "2026-02-01|2026-03-01|t"]
    );
    assert_eq!(
        v_native_total(&fx.pool),
        0,
        "V-NATIVE agrees with the writer"
    );
    let both = actual(&fx.pool);
    repair(&fx.pool, fx.publisher_id);
    assert_eq!(actual(&fx.pool), both, "repair equals ingestion");
    // Withdrawing the month and rewriting the quarter clears both refusals.
    exec(&fx.pool, &format!("UPDATE metric_record_revision SET status = 'RETRACTED' WHERE record_revision_id = (SELECT current_revision_id FROM metric_record WHERE {})", fx.record_where(&month)));
    country.value = 6;
    assert!(fx.ingest("quarter again", false, &[country]).is_some());
    assert_eq!(
        lines(
            &fx.pool,
            "SELECT concat_ws('|', month_start, span_months) AS line FROM metric_native_month"
        ),
        ["2026-01-01|3"]
    );
    assert!(actual(&fx.pool).refusals.is_empty());
    assert_eq!(
        v_native_total(&fx.pool),
        0,
        "V-NATIVE agrees after the clear"
    );
}

#[test]
fn the_registry_refuses_to_mix_month_and_quarter_on_a_pair_that_holds_the_other() {
    use crate::model::metric_platform_measure::crud::update_metric_platform_measure;
    use crate::model::metric_platform_measure::PatchMetricPlatformMeasure;

    let (_guard, fx) = setup();
    let patch = |grains: Vec<MetricReportingGrain>| PatchMetricPlatformMeasure {
        platform_code: PLATFORM.into(),
        measure_code: "net_units".into(),
        supported_grains: grains,
        supports_country: true,
        supports_institution: true,
        supports_publication: true,
        direct_collection: true,
        enabled: true,
    };
    // With no record the pair may switch grain; MONTH and QUARTER together
    // are the CHECK's to refuse.
    assert!(update_metric_platform_measure(
        &fx.pool,
        "test",
        &patch(vec![
            MetricReportingGrain::Month,
            MetricReportingGrain::Quarter
        ])
    )
    .is_err());
    // A MONTH record then blocks adding QUARTER.
    assert!(fx
        .ingest(
            "month",
            false,
            &[month_obs(0, true, date(2026, 2, 1), 0, 5)]
        )
        .is_some());
    let refused = update_metric_platform_measure(
        &fx.pool,
        "test",
        &patch(vec![
            MetricReportingGrain::Quarter,
            MetricReportingGrain::ReportingPeriod,
        ]),
    );
    assert!(matches!(
        refused,
        Err(ThothError::DatabaseConstraintError(_))
    ));
    // And a QUARTER record blocks adding MONTH.
    net_units_grains(&fx, "'QUARTER','REPORTING_PERIOD'");
    assert!(fx
        .ingest("quarter", false, &[quarter_obs(1, date(2026, 4, 1), 0, 2)])
        .is_some());
    let refused = update_metric_platform_measure(
        &fx.pool,
        "test",
        &patch(vec![
            MetricReportingGrain::Month,
            MetricReportingGrain::ReportingPeriod,
        ]),
    );
    assert!(matches!(
        refused,
        Err(ThothError::DatabaseConstraintError(_))
    ));
}

/// A served quarter counts in the buckets that contain it: `QUARTER` and
/// `YEAR` timelines and the range total. A `MONTH` timeline cuts it (no
/// bucket value, a `PARTIAL` pair). `OFF` refuses it, as it refuses a month.
#[test]
fn a_served_quarter_counts_in_quarter_and_year_buckets_and_is_cut_in_month_buckets() {
    let (_guard, fx) = setup();
    let (start, end) = (date(2026, 1, 1), date(2026, 7, 1));
    entitle_and_cover(&fx, start, end);
    net_units_grains(&fx, "'QUARTER','REPORTING_PERIOD'");
    let net_units = measure_id(&fx, "net_units");
    fx.ingest("q1", false, &[quarter_obs(0, date(2026, 1, 1), 0, 9)])
        .expect("the quarter is ingested");
    let read = |grain, serving| {
        metric_dashboard_with_month_serving(
            &fx.pool,
            &native_request(
                &fx,
                &[],
                &[net_units],
                (Some(start), end),
                grain,
                (false, false),
            ),
            serving,
        )
    };
    let buckets = |dashboard: &MetricDashboard| -> Vec<Option<String>> {
        dashboard
            .timeline
            .iter()
            .map(|bucket| bucket.value.map(|value| value.to_string()))
            .collect()
    };
    let total = |dashboard: &MetricDashboard| {
        dashboard
            .totals
            .iter()
            .find(|total| total.measure_id == net_units)
            .and_then(|total| total.value)
            .map(|value| value.to_string())
    };

    let quarters = read(MetricTimelineGrain::Quarter, MetricMonthServing::On).expect("quarters");
    assert_eq!(buckets(&quarters), [known("9"), known("0")]);
    assert_eq!(total(&quarters), known("9"));
    assert_eq!(quarters.coverage.status, MetricCoverageStatus::Complete);

    let years = read(MetricTimelineGrain::Year, MetricMonthServing::On).expect("year");
    assert_eq!(buckets(&years), [known("9")]);
    assert_eq!(total(&years), known("9"));

    let months = read(MetricTimelineGrain::Month, MetricMonthServing::On).expect("months");
    assert_eq!(months.timeline.len(), 6);
    assert_eq!(&buckets(&months)[..3], [None, None, None]);
    assert_eq!(total(&months), known("9"));
    assert_eq!(months.coverage.status, MetricCoverageStatus::Partial);

    assert_eq!(
        read(MetricTimelineGrain::Quarter, MetricMonthServing::Off),
        Err(MetricReadError::UnsupportedSourceGrain),
        "OFF refuses a quarter, as it refuses a month"
    );
}
