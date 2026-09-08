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

use super::crud::{create_metric_platform, metric_platform_by_code, update_metric_platform};
use super::{MetricPlatform, MetricPlatformOwnershipClass, NewMetricPlatform, PatchMetricPlatform};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::{setup_test_db, test_db_url, TestDbGuard};
use crate::schema::metric_platform;
use thoth_errors::ThothError;

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

// --------------------------------------------------------------------------
// `MET-WP1-12` protected administration coordinator
// --------------------------------------------------------------------------

/// Diesel join constructs that would turn a single-row `FOR UPDATE` into a
/// multi-table one. Amendment 2 prohibits a joined multi-table lock in every
/// Metrics registry coordinator, so each coordinator's source is checked for
/// all of them.
pub(crate) const FORBIDDEN_JOIN_CONSTRUCTS: [&str; 4] =
    ["inner_join", "left_join", "left_outer_join", "joinable!"];

/// One audit row, read back as plain scalars.
///
/// The audit table is read here through raw SQL rather than through a model
/// helper, because no read path for it exists in production code and this slice
/// deliberately does not add one.
#[derive(diesel::QueryableByName, Debug)]
pub(crate) struct AuditRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) entity: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) entity_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) action: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) actor: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    pub(crate) before_state: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub(crate) after_state: serde_json::Value,
}

/// Every audit row, oldest first.
pub(crate) fn audit_rows(pool: &PgPool) -> Vec<AuditRow> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT entity::text AS entity, entity_id, action::text AS action, actor, \
                before_state, after_state \
         FROM metric_registry_history ORDER BY created_at, metric_registry_history_id",
    )
    .load::<AuditRow>(&mut connection)
    .expect("Failed to read audit rows")
}

fn new_platform(code: &str) -> NewMetricPlatform {
    NewMetricPlatform {
        code: code.to_string(),
        display_name: format!("Platform {code}"),
        ownership_class: MetricPlatformOwnershipClass::External,
        enabled: true,
        public_description: Some("Initial description".to_string()),
    }
}

fn patch_platform(code: &str, display_name: &str, enabled: bool) -> PatchMetricPlatform {
    PatchMetricPlatform {
        code: code.to_string(),
        display_name: display_name.to_string(),
        enabled,
        public_description: Some("Initial description".to_string()),
    }
}

#[test]
fn create_persists_the_exact_values_and_audits_the_persisted_row() {
    let (_guard, pool) = setup_registry_db();

    let created = create_metric_platform(&pool, "actor-1", &new_platform("cloudfront_demo"))
        .expect("create must succeed");

    assert_eq!(created.code, "cloudfront_demo");
    assert_eq!(created.display_name, "Platform cloudfront_demo");
    assert_eq!(
        created.ownership_class,
        MetricPlatformOwnershipClass::External
    );
    assert!(created.enabled);
    assert_eq!(
        created.public_description.as_deref(),
        Some("Initial description")
    );
    // Database-owned values are present and were not supplied by the caller.
    assert_ne!(created.platform_id, Uuid::nil());
    assert_eq!(created.created_at, created.updated_at);

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 1, "exactly one audit row per creation");
    assert_eq!(audit[0].entity, "PLATFORM");
    assert_eq!(audit[0].action, "CREATE");
    assert_eq!(audit[0].actor, "actor-1");
    assert_eq!(audit[0].entity_id, created.platform_id);
    assert!(
        audit[0].before_state.is_none(),
        "a CREATE entry records no previous state"
    );
    // The after state is the exact persisted row, including the generated id
    // and the database-authored timestamps.
    assert_eq!(
        audit[0].after_state,
        serde_json::to_value(&created).expect("serialize persisted row")
    );
}

#[test]
fn a_code_is_stored_and_matched_exactly_with_no_normalisation() {
    let (_guard, pool) = setup_registry_db();

    // Codes that differ only by case or by surrounding/inner whitespace are
    // distinct codes. PostgreSQL accepts each of them under the nonblank CHECK,
    // and nothing in the coordinator folds them together.
    let codes = [
        "Mixed_Case",
        "mixed_case",
        " leading",
        "trailing ",
        "inner space",
        "unícode",
        "UNÍCODE",
    ];
    for code in codes {
        create_metric_platform(&pool, "actor-1", &new_platform(code))
            .unwrap_or_else(|error| panic!("create `{code}` must succeed: {error:?}"));
    }

    for code in codes {
        let found = metric_platform_by_code(&pool, code)
            .unwrap_or_else(|error| panic!("lookup `{code}` must succeed: {error:?}"));
        assert_eq!(found.code, code, "the stored code must be byte-identical");
    }

    // A variant that was never created is not found by folding onto one that
    // was: no trimming, case folding, ILIKE or Unicode normalisation.
    for absent in ["MIXED_CASE", "leading", "trailing", "innerspace", "unicode"] {
        assert!(
            matches!(
                metric_platform_by_code(&pool, absent),
                Err(ThothError::EntityNotFound)
            ),
            "`{absent}` must not resolve to a different stored code"
        );
    }

    assert_eq!(
        audit_rows(&pool).len(),
        codes.len(),
        "one audit row per created platform, and none for a lookup"
    );
}

#[test]
fn a_duplicate_code_fails_atomically_and_is_sanitised() {
    let (_guard, pool) = setup_registry_db();
    create_metric_platform(&pool, "actor-1", &new_platform("duplicate")).expect("first create");

    let error = create_metric_platform(&pool, "actor-1", &new_platform("duplicate"))
        .expect_err("a duplicate code must fail");

    let ThothError::DatabaseConstraintError(message) = &error else {
        panic!("a duplicate code must map to a bounded constraint error, got {error:?}");
    };
    assert_eq!(
        message.as_ref(),
        "A metric platform with this code already exists."
    );
    for leaked in [
        "metric_platform_code_key",
        "INSERT INTO",
        "DETAIL",
        "pg_",
        "duplicate key",
    ] {
        assert!(
            !message.contains(leaked),
            "the message must not leak `{leaked}`: {message}"
        );
    }

    // The rejected mutation left neither a canonical nor an audit row behind.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform)"),
        1
    );
    assert_eq!(audit_rows(&pool).len(), 1);
}

#[test]
fn a_blank_field_is_rejected_by_the_database_and_is_sanitised() {
    let (_guard, pool) = setup_registry_db();

    let mut blank_code = new_platform("x");
    blank_code.code = "   ".to_string();
    let error = create_metric_platform(&pool, "actor-1", &blank_code).expect_err("blank code");
    assert!(
        matches!(&error, ThothError::DatabaseConstraintError(message)
            if message.as_ref() == "Metric platform code must not be an empty string."),
        "got {error:?}"
    );

    let mut blank_name = new_platform("y");
    blank_name.display_name = String::new();
    let error = create_metric_platform(&pool, "actor-1", &blank_name).expect_err("blank name");
    assert!(
        matches!(&error, ThothError::DatabaseConstraintError(message)
            if message.as_ref() == "Metric platform display name must not be an empty string."),
        "got {error:?}"
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform)"),
        0
    );
    assert!(audit_rows(&pool).is_empty());
}

#[test]
fn update_replaces_only_the_mutable_fields_and_audits_before_and_after() {
    let (_guard, pool) = setup_registry_db();
    let created =
        create_metric_platform(&pool, "actor-1", &new_platform("target")).expect("create");

    let updated = update_metric_platform(
        &pool,
        "actor-2",
        &patch_platform("target", "Renamed platform", false),
    )
    .expect("update must succeed");

    // Mutable fields changed.
    assert_eq!(updated.display_name, "Renamed platform");
    assert!(!updated.enabled);
    // Immutable fields did not, and the identity is the same row.
    assert_eq!(updated.platform_id, created.platform_id);
    assert_eq!(updated.code, "target");
    assert_eq!(
        updated.ownership_class,
        MetricPlatformOwnershipClass::External
    );
    assert_eq!(updated.created_at, created.created_at);
    assert!(
        updated.updated_at > created.updated_at,
        "a real change must move the database-managed updated_at"
    );

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 2, "one CREATE plus one UPDATE");
    assert_eq!(audit[1].action, "UPDATE");
    assert_eq!(audit[1].actor, "actor-2");
    assert_eq!(audit[1].entity_id, created.platform_id);
    assert_eq!(
        audit[1]
            .before_state
            .as_ref()
            .expect("UPDATE records a before state"),
        &serde_json::to_value(&created).expect("serialize before"),
    );
    assert_eq!(
        audit[1].after_state,
        serde_json::to_value(&updated).expect("serialize after")
    );
}

#[test]
fn an_omitted_nullable_field_stores_sql_null_rather_than_retaining_the_old_value() {
    let (_guard, pool) = setup_registry_db();
    create_metric_platform(&pool, "actor-1", &new_platform("nullable")).expect("create");

    // Patch is a complete replacement: an absent publicDescription means NULL.
    let updated = update_metric_platform(
        &pool,
        "actor-1",
        &PatchMetricPlatform {
            code: "nullable".to_string(),
            display_name: "Platform nullable".to_string(),
            enabled: true,
            public_description: None,
        },
    )
    .expect("update must succeed");

    assert_eq!(updated.public_description, None);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_platform WHERE public_description IS NULL)"
        ),
        1,
        "the column must hold SQL NULL, not an empty string"
    );
    assert_eq!(audit_rows(&pool).len(), 2);
}

#[test]
fn a_true_no_op_update_writes_nothing_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    let created = create_metric_platform(&pool, "actor-1", &new_platform("noop")).expect("create");

    let returned = update_metric_platform(
        &pool,
        "actor-2",
        &patch_platform("noop", "Platform noop", true),
    )
    .expect("a no-op update must succeed");

    // The current persisted row is returned unchanged.
    assert_eq!(returned, created);
    assert_eq!(
        returned.updated_at, created.updated_at,
        "a no-op must not move the canonical timestamp"
    );
    assert_eq!(
        audit_rows(&pool).len(),
        1,
        "a no-op must not create an audit row"
    );
}

#[test]
fn an_unknown_code_is_reported_as_not_found_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();

    assert!(matches!(
        metric_platform_by_code(&pool, "absent"),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        update_metric_platform(&pool, "actor-1", &patch_platform("absent", "x", true)),
        Err(ThothError::EntityNotFound)
    ));
    assert!(audit_rows(&pool).is_empty());
}

#[test]
fn a_failing_audit_write_rolls_back_the_canonical_change() {
    let (_guard, pool) = setup_registry_db();

    // A blank actor violates `metric_registry_history_actor_check`. The
    // canonical INSERT has already succeeded inside the transaction when the
    // audit INSERT fails, so this exercises the real rollback path rather than
    // a pre-check: if the two writes were not one transaction, the platform
    // would survive without its audit evidence.
    let error = create_metric_platform(&pool, "   ", &new_platform("orphan"))
        .expect_err("a blank actor must fail the audit write");
    assert!(
        matches!(&error, ThothError::DatabaseConstraintError(message)
            if message.as_ref() == "Metric registry history actor must not be an empty string."),
        "the audit CHECK must surface bounded, got {error:?}"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_platform WHERE code = 'orphan')"
        ),
        0,
        "the canonical row must not survive a failed audit write"
    );
    assert!(audit_rows(&pool).is_empty());

    // The same rollback holds for UPDATE, after a successful create.
    let created =
        create_metric_platform(&pool, "actor-1", &new_platform("target")).expect("create");
    let error = update_metric_platform(&pool, "\t", &patch_platform("target", "Renamed", false))
        .expect_err("a blank actor must fail the audit write");
    assert!(matches!(error, ThothError::DatabaseConstraintError(_)));
    assert_eq!(
        metric_platform_by_code(&pool, "target").expect("row survives"),
        created,
        "a failed audit write must leave the canonical row exactly as it was"
    );
    assert_eq!(
        audit_rows(&pool).len(),
        1,
        "only the successful create is recorded"
    );
}

#[test]
fn two_competing_updates_serialise_and_their_audit_chain_matches_commit_order() {
    let (_guard, pool) = setup_registry_db();
    let created =
        create_metric_platform(&pool, "actor-1", &new_platform("contended")).expect("create");

    // Two real connections, two real transactions, one contended row.
    let first = {
        let pool = Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_platform(&pool, "actor-a", &patch_platform("contended", "A", true))
        })
    };
    let second = {
        let pool = Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_platform(&pool, "actor-b", &patch_platform("contended", "B", true))
        })
    };
    first.join().expect("thread a").expect("update a");
    second.join().expect("thread b").expect("update b");

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 3, "one CREATE and exactly two UPDATE entries");

    // Serialized last-write-wins: the two updates form a chain. The first
    // update's before state is the created row; the second update's before
    // state is exactly the first update's after state; the final after state is
    // what the table now holds. This holds whichever thread won.
    assert_eq!(
        audit[1].before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize created")
    );
    assert_eq!(
        audit[2].before_state.as_ref().expect("before"),
        &audit[1].after_state,
        "the audit chain must follow actual commit order with no gap"
    );

    let final_row = metric_platform_by_code(&pool, "contended").expect("final read");
    assert_eq!(
        audit[2].after_state,
        serde_json::to_value(&final_row).expect("serialize final"),
        "the last audit entry must describe the committed state"
    );
    assert!(["A", "B"].contains(&final_row.display_name.as_str()));
}

#[test]
fn the_coordinator_takes_exactly_one_application_row_lock() {
    // The lock topology is a specification requirement, so it is asserted
    // against the coordinator source itself: exactly one `for_update()` call,
    // on the canonical `metric_platform` row, and no joined multi-table lock.
    let source = include_str!("crud.rs");

    assert_eq!(
        source.matches(".for_update()").count(),
        1,
        "the platform coordinator must request exactly one row lock"
    );
    for forbidden in FORBIDDEN_JOIN_CONSTRUCTS {
        assert!(
            !source.contains(forbidden),
            "a joined multi-table FOR UPDATE is prohibited: found `{forbidden}`"
        );
    }
    // Whitespace-insensitive: the text between the lock and the statement that
    // opens it must name the canonical platform row and nothing else.
    let (before_lock, after_lock) = source
        .split_once(".for_update()")
        .expect("the coordinator must take one lock");
    let opening: String = before_lock
        .rsplit("let current")
        .next()
        .expect("the lock belongs to the current-state read")
        .split_whitespace()
        .collect();
    assert!(
        opening.contains("metric_platform::table.filter(metric_platform::code.eq(&data.code))"),
        "the one lock must be taken on the canonical metric_platform row selected by exact \
         code, found: {opening}"
    );
    assert!(
        !after_lock.contains("for_update"),
        "no second lock target may follow the canonical row lock"
    );
}
