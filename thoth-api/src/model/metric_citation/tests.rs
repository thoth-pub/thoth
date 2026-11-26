//! Database tests of the citation storage migration (`CITATIONS-01` C1): the
//! tables, the enum labels, every CHECK and unique rule of the spec, the
//! non-cascading foreign keys, the row models and the `down.sql` refusal.

use diesel::connection::SimpleConnection;
use diesel::result::Error as DieselError;
use diesel::{sql_query, Connection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use super::{
    MetricCitationLink, MetricCitationLinkStatus, MetricCitationMatchStatus,
    MetricCitationWorkState,
};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{
    enum_labels, insert_platform_row, scalar_i64, setup_registry_db,
};
use crate::model::metric_source_account::tests::insert_account_row;
use crate::model::tests::db::{create_imprint, create_publisher, create_work};
use crate::schema::{metric_citation_link, metric_citation_work_state};

const DOWN_SQL: &str = include_str!("../../../migrations/20261127_v1.15.0/down.sql");

struct Fixture {
    work_id: Uuid,
    source_id: Uuid,
    account_id: Uuid,
}

fn exec(pool: &PgPool, sql: &str) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(sql).execute(&mut connection)
}

fn fixture(pool: &PgPool) -> Fixture {
    let publisher = create_publisher(pool);
    let imprint = create_imprint(pool, &publisher);
    let work = create_work(pool, &imprint);
    let source_id = Uuid::new_v4();
    exec(
        pool,
        &format!(
            "INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) \
             VALUES ('{source_id}', 'openalex-citations', 'DRIVER', 'openalex-citations', TRUE)"
        ),
    )
    .expect("source");
    let platform_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, "openalex");
    let account_id = Uuid::new_v4();
    insert_account_row(pool, account_id, source_id, platform_id, "obp");
    Fixture {
        work_id: work.work_id,
        source_id,
        account_id,
    }
}

fn link_sql(f: &Fixture, id: &str, extra_cols: &str, extra_vals: &str) -> String {
    format!(
        "INSERT INTO metric_citation_link \
         (cited_work_id, source_id, citing_external_id, citing_sort_date{extra_cols}) \
         VALUES ('{}', '{}', '{id}', '2020-01-01'{extra_vals})",
        f.work_id, f.source_id
    )
}

fn state_sql(f: &Fixture, status: &str, extra_cols: &str, extra_vals: &str) -> String {
    format!(
        "INSERT INTO metric_citation_work_state \
         (cited_work_id, source_id, match_status{extra_cols}) \
         VALUES ('{}', '{}', '{status}'{extra_vals})",
        f.work_id, f.source_id
    )
}

#[test]
fn migration_creates_empty_tables_and_enums() {
    let (_guard, pool) = setup_registry_db();
    for table in [
        "metric_citation_work_state",
        "metric_citation_link",
        "metric_citation_refresh",
        "metric_citation_refresh_seen",
        "metric_citation_batch",
    ] {
        assert_eq!(
            scalar_i64(&pool, &format!("(SELECT COUNT(*) FROM {table})")),
            0
        );
    }
    assert_eq!(
        enum_labels(&pool, "metric_citation_match_status"),
        ["MATCHED", "NOT_FOUND"]
    );
    assert_eq!(
        enum_labels(&pool, "metric_citation_link_status"),
        ["ACTIVE", "WITHDRAWN"]
    );
    assert_eq!(
        enum_labels(&pool, "metric_citation_refresh_status"),
        ["RUNNING", "COMPLETED", "COMPLETED_WITH_ERRORS", "FAILED"]
    );
}

#[test]
fn work_state_rules() {
    let (_guard, pool) = setup_registry_db();
    let f = fixture(&pool);
    // MATCHED needs its identity and time.
    assert!(exec(&pool, &state_sql(&f, "MATCHED", "", "")).is_err());
    assert!(exec(
        &pool,
        &state_sql(&f, "MATCHED", ", last_matched_at", ", now()")
    )
    .is_err());
    // A work never matched holds no source values.
    assert!(exec(
        &pool,
        &state_sql(&f, "NOT_FOUND", ", reported_count", ", 3")
    )
    .is_err());
    assert!(exec(
        &pool,
        &state_sql(&f, "NOT_FOUND", ", external_work_id", ", 'W1'")
    )
    .is_err());
    // Negative counts are refused.
    assert!(exec(
        &pool,
        &state_sql(
            &f,
            "MATCHED",
            ", last_matched_at, external_work_id, reported_count",
            ", now(), 'W1', -1"
        )
    )
    .is_err());
    // A clean NOT_FOUND, then it becomes MATCHED, and a later NOT_FOUND keeps
    // the last MATCHED values (the CHECKs allow it).
    exec(&pool, &state_sql(&f, "NOT_FOUND", "", "")).expect("not found");
    exec(
        &pool,
        &format!(
            "UPDATE metric_citation_work_state SET match_status = 'MATCHED', \
             last_matched_at = now(), external_work_id = 'W1', reported_count = 7, \
             enumerated_count = 7 WHERE cited_work_id = '{}'",
            f.work_id
        ),
    )
    .expect("matched");
    exec(
        &pool,
        &format!(
            "UPDATE metric_citation_work_state SET match_status = 'NOT_FOUND', \
             unconfirmed = TRUE WHERE cited_work_id = '{}'",
            f.work_id
        ),
    )
    .expect("not found after matched keeps values");
    let mut connection = pool.get().unwrap();
    let rows: Vec<MetricCitationWorkState> = metric_citation_work_state::table
        .load(&mut connection)
        .expect("row model");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].match_status, MetricCitationMatchStatus::NotFound);
    assert_eq!(rows[0].external_work_id.as_deref(), Some("W1"));
    assert_eq!(rows[0].reported_count, Some(7));
    assert!(rows[0].unconfirmed);
    // Foreign keys do not cascade: the work cannot be deleted.
    assert!(exec(
        &pool,
        &format!("DELETE FROM work WHERE work_id = '{}'", f.work_id)
    )
    .is_err());
}

#[test]
fn link_rules() {
    let (_guard, pool) = setup_registry_db();
    let f = fixture(&pool);
    exec(
        &pool,
        &link_sql(
            &f,
            "W1",
            ", citing_doi, citing_url, citing_authors",
            ", '10.1/a', 'https://doi.org/10.1/a', '{\"names\": [\"A\"], \"total\": 1}'",
        ),
    )
    .expect("a good link");
    // Same external id, and a second ACTIVE link with the same DOI.
    assert!(exec(&pool, &link_sql(&f, "W1", "", "")).is_err());
    assert!(exec(&pool, &link_sql(&f, "W2", ", citing_doi", ", '10.1/a'")).is_err());
    // After the first is withdrawn the DOI is free for another ACTIVE row.
    exec(
        &pool,
        "UPDATE metric_citation_link SET status = 'WITHDRAWN', withdrawn_at = now() \
         WHERE citing_external_id = 'W1'",
    )
    .expect("withdraw");
    exec(&pool, &link_sql(&f, "W2", ", citing_doi", ", '10.1/a'")).expect("doi free again");
    // Status and withdrawn_at must agree.
    assert!(exec(
        &pool,
        "UPDATE metric_citation_link SET withdrawn_at = now() WHERE citing_external_id = 'W2'"
    )
    .is_err());
    assert!(exec(
        &pool,
        "UPDATE metric_citation_link SET status = 'WITHDRAWN' WHERE citing_external_id = 'W2'"
    )
    .is_err());
    // Shape and size rules.
    for (cols, vals) in [
        (", citing_doi", ", '10.1/UPPER'"),
        (", citing_doi", ", 'not-a-doi'"),
        (", citing_url", ", 'http://x.org/a'"),
        (", citing_url", ", 'https://user@x.org/a'"),
        (
            ", citing_authors",
            ", '{\"names\": [], \"total\": 1, \"x\": 1}'",
        ),
        (
            ", citing_authors",
            ", '{\"names\": [\"1\",\"2\",\"3\",\"4\",\"5\",\"6\"], \"total\": 6}'",
        ),
        (", citing_authors", ", '[]'"),
        (", citing_authors", ", '{\"names\": [1], \"total\": 1}'"),
        (", citing_authors", ", '{\"names\": [null], \"total\": 1}'"),
        (
            ", citing_authors",
            ", format('{\"names\": [\"%s\"], \"total\": 1}', repeat('n', 201))::jsonb",
        ),
        (
            ", citing_authors",
            ", '{\"names\": [\"A\"], \"total\": -1}'",
        ),
        (", citing_type", ", repeat('t', 65)"),
        (", citing_title", ", repeat('t', 1001)"),
        (", citing_venue", ", repeat('v', 501)"),
    ] {
        assert!(
            exec(&pool, &link_sql(&f, "Wbad", cols, vals)).is_err(),
            "{cols} {vals} must be refused"
        );
    }
    // Shape edge cases that are valid: no path, an `@` in the query, a name of
    // exactly 200 characters, and several ACTIVE links with no DOI.
    exec(
        &pool,
        &link_sql(&f, "Wq", ", citing_url", ", 'https://x.org?a=b@c'"),
    )
    .expect("query with @");
    exec(
        &pool,
        &link_sql(
            &f,
            "Wn",
            ", citing_authors",
            ", format('{\"names\": [\"%s\"], \"total\": 1}', repeat('n', 200))::jsonb",
        ),
    )
    .expect("200 characters");
    exec(&pool, &link_sql(&f, "Wnull1", "", "")).expect("no doi");
    exec(&pool, &link_sql(&f, "Wnull2", "", "")).expect("a second no doi");
    assert!(exec(&pool, &link_sql(&f, "   ", "", "")).is_err());
    assert!(exec(&pool, &link_sql(&f, &"x".repeat(201), "", "")).is_err());
    let mut connection = pool.get().unwrap();
    let rows: Vec<MetricCitationLink> = metric_citation_link::table
        .order(metric_citation_link::citing_external_id)
        .load(&mut connection)
        .expect("row model");
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[0].status, MetricCitationLinkStatus::Withdrawn);
    assert_eq!(rows[0].citing_external_id, "W1");
    assert!(rows[0].withdrawn_at.is_some());
    assert!(rows[1..]
        .iter()
        .all(|r| r.status == MetricCitationLinkStatus::Active));
    // Foreign keys do not cascade: neither the work nor the source can be
    // deleted while a link names them.
    assert!(exec(
        &pool,
        &format!("DELETE FROM work WHERE work_id = '{}'", f.work_id)
    )
    .is_err());
    assert!(exec(
        &pool,
        &format!(
            "DELETE FROM metric_source WHERE source_id = '{}'",
            f.source_id
        )
    )
    .is_err());
}

#[test]
fn refresh_seen_and_batch_rules() {
    let (_guard, pool) = setup_registry_db();
    let f = fixture(&pool);
    let refresh =
        |status: &str, done: &str| {
            format!(
            "INSERT INTO metric_citation_refresh (source_account_id, claim_token_hash, status{}) \
             VALUES ('{}', 'hash', '{status}'{})",
            if done.is_empty() { "" } else { ", completed_at" },
            f.account_id,
            if done.is_empty() { String::new() } else { format!(", {done}") }
        )
        };
    exec(&pool, &refresh("RUNNING", "")).expect("running");
    // Only one RUNNING refresh per account.
    assert!(exec(&pool, &refresh("RUNNING", "")).is_err());
    // RUNNING has no completion time; a terminal status needs one.
    assert!(exec(&pool, &refresh("COMPLETED", "")).is_err());
    exec(&pool, &refresh("COMPLETED", "now()")).expect("completed");
    assert!(exec(
        &pool,
        &format!(
            "INSERT INTO metric_citation_refresh (source_account_id, claim_token_hash, \
             completed_at) VALUES ('{}', 'h2', now())",
            f.account_id
        )
    )
    .is_err());
    // Counters are non-negative.
    assert!(exec(
        &pool,
        "UPDATE metric_citation_refresh SET links_merged = -1"
    )
    .is_err());

    let refresh_id = scalar_uuid(
        &pool,
        "SELECT refresh_id FROM metric_citation_refresh LIMIT 1",
    );
    let seen = |id: &str| {
        format!(
            "INSERT INTO metric_citation_refresh_seen (refresh_id, work_id, citing_external_id) VALUES ('{refresh_id}', '{}', '{id}')",
            f.work_id
        )
    };
    exec(&pool, &seen("W1")).expect("seen");
    assert!(exec(&pool, &seen("W1")).is_err());
    assert!(exec(&pool, &seen(&"x".repeat(201))).is_err());
    let batch = |key: &str| {
        format!(
            "INSERT INTO metric_citation_batch (refresh_id, batch_key, payload_hash, outcome) \
             VALUES ('{refresh_id}', '{key}', 'h', '{{}}')"
        )
    };
    exec(&pool, &batch("b1")).expect("batch");
    assert!(exec(&pool, &batch("b1")).is_err());
    assert!(exec(&pool, &batch("  ")).is_err());
    // A refresh with seen ids or batches cannot be deleted.
    assert!(exec(&pool, "DELETE FROM metric_citation_refresh").is_err());
}

fn scalar_uuid(pool: &PgPool, query: &str) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        refresh_id: Uuid,
    }
    let mut connection = pool.get().unwrap();
    sql_query(query)
        .get_result::<Row>(&mut connection)
        .expect("uuid")
        .refresh_id
}

const TABLES: [&str; 5] = [
    "metric_citation_batch",
    "metric_citation_refresh_seen",
    "metric_citation_refresh",
    "metric_citation_link",
    "metric_citation_work_state",
];

/// Run `down.sql` in a transaction that is always rolled back; the result is
/// its message when it refused, or `None` when it ran.
fn run_down(pool: &PgPool) -> Option<String> {
    let mut connection = pool.get().unwrap();
    let outcome: Result<(), DieselError> = connection.transaction(|connection| {
        connection.batch_execute(DOWN_SQL)?;
        // Prove the tables are gone inside the transaction.
        for table in TABLES {
            let gone = connection
                .batch_execute(&format!("SELECT 1 FROM public.{table}"))
                .is_err();
            assert!(gone, "{table} must be dropped by down.sql");
        }
        Err(DieselError::RollbackTransaction)
    });
    match outcome {
        Err(DieselError::RollbackTransaction) => None,
        Err(other) => Some(format!("{other:?}")),
        Ok(()) => unreachable!(),
    }
}

#[test]
fn down_runs_when_empty_and_refuses_for_every_table_that_holds_a_row() {
    let (_guard, pool) = setup_registry_db();
    let f = fixture(&pool);
    // Empty: the script runs and drops the five tables.
    assert_eq!(run_down(&pool), None);
    // One row in each table in turn: the script refuses.
    let refresh = |id: Uuid| {
        format!(
            "INSERT INTO metric_citation_refresh (refresh_id, source_account_id, \
             claim_token_hash) VALUES ('{id}', '{}', 'hash')",
            f.account_id
        )
    };
    let refresh_id = Uuid::new_v4();
    let cases: Vec<(&str, Vec<String>)> = vec![
        ("link", vec![link_sql(&f, "W1", "", "")]),
        ("work_state", vec![state_sql(&f, "NOT_FOUND", "", "")]),
        ("refresh", vec![refresh(refresh_id)]),
        (
            "seen",
            vec![
                refresh(refresh_id),
                format!(
                    "INSERT INTO metric_citation_refresh_seen (refresh_id, work_id, \
                     citing_external_id) VALUES ('{refresh_id}', '{}', 'W1')",
                    f.work_id
                ),
            ],
        ),
        (
            "batch",
            vec![
                refresh(refresh_id),
                format!(
                    "INSERT INTO metric_citation_batch (refresh_id, batch_key, payload_hash, \
                     outcome) VALUES ('{refresh_id}', 'b', 'h', '{{}}')"
                ),
            ],
        ),
    ];
    for (name, statements) in cases {
        for statement in &statements {
            exec(&pool, statement).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        }
        let message = run_down(&pool).unwrap_or_else(|| panic!("{name}: down must refuse"));
        assert!(message.contains("rollback refused"), "{name}: {message}");
        for table in ["metric_citation_batch", "metric_citation_refresh_seen"] {
            exec(&pool, &format!("DELETE FROM {table}")).unwrap();
        }
        for table in [
            "metric_citation_refresh",
            "metric_citation_link",
            "metric_citation_work_state",
        ] {
            exec(&pool, &format!("DELETE FROM {table}")).unwrap();
        }
    }
}
