//! Focused `MET-WP1-02` database tests for the `metric_source` acquisition
//! registry, extended by `MET-WP1-13` with the protected administration
//! coordinator: exact-code create/lookup/update, the driver-key invariant at
//! both boundaries, replacement semantics, atomic audit, no-op silence,
//! rollback, one-row locking and serialized concurrent updates.
//!
//! These tests reuse [`setup_registry_db`] from `metric_platform::tests`: it
//! reverts migrations down to and including the `MET-WP1-01` registry
//! migration and reapplies every pending migration, which restores the
//! pristine post-migration state of the `MET-WP1-02` source-state schema as
//! well. [`revert_through_source_state_migration`] additionally supports
//! targeted rollback evidence for the `MET-WP1-02` migration itself without
//! assuming it remains the newest migration forever.

use std::str::FromStr;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::crud::{
    create_metric_source, metric_source_by_code, update_metric_source, DRIVER_KEY_INVARIANT_MESSAGE,
};
use super::{
    MetricSource, MetricSourceAcquisitionType, NewMetricSource, PatchMetricSource,
    DRIVER_KEY_WHITESPACE,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{
    enum_labels, scalar_i64, serialized_update_chain, setup_registry_db, FORBIDDEN_JOIN_CONSTRUCTS,
};
use crate::model::metric_source_registry_history::tests::source_audit_rows;
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::schema::metric_source;
use thoth_errors::ThothError;

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

/// Insert one `DRIVER` source row whose optional day columns are SQL literals.
///
/// `MET-WP1-13`'s driver-key CHECK requires a `DRIVER` row to carry a nonblank
/// key, so the fixture supplies one; nothing else about the row changed.
fn insert_source_raw(
    pool: &PgPool,
    code: &str,
    lookback_sql: &str,
    finalization_sql: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_source \
             (code, acquisition_type, driver_key, enabled, default_lookback_days, \
              default_finalization_delay_days) \
         VALUES ($1, 'DRIVER', 'test_driver', TRUE, {lookback_sql}, {finalization_sql})"
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

// --------------------------------------------------------------------------
// `MET-WP1-13` protected administration coordinator
// --------------------------------------------------------------------------

fn new_source(code: &str) -> NewMetricSource {
    NewMetricSource {
        code: code.to_string(),
        acquisition_type: MetricSourceAcquisitionType::Driver,
        driver_key: Some("test_driver".to_string()),
        enabled: true,
        default_lookback_days: Some(30),
        default_finalization_delay_days: Some(2),
    }
}

fn patch_source(code: &str, enabled: bool, lookback: Option<i32>) -> PatchMetricSource {
    PatchMetricSource {
        code: code.to_string(),
        enabled,
        default_lookback_days: lookback,
        default_finalization_delay_days: Some(2),
    }
}

#[test]
fn metric_source_carries_exactly_the_expected_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(&pool, "metric_source"),
        vec![
            "metric_source_code_check",
            "metric_source_default_finalization_delay_days_check",
            "metric_source_default_lookback_days_check",
            "metric_source_driver_key_check",
        ],
        "metric_source must carry exactly the three predecessor CHECKs and the one \
         MET-WP1-13 driver-key CHECK"
    );
}

#[test]
fn create_persists_the_exact_values_and_audits_the_persisted_row() {
    let (_guard, pool) = setup_registry_db();

    let created = create_metric_source(&pool, "actor-1", &new_source("cdn_source"))
        .expect("create must succeed");
    assert_eq!(created.code, "cdn_source");
    assert_eq!(
        created.acquisition_type,
        MetricSourceAcquisitionType::Driver
    );
    assert_eq!(created.driver_key.as_deref(), Some("test_driver"));
    assert!(created.enabled);
    assert_eq!(created.default_lookback_days, Some(30));
    assert_eq!(created.default_finalization_delay_days, Some(2));
    assert_eq!(
        metric_source_by_code(&pool, "cdn_source").expect("lookup"),
        created,
        "the returned row must be the persisted row"
    );

    let audit = source_audit_rows(&pool);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].entity, "SOURCE");
    assert_eq!(audit[0].entity_id, created.source_id);
    assert_eq!(audit[0].action, "CREATE");
    assert_eq!(audit[0].actor, "actor-1");
    assert_eq!(audit[0].before_state, None);
    assert_eq!(
        audit[0].after_state,
        serde_json::to_value(&created).expect("serialize"),
        "after_state must be the exact persisted row"
    );
    assert_eq!(audit[0].after_state["acquisitionType"], "DRIVER");
    assert_eq!(audit[0].after_state["driverKey"], "test_driver");
}

#[test]
fn a_code_and_a_driver_key_are_stored_and_matched_exactly_with_no_normalisation() {
    let (_guard, pool) = setup_registry_db();
    let variants = [
        "Cdn-Source",
        "cdn-source",
        " cdn-source",
        "cdn-source ",
        "cdn_source",
        "cdn\u{00e9}",
        "cdn\u{0065}\u{0301}",
    ];
    for (index, code) in variants.into_iter().enumerate() {
        let mut data = new_source(code);
        data.driver_key = Some(format!(" Driver {index} "));
        let created = create_metric_source(&pool, "actor-1", &data)
            .unwrap_or_else(|error| panic!("code {code:?} must be a distinct identity: {error:?}"));
        assert_eq!(created.code, code, "the code must be stored exactly");
        assert_eq!(
            created.driver_key.as_deref(),
            Some(format!(" Driver {index} ").as_str()),
            "a valid driver key must be stored exactly, whitespace included"
        );
    }
    assert_eq!(
        metric_source_by_code(&pool, "Cdn-Source")
            .expect("exact")
            .code,
        "Cdn-Source"
    );
    assert!(matches!(
        metric_source_by_code(&pool, "CDN-SOURCE"),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        update_metric_source(&pool, "actor-1", &patch_source("CDN-SOURCE", false, None)),
        Err(ThothError::EntityNotFound)
    ));
    assert_eq!(source_audit_rows(&pool).len(), variants.len());
}

#[test]
fn the_driver_key_invariant_is_enforced_before_any_write() {
    let (_guard, pool) = setup_registry_db();

    let cases: [(&str, MetricSourceAcquisitionType, Option<&str>); 7] = [
        (
            "driver without key",
            MetricSourceAcquisitionType::Driver,
            None,
        ),
        (
            "driver with empty key",
            MetricSourceAcquisitionType::Driver,
            Some(""),
        ),
        (
            "driver with blank key",
            MetricSourceAcquisitionType::Driver,
            Some(" \t\n"),
        ),
        (
            "upload with key",
            MetricSourceAcquisitionType::PublisherUpload,
            Some("cloudfront"),
        ),
        (
            "operas with key",
            MetricSourceAcquisitionType::Operas,
            Some("x"),
        ),
        (
            "admin with key",
            MetricSourceAcquisitionType::AdminImport,
            Some("x"),
        ),
        (
            "admin with blank key",
            MetricSourceAcquisitionType::AdminImport,
            Some(" "),
        ),
    ];
    for (label, acquisition_type, driver_key) in cases {
        let data = NewMetricSource {
            code: "invalid".to_string(),
            acquisition_type,
            driver_key: driver_key.map(str::to_string),
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        };
        let error = create_metric_source(&pool, "actor-1", &data).unwrap_err();
        assert!(
            matches!(&error, ThothError::DatabaseConstraintError(message)
                if message.as_ref() == DRIVER_KEY_INVARIANT_MESSAGE),
            "{label}: expected the bounded invariant message, got {error:?}"
        );
    }
    assert_eq!(scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source)"), 0);
    assert!(source_audit_rows(&pool).is_empty());

    // The accepting side of the truth table, through the same coordinator.
    for (code, acquisition_type, driver_key) in [
        ("d", MetricSourceAcquisitionType::Driver, Some("cloudfront")),
        ("u", MetricSourceAcquisitionType::PublisherUpload, None),
        ("o", MetricSourceAcquisitionType::Operas, None),
        ("a", MetricSourceAcquisitionType::AdminImport, None),
    ] {
        let data = NewMetricSource {
            code: code.to_string(),
            acquisition_type,
            driver_key: driver_key.map(str::to_string),
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        };
        create_metric_source(&pool, "actor-1", &data)
            .unwrap_or_else(|error| panic!("{code} must be accepted: {error:?}"));
    }
    // The coordinator's message is byte-identical to the database mapping, so
    // a client cannot tell which boundary refused.
    let raw = sql_query(
        "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled) \
         VALUES ('raw', 'DRIVER', NULL, TRUE)",
    )
    .execute(&mut pool.get().expect("connection"))
    .expect_err("the CHECK must refuse");
    assert!(
        matches!(ThothError::from(raw), ThothError::DatabaseConstraintError(message)
        if message.as_ref() == DRIVER_KEY_INVARIANT_MESSAGE)
    );
}

/// The Unicode scalar values the driver-key invariant treats as whitespace,
/// as code points in ascending order.
fn driver_key_whitespace_code_points() -> Vec<i32> {
    let mut code_points: Vec<i32> = DRIVER_KEY_WHITESPACE
        .iter()
        .map(|&c| u32::from(c) as i32)
        .collect();
    code_points.sort_unstable();
    code_points
}

#[test]
fn the_driver_key_whitespace_set_is_exactly_the_frozen_unicode_white_space_set() {
    let frozen: Vec<i32> = (0x0009..=0x000D)
        .chain([0x0020, 0x0085, 0x00A0, 0x1680])
        .chain(0x2000..=0x200A)
        .chain([0x2028, 0x2029, 0x202F, 0x205F, 0x3000])
        .collect();
    assert_eq!(driver_key_whitespace_code_points(), frozen);

    // Every Unicode scalar value: the explicit set is the Unicode `White_Space`
    // property, and the application predicate refuses a one-character key
    // exactly when that character is in the set.
    let mut is_whitespace = Vec::new();
    let mut refused = Vec::new();
    for c in (0..=u32::from(char::MAX)).filter_map(char::from_u32) {
        if c.is_whitespace() {
            is_whitespace.push(u32::from(c) as i32);
        }
        if !MetricSourceAcquisitionType::Driver
            .accepts_driver_key(Some(&*c.encode_utf8(&mut [0; 4])))
        {
            refused.push(u32::from(c) as i32);
        }
    }
    assert_eq!(is_whitespace, frozen);
    assert_eq!(refused, frozen);
}

#[test]
fn the_stored_driver_key_check_classifies_exactly_the_shared_whitespace_set() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    #[derive(diesel::QueryableByName)]
    struct Expression {
        #[diesel(sql_type = diesel::sql_types::Text)]
        expression: String,
    }
    #[derive(diesel::QueryableByName)]
    struct CodePoint {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        code_point: i32,
    }

    // Evaluate the CHECK exactly as PostgreSQL stored it, not a copy of its
    // pattern, for a DRIVER row holding each one-character key in turn. NUL
    // and the surrogate range are not valid `TEXT` characters.
    let stored = sql_query(
        "SELECT pg_get_expr(conbin, conrelid) AS expression FROM pg_constraint \
         WHERE conrelid = 'metric_source'::regclass \
           AND conname = 'metric_source_driver_key_check'",
    )
    .get_result::<Expression>(&mut connection)
    .expect("the driver-key CHECK must exist")
    .expression;
    assert!(
        !stored.contains("[:") && !stored.contains("\\s"),
        "the CHECK must not use a locale-dependent character class: {stored}"
    );
    let refused: Vec<i32> = sql_query(format!(
        "SELECT i AS code_point \
           FROM generate_series(1, 1114111) AS i \
          CROSS JOIN LATERAL ( \
                SELECT 'DRIVER'::metric_source_acquisition_type AS acquisition_type, \
                       chr(i) AS driver_key) AS metric_source \
          WHERE (i < 55296 OR i > 57343) AND NOT ({stored}) \
          ORDER BY i"
    ))
    .load::<CodePoint>(&mut connection)
    .expect("Failed to evaluate the stored CHECK")
    .into_iter()
    .map(|row| row.code_point)
    .collect();
    assert_eq!(
        refused,
        driver_key_whitespace_code_points(),
        "the database must refuse a one-character DRIVER key exactly when the \
         application does"
    );
}

#[test]
fn the_driver_key_nonblank_decision_is_identical_at_both_boundaries() {
    use MetricSourceAcquisitionType::{AdminImport, Driver, Operas, PublisherUpload};
    let (_guard, pool) = setup_registry_db();

    // (label, acquisition type, driver key, whether the invariant accepts it)
    let vectors: [(&str, MetricSourceAcquisitionType, Option<&str>, bool); 14] = [
        ("empty string", Driver, Some(""), false),
        ("ASCII spaces only", Driver, Some("   "), false),
        ("ASCII tab/newline only", Driver, Some("\t\n"), false),
        ("U+00A0 only", Driver, Some("\u{00A0}"), false),
        ("U+2003 only", Driver, Some("\u{2003}"), false),
        ("ordinary nonblank ASCII", Driver, Some("cloudfront"), true),
        ("ordinary nonblank Unicode", Driver, Some("caf\u{00E9}"), true),
        (
            "every shared whitespace character",
            Driver,
            Some("\t\n\u{000B}\u{000C}\r \u{0085}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}"),
            false,
        ),
        (
            "surrounding Unicode whitespace around a real key",
            Driver,
            Some("\u{3000} \u{00A0}cloudfront\u{2003}\t"),
            true,
        ),
        // U+200B ZERO WIDTH SPACE is not `White_Space`, so it is nonblank.
        ("U+200B only", Driver, Some("\u{200B}"), true),
        ("DRIVER with NULL", Driver, None, false),
        ("PUBLISHER_UPLOAD with a key", PublisherUpload, Some("cloudfront"), false),
        ("OPERAS with a whitespace key", Operas, Some("\u{00A0}"), false),
        ("ADMIN_IMPORT with NULL", AdminImport, None, true),
    ];

    for (index, (label, acquisition_type, driver_key, expected)) in vectors.into_iter().enumerate()
    {
        // 1. The application predicate.
        assert_eq!(
            acquisition_type.accepts_driver_key(driver_key),
            expected,
            "{label}: application predicate"
        );

        // 2. PostgreSQL's metric_source_driver_key_check, on a raw INSERT that
        //    bypasses the coordinator and is always rolled back.
        let mut connection = pool.get().expect("Failed to get DB connection");
        let mut database_accepted = None;
        let _ = connection.transaction::<(), DieselError, _>(|connection| {
            let result = sql_query(
                "INSERT INTO metric_source (code, acquisition_type, driver_key, enabled) \
                 VALUES ($1, $2::metric_source_acquisition_type, $3, TRUE)",
            )
            .bind::<diesel::sql_types::Text, _>(format!("raw_{index}"))
            .bind::<diesel::sql_types::Text, _>(acquisition_type.to_string())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(driver_key)
            .execute(connection);
            database_accepted = Some(match result {
                Ok(_) => true,
                Err(DieselError::DatabaseError(DatabaseErrorKind::CheckViolation, info))
                    if info.constraint_name() == Some("metric_source_driver_key_check") =>
                {
                    false
                }
                Err(error) => panic!("{label}: unexpected database error {error:?}"),
            });
            Err(DieselError::RollbackTransaction)
        });
        assert_eq!(
            database_accepted,
            Some(expected),
            "{label}: metric_source_driver_key_check"
        );
        drop(connection);

        // 3. The coordinator: an accepted key persists exactly; a refused key
        //    writes neither a canonical row nor an audit row.
        let sources_before = scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source)");
        let audit_before = source_audit_rows(&pool).len();
        let code = format!("vector_{index}");
        let data = NewMetricSource {
            code: code.clone(),
            acquisition_type,
            driver_key: driver_key.map(str::to_string),
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        };
        match create_metric_source(&pool, "actor-1", &data) {
            Ok(created) => {
                assert!(expected, "{label}: the coordinator accepted a refused key");
                assert_eq!(
                    created.driver_key.as_deref(),
                    driver_key,
                    "{label}: returned exactly"
                );
                assert_eq!(
                    metric_source_by_code(&pool, &code)
                        .expect("lookup")
                        .driver_key
                        .as_deref(),
                    driver_key,
                    "{label}: persisted exactly"
                );
                assert_eq!(source_audit_rows(&pool).len(), audit_before + 1);
            }
            Err(error) => {
                assert!(!expected, "{label}: the coordinator refused: {error:?}");
                assert!(
                    matches!(&error, ThothError::DatabaseConstraintError(message)
                        if message.as_ref() == DRIVER_KEY_INVARIANT_MESSAGE),
                    "{label}: expected the bounded invariant message, got {error:?}"
                );
                assert_eq!(
                    scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source)"),
                    sources_before,
                    "{label}: no canonical write"
                );
                assert_eq!(
                    source_audit_rows(&pool).len(),
                    audit_before,
                    "{label}: no audit write"
                );
            }
        }
    }
}

#[test]
fn database_constraint_failures_are_atomic_and_sanitised() {
    let (_guard, pool) = setup_registry_db();
    create_metric_source(&pool, "actor-1", &new_source("dup")).expect("first");

    let cases: [(&str, NewMetricSource, &str); 4] = [
        (
            "duplicate code",
            new_source("dup"),
            "A metric source with this code already exists.",
        ),
        (
            "blank code",
            new_source("   "),
            "Metric source code must not be an empty string.",
        ),
        (
            "negative lookback",
            NewMetricSource {
                default_lookback_days: Some(-1),
                ..new_source("neg_lookback")
            },
            "Metric source default lookback days must not be negative.",
        ),
        (
            "negative finalization",
            NewMetricSource {
                default_finalization_delay_days: Some(-7),
                ..new_source("neg_final")
            },
            "Metric source default finalization delay days must not be negative.",
        ),
    ];
    for (label, data, expected) in cases {
        let error = create_metric_source(&pool, "actor-1", &data).expect_err(label);
        match &error {
            ThothError::DatabaseConstraintError(message) => {
                assert_eq!(message.as_ref(), expected, "{label}");
                for leaked in ["metric_source_", "violates", "DETAIL", "INSERT"] {
                    assert!(!message.contains(leaked), "{label} leaked `{leaked}`");
                }
            }
            other => panic!("{label} must be a bounded constraint error, got {other:?}"),
        }
    }
    assert_eq!(scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source)"), 1);
    assert_eq!(
        source_audit_rows(&pool).len(),
        1,
        "only the successful create is audited"
    );

    // Zero and NULL day defaults are accepted.
    let zero = NewMetricSource {
        default_lookback_days: Some(0),
        default_finalization_delay_days: None,
        ..new_source("zero_days")
    };
    let created = create_metric_source(&pool, "actor-1", &zero).expect("zero days");
    assert_eq!(created.default_lookback_days, Some(0));
    assert_eq!(created.default_finalization_delay_days, None);
}

#[test]
fn update_replaces_only_the_mutable_fields_and_audits_before_and_after() {
    let (_guard, pool) = setup_registry_db();
    let created = create_metric_source(&pool, "actor-1", &new_source("target")).expect("create");

    let updated = update_metric_source(&pool, "actor-2", &patch_source("target", false, Some(90)))
        .expect("update");
    assert_eq!(updated.source_id, created.source_id);
    assert_eq!(updated.code, "target", "code is immutable");
    assert_eq!(
        updated.acquisition_type, created.acquisition_type,
        "acquisition type is immutable"
    );
    assert_eq!(
        updated.driver_key, created.driver_key,
        "driver key is immutable"
    );
    assert!(!updated.enabled);
    assert_eq!(updated.default_lookback_days, Some(90));
    assert_eq!(updated.default_finalization_delay_days, Some(2));
    assert_eq!(
        metric_source_by_code(&pool, "target").expect("read"),
        updated
    );

    let audit = source_audit_rows(&pool);
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[1].action, "UPDATE");
    assert_eq!(audit[1].actor, "actor-2");
    assert_eq!(
        audit[1].before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize")
    );
    assert_eq!(
        audit[1].after_state,
        serde_json::to_value(&updated).expect("serialize")
    );
}

#[test]
fn the_patch_input_cannot_express_the_immutable_fields_and_the_set_clause_omits_them() {
    // Structural: the patch type has exactly the four approved members. A
    // field added to it would have to be added here, under review.
    let PatchMetricSource {
        code: _,
        enabled: _,
        default_lookback_days: _,
        default_finalization_delay_days: _,
    } = patch_source("c", true, None);

    // Source-level: the one UPDATE statement sets exactly the three mutable
    // columns.
    let source = include_str!("crud.rs");
    let set_clause = source
        .split_once("diesel::update(metric_source::table.find(current.source_id))")
        .expect("one update statement")
        .1
        .split_once(".returning(")
        .expect("returning")
        .0;
    for immutable in [
        "metric_source::code.eq",
        "metric_source::acquisition_type.eq",
        "metric_source::driver_key.eq",
    ] {
        assert!(
            !set_clause.contains(immutable),
            "the update SET clause must not write `{immutable}`"
        );
    }
    assert_eq!(source.matches("diesel::update(").count(), 1);
}

#[test]
fn an_omitted_nullable_field_stores_sql_null_rather_than_retaining_the_old_value() {
    let (_guard, pool) = setup_registry_db();
    create_metric_source(&pool, "actor-1", &new_source("nullable")).expect("create");
    let updated = update_metric_source(
        &pool,
        "actor-1",
        &PatchMetricSource {
            code: "nullable".to_string(),
            enabled: true,
            default_lookback_days: None,
            default_finalization_delay_days: None,
        },
    )
    .expect("update");
    assert_eq!(updated.default_lookback_days, None);
    assert_eq!(updated.default_finalization_delay_days, None);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source WHERE code = 'nullable' \
                AND default_lookback_days IS NULL AND default_finalization_delay_days IS NULL)",
        ),
        1
    );
}

#[test]
fn a_true_no_op_update_writes_nothing_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    let created = create_metric_source(&pool, "actor-1", &new_source("still")).expect("create");
    let returned = update_metric_source(&pool, "actor-9", &patch_source("still", true, Some(30)))
        .expect("no-op update");
    assert_eq!(returned, created);
    assert_eq!(
        source_audit_rows(&pool).len(),
        1,
        "a no-op must not record an audit row"
    );
}

#[test]
fn an_unknown_code_is_reported_as_not_found_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    assert!(matches!(
        metric_source_by_code(&pool, "absent"),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        update_metric_source(&pool, "actor-1", &patch_source("absent", true, None)),
        Err(ThothError::EntityNotFound)
    ));
    assert!(source_audit_rows(&pool).is_empty());
}

#[test]
fn a_failing_audit_write_rolls_back_the_canonical_change() {
    let (_guard, pool) = setup_registry_db();
    let error = create_metric_source(&pool, "   ", &new_source("orphan"))
        .expect_err("a blank actor must fail the audit write");
    assert!(
        matches!(&error, ThothError::DatabaseConstraintError(message)
            if message.as_ref() == "Metric source registry history actor must not be an empty string."),
        "the audit CHECK must surface bounded, got {error:?}"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_source WHERE code = 'orphan')"
        ),
        0,
        "the canonical row must not survive a failed audit write"
    );
    assert!(source_audit_rows(&pool).is_empty());

    let created = create_metric_source(&pool, "actor-1", &new_source("target")).expect("create");
    let error = update_metric_source(&pool, "\t", &patch_source("target", false, None))
        .expect_err("a blank actor must fail the audit write");
    assert!(matches!(error, ThothError::DatabaseConstraintError(_)));
    assert_eq!(
        metric_source_by_code(&pool, "target").expect("survives"),
        created
    );
    assert_eq!(source_audit_rows(&pool).len(), 1);
}

#[test]
fn two_competing_updates_serialise_and_their_audit_chain_matches_commit_order() {
    let (_guard, pool) = setup_registry_db();
    let created = create_metric_source(&pool, "actor-1", &new_source("contended")).expect("create");

    let first = {
        let pool = Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_source(&pool, "actor-a", &patch_source("contended", true, Some(11)))
        })
    };
    let second = {
        let pool = Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_source(&pool, "actor-b", &patch_source("contended", true, Some(22)))
        })
    };
    first.join().expect("thread a").expect("update a");
    second.join().expect("thread b").expect("update b");

    let mut audit = source_audit_rows(&pool);
    assert_eq!(audit.len(), 3, "one CREATE and exactly two UPDATE entries");
    // Order-independence: the chain is reconstructed from states, so reversing
    // the rows must not change the result.
    audit.reverse();
    let chain = serialized_update_chain(&audit, "SOURCE", created.source_id);
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
    let final_row = metric_source_by_code(&pool, "contended").expect("final");
    assert_eq!(
        chain[1].after_state,
        serde_json::to_value(&final_row).expect("serialize final")
    );
    let lookback_of = |state: &serde_json::Value| state["defaultLookbackDays"].as_i64();
    let mut written = [
        lookback_of(&chain[0].after_state),
        lookback_of(&chain[1].after_state),
    ];
    written.sort_unstable();
    assert_eq!(
        written,
        [Some(11), Some(22)],
        "both requested values were written"
    );
    for transition in &chain {
        let expected_actor = if lookback_of(&transition.after_state) == Some(11) {
            "actor-a"
        } else {
            "actor-b"
        };
        assert_eq!(transition.actor, expected_actor);
    }
    assert_eq!(
        Some(i64::from(final_row.default_lookback_days.expect("set"))),
        lookback_of(&chain[1].after_state),
        "the surviving row is the one the last committed transition wrote"
    );
}

#[test]
fn the_coordinator_takes_exactly_one_application_row_lock() {
    let source = include_str!("crud.rs");
    assert_eq!(
        source.matches(".for_update()").count(),
        1,
        "the source coordinator must request exactly one row lock"
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
        opening.contains("metric_source::table.filter(metric_source::code.eq(&data.code))"),
        "the one lock must be taken on the canonical metric_source row selected by exact \
         code, found: {opening}"
    );
    assert!(!after_lock.contains("for_update"));
}
