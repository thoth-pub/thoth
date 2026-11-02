//! Focused `MET-WP1-13` database tests for the `metric_source_registry_history`
//! audit table and the `MET-WP1-13` migration.
//!
//! These tests own the audit table's *shape* — columns, enums, constraints,
//! index inventory, the absence of a foreign key on the polymorphic
//! `entity_id` — the `metric_source` driver-key CHECK at the database
//! boundary, and the migration's apply/revert/reapply behaviour on empty and
//! populated databases. The *behaviour* of the coordinators that write the
//! audit — atomicity, before/after exactness, no-op silence and locking —
//! lives with each entity in `metric_source` and `metric_source_account`.

use diesel::pg::PgConnection;
use diesel::{sql_query, Connection, RunQueryDsl};
use diesel_migrations::MigrationHarness;

use super::{MetricSourceRegistryHistoryAction, MetricSourceRegistryHistoryEntity};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db, AuditRow};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;

/// The Diesel migration version of `thoth-api/migrations/20260912_v1.9.0`.
pub(crate) const MET_WP1_13_MIGRATION_VERSION: &str = "20260912";

/// Every column of the approved source-audit contract, with its exact type,
/// nullability and default.
const AUDIT_COLUMNS: [(&str, &str, bool, Option<&str>); 8] = [
    (
        "metric_source_registry_history_id",
        "uuid",
        true,
        Some("uuid_generate_v4()"),
    ),
    (
        "entity",
        "metric_source_registry_history_entity",
        true,
        None,
    ),
    ("entity_id", "uuid", true, None),
    (
        "action",
        "metric_source_registry_history_action",
        true,
        None,
    ),
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
/// cross-programme audit design having been smuggled into this slice.
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

/// Every audit row of the source-administration audit, ordered by
/// `created_at` then primary key.
///
/// Like `metric_platform::tests::audit_rows`, that ordering is a convenience
/// for sequential single-connection tests and is **not** commit order: the
/// concurrency evidence reconstructs order from the before/after states with
/// `serialized_update_chain` instead. The rows are read into the shared
/// [`AuditRow`] shape so that helper applies unchanged.
pub(crate) fn source_audit_rows(pool: &PgPool) -> Vec<AuditRow> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT entity::text AS entity, entity_id, action::text AS action, actor, \
                before_state, after_state \
         FROM metric_source_registry_history \
         ORDER BY created_at, metric_source_registry_history_id",
    )
    .load::<AuditRow>(&mut connection)
    .expect("Failed to read source audit rows")
}

/// Revert migrations until the `MET-WP1-13` migration itself has been reverted.
pub(crate) fn revert_through_source_admin_migration(connection: &mut PgConnection) {
    let applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_13_MIGRATION_VERSION);
    assert!(
        applied,
        "the MET-WP1-13 migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_13_MIGRATION_VERSION {
            return;
        }
    }
}

fn on_connection(connection: &mut PgConnection, query: &str) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(connection)
        .expect("Failed to run scalar query")
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
                      WHERE a.attrelid = 'metric_source_registry_history'::regclass \
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
              WHERE attrelid = 'metric_source_registry_history'::regclass \
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
                      WHERE attrelid = 'metric_source_registry_history'::regclass \
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
        enum_labels(&pool, "metric_source_registry_history_entity"),
        vec!["SOURCE", "SOURCE_ACCOUNT"],
        "no checkpoint or other entity may be auditable through this table"
    );
    assert_eq!(
        enum_labels(&pool, "metric_source_registry_history_action"),
        vec!["CREATE", "UPDATE"],
        "there is deliberately no DELETE action"
    );
    // The WP1-12 audit enums are untouched: this is a parallel audit, not an
    // extension of the closed registry inventory.
    assert_eq!(
        enum_labels(&pool, "metric_registry_history_entity"),
        vec!["PLATFORM", "MEASURE", "PLATFORM_MEASURE"],
    );

    for (variant, label) in [
        (MetricSourceRegistryHistoryEntity::Source, "SOURCE"),
        (
            MetricSourceRegistryHistoryEntity::SourceAccount,
            "SOURCE_ACCOUNT",
        ),
    ] {
        assert_db_enum_roundtrip::<
            MetricSourceRegistryHistoryEntity,
            crate::schema::sql_types::MetricSourceRegistryHistoryEntity,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_source_registry_history_entity"),
            variant,
        );
    }
    for (variant, label) in [
        (MetricSourceRegistryHistoryAction::Create, "CREATE"),
        (MetricSourceRegistryHistoryAction::Update, "UPDATE"),
    ] {
        assert_db_enum_roundtrip::<
            MetricSourceRegistryHistoryAction,
            crate::schema::sql_types::MetricSourceRegistryHistoryAction,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_source_registry_history_action"),
            variant,
        );
    }
}

#[test]
fn the_audit_table_has_exactly_the_approved_constraints_and_no_foreign_key() {
    let (_guard, pool) = setup_registry_db();

    for constraint in [
        "metric_source_registry_history_pkey",
        "metric_source_registry_history_actor_check",
        "metric_source_registry_history_action_before_state_check",
    ] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_constraint \
                      WHERE conrelid = 'metric_source_registry_history'::regclass \
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
              WHERE conrelid = 'metric_source_registry_history'::regclass)",
        ),
        3,
        "the audit table must carry exactly the primary key and the two named CHECKs"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_source_registry_history'::regclass AND contype = 'f')",
        ),
        0,
        "entity_id must not gain a foreign key"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'metric_source_registry_history'::regclass AND NOT tgisinternal)",
        ),
        0,
        "the append-only audit table must carry no trigger"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_index \
              WHERE indrelid = 'metric_source_registry_history'::regclass)",
        ),
        1,
        "the primary key is the complete intended index set for MET-WP1-13"
    );
}

#[test]
fn the_source_tables_gained_no_timestamp_column_and_no_trigger() {
    let (_guard, pool) = setup_registry_db();
    for table in ["metric_source", "metric_source_account"] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute \
                      WHERE attrelid = '{table}'::regclass \
                        AND attname IN ('created_at', 'updated_at') \
                        AND attnum > 0 AND NOT attisdropped)"
                ),
            ),
            0,
            "MET-WP1-13 must not add a timestamp column to {table}"
        );
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_trigger \
                      WHERE tgrelid = '{table}'::regclass AND NOT tgisinternal)"
                ),
            ),
            0,
            "{table} must not gain a trigger"
        );
    }
}

#[test]
fn the_driver_key_check_enforces_the_exact_truth_table_at_the_database() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_source'::regclass \
                AND conname = 'metric_source_driver_key_check' AND contype = 'c')",
        ),
        1,
        "the driver-key CHECK must exist under exactly its mapped name"
    );

    let insert = |connection: &mut PgConnection, code: &str, acquisition: &str, key: &str| {
        sql_query(format!(
            "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled) \
             VALUES ('{code}', '{acquisition}', {key}, TRUE)"
        ))
        .execute(connection)
    };

    // DRIVER requires a key with at least one non-whitespace character; it is
    // stored exactly, including surrounding whitespace.
    for (code, key) in [("d1", "'cloudfront'"), ("d2", "' padded '"), ("d3", "'K'")] {
        insert(&mut connection, code, "DRIVER", key)
            .unwrap_or_else(|error| panic!("DRIVER with key {key} must be accepted: {error:?}"));
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source WHERE code = 'd2' AND driver_key = ' padded ')",
        ),
        1,
        "a valid driver key must be stored exactly as supplied"
    );
    for (code, key) in [
        ("r1", "NULL"),
        ("r2", "''"),
        ("r3", "'   '"),
        ("r4", "E'\\t\\n'"),
    ] {
        let result = insert(&mut connection, code, "DRIVER", key);
        assert!(
            result.is_err(),
            "DRIVER with driver_key {key} must fail the CHECK: {result:?}"
        );
    }
    // Every non-DRIVER acquisition type requires NULL.
    for (index, acquisition) in ["PUBLISHER_UPLOAD", "OPERAS", "ADMIN_IMPORT"]
        .into_iter()
        .enumerate()
    {
        insert(&mut connection, &format!("n{index}"), acquisition, "NULL")
            .unwrap_or_else(|error| panic!("{acquisition} with NULL must be accepted: {error:?}"));
        let result = insert(
            &mut connection,
            &format!("x{index}"),
            acquisition,
            "'cloudfront'",
        );
        assert!(
            result.is_err(),
            "{acquisition} with a driver key must fail the CHECK: {result:?}"
        );
    }
}

#[test]
fn the_actor_check_rejects_a_blank_actor() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");
    for blank in ["", " ", "\t", "\n   "] {
        let result = sql_query(
            "INSERT INTO metric_source_registry_history \
                 (entity, entity_id, action, actor, after_state) \
             VALUES ('SOURCE', uuid_generate_v4(), 'CREATE', $1, '{}'::jsonb)",
        )
        .bind::<diesel::sql_types::Text, _>(blank)
        .execute(&mut connection);
        assert!(
            result.is_err(),
            "a blank actor ({blank:?}) must be rejected"
        );
    }
    sql_query(
        "INSERT INTO metric_source_registry_history \
             (entity, entity_id, action, actor, after_state) \
         VALUES ('SOURCE', uuid_generate_v4(), 'CREATE', 'user-1', '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a non-blank actor must be accepted");
}

#[test]
fn the_action_check_binds_before_state_to_the_action() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    assert!(
        sql_query(
            "INSERT INTO metric_source_registry_history \
                 (entity, entity_id, action, actor, before_state, after_state) \
             VALUES ('SOURCE_ACCOUNT', uuid_generate_v4(), 'CREATE', 'a', '{}'::jsonb, '{}'::jsonb)",
        )
        .execute(&mut connection)
        .is_err(),
        "a CREATE entry must not record a previous state"
    );
    assert!(
        sql_query(
            "INSERT INTO metric_source_registry_history \
                 (entity, entity_id, action, actor, after_state) \
             VALUES ('SOURCE_ACCOUNT', uuid_generate_v4(), 'UPDATE', 'a', '{}'::jsonb)",
        )
        .execute(&mut connection)
        .is_err(),
        "an UPDATE entry must record the state it overwrote"
    );
    sql_query(
        "INSERT INTO metric_source_registry_history \
             (entity, entity_id, action, actor, after_state) \
         VALUES ('SOURCE_ACCOUNT', uuid_generate_v4(), 'CREATE', 'a', '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a well-formed CREATE entry must be accepted");
    sql_query(
        "INSERT INTO metric_source_registry_history \
             (entity, entity_id, action, actor, before_state, after_state) \
         VALUES ('SOURCE_ACCOUNT', uuid_generate_v4(), 'UPDATE', 'a', '{}'::jsonb, '{}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("a well-formed UPDATE entry must be accepted");
}

#[test]
fn the_migration_reverts_and_reapplies_leaving_every_predecessor_intact() {
    let (_guard, pool) = setup_registry_db();
    let database_url = test_db_url();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_source_registry_history')",
        ),
        1
    );

    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    revert_through_source_admin_migration(&mut connection);

    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_source_registry_history')",
        ),
        0,
        "the downgrade must drop the source audit table"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type WHERE typnamespace = 'public'::regnamespace \
                AND typname LIKE 'metric_source_registry_history%')",
        ),
        0,
        "the downgrade must drop both source audit enums"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint WHERE conrelid = 'metric_source'::regclass \
                AND conname = 'metric_source_driver_key_check')",
        ),
        0,
        "the downgrade must drop the driver-key CHECK"
    );
    // Every predecessor slice survives, including WP1-12's audit and WP2-01A's
    // batch table, together with the migration-owned measure seeds.
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint', \
                                'metric_import', 'metric_import_error', \
                                'metric_record', 'metric_record_revision', \
                                'metric_record_provenance', 'metric_coverage', \
                                'metric_publisher_platform_approval', \
                                'metric_rollup_delta', 'metric_operas_mapping', \
                                'metric_operas_export', 'metric_operas_import', \
                                'metric_reconciliation_run', 'metric_reconciliation_issue', \
                                'metric_registry_history', 'metric_import_batch'))",
        ),
        21,
        "the downgrade must leave the MET-WP1-01..12 and MET-WP2-01A schema in place"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint WHERE conrelid = 'metric_source'::regclass)",
        ),
        5,
        "metric_source keeps its primary key, unique code, and three predecessor CHECKs"
    );

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the source-administration migration onward");
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_source_registry_history'::regclass)",
        ),
        3
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint WHERE conrelid = 'metric_source'::regclass \
                AND conname = 'metric_source_driver_key_check')",
        ),
        1
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "((SELECT COUNT(*) FROM metric_source_registry_history) \
              + (SELECT COUNT(*) FROM metric_source) \
              + (SELECT COUNT(*) FROM metric_source_account))",
        ),
        0,
        "the migration seeds no audit, source or account row"
    );
}

#[test]
fn applying_to_a_populated_database_preserves_valid_rows_and_fails_closed_on_a_violation() {
    let (_guard, _pool) = setup_registry_db();
    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database");
    revert_through_source_admin_migration(&mut connection);

    // A representative pre-migration population: one of each acquisition type
    // that already satisfies the invariant, one account with generic
    // pre-existing JSON, and (initially) nothing that violates it.
    sql_query(
        "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled, \
                                    default_lookback_days) VALUES \
             ('pre_driver', 'DRIVER', 'legacy_driver', TRUE, 7), \
             ('pre_upload', 'PUBLISHER_UPLOAD', NULL, TRUE, NULL), \
             ('pre_operas', 'OPERAS', NULL, FALSE, 0), \
             ('pre_admin', 'ADMIN_IMPORT', NULL, TRUE, NULL)",
    )
    .execute(&mut connection)
    .expect("pre-migration sources");
    sql_query(
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ('pre_platform', 'Pre', 'EXTERNAL', TRUE)",
    )
    .execute(&mut connection)
    .expect("pre-migration platform");
    sql_query(
        "INSERT INTO metric_source_account \
             (code, source_id, platform_id, external_key, configuration, enabled) \
         SELECT 'pre_account', s.source_id, p.platform_id, 'pre-key', \
                '{\"generic\": {\"nested\": [1, 2]}, \"note\": \"pre-existing\"}'::jsonb, TRUE \
           FROM metric_source s, metric_platform p \
          WHERE s.code = 'pre_driver' AND p.code = 'pre_platform'",
    )
    .execute(&mut connection)
    .expect("pre-migration account");

    #[derive(diesel::QueryableByName)]
    struct Snapshot {
        #[diesel(sql_type = diesel::sql_types::Text)]
        row: String,
    }
    let snapshot = |connection: &mut PgConnection| -> Vec<String> {
        sql_query(
            "SELECT row_to_json(t)::text AS row FROM ( \
                 SELECT 'source' AS kind, code, acquisition_type::text, driver_key, enabled, \
                        default_lookback_days, default_finalization_delay_days, NULL::jsonb \
                        AS configuration FROM metric_source \
                 UNION ALL \
                 SELECT 'account', code, NULL, external_key, enabled, NULL, NULL, configuration \
                   FROM metric_source_account) t ORDER BY kind, code",
        )
        .load::<Snapshot>(connection)
        .expect("snapshot")
        .into_iter()
        .map(|s| s.row)
        .collect()
    };
    let before = snapshot(&mut connection);
    assert_eq!(before.len(), 5);

    // Fail closed: a pre-existing DRIVER row without a usable driver key cannot
    // be silently rewritten, so the migration must refuse and leave the
    // database in its pre-migration state. The Unicode-whitespace-only keys
    // are exactly the rows a `[:space:]` CHECK would have admitted under the
    // C locale.
    for broken_key in [None, Some("\u{00A0}"), Some("\u{2003}\u{3000}\u{0085}")] {
        sql_query(
            "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled) \
             VALUES ('pre_broken', 'DRIVER', $1, TRUE)",
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(broken_key)
        .execute(&mut connection)
        .expect("a violating row is insertable before the CHECK exists");
        assert!(
            connection.run_pending_migrations(MIGRATIONS).is_err(),
            "the migration must fail closed on a pre-existing invariant violation \
             ({broken_key:?})"
        );
        assert_eq!(
            on_connection(
                &mut connection,
                "(SELECT COUNT(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace \
                    AND relname = 'metric_source_registry_history')",
            ),
            0,
            "a failed migration must leave no partial object behind ({broken_key:?})"
        );
        assert_eq!(
            on_connection(
                &mut connection,
                "(SELECT COUNT(*) FROM pg_constraint WHERE conrelid = 'metric_source'::regclass \
                    AND conname = 'metric_source_driver_key_check')",
            ),
            0,
            "a failed migration must not leave the CHECK behind ({broken_key:?})"
        );
        assert_eq!(
            on_connection(&mut connection, "(SELECT COUNT(*) FROM metric_source)"),
            5,
            "a failed migration must not delete any row ({broken_key:?})"
        );
        let preserved = sql_query(
            "SELECT 1 FROM metric_source \
             WHERE code = 'pre_broken' AND driver_key IS NOT DISTINCT FROM $1",
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(broken_key)
        .execute(&mut connection)
        .expect("read the violating row back");
        assert_eq!(
            preserved, 1,
            "a failed migration must not rewrite the violating row ({broken_key:?})"
        );

        // Remove the violation before trying the next one.
        sql_query("DELETE FROM metric_source WHERE code = 'pre_broken'")
            .execute(&mut connection)
            .expect("remove the violating fixture row");
    }

    // With the violation removed, the migration applies and rewrites nothing.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the migration must apply to a valid populated database");
    assert_eq!(
        snapshot(&mut connection),
        before,
        "every pre-existing source and account row, including generic pre-existing \
         configuration JSON, must be byte-identical after the migration"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_source_registry_history)",
        ),
        0,
        "the migration must not manufacture audit history for pre-existing rows"
    );
}

#[test]
fn deleting_a_source_row_leaves_its_audit_evidence_behind() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source (source_id, code, acquisition_type, enabled) \
         VALUES ('22222222-2222-4222-8222-222222222222', 'doomed', 'ADMIN_IMPORT', TRUE)",
    )
    .execute(&mut connection)
    .expect("source fixture");
    sql_query(
        "INSERT INTO metric_source_registry_history (entity, entity_id, action, actor, after_state) \
         VALUES ('SOURCE', '22222222-2222-4222-8222-222222222222', 'CREATE', 'actor-1', \
                 '{\"code\":\"doomed\"}'::jsonb)",
    )
    .execute(&mut connection)
    .expect("audit fixture");
    sql_query("DELETE FROM metric_source WHERE code = 'doomed'")
        .execute(&mut connection)
        .expect("an out-of-band delete of an unreferenced source must succeed");
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_registry_history \
              WHERE entity_id = '22222222-2222-4222-8222-222222222222')",
        ),
        1,
        "audit evidence must survive deletion of the row it describes"
    );
}
