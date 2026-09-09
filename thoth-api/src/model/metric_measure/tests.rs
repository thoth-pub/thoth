//! Focused `MET-WP1-01` database tests for the `metric_measure` registry and
//! its two migration-owned seed rows.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};

use super::crud::{create_metric_measure, metric_measure_by_code, update_metric_measure};
use super::{
    MetricMeasure, MetricMeasureCategory, MetricMeasureUnit, NewMetricMeasure, PatchMetricMeasure,
};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{
    audit_rows, display_name_of, enum_labels, scalar_i64, serialized_update_chain,
    setup_registry_db, FORBIDDEN_JOIN_CONSTRUCTS,
};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::schema::metric_measure;
use thoth_errors::ThothError;

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

// --------------------------------------------------------------------------
// `MET-WP1-12` protected administration coordinator
// --------------------------------------------------------------------------

fn new_measure(code: &str) -> NewMetricMeasure {
    NewMetricMeasure {
        code: code.to_string(),
        display_name: format!("Measure {code}"),
        category: MetricMeasureCategory::Usage,
        unit: MetricMeasureUnit::Count,
        allow_negative: false,
        public_visibility: true,
        additive_across_time: true,
        additive_across_works: true,
        definition: format!("What {code} counts."),
        methodology_version: Some("method/1".to_string()),
        enabled: true,
    }
}

fn patch_measure(code: &str, display_name: &str, enabled: bool) -> PatchMetricMeasure {
    PatchMetricMeasure {
        code: code.to_string(),
        display_name: display_name.to_string(),
        public_visibility: true,
        definition: format!("What {code} counts."),
        methodology_version: Some("method/1".to_string()),
        enabled,
    }
}

#[test]
fn create_persists_the_exact_values_and_audits_the_persisted_row() {
    let (_guard, pool) = setup_registry_db();

    let created =
        create_metric_measure(&pool, "actor-1", &new_measure("downloads")).expect("create");

    assert_eq!(created.code, "downloads");
    assert_eq!(created.category, MetricMeasureCategory::Usage);
    assert_eq!(created.unit, MetricMeasureUnit::Count);
    assert!(!created.allow_negative);
    assert_eq!(created.methodology_version.as_deref(), Some("method/1"));
    assert_eq!(created.created_at, created.updated_at);

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].entity, "MEASURE");
    assert_eq!(audit[0].action, "CREATE");
    assert_eq!(audit[0].entity_id, created.measure_id);
    assert!(audit[0].before_state.is_none());
    assert_eq!(
        audit[0].after_state,
        serde_json::to_value(&created).expect("serialize persisted row")
    );
}

#[test]
fn the_seeded_measures_are_administrable_by_their_stable_codes() {
    // Amendment 2 section F: this proof needs a fixture that genuinely
    // preserves the migration-owned seeds. `setup_registry_db` reverts through
    // the MET-WP1-01 registry migration and re-applies it, so the seeds are
    // present exactly as the migration wrote them, unlike the generic
    // truncating reset.
    let (_guard, pool) = setup_registry_db();

    for code in ["title_sessions", "net_units"] {
        let found = metric_measure_by_code(&pool, code)
            .unwrap_or_else(|error| panic!("seeded `{code}` must be addressable: {error:?}"));
        assert_eq!(found.code, code, "no UUID discovery is needed");
    }

    let before = metric_measure_by_code(&pool, "title_sessions").expect("seeded read");
    assert_eq!(before.definition, TITLE_SESSIONS_DEFINITION);
    assert_eq!(
        before.methodology_version.as_deref(),
        Some("cloudfront-title-session/2")
    );

    // Administering a seeded measure changes only its mutable fields; the
    // seeded identity and canonical semantics survive.
    let updated = update_metric_measure(
        &pool,
        "actor-1",
        &PatchMetricMeasure {
            code: "title_sessions".to_string(),
            display_name: "Title sessions (revised)".to_string(),
            public_visibility: before.public_visibility,
            definition: before.definition.clone(),
            methodology_version: Some("cloudfront-title-session/3".to_string()),
            enabled: before.enabled,
        },
    )
    .expect("update a seeded measure");

    assert_eq!(updated.measure_id, before.measure_id);
    assert_eq!(updated.code, "title_sessions");
    assert_eq!(updated.category, before.category);
    assert_eq!(updated.unit, before.unit);
    assert_eq!(updated.allow_negative, before.allow_negative);
    assert_eq!(updated.additive_across_time, before.additive_across_time);
    assert_eq!(updated.additive_across_works, before.additive_across_works);
    assert_eq!(updated.display_name, "Title sessions (revised)");
    assert_eq!(
        updated.methodology_version.as_deref(),
        Some("cloudfront-title-session/3")
    );

    // The other seed was not touched, and exactly the two seeds still exist.
    let untouched = metric_measure_by_code(&pool, "net_units").expect("other seed");
    assert_eq!(untouched.definition, NET_UNITS_DEFINITION);
    assert_eq!(untouched.methodology_version, None);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))"
        ),
        2,
        "administration must not re-seed or duplicate a migration-owned measure"
    );
}

#[test]
fn a_code_is_stored_and_matched_exactly_with_no_normalisation() {
    let (_guard, pool) = setup_registry_db();

    // A case variant of a seeded code is a different code, and creating it
    // neither collides with nor rewrites the seed.
    let created =
        create_metric_measure(&pool, "actor-1", &new_measure("Title_Sessions")).expect("create");
    assert_eq!(created.code, "Title_Sessions");

    let seed = metric_measure_by_code(&pool, "title_sessions").expect("seed still addressable");
    assert_ne!(seed.measure_id, created.measure_id);
    assert_eq!(seed.definition, TITLE_SESSIONS_DEFINITION);

    for absent in ["TITLE_SESSIONS", " title_sessions", "title_sessions "] {
        assert!(
            matches!(
                metric_measure_by_code(&pool, absent),
                Err(ThothError::EntityNotFound)
            ),
            "`{absent}` must not fold onto a stored code"
        );
    }
}

#[test]
fn a_duplicate_code_and_a_blank_field_fail_atomically_and_are_sanitised() {
    let (_guard, pool) = setup_registry_db();

    // The seeded code is already taken, so this exercises the real unique index.
    let error = create_metric_measure(&pool, "actor-1", &new_measure("title_sessions"))
        .expect_err("duplicate code");
    assert!(
        matches!(&error, ThothError::DatabaseConstraintError(message)
            if message.as_ref() == "A metric measure with this code already exists."),
        "got {error:?}"
    );

    let mut blank_definition = new_measure("blank_def");
    blank_definition.definition = "  ".to_string();
    let error =
        create_metric_measure(&pool, "actor-1", &blank_definition).expect_err("blank definition");
    let ThothError::DatabaseConstraintError(message) = &error else {
        panic!("expected a bounded constraint error, got {error:?}");
    };
    assert_eq!(
        message.as_ref(),
        "Metric measure definition must not be an empty string."
    );
    for leaked in ["metric_measure_definition_check", "CHECK", "INSERT", "pg_"] {
        assert!(!message.contains(leaked), "leaked `{leaked}`: {message}");
    }

    assert!(
        audit_rows(&pool).is_empty(),
        "a rejected create must write no audit row"
    );
}

#[test]
fn update_replaces_only_the_mutable_fields_and_a_no_op_audits_nothing() {
    let (_guard, pool) = setup_registry_db();
    let created = create_metric_measure(&pool, "actor-1", &new_measure("target")).expect("create");

    // A genuine no-op.
    let returned = update_metric_measure(
        &pool,
        "actor-2",
        &patch_measure("target", "Measure target", true),
    )
    .expect("no-op update");
    assert_eq!(returned, created);
    assert_eq!(returned.updated_at, created.updated_at);
    assert_eq!(audit_rows(&pool).len(), 1, "a no-op audits nothing");

    // A real change: only the mutable fields move.
    let updated = update_metric_measure(
        &pool,
        "actor-2",
        &PatchMetricMeasure {
            code: "target".to_string(),
            display_name: "Renamed".to_string(),
            public_visibility: false,
            definition: "A revised definition.".to_string(),
            methodology_version: None,
            enabled: false,
        },
    )
    .expect("real update");

    assert_eq!(updated.measure_id, created.measure_id);
    assert_eq!(updated.code, "target");
    assert_eq!(updated.category, created.category);
    assert_eq!(updated.unit, created.unit);
    assert_eq!(updated.allow_negative, created.allow_negative);
    assert_eq!(updated.additive_across_time, created.additive_across_time);
    assert_eq!(updated.additive_across_works, created.additive_across_works);
    assert_eq!(updated.created_at, created.created_at);
    assert!(updated.updated_at > created.updated_at);
    // An omitted nullable field stores SQL NULL rather than retaining the value.
    assert_eq!(updated.methodology_version, None);

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
fn an_unknown_code_is_reported_as_not_found_and_audits_nothing() {
    let (_guard, pool) = setup_registry_db();

    assert!(matches!(
        metric_measure_by_code(&pool, "absent"),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        update_metric_measure(&pool, "actor-1", &patch_measure("absent", "x", true)),
        Err(ThothError::EntityNotFound)
    ));
    assert!(audit_rows(&pool).is_empty());
}

#[test]
fn two_competing_updates_serialise_and_their_audit_chain_matches_commit_order() {
    let (_guard, pool) = setup_registry_db();
    let created =
        create_metric_measure(&pool, "actor-1", &new_measure("contended")).expect("create");

    let first = {
        let pool = std::sync::Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_measure(&pool, "actor-a", &patch_measure("contended", "A", true))
        })
    };
    let second = {
        let pool = std::sync::Arc::clone(&pool);
        std::thread::spawn(move || {
            update_metric_measure(&pool, "actor-b", &patch_measure("contended", "B", true))
        })
    };
    // Joining only waits for both transactions to finish. Neither the spawn
    // order nor the join order reveals which of them won the row lock, and
    // neither is used as evidence below.
    first.join().expect("thread a").expect("update a");
    second.join().expect("thread b").expect("update b");

    let audit = audit_rows(&pool);
    assert_eq!(audit.len(), 3, "one CREATE and exactly two UPDATE entries");

    // Serialized last-write-wins. The two transitions are ordered purely by
    // their exact before/after states; see `serialized_update_chain` for why the
    // row lock, and not `created_at`, is what makes that order authoritative.
    let chain = serialized_update_chain(&audit, "MEASURE", created.measure_id);
    assert_eq!(chain.len(), 2, "exactly two UPDATE transitions");

    assert_eq!(
        chain[0].before_state.as_ref().expect("before"),
        &serde_json::to_value(&created).expect("serialize created"),
        "the update that committed first must have overwritten the created row"
    );
    assert_eq!(
        chain[1].before_state.as_ref().expect("before"),
        &chain[0].after_state,
        "the update that committed second must have read the first one's \
         committed state, with no gap"
    );

    let final_row = metric_measure_by_code(&pool, "contended").expect("final read");
    assert_eq!(
        chain[1].after_state,
        serde_json::to_value(&final_row).expect("serialize final"),
        "the last transition must describe the committed state"
    );

    // Both administrators are represented exactly once, each paired with the
    // value it requested, whichever of them committed first.
    let mut actors = [chain[0].actor.as_str(), chain[1].actor.as_str()];
    actors.sort_unstable();
    assert_eq!(
        actors,
        ["actor-a", "actor-b"],
        "each competing update is audited exactly once"
    );
    let mut names = [
        display_name_of(&chain[0].after_state),
        display_name_of(&chain[1].after_state),
    ];
    names.sort_unstable();
    assert_eq!(names, ["A", "B"], "both requested values were written");
    for transition in &chain {
        let expected_actor = if display_name_of(&transition.after_state) == "A" {
            "actor-a"
        } else {
            "actor-b"
        };
        assert_eq!(
            transition.actor, expected_actor,
            "each transition must be attributed to the actor that requested it"
        );
    }
    assert_eq!(
        final_row.display_name,
        display_name_of(&chain[1].after_state),
        "the surviving row is the one the last committed transition wrote"
    );
}

#[test]
fn the_coordinator_takes_exactly_one_application_row_lock() {
    let source = include_str!("crud.rs");

    assert_eq!(
        source.matches(".for_update()").count(),
        1,
        "the measure coordinator must request exactly one row lock"
    );
    for forbidden in FORBIDDEN_JOIN_CONSTRUCTS {
        assert!(
            !source.contains(forbidden),
            "a joined multi-table FOR UPDATE is prohibited: found `{forbidden}`"
        );
    }
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
        opening.contains("metric_measure::table.filter(metric_measure::code.eq(&data.code))"),
        "the one lock must be taken on the canonical metric_measure row selected by exact \
         code, found: {opening}"
    );
    assert!(
        !after_lock.contains("for_update"),
        "no second lock target may follow the canonical row lock"
    );
}
