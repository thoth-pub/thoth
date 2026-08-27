//! Focused `MET-WP1-01` database tests for the `metric_platform_measure`
//! registry mapping, including the `supported_grains` array contract and the
//! non-cascading registry foreign keys.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use super::{MetricPlatformMeasure, MetricReportingGrain};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{
    enum_labels, insert_platform_row, scalar_i64, setup_registry_db,
};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::schema::metric_platform_measure;

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
