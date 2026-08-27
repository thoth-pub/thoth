//! Focused `MET-WP1-02` database tests for the `metric_source_checkpoint`
//! durable checkpoint/lease storage: identity, non-cascading account foreign
//! key, nullable JSONB cursor, progress/lease/error round-trips, the
//! repository-standard `updated_at` trigger and the exact index inventory.
//!
//! Deliberately absent: any claim, lease-acquisition, `FOR UPDATE SKIP
//! LOCKED` or stale-lease test. The operation-level concurrency protocol is
//! outside this slice and must not be pretend-tested here.

use chrono::NaiveDate;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

use super::MetricSourceCheckpoint;
use crate::db::PgPool;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_source_account::tests::{fixture_source_and_platform, insert_account_row};
use crate::model::Timestamp;
use crate::schema::metric_source_checkpoint;

/// Insert one referenced source/platform/account chain for checkpoint tests.
fn fixture_account(pool: &PgPool) -> Uuid {
    let (source_id, platform_id) = fixture_source_and_platform(pool);
    let source_account_id = Uuid::new_v4();
    insert_account_row(pool, source_account_id, source_id, platform_id, "account-1");
    source_account_id
}

fn insert_checkpoint_raw(
    pool: &PgPool,
    source_account_id: Uuid,
    partition_key: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_checkpoint (source_account_id, partition_key) \
         VALUES ($1, $2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Text, _>(partition_key)
    .execute(&mut connection)
}

#[test]
fn migration_seeds_no_checkpoint_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_checkpoint)"),
        0,
        "MET-WP1-02 must not seed any metric_source_checkpoint row"
    );
}

#[test]
fn checkpoint_deliberately_has_no_created_at_column() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND table_name = 'metric_source_checkpoint' \
                AND column_name = 'created_at')",
        ),
        0,
        "the approved design specifies no created_at column for checkpoints"
    );
}

#[test]
fn blank_partition_key_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_checkpoint_raw(&pool, source_account_id, blank);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank partition key {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn checkpoint_identity_is_unique_per_account_and_partition_key() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    let source_account_id = Uuid::new_v4();
    insert_account_row(
        &pool,
        source_account_id,
        source_id,
        platform_id,
        "account-1",
    );
    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("First insert must pass");
    let duplicate = insert_checkpoint_raw(&pool, source_account_id, "2026-07");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate (source_account_id, partition_key) pair must fail the unique \
         constraint: {duplicate:?}"
    );

    // The same partition key under another account is a different identity.
    let other_account_id = Uuid::new_v4();
    insert_account_row(&pool, other_account_id, source_id, platform_id, "account-2");
    insert_checkpoint_raw(&pool, other_account_id, "2026-07")
        .expect("The same partition key under another account must pass");
}

#[test]
fn checkpoint_foreign_key_requires_an_existing_account_and_restricts_deletion() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);

    let unknown_account = insert_checkpoint_raw(&pool, Uuid::new_v4(), "2026-07");
    assert!(
        matches!(
            unknown_account,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a checkpoint referencing an unknown account must fail the foreign key: \
         {unknown_account:?}"
    );

    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("Insert must pass");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let delete_account =
        sql_query("DELETE FROM metric_source_account WHERE source_account_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(source_account_id)
            .execute(&mut connection);
    assert!(
        matches!(
            delete_account,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting an account still referenced by a checkpoint must be restricted, \
         not cascaded: {delete_account:?}"
    );
}

#[test]
fn metric_source_checkpoint_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let cursor = json!({"page": 3, "token": "resume-token"});
    let discovered = Timestamp::parse_from_rfc3339("2026-08-01T12:00:00Z")
        .expect("Failed to parse the discovery timestamp");
    let completed = Timestamp::parse_from_rfc3339("2026-08-01T12:05:00Z")
        .expect("Failed to parse the completion timestamp");
    let lease_expiry = Timestamp::parse_from_rfc3339("2026-08-01T12:10:00Z")
        .expect("Failed to parse the lease expiry timestamp");
    let period_end = NaiveDate::from_ymd_opt(2026, 7, 31).expect("Failed to build the period end");

    let progressed_id: Uuid = diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("2026-07"),
            metric_source_checkpoint::cursor.eq(cursor.clone()),
            metric_source_checkpoint::last_discovered_at.eq(discovered),
            metric_source_checkpoint::last_completed_at.eq(completed),
            metric_source_checkpoint::last_successful_period_end.eq(period_end),
            metric_source_checkpoint::lease_owner.eq("sphinx-worker-1"),
            metric_source_checkpoint::lease_expires_at.eq(lease_expiry),
            metric_source_checkpoint::last_error.eq("upstream returned HTTP 503"),
        ))
        .returning(metric_source_checkpoint::source_checkpoint_id)
        .get_result(&mut connection)
        .expect("Failed to insert progressed checkpoint row");
    diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("2026-08"),
        ))
        .execute(&mut connection)
        .expect("Failed to insert fresh checkpoint row");

    let progressed: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load progressed checkpoint row");
    assert_eq!(progressed.source_checkpoint_id, progressed_id);
    assert_eq!(progressed.source_account_id, source_account_id);
    assert_eq!(progressed.partition_key, "2026-07");
    assert_eq!(progressed.cursor, Some(cursor));
    assert_eq!(progressed.last_discovered_at, Some(discovered));
    assert_eq!(progressed.last_completed_at, Some(completed));
    assert_eq!(progressed.last_successful_period_end, Some(period_end));
    assert_eq!(progressed.lease_owner.as_deref(), Some("sphinx-worker-1"));
    assert_eq!(progressed.lease_expires_at, Some(lease_expiry));
    assert_eq!(
        progressed.last_error.as_deref(),
        Some("upstream returned HTTP 503")
    );

    let fresh: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-08"))
        .first(&mut connection)
        .expect("Failed to load fresh checkpoint row");
    assert_eq!(fresh.cursor, None);
    assert_eq!(fresh.last_discovered_at, None);
    assert_eq!(fresh.last_completed_at, None);
    assert_eq!(fresh.last_successful_period_end, None);
    assert_eq!(fresh.lease_owner, None);
    assert_eq!(fresh.lease_expires_at, None);
    assert_eq!(fresh.last_error, None);
}

#[test]
fn checkpoint_updated_at_is_maintained_by_the_repository_standard_trigger() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("Insert must pass");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let initial: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load the fresh checkpoint row");

    sql_query(
        "UPDATE metric_source_checkpoint SET last_error = 'transient failure' \
         WHERE partition_key = '2026-07'",
    )
    .execute(&mut connection)
    .expect("Failed to update the checkpoint row");

    let updated: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load the updated checkpoint row");
    assert!(
        updated.updated_at > initial.updated_at,
        "the set_updated_at trigger must advance updated_at on update \
         ({:?} -> {:?})",
        initial.updated_at,
        updated.updated_at
    );
}

#[test]
fn lease_expiry_has_the_required_operational_index() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND tablename = 'metric_source_checkpoint' \
                AND indexname = 'metric_source_checkpoint_lease_expires_at_idx' \
                AND indexdef LIKE '%(lease_expires_at)%')",
        ),
        1,
        "the design-required operational index on lease_expires_at must exist"
    );
}

#[test]
fn source_state_tables_have_no_speculative_secondary_index() {
    let (_guard, pool) = setup_registry_db();
    // The complete intended index inventory is exactly: three primary keys,
    // metric_source(code) UNIQUE, the two composite identity UNIQUEs, and the
    // single operational lease-expiry index.
    for (table, expected) in [
        ("metric_source", 2),
        ("metric_source_account", 2),
        ("metric_source_checkpoint", 3),
    ] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_indexes \
                      WHERE schemaname = 'public' AND tablename = '{table}')"
                ),
            ),
            expected,
            "{table} must carry exactly its constraint-derived indexes \
             (plus, for checkpoints, the lease-expiry index)"
        );
    }
}
