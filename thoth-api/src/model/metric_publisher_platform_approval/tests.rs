//! Focused `MET-WP1-06` database tests for
//! `metric_publisher_platform_approval`: the closed approval-status enum,
//! the approved field/default contract, the complete authorized
//! foreign-key/uniqueness/index inventory, the non-null usage/sales/status
//! columns, the nullable `approved_by`/`approved_at`/`notes` columns and the
//! targeted revert/reapply of the publisher-platform-approval foundation
//! migration.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! implements no approval creation/transition/revocation service, no
//! `PUBLISHER_CONTROLLED` platform-ownership enforcement, no
//! package/capability entitlement check, no publisher-import authorization
//! and no GraphQL/admin surface, and nothing here pretends otherwise.

use std::str::FromStr;

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::{MetricPublisherPlatformApproval, MetricPublisherPlatformApprovalStatus};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::insert_publisher_row;
use crate::model::metric_platform::tests::{
    enum_labels, insert_platform_row, scalar_i64, setup_registry_db,
};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_publisher_platform_approval;

/// The Diesel migration version of `thoth-api/migrations/20260902_v1.9.0`.
const MET_WP1_06_MIGRATION_VERSION: &str = "20260902";

const APPROVAL_STATUSES: [(MetricPublisherPlatformApprovalStatus, &str); 3] = [
    (MetricPublisherPlatformApprovalStatus::Pending, "PENDING"),
    (MetricPublisherPlatformApprovalStatus::Approved, "APPROVED"),
    (MetricPublisherPlatformApprovalStatus::Revoked, "REVOKED"),
];

/// Revert migrations until the `MET-WP1-06` approval-foundation migration
/// itself has been reverted.
///
/// The same durable pattern as the earlier WP1 slices: a single
/// `revert_last_migration` would only mean "the approval migration" while it
/// happens to be the newest applied migration. Reverting down to and
/// including the target keeps the meaning under any later migration order,
/// and no future migration name is assumed or hard-coded.
fn revert_through_approval_migration(connection: &mut PgConnection) {
    let approval_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_06_MIGRATION_VERSION);
    assert!(
        approval_migration_applied,
        "the MET-WP1-06 approval migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_06_MIGRATION_VERSION {
            return;
        }
    }
}

/// The publisher/platform entities one approval row must resolve to.
struct ApprovalFixture {
    publisher_id: Uuid,
    platform_id: Uuid,
}

/// Insert one referenced publisher/platform pair and return their ids.
fn insert_approval_fixture(pool: &PgPool) -> ApprovalFixture {
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(pool, publisher_id);
    let platform_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, "approval_test_platform");
    ApprovalFixture {
        publisher_id,
        platform_id,
    }
}

/// Insert one minimal approval row through raw SQL so the database defaults
/// are exercised rather than restated by a Diesel fixture.
fn insert_approval_row(
    pool: &PgPool,
    publisher_id: Uuid,
    platform_id: Uuid,
    status: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_publisher_platform_approval \
             (publisher_id, platform_id, usage_submission_enabled, \
              sales_submission_enabled, approval_status) \
         VALUES ($1, $2, false, false, '{status}')"
    ))
    .bind::<diesel::sql_types::Uuid, _>(publisher_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .execute(&mut connection)
}

fn delete_row(pool: &PgPool, table: &str, id_column: &str, id: Uuid) -> Result<usize, DieselError> {
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
fn approval_status_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_publisher_platform_approval_status"),
        vec!["PENDING", "APPROVED", "REVOKED"],
        "metric_publisher_platform_approval_status must carry exactly the three approved labels"
    );
}

#[test]
fn approval_status_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in APPROVAL_STATUSES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(
            MetricPublisherPlatformApprovalStatus::from_str(label).unwrap(),
            variant
        );
    }
    assert!(MetricPublisherPlatformApprovalStatus::from_str("OTHER").is_err());
    assert!(MetricPublisherPlatformApprovalStatus::from_str("ACTIVE").is_err());
    assert!(MetricPublisherPlatformApprovalStatus::from_str("pending").is_err());
}

#[test]
fn every_approval_status_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in APPROVAL_STATUSES {
        assert_db_enum_roundtrip::<
            MetricPublisherPlatformApprovalStatus,
            crate::schema::sql_types::MetricPublisherPlatformApprovalStatus,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_publisher_platform_approval_status"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_approval_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_publisher_platform_approval)"
        ),
        0,
        "MET-WP1-06 must not seed any metric_publisher_platform_approval row"
    );
}

#[test]
fn approval_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let approved_at = Timestamp::parse_from_rfc3339("2026-09-02T11:45:00Z").unwrap();
    let approval_id: Uuid = diesel::insert_into(metric_publisher_platform_approval::table)
        .values((
            metric_publisher_platform_approval::publisher_id.eq(fixture.publisher_id),
            metric_publisher_platform_approval::platform_id.eq(fixture.platform_id),
            metric_publisher_platform_approval::usage_submission_enabled.eq(true),
            metric_publisher_platform_approval::sales_submission_enabled.eq(false),
            metric_publisher_platform_approval::approval_status
                .eq(MetricPublisherPlatformApprovalStatus::Approved),
            metric_publisher_platform_approval::approved_by.eq(Some(Uuid::new_v4())),
            metric_publisher_platform_approval::approved_at.eq(Some(approved_at)),
            metric_publisher_platform_approval::notes.eq(Some("some notes")),
        ))
        .returning(metric_publisher_platform_approval::publisher_platform_approval_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated approval row");

    let loaded: MetricPublisherPlatformApproval = metric_publisher_platform_approval::table
        .filter(metric_publisher_platform_approval::publisher_platform_approval_id.eq(approval_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated approval row");
    assert_eq!(loaded.publisher_platform_approval_id, approval_id);
    assert_eq!(loaded.publisher_id, fixture.publisher_id);
    assert_eq!(loaded.platform_id, fixture.platform_id);
    assert!(loaded.usage_submission_enabled);
    assert!(!loaded.sales_submission_enabled);
    assert_eq!(
        loaded.approval_status,
        MetricPublisherPlatformApprovalStatus::Approved
    );
    assert!(loaded.approved_by.is_some());
    assert_eq!(
        loaded.approved_at,
        Some(approved_at),
        "a non-null approved_at must round-trip through PostgreSQL/Diesel unchanged"
    );
    assert_eq!(loaded.notes.as_deref(), Some("some notes"));
}

#[test]
fn usage_and_sales_submission_flags_are_independently_representable() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // The usage-only combination is covered by the complete round-trip above.
    // The approved design keeps the two flags independent, so the sales-only
    // combination must be representable on its own: an approval may permit
    // sales submissions without permitting usage submissions.
    let approval_id: Uuid = diesel::insert_into(metric_publisher_platform_approval::table)
        .values((
            metric_publisher_platform_approval::publisher_id.eq(fixture.publisher_id),
            metric_publisher_platform_approval::platform_id.eq(fixture.platform_id),
            metric_publisher_platform_approval::usage_submission_enabled.eq(false),
            metric_publisher_platform_approval::sales_submission_enabled.eq(true),
            metric_publisher_platform_approval::approval_status
                .eq(MetricPublisherPlatformApprovalStatus::Approved),
        ))
        .returning(metric_publisher_platform_approval::publisher_platform_approval_id)
        .get_result(&mut connection)
        .expect("Failed to insert the sales-only approval row");

    let loaded: MetricPublisherPlatformApproval = metric_publisher_platform_approval::table
        .filter(metric_publisher_platform_approval::publisher_platform_approval_id.eq(approval_id))
        .first(&mut connection)
        .expect("Failed to load the sales-only approval row");
    assert!(
        !loaded.usage_submission_enabled,
        "the sales-only combination must keep usage_submission_enabled false"
    );
    assert!(
        loaded.sales_submission_enabled,
        "the sales-only combination must keep sales_submission_enabled true"
    );
}

#[test]
fn approval_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    insert_approval_row(&pool, fixture.publisher_id, fixture.platform_id, "PENDING")
        .expect("Failed to insert the defaulted approval row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricPublisherPlatformApproval = metric_publisher_platform_approval::table
        .first(&mut connection)
        .expect("Failed to load the defaulted approval row");
    assert_ne!(
        loaded.publisher_platform_approval_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a publisher_platform_approval_id"
    );
    assert_eq!(
        loaded.approved_by, None,
        "approved_by must default to NULL when not supplied"
    );
    assert_eq!(
        loaded.approved_at, None,
        "approved_at must default to NULL when not supplied"
    );
    assert_eq!(
        loaded.notes, None,
        "notes is an optional annotation and must default to NULL"
    );
}

#[test]
fn nullable_audit_and_notes_fields_round_trip() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    insert_approval_row(&pool, fixture.publisher_id, fixture.platform_id, "PENDING")
        .expect("Failed to insert the approval row with no audit/notes fields");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricPublisherPlatformApproval = metric_publisher_platform_approval::table
        .first(&mut connection)
        .expect("Failed to load the approval row");
    assert_eq!(loaded.approved_by, None);
    assert_eq!(loaded.approved_at, None);
    assert_eq!(loaded.notes, None);

    let arbitrary_actor = Uuid::new_v4();
    diesel::update(
        metric_publisher_platform_approval::table.filter(
            metric_publisher_platform_approval::publisher_platform_approval_id
                .eq(loaded.publisher_platform_approval_id),
        ),
    )
    .set((
        metric_publisher_platform_approval::approved_by.eq(Some(arbitrary_actor)),
        metric_publisher_platform_approval::notes.eq(Some("a free-text annotation")),
    ))
    .execute(&mut connection)
    .expect("Failed to set approved_by/notes");
    let updated: MetricPublisherPlatformApproval = metric_publisher_platform_approval::table
        .filter(
            metric_publisher_platform_approval::publisher_platform_approval_id
                .eq(loaded.publisher_platform_approval_id),
        )
        .first(&mut connection)
        .expect("Failed to reload the approval row");
    assert_eq!(
        updated.approved_by,
        Some(arbitrary_actor),
        "an arbitrary syntactically valid UUID must be representable in approved_by, \
         proving no unapproved FK/identity relationship was introduced"
    );
    assert_eq!(updated.notes.as_deref(), Some("a free-text annotation"));
}

#[test]
fn usage_sales_and_status_fields_are_non_null() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    for (column, value_sql) in [
        ("usage_submission_enabled", "NULL"),
        ("sales_submission_enabled", "NULL"),
        ("approval_status", "NULL"),
    ] {
        let result = sql_query(format!(
            "INSERT INTO metric_publisher_platform_approval \
                 (publisher_id, platform_id, usage_submission_enabled, \
                  sales_submission_enabled, approval_status) \
             VALUES ($1, $2, {usage}, {sales}, {status})",
            usage = if column == "usage_submission_enabled" {
                value_sql
            } else {
                "false"
            },
            sales = if column == "sales_submission_enabled" {
                value_sql
            } else {
                "false"
            },
            status = if column == "approval_status" {
                value_sql
            } else {
                "'PENDING'"
            },
        ))
        .bind::<diesel::sql_types::Uuid, _>(fixture.publisher_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
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
fn invalid_approval_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    let unknown = Uuid::new_v4();

    for (label, publisher_id, platform_id) in [
        ("publisher", unknown, fixture.platform_id),
        ("platform", fixture.publisher_id, unknown),
    ] {
        let result = insert_approval_row(&pool, publisher_id, platform_id, "PENDING");
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
fn deleting_a_referenced_publisher_or_platform_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    insert_approval_row(&pool, fixture.publisher_id, fixture.platform_id, "PENDING")
        .expect("Failed to insert the referencing approval row");

    for (table, id_column, id) in [
        ("publisher", "publisher_id", fixture.publisher_id),
        ("metric_platform", "platform_id", fixture.platform_id),
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
             approval state, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_publisher_platform_approval)"
        ),
        1,
        "the approval row must survive the restricted deletions"
    );
}

#[test]
fn duplicate_publisher_platform_pair_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_approval_fixture(&pool);
    insert_approval_row(&pool, fixture.publisher_id, fixture.platform_id, "PENDING")
        .expect("Failed to insert the first approval row");

    let result = insert_approval_row(&pool, fixture.publisher_id, fixture.platform_id, "APPROVED");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate (publisher_id, platform_id) pair must be rejected, got {result:?}"
    );
}

#[test]
fn metric_publisher_platform_approval_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_publisher_platform_approval");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_publisher_platform_approval_platform_id_fkey",
            "metric_publisher_platform_approval_publisher_id_fkey",
        ],
        "metric_publisher_platform_approval must carry exactly the two authorized foreign keys"
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
fn metric_publisher_platform_approval_has_no_index_beyond_its_authorized_constraints() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_publisher_platform_approval"),
        vec![
            "metric_publisher_platform_approval_pkey",
            "metric_publisher_platform_approval_publisher_id_platform_id_key",
        ],
        "metric_publisher_platform_approval must carry exactly the primary-key index and \
         the (publisher_id, platform_id) uniqueness index, and no speculative secondary index"
    );
}

#[test]
fn reverting_through_the_approval_migration_removes_it_and_reapplication_restores_it() {
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

    revert_through_approval_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_publisher_platform_approval')",
        ),
        0,
        "the downgrade must drop the MET-WP1-06 table"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname = 'metric_publisher_platform_approval_status')",
        ),
        0,
        "the downgrade must drop the MET-WP1-06 enum type"
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
                                'metric_record_revision', 'metric_record_provenance', \
                                'metric_coverage'))",
        ),
        12,
        "the downgrade must leave the MET-WP1-01/02/03/04/05 schema in place"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "the downgrade must leave the measure seeds byte-identical"
    );

    // Reapplication recreates the enum and the empty table.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the approval migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname = 'metric_publisher_platform_approval_status')",
        ),
        1,
        "reapplication must recreate the enum type cleanly"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_publisher_platform_approval)",
        ),
        0,
        "reapplication must seed no approval row"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "reapplication must leave the measure seeds byte-identical"
    );
}
