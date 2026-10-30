//! Focused `MET-WP1-12` database tests for the `metric_registry_history`
//! audit table.
//!
//! These tests own the audit table's *shape*: its columns, enums, constraints,
//! index inventory, the absence of a foreign key on the polymorphic
//! `entity_id`, the absence of any timestamp added to
//! `metric_platform_measure`, and the migration's apply/revert/reapply
//! behaviour. The *behaviour* of the coordinators that write it — atomicity,
//! before/after exactness, no-op silence and locking — lives with each registry
//! in `metric_platform`, `metric_measure` and `metric_platform_measure`.

use diesel::pg::PgConnection;
use diesel::{sql_query, Connection, RunQueryDsl};
use diesel_migrations::MigrationHarness;

use super::{MetricRegistryHistoryAction, MetricRegistryHistoryEntity};
use crate::db::MIGRATIONS;
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;

/// The Diesel migration version of `thoth-api/migrations/20260908_v1.9.0`.
pub(crate) const MET_WP1_12_MIGRATION_VERSION: &str = "20260908";

/// Every column of the approved audit contract, with its exact type,
/// nullability and default.
///
/// The tuple is `(name, type, not_null, default)`. Stating all four per column
/// is the point: a column that silently became nullable, lost its default or
/// changed type would still satisfy a name-only inventory.
const AUDIT_COLUMNS: [(&str, &str, bool, Option<&str>); 8] = [
    (
        "metric_registry_history_id",
        "uuid",
        true,
        Some("uuid_generate_v4()"),
    ),
    ("entity", "metric_registry_history_entity", true, None),
    ("entity_id", "uuid", true, None),
    ("action", "metric_registry_history_action", true, None),
    ("actor", "text", true, None),
    ("before_state", "jsonb", false, None),
    ("after_state", "jsonb", true, None),
    (
        "created_at",
        "timestamp with time zone",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
];

/// Column names that would betray a mutable, deletable, queryable or
/// cross-programme audit design having been smuggled into this append-only
/// slice. The approved contract names none of them.
const DEFERRED_AUDIT_COLUMNS: [&str; 8] = [
    "deleted_at",
    "entity_code",
    "reason",
    "request_id",
    "retention_expires_at",
    "source",
    "superseded_by",
    "updated_at",
];

/// Revert migrations until the `MET-WP1-12` administration migration itself has
/// been reverted.
///
/// Reverting *through* the target rather than calling `revert_last_migration`
/// once keeps the meaning under any future migration order, exactly as the
/// predecessor Metrics slices do.
fn revert_through_admin_migration(connection: &mut PgConnection) {
    let applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_12_MIGRATION_VERSION);
    assert!(
        applied,
        "the MET-WP1-12 administration migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_12_MIGRATION_VERSION {
            return;
        }
    }
}

#[test]
fn the_audit_table_has_exactly_the_approved_column_contract() {
    let (_guard, pool) = setup_registry_db();

    for (name, sql_type, not_null, default) in AUDIT_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute a \
                       LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
                      WHERE a.attrelid = 'metric_registry_history'::regclass \
                        AND a.attnum > 0 AND NOT a.attisdropped \
                        AND a.attname = '{name}' \
                        AND format_type(a.atttypid, a.atttypmod) = '{sql_type}' \
                        AND a.attnotnull = {not_null} \
                        AND pg_get_expr(d.adbin, d.adrelid) IS NOT DISTINCT FROM {})",
                    match default {
                        Some(expression) => format!("'{expression}'"),
                        None => "NULL".to_string(),
                    }
                ),
            ),
            1,
            "column `{name}` must be {sql_type}, not_null={not_null}, default={default:?}"
        );
    }

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_registry_history'::regclass \
                AND attnum > 0 AND NOT attisdropped)",
        ),
        AUDIT_COLUMNS.len() as i64,
        "the audit table must carry exactly the approved columns and no others"
    );

    for column in DEFERRED_AUDIT_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute \
                      WHERE attrelid = 'metric_registry_history'::regclass \
                        AND attname = '{column}' AND attnum > 0 AND NOT attisdropped)"
                ),
            ),
            0,
            "`{column}` is not part of the approved append-only audit contract"
        );
    }
}

#[test]
fn the_audit_enums_hold_exactly_the_approved_values() {
    let (_guard, pool) = setup_registry_db();

    assert_eq!(
        enum_labels(&pool, "metric_registry_history_entity"),
        vec!["PLATFORM", "MEASURE", "PLATFORM_MEASURE"],
    );
    // There is deliberately no DELETE action: no delete mutation exists.
    assert_eq!(
        enum_labels(&pool, "metric_registry_history_action"),
        vec!["CREATE", "UPDATE"],
    );

    for (variant, label) in [
        (MetricRegistryHistoryEntity::Platform, "PLATFORM"),
        (MetricRegistryHistoryEntity::Measure, "MEASURE"),
        (
            MetricRegistryHistoryEntity::PlatformMeasure,
            "PLATFORM_MEASURE",
        ),
    ] {
        assert_db_enum_roundtrip::<
            MetricRegistryHistoryEntity,
            crate::schema::sql_types::MetricRegistryHistoryEntity,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_registry_history_entity"),
            variant,
        );
    }
    for (variant, label) in [
        (MetricRegistryHistoryAction::Create, "CREATE"),
        (MetricRegistryHistoryAction::Update, "UPDATE"),
    ] {
        assert_db_enum_roundtrip::<
            MetricRegistryHistoryAction,
            crate::schema::sql_types::MetricRegistryHistoryAction,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_registry_history_action"),
            variant,
        );
    }
}

#[test]
fn the_audit_table_has_exactly_the_approved_constraints_and_no_foreign_key() {
    let (_guard, pool) = setup_registry_db();

    for constraint in [
        "metric_registry_history_pkey",
        "metric_registry_history_actor_check",
        "metric_registry_history_action_before_state_check",
    ] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_constraint \
                      WHERE conrelid = 'metric_registry_history'::regclass \
                        AND conname = '{constraint}')"
                ),
            ),
            1,
            "`{constraint}` must exist under exactly that name, because the \
             application error boundary maps it by name"
        );
    }

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_registry_history'::regclass)",
        ),
        3,
        "the audit table must carry exactly the primary key and the two named CHECKs"
    );

    // The polymorphic entity_id deliberately carries no foreign key: audit
    // evidence must not be cascade-deleted with the registry state it records.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_registry_history'::regclass AND contype = 'f')",
        ),
        0,
        "entity_id must not gain a foreign key"
    );

    // Append-only: no updated_at column, therefore no timestamp trigger.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'metric_registry_history'::regclass AND NOT tgisinternal)",
        ),
        0,
        "the append-only audit table must carry no trigger"
    );

    // PK-only index inventory: no approved audit-history access path exists yet.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_index WHERE indrelid = 'metric_registry_history'::regclass)",
        ),
        1,
        "the primary key is the complete intended index set for MET-WP1-12"
    );
}

#[test]
fn metric_platform_measure_gained_no_timestamp_column() {
    let (_guard, pool) = setup_registry_db();

    for column in ["created_at", "updated_at"] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute \
                      WHERE attrelid = 'metric_platform_measure'::regclass \
                        AND attname = '{column}' AND attnum > 0 AND NOT attisdropped)"
                ),
            ),
            0,
            "MET-WP1-12 must not add `{column}` to metric_platform_measure: audit \
             created_at records mutation time without changing the design §6.3 row shape"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'metric_platform_measure'::regclass AND NOT tgisinternal)",
        ),
        0,
        "metric_platform_measure must not gain a timestamp trigger"
    );
}

#[test]
fn the_actor_check_rejects_a_blank_actor() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    for blank in ["", " ", "\t", "\n   "] {
        let result = sql_query(
            "INSERT INTO metric_registry_history (entity, entity_id, action, actor, after_state) \
             VALUES ('PLATFORM', uuid_generate_v4(), 'CREATE', $1, '{}'::jsonb)",
        )
        .bind::<diesel::sql_types::Text, _>(blank)
        .execute(&mut connection);

        assert!(
            result.is_err(),
            "a blank actor ({blank:?}) must be rejected by the database"
        );
    }

    // A real actor is accepted, so the CHECK is not rejecting everything.
    sql_query(
        "INSERT INTO metric_registry_history (entity, entity_id, action, actor, after_state) \
         VALUES ('PLATFORM', uuid_generate_v4(), 'CREATE', 'user-1', '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a non-blank actor must be accepted");
}

#[test]
fn the_action_check_binds_before_state_to_the_action() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // CREATE requires before_state IS NULL.
    assert!(
        sql_query(
            "INSERT INTO metric_registry_history \
                 (entity, entity_id, action, actor, before_state, after_state) \
             VALUES ('MEASURE', uuid_generate_v4(), 'CREATE', 'a', '{}'::jsonb, '{}'::jsonb)",
        )
        .execute(&mut connection)
        .is_err(),
        "a CREATE entry must not record a previous state"
    );

    // UPDATE requires before_state IS NOT NULL.
    assert!(
        sql_query(
            "INSERT INTO metric_registry_history (entity, entity_id, action, actor, after_state) \
             VALUES ('MEASURE', uuid_generate_v4(), 'UPDATE', 'a', '{}'::jsonb)",
        )
        .execute(&mut connection)
        .is_err(),
        "an UPDATE entry must record the state it overwrote"
    );

    // Both well-formed shapes are accepted.
    sql_query(
        "INSERT INTO metric_registry_history (entity, entity_id, action, actor, after_state) \
         VALUES ('MEASURE', uuid_generate_v4(), 'CREATE', 'a', '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a well-formed CREATE entry must be accepted");
    sql_query(
        "INSERT INTO metric_registry_history \
             (entity, entity_id, action, actor, before_state, after_state) \
         VALUES ('MEASURE', uuid_generate_v4(), 'UPDATE', 'a', '{}'::jsonb, '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a well-formed UPDATE entry must be accepted");
}

/// One scalar `BIGINT` result on a caller-owned connection.
///
/// The revert/reapply test drives migrations on its own connection, so it
/// cannot use the pool-based [`scalar_i64`]: reverting drops and recreates the
/// registry enum types, and a pooled connection could still hold their previous
/// type OIDs in its metadata cache.
fn on_connection(connection: &mut PgConnection, query: &str) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(connection)
        .expect("Failed to run scalar query")
}

#[test]
fn the_migration_reverts_and_reapplies_leaving_every_predecessor_intact() {
    let (_guard, pool) = setup_registry_db();
    let database_url = test_db_url();

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_registry_history')",
        ),
        1,
        "the audit table must exist before reverting"
    );

    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    revert_through_admin_migration(&mut connection);

    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_registry_history')",
        ),
        0,
        "the downgrade must drop the audit table"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname LIKE 'metric_registry_history%')",
        ),
        0,
        "the downgrade must drop both audit enums"
    );
    // Every predecessor Metrics slice survives the downgrade untouched.
    assert_eq!(
        on_connection(
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
                                'metric_operas_export', 'metric_operas_import', \
                                'metric_reconciliation_run', 'metric_reconciliation_issue'))",
        ),
        19,
        "the downgrade must leave the MET-WP1-01..11 schema in place"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2,
        "the downgrade must not disturb the migration-owned measure seeds"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('work', 'publication', 'institution', 'publisher'))",
        ),
        4,
        "the downgrade must not touch the bibliographic schema"
    );

    // Re-apply, and the audit contract is back exactly as specified.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the administration migration onward");

    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_registry_history'::regclass)",
        ),
        3,
        "the reapply must restore the primary key and both named CHECKs"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_registry_history)",
        ),
        0,
        "the migration seeds no audit row"
    );
}

#[test]
fn deleting_a_registry_row_leaves_its_audit_evidence_behind() {
    // The polymorphic `entity_id` carries no foreign key precisely so that audit
    // evidence outlives the registry state it describes. Asserting the absence
    // of a constraint proves the schema; deleting the referenced row proves the
    // behaviour that absence is there to deliver.
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    sql_query(
        "INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) \
         VALUES ('11111111-1111-4111-8111-111111111111', 'doomed', 'Doomed', 'EXTERNAL', TRUE)",
    )
    .execute(&mut connection)
    .expect("Failed to insert platform fixture");
    sql_query(
        "INSERT INTO metric_registry_history (entity, entity_id, action, actor, after_state) \
         VALUES ('PLATFORM', '11111111-1111-4111-8111-111111111111', 'CREATE', 'actor-1', \
                 '{\"code\":\"doomed\"}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("Failed to insert audit fixture");

    sql_query("DELETE FROM metric_platform WHERE code = 'doomed'")
        .execute(&mut connection)
        .expect("an out-of-band delete of an unreferenced platform must succeed");

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_registry_history \
              WHERE entity_id = '11111111-1111-4111-8111-111111111111')",
        ),
        1,
        "audit evidence must survive deletion of the row it describes"
    );
}
