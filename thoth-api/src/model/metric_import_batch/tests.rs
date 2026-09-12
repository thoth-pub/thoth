//! Focused `MET-WP2-01A` database tests for `metric_import_batch`: the exact
//! approved column contract, the batch-key identity rules, the non-cascading
//! import foreign key, the composite unique key that lets provenance
//! reference a batch only within its own import, and the deliberate absence of
//! any status, counter, completion or result-JSON column.
//!
//! This module also owns the migration evidence for the whole `MET-WP2-01A`
//! slice, because the slice's up/down migration is a single unit: the
//! populated-database source-account backfill, the inactive rollback, each of
//! the three fail-closed rollback guards independently, and
//! apply/revert/reapply through the embedded Diesel runner.
//!
//! These tests assert **schema** behaviour only. `MET-WP2-01A` implements no
//! ingestion, batching, replay or hashing behaviour, and nothing here pretends
//! otherwise.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricImportBatch;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::{
    check_constraint_names, fixture_source_account, insert_import_row,
};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_names};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_import_batch;

/// The Diesel migration version of `thoth-api/migrations/20260909_v1.9.0`.
pub(crate) const MET_WP2_01A_MIGRATION_VERSION: &str = "20260909";

/// Every column of the approved batch contract, with its exact type,
/// nullability and default.
///
/// The tuple is `(name, type, not_null, default)`. Stating all four per column
/// is the point: a column that silently became nullable, lost its default or
/// changed type would still satisfy a name-only inventory.
const BATCH_COLUMNS: [(&str, &str, bool, Option<&str>); 5] = [
    ("import_batch_id", "uuid", true, Some("uuid_generate_v4()")),
    ("import_id", "uuid", true, None),
    ("batch_key", "text", true, None),
    ("request_hash", "text", true, None),
    (
        "created_at",
        "timestamp with time zone",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
];

/// Column names that would betray a second per-row classification store, an
/// opaque result blob or speculative runtime state having been smuggled into
/// this slice. The approved contract names none of them.
const DEFERRED_BATCH_COLUMNS: [&str; 12] = [
    "accepted_count",
    "classification",
    "completed_at",
    "conflict_count",
    "duplicate_count",
    "outcome",
    "result",
    "result_json",
    "rejected_count",
    "row_count",
    "status",
    "updated_at",
];

/// Revert migrations until the `MET-WP2-01A` ingestion-contract migration
/// itself has been reverted.
///
/// Reverting *through* the target rather than calling `revert_last_migration`
/// once keeps the meaning under any future migration order, exactly as every
/// predecessor Metrics slice does.
pub(crate) fn revert_through_ingestion_contract_migration(connection: &mut PgConnection) {
    let applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP2_01A_MIGRATION_VERSION);
    assert!(
        applied,
        "the MET-WP2-01A ingestion-contract migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP2_01A_MIGRATION_VERSION {
            return;
        }
    }
}

/// Attempt to revert the `MET-WP2-01A` migration, returning its error.
///
/// The guard tests need the failure itself, so this deliberately does not
/// unwrap. Reverting *through* the newer migrations first keeps the meaning
/// under any future migration order, exactly as
/// [`revert_through_ingestion_contract_migration`] does: the error returned is
/// always `MET-WP2-01A`'s, never whichever migration happens to be newest.
///
/// `revert_last_migration` reverts the greatest applied version, so the same
/// `applied_migrations` list Diesel itself consults says which migration the
/// next call will target. Anything newer is peeled off first and must succeed;
/// a newer migration refusing its own rollback is a different failure and is
/// reported as one rather than being mistaken for this guard.
fn try_revert_ingestion_contract_migration(connection: &mut PgConnection) -> String {
    let applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations");
    assert!(
        applied
            .iter()
            .any(|version| version.to_string() == MET_WP2_01A_MIGRATION_VERSION),
        "the MET-WP2-01A ingestion-contract migration must be applied before reverting through it"
    );

    loop {
        let next = connection
            .applied_migrations()
            .expect("Failed to read applied migrations")
            .first()
            .expect("the MET-WP2-01A ingestion-contract migration must still be applied")
            .to_string();
        if next == MET_WP2_01A_MIGRATION_VERSION {
            return match connection.revert_last_migration(MIGRATIONS) {
                Ok(version) => panic!(
                    "the rollback must have failed closed, but it reverted {version} \
                     and discarded the MET-WP2-01A contract"
                ),
                Err(error) => error.to_string(),
            };
        }
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to revert {next}, a migration newer than MET-WP2-01A, \
                     before reaching the MET-WP2-01A rollback guard: {error}"
                )
            });
        assert_eq!(
            reverted.to_string(),
            next,
            "reverting must have targeted the newest applied migration"
        );
    }
}

/// One scalar `BIGINT` result on a caller-owned connection.
///
/// The migration tests drive migrations on their own connection and cannot use
/// the pool-based [`scalar_i64`]: reverting and reapplying changes table
/// shapes a pooled connection may still have cached.
fn on_connection(connection: &mut PgConnection, query: &str) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(connection)
        .expect("Failed to run scalar query")
}

/// Insert one batch row through raw SQL so database defaults are exercised
/// rather than restated by a Diesel fixture.
fn insert_batch_raw(
    pool: &PgPool,
    import_id: Uuid,
    batch_key: &str,
    request_hash: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_import_batch (import_id, batch_key, request_hash) \
         VALUES ($1, $2, $3)",
    )
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Text, _>(batch_key)
    .bind::<diesel::sql_types::Text, _>(request_hash)
    .execute(&mut connection)
}

/// One import under a fresh source account.
fn fixture_import(pool: &PgPool) -> Uuid {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);
    import_id
}

/// A second import under the same source account as `import_id`.
///
/// Batch-key identity is scoped to the import, not to the account, so reusing
/// one account is the sharper fixture: it proves the scope is genuinely the
/// import.
fn sibling_import(pool: &PgPool, import_id: Uuid) -> Uuid {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let source_account_id: Uuid = diesel::select(diesel::dsl::sql::<diesel::sql_types::Uuid>(
        &format!("(SELECT source_account_id FROM metric_import WHERE import_id = '{import_id}')"),
    ))
    .get_result(&mut connection)
    .expect("Failed to read the source account of the existing import");
    drop(connection);

    let sibling_id = Uuid::new_v4();
    insert_import_row(pool, sibling_id, source_account_id);
    sibling_id
}

#[test]
fn migration_seeds_no_batch_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_batch)"),
        0,
        "MET-WP2-01A must not seed any metric_import_batch row"
    );
}

#[test]
fn the_batch_table_has_exactly_the_approved_column_contract() {
    let (_guard, pool) = setup_registry_db();

    for (name, sql_type, not_null, default) in BATCH_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute a \
                       LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
                      WHERE a.attrelid = 'metric_import_batch'::regclass \
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
              WHERE attrelid = 'metric_import_batch'::regclass \
                AND attnum > 0 AND NOT attisdropped)",
        ),
        BATCH_COLUMNS.len() as i64,
        "the batch table must carry exactly the approved columns and no others"
    );

    for column in DEFERRED_BATCH_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute \
                      WHERE attrelid = 'metric_import_batch'::regclass \
                        AND attname = '{column}' AND NOT attisdropped)"
                ),
            ),
            0,
            "`{column}` must not exist: the authoritative per-row outcome is \
             metric_record_provenance, and this slice stores no duplicate \
             classification, counter, status or opaque result"
        );
    }

    // No JSON column of any name, so an opaque result blob cannot have been
    // smuggled in under a different label.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_attribute a \
              WHERE a.attrelid = 'metric_import_batch'::regclass \
                AND a.attnum > 0 AND NOT a.attisdropped \
                AND format_type(a.atttypid, a.atttypmod) IN ('json', 'jsonb'))",
        ),
        0,
        "the batch table must carry no JSON column at all"
    );
}

#[test]
fn the_batch_table_has_exactly_the_approved_constraints() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        check_constraint_names(&pool, "metric_import_batch"),
        vec![
            "metric_import_batch_batch_key_check",
            "metric_import_batch_request_hash_check",
        ],
        "metric_import_batch must carry exactly the two authorized nonblank CHECKs"
    );
    assert_eq!(
        index_names(&pool, "metric_import_batch"),
        vec![
            "metric_import_batch_import_id_batch_key_key",
            "metric_import_batch_import_id_import_batch_id_key",
            "metric_import_batch_pkey",
        ],
        "metric_import_batch must carry exactly its primary key and the two \
         approved unique keys, with no speculative index"
    );

    let keys = foreign_keys(&pool, "metric_import_batch");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_import_batch_import_id_fkey"],
        "metric_import_batch must carry exactly the one authorized foreign key"
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
fn the_composite_unique_key_covers_exactly_import_and_batch_id() {
    let (_guard, pool) = setup_registry_db();
    // This key exists only so metric_record_provenance can reference a batch
    // by (import_id, import_batch_id) and therefore cannot reach a batch in
    // another import. If its column set ever changed, that referential
    // guarantee would silently disappear.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_import_batch'::regclass \
                AND conname = 'metric_import_batch_import_id_import_batch_id_key' \
                AND contype = 'u' \
                AND (SELECT array_agg(attname::text ORDER BY attname) \
                       FROM pg_attribute \
                      WHERE attrelid = conrelid AND attnum = ANY(conkey)) \
                    = ARRAY['import_batch_id', 'import_id'])",
        ),
        1,
        "the composite unique key must cover exactly (import_id, import_batch_id)"
    );
}

#[test]
fn batch_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    insert_batch_raw(&pool, import_id, "batch-1", "request-hash-1")
        .expect("Failed to insert the defaulted batch row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricImportBatch = metric_import_batch::table
        .first(&mut connection)
        .expect("Failed to load the defaulted batch row");
    assert_ne!(
        loaded.import_batch_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate an import_batch_id"
    );
    assert_eq!(loaded.import_id, import_id);
    assert_eq!(loaded.batch_key, "batch-1");
    assert_eq!(loaded.request_hash, "request-hash-1");
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
}

#[test]
fn blank_batch_keys_and_request_hashes_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    for blank in ["", " ", "   ", "\t", "\n"] {
        for (batch_key, request_hash) in [(blank, "request-hash-1"), ("batch-1", blank)] {
            let result = insert_batch_raw(&pool, import_id, batch_key, request_hash);
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "a blank value ({blank:?}) must be rejected by a check constraint, got {result:?}"
            );
        }
    }
}

#[test]
fn a_batch_requires_an_existing_import_and_cannot_cascade_it_away() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);

    let unknown_import = insert_batch_raw(&pool, Uuid::new_v4(), "batch-1", "request-hash-1");
    assert!(
        matches!(
            unknown_import,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a batch referencing an unknown import must be rejected, got {unknown_import:?}"
    );

    insert_batch_raw(&pool, import_id, "batch-1", "request-hash-1")
        .expect("the referencing batch row must be accepted");
    let deletion = delete_row(&pool, "metric_import", "import_id", import_id);
    assert!(
        matches!(
            deletion,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced import must be restricted, not cascade away durable \
         batch evidence, got {deletion:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_batch)"),
        1,
        "the batch row must survive the restricted deletion"
    );
}

#[test]
fn a_batch_key_is_unique_within_its_import_and_reusable_across_imports() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    insert_batch_raw(&pool, import_id, "batch-1", "request-hash-1")
        .expect("the first batch must be accepted");

    // The same key under the same import is the same batch attempt. Deciding
    // what a differing request hash then means is MET-WP2-01B behaviour; the
    // database simply refuses the duplicate identity, whether the hash
    // matches or not.
    for request_hash in ["request-hash-1", "request-hash-2"] {
        let duplicate = insert_batch_raw(&pool, import_id, "batch-1", request_hash);
        assert!(
            matches!(
                duplicate,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _
                ))
            ),
            "a duplicate (import_id, batch_key) must fail regardless of request hash \
             ({request_hash}), got {duplicate:?}"
        );
    }

    // Distinct keys under one import are distinct batches.
    insert_batch_raw(&pool, import_id, "batch-2", "request-hash-2")
        .expect("a second batch key under the same import must be accepted");

    // The same key under another import is a different batch entirely.
    let other_import_id = sibling_import(&pool, import_id);
    insert_batch_raw(&pool, other_import_id, "batch-1", "request-hash-1")
        .expect("the same batch key under another import must be accepted");

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_batch)"),
        3
    );
}

#[test]
fn metric_import_batch_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let import_batch_id: Uuid = diesel::insert_into(metric_import_batch::table)
        .values((
            metric_import_batch::import_id.eq(import_id),
            metric_import_batch::batch_key.eq("batch-key-42"),
            metric_import_batch::request_hash.eq("request-hash-42"),
        ))
        .returning(metric_import_batch::import_batch_id)
        .get_result(&mut connection)
        .expect("Failed to insert the batch row through Diesel");

    let loaded: MetricImportBatch = metric_import_batch::table
        .filter(metric_import_batch::import_batch_id.eq(import_batch_id))
        .first(&mut connection)
        .expect("Failed to load the batch row through Diesel");
    assert_eq!(loaded.import_batch_id, import_batch_id);
    assert_eq!(loaded.import_id, import_id);
    assert_eq!(loaded.batch_key, "batch-key-42");
    assert_eq!(loaded.request_hash, "request-hash-42");
    assert!(loaded.created_at > Timestamp::default());
}

#[test]
fn the_request_hash_is_opaque_nonblank_text_with_no_imposed_algorithm() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    // MET-WP2-01A stores the hash and fixes no canonicalization, encoding or
    // length: choosing those belongs to MET-WP2-01B. Anything nonblank is
    // storable, and nothing here may pretend a format was decided.
    for (index, request_hash) in [
        "0",
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "not-a-hash-at-all",
        "\u{00e9}",
    ]
    .into_iter()
    .enumerate()
    {
        insert_batch_raw(&pool, import_id, &format!("batch-{index}"), request_hash)
            .unwrap_or_else(|error| panic!("request hash {request_hash:?} must store: {error:?}"));
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_batch)"),
        4
    );
}

// ---------------------------------------------------------------------------
// MET-WP2-01A migration evidence.
// ---------------------------------------------------------------------------

/// Insert one pre-`MET-WP2-01A` source account directly on a caller-owned
/// connection, without a stable code.
///
/// This is only usable while the migration is reverted, which is exactly the
/// point: it produces the populated pre-01A state the backfill must handle.
fn insert_pre_migration_account(
    connection: &mut PgConnection,
    source_account_id: Uuid,
    source_id: Uuid,
    platform_id: Uuid,
    external_key: &str,
) {
    sql_query(
        "INSERT INTO metric_source_account \
             (source_account_id, source_id, platform_id, external_key, enabled) \
         VALUES ($1, $2, $3, $4, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Text, _>(external_key)
    .execute(connection)
    .expect("Failed to insert the pre-migration source account row");
}

#[test]
fn a_populated_database_migrates_reverts_and_reapplies_deterministically() {
    // The pool is deliberately unused: this test drives migrations on its own
    // connection, because reverting and reapplying changes table shapes a
    // pooled connection could still have cached.
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    // Go back to the pre-01A contract and populate it, so the backfill is
    // exercised against real rows rather than an empty table.
    revert_through_ingestion_contract_migration(&mut connection);
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_source_account'::regclass \
                AND attname = 'code' AND NOT attisdropped)",
        ),
        0,
        "the pre-01A contract must have no source-account code column"
    );

    let source_id = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    let accounts = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    sql_query(
        "INSERT INTO metric_source (source_id, code, acquisition_type, enabled) \
         VALUES ($1, 'pre_migration_source', 'PUBLISHER_UPLOAD', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .execute(&mut connection)
    .expect("Failed to insert the pre-migration source row");
    sql_query(
        "INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) \
         VALUES ($1, 'pre_migration_platform', 'Pre-migration platform', 'EXTERNAL', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .execute(&mut connection)
    .expect("Failed to insert the pre-migration platform row");
    for (index, source_account_id) in accounts.into_iter().enumerate() {
        insert_pre_migration_account(
            &mut connection,
            source_account_id,
            source_id,
            platform_id,
            &format!("pre-migration-account-{index}"),
        );
    }

    // Apply MET-WP2-01A over the populated table.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply MET-WP2-01A over a populated database");

    for source_account_id in accounts {
        assert_eq!(
            on_connection(
                &mut connection,
                &format!(
                    "(SELECT COUNT(*) FROM metric_source_account \
                      WHERE source_account_id = '{source_account_id}' \
                        AND code = '{source_account_id}')"
                ),
            ),
            1,
            "every pre-existing row must be backfilled with exactly its own \
             source_account_id::text"
        );
    }
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_source_account \
              WHERE code IS DISTINCT FROM source_account_id::text)",
        ),
        0,
        "the backfill must be deterministic for every row, with no exception"
    );
    // The pre-existing source-scoped identities survive the migration intact.
    for (index, source_account_id) in accounts.into_iter().enumerate() {
        assert_eq!(
            on_connection(
                &mut connection,
                &format!(
                    "(SELECT COUNT(*) FROM metric_source_account \
                      WHERE source_account_id = '{source_account_id}' \
                        AND source_id = '{source_id}' \
                        AND external_key = 'pre-migration-account-{index}')"
                ),
            ),
            1,
            "the migration must not disturb existing external_key identity"
        );
    }

    // The contract is still inactive, so rollback is permitted and exact.
    revert_through_ingestion_contract_migration(&mut connection);
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_source_account'::regclass \
                AND attname = 'code' AND NOT attisdropped)",
        ),
        0,
        "the rollback must remove the stable source-account code"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_import_batch')",
        ),
        0,
        "the rollback must remove the batch table"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_record_provenance'::regclass \
                AND attname IN ('import_batch_id', 'batch_row_index') \
                AND NOT attisdropped)",
        ),
        0,
        "the rollback must remove the provenance batch linkage columns"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_record_provenance'::regclass \
                AND attname IN ('identity_hash', 'content_hash') AND attnotnull)",
        ),
        2,
        "the rollback must restore both provenance hashes to NOT NULL"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_record_overlap_lookup_idx')",
        ),
        0,
        "the rollback must remove the overlap index"
    );
    // The populated pre-01A rows themselves survive untouched.
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_source_account \
              WHERE external_key LIKE 'pre-migration-account-%')",
        ),
        accounts.len() as i64,
        "the rollback must preserve the pre-existing source-account rows"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_source_account'::regclass \
                AND conname IN ('metric_source_account_source_id_external_key_key', \
                                'metric_source_account_external_key_check'))",
        ),
        2,
        "the rollback must leave the pre-01A source-account contract intact"
    );
    // Nothing outside MET-WP2-01A was reversed.
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
                                'metric_reconciliation_run', 'metric_reconciliation_issue', \
                                'metric_registry_history'))",
        ),
        20,
        "the rollback must leave the MET-WP1-01..12 schema in place"
    );

    // Reapply: the whole contract returns, and the backfill is deterministic
    // a second time.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply MET-WP2-01A");
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_source_account \
              WHERE code IS DISTINCT FROM source_account_id::text)",
        ),
        0,
        "the reapplied backfill must be identical"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_import_batch)",
        ),
        0,
        "the reapplied migration seeds no batch row"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_record_overlap_lookup_idx')",
        ),
        1,
        "the reapplied migration must restore the overlap index"
    );
}

#[test]
fn rollback_fails_closed_when_a_batch_row_exists() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    insert_batch_raw(&pool, import_id, "batch-1", "request-hash-1")
        .expect("the batch row must be accepted");

    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    let error = try_revert_ingestion_contract_migration(&mut connection);
    assert!(
        error.contains("metric_import_batch holds"),
        "the rollback must refuse to discard committed batch evidence: {error}"
    );

    // The refusal must leave the contract completely intact.
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_import_batch)",
        ),
        1,
        "the refused rollback must preserve the batch row"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_record_overlap_lookup_idx')",
        ),
        1,
        "the refused rollback must not have partially removed the contract"
    );
}

#[test]
fn rollback_fails_closed_when_a_provenance_hash_is_null() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) =
        crate::model::metric_record_revision::tests::fixture_record(&pool, "identity-a");
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        // Only the amended REJECTED invariant can have written this, so its
        // presence proves the new contract is already in use.
        sql_query(
            "INSERT INTO metric_record_provenance (import_id, classification) \
             VALUES ($1, 'REJECTED')",
        )
        .bind::<diesel::sql_types::Uuid, _>(fixture.import_id)
        .execute(&mut connection)
        .expect("a fully unhashed REJECTED row must be accepted");
    }

    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    let error = try_revert_ingestion_contract_migration(&mut connection);
    assert!(
        error.contains("NULL"),
        "the rollback must refuse to destroy or fabricate rejection evidence: {error}"
    );

    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_record_provenance \
              WHERE identity_hash IS NULL AND content_hash IS NULL)",
        ),
        1,
        "the refused rollback must preserve the unhashed rejection row"
    );
}

#[test]
fn rollback_fails_closed_when_a_stable_code_was_assigned() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) =
        crate::model::metric_source_account::tests::fixture_source_and_platform(&pool);
    let source_account_id = Uuid::new_v4();
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        // A code that is not the deterministic backfill value is assigned
        // identity: nothing else in the schema records it, so dropping the
        // column would lose it permanently.
        sql_query(
            "INSERT INTO metric_source_account \
                 (source_account_id, code, source_id, platform_id, external_key, enabled) \
             VALUES ($1, 'operator-assigned-code', $2, $3, 'account-1', TRUE)",
        )
        .bind::<diesel::sql_types::Uuid, _>(source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(source_id)
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .execute(&mut connection)
        .expect("the assigned-code account row must be accepted");
    }

    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    let error = try_revert_ingestion_contract_migration(&mut connection);
    assert!(
        error.contains("stable code"),
        "the rollback must refuse to discard assigned stable identity: {error}"
    );

    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_source_account \
              WHERE code = 'operator-assigned-code')",
        ),
        1,
        "the refused rollback must preserve the assigned code"
    );
}

#[test]
fn a_deterministically_backfilled_account_does_not_block_rollback() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) =
        crate::model::metric_source_account::tests::fixture_source_and_platform(&pool);
    let source_account_id = Uuid::new_v4();
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        // The mirror image of the guard test above: a row whose code still
        // equals its deterministic backfill value carries no assigned
        // identity, so removing the column loses nothing and the rollback is
        // allowed to proceed.
        sql_query(
            "INSERT INTO metric_source_account \
                 (source_account_id, code, source_id, platform_id, external_key, enabled) \
             VALUES ($1, $1::text, $2, $3, 'account-1', TRUE)",
        )
        .bind::<diesel::sql_types::Uuid, _>(source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(source_id)
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .execute(&mut connection)
        .expect("the backfill-valued account row must be accepted");
    }

    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    revert_through_ingestion_contract_migration(&mut connection);
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_source_account'::regclass \
                AND attname = 'code' AND NOT attisdropped)",
        ),
        0,
        "an inactive contract must still be reversible"
    );
    assert_eq!(
        on_connection(
            &mut connection,
            &format!(
                "(SELECT COUNT(*) FROM metric_source_account \
                  WHERE source_account_id = '{source_account_id}')"
            ),
        ),
        1,
        "the account row itself must survive the rollback"
    );
}
