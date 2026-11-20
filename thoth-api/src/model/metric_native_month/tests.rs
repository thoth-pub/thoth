//! Tests of the native-month projection (`MET-WP4-MONTH-01`): T15, the
//! projection's exactness through the real coordinator; T3, the migration;
//! and the register's checks, run as the register gives them.
//!
//! The oracle reads `metric_record` and its revisions with plain reads and
//! resolves each cell in Rust by the register's generic rule, sharing no SQL
//! with the writer or with V-NATIVE.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Datelike, Days, Months, NaiveDate};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{Date, Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use thoth_errors::ThothError;
use uuid::Uuid;

use super::{
    recompute, recompute_metric_native_months, NativeMonthKey, NativeMonthKeys,
    RECOMPUTE_STATEMENTS, REPAIR_CHUNK_WORKS, REPAIR_STATEMENTS,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_identifier_quarantine_reconciliation::reconcile_metric_identifier_quarantine;
use crate::model::metric_ingestion::{
    ingest_metric_batch, MetricIngestionBatch, MetricIngestionOutcome, NormalizedMetricObservation,
    SUPPORTED_SCHEMA_VERSION,
};
use crate::model::metric_platform::tests::setup_registry_db;
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record_provenance::MetricRecordProvenanceClassification as Class;
use crate::model::metric_rollup_delta::crud::claim_metric_rollup_deltas;
use crate::model::metric_rollup_delta::tests::{captured, logging_pool};
use crate::model::tests::db::{test_db_url, TestDbGuard};

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

fn v_native(pool: &PgPool) -> BTreeMap<String, i64> {
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

fn v_native_total(pool: &PgPool) -> i64 {
    v_native(pool).values().sum()
}

fn v_native_keys(pool: &PgPool) -> Vec<serde_json::Value> {
    let full = v_native_sql();
    let at = full.rfind("\nSELECT\n").expect("V-NATIVE's final SELECT");
    let keys = register_sql("V-NATIVE's key form");
    json_rows(pool, &format!("{}\n{keys}", &full[..at]))
}

fn v_month(pool: &PgPool) -> BTreeMap<String, serde_json::Value> {
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

fn v_conflict(pool: &PgPool) -> i64 {
    json_rows(pool, &register_sql("**V-CONFLICT**, once rollups"))[0]["count"]
        .as_i64()
        .expect("a count")
}

fn v_plan(
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

fn revert(connection: &mut PgConnection) -> Result<(), String> {
    let reverted = connection
        .revert_last_migration(MIGRATIONS)
        .map_err(|error| error.to_string())?;
    assert_eq!(
        reverted.to_string(),
        MIGRATION,
        "this stage's migration is the latest"
    );
    Ok(())
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
