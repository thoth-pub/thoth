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
//!
//! Extended again by `MET-WP1-13` with the protected administration
//! coordinator and the closed typed configuration: the canonicalization and
//! stored-configuration decoder truth tables, exact-code create/lookup/update,
//! the source compatibility matrix, fail-closed handling of unsupported stored
//! values, semantic-JSONB no-op detection, atomic audit and rollback, one-row
//! locking, and serialized concurrent updates.

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

// --------------------------------------------------------------------------
// `MET-WP1-13` protected administration coordinator and typed configuration
// --------------------------------------------------------------------------

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use diesel::Connection;

use super::crud::{
    create_metric_source_account, metric_source_account_by_code, update_metric_source_account,
};
use super::{
    ConfigurationError, MetricCloudFrontLegacyS3ConfigurationInput,
    MetricSourceAccountConfiguration, MetricSourceAccountConfigurationInput,
    MetricSourceAccountConfigurationKind, NewMetricSourceAccount, PatchMetricSourceAccount,
};
use crate::model::metric_platform::tests::{serialized_update_chain, FORBIDDEN_JOIN_CONSTRUCTS};
use crate::model::metric_source::crud::create_metric_source;
use crate::model::metric_source::{MetricSourceAcquisitionType, NewMetricSource};
use crate::model::metric_source_registry_history::tests::source_audit_rows;
use serde_json::Value as JsonValue;
use thoth_errors::ThothError;

/// A fictional hostname used as `external_key` throughout. No real provider
/// value appears anywhere in this module.
const HOST: &str = "cdn.example-press.test";

/// The fixture publisher every CloudFront account is pinned to (Specification
/// Amendment 2). Deterministic and fictional.
const FIXTURE_PUBLISHER_ID: Uuid = Uuid::from_u128(0x5f5f_0000_0000_4000_8000_0000_0000_0001);

fn empty() -> MetricSourceAccountConfigurationInput {
    MetricSourceAccountConfigurationInput {
        kind: MetricSourceAccountConfigurationKind::Empty,
        cloudfront_legacy_s3: None,
    }
}

fn cloudfront(hostname: &str, bucket: &str, prefix: &str) -> MetricSourceAccountConfigurationInput {
    MetricSourceAccountConfigurationInput {
        kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
        cloudfront_legacy_s3: Some(MetricCloudFrontLegacyS3ConfigurationInput {
            hostname: hostname.to_string(),
            bucket: bucket.to_string(),
            prefix: prefix.to_string(),
        }),
    }
}

/// Create the two fixture sources through the real coordinator: one CloudFront
/// driver source and one admin-import source, plus one platform.
fn fixture_sources_and_platform(pool: &PgPool) -> Uuid {
    create_metric_source(
        pool,
        "fixture",
        &NewMetricSource {
            code: "cf_source".to_string(),
            acquisition_type: MetricSourceAcquisitionType::Driver,
            driver_key: Some("cloudfront".to_string()),
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        },
    )
    .expect("cloudfront source");
    create_metric_source(
        pool,
        "fixture",
        &NewMetricSource {
            code: "admin_source".to_string(),
            acquisition_type: MetricSourceAcquisitionType::AdminImport,
            driver_key: None,
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        },
    )
    .expect("admin source");
    let platform_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, "fixture_platform");
    // A distinct name: `insert_publisher_row` uses a fixed one, and tests that
    // plant a second publisher would otherwise trip `publisher_uniq_idx`.
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("INSERT INTO publisher (publisher_id, publisher_name) VALUES ($1, 'Fixture Press')")
        .bind::<diesel::sql_types::Uuid, _>(FIXTURE_PUBLISHER_ID)
        .execute(&mut connection)
        .expect("Failed to insert the fixture publisher");
    platform_id
}

fn new_account(
    code: &str,
    source_code: &str,
    external_key: &str,
    configuration: MetricSourceAccountConfigurationInput,
) -> NewMetricSourceAccount {
    NewMetricSourceAccount {
        code: code.to_string(),
        source_code: source_code.to_string(),
        platform_code: "fixture_platform".to_string(),
        external_key: external_key.to_string(),
        // Pinned for every fixture account so CloudFront creation satisfies
        // Amendment 2; the EMPTY-without-publisher case is tested explicitly.
        expected_publisher_id: Some(FIXTURE_PUBLISHER_ID),
        configuration,
        enabled: true,
    }
}

fn patch_account(
    code: &str,
    configuration: MetricSourceAccountConfigurationInput,
    enabled: bool,
) -> PatchMetricSourceAccount {
    PatchMetricSourceAccount {
        code: code.to_string(),
        configuration,
        enabled,
    }
}

/// Overwrite one account's stored JSON through raw SQL, bypassing the typed
/// surface, to plant pre-existing or unsupported values.
fn plant_stored_configuration(pool: &PgPool, code: &str, json_text: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("UPDATE metric_source_account SET configuration = $1::jsonb WHERE code = $2")
        .bind::<diesel::sql_types::Text, _>(json_text)
        .bind::<diesel::sql_types::Text, _>(code)
        .execute(&mut connection)
        .expect("plant stored configuration");
}

fn expect_constraint_message(error: &ThothError, expected: &str, label: &str) {
    match error {
        ThothError::DatabaseConstraintError(message) => {
            assert_eq!(message.as_ref(), expected, "{label}")
        }
        other => panic!("{label}: expected a bounded message, got {other:?}"),
    }
}

// ---- pure decoder truth table ---------------------------------------------

#[test]
fn canonicalization_follows_the_exact_truth_table() {
    let (json, decoded) = empty().canonicalize(HOST).expect("EMPTY");
    assert_eq!(json, json!({}));
    assert_eq!(decoded.kind, MetricSourceAccountConfigurationKind::Empty);
    assert!(decoded.cloudfront_legacy_s3.is_none());

    let (json, decoded) = cloudfront(HOST, " Bucket-One ", "logs/prefix/")
        .canonicalize(HOST)
        .expect("CLOUDFRONT");
    // Exact key set at every level, fixed constants, exact supplied values —
    // and no key ordering, whitespace or byte-level assumption anywhere.
    let object = json.as_object().expect("object");
    let mut keys: Vec<&String> = object.keys().collect();
    keys.sort();
    assert_eq!(keys, ["hostname", "logging", "schemaVersion"]);
    assert_eq!(object["schemaVersion"], "cloudfront-source-account/1");
    assert_eq!(object["hostname"], HOST);
    let logging = object["logging"].as_object().expect("logging object");
    let mut logging_keys: Vec<&String> = logging.keys().collect();
    logging_keys.sort();
    assert_eq!(logging_keys, ["bucket", "mode", "prefix"]);
    assert_eq!(logging["mode"], "LEGACY_S3");
    assert_eq!(
        logging["bucket"], " Bucket-One ",
        "stored exactly, untrimmed"
    );
    assert_eq!(logging["prefix"], "logs/prefix/");
    assert_eq!(
        decoded.cloudfront_legacy_s3.expect("payload").bucket,
        " Bucket-One "
    );

    for (label, input, expected) in [
        (
            "EMPTY with payload",
            MetricSourceAccountConfigurationInput {
                kind: MetricSourceAccountConfigurationKind::Empty,
                cloudfront_legacy_s3: cloudfront(HOST, "b", "p").cloudfront_legacy_s3,
            },
            ConfigurationError::EmptyWithPayload,
        ),
        (
            "CLOUDFRONT without payload",
            MetricSourceAccountConfigurationInput {
                kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
                cloudfront_legacy_s3: None,
            },
            ConfigurationError::CloudFrontWithoutPayload,
        ),
        (
            "blank hostname",
            cloudfront("  ", "b", "p"),
            ConfigurationError::BlankRoutingValue,
        ),
        (
            "blank bucket",
            cloudfront(HOST, "", "p"),
            ConfigurationError::BlankRoutingValue,
        ),
        (
            "blank prefix",
            cloudfront(HOST, "b", "\t\n"),
            ConfigurationError::BlankRoutingValue,
        ),
        (
            "hostname differs by case",
            cloudfront("CDN.example-press.test", "b", "p"),
            ConfigurationError::HostnameMismatch,
        ),
        (
            "hostname differs by whitespace",
            cloudfront("cdn.example-press.test ", "b", "p"),
            ConfigurationError::HostnameMismatch,
        ),
    ] {
        assert_eq!(input.canonicalize(HOST).unwrap_err(), expected, "{label}");
    }
}

#[test]
fn the_stored_configuration_decoder_accepts_exactly_two_shapes() {
    let ok_empty = MetricSourceAccountConfiguration::decode_stored(&json!({}), HOST).expect("{}");
    assert_eq!(ok_empty.kind, MetricSourceAccountConfigurationKind::Empty);

    let canonical = json!({
        "schemaVersion": "cloudfront-source-account/1",
        "hostname": HOST,
        "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"},
    });
    let decoded = MetricSourceAccountConfiguration::decode_stored(&canonical, HOST).expect("cf");
    assert_eq!(
        decoded.kind,
        MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1
    );
    let payload = decoded.cloudfront_legacy_s3.expect("payload");
    assert_eq!(
        (
            payload.hostname.as_str(),
            payload.bucket.as_str(),
            payload.prefix.as_str()
        ),
        (HOST, "b", "p")
    );

    // Key order is not a contract semantic: the same value with keys in a
    // different order decodes identically.
    let reordered: JsonValue = serde_json::from_str(
        r#"{"logging":{"prefix":"p","bucket":"b","mode":"LEGACY_S3"},"hostname":"cdn.example-press.test","schemaVersion":"cloudfront-source-account/1"}"#,
    )
    .expect("json");
    assert_eq!(
        reordered, canonical,
        "serde_json::Value equality is semantic"
    );
    assert_eq!(
        MetricSourceAccountConfiguration::decode_stored(&reordered, HOST).expect("reordered"),
        MetricSourceAccountConfiguration::decode_stored(&canonical, HOST).expect("canonical")
    );

    let unsupported: [(&str, JsonValue); 16] = [
        ("array", json!([])),
        ("string", json!("{}")),
        ("null", JsonValue::Null),
        (
            "generic legacy object",
            json!({"legacy": true, "token": "not-a-real-secret"}),
        ),
        (
            "extra top-level key",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"}, "extra": 1}),
        ),
        (
            "extra logging key",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p", "region": "x"}}),
        ),
        (
            "missing logging key",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "b"}}),
        ),
        (
            "unknown schema version",
            json!({"schemaVersion": "cloudfront-source-account/2", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"}}),
        ),
        (
            "unknown logging mode",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "STANDARD_V2", "bucket": "b", "prefix": "p"}}),
        ),
        (
            "blank bucket",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "  ", "prefix": "p"}}),
        ),
        (
            "non-string prefix",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": 7}}),
        ),
        (
            "logging not an object",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": HOST,
                   "logging": "LEGACY_S3"}),
        ),
        (
            "hostname inconsistent with external key",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": "other.test",
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"}}),
        ),
        (
            "hostname case variant",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": "CDN.example-press.test",
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"}}),
        ),
        (
            "empty string hostname",
            json!({"schemaVersion": "cloudfront-source-account/1", "hostname": "",
                   "logging": {"mode": "LEGACY_S3", "bucket": "b", "prefix": "p"}}),
        ),
        (
            "schemaVersion only",
            json!({"schemaVersion": "cloudfront-source-account/1"}),
        ),
    ];
    for (label, stored) in unsupported {
        assert_eq!(
            MetricSourceAccountConfiguration::decode_stored(&stored, HOST).unwrap_err(),
            ConfigurationError::UnsupportedStored,
            "{label} must fail closed"
        );
    }
}

// ---- create ----------------------------------------------------------------

#[test]
fn create_resolves_codes_persists_canonical_json_and_audits_the_persisted_row() {
    let (_guard, pool) = setup_registry_db();
    let platform_id = fixture_sources_and_platform(&pool);

    let cf = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account(
            "cf_account",
            "cf_source",
            HOST,
            cloudfront(HOST, "bucket-a", "prefix/a/"),
        ),
    )
    .expect("cloudfront account");
    assert_eq!(cf.code, "cf_account");
    assert_eq!(cf.platform_id, platform_id);
    assert_eq!(cf.external_key, HOST);
    assert_eq!(cf.expected_publisher_id, Some(FIXTURE_PUBLISHER_ID));
    assert!(cf.enabled);
    assert_eq!(
        cf.configuration,
        json!({
            "schemaVersion": "cloudfront-source-account/1",
            "hostname": HOST,
            "logging": {"mode": "LEGACY_S3", "bucket": "bucket-a", "prefix": "prefix/a/"},
        }),
        "the persisted JSONB must be semantically exactly the approved shape"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source_account WHERE code = 'cf_account' \
                AND configuration = '{\"schemaVersion\":\"cloudfront-source-account/1\",\
                \"hostname\":\"cdn.example-press.test\",\"logging\":{\"mode\":\"LEGACY_S3\",\
                \"bucket\":\"bucket-a\",\"prefix\":\"prefix/a/\"}}'::jsonb)",
        ),
        1,
        "PostgreSQL JSONB equality must agree"
    );

    let empty_account = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("admin_account", "admin_source", "partition-1", empty()),
    )
    .expect("admin account");
    assert_eq!(empty_account.configuration, json!({}));

    assert_eq!(
        metric_source_account_by_code(&pool, "cf_account").expect("lookup"),
        cf
    );
    let decoded = cf.decoded_configuration().expect("decoded");
    assert_eq!(
        decoded.kind,
        MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1
    );

    // Two fixture-source CREATEs plus two account CREATEs, each with the exact
    // persisted row and no before state.
    let audit = source_audit_rows(&pool);
    let accounts: Vec<_> = audit
        .iter()
        .filter(|row| row.entity == "SOURCE_ACCOUNT")
        .collect();
    assert_eq!(accounts.len(), 2);
    assert!(accounts
        .iter()
        .all(|row| row.action == "CREATE" && row.before_state.is_none()));
    let cf_audit = accounts
        .iter()
        .find(|row| row.entity_id == cf.source_account_id)
        .expect("cf audit");
    assert_eq!(
        cf_audit.after_state,
        serde_json::to_value(&cf).expect("serialize")
    );
    assert_eq!(
        cf_audit.after_state["configuration"]["logging"]["mode"],
        "LEGACY_S3"
    );
    assert_eq!(cf_audit.actor, "actor-1");
}

#[test]
fn create_enforces_the_source_compatibility_matrix_and_the_truth_table_before_writing() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);

    let cases = [
        (
            "cloudfront source with EMPTY",
            new_account("a1", "cf_source", HOST, empty()),
            ConfigurationError::SourceRequiresCloudFront.message(),
        ),
        (
            "admin source with CloudFront",
            new_account("a2", "admin_source", HOST, cloudfront(HOST, "b", "p")),
            ConfigurationError::SourceRequiresEmpty.message(),
        ),
        (
            "EMPTY with payload on admin source",
            new_account(
                "a3",
                "admin_source",
                HOST,
                MetricSourceAccountConfigurationInput {
                    kind: MetricSourceAccountConfigurationKind::Empty,
                    cloudfront_legacy_s3: cloudfront(HOST, "b", "p").cloudfront_legacy_s3,
                },
            ),
            ConfigurationError::EmptyWithPayload.message(),
        ),
        (
            "CloudFront without payload on cloudfront source",
            new_account(
                "a4",
                "cf_source",
                HOST,
                MetricSourceAccountConfigurationInput {
                    kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
                    cloudfront_legacy_s3: None,
                },
            ),
            ConfigurationError::CloudFrontWithoutPayload.message(),
        ),
        (
            "blank bucket",
            new_account("a5", "cf_source", HOST, cloudfront(HOST, " ", "p")),
            ConfigurationError::BlankRoutingValue.message(),
        ),
        (
            "hostname differs from external key",
            new_account("a6", "cf_source", HOST, cloudfront("other.test", "b", "p")),
            ConfigurationError::HostnameMismatch.message(),
        ),
    ];
    for (label, data, expected) in cases {
        let error = create_metric_source_account(&pool, "actor-1", &data).expect_err(label);
        expect_constraint_message(&error, expected, label);
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        0
    );
    assert_eq!(
        source_audit_rows(&pool)
            .iter()
            .filter(|row| row.entity == "SOURCE_ACCOUNT")
            .count(),
        0,
        "a rejected create must write no audit row"
    );
}

#[test]
fn create_fails_safely_on_unknown_codes_and_reports_constraint_failures_bounded() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);

    for (label, data) in [
        (
            "unknown source code",
            new_account("u1", "no_such_source", HOST, empty()),
        ),
        (
            "source code case variant",
            new_account("u2", "ADMIN_SOURCE", HOST, empty()),
        ),
        (
            "unknown platform code",
            NewMetricSourceAccount {
                platform_code: "no_such_platform".to_string(),
                ..new_account("u3", "admin_source", HOST, empty())
            },
        ),
    ] {
        assert!(
            matches!(
                create_metric_source_account(&pool, "actor-1", &data),
                Err(ThothError::EntityNotFound)
            ),
            "{label} must fail as not found"
        );
    }

    create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("first", "admin_source", "key-1", empty()),
    )
    .expect("first");
    let cases = [
        (
            "duplicate account code",
            new_account("first", "admin_source", "key-2", empty()),
            "A metric source account with this code already exists.",
        ),
        (
            "duplicate (source, external_key)",
            new_account("second", "admin_source", "key-1", empty()),
            "A metric source account with this external key already exists for this metric source.",
        ),
        (
            "blank account code",
            new_account("  ", "admin_source", "key-3", empty()),
            "Metric source account code must not be an empty string.",
        ),
        (
            "blank external key",
            new_account("third", "admin_source", "\t", empty()),
            "Metric source account external key must not be an empty string.",
        ),
        (
            "unknown expected publisher",
            NewMetricSourceAccount {
                expected_publisher_id: Some(Uuid::new_v4()),
                ..new_account("fourth", "admin_source", "key-4", empty())
            },
            "The expected publisher of a metric source account must be an existing publisher.",
        ),
    ];
    for (label, data, expected) in cases {
        let error = create_metric_source_account(&pool, "actor-1", &data).expect_err(label);
        expect_constraint_message(&error, expected, label);
        if let ThothError::DatabaseConstraintError(message) = &error {
            for leaked in [
                "metric_source_account_",
                "violates",
                "DETAIL",
                "INSERT",
                "fkey",
            ] {
                assert!(!message.contains(leaked), "{label} leaked `{leaked}`");
            }
        }
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        1
    );

    // A known publisher is stored as the existing FK value, and an EMPTY
    // account may omit the pin entirely: Amendment 2 requires it only for
    // CloudFront accounts.
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);
    let pinned = create_metric_source_account(
        &pool,
        "actor-1",
        &NewMetricSourceAccount {
            expected_publisher_id: Some(publisher_id),
            ..new_account("pinned", "admin_source", "key-5", empty())
        },
    )
    .expect("pinned");
    assert_eq!(pinned.expected_publisher_id, Some(publisher_id));
    let unpinned = create_metric_source_account(
        &pool,
        "actor-1",
        &NewMetricSourceAccount {
            expected_publisher_id: None,
            ..new_account("unpinned", "admin_source", "key-6", empty())
        },
    )
    .expect("an EMPTY account needs no expected publisher");
    assert_eq!(unpinned.expected_publisher_id, None);
}

#[test]
fn a_cloudfront_account_requires_an_existing_expected_publisher_at_creation() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);

    // Amendment 2: no pin, no CloudFront account. Decided before any write.
    let error = create_metric_source_account(
        &pool,
        "actor-1",
        &NewMetricSourceAccount {
            expected_publisher_id: None,
            ..new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p"))
        },
    )
    .expect_err("a CloudFront account without expectedPublisherId must be refused");
    expect_constraint_message(
        &error,
        ConfigurationError::CloudFrontRequiresExpectedPublisher.message(),
        "cloudfront without publisher",
    );

    // A pin that names no real publisher is refused by the database with the
    // bounded FK message, still atomically.
    let error = create_metric_source_account(
        &pool,
        "actor-1",
        &NewMetricSourceAccount {
            expected_publisher_id: Some(Uuid::new_v4()),
            ..new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p"))
        },
    )
    .expect_err("an unknown publisher must be refused");
    expect_constraint_message(
        &error,
        "The expected publisher of a metric source account must be an existing publisher.",
        "cloudfront with unknown publisher",
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        0
    );
    assert!(
        source_audit_rows(&pool)
            .iter()
            .all(|row| row.entity == "SOURCE"),
        "a refused create must write no account audit row"
    );

    // With a real pin the account is created, and the pin is immutable: the
    // patch input has no field for it and the UPDATE never writes it.
    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p")),
    )
    .expect("pinned CloudFront account");
    assert_eq!(created.expected_publisher_id, Some(FIXTURE_PUBLISHER_ID));
    let updated = update_metric_source_account(
        &pool,
        "actor-2",
        &patch_account("cf", cloudfront(HOST, "b2", "p"), false),
    )
    .expect("update");
    assert_eq!(updated.expected_publisher_id, Some(FIXTURE_PUBLISHER_ID));
}

#[test]
fn the_stable_code_is_matched_exactly_through_the_coordinator() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    for (index, code) in [
        "Acc-Code",
        "acc-code",
        " acc-code",
        "acc\u{00e9}",
        "acc\u{0065}\u{0301}",
    ]
    .into_iter()
    .enumerate()
    {
        create_metric_source_account(
            &pool,
            "actor-1",
            &new_account(code, "admin_source", &format!("key-{index}"), empty()),
        )
        .unwrap_or_else(|error| panic!("{code:?} must be a distinct identity: {error:?}"));
    }
    assert_eq!(
        metric_source_account_by_code(&pool, "Acc-Code")
            .expect("exact")
            .code,
        "Acc-Code"
    );
    assert!(matches!(
        metric_source_account_by_code(&pool, "ACC-CODE"),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        update_metric_source_account(&pool, "actor-1", &patch_account("ACC-CODE", empty(), false)),
        Err(ThothError::EntityNotFound)
    ));
}

// ---- update ----------------------------------------------------------------

#[test]
fn update_replaces_exactly_configuration_and_enabled_and_audits_persisted_states() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account(
            "cf",
            "cf_source",
            HOST,
            cloudfront(HOST, "bucket-a", "prefix/a/"),
        ),
    )
    .expect("create");

    let updated = update_metric_source_account(
        &pool,
        "actor-2",
        &patch_account("cf", cloudfront(HOST, "bucket-b", "prefix/b/"), false),
    )
    .expect("update");
    assert_eq!(updated.source_account_id, created.source_account_id);
    assert_eq!(updated.code, created.code);
    assert_eq!(updated.source_id, created.source_id);
    assert_eq!(updated.platform_id, created.platform_id);
    assert_eq!(updated.external_key, created.external_key);
    assert_eq!(updated.expected_publisher_id, created.expected_publisher_id);
    assert!(!updated.enabled);
    assert_eq!(updated.configuration["logging"]["bucket"], "bucket-b");
    assert_eq!(updated.configuration["logging"]["prefix"], "prefix/b/");
    assert_eq!(updated.configuration["hostname"], HOST);
    assert_eq!(
        metric_source_account_by_code(&pool, "cf").expect("read"),
        updated
    );

    let audit = source_audit_rows(&pool);
    let update = audit
        .iter()
        .find(|row| row.entity == "SOURCE_ACCOUNT" && row.action == "UPDATE")
        .expect("update audit");
    assert_eq!(update.actor, "actor-2");
    assert_eq!(
        update.before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize")
    );
    assert_eq!(
        update.after_state,
        serde_json::to_value(&updated).expect("serialize")
    );

    // Structural immutability: the patch type has exactly three members and the
    // one UPDATE statement sets exactly the two mutable columns.
    let PatchMetricSourceAccount {
        code: _,
        configuration: _,
        enabled: _,
    } = patch_account("x", empty(), true);
    let source = include_str!("crud.rs");
    let set_clause = source
        .split_once("diesel::update(metric_source_account::table.find(current.source_account_id))")
        .expect("one update statement")
        .1
        .split_once(".returning(")
        .expect("returning")
        .0;
    for immutable in [
        "metric_source_account::code.eq",
        "metric_source_account::source_id.eq",
        "metric_source_account::platform_id.eq",
        "metric_source_account::external_key.eq",
        "metric_source_account::expected_publisher_id.eq",
    ] {
        assert!(
            !set_clause.contains(immutable),
            "SET must not write `{immutable}`"
        );
    }
    assert_eq!(source.matches("diesel::update(").count(), 1);
}

#[test]
fn update_enforces_the_compatibility_matrix_against_the_immutable_source() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let cf = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p")),
    )
    .expect("cf");
    let admin = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("admin", "admin_source", "key", empty()),
    )
    .expect("admin");

    let cases = [
        (
            "cloudfront account to EMPTY",
            patch_account("cf", empty(), true),
            ConfigurationError::SourceRequiresCloudFront.message(),
        ),
        (
            "admin account to CloudFront",
            patch_account("admin", cloudfront("key", "b", "p"), true),
            ConfigurationError::SourceRequiresEmpty.message(),
        ),
        (
            "hostname must equal the immutable external key",
            patch_account("cf", cloudfront("other.test", "b", "p"), true),
            ConfigurationError::HostnameMismatch.message(),
        ),
        (
            "blank prefix",
            patch_account("cf", cloudfront(HOST, "b", ""), true),
            ConfigurationError::BlankRoutingValue.message(),
        ),
    ];
    for (label, patch, expected) in cases {
        let error = update_metric_source_account(&pool, "actor-2", &patch).expect_err(label);
        expect_constraint_message(&error, expected, label);
    }
    assert_eq!(metric_source_account_by_code(&pool, "cf").expect("cf"), cf);
    assert_eq!(
        metric_source_account_by_code(&pool, "admin").expect("admin"),
        admin
    );
    assert!(
        source_audit_rows(&pool)
            .iter()
            .all(|row| row.action == "CREATE"),
        "rejected updates must write no audit row"
    );
}

#[test]
fn a_no_op_update_is_decided_by_semantic_jsonb_equality_not_by_text() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p")),
    )
    .expect("create");

    // Re-plant the same value with different key order and whitespace. PostgreSQL
    // JSONB has no key order or whitespace, and neither does the coordinator's
    // comparison, so an update carrying the same values must be a no-op.
    plant_stored_configuration(
        &pool,
        "cf",
        r#"{ "logging" : { "prefix":"p" , "bucket":"b" , "mode":"LEGACY_S3" } ,
            "hostname":"cdn.example-press.test" , "schemaVersion":"cloudfront-source-account/1" }"#,
    );
    let returned = update_metric_source_account(
        &pool,
        "actor-9",
        &patch_account("cf", cloudfront(HOST, "b", "p"), true),
    )
    .expect("no-op");
    assert_eq!(
        returned, created,
        "value-equal stored JSON is the same configuration"
    );
    assert!(
        source_audit_rows(&pool)
            .iter()
            .all(|row| row.action == "CREATE"),
        "a no-op must not record an audit row"
    );

    // A value difference — even one differing only in a routing string's
    // case or whitespace — is a real change, because values are never
    // normalized.
    let changed = update_metric_source_account(
        &pool,
        "actor-9",
        &patch_account("cf", cloudfront(HOST, "B", "p"), true),
    )
    .expect("real change");
    assert_eq!(changed.configuration["logging"]["bucket"], "B");
    assert_eq!(
        source_audit_rows(&pool)
            .iter()
            .filter(|row| row.action == "UPDATE")
            .count(),
        1
    );
}

#[test]
fn unsupported_stored_configuration_fails_closed_without_disclosure_update_or_audit() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("cf", "cf_source", HOST, cloudfront(HOST, "b", "p")),
    )
    .expect("cf");
    create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("admin", "admin_source", "key", empty()),
    )
    .expect("admin");
    let audit_before = source_audit_rows(&pool).len();

    let planted: [(&str, &str, &str); 6] = [
        (
            "generic pre-existing JSON with secret-looking keys",
            "cf",
            r#"{"legacy": true, "accessKeyId": "AKIA-NOT-REAL", "note": "pre-typed"}"#,
        ),
        (
            "cloudfront shape with an extra key",
            "cf",
            r#"{"schemaVersion":"cloudfront-source-account/1","hostname":"cdn.example-press.test",
                "logging":{"mode":"LEGACY_S3","bucket":"b","prefix":"p"},"region":"eu-west-9"}"#,
        ),
        (
            "cloudfront shape with an unknown mode",
            "cf",
            r#"{"schemaVersion":"cloudfront-source-account/1","hostname":"cdn.example-press.test",
                "logging":{"mode":"STANDARD_V2","bucket":"b","prefix":"p"}}"#,
        ),
        (
            "structurally valid cloudfront shape inconsistent with external key",
            "cf",
            r#"{"schemaVersion":"cloudfront-source-account/1","hostname":"other.test",
                "logging":{"mode":"LEGACY_S3","bucket":"b","prefix":"p"}}"#,
        ),
        ("empty object on a cloudfront driver account", "cf", "{}"),
        (
            "cloudfront shape on an admin-import account",
            "admin",
            r#"{"schemaVersion":"cloudfront-source-account/1","hostname":"key",
                "logging":{"mode":"LEGACY_S3","bucket":"b","prefix":"p"}}"#,
        ),
    ];
    for (label, code, json_text) in planted {
        plant_stored_configuration(&pool, code, json_text);
        let expected = ConfigurationError::UnsupportedStored.message();

        let lookup = metric_source_account_by_code(&pool, code).expect_err(label);
        expect_constraint_message(&lookup, expected, label);

        let patch = if code == "cf" {
            patch_account("cf", cloudfront(HOST, "new-bucket", "new-prefix"), false)
        } else {
            patch_account("admin", empty(), false)
        };
        let update = update_metric_source_account(&pool, "actor-2", &patch).expect_err(label);
        expect_constraint_message(&update, expected, label);

        // No disclosure: nothing from the planted value appears in either error.
        for error in [&lookup, &update] {
            let rendered = format!("{error} {error:?}");
            for fragment in [
                "legacy",
                "AKIA",
                "accessKeyId",
                "eu-west-9",
                "STANDARD_V2",
                "other.test",
                "region",
            ] {
                assert!(
                    !rendered.contains(fragment),
                    "{label}: the error must not leak `{fragment}`: {rendered}"
                );
            }
        }
        // No write: the planted value is still exactly what is stored, the
        // enabled flag did not move, and nothing was audited.
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM metric_source_account WHERE code = '{code}' \
                        AND configuration = '{}'::jsonb AND enabled)",
                    json_text.replace('\'', "''")
                ),
            ),
            1,
            "{label}: the canonical row must be untouched"
        );
        assert_eq!(
            source_audit_rows(&pool).len(),
            audit_before,
            "{label}: no audit row"
        );
    }
}

#[test]
fn a_failing_audit_write_rolls_back_the_canonical_change() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let error = create_metric_source_account(
        &pool,
        "  ",
        &new_account("orphan", "admin_source", "key", empty()),
    )
    .expect_err("a blank actor must fail the audit write");
    expect_constraint_message(
        &error,
        "Metric source registry history actor must not be an empty string.",
        "create",
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_account)"),
        0,
        "the canonical row must not survive a failed audit write"
    );

    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("target", "admin_source", "key", empty()),
    )
    .expect("create");
    let error = update_metric_source_account(&pool, "\n", &patch_account("target", empty(), false))
        .expect_err("a blank actor must fail the audit write");
    assert!(matches!(error, ThothError::DatabaseConstraintError(_)));
    assert_eq!(
        metric_source_account_by_code(&pool, "target").expect("survives"),
        created
    );
}

// ---- concurrency and locking -----------------------------------------------

#[test]
fn two_competing_updates_serialise_and_their_audit_chain_matches_commit_order() {
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &new_account("contended", "cf_source", HOST, cloudfront(HOST, "b0", "p")),
    )
    .expect("create");

    let spawn = |actor: &'static str, bucket: &'static str| {
        let pool = Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_source_account(
                &pool,
                actor,
                &patch_account("contended", cloudfront(HOST, bucket, "p"), true),
            )
        })
    };
    let first = spawn("actor-a", "bA");
    let second = spawn("actor-b", "bB");
    first.join().expect("thread a").expect("update a");
    second.join().expect("thread b").expect("update b");

    let mut audit = source_audit_rows(&pool);
    audit.reverse();
    let chain = serialized_update_chain(&audit, "SOURCE_ACCOUNT", created.source_account_id);
    assert_eq!(chain.len(), 2);
    assert_eq!(
        chain[0].before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize created")
    );
    assert_eq!(
        chain[1].before_state.as_ref().expect("before"),
        &chain[0].after_state,
        "the second committed update must have read the first one's committed state"
    );
    let final_row = metric_source_account_by_code(&pool, "contended").expect("final");
    assert_eq!(
        chain[1].after_state,
        serde_json::to_value(&final_row).expect("serialize final")
    );
    let bucket_of = |state: &JsonValue| {
        state["configuration"]["logging"]["bucket"]
            .as_str()
            .map(str::to_string)
    };
    let mut written = [
        bucket_of(&chain[0].after_state),
        bucket_of(&chain[1].after_state),
    ];
    written.sort();
    assert_eq!(written, [Some("bA".to_string()), Some("bB".to_string())]);
    for transition in &chain {
        let expected_actor = if bucket_of(&transition.after_state).as_deref() == Some("bA") {
            "actor-a"
        } else {
            "actor-b"
        };
        assert_eq!(transition.actor, expected_actor);
    }
    assert_eq!(
        bucket_of(&serde_json::to_value(&final_row).expect("final")),
        bucket_of(&chain[1].after_state)
    );
}

#[test]
fn the_coordinator_takes_exactly_one_application_row_lock() {
    let source = include_str!("crud.rs");
    assert_eq!(
        source.matches(".for_update()").count(),
        1,
        "the account coordinator must request exactly one row lock"
    );
    for forbidden in FORBIDDEN_JOIN_CONSTRUCTS {
        assert!(
            !source.contains(forbidden),
            "a joined multi-table FOR UPDATE is prohibited: found `{forbidden}`"
        );
    }
    let (before_lock, after_lock) = source.split_once(".for_update()").expect("one lock");
    let opening: String = before_lock
        .rsplit("let current")
        .next()
        .expect("the lock belongs to the current-state read")
        .split_whitespace()
        .collect();
    assert!(
        opening.contains(
            "metric_source_account::table.filter(metric_source_account::code.eq(&data.code))"
        ),
        "the one lock must be taken on the canonical metric_source_account row selected by \
         exact code, found: {opening}"
    );
    assert!(!after_lock.contains("for_update"));
}

#[test]
fn an_account_update_does_not_wait_for_locks_on_its_source_platform_or_publisher() {
    // Runtime falsification of the parent-lock rule: another transaction holds
    // FOR UPDATE on the account's source, platform and publisher rows for the
    // whole duration of the update. If the coordinator locked any parent, the
    // update would block until the holder released; instead it must commit
    // while the holder is still holding.
    let (_guard, pool) = setup_registry_db();
    fixture_sources_and_platform(&pool);
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);
    let created = create_metric_source_account(
        &pool,
        "actor-1",
        &NewMetricSourceAccount {
            expected_publisher_id: Some(publisher_id),
            ..new_account("held", "cf_source", HOST, cloudfront(HOST, "b", "p"))
        },
    )
    .expect("create");

    let released = Arc::new(AtomicBool::new(false));
    let (locked_tx, locked_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let holder = {
        let pool = Arc::clone(&pool);
        let released = Arc::clone(&released);
        let source_id = created.source_id;
        let platform_id = created.platform_id;
        std::thread::spawn(move || {
            let mut connection = pool.get().expect("holder connection");
            connection
                .transaction::<_, diesel::result::Error, _>(|connection| {
                    sql_query(
                        "SELECT source_id FROM metric_source WHERE source_id = $1 FOR UPDATE",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(source_id)
                    .execute(connection)?;
                    sql_query(
                        "SELECT platform_id FROM metric_platform WHERE platform_id = $1 FOR UPDATE",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(platform_id)
                    .execute(connection)?;
                    sql_query(
                        "SELECT publisher_id FROM publisher WHERE publisher_id = $1 FOR UPDATE",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(publisher_id)
                    .execute(connection)?;
                    locked_tx.send(()).expect("signal locked");
                    // Hold until the test says the update finished, or give up
                    // after a bounded wait so a blocked update fails rather than
                    // hanging the suite.
                    let _ = release_rx.recv_timeout(Duration::from_secs(15));
                    Ok(())
                })
                .expect("holder transaction");
            released.store(true, Ordering::SeqCst);
        })
    };
    locked_rx.recv().expect("holder took its locks");

    let updated = update_metric_source_account(
        &pool,
        "actor-2",
        &patch_account("held", cloudfront(HOST, "b2", "p"), false),
    )
    .expect("the account update must not wait for parent-row locks");
    assert!(
        !released.load(Ordering::SeqCst),
        "the update completed only after the parent locks were released, so it waited on them"
    );
    release_tx.send(()).expect("release the holder");
    holder.join().expect("holder thread");
    assert_eq!(updated.configuration["logging"]["bucket"], "b2");
}
