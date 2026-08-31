//! Focused `MET-WP1-01` database tests for the `metric_platform` registry.
//!
//! [`setup_registry_db`] is shared by every metric registry test module. It
//! restores the pristine post-migration registry state through the embedded
//! Diesel migration harness by reverting migrations in reverse order **until
//! the `MET-WP1-01` registry migration itself has been reverted** and then
//! re-running every pending migration. Later repository migrations are
//! deliberately tolerated: nothing here assumes the registry migration is
//! the newest one, mirroring the durable `revert_through_be04` pattern in
//! `distribution_job/tests.rs`. This bounded targeted revert is still
//! rollback behaviour the repository CLI's `cargo run migrate --revert`
//! (`revert_all_migrations`) cannot evidence.

use std::str::FromStr;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::{MetricPlatform, MetricPlatformOwnershipClass};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::{setup_test_db, test_db_url, TestDbGuard};
use crate::schema::metric_platform;

/// The Diesel migration version of `thoth-api/migrations/20260826_v1.9.0`.
pub(crate) const MET_WP1_01_MIGRATION_VERSION: &str = "20260826";

/// Revert migrations until the `MET-WP1-01` registry migration itself has
/// been reverted.
///
/// A single `revert_last_migration` reverts the registry migration only
/// while it happens to be the newest applied migration; once any later
/// repository migration exists, that call would silently revert the later
/// migration instead. Reverting down to and including the target keeps the
/// meaning under any migration order — the same durable pattern as
/// `revert_through_be04` in `distribution_job/tests.rs` — and the caller's
/// subsequent `run_pending_migrations` re-applies everything in order. No
/// future migration name is assumed or hard-coded.
pub(crate) fn revert_through_registry_migration(connection: &mut PgConnection) {
    let registry_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_01_MIGRATION_VERSION);
    assert!(
        registry_migration_applied,
        "the MET-WP1-01 registry migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_01_MIGRATION_VERSION {
            return;
        }
    }
}

/// A pristine post-migration Metrics registry on the locked test database.
///
/// The shared test harness truncates every table between tests, which also
/// removes the migration-owned `metric_measure` seed rows. This helper
/// re-establishes the exact post-migration registry state by reverting
/// migrations down to and including the `MET-WP1-01` registry migration and
/// then re-running every pending migration through the embedded migration
/// harness. Later migrations are tolerated: they are reverted on the way
/// down and restored by the reapply.
///
/// The revert/reapply cycle drops and recreates the registry enum types, so
/// their PostgreSQL type OIDs change. The returned pool is therefore a fresh
/// dedicated pool created after the reapply: a connection from the long-lived
/// shared pool could still hold the previous OIDs in its type-metadata cache
/// and fail custom-enum binds with a stale-OID lookup error.
pub(crate) fn setup_registry_db() -> (TestDbGuard, Arc<PgPool>) {
    let (guard, _shared_pool) = setup_test_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    revert_through_registry_migration(&mut connection);
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the registry migration onward");
    drop(connection);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("Failed to create a fresh registry test pool");
    (guard, Arc::new(pool))
}

/// One scalar `BIGINT` result, for raw-SQL schema assertions.
pub(crate) fn scalar_i64(pool: &PgPool, query: &str) -> i64 {
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(&mut connection)
        .expect("Failed to run scalar query")
}

/// The ordered labels of one PostgreSQL enum type.
pub(crate) fn enum_labels(pool: &PgPool, type_name: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Label {
        #[diesel(sql_type = diesel::sql_types::Text)]
        label: String,
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "SELECT unnest(enum_range(NULL::public.{type_name}))::text AS label"
    ))
    .load::<Label>(&mut connection)
    .expect("Failed to read enum labels")
    .into_iter()
    .map(|row| row.label)
    .collect()
}

/// Insert one `metric_platform` row with an explicit id through raw SQL.
pub(crate) fn insert_platform_row(pool: &PgPool, platform_id: Uuid, code: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_platform \
             (platform_id, code, display_name, ownership_class, enabled) \
         VALUES ($1, $2, $3, 'EXTERNAL', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(format!("Platform {code}"))
    .execute(&mut connection)
    .expect("Failed to insert metric_platform fixture row");
}

fn insert_platform_raw(
    pool: &PgPool,
    code: &str,
    display_name: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ($1, $2, 'THOTH_MANAGED', TRUE)",
    )
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(display_name)
    .execute(&mut connection)
}

#[test]
fn ownership_class_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_platform_ownership_class"),
        ["THOTH_MANAGED", "PUBLISHER_CONTROLLED", "EXTERNAL"]
    );
}

#[test]
fn migration_seeds_no_platform_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform)"),
        0,
        "MET-WP1-01 must not seed any metric_platform row"
    );
}

#[test]
fn duplicate_platform_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    insert_platform_raw(&pool, "test_platform", "Test platform").expect("First insert must pass");
    let duplicate = insert_platform_raw(&pool, "test_platform", "Different display name");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "duplicate platform code must fail the unique constraint: {duplicate:?}"
    );
}

#[test]
fn blank_platform_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_platform_raw(&pool, blank, "Display name");
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank platform code {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn blank_platform_display_name_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_platform_raw(&pool, "test_platform", blank);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank platform display name {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn platform_updated_at_is_maintained_by_the_repository_standard_trigger() {
    let (_guard, pool) = setup_registry_db();
    insert_platform_raw(&pool, "test_platform", "Test platform").expect("Insert must pass");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let initial: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM metric_platform \
          WHERE code = 'test_platform' AND created_at = updated_at)",
    ))
    .get_result(&mut connection)
    .expect("Failed to read initial timestamps");
    assert_eq!(
        initial, 1,
        "a fresh row starts with created_at = updated_at"
    );

    sql_query("UPDATE metric_platform SET enabled = FALSE WHERE code = 'test_platform'")
        .execute(&mut connection)
        .expect("Failed to update metric_platform row");

    let advanced: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM metric_platform \
          WHERE code = 'test_platform' AND updated_at > created_at)",
    ))
    .get_result(&mut connection)
    .expect("Failed to read updated timestamps");
    assert_eq!(
        advanced, 1,
        "the set_updated_at trigger must advance updated_at on update"
    );
}

const OWNERSHIP_CLASSES: [(MetricPlatformOwnershipClass, &str); 3] = [
    (MetricPlatformOwnershipClass::ThothManaged, "THOTH_MANAGED"),
    (
        MetricPlatformOwnershipClass::PublisherControlled,
        "PUBLISHER_CONTROLLED",
    ),
    (MetricPlatformOwnershipClass::External, "EXTERNAL"),
];

#[test]
fn ownership_class_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in OWNERSHIP_CLASSES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(
            MetricPlatformOwnershipClass::from_str(label).unwrap(),
            variant
        );
        let json = format!("\"{label}\"");
        assert_eq!(serde_json::to_string(&variant).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<MetricPlatformOwnershipClass>(&json).unwrap(),
            variant
        );
    }
    assert!(MetricPlatformOwnershipClass::from_str("OTHER").is_err());
    assert!(MetricPlatformOwnershipClass::from_str("thoth_managed").is_err());
}

#[test]
fn every_ownership_class_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in OWNERSHIP_CLASSES {
        assert_db_enum_roundtrip::<
            MetricPlatformOwnershipClass,
            crate::schema::sql_types::MetricPlatformOwnershipClass,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_platform_ownership_class"),
            variant,
        );
    }
}

#[test]
fn metric_platform_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    let described_id: Uuid = diesel::insert_into(metric_platform::table)
        .values((
            metric_platform::code.eq("described_platform"),
            metric_platform::display_name.eq("Described platform"),
            metric_platform::ownership_class.eq(MetricPlatformOwnershipClass::ThothManaged),
            metric_platform::enabled.eq(true),
            metric_platform::public_description.eq("A platform with a public description."),
        ))
        .returning(metric_platform::platform_id)
        .get_result(&mut connection)
        .expect("Failed to insert described platform row");
    diesel::insert_into(metric_platform::table)
        .values((
            metric_platform::code.eq("undescribed_platform"),
            metric_platform::display_name.eq("Undescribed platform"),
            metric_platform::ownership_class.eq(MetricPlatformOwnershipClass::External),
            metric_platform::enabled.eq(false),
        ))
        .execute(&mut connection)
        .expect("Failed to insert undescribed platform row");

    let described: MetricPlatform = metric_platform::table
        .filter(metric_platform::code.eq("described_platform"))
        .first(&mut connection)
        .expect("Failed to load described platform row");
    assert_eq!(described.platform_id, described_id);
    assert_eq!(described.code, "described_platform");
    assert_eq!(described.display_name, "Described platform");
    assert_eq!(
        described.ownership_class,
        MetricPlatformOwnershipClass::ThothManaged
    );
    assert!(described.enabled);
    assert_eq!(
        described.public_description.as_deref(),
        Some("A platform with a public description.")
    );

    let undescribed: MetricPlatform = metric_platform::table
        .filter(metric_platform::code.eq("undescribed_platform"))
        .first(&mut connection)
        .expect("Failed to load undescribed platform row");
    assert_eq!(
        undescribed.ownership_class,
        MetricPlatformOwnershipClass::External
    );
    assert!(!undescribed.enabled);
    assert_eq!(undescribed.public_description, None);
}

#[test]
fn reverting_through_the_registry_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();

    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database");
    revert_through_registry_migration(&mut connection);

    // In the reverted state the MET-WP1-01 objects are gone.
    let registry_objects: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_class \
          WHERE relnamespace = 'public'::regnamespace \
            AND relname IN ('metric_platform', 'metric_measure', 'metric_platform_measure'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count registry tables");
    assert_eq!(
        registry_objects, 0,
        "the registry downgrade must drop all three registry tables"
    );
    let registry_types: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_type \
          WHERE typnamespace = 'public'::regnamespace \
            AND typname IN ('metric_platform_ownership_class', 'metric_measure_category', \
                            'metric_measure_unit', 'metric_reporting_grain'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count registry enum types");
    assert_eq!(
        registry_types, 0,
        "the registry downgrade must drop all four registry enum types"
    );
    let pre_existing_tables: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM pg_class \
          WHERE relnamespace = 'public'::regnamespace \
            AND relname IN ('publisher', 'work', 'publication', 'institution'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count pre-existing tables");
    assert_eq!(
        pre_existing_tables, 4,
        "the registry downgrade must leave pre-existing tables in place"
    );
    let latest_after_revert = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .map(ToString::to_string)
        .max()
        .expect("No migrations are applied to the test database");
    assert!(
        latest_after_revert.as_str() < MET_WP1_01_MIGRATION_VERSION,
        "after reverting through the registry migration the ledger must contain \
         neither the MET-WP1-01 version nor any later version (found \
         {latest_after_revert})"
    );

    // Reapplication restores the registry and its exact seeds.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the registry migration onward");
    let seeds: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM metric_measure \
          WHERE code IN ('title_sessions', 'net_units'))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count restored seed rows");
    assert_eq!(seeds, 2, "reapplication must restore exactly the two seeds");
    let empty_registries: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "((SELECT COUNT(*) FROM metric_platform) \
          + (SELECT COUNT(*) FROM metric_platform_measure))",
    ))
    .get_result(&mut connection)
    .expect("Failed to count restored registry rows");
    assert_eq!(
        empty_registries, 0,
        "reapplication must seed no platform and no platform-measure row"
    );
}
