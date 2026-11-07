//! Focused `MET-WP1-01` database tests for the `metric_platform_measure`
//! registry mapping, including the `supported_grains` array contract and the
//! non-cascading registry foreign keys.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use super::crud::{
    create_metric_platform_measure, metric_platform_measure_by_codes,
    update_metric_platform_measure,
};
use super::{
    MetricPlatformMeasure, MetricReportingGrain, NewMetricPlatformMeasure,
    PatchMetricPlatformMeasure,
};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{
    audit_rows, enum_labels, insert_platform_row, scalar_i64, setup_registry_db,
    FORBIDDEN_JOIN_CONSTRUCTS,
};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::schema::metric_platform_measure;
use thoth_errors::ThothError;

/// Insert one `metric_measure` row with an explicit id through raw SQL.
fn insert_measure_row(pool: &PgPool, measure_id: Uuid, code: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_measure \
             (measure_id, code, display_name, category, unit, allow_negative, \
              additive_across_time, additive_across_works, definition, enabled) \
         VALUES ($1, $2, $3, 'USAGE', 'COUNT', FALSE, TRUE, TRUE, 'Some definition.', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(format!("Measure {code}"))
    .execute(&mut connection)
    .expect("Failed to insert metric_measure fixture row");
}

/// One referenced platform/measure pair for mapping tests.
fn fixture_pair(pool: &PgPool) -> (Uuid, Uuid) {
    let platform_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, "test_platform");
    insert_measure_row(pool, measure_id, "test_measure");
    (platform_id, measure_id)
}

/// Insert one mapping row whose `supported_grains` is a SQL array literal.
fn insert_mapping_raw(
    pool: &PgPool,
    platform_id: Uuid,
    measure_id: Uuid,
    supported_grains_sql: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_platform_measure \
             (platform_id, measure_id, supported_grains, supports_country, \
              supports_institution, supports_publication, direct_collection, enabled) \
         VALUES ($1, $2, {supported_grains_sql}, TRUE, FALSE, FALSE, TRUE, TRUE)"
    ))
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .execute(&mut connection)
}

fn mapping_count(pool: &PgPool) -> i64 {
    scalar_i64(pool, "(SELECT COUNT(*) FROM metric_platform_measure)")
}

#[test]
fn reporting_grain_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_reporting_grain"),
        ["DAY", "MONTH", "REPORTING_PERIOD"]
    );
}

#[test]
fn migration_seeds_no_platform_measure_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        mapping_count(&pool),
        0,
        "MET-WP1-01 must not seed any metric_platform_measure row"
    );
}

#[test]
fn platform_measure_deliberately_has_no_timestamp_columns() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND table_name = 'metric_platform_measure' \
                AND column_name IN ('created_at', 'updated_at'))",
        ),
        0,
        "the approved design (§6.3) deliberately omits timestamps on metric_platform_measure"
    );
}

#[test]
fn empty_supported_grains_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    let result = insert_mapping_raw(
        &pool,
        platform_id,
        measure_id,
        "'{}'::metric_reporting_grain[]",
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::CheckViolation,
                _
            ))
        ),
        "an empty supported_grains array must fail the check constraint: {result:?}"
    );
}

#[test]
fn null_supported_grain_element_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    for array_sql in [
        "ARRAY[NULL]::metric_reporting_grain[]",
        "ARRAY['DAY', NULL]::metric_reporting_grain[]",
    ] {
        let result = insert_mapping_raw(&pool, platform_id, measure_id, array_sql);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a NULL supported_grains element ({array_sql}) must fail the check constraint: \
             {result:?}"
        );
    }
}

#[test]
fn duplicate_supported_grain_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    for array_sql in [
        "ARRAY['DAY', 'DAY']::metric_reporting_grain[]",
        "ARRAY['MONTH', 'MONTH']::metric_reporting_grain[]",
        "ARRAY['REPORTING_PERIOD', 'REPORTING_PERIOD']::metric_reporting_grain[]",
        "ARRAY['DAY', 'MONTH', 'DAY']::metric_reporting_grain[]",
    ] {
        let result = insert_mapping_raw(&pool, platform_id, measure_id, array_sql);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a duplicate supported grain ({array_sql}) must fail the check constraint: \
             {result:?}"
        );
    }
}

#[test]
fn duplicate_platform_measure_pair_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    insert_mapping_raw(
        &pool,
        platform_id,
        measure_id,
        "ARRAY['DAY']::metric_reporting_grain[]",
    )
    .expect("First mapping insert must pass");
    let duplicate = insert_mapping_raw(
        &pool,
        platform_id,
        measure_id,
        "ARRAY['MONTH']::metric_reporting_grain[]",
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate (platform_id, measure_id) pair must fail the unique constraint: \
         {duplicate:?}"
    );
}

#[test]
fn invalid_foreign_keys_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);

    let unknown_platform = insert_mapping_raw(
        &pool,
        Uuid::new_v4(),
        measure_id,
        "ARRAY['DAY']::metric_reporting_grain[]",
    );
    assert!(
        matches!(
            unknown_platform,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown platform_id must fail the foreign key: {unknown_platform:?}"
    );

    let unknown_measure = insert_mapping_raw(
        &pool,
        platform_id,
        Uuid::new_v4(),
        "ARRAY['DAY']::metric_reporting_grain[]",
    );
    assert!(
        matches!(
            unknown_measure,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown measure_id must fail the foreign key: {unknown_measure:?}"
    );
}

#[test]
fn deleting_a_referenced_platform_fails_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    insert_mapping_raw(
        &pool,
        platform_id,
        measure_id,
        "ARRAY['DAY']::metric_reporting_grain[]",
    )
    .expect("Mapping insert must pass");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let delete = sql_query("DELETE FROM metric_platform WHERE platform_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .execute(&mut connection);
    assert!(
        matches!(
            delete,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced platform must fail closed: {delete:?}"
    );
    drop(connection);
    assert_eq!(
        mapping_count(&pool),
        1,
        "the mapping row must survive the rejected platform deletion"
    );
}

#[test]
fn deleting_a_referenced_measure_fails_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let (platform_id, measure_id) = fixture_pair(&pool);
    insert_mapping_raw(
        &pool,
        platform_id,
        measure_id,
        "ARRAY['DAY']::metric_reporting_grain[]",
    )
    .expect("Mapping insert must pass");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let delete = sql_query("DELETE FROM metric_measure WHERE measure_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(measure_id)
        .execute(&mut connection);
    assert!(
        matches!(
            delete,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced measure must fail closed: {delete:?}"
    );
    drop(connection);
    assert_eq!(
        mapping_count(&pool),
        1,
        "the mapping row must survive the rejected measure deletion"
    );
}

const REPORTING_GRAINS: [(MetricReportingGrain, &str); 3] = [
    (MetricReportingGrain::Day, "DAY"),
    (MetricReportingGrain::Month, "MONTH"),
    (MetricReportingGrain::ReportingPeriod, "REPORTING_PERIOD"),
];

#[test]
fn reporting_grain_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in REPORTING_GRAINS {
        assert_eq!(variant.to_string(), label);
        assert_eq!(MetricReportingGrain::from_str(label).unwrap(), variant);
        let json = format!("\"{label}\"");
        assert_eq!(serde_json::to_string(&variant).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<MetricReportingGrain>(&json).unwrap(),
            variant
        );
    }
    assert!(MetricReportingGrain::from_str("YEAR").is_err());
    assert!(MetricReportingGrain::from_str("day").is_err());
}

#[test]
fn every_reporting_grain_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in REPORTING_GRAINS {
        assert_db_enum_roundtrip::<
            MetricReportingGrain,
            crate::schema::sql_types::MetricReportingGrain,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_reporting_grain"),
            variant,
        );
    }
}

#[test]
fn supported_grains_vec_round_trips_through_diesel_with_order_preserved() {
    let (_guard, pool) = setup_registry_db();
    let platform_id = Uuid::new_v4();
    let first_measure_id = Uuid::new_v4();
    let second_measure_id = Uuid::new_v4();
    insert_platform_row(&pool, platform_id, "test_platform");
    insert_measure_row(&pool, first_measure_id, "test_measure");
    insert_measure_row(&pool, second_measure_id, "test_measure_2");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let ascending = vec![
        MetricReportingGrain::Day,
        MetricReportingGrain::Month,
        MetricReportingGrain::ReportingPeriod,
    ];
    let descending = vec![
        MetricReportingGrain::ReportingPeriod,
        MetricReportingGrain::Day,
    ];
    let first_mapping_id: Uuid = diesel::insert_into(metric_platform_measure::table)
        .values((
            metric_platform_measure::platform_id.eq(platform_id),
            metric_platform_measure::measure_id.eq(first_measure_id),
            metric_platform_measure::supported_grains.eq(ascending.clone()),
            metric_platform_measure::supports_country.eq(true),
            metric_platform_measure::supports_institution.eq(false),
            metric_platform_measure::supports_publication.eq(true),
            metric_platform_measure::direct_collection.eq(true),
            metric_platform_measure::enabled.eq(true),
        ))
        .returning(metric_platform_measure::platform_measure_id)
        .get_result(&mut connection)
        .expect("Failed to insert multi-grain mapping row");
    diesel::insert_into(metric_platform_measure::table)
        .values((
            metric_platform_measure::platform_id.eq(platform_id),
            metric_platform_measure::measure_id.eq(second_measure_id),
            metric_platform_measure::supported_grains.eq(descending.clone()),
            metric_platform_measure::supports_country.eq(false),
            metric_platform_measure::supports_institution.eq(true),
            metric_platform_measure::supports_publication.eq(false),
            metric_platform_measure::direct_collection.eq(false),
            metric_platform_measure::enabled.eq(false),
        ))
        .execute(&mut connection)
        .expect("Failed to insert descending-grain mapping row");

    let first: MetricPlatformMeasure = metric_platform_measure::table
        .filter(metric_platform_measure::measure_id.eq(first_measure_id))
        .first(&mut connection)
        .expect("Failed to load multi-grain mapping row");
    assert_eq!(first.platform_measure_id, first_mapping_id);
    assert_eq!(first.platform_id, platform_id);
    assert_eq!(first.measure_id, first_measure_id);
    assert_eq!(
        first.supported_grains, ascending,
        "the persisted grain array must preserve insertion order"
    );
    assert!(first.supports_country);
    assert!(!first.supports_institution);
    assert!(first.supports_publication);
    assert!(first.direct_collection);
    assert!(first.enabled);

    let second: MetricPlatformMeasure = metric_platform_measure::table
        .filter(metric_platform_measure::measure_id.eq(second_measure_id))
        .first(&mut connection)
        .expect("Failed to load descending-grain mapping row");
    assert_eq!(
        second.supported_grains, descending,
        "the persisted grain array must not be reordered or normalized"
    );
    assert!(!second.supports_country);
    assert!(second.supports_institution);
    assert!(!second.supports_publication);
    assert!(!second.direct_collection);
    assert!(!second.enabled);
}

// --------------------------------------------------------------------------
// `MET-WP1-12` protected administration coordinator
// --------------------------------------------------------------------------

/// Insert one platform row, so the mapping coordinator has a real,
/// exactly-coded platform to resolve. The measure side of every pair below is a
/// migration-owned seed, which `setup_registry_db` preserves.
fn seed_platform(pool: &PgPool, platform_code: &str) {
    insert_platform_row(pool, Uuid::new_v4(), platform_code);
}

fn new_mapping(platform_code: &str, measure_code: &str) -> NewMetricPlatformMeasure {
    NewMetricPlatformMeasure {
        platform_code: platform_code.to_string(),
        measure_code: measure_code.to_string(),
        supported_grains: vec![MetricReportingGrain::Day, MetricReportingGrain::Month],
        supports_country: true,
        supports_institution: false,
        supports_publication: true,
        direct_collection: false,
        enabled: true,
    }
}

fn patch_mapping(
    platform_code: &str,
    measure_code: &str,
    grains: Vec<MetricReportingGrain>,
    direct_collection: bool,
) -> PatchMetricPlatformMeasure {
    PatchMetricPlatformMeasure {
        platform_code: platform_code.to_string(),
        measure_code: measure_code.to_string(),
        supported_grains: grains,
        supports_country: true,
        supports_institution: false,
        supports_publication: true,
        direct_collection,
        enabled: true,
    }
}

#[test]
fn create_resolves_the_code_pair_and_audits_the_persisted_row() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");

    let created =
        create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
            .expect("create");

    assert_eq!(
        created.supported_grains,
        vec![MetricReportingGrain::Day, MetricReportingGrain::Month]
    );
    assert!(created.supports_country);
    assert!(!created.direct_collection);
    assert_ne!(created.platform_id, created.measure_id);

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].entity, "PLATFORM_MEASURE");
    assert_eq!(audit[0].action, "CREATE");
    assert_eq!(audit[0].entity_id, created.platform_measure_id);
    assert!(audit[0].before_state.is_none());
    assert_eq!(
        audit[0].after_state,
        serde_json::to_value(&created).expect("serialize persisted row")
    );

    // The lookup resolves the same row through the same exact code semantics.
    let found = metric_platform_measure_by_codes(&pool, "platform_a", "net_units").expect("lookup");
    assert_eq!(found, created);
}

#[test]
fn code_resolution_is_exact_at_every_entry_point() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "Platform_A");

    create_metric_platform_measure(&pool, "actor-1", &new_mapping("Platform_A", "net_units"))
        .expect("create");

    // Neither the platform code nor the measure code is folded.
    for (platform, measure) in [
        ("platform_a", "net_units"),
        ("PLATFORM_A", "net_units"),
        (" Platform_A", "net_units"),
        ("Platform_A", "NET_UNITS"),
        ("Platform_A", "net_units "),
    ] {
        assert!(
            matches!(
                metric_platform_measure_by_codes(&pool, platform, measure),
                Err(ThothError::EntityNotFound)
            ),
            "`{platform}`/`{measure}` must not fold onto the stored pair"
        );
        assert!(
            matches!(
                update_metric_platform_measure(
                    &pool,
                    "actor-1",
                    &patch_mapping(platform, measure, vec![MetricReportingGrain::Day], true),
                ),
                Err(ThothError::EntityNotFound)
            ),
            "`{platform}`/`{measure}` must not select the stored pair for update"
        );
    }
    assert_eq!(audit_rows(&pool).len(), 1, "no rejected call audited");
}

#[test]
fn a_duplicate_pair_fails_atomically_and_is_sanitised() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");
    create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
        .expect("first create");

    let error =
        create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
            .expect_err("duplicate pair");

    let ThothError::DatabaseConstraintError(message) = &error else {
        panic!("expected a bounded constraint error, got {error:?}");
    };
    assert_eq!(
        message.as_ref(),
        "A mapping between this metric platform and this metric measure already exists."
    );
    for leaked in [
        "metric_platform_measure_platform_id_measure_id_key",
        "duplicate key",
        "INSERT",
        "pg_",
    ] {
        assert!(!message.contains(leaked), "leaked `{leaked}`: {message}");
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform_measure)"),
        1
    );
    assert_eq!(audit_rows(&pool).len(), 1);
}

#[test]
fn an_unknown_platform_or_measure_code_is_reported_as_not_found() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");

    // The coordinator resolves both codes with ordinary reads before touching
    // the mapping, so an unknown code fails as EntityNotFound rather than as a
    // foreign-key violation.
    assert!(matches!(
        create_metric_platform_measure(&pool, "actor-1", &new_mapping("absent", "net_units")),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "absent")),
        Err(ThothError::EntityNotFound)
    ));
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform_measure)"),
        0
    );
    assert!(audit_rows(&pool).is_empty());
}

#[test]
fn the_supported_grains_check_still_rejects_empty_and_duplicate_arrays() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");

    for grains in [
        vec![],
        vec![MetricReportingGrain::Day, MetricReportingGrain::Day],
        vec![
            MetricReportingGrain::Month,
            MetricReportingGrain::Day,
            MetricReportingGrain::Month,
        ],
    ] {
        let mut mapping = new_mapping("platform_a", "net_units");
        mapping.supported_grains = grains.clone();
        let error = match create_metric_platform_measure(&pool, "actor-1", &mapping) {
            Ok(row) => panic!("{grains:?} must be rejected by the database, got {row:?}"),
            Err(error) => error,
        };

        let ThothError::DatabaseConstraintError(message) = &error else {
            panic!("{grains:?} must map to a bounded constraint error, got {error:?}");
        };
        assert_eq!(
            message.as_ref(),
            "Supported grains must list at least one reporting grain, with no duplicates."
        );
        for leaked in [
            "metric_platform_measure_supported_grains_check",
            "cardinality",
            "array_positions",
            "CHECK",
        ] {
            assert!(!message.contains(leaked), "leaked `{leaked}`: {message}");
        }
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform_measure)"),
        0,
        "every rejected mapping must leave no canonical row"
    );
    assert!(
        audit_rows(&pool).is_empty(),
        "every rejected mapping must leave no audit row"
    );
}

#[test]
fn update_replaces_only_the_mutable_fields_and_cannot_move_the_pair() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");
    seed_platform(&pool, "platform_b");
    let created =
        create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
            .expect("create");

    // A genuine no-op writes and audits nothing.
    let returned = update_metric_platform_measure(
        &pool,
        "actor-2",
        &patch_mapping(
            "platform_a",
            "net_units",
            vec![MetricReportingGrain::Day, MetricReportingGrain::Month],
            false,
        ),
    )
    .expect("no-op update");
    assert_eq!(returned, created);
    assert_eq!(audit_rows(&pool).len(), 1);

    // A real change moves only the approved mutable fields.
    let updated = update_metric_platform_measure(
        &pool,
        "actor-2",
        &PatchMetricPlatformMeasure {
            platform_code: "platform_a".to_string(),
            measure_code: "net_units".to_string(),
            supported_grains: vec![MetricReportingGrain::ReportingPeriod],
            supports_country: false,
            supports_institution: true,
            supports_publication: false,
            direct_collection: true,
            enabled: false,
        },
    )
    .expect("real update");

    assert_eq!(updated.platform_measure_id, created.platform_measure_id);
    assert_eq!(updated.platform_id, created.platform_id);
    assert_eq!(updated.measure_id, created.measure_id);
    assert_eq!(
        updated.supported_grains,
        vec![MetricReportingGrain::ReportingPeriod]
    );
    assert!(updated.direct_collection);
    assert!(!updated.enabled);

    // The identity could not be moved to the other platform: the patch input
    // has no field that could express it, and the mapping for the other pair
    // simply does not exist.
    assert!(matches!(
        metric_platform_measure_by_codes(&pool, "platform_b", "net_units"),
        Err(ThothError::EntityNotFound)
    ));

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 2);
    assert_eq!(
        audit[1].before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize before")
    );
    assert_eq!(
        audit[1].after_state,
        serde_json::to_value(&updated).expect("serialize after")
    );
}

#[test]
fn updating_a_mapping_does_not_lock_its_platform_or_measure_rows() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");
    create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
        .expect("create");

    // Hold a real FOR UPDATE lock on the referenced platform row on one
    // connection, and prove the mapping update still completes on another. If
    // the coordinator locked its parents, this would block until the holder
    // committed and the test would time out.
    let holder_url = crate::model::tests::db::test_db_url();
    let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
    let (held_tx, held_rx) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let mut connection = diesel::pg::PgConnection::establish(&holder_url)
            .expect("Failed to connect to the test database");
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                sql_query(
                    "SELECT platform_id FROM metric_platform WHERE code = 'platform_a' FOR UPDATE",
                )
                .execute(connection)?;
                held_tx.send(()).expect("signal that the lock is held");
                release_rx
                    .recv_timeout(std::time::Duration::from_secs(30))
                    .expect("wait for release");
                Ok(())
            })
            .expect("holder transaction");
    });

    held_rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the holder must acquire the platform lock");

    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let updater = {
        let pool = std::sync::Arc::clone(&pool);
        std::thread::spawn(move || {
            let result = update_metric_platform_measure(
                &pool,
                "actor-2",
                &patch_mapping(
                    "platform_a",
                    "net_units",
                    vec![MetricReportingGrain::Day],
                    true,
                ),
            );
            done_tx.send(()).ok();
            result
        })
    };

    let unblocked = done_rx
        .recv_timeout(std::time::Duration::from_secs(15))
        .is_ok();
    release_tx.send(()).expect("release the holder");
    holder.join().expect("holder thread");
    let updated = updater.join().expect("updater thread").expect("update");

    assert!(
        unblocked,
        "the mapping update blocked on the platform row lock, so the coordinator \
         is locking rows it must not lock"
    );
    assert!(updated.direct_collection);
}

#[test]
fn concurrent_platform_and_mapping_updates_do_not_deadlock() {
    let (_guard, pool) = setup_registry_db();
    seed_platform(&pool, "platform_a");
    create_metric_platform_measure(&pool, "actor-1", &new_mapping("platform_a", "net_units"))
        .expect("create");

    // The registry coordinators take one lock each, on different tables, so no
    // application-defined lock cycle exists between them however they interleave.
    let platform = {
        let pool = std::sync::Arc::clone(&pool);
        std::thread::spawn(move || {
            crate::model::metric_platform::crud::update_metric_platform(
                &pool,
                "actor-a",
                &crate::model::metric_platform::PatchMetricPlatform {
                    code: "platform_a".to_string(),
                    display_name: "Renamed".to_string(),
                    enabled: true,
                    public_description: None,
                },
            )
        })
    };
    let mapping = {
        let pool = std::sync::Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_platform_measure(
                &pool,
                "actor-b",
                &patch_mapping(
                    "platform_a",
                    "net_units",
                    vec![MetricReportingGrain::Month],
                    true,
                ),
            )
        })
    };

    platform
        .join()
        .expect("platform thread")
        .expect("platform update");
    mapping
        .join()
        .expect("mapping thread")
        .expect("mapping update");

    assert_eq!(
        audit_rows(&pool).len(),
        3,
        "one mapping CREATE plus one audited update on each registry"
    );
}

#[test]
fn the_coordinator_locks_only_the_mapping_row() {
    let source = include_str!("crud.rs");

    assert_eq!(
        source.matches(".for_update()").count(),
        1,
        "the mapping coordinator must request exactly one row lock"
    );
    for forbidden in FORBIDDEN_JOIN_CONSTRUCTS {
        assert!(
            !source.contains(forbidden),
            "a joined multi-table FOR UPDATE is prohibited: found `{forbidden}`"
        );
    }
    // The single lock is on the mapping table, and `resolve_pair` — which reads
    // the platform and the measure — carries no lock at all.
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
        opening.contains("metric_platform_measure::table"),
        "the lock must be taken on the metric_platform_measure row itself, found: {opening}"
    );
    for parent in ["metric_platform::table", "metric_measure::table"] {
        assert!(
            !opening.contains(parent),
            "the locked statement must not reach `{parent}`: {opening}"
        );
    }
    assert!(
        !after_lock.contains("for_update"),
        "no second lock target may follow the canonical row lock"
    );

    // Code resolution is a separate, unlocked helper.
    let resolve = source
        .split_once("fn resolve_pair(")
        .expect("code resolution helper")
        .1
        .split_once("\n}\n")
        .expect("helper body")
        .0;
    assert!(
        !resolve.contains("for_update"),
        "platform and measure code resolution must remain non-locking reads"
    );
}
