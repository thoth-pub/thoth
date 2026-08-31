//! Focused `MET-WP1-02` database tests for the `metric_source` acquisition
//! registry.
//!
//! These tests reuse [`setup_registry_db`] from `metric_platform::tests`: it
//! reverts migrations down to and including the `MET-WP1-01` registry
//! migration and reapplies every pending migration, which restores the
//! pristine post-migration state of the `MET-WP1-02` source-state schema as
//! well. [`revert_through_source_state_migration`] additionally supports
//! targeted rollback evidence for the `MET-WP1-02` migration itself without
//! assuming it remains the newest migration forever.

use std::str::FromStr;

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::{MetricSource, MetricSourceAcquisitionType};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::schema::metric_source;

/// The Diesel migration version of `thoth-api/migrations/20260827_v1.9.0`.
pub(crate) const MET_WP1_02_MIGRATION_VERSION: &str = "20260827";

/// Revert migrations until the `MET-WP1-02` source-state migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_registry_migration`: a single
/// `revert_last_migration` reverts this migration only while it happens to be
/// the newest applied migration; reverting down to and including the target
/// keeps the meaning under any later migration order, and the caller's
/// subsequent `run_pending_migrations` re-applies everything in order.
pub(crate) fn revert_through_source_state_migration(connection: &mut PgConnection) {
    let source_state_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_02_MIGRATION_VERSION);
    assert!(
        source_state_migration_applied,
        "the MET-WP1-02 source-state migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_02_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one `metric_source` row with an explicit id through raw SQL.
pub(crate) fn insert_source_row(pool: &PgPool, source_id: Uuid, code: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source (source_id, code, acquisition_type, enabled) \
         VALUES ($1, $2, 'ADMIN_IMPORT', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Text, _>(code)
    .execute(&mut connection)
    .expect("Failed to insert metric_source fixture row");
}

/// Insert one source row whose optional day columns are SQL literals.
fn insert_source_raw(
    pool: &PgPool,
    code: &str,
    lookback_sql: &str,
    finalization_sql: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_source \
             (code, acquisition_type, enabled, default_lookback_days, \
              default_finalization_delay_days) \
         VALUES ($1, 'DRIVER', TRUE, {lookback_sql}, {finalization_sql})"
    ))
    .bind::<diesel::sql_types::Text, _>(code)
    .execute(&mut connection)
}

#[test]
fn acquisition_type_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_source_acquisition_type"),
        ["DRIVER", "PUBLISHER_UPLOAD", "OPERAS", "ADMIN_IMPORT"]
    );
}

#[test]
fn migration_seeds_no_source_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source)"),
        0,
        "MET-WP1-02 must not seed any metric_source row"
    );
}

#[test]
fn source_deliberately_has_no_timestamp_columns() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND table_name = 'metric_source' \
                AND column_name IN ('created_at', 'updated_at'))",
        ),
        0,
        "the approved design deliberately omits timestamps on metric_source"
    );
}

#[test]
fn source_state_primary_keys_use_the_repository_standard_uuid_default() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND ((table_name = 'metric_source' \
                      AND column_name = 'source_id') \
                  OR (table_name = 'metric_source_account' \
                      AND column_name = 'source_account_id') \
                  OR (table_name = 'metric_source_checkpoint' \
                      AND column_name = 'source_checkpoint_id')) \
                AND column_default LIKE '%uuid_generate_v4()%')",
        ),
        3,
        "all three source-state primary keys must default to uuid_generate_v4()"
    );
}

#[test]
fn duplicate_source_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    insert_source_raw(&pool, "test_source", "NULL", "NULL").expect("First insert must pass");
    let duplicate = insert_source_raw(&pool, "test_source", "NULL", "NULL");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "duplicate source code must fail the unique constraint: {duplicate:?}"
    );
}

#[test]
fn blank_source_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_source_raw(&pool, blank, "NULL", "NULL");
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank source code {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn negative_day_defaults_are_rejected_and_non_negative_values_accepted() {
    let (_guard, pool) = setup_registry_db();
    for (code, lookback, finalization) in [
        ("negative_lookback", "-1", "NULL"),
        ("negative_finalization", "NULL", "-1"),
        ("very_negative_lookback", "-30", "0"),
    ] {
        let result = insert_source_raw(&pool, code, lookback, finalization);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "negative day values ({lookback}, {finalization}) must fail the check \
             constraint: {result:?}"
        );
    }
    for (code, lookback, finalization) in [
        ("zero_days", "0", "0"),
        ("positive_days", "30", "14"),
        ("unset_days", "NULL", "NULL"),
    ] {
        insert_source_raw(&pool, code, lookback, finalization).unwrap_or_else(|error| {
            panic!("non-negative day values ({lookback}, {finalization}) must pass: {error:?}")
        });
    }
}

const ACQUISITION_TYPES: [(MetricSourceAcquisitionType, &str); 4] = [
    (MetricSourceAcquisitionType::Driver, "DRIVER"),
    (
        MetricSourceAcquisitionType::PublisherUpload,
        "PUBLISHER_UPLOAD",
    ),
    (MetricSourceAcquisitionType::Operas, "OPERAS"),
    (MetricSourceAcquisitionType::AdminImport, "ADMIN_IMPORT"),
];

#[test]
fn acquisition_type_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in ACQUISITION_TYPES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(
            MetricSourceAcquisitionType::from_str(label).unwrap(),
            variant
        );
        let json = format!("\"{label}\"");
        assert_eq!(serde_json::to_string(&variant).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<MetricSourceAcquisitionType>(&json).unwrap(),
            variant
        );
    }
    assert!(MetricSourceAcquisitionType::from_str("OTHER").is_err());
    assert!(MetricSourceAcquisitionType::from_str("driver").is_err());
}

#[test]
fn every_acquisition_type_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in ACQUISITION_TYPES {
        assert_db_enum_roundtrip::<
            MetricSourceAcquisitionType,
            crate::schema::sql_types::MetricSourceAcquisitionType,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_source_acquisition_type"),
            variant,
        );
    }
}

#[test]
fn metric_source_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    let driven_id: Uuid = diesel::insert_into(metric_source::table)
        .values((
            metric_source::code.eq("driven_source"),
            metric_source::acquisition_type.eq(MetricSourceAcquisitionType::Driver),
            metric_source::driver_key.eq("some_driver"),
            metric_source::enabled.eq(true),
            metric_source::default_lookback_days.eq(30),
            metric_source::default_finalization_delay_days.eq(0),
        ))
        .returning(metric_source::source_id)
        .get_result(&mut connection)
        .expect("Failed to insert driven source row");
    diesel::insert_into(metric_source::table)
        .values((
            metric_source::code.eq("uploaded_source"),
            metric_source::acquisition_type.eq(MetricSourceAcquisitionType::PublisherUpload),
            metric_source::enabled.eq(false),
        ))
        .execute(&mut connection)
        .expect("Failed to insert uploaded source row");

    let driven: MetricSource = metric_source::table
        .filter(metric_source::code.eq("driven_source"))
        .first(&mut connection)
        .expect("Failed to load driven source row");
    assert_eq!(driven.source_id, driven_id);
    assert_eq!(driven.code, "driven_source");
    assert_eq!(driven.acquisition_type, MetricSourceAcquisitionType::Driver);
    assert_eq!(driven.driver_key.as_deref(), Some("some_driver"));
    assert!(driven.enabled);
    assert_eq!(driven.default_lookback_days, Some(30));
    assert_eq!(driven.default_finalization_delay_days, Some(0));

    let uploaded: MetricSource = metric_source::table
        .filter(metric_source::code.eq("uploaded_source"))
        .first(&mut connection)
        .expect("Failed to load uploaded source row");
    assert_eq!(
        uploaded.acquisition_type,
        MetricSourceAcquisitionType::PublisherUpload
    );
    assert_eq!(uploaded.driver_key, None);
    assert!(!uploaded.enabled);
    assert_eq!(uploaded.default_lookback_days, None);
    assert_eq!(uploaded.default_finalization_delay_days, None);
}

#[test]
fn reverting_through_the_source_state_migration_removes_it_and_leaves_the_registry_intact() {
    let (_guard, _pool) = setup_registry_db();

    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database");

    // The two MET-WP1-01 measure seeds, captured in full before the targeted
    // revert so byte-level preservation can be asserted afterwards.
    let seed_snapshot = |connection: &mut PgConnection| -> Vec<String> {
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
    let seeds_before = seed_snapshot(&mut connection);
    assert_eq!(seeds_before.len(), 2, "both measure seeds must exist");

    revert_through_source_state_migration(&mut connection);

    // In the reverted state the MET-WP1-02 objects are gone...
    let source_state_tables: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_class \
          WHERE relnamespace = 'public'::regnamespace \
            AND relname IN ('metric_source', 'metric_source_account', \
                            'metric_source_checkpoint'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count source-state tables");
    assert_eq!(
        source_state_tables, 0,
        "the source-state downgrade must drop all three source-state tables"
    );
    let acquisition_types: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_type \
          WHERE typnamespace = 'public'::regnamespace \
            AND typname = 'metric_source_acquisition_type')",
    ))
    .get_result(&mut connection)
    .expect("Failed to count the acquisition enum type");
    assert_eq!(
        acquisition_types, 0,
        "the source-state downgrade must drop the acquisition enum type"
    );

    // ...while the MET-WP1-01 registry schema and its exact seeds survive.
    let registry_tables: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_class \
          WHERE relnamespace = 'public'::regnamespace \
            AND relname IN ('metric_platform', 'metric_measure', 'metric_platform_measure'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count registry tables");
    assert_eq!(
        registry_tables, 3,
        "the source-state downgrade must leave the MET-WP1-01 registry tables in place"
    );
    assert_eq!(
        seed_snapshot(&mut connection),
        seeds_before,
        "the source-state downgrade must leave the measure seeds byte-identical"
    );

    // Reapplication restores the empty source-state schema and leaves the
    // seeds byte-identical.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the source-state migration onward");
    let empty_source_state: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "((SELECT COUNT(*) FROM metric_source) \
          + (SELECT COUNT(*) FROM metric_source_account) \
          + (SELECT COUNT(*) FROM metric_source_checkpoint))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count restored source-state rows");
    assert_eq!(
        empty_source_state, 0,
        "reapplication must seed no source, account or checkpoint row"
    );
    assert_eq!(
        seed_snapshot(&mut connection),
        seeds_before,
        "reapplication must leave the measure seeds byte-identical"
    );
}
