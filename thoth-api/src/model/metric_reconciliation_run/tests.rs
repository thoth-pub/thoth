//! Focused `MET-WP1-11` database tests for `metric_reconciliation_run`: the
//! approved six-field contract, the repository-standard UUID identity default,
//! required schemaless `scope`, opaque required `status`, the deliberately
//! defaultless required `started_at`, nullable `completed_at`, the `'{}'`
//! `summary` default, the exact column/check/index/default inventory, the
//! absence of any reconciliation enum, index, trigger, stored procedure, seed
//! row, runtime state machine or GraphQL surface, and the targeted
//! revert/reapply of the reconciliation migration, which owns both tables.
//!
//! The raw-SQL fixture helpers are shared with
//! `metric_reconciliation_issue/tests.rs`, the only other module this task's
//! write budget contains: no third model's test module is widened.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no reconciliation run at runtime and implements no reconciliation
//! execution, comparison of source manifests, canonical records, rollups or
//! OPERAS ledgers, issue classification, status/type/severity vocabulary,
//! resolution workflow, loop prevention, divergence handling, snapshot or
//! rolling scan, completeness determination, claim, lease, retry or backoff
//! behaviour, and nothing here pretends otherwise. In particular an accepted
//! `status` string is evidence of nonblank-text storage only, never of a
//! recognised reconciliation state, and an accepted `scope` or `summary`
//! document is evidence of JSONB storage only, never of a validated
//! reconciliation scope or outcome.
//!
//! Completeness boundary (section 15.5, reviewed): persisting reconciliation
//! runs does not solve, weaken or narrow the OPERAS inbound-completeness
//! blocker. These tests therefore assert the *absence* of any completeness,
//! coverage, cursor, scan or snapshot column, and no test here may be read as
//! evidence that reconciliation is complete. WP9 owns completeness reporting.
//!
//! Indexing boundary (reviewed and closed): the complete MET-WP1-11 index
//! inventory is this table's primary key and the issue table's primary key.
//! The inventory assertion below is exact, and WP9 may add a reconciliation
//! index only from an actual access pattern with query-plan evidence.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricReconciliationRun;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{index_definition, index_names};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_reconciliation_run;

/// The Diesel migration version of `thoth-api/migrations/20260907_v1.9.0`.
pub(crate) const MET_WP1_11_MIGRATION_VERSION: &str = "20260907";

/// Column names that would betray an invented reconciliation runtime — a
/// status/timestamp state machine, a claim/lease/retry protocol, a resolution
/// workflow or a completeness claim — having been smuggled into this
/// persistence-only slice. The approved design names none of them.
const DEFERRED_RUN_COLUMNS: [&str; 16] = [
    "attempt_count",
    "claim_token",
    "claimed_at",
    "claimed_by",
    "completeness",
    "created_at",
    "cursor",
    "heartbeat_at",
    "is_complete",
    "lease_expires_at",
    "next_attempt_at",
    "retry_at",
    "scan_id",
    "snapshot_id",
    "unverified",
    "updated_at",
];

/// One column value in a raw-SQL fixture insert.
///
/// The three states are genuinely different for this table: `Omitted` proves
/// what the database does when the writer supplies nothing — which is the
/// whole point of the `started_at` and `summary` decisions — while `Null`
/// proves explicit-NULL handling and `Value` proves round-tripping.
pub(crate) enum Column<'a> {
    /// Left out of the INSERT entirely, so any database default, or its
    /// deliberate absence, is exercised rather than restated by the fixture.
    Omitted,
    /// Supplied as an explicit SQL NULL.
    Null,
    /// Supplied as this literal, cast to the column's type.
    Value(&'a str),
}

/// Append one fixture column to a raw-SQL INSERT under construction.
///
/// Fixture literals are quoted by doubling embedded apostrophes and cast
/// explicitly, so JSON documents and timestamps reach the column typed rather
/// than through an inferred parameter type.
pub(crate) fn push_column(
    columns: &mut Vec<&'static str>,
    values: &mut Vec<String>,
    name: &'static str,
    column: Column<'_>,
    cast: &str,
) {
    match column {
        Column::Omitted => {}
        Column::Null => {
            columns.push(name);
            values.push("NULL".to_string());
        }
        Column::Value(value) => {
            columns.push(name);
            values.push(format!("'{}'{cast}", value.replace('\'', "''")));
        }
    }
}

/// The column values one raw-SQL reconciliation-run insert supplies.
pub(crate) struct RunRow<'a> {
    pub(crate) run_id: Column<'a>,
    pub(crate) scope: Column<'a>,
    pub(crate) status: Column<'a>,
    pub(crate) started_at: Column<'a>,
    pub(crate) completed_at: Column<'a>,
    pub(crate) summary: Column<'a>,
}

impl RunRow<'_> {
    /// The minimal valid row: an arbitrary nonblank fixture scope and status
    /// plus the required execution start, with the identity, completion and
    /// summary columns left out so their defaults — and `started_at`'s
    /// deliberate lack of one — are exercised rather than restated. None of
    /// these values is approved reconciliation data.
    pub(crate) fn minimal() -> Self {
        Self {
            run_id: Column::Omitted,
            scope: Column::Value(r#"{"fixture":"scope"}"#),
            status: Column::Value("fixture-status"),
            started_at: Column::Value(FIXTURE_STARTED_AT),
            completed_at: Column::Omitted,
            summary: Column::Omitted,
        }
    }
}

/// An arbitrary fixture execution start. It is deliberately not "now": the
/// writer supplies the actual reconciliation-execution start, so the tests
/// must be able to prove the stored value is the supplied one.
pub(crate) const FIXTURE_STARTED_AT: &str = "2026-03-04T05:06:07.891011Z";

/// Insert one reconciliation run through raw SQL.
pub(crate) fn insert_run_row(pool: &PgPool, row: RunRow<'_>) -> Result<usize, DieselError> {
    let mut columns: Vec<&'static str> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    push_column(&mut columns, &mut values, "run_id", row.run_id, "::uuid");
    push_column(&mut columns, &mut values, "scope", row.scope, "::jsonb");
    push_column(&mut columns, &mut values, "status", row.status, "");
    push_column(
        &mut columns,
        &mut values,
        "started_at",
        row.started_at,
        "::timestamptz",
    );
    push_column(
        &mut columns,
        &mut values,
        "completed_at",
        row.completed_at,
        "::timestamptz",
    );
    push_column(&mut columns, &mut values, "summary", row.summary, "::jsonb");

    let mut connection = pool.get().expect("Failed to get DB connection");
    if columns.is_empty() {
        return sql_query("INSERT INTO metric_reconciliation_run DEFAULT VALUES")
            .execute(&mut connection);
    }
    sql_query(format!(
        "INSERT INTO metric_reconciliation_run ({}) VALUES ({})",
        columns.join(", "),
        values.join(", ")
    ))
    .execute(&mut connection)
}

/// Insert one minimal reconciliation run and return its generated id.
pub(crate) fn fixture_run(pool: &PgPool) -> Uuid {
    insert_run_row(pool, RunRow::minimal())
        .expect("Failed to insert the fixture reconciliation run");
    only_run(pool).run_id
}

/// The single stored reconciliation run.
fn only_run(pool: &PgPool) -> MetricReconciliationRun {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_reconciliation_run::table
        .first(&mut connection)
        .expect("Failed to load the stored reconciliation run")
}

#[test]
fn migration_seeds_no_reconciliation_run_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        0,
        "MET-WP1-11 must not seed any metric_reconciliation_run row: this slice \
         executes no reconciliation and compares nothing"
    );
}

#[test]
fn a_complete_reconciliation_run_round_trips_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Fixture values only. None of these is a real reconciliation scope,
    // status or summary.
    let run_id = Uuid::new_v4();
    let started_at = Timestamp::parse_from_rfc3339(FIXTURE_STARTED_AT)
        .expect("Failed to parse the fixture execution start");
    let completed_at = Timestamp::parse_from_rfc3339("2026-03-04T06:07:08.910111Z")
        .expect("Failed to parse the fixture completion time");
    let scope = serde_json::json!({"fixture": "scope", "ledgers": ["a", "b"]});
    let summary = serde_json::json!({"fixture": "summary", "counted": 3});

    diesel::insert_into(metric_reconciliation_run::table)
        .values((
            metric_reconciliation_run::run_id.eq(run_id),
            metric_reconciliation_run::scope.eq(&scope),
            metric_reconciliation_run::status.eq("fixture-status"),
            metric_reconciliation_run::started_at.eq(started_at),
            metric_reconciliation_run::completed_at.eq(Some(completed_at)),
            metric_reconciliation_run::summary.eq(&summary),
        ))
        .execute(&mut connection)
        .expect("Failed to insert the complete reconciliation run");

    let loaded: MetricReconciliationRun = metric_reconciliation_run::table
        .filter(metric_reconciliation_run::run_id.eq(run_id))
        .first(&mut connection)
        .expect("Failed to load the complete reconciliation run");
    assert_eq!(
        loaded,
        MetricReconciliationRun {
            run_id,
            scope,
            status: "fixture-status".to_string(),
            started_at,
            completed_at: Some(completed_at),
            summary,
        },
        "every approved reconciliation-run field must round-trip through the \
         manually maintained Diesel contract unchanged"
    );
}

#[test]
fn the_repository_standard_uuid_default_supplies_a_run_identity() {
    let (_guard, pool) = setup_registry_db();
    insert_run_row(&pool, RunRow::minimal())
        .expect("a run inserted without an explicit identity must be accepted");
    insert_run_row(
        &pool,
        RunRow {
            scope: Column::Value(r#"{"fixture":"second-scope"}"#),
            ..RunRow::minimal()
        },
    )
    .expect("a second run inserted without an explicit identity must be accepted");

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT run_id) FROM metric_reconciliation_run)"
        ),
        2,
        "the repository-standard UUID default must generate a distinct identity \
         per run when the insert omits run_id"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_reconciliation_run \
              WHERE run_id = '00000000-0000-0000-0000-000000000000'::uuid)"
        ),
        0,
        "the generated identity must not be the nil UUID"
    );
}

#[test]
fn arbitrary_structured_scope_round_trips_and_no_json_schema_is_imposed() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Deliberately mixed and deliberately not a vocabulary: accepting all of
    // these is the point. Reconciliation may cover different combinations of
    // source, canonical, rollup and OPERAS state, and the approved design fixes
    // no scope schema, required key or scalar code, so the column stores
    // structured JSON and nothing more.
    let scopes = [
        serde_json::json!({}),
        serde_json::json!({"ledgers": ["operas_export", "operas_import"]}),
        serde_json::json!({"nested": {"deeply": {"structured": [1, 2, 3]}}}),
        serde_json::json!([{"not": "an object at all"}]),
        serde_json::json!("a bare JSON string"),
        serde_json::json!(0),
        serde_json::json!(null),
        serde_json::json!({"ünïcödé": "välüé", "empty": {}, "flag": false}),
    ];

    for (index, scope) in scopes.iter().enumerate() {
        let run_id = Uuid::new_v4();
        diesel::insert_into(metric_reconciliation_run::table)
            .values((
                metric_reconciliation_run::run_id.eq(run_id),
                metric_reconciliation_run::scope.eq(scope),
                metric_reconciliation_run::status.eq(format!("fixture-status-{index}")),
                metric_reconciliation_run::started_at.eq(Timestamp::default()),
            ))
            .execute(&mut connection)
            .unwrap_or_else(|error| {
                panic!("arbitrary structured scope {scope} must be accepted: {error:?}")
            });

        let stored: serde_json::Value = metric_reconciliation_run::table
            .filter(metric_reconciliation_run::run_id.eq(run_id))
            .select(metric_reconciliation_run::scope)
            .first(&mut connection)
            .expect("Failed to load the stored scope");
        assert_eq!(
            &stored, scope,
            "{scope} must round-trip unchanged; storing it is not a claim that \
             it is a validated reconciliation scope"
        );
    }
}

#[test]
fn scope_is_required_and_carries_no_default() {
    let (_guard, pool) = setup_registry_db();

    // A run must state what it covered, and this slice defines no default
    // coverage, so the database must neither invent one nor accept its absence.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_run' \
                AND column_name = 'scope' AND column_default IS NOT NULL)"
        ),
        0,
        "scope must carry no database default"
    );

    let result = insert_run_row(
        &pool,
        RunRow {
            scope: Column::Omitted,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "a run omitting scope must be rejected, got {result:?}"
    );
    let result = insert_run_row(
        &pool,
        RunRow {
            scope: Column::Null,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicit NULL scope must be rejected, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        0
    );
}

#[test]
fn arbitrary_nonblank_status_text_round_trips() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Deliberately mixed and deliberately not a vocabulary. The approved design
    // names `status` but supplies no closed states, so this slice must not
    // invent RUNNING, COMPLETED, FAILED, UNVERIFIED or any other label by
    // analogy. The padded value must round-trip byte-for-byte, proving nothing
    // trims, normalizes or case-folds it.
    let values = [
        "fixture-value",
        "RUNNING",
        "completed",
        "a",
        "  padded fixture value  ",
        "ünïcödé välüé",
        "not a status at all",
        "0123456789abcdef",
    ];

    for (index, value) in values.into_iter().enumerate() {
        let run_id = Uuid::new_v4();
        insert_run_row(
            &pool,
            RunRow {
                run_id: Column::Value(&run_id.to_string()),
                status: Column::Value(value),
                scope: Column::Value(&format!(r#"{{"fixture":{index}}}"#)),
                ..RunRow::minimal()
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank status {value:?} must be accepted: {error:?}")
        });

        let stored: String = metric_reconciliation_run::table
            .filter(metric_reconciliation_run::run_id.eq(run_id))
            .select(metric_reconciliation_run::status)
            .first(&mut connection)
            .expect("Failed to load the stored status");
        assert_eq!(
            stored, value,
            "{value:?} must round-trip unchanged; storing it is not a claim that \
             it is a recognised reconciliation state"
        );
    }
}

#[test]
fn blank_and_whitespace_only_status_is_rejected() {
    let (_guard, pool) = setup_registry_db();

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let result = insert_run_row(
            &pool,
            RunRow {
                status: Column::Value(blank),
                ..RunRow::minimal()
            },
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank/whitespace-only status variant {index} ({blank:?}) must be \
             rejected by the required-text CHECK, got {result:?}"
        );
    }

    let result = insert_run_row(
        &pool,
        RunRow {
            status: Column::Null,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "a NULL status must be rejected, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        0,
        "no rejected reconciliation run may have been stored"
    );
}

#[test]
fn started_at_has_no_database_default_and_must_be_supplied_by_the_writer() {
    let (_guard, pool) = setup_registry_db();

    // The reviewed amendment: `started_at` is the actual reconciliation
    // execution start supplied by the writer, and the approved design does not
    // establish that Thoth's durable insertion time and the execution start are
    // the same event. The database must therefore never silently substitute
    // CURRENT_TIMESTAMP, unlike the repository-standard `created_at` columns
    // elsewhere in Metrics.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_run' \
                AND column_name = 'started_at' AND column_default IS NOT NULL)"
        ),
        0,
        "started_at must carry no database default: substituting the insertion \
         time would overstate reconciliation execution-start semantics"
    );

    let result = insert_run_row(
        &pool,
        RunRow {
            started_at: Column::Omitted,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "a run omitting started_at must fail rather than be given the current \
         time, got {result:?}"
    );
    let result = insert_run_row(
        &pool,
        RunRow {
            started_at: Column::Null,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicit NULL started_at must be rejected, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        0
    );

    // An explicitly supplied execution start must round-trip exactly, including
    // its sub-second precision, and must not be replaced by the insertion time.
    insert_run_row(&pool, RunRow::minimal()).expect("a run supplying started_at must be accepted");
    let loaded = only_run(&pool);
    assert_eq!(
        loaded.started_at,
        Timestamp::parse_from_rfc3339(FIXTURE_STARTED_AT)
            .expect("Failed to parse the fixture execution start"),
        "the supplied started_at must round-trip exactly"
    );
}

#[test]
fn completed_at_is_nullable_and_carries_no_default_or_status_invariant() {
    let (_guard, pool) = setup_registry_db();

    // The row exists before completion, and the database does not decide when a
    // run is complete: the status vocabulary and its transition graph are not
    // design-fixed and remain WP9-owned, so no cross-column rule ties
    // completed_at to status.
    insert_run_row(&pool, RunRow::minimal()).expect("a run omitting completed_at must be accepted");
    assert_eq!(
        only_run(&pool).completed_at,
        None,
        "an omitted completed_at must remain NULL rather than acquire a default"
    );

    insert_run_row(
        &pool,
        RunRow {
            scope: Column::Value(r#"{"fixture":"explicit-null"}"#),
            completed_at: Column::Null,
            ..RunRow::minimal()
        },
    )
    .expect("an explicit NULL completed_at must be accepted");

    // A completion time with no matching status change, and a status change
    // with no completion time, must both stay representable.
    insert_run_row(
        &pool,
        RunRow {
            scope: Column::Value(r#"{"fixture":"completed"}"#),
            status: Column::Value("fixture-still-running"),
            completed_at: Column::Value("2026-03-04T06:07:08Z"),
            ..RunRow::minimal()
        },
    )
    .expect(
        "a completed_at with an arbitrary status must be accepted: no status \
         or timestamp state machine is authorized in this slice",
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        3
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_reconciliation_run WHERE completed_at IS NULL)"
        ),
        2
    );
}

#[test]
fn summary_defaults_to_an_empty_object_and_arbitrary_structure_round_trips() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    insert_run_row(&pool, RunRow::minimal()).expect("a run omitting summary must be accepted");
    assert_eq!(
        only_run(&pool).summary,
        serde_json::json!({}),
        "an omitted summary must default to an empty object, so a run exists \
         before its machine-readable outcome is known"
    );

    // No required keys and no semantic interpretation of an empty object: the
    // approved design calls the field `summary` but defines no schema.
    let summaries = [
        serde_json::json!({}),
        serde_json::json!({"missing_exports": 4, "value_divergences": 0}),
        serde_json::json!({"nested": {"by_platform": [{"id": 1}, {"id": 2}]}}),
        serde_json::json!([1, 2, 3]),
        serde_json::json!("a bare JSON string"),
        serde_json::json!(null),
    ];
    for (index, summary) in summaries.iter().enumerate() {
        let run_id = Uuid::new_v4();
        diesel::insert_into(metric_reconciliation_run::table)
            .values((
                metric_reconciliation_run::run_id.eq(run_id),
                metric_reconciliation_run::scope.eq(serde_json::json!({"fixture": index})),
                metric_reconciliation_run::status.eq("fixture-status"),
                metric_reconciliation_run::started_at.eq(Timestamp::default()),
                metric_reconciliation_run::summary.eq(summary),
            ))
            .execute(&mut connection)
            .unwrap_or_else(|error| {
                panic!("arbitrary structured summary {summary} must be accepted: {error:?}")
            });

        let stored: serde_json::Value = metric_reconciliation_run::table
            .filter(metric_reconciliation_run::run_id.eq(run_id))
            .select(metric_reconciliation_run::summary)
            .first(&mut connection)
            .expect("Failed to load the stored summary");
        assert_eq!(
            &stored, summary,
            "{summary} must round-trip unchanged; storing it is not a claim that \
             it is a validated reconciliation outcome"
        );
    }

    // An explicit NULL is still rejected: the column is NOT NULL with a default,
    // not nullable.
    let result = insert_run_row(
        &pool,
        RunRow {
            scope: Column::Value(r#"{"fixture":"null-summary"}"#),
            summary: Column::Null,
            ..RunRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicit NULL summary must be rejected, got {result:?}"
    );
}

#[test]
fn metric_reconciliation_run_has_exactly_the_approved_columns() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let columns: Vec<(String, String, String)> = sql_query(
        "SELECT column_name::text, data_type::text, is_nullable::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_run' \
         ORDER BY ordinal_position",
    )
    .load::<ColumnRow>(&mut connection)
    .expect("Failed to read the metric_reconciliation_run columns")
    .into_iter()
    .map(|column| (column.column_name, column.data_type, column.is_nullable))
    .collect();
    let observed: Vec<(&str, &str, &str)> = columns
        .iter()
        .map(|(name, data_type, nullable)| (name.as_str(), data_type.as_str(), nullable.as_str()))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("run_id", "uuid", "NO"),
            ("scope", "jsonb", "NO"),
            ("status", "text", "NO"),
            ("started_at", "timestamp with time zone", "NO"),
            ("completed_at", "timestamp with time zone", "YES"),
            ("summary", "jsonb", "NO"),
        ],
        "metric_reconciliation_run must carry exactly the six approved design \
         fields, in the approved order and nullability, with no created_at, \
         updated_at, lease, retry, heartbeat, cursor, scan, snapshot or \
         completeness column"
    );

    // Exactly two defaults: the repository-standard UUID identity and the empty
    // summary object. In particular started_at has none, because it is the
    // execution start supplied by the writer, and status has none, because this
    // slice defines no initial reconciliation state.
    #[derive(diesel::QueryableByName)]
    struct DefaultRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_default: String,
    }
    let defaults: Vec<(String, String)> = sql_query(
        "SELECT column_name::text, column_default::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_run' \
           AND column_default IS NOT NULL \
         ORDER BY ordinal_position",
    )
    .load::<DefaultRow>(&mut connection)
    .expect("Failed to read the metric_reconciliation_run defaults")
    .into_iter()
    .map(|row| (row.column_name, row.column_default))
    .collect();
    assert_eq!(
        defaults,
        vec![
            ("run_id".to_string(), "uuid_generate_v4()".to_string()),
            ("summary".to_string(), "'{}'::jsonb".to_string()),
        ],
        "only run_id and summary may carry a default: started_at must have \
         none, so an omitted execution start fails instead of silently becoming \
         the insertion time, and status must have none, because this slice \
         defines no initial reconciliation state"
    );

    // No runtime column was smuggled onto the run.
    for column in DEFERRED_RUN_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_reconciliation_run' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-11 must not add the deferred reconciliation-run column {column}"
        );
    }
}

#[test]
fn metric_reconciliation_run_has_exactly_the_approved_checks() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed: one nonblank required-text rule on status.
    // In particular there is no CHECK enumerating status values and no
    // cross-column rule tying status to started_at, completed_at or summary.
    assert_eq!(
        check_constraint_names(&pool, "metric_reconciliation_run"),
        vec!["metric_reconciliation_run_status_check"],
        "metric_reconciliation_run must carry exactly the one approved CHECK"
    );

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let definitions: Vec<(String, String)> = sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = 'public.metric_reconciliation_run'::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read the CHECK definitions")
    .into_iter()
    .map(|row| (row.conname, row.definition))
    .collect();
    for (name, definition) in &definitions {
        assert!(
            definition.contains("[^[:space:]]"),
            "{name} must be the existing Metrics required-text idiom and \
             nothing stronger — no vocabulary, transition graph or timestamp \
             invariant: {definition}"
        );
        assert!(
            !definition.contains("completed_at") && !definition.contains("started_at"),
            "{name} must not encode a status/timestamp state machine: {definition}"
        );
    }
}

#[test]
fn metric_reconciliation_run_has_exactly_the_required_indexes_and_no_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_reconciliation_run"),
        vec!["metric_reconciliation_run_pkey"],
        "metric_reconciliation_run must carry exactly its primary-key index; no \
         status, started_at, completed_at, scope or summary index may exist. \
         WP9 owns any later reconciliation indexing and may add one only from \
         an actual access pattern with query-plan evidence"
    );
    let primary_key = index_definition(
        &pool,
        "metric_reconciliation_run",
        "metric_reconciliation_run_pkey",
    );
    assert!(
        primary_key.contains("UNIQUE") && primary_key.contains("(run_id)"),
        "the primary key must be the surrogate run identity: {primary_key}"
    );

    // The run is the parent of the reconciliation audit unit and references
    // nothing: it carries no link to a source, platform, measure, import,
    // record, rollup or OPERAS ledger, because the approved design's run
    // shorthand names none and scope is deliberately schemaless JSONB.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_reconciliation_run'::regclass AND contype = 'f')"
        ),
        0,
        "metric_reconciliation_run must carry no foreign key"
    );
}

#[test]
fn no_reconciliation_enum_trigger_procedure_or_graphql_surface_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // No reconciliation status, issue-type or severity vocabulary was created.
    // `typtype = 'e'` restricts the count to enums, because PostgreSQL always
    // creates an implicit composite type named after the table itself.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typtype = 'e' \
                AND typname LIKE 'metric_reconciliation%')",
        ),
        0,
        "MET-WP1-11 must create no reconciliation enum type: the status, \
         issue-type and severity vocabularies are deliberately undefined at \
         this stage and remain WP9-owned"
    );

    // status stays plain text on the run.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_run' \
                AND column_name = 'status' AND data_type = 'text')",
        ),
        1,
        "status must remain unconstrained TEXT, not a PostgreSQL enum"
    );

    // No trigger or stored procedure implements a state machine, an automatic
    // run/issue lifecycle or a completeness determination.
    for table in ["metric_reconciliation_run", "metric_reconciliation_issue"] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_trigger \
                      WHERE tgrelid = 'public.{table}'::regclass AND NOT tgisinternal)"
                ),
            ),
            0,
            "MET-WP1-11 must install no trigger on {table}"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_proc \
              WHERE pronamespace = 'public'::regnamespace \
                AND proname LIKE '%reconciliation%')",
        ),
        0,
        "MET-WP1-11 must create no stored procedure for the reconciliation ledger"
    );

    // The complete MET-WP1-11 index inventory across both tables is exactly the
    // two primary keys.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND tablename IN ('metric_reconciliation_run', 'metric_reconciliation_issue'))",
        ),
        2,
        "MET-WP1-11 must create exactly two reconciliation indexes, both \
         primary keys, and no secondary index on any reconciliation column"
    );

    // No GraphQL surface: persistence and domain types exist, but nothing is
    // exposed. `recordMetricReconciliation` and every reconciliation query,
    // mutation, input and output type belong to the later WP9 API work.
    let sdl = crate::graphql::create_schema().as_sdl();
    assert!(
        !sdl.to_lowercase().contains("reconciliation"),
        "MET-WP1-11 must add no GraphQL API surface: the public SDL must not \
         mention reconciliation in any type, field, argument or description"
    );
}

/// Revert migrations until the `MET-WP1-11` reconciliation migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_operas_import_migration` and its
/// predecessors: a bare `revert_last_migration` would only mean "the
/// reconciliation migration" while it happens to be the newest applied
/// migration. Reverting down to and including the target keeps the meaning
/// under any later migration order, and no future migration name is assumed or
/// hard-coded.
fn revert_through_reconciliation_migration(connection: &mut PgConnection) {
    let reconciliation_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_11_MIGRATION_VERSION);
    assert!(
        reconciliation_migration_applied,
        "the MET-WP1-11 reconciliation migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_11_MIGRATION_VERSION {
            return;
        }
    }
}

#[test]
fn reverting_through_the_reconciliation_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    let count_objects = |connection: &mut PgConnection, query: &str| -> i64 {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
            .get_result(connection)
            .expect("Failed to count schema objects")
    };

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_reconciliation_run', 'metric_reconciliation_issue'))",
        ),
        2,
        "both reconciliation tables must exist before reverting"
    );

    revert_through_reconciliation_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_reconciliation_run', 'metric_reconciliation_issue'))",
        ),
        0,
        "the downgrade must drop exactly the two MET-WP1-11 tables"
    );

    // Every predecessor Metrics slice survives, including the table the issue
    // ledger references and the three merged OPERAS ledgers.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint', \
                                'metric_import', 'metric_import_error', \
                                'metric_record', 'metric_record_revision', \
                                'metric_record_provenance', 'metric_coverage', \
                                'metric_publisher_platform_approval', \
                                'metric_rollup_delta', 'metric_operas_mapping', \
                                'metric_operas_export', 'metric_operas_import'))",
        ),
        17,
        "the downgrade must leave the MET-WP1-01..10 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint WHERE conname = 'metric_record_pkey')",
        ),
        1,
        "the downgrade must not drop the referenced predecessor primary key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('work', 'publication', 'institution', 'publisher'))",
        ),
        4,
        "the downgrade must not touch the bibliographic schema"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2,
        "the downgrade must leave the MET-WP1-01 measure seeds in place"
    );

    // Reapplication recreates both empty tables with exactly their two indexes,
    // two foreign keys and four CHECKs.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the reconciliation migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "((SELECT COUNT(*) FROM metric_reconciliation_run) \
              + (SELECT COUNT(*) FROM metric_reconciliation_issue))"
        ),
        0,
        "reapplication must seed no reconciliation row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND tablename IN ('metric_reconciliation_run', 'metric_reconciliation_issue'))",
        ),
        2,
        "reapplication must restore exactly the two primary-key indexes"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid IN ('public.metric_reconciliation_run'::regclass, \
                                 'public.metric_reconciliation_issue'::regclass) \
                AND contype = 'f')",
        ),
        2,
        "reapplication must restore the two non-cascading foreign keys"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid IN ('public.metric_reconciliation_run'::regclass, \
                                 'public.metric_reconciliation_issue'::regclass) \
                AND contype = 'c')",
        ),
        4,
        "reapplication must restore the four approved CHECKs"
    );
}
