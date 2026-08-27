//! Focused `MET-WP1-01` database tests for the `metric_measure` registry and
//! its two migration-owned seed rows.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};

use super::{MetricMeasure, MetricMeasureCategory, MetricMeasureUnit};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::schema::metric_measure;

/// The exact approved `title_sessions` definition (amendment B4 of #836).
pub(crate) const TITLE_SESSIONS_DEFINITION: &str = "Count of title sessions: one or more \
successful qualifying requests for the same work by the same transient user during a rolling \
30-minute session, attributed to the UTC date on which the session began and counted once per \
DOI and country within that session.";

/// The exact approved `net_units` definition (amendment B4 of #836).
pub(crate) const NET_UNITS_DEFINITION: &str = "Signed net sales units for a work over the \
reported period; positive values represent net units sold and negative values represent \
refunds or returns as reported by the source.";

#[derive(diesel::QueryableByName, Debug, PartialEq, Eq)]
struct SeedRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    display_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    category: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    unit: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    allow_negative: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    public_visibility: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    additive_across_time: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    additive_across_works: bool,
    #[diesel(sql_type = diesel::sql_types::Text)]
    definition: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    methodology_version: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    enabled: bool,
}

fn seed_row(pool: &PgPool, code: &str) -> SeedRow {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT code, display_name, category::text AS category, unit::text AS unit, \
                allow_negative, public_visibility, additive_across_time, \
                additive_across_works, definition, methodology_version, enabled \
         FROM metric_measure WHERE code = $1",
    )
    .bind::<diesel::sql_types::Text, _>(code)
    .get_result(&mut connection)
    .expect("Failed to read seeded metric_measure row")
}

fn insert_measure_raw(
    pool: &PgPool,
    code: &str,
    display_name: &str,
    definition: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_measure \
             (code, display_name, category, unit, allow_negative, additive_across_time, \
              additive_across_works, definition, enabled) \
         VALUES ($1, $2, 'USAGE', 'COUNT', FALSE, TRUE, TRUE, $3, TRUE)",
    )
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(display_name)
    .bind::<diesel::sql_types::Text, _>(definition)
    .execute(&mut connection)
}

#[test]
fn category_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_measure_category"),
        ["USAGE", "SALES"]
    );
}

#[test]
fn unit_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(enum_labels(&pool, "metric_measure_unit"), ["COUNT"]);
}

#[test]
fn migration_seeds_exactly_the_two_approved_measures() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_measure)"),
        2,
        "MET-WP1-01 must seed exactly two metric_measure rows"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_measure \
              WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2,
        "the seeded measure codes must be title_sessions and net_units"
    );
}

#[test]
fn title_sessions_seed_matches_the_approved_specification() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        seed_row(&pool, "title_sessions"),
        SeedRow {
            code: "title_sessions".to_string(),
            display_name: "Title sessions".to_string(),
            category: "USAGE".to_string(),
            unit: "COUNT".to_string(),
            allow_negative: false,
            public_visibility: true,
            additive_across_time: true,
            additive_across_works: true,
            definition: TITLE_SESSIONS_DEFINITION.to_string(),
            methodology_version: Some("cloudfront-title-session/2".to_string()),
            enabled: true,
        }
    );
}

#[test]
fn net_units_seed_matches_the_approved_specification() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        seed_row(&pool, "net_units"),
        SeedRow {
            code: "net_units".to_string(),
            display_name: "Net units".to_string(),
            category: "SALES".to_string(),
            unit: "COUNT".to_string(),
            allow_negative: true,
            public_visibility: true,
            additive_across_time: true,
            additive_across_works: true,
            definition: NET_UNITS_DEFINITION.to_string(),
            methodology_version: None,
            enabled: true,
        }
    );
}

#[test]
fn duplicate_measure_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let duplicate = insert_measure_raw(
        &pool,
        "title_sessions",
        "Duplicate of a seeded code",
        "Some definition.",
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "duplicate measure code must fail the unique constraint: {duplicate:?}"
    );
}

#[test]
fn blank_measure_code_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_measure_raw(&pool, blank, "Display name", "Some definition.");
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank measure code {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn blank_measure_display_name_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_measure_raw(&pool, "test_measure", blank, "Some definition.");
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank measure display name {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn blank_measure_definition_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_measure_raw(&pool, "test_measure", "Display name", blank);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank measure definition {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn category_and_unit_string_conversions_round_trip_and_reject_unknown_values() {
    for (variant, label) in [
        (MetricMeasureCategory::Usage, "USAGE"),
        (MetricMeasureCategory::Sales, "SALES"),
    ] {
        assert_eq!(variant.to_string(), label);
        assert_eq!(MetricMeasureCategory::from_str(label).unwrap(), variant);
        let json = format!("\"{label}\"");
        assert_eq!(serde_json::to_string(&variant).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<MetricMeasureCategory>(&json).unwrap(),
            variant
        );
    }
    assert!(MetricMeasureCategory::from_str("REVENUE").is_err());

    assert_eq!(MetricMeasureUnit::Count.to_string(), "COUNT");
    assert_eq!(
        MetricMeasureUnit::from_str("COUNT").unwrap(),
        MetricMeasureUnit::Count
    );
    assert_eq!(
        serde_json::to_string(&MetricMeasureUnit::Count).unwrap(),
        "\"COUNT\""
    );
    assert_eq!(
        serde_json::from_str::<MetricMeasureUnit>("\"COUNT\"").unwrap(),
        MetricMeasureUnit::Count
    );
    assert!(MetricMeasureUnit::from_str("CURRENCY").is_err());
}

#[test]
fn every_category_and_unit_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in [
        (MetricMeasureCategory::Usage, "USAGE"),
        (MetricMeasureCategory::Sales, "SALES"),
    ] {
        assert_db_enum_roundtrip::<
            MetricMeasureCategory,
            crate::schema::sql_types::MetricMeasureCategory,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_measure_category"),
            variant,
        );
    }
    assert_db_enum_roundtrip::<MetricMeasureUnit, crate::schema::sql_types::MetricMeasureUnit>(
        pool.as_ref(),
        "'COUNT'::metric_measure_unit",
        MetricMeasureUnit::Count,
    );
}

#[test]
fn seeded_measures_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    let measures: Vec<MetricMeasure> = metric_measure::table
        .order(metric_measure::code.asc())
        .load(&mut connection)
        .expect("Failed to load seeded metric_measure rows");
    assert_eq!(measures.len(), 2);

    let net_units = &measures[0];
    assert_eq!(net_units.code, "net_units");
    assert_eq!(net_units.display_name, "Net units");
    assert_eq!(net_units.category, MetricMeasureCategory::Sales);
    assert_eq!(net_units.unit, MetricMeasureUnit::Count);
    assert!(net_units.allow_negative);
    assert!(net_units.public_visibility);
    assert!(net_units.additive_across_time);
    assert!(net_units.additive_across_works);
    assert_eq!(net_units.definition, NET_UNITS_DEFINITION);
    assert_eq!(net_units.methodology_version, None);
    assert!(net_units.enabled);

    let title_sessions = &measures[1];
    assert_eq!(title_sessions.code, "title_sessions");
    assert_eq!(title_sessions.display_name, "Title sessions");
    assert_eq!(title_sessions.category, MetricMeasureCategory::Usage);
    assert_eq!(title_sessions.unit, MetricMeasureUnit::Count);
    assert!(!title_sessions.allow_negative);
    assert!(title_sessions.public_visibility);
    assert!(title_sessions.additive_across_time);
    assert!(title_sessions.additive_across_works);
    assert_eq!(title_sessions.definition, TITLE_SESSIONS_DEFINITION);
    assert_eq!(
        title_sessions.methodology_version.as_deref(),
        Some("cloudfront-title-session/2")
    );
    assert!(title_sessions.enabled);
}
