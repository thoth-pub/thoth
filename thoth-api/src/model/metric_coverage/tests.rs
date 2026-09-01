//! Focused `MET-WP1-05` database tests for `metric_coverage`: the closed
//! coverage-status enum, the approved field/default contract, the complete
//! authorized CHECK and foreign-key inventory, half-open period ordering, the
//! non-null status/boolean columns, the nullable `notes` column and the
//! targeted revert/reapply of the coverage-foundation migration.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! implements no coverage calculation, finalization, zero-versus-unknown
//! behaviour or normalized ingestion/`ingestMetricBatch` transaction, and
//! nothing here pretends otherwise.

use std::str::FromStr;

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::{MetricCoverage, MetricCoverageStatus};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::{fixture_source_account, insert_import_row};
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::schema::metric_coverage;

/// The Diesel migration version of `thoth-api/migrations/20260901_v1.9.0`.
pub(crate) const MET_WP1_05_MIGRATION_VERSION: &str = "20260901";

const COVERAGE_STATUSES: [(MetricCoverageStatus, &str); 3] = [
    (MetricCoverageStatus::Complete, "COMPLETE"),
    (MetricCoverageStatus::Partial, "PARTIAL"),
    (MetricCoverageStatus::Unknown, "UNKNOWN"),
];

/// Revert migrations until the `MET-WP1-05` coverage-foundation migration
/// itself has been reverted.
///
/// The same durable pattern as the earlier WP1 slices: a single
/// `revert_last_migration` would only mean "the coverage migration" while it
/// happens to be the newest applied migration. Reverting down to and
/// including the target keeps the meaning under any later migration order,
/// and no future migration name is assumed or hard-coded.
pub(crate) fn revert_through_coverage_migration(connection: &mut PgConnection) {
    let coverage_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_05_MIGRATION_VERSION);
    assert!(
        coverage_migration_applied,
        "the MET-WP1-05 coverage migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_05_MIGRATION_VERSION {
            return;
        }
    }
}

/// The registry/source/import entities one coverage row must resolve to.
pub(crate) struct CoverageFixture {
    pub(crate) source_account_id: Uuid,
    pub(crate) import_id: Uuid,
    pub(crate) platform_id: Uuid,
    pub(crate) measure_id: Uuid,
}

/// Insert one referenced source account/import pair and return them with the
/// registry ids a coverage row needs.
pub(crate) fn insert_coverage_fixture(pool: &PgPool) -> CoverageFixture {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);

    let mut connection = pool.get().expect("Failed to get DB connection");
    let (platform_id, measure_id) = registry_ids(&mut connection);
    CoverageFixture {
        source_account_id,
        import_id,
        platform_id,
        measure_id,
    }
}

/// The platform routed by the fixture source account and one seeded measure.
fn registry_ids(connection: &mut PgConnection) -> (Uuid, Uuid) {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        platform_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        measure_id: Uuid,
    }
    sql_query(
        "SELECT (SELECT platform_id FROM metric_platform ORDER BY code LIMIT 1) AS platform_id, \
                (SELECT measure_id FROM metric_measure WHERE code = 'title_sessions') AS measure_id",
    )
    .get_result::<Row>(connection)
    .map(|row| (row.platform_id, row.measure_id))
    .expect("Failed to read the fixture registry ids")
}

/// Insert one minimal coverage row through raw SQL so the database defaults
/// are exercised rather than restated by a Diesel fixture.
pub(crate) fn insert_coverage_row(
    pool: &PgPool,
    fixture: &CoverageFixture,
    coverage_id: Option<Uuid>,
    status: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match coverage_id {
        Some(coverage_id) => sql_query(format!(
            "INSERT INTO metric_coverage \
                 (coverage_id, source_account_id, import_id, platform_id, measure_id, \
                  period_start, period_end, coverage_status, country_coverage, \
                  institution_coverage) \
             VALUES ($1, $2, $3, $4, $5, DATE '2026-07-01', DATE '2026-08-01', \
                     '{status}', false, false)"
        ))
        .bind::<diesel::sql_types::Uuid, _>(coverage_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.import_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .execute(&mut connection),
        None => sql_query(format!(
            "INSERT INTO metric_coverage \
                 (source_account_id, import_id, platform_id, measure_id, \
                  period_start, period_end, coverage_status, country_coverage, \
                  institution_coverage) \
             VALUES ($1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', \
                     '{status}', false, false)"
        ))
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.import_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .execute(&mut connection),
    }
}

/// Insert one coverage row overriding only the reporting period.
fn insert_coverage_with_period(
    pool: &PgPool,
    fixture: &CoverageFixture,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_coverage \
             (source_account_id, import_id, platform_id, measure_id, \
              period_start, period_end, coverage_status, country_coverage, \
              institution_coverage) \
         VALUES ($1, $2, $3, $4, $5, $6, 'UNKNOWN', false, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.import_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
    .bind::<diesel::sql_types::Date, _>(period_start)
    .bind::<diesel::sql_types::Date, _>(period_end)
    .execute(&mut connection)
}

pub(crate) fn delete_row(
    pool: &PgPool,
    table: &str,
    id_column: &str,
    id: Uuid,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!("DELETE FROM {table} WHERE {id_column} = $1"))
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut connection)
}

/// The sorted names of one table's foreign-key constraints, with definitions.
fn foreign_keys(pool: &PgPool, table: &str) -> Vec<(String, String)> {
    #[derive(diesel::QueryableByName)]
    struct ForeignKey {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = $1::regclass AND c.contype = 'f' \
         ORDER BY c.conname",
    )
    .bind::<diesel::sql_types::Text, _>(format!("public.{table}"))
    .load::<ForeignKey>(&mut connection)
    .expect("Failed to read foreign keys")
    .into_iter()
    .map(|key| (key.conname, key.definition))
    .collect()
}

/// The sorted index names of one table.
fn index_names(pool: &PgPool, table: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        indexname: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT indexname::text AS indexname FROM pg_indexes \
         WHERE schemaname = 'public' AND tablename = $1 ORDER BY indexname",
    )
    .bind::<diesel::sql_types::Text, _>(table)
    .load::<Row>(&mut connection)
    .expect("Failed to read index names")
    .into_iter()
    .map(|row| row.indexname)
    .collect()
}

#[test]
fn coverage_status_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_coverage_status"),
        vec!["COMPLETE", "PARTIAL", "UNKNOWN"],
        "metric_coverage_status must carry exactly the three approved labels"
    );
}

#[test]
fn coverage_status_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in COVERAGE_STATUSES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(MetricCoverageStatus::from_str(label).unwrap(), variant);
    }
    assert!(MetricCoverageStatus::from_str("OTHER").is_err());
    assert!(MetricCoverageStatus::from_str("FULL").is_err());
    assert!(MetricCoverageStatus::from_str("complete").is_err());
}

#[test]
fn every_coverage_status_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in COVERAGE_STATUSES {
        assert_db_enum_roundtrip::<
            MetricCoverageStatus,
            crate::schema::sql_types::MetricCoverageStatus,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_coverage_status"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_coverage_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_coverage)"),
        0,
        "MET-WP1-05 must not seed any metric_coverage row"
    );
}

#[test]
fn metric_coverage_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let coverage_id: Uuid = diesel::insert_into(metric_coverage::table)
        .values((
            metric_coverage::source_account_id.eq(fixture.source_account_id),
            metric_coverage::import_id.eq(fixture.import_id),
            metric_coverage::platform_id.eq(fixture.platform_id),
            metric_coverage::measure_id.eq(fixture.measure_id),
            metric_coverage::period_start.eq(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
            metric_coverage::period_end.eq(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
            metric_coverage::coverage_status.eq(MetricCoverageStatus::Partial),
            metric_coverage::country_coverage.eq(true),
            metric_coverage::institution_coverage.eq(false),
            metric_coverage::notes.eq("some notes"),
        ))
        .returning(metric_coverage::coverage_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated coverage row");

    let loaded: MetricCoverage = metric_coverage::table
        .filter(metric_coverage::coverage_id.eq(coverage_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated coverage row");
    assert_eq!(loaded.coverage_id, coverage_id);
    assert_eq!(loaded.source_account_id, fixture.source_account_id);
    assert_eq!(loaded.import_id, fixture.import_id);
    assert_eq!(loaded.platform_id, fixture.platform_id);
    assert_eq!(loaded.measure_id, fixture.measure_id);
    assert_eq!(
        loaded.period_start,
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
    );
    assert_eq!(
        loaded.period_end,
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()
    );
    assert_eq!(loaded.coverage_status, MetricCoverageStatus::Partial);
    assert!(loaded.country_coverage);
    assert!(!loaded.institution_coverage);
    assert_eq!(loaded.notes.as_deref(), Some("some notes"));
}

#[test]
fn coverage_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    insert_coverage_row(&pool, &fixture, None, "UNKNOWN")
        .expect("Failed to insert the defaulted coverage row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricCoverage = metric_coverage::table
        .first(&mut connection)
        .expect("Failed to load the defaulted coverage row");
    assert_ne!(
        loaded.coverage_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a coverage_id"
    );
    assert_eq!(
        loaded.notes, None,
        "notes is an optional annotation and must default to NULL"
    );
}

#[test]
fn nullable_notes_round_trip() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    insert_coverage_row(&pool, &fixture, None, "COMPLETE")
        .expect("Failed to insert the coverage row with no notes");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricCoverage = metric_coverage::table
        .first(&mut connection)
        .expect("Failed to load the coverage row");
    assert_eq!(loaded.notes, None);

    diesel::update(
        metric_coverage::table.filter(metric_coverage::coverage_id.eq(loaded.coverage_id)),
    )
    .set(metric_coverage::notes.eq(Some("a free-text annotation")))
    .execute(&mut connection)
    .expect("Failed to set notes");
    let with_notes: MetricCoverage = metric_coverage::table
        .filter(metric_coverage::coverage_id.eq(loaded.coverage_id))
        .first(&mut connection)
        .expect("Failed to reload the coverage row");
    assert_eq!(with_notes.notes.as_deref(), Some("a free-text annotation"));
}

#[test]
fn coverage_period_ordering_is_enforced() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    let july = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let august = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

    insert_coverage_with_period(&pool, &fixture, july, august)
        .expect("a correctly ordered half-open period must be accepted");

    for (label, start, end) in [("inverted", august, july), ("empty", july, july)] {
        let result = insert_coverage_with_period(&pool, &fixture, start, end);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "an {label} period must be rejected because period_end > period_start, got {result:?}"
        );
    }
}

#[test]
fn coverage_status_and_boolean_fields_are_non_null() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    for (column, value_sql) in [
        ("coverage_status", "NULL"),
        ("country_coverage", "NULL"),
        ("institution_coverage", "NULL"),
    ] {
        let result = sql_query(format!(
            "INSERT INTO metric_coverage \
                 (source_account_id, import_id, platform_id, measure_id, \
                  period_start, period_end, coverage_status, country_coverage, \
                  institution_coverage) \
             VALUES ($1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', \
                     {coverage_status}, {country_coverage}, {institution_coverage})",
            coverage_status = if column == "coverage_status" {
                value_sql
            } else {
                "'UNKNOWN'"
            },
            country_coverage = if column == "country_coverage" {
                value_sql
            } else {
                "false"
            },
            institution_coverage = if column == "institution_coverage" {
                value_sql
            } else {
                "false"
            },
        ))
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.import_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "a NULL {column} must be rejected by a NOT NULL constraint, got {result:?}"
        );
    }
}

#[test]
fn invalid_coverage_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");
    let unknown = Uuid::new_v4();

    for (label, source_account_id, import_id, platform_id, measure_id) in [
        (
            "source account",
            unknown,
            fixture.import_id,
            fixture.platform_id,
            fixture.measure_id,
        ),
        (
            "import",
            fixture.source_account_id,
            unknown,
            fixture.platform_id,
            fixture.measure_id,
        ),
        (
            "platform",
            fixture.source_account_id,
            fixture.import_id,
            unknown,
            fixture.measure_id,
        ),
        (
            "measure",
            fixture.source_account_id,
            fixture.import_id,
            fixture.platform_id,
            unknown,
        ),
    ] {
        let result = sql_query(
            "INSERT INTO metric_coverage \
                 (source_account_id, import_id, platform_id, measure_id, \
                  period_start, period_end, coverage_status, country_coverage, \
                  institution_coverage) \
             VALUES ($1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', \
                     'UNKNOWN', false, false)",
        )
        .bind::<diesel::sql_types::Uuid, _>(source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .bind::<diesel::sql_types::Uuid, _>(measure_id)
        .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "an unknown {label} must be rejected, got {result:?}"
        );
    }
}

#[test]
fn deleting_a_referenced_entity_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_coverage_fixture(&pool);
    insert_coverage_row(&pool, &fixture, None, "UNKNOWN")
        .expect("Failed to insert the referencing coverage row");

    for (table, id_column, id) in [
        (
            "metric_source_account",
            "source_account_id",
            fixture.source_account_id,
        ),
        ("metric_import", "import_id", fixture.import_id),
        ("metric_platform", "platform_id", fixture.platform_id),
        ("metric_measure", "measure_id", fixture.measure_id),
    ] {
        let result = delete_row(&pool, table, id_column, id);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "deleting a referenced {table} must be restricted, not cascade away \
             coverage history, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_coverage)"),
        1,
        "the coverage row must survive the restricted deletions"
    );
}

#[test]
fn metric_coverage_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // The set is closed: no coverage uniqueness rule beyond the primary key
    // may exist at this foundation stage.
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(&pool, "metric_coverage"),
        vec!["metric_coverage_period_check"],
        "metric_coverage must carry exactly the one authorized CHECK constraint"
    );
}

#[test]
fn metric_coverage_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_coverage");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_coverage_import_id_fkey",
            "metric_coverage_measure_id_fkey",
            "metric_coverage_platform_id_fkey",
            "metric_coverage_source_account_id_fkey",
        ],
        "metric_coverage must carry exactly the four authorized foreign keys"
    );
    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour: {definition}"
        );
    }
}

#[test]
fn metric_coverage_has_no_index_beyond_its_primary_key() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_coverage"),
        vec!["metric_coverage_pkey"],
        "metric_coverage must carry exactly its primary key and no speculative \
         secondary or uniqueness index"
    );
}

#[test]
fn reverting_through_the_coverage_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    let count_objects = |connection: &mut PgConnection, query: &str| -> i64 {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
            .get_result(connection)
            .expect("Failed to count schema objects")
    };
    let measure_seeds = |connection: &mut PgConnection| -> Vec<String> {
        #[derive(diesel::QueryableByName)]
        struct SeedRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            row: String,
        }
        sql_query(
            "SELECT row_to_json(metric_measure)::text AS row FROM metric_measure \
             WHERE code IN ('title_sessions', 'net_units') ORDER BY code",
        )
        .load::<SeedRow>(connection)
        .expect("Failed to snapshot the metric_measure seed rows")
        .into_iter()
        .map(|seed| seed.row)
        .collect()
    };
    let seeds_before = measure_seeds(&mut connection);
    assert_eq!(seeds_before.len(), 2, "both measure seeds must exist");

    revert_through_coverage_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace AND relname = 'metric_coverage')",
        ),
        0,
        "the downgrade must drop the MET-WP1-05 table"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace AND typname = 'metric_coverage_status')",
        ),
        0,
        "the downgrade must drop the MET-WP1-05 enum type"
    );

    // ...while every earlier Metrics slice survives, together with the exact
    // measure seeds.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint', \
                                'metric_import', 'metric_import_error', 'metric_record', \
                                'metric_record_revision', 'metric_record_provenance'))",
        ),
        11,
        "the downgrade must leave the MET-WP1-01/02/03/04 schema in place"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "the downgrade must leave the measure seeds byte-identical"
    );

    // Reapplication recreates the enum and the empty table.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the coverage migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace AND typname = 'metric_coverage_status')",
        ),
        1,
        "reapplication must recreate the enum type cleanly"
    );
    assert_eq!(
        count_objects(&mut connection, "(SELECT COUNT(*) FROM metric_coverage)",),
        0,
        "reapplication must seed no coverage row"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "reapplication must leave the measure seeds byte-identical"
    );
}
