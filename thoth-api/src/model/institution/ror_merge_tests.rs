//! ROR-SYNC-01B (thoth-pub/thoth#1023): the migration that merges institutions
//! sharing a ROR ID and makes the ROR ID unique, the pre-check queries of the
//! release notes, and the forward repair (spec R11.1 §5).
//!
//! Every test reverts the migration, seeds the database it runs on, and leaves
//! the migration applied on an empty database when it ends, even on a panic
//! ([`Restore`]).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::sql_types::{BigInt, Bool, Integer, Nullable, Text};
use diesel::{Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use serde_json::{json, Value};
use thoth_errors::ThothError;
use uuid::Uuid;

use crate::db::{PgPool, MIGRATIONS};
use crate::model::imprint::Imprint;
use crate::model::institution::{Institution, NewInstitution};
use crate::model::tests::db::{
    create_contribution, create_contributor, create_imprint, create_publisher, create_work,
    reset_db, setup_test_db, test_db_url, TestDbGuard,
};
use crate::model::{Crud, Ror};

const B_VERSION: &str = "20261114";
/// ROR-SYNC-01C's migration, which adds `ror_status` and `ror_successor`.
const C_VERSION: &str = "20261113";
const UP_SQL: &str = include_str!("../../../migrations/20261114_v1.10.0/up.sql");
const DOWN_SQL: &str = include_str!("../../../migrations/20261114_v1.10.0/down.sql");
const PRECHECK_SQL: &str = include_str!("../../../ops/ror-sync-01b/precheck.sql");
const REPAIR_SQL: &str = include_str!("../../../ops/ror-sync-01b/repair.sql");
const ROR_CONFLICT: &str = "An institution with this ROR ID already exists.";
const DOI_CONFLICT: &str = "An institution with this DOI already exists.";
/// The terminate pass of the kill switch, spec R11.1 §6.1 step (2)(b), verbatim.
const TERMINATE_PASS: &str = "SELECT pg_terminate_backend(l.pid) FROM pg_locks l JOIN pg_stat_activity a ON a.pid = l.pid WHERE l.locktype = 'relation' AND l.database = (SELECT oid FROM pg_database WHERE datname = current_database()) AND l.relation = 'institution'::regclass AND l.mode IN ('ShareLock', 'ShareRowExclusiveLock', 'ExclusiveLock', 'AccessExclusiveLock') AND a.usename = current_user";
const TENTH_REFERENCE_TABLE: &str = "ror_sync_01b_tenth_reference";
const OTHER_ROLE: &str = "ror_sync_01b_other_role";
const OTHER_ROLE_PASSWORD: &str = "ror-sync-01b-other";
/// The tables whose row counts §4 I2 constrains.
const LINK_TABLES: [&str; 4] = ["affiliation", "funding", "book_review", "endorsement"];

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

fn connect() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database")
}

fn exec(connection: &mut PgConnection, sql: &str) {
    connection
        .batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

fn id(connection: &mut PgConnection, sql: &str) -> Uuid {
    diesel::sql_query(sql)
        .get_result::<IdRow>(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .id
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

/// `sql` selects one bigint named `n`.
fn count(connection: &mut PgConnection, sql: &str) -> i64 {
    diesel::sql_query(sql)
        .get_result::<CountRow>(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .n
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Nullable<Text>)]
    t: Option<String>,
}

/// `sql` selects one nullable text named `t`.
fn texts(connection: &mut PgConnection, sql: &str) -> Vec<Option<String>> {
    diesel::sql_query(sql)
        .load::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .into_iter()
        .map(|row| row.t)
        .collect()
}

fn text(connection: &mut PgConnection, sql: &str) -> Option<String> {
    let mut rows = texts(connection, sql);
    assert_eq!(rows.len(), 1, "{sql} returned {} rows", rows.len());
    rows.pop().unwrap()
}

fn backend_pid(connection: &mut PgConnection) -> i32 {
    #[derive(QueryableByName)]
    struct Pid {
        #[diesel(sql_type = Integer)]
        pid: i32,
    }
    diesel::sql_query("SELECT pg_backend_pid() AS pid")
        .get_result::<Pid>(connection)
        .expect("Failed to read the backend pid")
        .pid
}

fn b_applied(connection: &mut PgConnection) -> bool {
    count(
        connection,
        &format!(
            "SELECT count(*) AS n FROM __diesel_schema_migrations WHERE version = '{B_VERSION}'"
        ),
    ) == 1
}

fn c_applied(connection: &mut PgConnection) -> bool {
    count(
        connection,
        &format!(
            "SELECT count(*) AS n FROM __diesel_schema_migrations WHERE version = '{C_VERSION}'"
        ),
    ) == 1
}

fn unique_index_exists(connection: &mut PgConnection) -> bool {
    count(
        connection,
        "SELECT count(*) AS n FROM pg_indexes WHERE schemaname = 'public' \
         AND indexname = 'institution_ror_uniq_idx'",
    ) == 1
}

/// Revert every migration newer than B, then B itself.
fn revert_through_b(connection: &mut PgConnection) {
    assert!(
        b_applied(connection),
        "B must be applied before reverting it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert a migration");
        if reverted.to_string() == B_VERSION {
            return;
        }
    }
}

/// Run every pending migration (B and anything newer).
fn apply(connection: &mut PgConnection) -> Result<(), String> {
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn random_ror() -> String {
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz0123456789";
    let bytes = Uuid::new_v4().into_bytes();
    let body: String = bytes[..6]
        .iter()
        .map(|byte| ALPHABET[*byte as usize % ALPHABET.len()] as char)
        .collect();
    format!("https://ror.org/0{body}{:02}", bytes[6] % 100)
}

fn random_doi() -> String {
    let n = u64::from_le_bytes(Uuid::new_v4().into_bytes()[..8].try_into().unwrap());
    format!("https://doi.org/10.13039/{}", 100_000_000 + n % 800_000_000)
}

/// Every row of the tables the migration may touch, as text, per table.
fn snapshot(connection: &mut PgConnection) -> BTreeMap<&'static str, String> {
    [
        "institution",
        "affiliation",
        "funding",
        "book_review",
        "endorsement",
        "institution_history",
        "work",
        "metric_rollup_work_institution_year",
    ]
    .into_iter()
    .map(|table| {
        let rows = text(
            connection,
            &format!(
                "SELECT coalesce(string_agg(t::text, E'\\n' ORDER BY t::text), '') AS t FROM {table} t"
            ),
        )
        .unwrap_or_default();
        (table, rows)
    })
    .collect()
}

fn table_count(connection: &mut PgConnection, table: &str) -> i64 {
    count(connection, &format!("SELECT count(*) AS n FROM {table}"))
}

/// `institution_id` to `to_jsonb(row)` for every institution.
fn institution_rows(connection: &mut PgConnection) -> BTreeMap<Uuid, Value> {
    texts(
        connection,
        "SELECT institution_id::text || '|' || to_jsonb(i)::text AS t FROM institution i",
    )
    .into_iter()
    .map(|row| {
        let row = row.unwrap();
        let (id, json) = row.split_once('|').unwrap();
        (id.parse().unwrap(), serde_json::from_str(json).unwrap())
    })
    .collect()
}

/// `work_id` to `updated_at_with_relations`, as text.
fn work_bumps(connection: &mut PgConnection) -> BTreeMap<Uuid, String> {
    texts(
        connection,
        "SELECT work_id::text || '|' || updated_at_with_relations::text AS t FROM work",
    )
    .into_iter()
    .map(|row| {
        let row = row.unwrap();
        let (id, at) = row.split_once('|').unwrap();
        (id.parse().unwrap(), at.to_string())
    })
    .collect()
}

#[derive(Debug)]
struct MergeRow {
    on: Uuid,
    data: Value,
}

impl MergeRow {
    fn merged_id(&self) -> Uuid {
        self.data["merged_row"]["institution_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    fn ids(&self, key: &str) -> BTreeSet<Uuid> {
        self.data[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key} is not an array in {}", self.data))
            .iter()
            .map(|id| id.as_str().unwrap().parse().unwrap())
            .collect()
    }
}

/// The `ROR-SYNC-01B` merge rows, keyed by merged `institution_id`.
fn merge_rows(connection: &mut PgConnection) -> BTreeMap<Uuid, MergeRow> {
    texts(
        connection,
        "SELECT institution_id::text || '|' || jsonb_typeof(data) || '|' || (data #>> '{}') AS t \
         FROM institution_history WHERE user_id = 'ROR-SYNC-01B'",
    )
    .into_iter()
    .map(|row| {
        let row = row.unwrap();
        let mut parts = row.splitn(3, '|');
        let on = parts.next().unwrap().parse().unwrap();
        assert_eq!(
            parts.next().unwrap(),
            "string",
            "a merge row's data is a JSON string, as every history row's"
        );
        let data: Value = serde_json::from_str(parts.next().unwrap()).unwrap();
        let merge = MergeRow { on, data };
        (merge.merged_id(), merge)
    })
    .collect()
}

/// The three queries of `precheck.sql`, without their trailing semicolons.
fn precheck_queries() -> Vec<&'static str> {
    let queries: Vec<&str> = PRECHECK_SQL
        .split(";\n")
        .map(str::trim)
        .filter(|part| part.contains("SELECT"))
        .collect();
    assert_eq!(queries.len(), 3, "precheck.sql holds exactly three queries");
    queries
}

fn query_1(connection: &mut PgConnection) -> BTreeSet<String> {
    texts(
        connection,
        &format!(
            "SELECT referencing_table::text || '.' || column_name::text || '.' || confdeltype::text AS t \
             FROM ({}) q",
            precheck_queries()[0]
        ),
    )
    .into_iter()
    .map(Option::unwrap)
    .collect()
}

#[derive(QueryableByName, Debug, Clone)]
struct Query2Row {
    #[diesel(sql_type = Text)]
    ror: String,
    #[diesel(sql_type = Text)]
    institution_id: String,
    #[diesel(sql_type = BigInt)]
    affiliations: i64,
    #[diesel(sql_type = BigInt)]
    fundings: i64,
    #[diesel(sql_type = BigInt)]
    book_reviews: i64,
    #[diesel(sql_type = BigInt)]
    endorsements: i64,
    #[diesel(sql_type = BigInt)]
    metrics_references: i64,
    #[diesel(sql_type = Nullable<Bool>)]
    survivor: Option<bool>,
    #[diesel(sql_type = Nullable<Integer>)]
    deciding_rule: Option<i32>,
    #[diesel(sql_type = Bool)]
    group_blocked: bool,
}

fn query_2(connection: &mut PgConnection) -> Vec<Query2Row> {
    let sql = format!(
        "SELECT ror, institution_id::text AS institution_id, affiliations, fundings, book_reviews, \
                endorsements, metrics_references, survivor, deciding_rule, group_blocked \
         FROM ({}) q",
        precheck_queries()[1]
    );
    diesel::sql_query(&sql)
        .load::<Query2Row>(connection)
        .unwrap_or_else(|error| panic!("query 2: {error}"))
}

fn query_3(connection: &mut PgConnection) -> BTreeSet<Uuid> {
    texts(
        connection,
        &format!(
            "SELECT work_id::text AS t FROM ({}) q",
            precheck_queries()[2]
        ),
    )
    .into_iter()
    .map(|id| id.unwrap().parse().unwrap())
    .collect()
}

// ---------------------------------------------------------------------------
// Test lifecycle
// ---------------------------------------------------------------------------

/// Leaves the test database as every other test expects it: no rows, no
/// fixture objects, every migration applied. Runs on drop, so also after a
/// panic.
struct Restore(Arc<PgPool>);

impl Drop for Restore {
    fn drop(&mut self) {
        let mut connection = connect();
        let _ = connection.batch_execute(&format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE usename = '{OTHER_ROLE}'"
        ));
        let _ = connection.batch_execute(&format!(
            "DROP TABLE IF EXISTS {TENTH_REFERENCE_TABLE}; \
             ALTER TABLE institution DROP COLUMN IF EXISTS ror_sync_01b_standin; \
             DO $$ BEGIN \
               IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{OTHER_ROLE}') THEN \
                 EXECUTE 'DROP OWNED BY {OTHER_ROLE}'; EXECUTE 'DROP ROLE {OTHER_ROLE}'; \
               END IF; \
             END $$;"
        ));
        drop_stand_in_c_columns(&mut connection);
        let _ = reset_db(&self.0);
        if let Err(error) = apply(&mut connection) {
            eprintln!("ROR-SYNC-01B tests could not re-apply the migrations: {error}");
        }
    }
}

/// Lock the test database, revert B, and arm [`Restore`]. The returned
/// connection is the one that runs the migration.
fn begin() -> (TestDbGuard, Arc<PgPool>, Restore, PgConnection) {
    let (guard, pool) = setup_test_db();
    let restore = Restore(pool.clone());
    let mut connection = connect();
    revert_through_b(&mut connection);
    assert!(!unique_index_exists(&mut connection));
    (guard, pool, restore, connection)
}

/// ROR-SYNC-01C's `ror_status` and `ror_successor`, for the repair test, on a
/// branch where C's migration is not applied. Removed by [`Restore`].
fn add_stand_in_c_columns(connection: &mut PgConnection) {
    if c_applied(connection) {
        return;
    }
    exec(
        connection,
        "CREATE TYPE ror_status AS ENUM ('active', 'inactive', 'withdrawn'); \
         ALTER TABLE institution \
             ADD COLUMN ror_status ror_status NOT NULL DEFAULT 'active', \
             ADD COLUMN ror_successor text NULL, \
             ADD CONSTRAINT institution_ror_state_check \
                 CHECK (ror IS NOT NULL OR (ror_status = 'active' AND ror_successor IS NULL));",
    );
}

fn drop_stand_in_c_columns(connection: &mut PgConnection) {
    // Never panics: it runs inside `Restore::drop`. When in doubt, it leaves
    // the columns alone.
    let applied = diesel::sql_query(format!(
        "SELECT count(*) AS n FROM __diesel_schema_migrations WHERE version = '{C_VERSION}'"
    ))
    .get_result::<CountRow>(connection)
    .map_or(true, |row| row.n == 1);
    if !applied {
        let _ = connection.batch_execute(
            "ALTER TABLE institution DROP COLUMN IF EXISTS ror_status, \
                                     DROP COLUMN IF EXISTS ror_successor; \
             DROP TYPE IF EXISTS ror_status;",
        );
    }
}

// ---------------------------------------------------------------------------
// Seeding
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct WorkRef {
    work: Uuid,
    contribution: Uuid,
}

#[derive(Clone, Debug)]
struct Link {
    table: &'static str,
    id: Uuid,
    institution: Uuid,
    /// The work the link's trigger bumps; none for a history row.
    work: Option<Uuid>,
}

struct Seed {
    pool: Arc<PgPool>,
    connection: PgConnection,
    imprint: Imprint,
    platform_and_measure: Option<(Uuid, Uuid)>,
    links: Vec<Link>,
}

impl Seed {
    fn new(pool: &Arc<PgPool>) -> Self {
        let publisher = create_publisher(pool);
        let imprint = create_imprint(pool, &publisher);
        Self {
            pool: pool.clone(),
            connection: connect(),
            imprint,
            platform_and_measure: None,
            links: Vec::new(),
        }
    }

    fn work(&mut self) -> WorkRef {
        let work = create_work(&self.pool, &self.imprint);
        let contributor = create_contributor(&self.pool);
        let contribution = create_contribution(&self.pool, &work, &contributor);
        WorkRef {
            work: work.work_id,
            contribution: contribution.contribution_id,
        }
    }

    fn institution(&mut self, ror: Option<&str>, doi: bool, created_at: &str) -> Uuid {
        let ror = ror.map_or("NULL".to_string(), |ror| format!("'{ror}'"));
        let doi = if doi {
            format!("'{}'", random_doi())
        } else {
            "NULL".to_string()
        };
        id(
            &mut self.connection,
            &format!(
                "INSERT INTO institution (institution_name, ror, institution_doi, country_code, created_at) \
                 VALUES ('Institution {}', {ror}, {doi}, 'gbr', '{created_at}') \
                 RETURNING institution_id AS id",
                Uuid::new_v4()
            ),
        )
    }

    fn record(
        &mut self,
        table: &'static str,
        id: Uuid,
        institution: Uuid,
        work: Option<Uuid>,
    ) -> Uuid {
        self.links.push(Link {
            table,
            id,
            institution,
            work,
        });
        id
    }

    fn affiliation(&mut self, work: WorkRef, institution: Uuid) -> Uuid {
        let link = id(
            &mut self.connection,
            &format!(
                "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
                 SELECT '{c}', '{institution}', coalesce(max(affiliation_ordinal), 0) + 1 \
                 FROM affiliation WHERE contribution_id = '{c}' \
                 RETURNING affiliation_id AS id",
                c = work.contribution
            ),
        );
        self.record("affiliation", link, institution, Some(work.work))
    }

    fn funding(&mut self, work: WorkRef, institution: Uuid) -> Uuid {
        let link = id(
            &mut self.connection,
            &format!(
                "INSERT INTO funding (work_id, institution_id) VALUES ('{}', '{institution}') \
                 RETURNING funding_id AS id",
                work.work
            ),
        );
        self.record("funding", link, institution, Some(work.work))
    }

    fn book_review(&mut self, work: WorkRef, institution: Uuid) -> Uuid {
        let link = id(
            &mut self.connection,
            &format!(
                "INSERT INTO book_review (work_id, reviewer_institution_id, review_ordinal) \
                 SELECT '{w}', '{institution}', coalesce(max(review_ordinal), 0) + 1 \
                 FROM book_review WHERE work_id = '{w}' \
                 RETURNING book_review_id AS id",
                w = work.work
            ),
        );
        self.record("book_review", link, institution, Some(work.work))
    }

    fn endorsement(&mut self, work: WorkRef, institution: Uuid) -> Uuid {
        let link = id(
            &mut self.connection,
            &format!(
                "INSERT INTO endorsement (work_id, author_institution_id, author_name, endorsement_ordinal) \
                 SELECT '{w}', '{institution}', 'Endorser', coalesce(max(endorsement_ordinal), 0) + 1 \
                 FROM endorsement WHERE work_id = '{w}' \
                 RETURNING endorsement_id AS id",
                w = work.work
            ),
        );
        self.record("endorsement", link, institution, Some(work.work))
    }

    fn history(&mut self, institution: Uuid) -> Uuid {
        let link = id(
            &mut self.connection,
            &format!(
                "INSERT INTO institution_history (institution_id, user_id, data) \
                 VALUES ('{institution}', 'seed-user', to_jsonb('seeded history'::text)) \
                 RETURNING institution_history_id AS id"
            ),
        );
        self.record("institution_history", link, institution, None)
    }

    /// A Metrics reference: a yearly per-institution rollup row.
    fn metrics(&mut self, work: WorkRef, institution: Uuid) {
        let (platform, measure) = match self.platform_and_measure {
            Some(pair) => pair,
            None => {
                let platform = id(
                    &mut self.connection,
                    &format!(
                        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
                         VALUES ('ror_sync_01b_{}', 'ROR-SYNC-01B platform', 'EXTERNAL', TRUE) \
                         RETURNING platform_id AS id",
                        Uuid::new_v4().simple()
                    ),
                );
                let measure = id(
                    &mut self.connection,
                    &format!(
                        "INSERT INTO metric_measure (code, display_name, category, unit, allow_negative, \
                             additive_across_time, additive_across_works, definition, enabled) \
                         VALUES ('ror_sync_01b_{}', 'ROR-SYNC-01B measure', 'USAGE', 'COUNT', FALSE, \
                             TRUE, TRUE, 'ROR-SYNC-01B test measure', TRUE) \
                         RETURNING measure_id AS id",
                        Uuid::new_v4().simple()
                    ),
                );
                self.platform_and_measure = Some((platform, measure));
                (platform, measure)
            }
        };
        exec(
            &mut self.connection,
            &format!(
                "INSERT INTO metric_rollup_work_institution_year (work_id, platform_id, measure_id, \
                     year_start, institution_id, value, requires_country_coverage, watermark) \
                 VALUES ('{}', '{platform}', '{measure}', DATE '2026-01-01', '{institution}', 1, FALSE, 1)",
                work.work
            ),
        );
    }

    /// `n` affiliations of `institution`, each on a new work.
    fn references(&mut self, institution: Uuid, n: usize) {
        for _ in 0..n {
            let work = self.work();
            self.affiliation(work, institution);
        }
    }

    fn links_of(&self, institution: Uuid) -> Vec<Link> {
        self.links
            .iter()
            .filter(|link| link.institution == institution)
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The outcome of a successful migration, checked against the seed and the
// pre-check queries
// ---------------------------------------------------------------------------

struct Before {
    institutions: BTreeMap<Uuid, Value>,
    counts: BTreeMap<&'static str, i64>,
    bumps: BTreeMap<Uuid, String>,
    query_2: Vec<Query2Row>,
    query_3: BTreeSet<Uuid>,
}

fn before(connection: &mut PgConnection) -> Before {
    let mut counts = BTreeMap::new();
    for table in LINK_TABLES
        .into_iter()
        .chain(["institution", "institution_history"])
    {
        counts.insert(table, table_count(connection, table));
    }
    Before {
        institutions: institution_rows(connection),
        counts,
        bumps: work_bumps(connection),
        query_2: query_2(connection),
        query_3: query_3(connection),
    }
}

/// What a merge is expected to do to one merged row.
struct Expected {
    survivor: Uuid,
    rule: i64,
    duplicate_affiliations: BTreeSet<Uuid>,
    duplicate_fundings: BTreeSet<Uuid>,
}

fn merged_into(survivor: Uuid, rule: i64) -> Expected {
    Expected {
        survivor,
        rule,
        duplicate_affiliations: BTreeSet::new(),
        duplicate_fundings: BTreeSet::new(),
    }
}

/// Assert everything spec §5.1 and Appendix B require of a migration that
/// merged exactly `expected` (merged row to its expectation).
fn assert_merged(
    connection: &mut PgConnection,
    seed: &Seed,
    before: &Before,
    expected: &BTreeMap<Uuid, Expected>,
) {
    // Exactly one row per ROR ID, enforced by the new index.
    assert!(unique_index_exists(connection));
    assert_eq!(
        count(
            connection,
            "SELECT count(*) AS n FROM (SELECT ror FROM institution WHERE ror IS NOT NULL \
             GROUP BY ror HAVING count(*) > 1) d"
        ),
        0
    );

    // The merged rows are gone; every other row is unchanged in every column,
    // `updated_at` included (I4, I6).
    let after = institution_rows(connection);
    for (institution, row) in &before.institutions {
        if expected.contains_key(institution) {
            assert!(!after.contains_key(institution), "{institution} was merged");
        } else {
            assert_eq!(
                after.get(institution),
                Some(row),
                "{institution} is unchanged"
            );
        }
    }
    assert_eq!(after.len(), before.institutions.len() - expected.len());

    // Row counts (I2).
    for table in LINK_TABLES {
        assert_eq!(
            table_count(connection, table),
            before.counts[table],
            "{table}"
        );
    }
    assert_eq!(
        table_count(connection, "institution_history"),
        before.counts["institution_history"] + expected.len() as i64
    );

    // Every reference of a merged row is on its survivor; every other
    // reference is where it was.
    for link in &seed.links {
        let now = id(
            connection,
            &format!(
                "SELECT {fk} AS id FROM {table} WHERE {table}_id = '{id}'",
                fk = match link.table {
                    "book_review" => "reviewer_institution_id",
                    "endorsement" => "author_institution_id",
                    _ => "institution_id",
                },
                table = link.table,
                id = link.id
            ),
        );
        let want = expected
            .get(&link.institution)
            .map_or(link.institution, |merge| merge.survivor);
        assert_eq!(now, want, "{} {}", link.table, link.id);
    }

    // One merge row per merged row, on its survivor, listing exactly what
    // moved and the group-wide duplicates.
    let merges = merge_rows(connection);
    assert_eq!(
        merges.keys().copied().collect::<BTreeSet<_>>(),
        expected.keys().copied().collect::<BTreeSet<_>>()
    );
    for (merged, want) in expected {
        let merge = &merges[merged];
        assert_eq!(merge.on, want.survivor);
        assert_eq!(merge.data["merge"], json!("ROR-SYNC-01B"));
        assert_eq!(merge.data["survivor_id"], json!(want.survivor.to_string()));
        assert_eq!(merge.data["survivor_rule"], json!(want.rule), "{merged}");
        assert_eq!(merge.data["ror"], before.institutions[merged]["ror"]);
        assert_eq!(merge.data["merged_row"], before.institutions[merged]);
        let links = seed.links_of(*merged);
        for (table, key) in [
            ("affiliation", "affiliation_ids"),
            ("funding", "funding_ids"),
            ("book_review", "book_review_ids"),
            ("endorsement", "endorsement_ids"),
            ("institution_history", "institution_history_ids"),
        ] {
            let moved: BTreeSet<Uuid> = links
                .iter()
                .filter(|link| link.table == table)
                .map(|link| link.id)
                .collect();
            assert_eq!(merge.ids(key), moved, "{key} of {merged}");
        }
        assert_eq!(
            merge.ids("duplicate_affiliation_ids"),
            want.duplicate_affiliations
        );
        assert_eq!(merge.ids("duplicate_funding_ids"), want.duplicate_fundings);
    }

    // The works whose links moved are bumped, and no other.
    let bumps = work_bumps(connection);
    let moved_works: BTreeSet<Uuid> = seed
        .links
        .iter()
        .filter(|link| expected.contains_key(&link.institution))
        .filter_map(|link| link.work)
        .collect();
    let bumped: BTreeSet<Uuid> = bumps
        .iter()
        .filter(|(work, at)| before.bumps.get(*work) != Some(*at))
        .map(|(work, _)| *work)
        .collect();
    assert_eq!(bumped, moved_works);

    // Appendix B: query 2's groups, references, survivors and rules, and
    // query 3's works, as they stood before the migration, equal what it did.
    let merged_rors: BTreeSet<String> = expected
        .keys()
        .map(|merged| {
            before.institutions[merged]["ror"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let query_2_rors: BTreeSet<String> = before.query_2.iter().map(|row| row.ror.clone()).collect();
    assert_eq!(
        query_2_rors, merged_rors,
        "query 2 lists exactly the merged groups"
    );
    for row in &before.query_2 {
        assert!(!row.group_blocked);
        let institution: Uuid = row.institution_id.parse().unwrap();
        match row.survivor {
            Some(true) => {
                for merge in merges
                    .values()
                    .filter(|merge| merge.data["ror"] == json!(row.ror))
                {
                    assert_eq!(merge.on, institution, "query 2's survivor for {}", row.ror);
                    assert_eq!(
                        merge.data["survivor_rule"],
                        json!(row.deciding_rule.unwrap()),
                        "query 2's deciding rule for {}",
                        row.ror
                    );
                }
            }
            Some(false) => {
                let merge = &merges[&institution];
                assert_eq!(merge.ids("affiliation_ids").len() as i64, row.affiliations);
                assert_eq!(merge.ids("funding_ids").len() as i64, row.fundings);
                assert_eq!(merge.ids("book_review_ids").len() as i64, row.book_reviews);
                assert_eq!(merge.ids("endorsement_ids").len() as i64, row.endorsements);
                assert_eq!(row.metrics_references, 0);
                assert_eq!(
                    merge.data["survivor_rule"],
                    json!(row.deciding_rule.unwrap())
                );
            }
            None => panic!("query 2 marked no survivor in an unblocked group"),
        }
    }
    assert_eq!(
        before.query_3, bumped,
        "query 3 lists exactly the bumped works"
    );
}

// ---------------------------------------------------------------------------
// §5.1 Populated database
// ---------------------------------------------------------------------------

struct Populated {
    seed: Seed,
    expected: BTreeMap<Uuid, Expected>,
    survivor_a: Uuid,
    merged_a: [Uuid; 2],
    survivor_b: Uuid,
    merged_b: Uuid,
}

/// Group A, three rows: A1 holds the DOI (rule 1); A2 and A3 are merged, both
/// affiliated on one contribution and both funding one work (group-wide
/// duplicates), and A2 also funds a work that A1 funds. Group B, two rows: B1
/// has a Metrics reference (rule 0); B2, older and with more references, is
/// merged with every reference type. Plus a ROR ID held once and two rows
/// without one, each with links, which nothing touches.
fn populated(pool: &Arc<PgPool>) -> Populated {
    let mut seed = Seed::new(pool);
    let ror_a = random_ror();
    let ror_b = random_ror();
    let a1 = seed.institution(Some(&ror_a), true, "2022-01-01 00:00:00+00");
    let a2 = seed.institution(Some(&ror_a), false, "2021-01-01 00:00:00+00");
    let a3 = seed.institution(Some(&ror_a), false, "2020-01-01 00:00:00+00");
    let b1 = seed.institution(Some(&ror_b), false, "2024-01-01 00:00:00+00");
    let b2 = seed.institution(Some(&ror_b), false, "2019-01-01 00:00:00+00");
    let single = seed.institution(Some(&random_ror()), true, "2020-06-01 00:00:00+00");
    let no_ror_1 = seed.institution(None, false, "2020-06-01 00:00:00+00");
    let no_ror_2 = seed.institution(None, false, "2020-06-01 00:00:00+00");

    let shared = seed.work();
    let co_funded = seed.work();
    let a1_and_a2_funded = seed.work();
    let reviewed = seed.work();
    let survivor_only = seed.work();
    let metrics_only = seed.work();
    let others = seed.work();

    seed.affiliation(survivor_only, a1);
    seed.funding(a1_and_a2_funded, a1);
    seed.history(a1);

    let a2_shared = seed.affiliation(shared, a2);
    let a3_shared = seed.affiliation(shared, a3);
    let a2_co_funding = seed.funding(co_funded, a2);
    let a3_co_funding = seed.funding(co_funded, a3);
    let a2_a1_funding = seed.funding(a1_and_a2_funded, a2);
    let a2_own = seed.work();
    seed.affiliation(a2_own, a2);
    seed.book_review(reviewed, a2);
    seed.history(a2);
    let a3_own = seed.work();
    seed.affiliation(a3_own, a3);
    seed.endorsement(reviewed, a3);
    seed.history(a3);
    seed.history(a3);

    seed.metrics(metrics_only, b1);
    for _ in 0..3 {
        let work = seed.work();
        seed.affiliation(work, b2);
    }
    let b2_work = seed.work();
    seed.funding(b2_work, b2);
    seed.book_review(b2_work, b2);
    seed.endorsement(b2_work, b2);
    seed.history(b2);

    seed.affiliation(shared, single);
    seed.funding(others, single);
    seed.affiliation(others, no_ror_1);
    seed.book_review(others, no_ror_2);

    let mut expected = BTreeMap::new();
    expected.insert(
        a2,
        Expected {
            survivor: a1,
            rule: 1,
            duplicate_affiliations: BTreeSet::from([a2_shared]),
            duplicate_fundings: BTreeSet::from([a2_co_funding, a2_a1_funding]),
        },
    );
    expected.insert(
        a3,
        Expected {
            survivor: a1,
            rule: 1,
            duplicate_affiliations: BTreeSet::from([a3_shared]),
            duplicate_fundings: BTreeSet::from([a3_co_funding]),
        },
    );
    expected.insert(b2, merged_into(b1, 0));

    Populated {
        seed,
        expected,
        survivor_a: a1,
        merged_a: [a2, a3],
        survivor_b: b1,
        merged_b: b2,
    }
}

#[test]
fn a_populated_database_keeps_one_row_per_ror_id_and_every_link_and_history_row() {
    let (_guard, pool, _restore, mut connection) = begin();
    let fixture = populated(&pool);
    let before = before(&mut connection);

    apply(&mut connection).expect("the migration applies to the populated database");

    assert!(b_applied(&mut connection));
    assert_merged(&mut connection, &fixture.seed, &before, &fixture.expected);
    // History moved, and the merged rows' `data` still names them.
    for merged in fixture.merged_a.iter().chain([&fixture.merged_b]) {
        for link in fixture.seed.links_of(*merged) {
            if link.table == "institution_history" {
                assert_eq!(
                    text(
                        &mut connection,
                        &format!(
                            "SELECT data #>> '{{}}' AS t FROM institution_history \
                             WHERE institution_history_id = '{}'",
                            link.id
                        )
                    ),
                    Some("seeded history".to_string())
                );
            }
        }
    }
    assert_eq!(
        count(
            &mut connection,
            &format!(
                "SELECT count(*) AS n FROM institution_history WHERE user_id = 'ROR-SYNC-01B' \
                 AND institution_id IN ('{}', '{}')",
                fixture.survivor_a, fixture.survivor_b
            )
        ),
        3
    );
    // Nothing was nulled.
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS n FROM book_review WHERE reviewer_institution_id IS NULL"
        ),
        0
    );
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS n FROM endorsement WHERE author_institution_id IS NULL"
        ),
        0
    );
}

// ---------------------------------------------------------------------------
// §5.2 Survivor rule
// ---------------------------------------------------------------------------

/// Seed one group per entry: each row's DOI, Metrics reference, number of
/// affiliations and `created_at`. Returns the row IDs per group.
struct RowSpec {
    doi: bool,
    metrics: bool,
    references: usize,
    created_at: &'static str,
}

const fn row(doi: bool, metrics: bool, references: usize, created_at: &'static str) -> RowSpec {
    RowSpec {
        doi,
        metrics,
        references,
        created_at,
    }
}

fn group(seed: &mut Seed, rows: &[RowSpec]) -> Vec<Uuid> {
    let ror = random_ror();
    rows.iter()
        .map(|spec| {
            let institution = seed.institution(Some(&ror), spec.doi, spec.created_at);
            seed.references(institution, spec.references);
            if spec.metrics {
                let work = seed.work();
                seed.metrics(work, institution);
            }
            institution
        })
        .collect()
}

/// Seed `groups`, run the migration, and assert that row `survivor` of each
/// group survived under `rule`, and everything [`assert_merged`] checks.
fn assert_survivors(groups: Vec<(Vec<RowSpec>, usize, i64)>) {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    let mut expected = BTreeMap::new();
    for (rows, survivor, rule) in groups {
        let ids = group(&mut seed, &rows);
        for (index, institution) in ids.iter().enumerate() {
            if index != survivor {
                expected.insert(*institution, merged_into(ids[survivor], rule));
            }
        }
    }
    let before = before(&mut connection);
    apply(&mut connection).expect("the migration applies");
    assert_merged(&mut connection, &seed, &before, &expected);
}

#[test]
fn rule_0_the_metrics_referenced_row_survives() {
    assert_survivors(vec![
        (
            vec![
                row(false, true, 0, "2024-01-01 00:00:00+00"),
                row(false, false, 3, "2020-01-01 00:00:00+00"),
            ],
            0,
            0,
        ),
        // The Metrics-referenced row also holds the DOI: rules 0 and 1 agree.
        (
            vec![
                row(false, false, 3, "2020-01-01 00:00:00+00"),
                row(true, true, 0, "2024-01-01 00:00:00+00"),
            ],
            1,
            0,
        ),
    ]);
}

#[test]
fn rule_1_the_doi_holder_survives() {
    assert_survivors(vec![(
        vec![
            row(false, false, 3, "2020-01-01 00:00:00+00"),
            row(true, false, 0, "2024-01-01 00:00:00+00"),
        ],
        1,
        1,
    )]);
}

#[test]
fn rule_2_the_row_with_most_references_survives() {
    assert_survivors(vec![(
        vec![
            row(false, false, 1, "2020-01-01 00:00:00+00"),
            row(false, false, 3, "2024-01-01 00:00:00+00"),
        ],
        1,
        2,
    )]);
}

#[test]
fn rule_3_the_oldest_row_survives() {
    assert_survivors(vec![(
        vec![
            row(false, false, 1, "2024-01-01 00:00:00+00"),
            row(false, false, 1, "2020-01-01 00:00:00+00"),
        ],
        1,
        3,
    )]);
}

#[test]
fn rule_4_the_lowest_id_survives() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    let ids = group(
        &mut seed,
        &[
            row(false, false, 1, "2022-01-01 00:00:00+00"),
            row(false, false, 1, "2022-01-01 00:00:00+00"),
        ],
    );
    let survivor = *ids.iter().min().unwrap();
    let merged = *ids.iter().max().unwrap();
    let expected = BTreeMap::from([(merged, merged_into(survivor, 4))]);
    let before = before(&mut connection);
    apply(&mut connection).expect("the migration applies");
    assert_merged(&mut connection, &seed, &before, &expected);
}

#[test]
fn a_tie_on_rule_2_is_decided_among_the_tied_rows_by_rule_3() {
    // The third row is the oldest, but has fewer references, so rule 3 only
    // chooses between the two rows tied on rule 2.
    assert_survivors(vec![(
        vec![
            row(false, false, 2, "2022-01-01 00:00:00+00"),
            row(false, false, 2, "2021-01-01 00:00:00+00"),
            row(false, false, 1, "2019-01-01 00:00:00+00"),
        ],
        1,
        3,
    )]);
}

// ---------------------------------------------------------------------------
// §5.2 Aborts
// ---------------------------------------------------------------------------

/// Run the migration, expect it to abort, and assert that nothing changed and
/// that its message names exactly query 2's blocked groups and every blocker
/// in `reasons`.
fn assert_aborts(connection: &mut PgConnection, reasons: &[&str]) -> String {
    let before = snapshot(connection);
    let query_2_before = query_2(connection);
    let error = apply(connection).expect_err("the migration aborts");
    assert!(error.contains("ROR-SYNC-01B cannot merge"), "{error}");
    assert!(error.contains("Nothing was changed"), "{error}");
    for reason in reasons {
        assert!(error.contains(reason), "{reason:?} missing from {error}");
    }
    assert!(!b_applied(connection));
    assert!(!unique_index_exists(connection));
    assert_eq!(
        snapshot(connection),
        before,
        "an abort changes nothing (I5)"
    );

    for row in &query_2_before {
        if row.group_blocked {
            assert!(error.contains(&row.ror), "{} is blocked: {error}", row.ror);
            assert!(error.contains(&row.institution_id), "{error}");
            assert_eq!(row.survivor, None);
            assert_eq!(row.deciding_rule, None);
        } else {
            assert!(
                !error.contains(&row.ror),
                "{} is not blocked: {error}",
                row.ror
            );
        }
    }
    error
}

const TWO_METRICS: &str = "two rows have Metrics references. Remedy: contact the maintainers.";
const TWO_DOIS: &str =
    "two rows hold a DOI. Remedy: clear the DOI by hand on the row that should not survive.";
const RULES_DISAGREE: &str = "the row with Metrics references (rule 0) and the row holding the DOI (rule 1) differ. Remedy: move the DOI by hand to the Metrics-referenced row (clear it on the holder first, because the DOI index is unique), or contact the maintainers.";
const TENTH_FOREIGN_KEY: &str = "The foreign keys on institution differ from the nine this migration expects (unexpected: public.ror_sync_01b_tenth_reference.institution_id.a; missing: none). Remedy: contact the maintainers before upgrading.";

fn add_tenth_foreign_key(connection: &mut PgConnection) {
    exec(
        connection,
        &format!(
            "CREATE TABLE {TENTH_REFERENCE_TABLE} ( \
                 id serial PRIMARY KEY, \
                 institution_id uuid REFERENCES institution (institution_id))"
        ),
    );
}

#[test]
fn two_metrics_referenced_rows_abort() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    group(
        &mut seed,
        &[
            row(false, true, 0, "2020-01-01 00:00:00+00"),
            row(false, true, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    group(
        &mut seed,
        &[
            row(true, false, 0, "2020-01-01 00:00:00+00"),
            row(false, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    assert_aborts(&mut connection, &[TWO_METRICS]);
}

#[test]
fn two_doi_holders_abort() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    group(
        &mut seed,
        &[
            row(true, false, 0, "2020-01-01 00:00:00+00"),
            row(true, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    assert_aborts(&mut connection, &[TWO_DOIS]);
}

#[test]
fn rules_0_and_1_disagreeing_abort() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    group(
        &mut seed,
        &[
            row(false, true, 0, "2020-01-01 00:00:00+00"),
            row(true, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    assert_aborts(&mut connection, &[RULES_DISAGREE]);
}

#[test]
fn a_tenth_foreign_key_aborts_and_query_1_lists_it() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    group(
        &mut seed,
        &[
            row(true, false, 0, "2020-01-01 00:00:00+00"),
            row(false, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    add_tenth_foreign_key(&mut connection);
    assert!(query_1(&mut connection).contains(&format!("{TENTH_REFERENCE_TABLE}.institution_id.a")));
    // The tenth key blocks every group; query 2 sees none of the groups as
    // blocked, so the message names the key and no group.
    let before = snapshot(&mut connection);
    let error = apply(&mut connection).expect_err("the migration aborts");
    assert!(error.contains(TENTH_FOREIGN_KEY), "{error}");
    assert!(error.contains("1 blocker(s)"), "{error}");
    assert!(!error.contains("ROR ID https://ror.org"), "{error}");
    assert!(!b_applied(&mut connection));
    assert!(!unique_index_exists(&mut connection));
    assert_eq!(snapshot(&mut connection), before);
}

#[test]
fn the_abort_message_names_every_blocker_across_all_groups() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    group(
        &mut seed,
        &[
            row(false, true, 0, "2020-01-01 00:00:00+00"),
            row(false, true, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    group(
        &mut seed,
        &[
            row(true, false, 0, "2020-01-01 00:00:00+00"),
            row(true, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    group(
        &mut seed,
        &[
            row(false, true, 0, "2020-01-01 00:00:00+00"),
            row(true, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    // An unblocked group is not merged either.
    group(
        &mut seed,
        &[
            row(true, false, 0, "2020-01-01 00:00:00+00"),
            row(false, false, 1, "2021-01-01 00:00:00+00"),
        ],
    );
    add_tenth_foreign_key(&mut connection);
    let error = assert_aborts(
        &mut connection,
        &[TWO_METRICS, TWO_DOIS, RULES_DISAGREE, TENTH_FOREIGN_KEY],
    );
    assert!(error.contains("4 blocker(s)"), "{error}");
}

// ---------------------------------------------------------------------------
// §5.4, §5.5 No duplicates; the foreign-key set
// ---------------------------------------------------------------------------

#[test]
fn without_duplicates_the_migration_merges_nothing_and_creates_the_index() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    let a = seed.institution(Some(&random_ror()), true, "2020-01-01 00:00:00+00");
    seed.institution(Some(&random_ror()), false, "2020-01-01 00:00:00+00");
    seed.institution(None, false, "2020-01-01 00:00:00+00");
    seed.institution(None, false, "2020-01-01 00:00:00+00");
    seed.references(a, 2);
    let before = snapshot(&mut connection);
    assert!(query_2(&mut connection).is_empty());

    apply(&mut connection).expect("the migration applies");

    assert!(unique_index_exists(&mut connection));
    assert_eq!(snapshot(&mut connection), before);
    assert!(merge_rows(&mut connection).is_empty());
}

#[test]
fn the_foreign_keys_on_institution_are_exactly_the_nine_of_the_spec() {
    let (_guard, _pool) = setup_test_db();
    let mut connection = connect();
    // (referencing table, column, delete action) as of ROR-SYNC-01B. A later
    // stage that adds a foreign key on `institution` updates this list; the
    // migration, which has already run, is unaffected.
    let expected: BTreeSet<String> = [
        "affiliation.institution_id.c",
        "book_review.reviewer_institution_id.n",
        "endorsement.author_institution_id.n",
        "funding.institution_id.c",
        "institution_history.institution_id.c",
        "metric_record.institution_id.a",
        "metric_rollup_work_day.institution_id.a",
        "metric_rollup_work_institution_month.institution_id.a",
        "metric_rollup_work_institution_year.institution_id.a",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(query_1(&mut connection), expected);
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS n FROM pg_constraint WHERE contype = 'f' \
             AND confrelid = 'institution'::regclass"
        ),
        9
    );
}

// ---------------------------------------------------------------------------
// §5.6 Index and errors
// ---------------------------------------------------------------------------

fn new_institution(ror: Option<&str>, doi: Option<String>) -> NewInstitution {
    NewInstitution {
        institution_name: format!("Institution {}", Uuid::new_v4()),
        institution_doi: doi.map(|doi| doi.parse().unwrap()),
        ror: ror.map(|ror| ror.parse::<Ror>().unwrap()),
        country_code: None,
    }
}

#[test]
fn a_second_institution_with_a_ror_id_or_doi_gets_the_pinned_message() {
    let (_guard, pool) = setup_test_db();
    let ror = random_ror();
    let doi = random_doi();
    Institution::create(&pool, &new_institution(Some(&ror), Some(doi.clone())))
        .expect("the first holder is created");

    let by_ror = Institution::create(&pool, &new_institution(Some(&ror), None))
        .expect_err("a second ROR holder is refused");
    assert_eq!(by_ror.to_string(), ROR_CONFLICT);
    assert!(matches!(by_ror, ThothError::DatabaseConstraintError(_)));

    let by_doi = Institution::create(&pool, &new_institution(Some(&random_ror()), Some(doi)))
        .expect_err("a second DOI holder is refused");
    assert_eq!(by_doi.to_string(), DOI_CONFLICT);

    // Any number of rows may have no ROR ID.
    Institution::create(&pool, &new_institution(None, None)).unwrap();
    Institution::create(&pool, &new_institution(None, None)).unwrap();
}

#[test]
fn of_two_concurrent_creates_with_one_ror_id_one_succeeds_and_the_other_gets_the_message() {
    let (_guard, _pool) = setup_test_db();
    let ror = random_ror();
    let insert = format!(
        "INSERT INTO institution (institution_name, ror) VALUES ('Concurrent {}', '{ror}')",
        Uuid::new_v4()
    );
    let mut first = connect();
    exec(&mut first, "BEGIN");
    exec(&mut first, &insert);

    let second = Probe::spawn(vec![insert.clone()]);
    let mut observer = connect();
    second.wait_blocked(&mut observer);
    exec(&mut first, "COMMIT");

    let error = second.join().expect_err("the second create fails");
    assert_eq!(ThothError::from(error).to_string(), ROR_CONFLICT);
    assert_eq!(
        count(
            &mut observer,
            &format!("SELECT count(*) AS n FROM institution WHERE ror = '{ror}'")
        ),
        1
    );
}

// ---------------------------------------------------------------------------
// §5.3 Concurrency
// ---------------------------------------------------------------------------

/// A statement run on its own connection, in its own thread.
struct Probe {
    pid: i32,
    handle: JoinHandle<Result<(), diesel::result::Error>>,
}

impl Probe {
    fn spawn(statements: Vec<String>) -> Self {
        Self::spawn_with(test_db_url(), statements)
    }

    fn spawn_with(url: String, statements: Vec<String>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut connection = PgConnection::establish(&url).expect("probe connects");
            connection.batch_execute("SET statement_timeout = '60s'")?;
            sender.send(backend_pid(&mut connection)).unwrap();
            for statement in statements {
                connection.batch_execute(&statement)?;
            }
            Ok(())
        });
        let pid = receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("the probe connected");
        Self { pid, handle }
    }

    /// Wait until the probe's backend waits on a lock.
    fn wait_blocked(&self, observer: &mut PgConnection) {
        wait_for_lock_wait(observer, self.pid, || {
            assert!(
                !self.handle.is_finished(),
                "backend {} finished instead of waiting",
                self.pid
            );
        });
    }

    fn join(self) -> Result<(), diesel::result::Error> {
        self.handle.join().expect("the probe thread panicked")
    }
}

/// Wait until backend `pid` waits on a lock, calling `check` between polls.
fn wait_for_lock_wait(observer: &mut PgConnection, pid: i32, check: impl Fn()) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let waiting = count(
            observer,
            &format!(
                "SELECT count(*) AS n FROM pg_stat_activity \
                 WHERE pid = {pid} AND wait_event_type = 'Lock'"
            ),
        );
        if waiting == 1 {
            return;
        }
        check();
        assert!(
            Instant::now() < deadline,
            "backend {pid} never waited on a lock"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

/// Run `sql` on a new connection and assert that it returns within a second,
/// without waiting on any lock.
fn at_once(sql: &str) {
    let mut connection = connect();
    exec(&mut connection, "SET statement_timeout = '5s'");
    let started = Instant::now();
    connection
        .batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql} did not return at once: {error}"));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{sql} took {:?}",
        started.elapsed()
    );
}

/// Open a connection, start a transaction and run the whole of `up.sql` in
/// it, leaving it open: the migration holding its locks before commit.
fn hold_migration() -> PgConnection {
    let mut hold = connect();
    exec(&mut hold, "BEGIN");
    exec(&mut hold, UP_SQL);
    hold
}

/// Commit a held migration as the runner would, recording its version.
fn commit_migration(mut hold: PgConnection) {
    exec(
        &mut hold,
        &format!(
            "INSERT INTO __diesel_schema_migrations (version) VALUES ('{B_VERSION}'); COMMIT;"
        ),
    );
}

fn is_foreign_key_violation(result: &Result<(), diesel::result::Error>) -> bool {
    matches!(
        result,
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _
        ))
    )
}

#[test]
fn while_the_migration_holds_its_locks_reads_go_on_and_references_to_group_rows_wait() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    let ror = random_ror();
    let survivor = seed.institution(Some(&ror), true, "2020-01-01 00:00:00+00");
    let merged = seed.institution(Some(&ror), false, "2021-01-01 00:00:00+00");
    let outside = seed.institution(Some(&random_ror()), false, "2020-01-01 00:00:00+00");
    let bumped = seed.work();
    let merged_affiliation = seed.affiliation(bumped, merged);
    let quiet = [seed.work(), seed.work(), seed.work()];
    let outside_review = seed.book_review(quiet[0], outside);
    let counts_before: BTreeMap<&str, i64> = LINK_TABLES
        .into_iter()
        .map(|table| (table, table_count(&mut connection, table)))
        .collect();

    let hold = hold_migration();

    // A read of `institution` returns at once.
    at_once("SELECT count(*) FROM institution");
    // An affiliation on an institution outside the groups, on a work the
    // merge does not bump, returns at once.
    at_once(&format!(
        "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
         VALUES ('{}', '{outside}', 1)",
        quiet[1].contribution
    ));

    let waits = [
        // A new reference to a merged row.
        Probe::spawn(vec![format!(
            "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             VALUES ('{}', '{merged}', 2)",
            quiet[1].contribution
        )]),
        // A book review outside the groups repointed to a merged row.
        Probe::spawn(vec![format!(
            "UPDATE book_review SET reviewer_institution_id = '{merged}' \
             WHERE book_review_id = '{outside_review}'"
        )]),
        // The API's full-patch update of a repointed affiliation: only the
        // ordinal changes, and the patch still names the merged row.
        Probe::spawn(vec![format!(
            "UPDATE affiliation SET contribution_id = '{}', institution_id = '{merged}', \
                 affiliation_ordinal = 9, position = NULL \
             WHERE affiliation_id = '{merged_affiliation}'",
            bumped.contribution
        )]),
        // A new reference to the survivor.
        Probe::spawn(vec![format!(
            "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             VALUES ('{}', '{survivor}', 1)",
            quiet[2].contribution
        )]),
        // Creates, with a new ROR ID and with the group's.
        Probe::spawn(vec![format!(
            "INSERT INTO institution (institution_name, ror) VALUES ('New', '{}')",
            random_ror()
        )]),
        Probe::spawn(vec![format!(
            "INSERT INTO institution (institution_name, ror) VALUES ('Twin', '{ror}')"
        )]),
        // An affiliation on an institution outside the groups, on a work the
        // merge bumps.
        Probe::spawn(vec![format!(
            "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             VALUES ('{}', '{outside}', 2)",
            bumped.contribution
        )]),
        // Metrics ingestion's lock on a group row.
        Probe::spawn(vec![
            "BEGIN".to_string(),
            format!("SELECT 1 FROM institution WHERE institution_id = '{survivor}' FOR SHARE"),
            "COMMIT".to_string(),
        ]),
    ];
    let mut observer = connect();
    for probe in &waits {
        probe.wait_blocked(&mut observer);
    }
    thread::sleep(Duration::from_millis(200));
    assert!(waits.iter().all(|probe| !probe.handle.is_finished()));

    commit_migration(hold);

    let [to_merged, review_to_merged, full_patch, to_survivor, create_new, create_twin, outside_on_bumped, for_share] =
        waits;
    assert!(is_foreign_key_violation(&to_merged.join()));
    assert!(is_foreign_key_violation(&review_to_merged.join()));
    assert!(is_foreign_key_violation(&full_patch.join()));
    to_survivor
        .join()
        .expect("a reference to the survivor succeeds");
    create_new
        .join()
        .expect("a create with a new ROR ID succeeds");
    let twin = create_twin
        .join()
        .expect_err("a create with the group's ROR ID fails");
    assert_eq!(ThothError::from(twin).to_string(), ROR_CONFLICT);
    outside_on_bumped
        .join()
        .expect("an affiliation on a bumped work succeeds after the commit");
    for_share
        .join()
        .expect("FOR SHARE on the survivor succeeds after the commit");

    // The full-patch update changed nothing.
    assert_eq!(
        text(
            &mut connection,
            &format!(
                "SELECT institution_id::text || ' ' || affiliation_ordinal AS t FROM affiliation \
                 WHERE affiliation_id = '{merged_affiliation}'"
            )
        ),
        Some(format!("{survivor} 1"))
    );
    // Nothing was cascade-deleted or nulled: three affiliations were added.
    for table in LINK_TABLES {
        let added = if table == "affiliation" { 3 } else { 0 };
        assert_eq!(
            table_count(&mut connection, table),
            counts_before[table] + added,
            "{table}"
        );
    }
    assert_eq!(
        text(
            &mut connection,
            &format!(
                "SELECT reviewer_institution_id::text AS t FROM book_review \
                 WHERE book_review_id = '{outside_review}'"
            )
        ),
        Some(outside.to_string())
    );
}

#[test]
fn a_second_copy_waits_then_merges_nothing_and_fails_at_the_index() {
    let (_guard, pool, _restore, mut connection) = begin();
    let fixture = populated(&pool);
    let before = before(&mut connection);

    let hold = hold_migration();
    let second = Probe::spawn(vec!["BEGIN".to_string(), UP_SQL.to_string()]);
    let mut observer = connect();
    second.wait_blocked(&mut observer);
    commit_migration(hold);

    let error = second.join().expect_err("the second copy fails");
    assert!(
        error.to_string().contains("institution_ror_uniq_idx")
            && error.to_string().contains("already exists"),
        "{error}"
    );
    // The database is as the first copy left it.
    assert_merged(&mut connection, &fixture.seed, &before, &fixture.expected);
}

/// The guard session of the kill switch, spec §6.1 step (2)(a), in a thread
/// that holds its transaction until released.
struct GuardSession {
    pid: i32,
    granted: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    handle: JoinHandle<Result<(), diesel::result::Error>>,
}

impl GuardSession {
    fn start() -> Self {
        let (pid_sender, pid_receiver) = mpsc::channel();
        let (granted_sender, granted) = mpsc::channel();
        let (release, release_receiver) = mpsc::channel::<()>();
        let handle = thread::spawn(move || {
            let mut connection = connect();
            connection.batch_execute("SET idle_in_transaction_session_timeout = 0")?;
            // `transaction_timeout` exists from PostgreSQL 17, which both
            // environments run.
            if count(
                &mut connection,
                "SELECT current_setting('server_version_num')::bigint AS n",
            ) >= 170_000
            {
                connection.batch_execute("SET transaction_timeout = 0")?;
            }
            pid_sender.send(backend_pid(&mut connection)).unwrap();
            connection.batch_execute(
                "BEGIN; SET LOCAL lock_timeout = 0; SET LOCAL statement_timeout = 0; \
                 LOCK TABLE institution IN ROW EXCLUSIVE MODE;",
            )?;
            granted_sender.send(()).unwrap();
            let _ = release_receiver.recv();
            connection.batch_execute("ROLLBACK")
        });
        let pid = pid_receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("the guard connected");
        Self {
            pid,
            granted,
            release,
            handle,
        }
    }

    fn wait_granted(&self) {
        self.granted
            .recv_timeout(Duration::from_secs(20))
            .expect("the guard's LOCK TABLE is granted");
    }

    /// The guard check of §6.1 step (3).
    fn holds(&self, observer: &mut PgConnection) -> Option<bool> {
        texts(
            observer,
            &format!(
                "SELECT granted::text AS t FROM pg_locks WHERE pid = {} AND locktype = 'relation' \
                 AND relation = 'institution'::regclass AND mode = 'RowExclusiveLock'",
                self.pid
            ),
        )
        .pop()
        .map(|granted| granted.as_deref() == Some("true"))
    }

    fn release(self) {
        self.release.send(()).unwrap();
        self.handle
            .join()
            .expect("the guard thread panicked")
            .expect("the guard rolls back");
    }
}

/// One terminate pass, run in autocommit mode; returns how many backends it
/// signalled.
fn terminate_pass(pass: &mut PgConnection) -> usize {
    #[derive(QueryableByName)]
    struct Terminated {
        #[diesel(sql_type = Bool)]
        #[allow(dead_code)]
        pg_terminate_backend: bool,
    }
    diesel::sql_query(TERMINATE_PASS)
        .load::<Terminated>(pass)
        .expect("the terminate pass raises no error")
        .len()
}

fn alive(observer: &mut PgConnection, pid: i32) -> bool {
    count(
        observer,
        &format!("SELECT count(*) AS n FROM pg_stat_activity WHERE pid = {pid}"),
    ) == 1
}

/// `test_db_url()` with the second role's credentials.
fn other_role_url() -> String {
    let url = test_db_url();
    let (scheme, rest) = url.split_once("://").unwrap();
    let host = rest.rsplit_once('@').map_or(rest, |(_, host)| host);
    format!("{scheme}://{OTHER_ROLE}:{OTHER_ROLE_PASSWORD}@{host}")
}

#[test]
fn the_kill_switch_ends_every_copy_and_nothing_else() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut seed = Seed::new(&pool);
    let ror = random_ror();
    let survivor = seed.institution(Some(&ror), true, "2020-01-01 00:00:00+00");
    let merged = seed.institution(Some(&ror), false, "2021-01-01 00:00:00+00");
    seed.references(merged, 1);
    let outside = seed.institution(Some(&random_ror()), false, "2020-01-01 00:00:00+00");
    let to_delete = seed.institution(None, false, "2020-01-01 00:00:00+00");
    let waited_update = seed.institution(None, false, "2020-01-01 00:00:00+00");
    let waited_delete = seed.institution(None, false, "2020-01-01 00:00:00+00");
    let work = seed.work();
    let before = snapshot(&mut connection);
    let mut pass = connect();
    let mut observer = connect();

    // A copy held inside its transaction: it holds the migration's table lock
    // and waits on a row lock the test holds on a group row.
    let mut row_holder = connect();
    exec(
        &mut row_holder,
        &format!("BEGIN; SELECT 1 FROM institution WHERE institution_id = '{merged}' FOR UPDATE;"),
    );
    let first_copy = Probe::spawn(vec!["BEGIN".to_string(), UP_SQL.to_string()]);
    first_copy.wait_blocked(&mut observer);

    // The guard waits behind it; one pass ends the copy and the guard is
    // granted.
    let guard = GuardSession::start();
    wait_for_lock_wait(&mut observer, guard.pid, || {});
    assert_eq!(guard.holds(&mut observer), Some(false));
    assert_eq!(terminate_pass(&mut pass), 1);
    guard.wait_granted();
    assert_eq!(guard.holds(&mut observer), Some(true));
    assert!(first_copy.join().is_err(), "the pass ended the first copy");
    exec(&mut row_holder, "ROLLBACK");

    // A copy started after that waits on the table lock and is ended by the
    // next pass.
    let second_copy = Probe::spawn(vec!["BEGIN".to_string(), UP_SQL.to_string()]);
    second_copy.wait_blocked(&mut observer);
    assert_eq!(terminate_pass(&mut pass), 1);
    assert!(
        second_copy.join().is_err(),
        "the pass ended the second copy"
    );
    assert!(!b_applied(&mut connection));
    assert_eq!(
        snapshot(&mut connection),
        before,
        "the database is unchanged"
    );

    // With the guard held and no copy waiting, ordinary statements return at
    // once and the pass matches nothing.
    at_once("SELECT count(*) FROM institution");
    at_once("INSERT INTO institution (institution_name) VALUES ('During the guard')");
    at_once(&format!(
        "UPDATE institution SET institution_name = 'Renamed' WHERE institution_id = '{outside}'"
    ));
    at_once(&format!(
        "DELETE FROM institution WHERE institution_id = '{to_delete}'"
    ));
    at_once(&format!(
        "INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
         VALUES ('{}', '{outside}', 1)",
        work.contribution
    ));
    at_once(&format!(
        "BEGIN; SELECT 1 FROM institution WHERE institution_id = '{survivor}' FOR SHARE; COMMIT;"
    ));
    assert_eq!(terminate_pass(&mut pass), 0);

    // A stand-in for another migration's ALTER TABLE waits behind the guard
    // and is ended by the next pass.
    let alter = Probe::spawn(vec![
        "ALTER TABLE institution ADD COLUMN ror_sync_01b_standin integer".to_string(),
    ]);
    alter.wait_blocked(&mut observer);
    assert_eq!(terminate_pass(&mut pass), 1);
    assert!(alter.join().is_err());
    assert_eq!(
        count(
            &mut observer,
            "SELECT count(*) AS n FROM information_schema.columns \
             WHERE table_name = 'institution' AND column_name = 'ror_sync_01b_standin'"
        ),
        0
    );

    // An update and a delete each waiting on another session's uncommitted
    // update of the same row are not matched, and complete once it ends.
    let mut first_writer = connect();
    exec(
        &mut first_writer,
        &format!(
            "BEGIN; UPDATE institution SET institution_name = 'First' \
             WHERE institution_id = '{waited_update}';"
        ),
    );
    let mut second_writer = connect();
    exec(
        &mut second_writer,
        &format!(
            "BEGIN; UPDATE institution SET institution_name = 'First' \
             WHERE institution_id = '{waited_delete}';"
        ),
    );
    let waiting_update = Probe::spawn(vec![format!(
        "UPDATE institution SET institution_name = 'Second' WHERE institution_id = '{waited_update}'"
    )]);
    let waiting_delete = Probe::spawn(vec![format!(
        "DELETE FROM institution WHERE institution_id = '{waited_delete}'"
    )]);
    waiting_update.wait_blocked(&mut observer);
    waiting_delete.wait_blocked(&mut observer);
    assert_eq!(terminate_pass(&mut pass), 0);
    exec(&mut first_writer, "COMMIT");
    exec(&mut second_writer, "COMMIT");
    waiting_update.join().expect("the waiting update completes");
    waiting_delete.join().expect("the waiting delete completes");

    // A backend of a second role that awaits SHARE on `institution` behind
    // the guard is not matched, and the pass raises no error.
    exec(
        &mut connection,
        &format!(
            "CREATE ROLE {OTHER_ROLE} LOGIN PASSWORD '{OTHER_ROLE_PASSWORD}'; \
             GRANT USAGE ON SCHEMA public TO {OTHER_ROLE}; \
             GRANT SELECT, UPDATE ON institution TO {OTHER_ROLE};"
        ),
    );
    let other_waiting = Probe::spawn_with(
        other_role_url(),
        vec![
            "BEGIN".to_string(),
            "LOCK TABLE institution IN SHARE MODE".to_string(),
            "ROLLBACK".to_string(),
        ],
    );
    other_waiting.wait_blocked(&mut observer);
    assert_eq!(terminate_pass(&mut pass), 0);
    assert!(alive(&mut observer, other_waiting.pid));

    // Releasing the guard (§6.1 step (3)) lets it go on; the guard check then
    // finds no lock for the guard's pid.
    let guard_pid = guard.pid;
    guard.release();
    other_waiting
        .join()
        .expect("the second role's lock is granted once the guard is released");
    assert_eq!(
        texts(
            &mut observer,
            &format!(
                "SELECT granted::text AS t FROM pg_locks WHERE pid = {guard_pid} \
                 AND locktype = 'relation' AND relation = 'institution'::regclass"
            )
        ),
        Vec::<Option<String>>::new()
    );

    // A backend of a second role that holds SHARE on `institution` is not
    // matched either.
    let mut other_holder =
        PgConnection::establish(&other_role_url()).expect("the second role connects");
    exec(
        &mut other_holder,
        "BEGIN; LOCK TABLE institution IN SHARE MODE;",
    );
    let other_pid = backend_pid(&mut other_holder);
    assert_eq!(terminate_pass(&mut pass), 0);
    assert!(alive(&mut observer, other_pid));
    exec(&mut other_holder, "ROLLBACK");
    drop(other_holder);

    assert!(!b_applied(&mut connection));
}

// ---------------------------------------------------------------------------
// §5.7 Forward repair
// ---------------------------------------------------------------------------

#[derive(QueryableByName, Debug)]
struct ReportRow {
    #[diesel(sql_type = Text)]
    tbl: String,
    #[diesel(sql_type = Text)]
    row_id: String,
    #[diesel(sql_type = Text)]
    outcome: String,
    #[diesel(sql_type = Nullable<Text>)]
    now_references: Option<String>,
}

/// Run `repair.sql` for one merge row in one transaction, as
/// `psql --single-transaction` does, and return its report.
fn repair(merge_row: Uuid) -> Result<Vec<ReportRow>, String> {
    let mut connection = connect();
    connection
        .transaction(|connection| {
            connection.batch_execute(&format!(
                "SET LOCAL ror_sync_01b.merge_row = '{merge_row}'"
            ))?;
            connection.batch_execute(REPAIR_SQL)?;
            diesel::sql_query(
                "SELECT tbl, row_id::text AS row_id, outcome, now_references::text AS now_references \
                 FROM ror_sync_01b_repair_report ORDER BY tbl, row_id",
            )
            .load::<ReportRow>(connection)
        })
        .map_err(|error: diesel::result::Error| error.to_string())
}

fn merge_row_id(connection: &mut PgConnection, merged: Uuid) -> Uuid {
    id(
        connection,
        &format!(
            "SELECT institution_history_id AS id FROM institution_history \
             WHERE user_id = 'ROR-SYNC-01B' \
               AND (data #>> '{{}}')::jsonb #>> '{{merged_row,institution_id}}' = '{merged}'"
        ),
    )
}

#[test]
fn the_repair_restores_a_merged_row_and_its_links_and_reports_what_it_left() {
    let (_guard, pool, _restore, mut connection) = begin();
    let mut fixture = populated(&pool);
    let institutions_before = institution_rows(&mut connection);
    apply(&mut connection).expect("the migration applies");

    let merged = fixture.merged_a[0];
    let survivor = fixture.survivor_a;
    let links = fixture.seed.links_of(merged);
    let affiliations: Vec<Uuid> = links
        .iter()
        .filter(|link| link.table == "affiliation")
        .map(|link| link.id)
        .collect();
    assert_eq!(affiliations.len(), 2);
    // After the merge, one listed affiliation is deleted and one is moved to
    // a third institution.
    let deleted = affiliations[0];
    let moved_away = affiliations[1];
    let third = fixture
        .seed
        .institution(Some(&random_ror()), false, "2020-01-01 00:00:00+00");
    exec(
        &mut connection,
        &format!(
            "DELETE FROM affiliation WHERE affiliation_id = '{deleted}'; \
             UPDATE affiliation SET institution_id = '{third}' WHERE affiliation_id = '{moved_away}';"
        ),
    );
    let merge_row = merge_row_id(&mut connection, merged);

    let report = repair(merge_row).expect("the repair runs");

    // The merged row is back, as it was, with no ROR ID.
    let restored = institution_rows(&mut connection)
        .remove(&merged)
        .expect("the merged row is restored");
    let mut want = institutions_before[&merged].clone();
    want["ror"] = Value::Null;
    assert_eq!(restored, want);

    // Every listed link that still referenced the survivor is back; the two
    // others are reported and left alone.
    for link in &links {
        let (fk, pk) = match link.table {
            "book_review" => ("reviewer_institution_id", "book_review_id"),
            "endorsement" => ("author_institution_id", "endorsement_id"),
            "affiliation" => ("institution_id", "affiliation_id"),
            "funding" => ("institution_id", "funding_id"),
            _ => ("institution_id", "institution_history_id"),
        };
        let now = texts(
            &mut connection,
            &format!(
                "SELECT {fk}::text AS t FROM {} WHERE {pk} = '{}'",
                link.table, link.id
            ),
        );
        let entry = report
            .iter()
            .find(|row| row.row_id == link.id.to_string())
            .unwrap_or_else(|| panic!("{} {} is reported", link.table, link.id));
        assert_eq!(entry.tbl, link.table);
        if link.id == deleted {
            assert!(now.is_empty());
            assert_eq!(entry.outcome, "missing");
            assert_eq!(entry.now_references, None);
        } else if link.id == moved_away {
            assert_eq!(now, vec![Some(third.to_string())]);
            assert_eq!(entry.outcome, "elsewhere");
            assert_eq!(entry.now_references, Some(third.to_string()));
        } else {
            assert_eq!(
                now,
                vec![Some(merged.to_string())],
                "{} {}",
                link.table,
                link.id
            );
            assert_eq!(entry.outcome, "moved");
        }
    }
    assert_eq!(report.len(), links.len());
    // The survivor keeps its own links.
    for link in fixture.seed.links_of(survivor) {
        if link.table == "affiliation" {
            assert_eq!(
                text(
                    &mut connection,
                    &format!(
                        "SELECT institution_id::text AS t FROM affiliation WHERE affiliation_id = '{}'",
                        link.id
                    )
                ),
                Some(survivor.to_string())
            );
        }
    }

    // One repair history row on the restored row, naming the merge row.
    let repair_rows = texts(
        &mut connection,
        &format!(
            "SELECT data #>> '{{}}' AS t FROM institution_history \
             WHERE user_id = 'ROR-SYNC-01B-REPAIR' AND institution_id = '{merged}'"
        ),
    );
    assert_eq!(repair_rows.len(), 1);
    let data: Value = serde_json::from_str(repair_rows[0].as_deref().unwrap()).unwrap();
    assert_eq!(data["merge_row"], json!(merge_row.to_string()));
    assert_eq!(data["restored_institution_id"], json!(merged.to_string()));

    // A second run refuses: the merged ID exists.
    let refused = repair(merge_row).expect_err("the repair refuses");
    assert!(refused.contains("already exists"), "{refused}");
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS n FROM institution_history WHERE user_id = 'ROR-SYNC-01B-REPAIR'"
        ),
        1
    );
}

#[test]
fn the_repair_restores_the_ror_state_columns_to_their_defaults() {
    let (_guard, pool, _restore, mut connection) = begin();
    add_stand_in_c_columns(&mut connection);
    let mut seed = Seed::new(&pool);
    let ror = random_ror();
    let survivor = seed.institution(Some(&ror), true, "2020-01-01 00:00:00+00");
    let merged = seed.institution(Some(&ror), false, "2021-01-01 00:00:00+00");
    seed.references(merged, 1);
    exec(
        &mut connection,
        &format!(
            "UPDATE institution SET ror_status = 'inactive' WHERE institution_id = '{merged}'"
        ),
    );
    apply(&mut connection).expect("the migration applies");
    let merge_row = merge_row_id(&mut connection, merged);
    let merges = merge_rows(&mut connection);
    assert_eq!(
        merges[&merged].data["merged_row"]["ror_status"],
        json!("inactive")
    );
    assert_eq!(merges[&merged].on, survivor);

    repair(merge_row).expect("the repair runs");

    assert_eq!(
        text(
            &mut connection,
            &format!(
                "SELECT ror_status::text || ' ' || coalesce(ror_successor, 'null') || ' ' \
                     || coalesce(ror, 'null') AS t \
                 FROM institution WHERE institution_id = '{merged}'"
            )
        ),
        Some("active null null".to_string())
    );
}

// ---------------------------------------------------------------------------
// §5.8 Down and up; §5.9 the timeout mechanism
// ---------------------------------------------------------------------------

#[test]
fn down_then_up_and_the_manual_rollback_pair_then_the_runner_restore_the_schema() {
    let (_guard, pool, _restore, mut connection) = begin();
    let fixture = populated(&pool);
    apply(&mut connection).expect("the migration applies");
    let merged = snapshot(&mut connection);

    // down.sql, then up.sql, through the runner.
    revert_through_b(&mut connection);
    assert!(!unique_index_exists(&mut connection));
    apply(&mut connection).expect("up.sql runs again on the merged database");
    assert!(unique_index_exists(&mut connection));
    assert_eq!(snapshot(&mut connection), merged, "nothing more is merged");

    // The manual rollback pair of S10, in one transaction, then the runner.
    connection
        .transaction(|connection| {
            connection.batch_execute(DOWN_SQL)?;
            connection.batch_execute(&format!(
                "DELETE FROM __diesel_schema_migrations WHERE version = '{B_VERSION}'"
            ))
        })
        .expect("the rollback pair runs");
    assert!(!unique_index_exists(&mut connection));
    assert!(!b_applied(&mut connection));
    apply(&mut connection).expect("the runner applies B again");
    assert!(unique_index_exists(&mut connection));
    assert!(b_applied(&mut connection));
    assert_eq!(snapshot(&mut connection), merged);
    assert_eq!(merge_rows(&mut connection).len(), fixture.expected.len());
}

#[test]
fn a_set_local_statement_timeout_bounds_a_later_statement_of_the_same_batch() {
    let (_guard, _pool) = setup_test_db();
    let mut connection = connect();
    let error = connection
        .transaction(|connection| {
            connection.batch_execute("SET LOCAL statement_timeout = '50ms'; SELECT pg_sleep(1);")
        })
        .expect_err("the sleep is cancelled");
    assert!(error.to_string().contains("statement timeout"), "{error}");
}
