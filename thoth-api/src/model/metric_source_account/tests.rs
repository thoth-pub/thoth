//! Focused `MET-WP1-02` database tests for the `metric_source_account`
//! source-partition state, including the `(source_id, external_key)` identity
//! contract, the non-secret JSONB configuration column and the non-cascading
//! foreign keys to source, platform and publisher.
//!
//! Extended by `MET-WP2-01A` with the globally unique stable `code`: its
//! deterministic populated-database backfill, its nonblank and global
//! uniqueness rules, the requirement that new rows supply one explicitly, its
//! exact `TEXT` identity, and proof that none of it disturbed the existing
//! `external_key` identity contract.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

use super::MetricSourceAccount;
use crate::db::PgPool;
use crate::model::metric_platform::tests::{insert_platform_row, scalar_i64, setup_registry_db};
use crate::model::metric_source::tests::insert_source_row;
use crate::schema::metric_source_account;

/// Insert one referenced source/platform pair for account tests.
pub(crate) fn fixture_source_and_platform(pool: &PgPool) -> (Uuid, Uuid) {
    let source_id = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    insert_source_row(pool, source_id, "test_source");
    insert_platform_row(pool, platform_id, "test_platform");
    (source_id, platform_id)
}

/// Insert one `metric_source_account` row with an explicit id through raw SQL.
///
/// `MET-WP2-01A` made `code` mandatory. The call interface is deliberately
/// unchanged for the consuming import and checkpoint test modules: the helper
/// supplies the caller's own `source_account_id` as the stable code, which is
/// deterministic and globally unique without inventing a naming convention
/// that no specification has approved.
pub(crate) fn insert_account_row(
    pool: &PgPool,
    source_account_id: Uuid,
    source_id: Uuid,
    platform_id: Uuid,
    external_key: &str,
) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_account \
             (source_account_id, code, source_id, platform_id, external_key, enabled) \
         VALUES ($1, $2, $3, $4, $5, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Text, _>(source_account_id.to_string())
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Text, _>(external_key)
    .execute(&mut connection)
    .expect("Failed to insert metric_source_account fixture row");
}

/// Insert one `publisher` row for the optional expected-publisher FK.
fn insert_publisher_row(pool: &PgPool, publisher_id: Uuid) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("INSERT INTO publisher (publisher_id, publisher_name) VALUES ($1, 'Test publisher')")
        .bind::<diesel::sql_types::Uuid, _>(publisher_id)
        .execute(&mut connection)
        .expect("Failed to insert publisher fixture row");
}

/// Insert one account without an explicit id, supplying the stable code the
/// post-migration contract requires.
///
/// The code is an explicit parameter rather than something derived from
/// `external_key`, so a test can vary either identity independently and prove
/// that the source-scoped and global contracts are genuinely separate.
fn insert_account_raw(
    pool: &PgPool,
    source_id: Uuid,
    platform_id: Uuid,
    external_key: &str,
    code: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_account \
             (source_id, platform_id, external_key, code, enabled) \
         VALUES ($1, $2, $3, $4, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Text, _>(external_key)
    .bind::<diesel::sql_types::Text, _>(code)
    .execute(&mut connection)
}

/// Insert one account omitting `code` entirely, for the NOT NULL assertion.
fn insert_account_without_code(
    pool: &PgPool,
    source_id: Uuid,
    platform_id: Uuid,
    external_key: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_account \
             (source_id, platform_id, external_key, enabled) \
         VALUES ($1, $2, $3, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Text, _>(external_key)
    .execute(&mut connection)
}

fn delete_row(pool: &PgPool, table: &str, id_column: &str, id: Uuid) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!("DELETE FROM {table} WHERE {id_column} = $1"))
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut connection)
}

#[test]
fn migration_seeds_no_source_account_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        0,
        "MET-WP1-02 must not seed any metric_source_account row"
    );
}

#[test]
fn source_account_deliberately_has_no_timestamp_columns() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND table_name = 'metric_source_account' \
                AND column_name IN ('created_at', 'updated_at'))",
        ),
        0,
        "the approved design deliberately omits timestamps on metric_source_account"
    );
}

#[test]
fn blank_external_key_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    for (index, blank) in ["", " ", "   ", "\t", "\n"].into_iter().enumerate() {
        let result = insert_account_raw(
            &pool,
            source_id,
            platform_id,
            blank,
            &format!("code-{index}"),
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank external key {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn account_identity_is_unique_per_source_and_external_key() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    insert_account_raw(&pool, source_id, platform_id, "account-1", "code-1")
        .expect("First insert must pass");
    // A distinct code isolates the assertion: only the source-scoped
    // (source_id, external_key) identity can be what collides here.
    let duplicate = insert_account_raw(&pool, source_id, platform_id, "account-1", "code-2");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate (source_id, external_key) pair must fail the unique constraint: \
         {duplicate:?}"
    );

    // The same external key under another source is a different identity.
    let other_source_id = Uuid::new_v4();
    insert_source_row(&pool, other_source_id, "other_source");
    insert_account_raw(&pool, other_source_id, platform_id, "account-1", "code-3")
        .expect("The same external key under another source must pass");
}

#[test]
fn account_foreign_keys_require_existing_rows() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);

    for (case, result) in [
        (
            "unknown source",
            insert_account_raw(&pool, Uuid::new_v4(), platform_id, "account-1", "code-1"),
        ),
        (
            "unknown platform",
            insert_account_raw(&pool, source_id, Uuid::new_v4(), "account-2", "code-2"),
        ),
    ] {
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "an account referencing an {case} must fail the foreign key: {result:?}"
        );
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    let unknown_publisher = sql_query(
        "INSERT INTO metric_source_account \
             (source_id, platform_id, external_key, code, expected_publisher_id, enabled) \
         VALUES ($1, $2, 'account-3', 'code-3', $3, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .execute(&mut connection);
    assert!(
        matches!(
            unknown_publisher,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an account referencing an unknown publisher must fail the foreign key: \
         {unknown_publisher:?}"
    );
}

#[test]
fn deleting_referenced_rows_is_restricted_rather_than_cascaded() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);

    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_account \
             (source_id, platform_id, external_key, code, expected_publisher_id, enabled) \
         VALUES ($1, $2, 'account-1', 'code-1', $3, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(publisher_id)
    .execute(&mut connection)
    .expect("Failed to insert the referencing account row");
    drop(connection);

    for (table, id_column, id) in [
        ("metric_source", "source_id", source_id),
        ("metric_platform", "platform_id", platform_id),
        ("publisher", "publisher_id", publisher_id),
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
            "deleting a {table} row still referenced by a source account must be \
             restricted, not cascaded: {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        1,
        "the referencing account row must survive the restricted deletions"
    );
}

#[test]
fn configuration_is_non_null_jsonb_defaulting_to_an_empty_object() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    insert_account_raw(&pool, source_id, platform_id, "account-1", "code-1")
        .expect("Insert must pass");
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_account \
              WHERE external_key = 'account-1' AND configuration = '{}'::jsonb)",
        ),
        1,
        "an account inserted without configuration must default to the empty object"
    );

    let mut connection = pool.get().expect("Failed to get DB connection");
    let explicit_null = sql_query(
        "INSERT INTO metric_source_account \
             (source_id, platform_id, external_key, code, configuration, enabled) \
         VALUES ($1, $2, 'account-2', 'code-2', NULL, TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_id)
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .execute(&mut connection);
    assert!(
        matches!(
            explicit_null,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicitly NULL configuration must fail the NOT NULL constraint: \
         {explicit_null:?}"
    );
}

#[test]
fn metric_source_account_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let configuration = json!({
        "report_path": "reports/monthly",
        "grains": ["DAY", "MONTH"],
        "nested": {"retries_documented_elsewhere": true},
    });
    let configured_id: Uuid = diesel::insert_into(metric_source_account::table)
        .values((
            metric_source_account::source_id.eq(source_id),
            metric_source_account::platform_id.eq(platform_id),
            metric_source_account::external_key.eq("configured-account"),
            metric_source_account::code.eq("configured-account-code"),
            metric_source_account::expected_publisher_id.eq(publisher_id),
            metric_source_account::configuration.eq(configuration.clone()),
            metric_source_account::enabled.eq(true),
        ))
        .returning(metric_source_account::source_account_id)
        .get_result(&mut connection)
        .expect("Failed to insert configured account row");
    diesel::insert_into(metric_source_account::table)
        .values((
            metric_source_account::source_id.eq(source_id),
            metric_source_account::platform_id.eq(platform_id),
            metric_source_account::external_key.eq("bare-account"),
            metric_source_account::code.eq("bare-account-code"),
            metric_source_account::enabled.eq(false),
        ))
        .execute(&mut connection)
        .expect("Failed to insert bare account row");

    let configured: MetricSourceAccount = metric_source_account::table
        .filter(metric_source_account::external_key.eq("configured-account"))
        .first(&mut connection)
        .expect("Failed to load configured account row");
    assert_eq!(configured.source_account_id, configured_id);
    assert_eq!(configured.source_id, source_id);
    assert_eq!(configured.platform_id, platform_id);
    assert_eq!(configured.external_key, "configured-account");
    assert_eq!(configured.code, "configured-account-code");
    assert_eq!(configured.expected_publisher_id, Some(publisher_id));
    assert_eq!(configured.configuration, configuration);
    assert!(configured.enabled);

    let bare: MetricSourceAccount = metric_source_account::table
        .filter(metric_source_account::external_key.eq("bare-account"))
        .first(&mut connection)
        .expect("Failed to load bare account row");
    assert_eq!(bare.code, "bare-account-code");
    assert_eq!(bare.expected_publisher_id, None);
    assert_eq!(bare.configuration, json!({}));
    assert!(!bare.enabled);
}

// ---------------------------------------------------------------------------
// MET-WP2-01A: the globally unique stable source-account code.
// ---------------------------------------------------------------------------

#[test]
fn a_new_account_must_supply_a_stable_code() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    // The migration backfills pre-existing rows, but it prescribes nothing
    // about new ones: every row created after MET-WP2-01A supplies its own
    // code explicitly, and the database refuses to invent one.
    let result = insert_account_without_code(&pool, source_id, platform_id, "account-1");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an account created without a stable code must fail the NOT NULL \
         constraint rather than receive a generated one: {result:?}"
    );
}

#[test]
fn blank_stable_codes_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    for (index, blank) in ["", " ", "   ", "\t", "\n"].into_iter().enumerate() {
        let result = insert_account_raw(
            &pool,
            source_id,
            platform_id,
            &format!("account-{index}"),
            blank,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank stable code {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn the_stable_code_is_globally_unique_across_sources() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    insert_account_raw(&pool, source_id, platform_id, "account-1", "shared-code")
        .expect("the first account must be accepted");

    // Unlike external_key, the stable code is not scoped to its source: the
    // same code under a different source is still the same global identity
    // and must collide.
    let other_source_id = Uuid::new_v4();
    insert_source_row(&pool, other_source_id, "other_source");
    let duplicate = insert_account_raw(
        &pool,
        other_source_id,
        platform_id,
        "account-1",
        "shared-code",
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate stable code must fail the global unique constraint even \
         under another source: {duplicate:?}"
    );
}

#[test]
fn the_stable_code_is_exact_text_identity() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    // Codes differing only by case, surrounding whitespace, internal
    // separator or Unicode composition are distinct identities. Nothing
    // trims, folds, normalizes or aliases them, so all of these coexist.
    let codes = [
        "Account-Code",
        "account-code",
        "ACCOUNT-CODE",
        " account-code",
        "account-code ",
        "account_code",
        "account\u{00e9}",
        "account\u{0065}\u{0301}",
    ];
    for (index, code) in codes.into_iter().enumerate() {
        insert_account_raw(
            &pool,
            source_id,
            platform_id,
            &format!("account-{index}"),
            code,
        )
        .unwrap_or_else(|error| panic!("code {code:?} must be a distinct identity: {error:?}"));
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT code) FROM metric_source_account)"
        ),
        codes.len() as i64,
        "every code variant must remain a distinct exact-TEXT identity"
    );

    // Lookup is literal too: the stored code matches only itself.
    let mut connection = pool.get().expect("Failed to get DB connection");
    let exact: i64 = metric_source_account::table
        .filter(metric_source_account::code.eq("Account-Code"))
        .count()
        .get_result(&mut connection)
        .expect("Failed to count the exactly matching rows");
    assert_eq!(exact, 1, "an exact code must match exactly one row");
}

#[test]
fn the_stable_code_leaves_the_external_key_identity_contract_untouched() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    insert_account_raw(&pool, source_id, platform_id, "account-1", "code-1")
        .expect("the first account must be accepted");

    // Distinct codes cannot rescue a duplicated source-scoped identity...
    let duplicate_key = insert_account_raw(&pool, source_id, platform_id, "account-1", "code-2");
    assert!(
        matches!(
            duplicate_key,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "(source_id, external_key) must still be unique after MET-WP2-01A: {duplicate_key:?}"
    );

    // ...and a distinct external key still needs a nonblank value, so the
    // code did not become a substitute for it.
    let blank_key = insert_account_raw(&pool, source_id, platform_id, "  ", "code-3");
    assert!(
        matches!(
            blank_key,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::CheckViolation,
                _
            ))
        ),
        "the external_key nonblank check must survive MET-WP2-01A: {blank_key:?}"
    );

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_source_account'::regclass \
                AND conname = 'metric_source_account_source_id_external_key_key')",
        ),
        1,
        "the original source-scoped unique constraint must still exist under its \
         original name"
    );
}

#[test]
fn metric_source_account_carries_exactly_the_expected_code_constraints() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(&pool, "metric_source_account"),
        vec![
            "metric_source_account_code_check",
            "metric_source_account_external_key_check",
        ],
        "metric_source_account must carry exactly the original external_key check \
         and the one MET-WP2-01A code check"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_source_account'::regclass \
                AND attname = 'code' AND attnotnull)",
        ),
        1,
        "code must be NOT NULL after the migration"
    );
}
