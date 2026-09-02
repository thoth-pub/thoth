//! Focused `MET-WP1-08` database tests for `metric_operas_mapping`: the
//! approved field/default contract, both `enabled` states, arbitrary nonblank
//! URI configuration text, blank/whitespace-only URI rejection, registered
//! platform/measure referential integrity, one-mapping-per-pair uniqueness,
//! restricted (non-cascading) deletion, the exact column/check/foreign-key/
//! index inventory and the targeted revert/reapply of the OPERAS mapping
//! migration.
//!
//! The registry fixtures are the existing `pub(crate)` helpers defined by
//! `metric_platform/tests.rs`, `metric_import/tests.rs` and
//! `metric_record/tests.rs`, consumed as-is. The `metric_measure` and
//! `metric_platform_measure` fixtures below are local raw-SQL helpers rather
//! than reuses of the private equivalents in
//! `metric_platform_measure/tests.rs`: no other model's test module is
//! widened, because this task's write budget contains only this file.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no mapping at runtime, approves no real OPERAS event, measure or
//! uploader URI, and implements no export eligibility, capability
//! enforcement, payload construction, delivery, inbound synchronization or
//! reconciliation. Nothing here pretends otherwise: in particular, an
//! accepted URI string is evidence of nonblank-text storage only, never of
//! URI validity, reachability or OPERAS approval.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricOperasMapping;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{insert_platform_row, scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_definition, index_names};
use crate::model::tests::db::test_db_url;
use crate::schema::metric_operas_mapping;

/// The Diesel migration version of `thoth-api/migrations/20260904_v1.9.0`.
const MET_WP1_08_MIGRATION_VERSION: &str = "20260904";

/// The OPERAS synchronization and reconciliation ledgers named by the approved
/// design. They remain approved *future* architecture: MET-WP1-08 must not
/// create any of them.
const DEFERRED_OPERAS_TABLES: [&str; 4] = [
    "metric_operas_export",
    "metric_operas_import",
    "metric_reconciliation_issue",
    "metric_reconciliation_run",
];

/// Column names that would betray a delivery, claim, retry, loop-prevention
/// or audit protocol — or a duplicated registry flag — having been smuggled
/// into this configuration-only slice. `direct_collection` belongs to
/// `metric_platform_measure` and must not be mirrored here; the approved
/// mapping shorthand carries no timestamps.
const DEFERRED_MAPPING_COLUMNS: [&str; 10] = [
    "attempt_count",
    "claimed_at",
    "claimed_by",
    "created_at",
    "direct_collection",
    "last_error",
    "remote_event_id",
    "request_hash",
    "status",
    "updated_at",
];

/// Revert migrations until the `MET-WP1-08` OPERAS mapping migration itself
/// has been reverted.
///
/// The same durable pattern as `revert_through_rollup_delta_migration` and its
/// predecessors: a bare `revert_last_migration` would only mean "the OPERAS
/// mapping migration" while it happens to be the newest applied migration.
/// Reverting down to and including the target keeps the meaning under any
/// later migration order, and no future migration name is assumed or
/// hard-coded.
fn revert_through_operas_mapping_migration(connection: &mut PgConnection) {
    let operas_mapping_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_08_MIGRATION_VERSION);
    assert!(
        operas_mapping_migration_applied,
        "the MET-WP1-08 OPERAS mapping migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_08_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one `metric_measure` row with an explicit id through raw SQL.
///
/// Local to this module: the equivalent helper in
/// `metric_platform_measure/tests.rs` is private, and this task's write budget
/// does not permit widening that file.
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

/// Insert one `metric_platform_measure` registry row through raw SQL and
/// return its primary key.
fn insert_platform_measure_row(pool: &PgPool, platform_id: Uuid, measure_id: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        platform_measure_id: Uuid,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_platform_measure \
             (platform_id, measure_id, supported_grains, supports_country, \
              supports_institution, supports_publication, direct_collection, enabled) \
         VALUES ($1, $2, ARRAY['MONTH']::public.metric_reporting_grain[], \
                 TRUE, FALSE, FALSE, FALSE, TRUE) \
         RETURNING platform_measure_id",
    )
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .get_result::<Row>(&mut connection)
    .expect("Failed to insert metric_platform_measure fixture row")
    .platform_measure_id
}

/// One platform/measure pair registered together in `metric_platform_measure`,
/// which is exactly what an OPERAS mapping is permitted to name.
struct RegisteredPair {
    platform_id: Uuid,
    measure_id: Uuid,
    platform_measure_id: Uuid,
}

/// Register one fresh platform/measure pair. `label` keeps the registry `code`
/// values unique across pairs within one test.
fn fixture_registered_pair(pool: &PgPool, label: &str) -> RegisteredPair {
    let platform_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, &format!("platform_{label}"));
    insert_measure_row(pool, measure_id, &format!("measure_{label}"));
    let platform_measure_id = insert_platform_measure_row(pool, platform_id, measure_id);
    RegisteredPair {
        platform_id,
        measure_id,
        platform_measure_id,
    }
}

/// Insert one OPERAS mapping through raw SQL so the database default and
/// constraints are exercised rather than restated by a Diesel fixture.
#[allow(clippy::too_many_arguments)]
fn insert_mapping_row(
    pool: &PgPool,
    mapping_id: Option<Uuid>,
    platform_id: Uuid,
    measure_id: Uuid,
    event_uri: &str,
    measure_uri: &str,
    uploader_uri: &str,
    enabled: bool,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match mapping_id {
        Some(mapping_id) => sql_query(
            "INSERT INTO metric_operas_mapping \
                 (mapping_id, platform_id, measure_id, event_uri, measure_uri, \
                  uploader_uri, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(mapping_id)
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .bind::<diesel::sql_types::Uuid, _>(measure_id)
        .bind::<diesel::sql_types::Text, _>(event_uri)
        .bind::<diesel::sql_types::Text, _>(measure_uri)
        .bind::<diesel::sql_types::Text, _>(uploader_uri)
        .bind::<diesel::sql_types::Bool, _>(enabled)
        .execute(&mut connection),
        None => sql_query(
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, measure_uri, uploader_uri, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .bind::<diesel::sql_types::Uuid, _>(measure_id)
        .bind::<diesel::sql_types::Text, _>(event_uri)
        .bind::<diesel::sql_types::Text, _>(measure_uri)
        .bind::<diesel::sql_types::Text, _>(uploader_uri)
        .bind::<diesel::sql_types::Bool, _>(enabled)
        .execute(&mut connection),
    }
}

/// The single stored mapping row.
fn only_mapping(pool: &PgPool) -> MetricOperasMapping {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_operas_mapping::table
        .first(&mut connection)
        .expect("Failed to load the stored OPERAS mapping")
}

#[test]
fn migration_seeds_no_operas_mapping_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        0,
        "MET-WP1-08 must not seed any metric_operas_mapping row: real OPERAS \
         event/measure/uploader URI and platform/measure mappings remain \
         unapproved external inputs"
    );
}

#[test]
fn a_complete_operas_mapping_round_trips_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    // These URI strings are fixture text. They are not approved OPERAS values
    // and the database applies no URI semantics to them.
    let mapping_id: Uuid = diesel::insert_into(metric_operas_mapping::table)
        .values((
            metric_operas_mapping::platform_id.eq(pair.platform_id),
            metric_operas_mapping::measure_id.eq(pair.measure_id),
            metric_operas_mapping::event_uri.eq("fixture-event-uri"),
            metric_operas_mapping::measure_uri.eq("fixture-measure-uri"),
            metric_operas_mapping::uploader_uri.eq("fixture-uploader-uri"),
            metric_operas_mapping::enabled.eq(true),
        ))
        .returning(metric_operas_mapping::mapping_id)
        .get_result(&mut connection)
        .expect("Failed to insert the OPERAS mapping");

    let loaded: MetricOperasMapping = metric_operas_mapping::table
        .filter(metric_operas_mapping::mapping_id.eq(mapping_id))
        .first(&mut connection)
        .expect("Failed to load the OPERAS mapping");
    assert_eq!(
        loaded,
        MetricOperasMapping {
            mapping_id,
            platform_id: pair.platform_id,
            measure_id: pair.measure_id,
            event_uri: "fixture-event-uri".to_string(),
            measure_uri: "fixture-measure-uri".to_string(),
            uploader_uri: "fixture-uploader-uri".to_string(),
            enabled: true,
        }
    );
}

#[test]
fn both_enabled_states_round_trip_without_an_implicit_default() {
    let (_guard, pool) = setup_registry_db();

    for (label, enabled) in [("enabled", true), ("disabled", false)] {
        let pair = fixture_registered_pair(&pool, label);
        insert_mapping_row(
            &pool,
            None,
            pair.platform_id,
            pair.measure_id,
            "fixture-event-uri",
            "fixture-measure-uri",
            "fixture-uploader-uri",
            enabled,
        )
        .unwrap_or_else(|error| panic!("enabled = {enabled} must be accepted: {error:?}"));

        let mut connection = pool.get().expect("Failed to get DB connection");
        let stored: bool = metric_operas_mapping::table
            .filter(metric_operas_mapping::platform_id.eq(pair.platform_id))
            .select(metric_operas_mapping::enabled)
            .first(&mut connection)
            .expect("Failed to load the stored enabled flag");
        assert_eq!(
            stored, enabled,
            "enabled = {enabled} must round-trip exactly"
        );
    }

    // Both states are representable, so no database default is inferable from
    // the stored rows; the NOT NULL test below proves the column genuinely has
    // no default to fall back on.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        2
    );
}

#[test]
fn arbitrary_nonblank_uri_text_round_trips_in_every_uri_column() {
    let (_guard, pool) = setup_registry_db();

    // Deliberately mixed: an http URI, a URN, a string that is not a URI at
    // all, a padded string and non-ASCII text. Accepting all of them is the
    // point — this slice stores nonblank configuration text and applies no
    // scheme, parser, host, normalization or uniqueness rule. The padded value
    // must round-trip byte-for-byte, proving nothing trims or normalizes it.
    let values = [
        "https://metrics.operas-eu.org/fixture/event",
        "urn:fixture:operas:measure",
        "not-a-uri-at-all",
        "  padded fixture value  ",
        "ünïcödé/fixture/path",
    ];

    for (column_index, column) in ["event_uri", "measure_uri", "uploader_uri"]
        .into_iter()
        .enumerate()
    {
        for (value_index, value) in values.into_iter().enumerate() {
            let pair = fixture_registered_pair(&pool, &format!("c{column_index}v{value_index}"));
            let (event_uri, measure_uri, uploader_uri) = match column {
                "event_uri" => (value, "baseline-measure-uri", "baseline-uploader-uri"),
                "measure_uri" => ("baseline-event-uri", value, "baseline-uploader-uri"),
                _ => ("baseline-event-uri", "baseline-measure-uri", value),
            };
            insert_mapping_row(
                &pool,
                None,
                pair.platform_id,
                pair.measure_id,
                event_uri,
                measure_uri,
                uploader_uri,
                true,
            )
            .unwrap_or_else(|error| {
                panic!("arbitrary nonblank {column} {value:?} must be accepted: {error:?}")
            });

            let mut connection = pool.get().expect("Failed to get DB connection");
            let stored: (String, String, String) = metric_operas_mapping::table
                .filter(metric_operas_mapping::platform_id.eq(pair.platform_id))
                .select((
                    metric_operas_mapping::event_uri,
                    metric_operas_mapping::measure_uri,
                    metric_operas_mapping::uploader_uri,
                ))
                .first(&mut connection)
                .expect("Failed to load the stored URI columns");
            assert_eq!(
                stored,
                (
                    event_uri.to_string(),
                    measure_uri.to_string(),
                    uploader_uri.to_string()
                ),
                "{column} = {value:?} must round-trip unchanged, and must not \
                 leak into another URI column"
            );
        }
    }
}

#[test]
fn blank_and_whitespace_only_event_uri_is_rejected() {
    assert_blank_uri_rejected("event_uri");
}

#[test]
fn blank_and_whitespace_only_measure_uri_is_rejected() {
    assert_blank_uri_rejected("measure_uri");
}

#[test]
fn blank_and_whitespace_only_uploader_uri_is_rejected() {
    assert_blank_uri_rejected("uploader_uri");
}

/// Every blank/whitespace-only value of one URI column must be refused by the
/// required-text CHECK, while the other two columns stay valid.
fn assert_blank_uri_rejected(column: &str) {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let (event_uri, measure_uri, uploader_uri) = match column {
            "event_uri" => (blank, "baseline-measure-uri", "baseline-uploader-uri"),
            "measure_uri" => ("baseline-event-uri", blank, "baseline-uploader-uri"),
            _ => ("baseline-event-uri", "baseline-measure-uri", blank),
        };
        let result = insert_mapping_row(
            &pool,
            None,
            pair.platform_id,
            pair.measure_id,
            event_uri,
            measure_uri,
            uploader_uri,
            true,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank/whitespace-only {column} variant {index} ({blank:?}) must be \
             rejected by the required-text CHECK, got {result:?}"
        );
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        0,
        "no rejected mapping may have been stored"
    );
}

#[test]
fn mapping_id_is_generated_when_omitted_and_honoured_when_supplied() {
    let (_guard, pool) = setup_registry_db();

    let generated_pair = fixture_registered_pair(&pool, "generated");
    insert_mapping_row(
        &pool,
        None,
        generated_pair.platform_id,
        generated_pair.measure_id,
        "fixture-event-uri",
        "fixture-measure-uri",
        "fixture-uploader-uri",
        true,
    )
    .expect("Failed to insert the defaulted OPERAS mapping");

    let generated = only_mapping(&pool);
    assert_ne!(
        generated.mapping_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a mapping_id"
    );

    let explicit_pair = fixture_registered_pair(&pool, "explicit");
    let explicit_id = Uuid::new_v4();
    insert_mapping_row(
        &pool,
        Some(explicit_id),
        explicit_pair.platform_id,
        explicit_pair.measure_id,
        "fixture-event-uri",
        "fixture-measure-uri",
        "fixture-uploader-uri",
        false,
    )
    .expect("Failed to insert the explicitly identified OPERAS mapping");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let stored: Vec<Uuid> = metric_operas_mapping::table
        .filter(metric_operas_mapping::mapping_id.eq(explicit_id))
        .select(metric_operas_mapping::mapping_id)
        .load(&mut connection)
        .expect("Failed to load the explicitly identified mapping");
    assert_eq!(
        stored,
        vec![explicit_id],
        "an explicitly supplied mapping_id must be stored as given"
    );
    assert_ne!(
        generated.mapping_id, explicit_id,
        "the generated identity must be independent of the explicit one"
    );
}

#[test]
fn operas_mapping_not_null_columns_are_enforced() {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Every column is required. platform_id, measure_id, the three URI columns
    // and enabled carry no default at all, so omitting any of them must fail
    // rather than silently resolve to an invented value; mapping_id has the
    // repository-standard UUID default, so it is probed with an explicit NULL.
    let statements = [
        (
            "mapping_id",
            "INSERT INTO metric_operas_mapping \
                 (mapping_id, platform_id, measure_id, event_uri, measure_uri, \
                  uploader_uri, enabled) \
             VALUES (NULL, $1, $2, 'e', 'm', 'u', TRUE)",
        ),
        (
            "platform_id",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, measure_uri, uploader_uri, enabled) \
             VALUES (NULL, $2, 'e', 'm', 'u', TRUE)",
        ),
        (
            "measure_id",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, measure_uri, uploader_uri, enabled) \
             VALUES ($1, NULL, 'e', 'm', 'u', TRUE)",
        ),
        (
            "event_uri",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, measure_uri, uploader_uri, enabled) \
             VALUES ($1, $2, 'm', 'u', TRUE)",
        ),
        (
            "measure_uri",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, uploader_uri, enabled) \
             VALUES ($1, $2, 'e', 'u', TRUE)",
        ),
        (
            "uploader_uri",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, measure_uri, enabled) \
             VALUES ($1, $2, 'e', 'm', TRUE)",
        ),
        (
            "enabled",
            "INSERT INTO metric_operas_mapping \
                 (platform_id, measure_id, event_uri, measure_uri, uploader_uri) \
             VALUES ($1, $2, 'e', 'm', 'u')",
        ),
    ];

    for (label, statement) in statements {
        let result = sql_query(statement)
            .bind::<diesel::sql_types::Uuid, _>(pair.platform_id)
            .bind::<diesel::sql_types::Uuid, _>(pair.measure_id)
            .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "{label} must be NOT NULL with no invented default, got {result:?}"
        );
    }
}

#[test]
fn an_unknown_platform_measure_pair_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");

    for (label, platform_id, measure_id) in [
        ("an unknown platform", Uuid::new_v4(), pair.measure_id),
        ("an unknown measure", pair.platform_id, Uuid::new_v4()),
        ("both unknown", Uuid::new_v4(), Uuid::new_v4()),
    ] {
        let result = insert_mapping_row(
            &pool,
            None,
            platform_id,
            measure_id,
            "fixture-event-uri",
            "fixture-measure-uri",
            "fixture-uploader-uri",
            true,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "{label} must be rejected by the composite registry foreign key, \
             got {result:?}"
        );
    }
}

#[test]
fn a_real_platform_and_real_measure_not_registered_together_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let first = fixture_registered_pair(&pool, "a");
    let second = fixture_registered_pair(&pool, "b");

    // Every id here exists and is individually valid, but the crossed
    // combinations are not registered pairs in metric_platform_measure. Only
    // the composite foreign key catches this; two independent single-column
    // keys would admit both.
    for (platform_id, measure_id) in [
        (first.platform_id, second.measure_id),
        (second.platform_id, first.measure_id),
    ] {
        let result = insert_mapping_row(
            &pool,
            None,
            platform_id,
            measure_id,
            "fixture-event-uri",
            "fixture-measure-uri",
            "fixture-uploader-uri",
            true,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "a mapping must not name a real platform and a real measure that \
             are not registered together, got {result:?}"
        );
    }

    // The genuinely registered pairs remain acceptable.
    for (label, pair) in [("first", &first), ("second", &second)] {
        insert_mapping_row(
            &pool,
            None,
            pair.platform_id,
            pair.measure_id,
            "fixture-event-uri",
            "fixture-measure-uri",
            "fixture-uploader-uri",
            true,
        )
        .unwrap_or_else(|error| panic!("the registered {label} pair must be accepted: {error:?}"));
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        2
    );
}

#[test]
fn at_most_one_mapping_is_permitted_per_registered_pair() {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");

    insert_mapping_row(
        &pool,
        None,
        pair.platform_id,
        pair.measure_id,
        "fixture-event-uri",
        "fixture-measure-uri",
        "fixture-uploader-uri",
        true,
    )
    .expect("the first mapping for a registered pair must be accepted");

    // A different mapping_id, different URIs and a different enabled state
    // still name the same pair, so the canonical mapping stays unambiguous.
    let duplicate = insert_mapping_row(
        &pool,
        Some(Uuid::new_v4()),
        pair.platform_id,
        pair.measure_id,
        "other-event-uri",
        "other-measure-uri",
        "other-uploader-uri",
        false,
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a second OPERAS mapping for the same registered platform/measure pair \
         must be rejected so enabled-state and later mapping_id selection stay \
         unambiguous, got {duplicate:?}"
    );

    // Uniqueness is per pair: another registered pair gets its own mapping.
    let other = fixture_registered_pair(&pool, "b");
    insert_mapping_row(
        &pool,
        None,
        other.platform_id,
        other.measure_id,
        "fixture-event-uri",
        "fixture-measure-uri",
        "fixture-uploader-uri",
        true,
    )
    .expect("a mapping for a different registered pair must be accepted");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        2
    );
}

#[test]
fn deleting_a_referenced_platform_measure_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let pair = fixture_registered_pair(&pool, "a");
    insert_mapping_row(
        &pool,
        None,
        pair.platform_id,
        pair.measure_id,
        "fixture-event-uri",
        "fixture-measure-uri",
        "fixture-uploader-uri",
        true,
    )
    .expect("the referencing mapping must be accepted");

    let result = delete_row(
        &pool,
        "metric_platform_measure",
        "platform_measure_id",
        pair.platform_measure_id,
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced metric_platform_measure row must be restricted, \
         not cascade away OPERAS mapping configuration, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        1,
        "the mapping row must survive the restricted deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_platform_measure)"),
        1,
        "the referenced registry pair must survive the restricted deletion"
    );
}

#[test]
fn metric_operas_mapping_has_exactly_the_approved_columns() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct Column {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let columns: Vec<(String, String, String)> = sql_query(
        "SELECT column_name::text, data_type::text, is_nullable::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_operas_mapping' \
         ORDER BY ordinal_position",
    )
    .load::<Column>(&mut connection)
    .expect("Failed to read the metric_operas_mapping columns")
    .into_iter()
    .map(|column| (column.column_name, column.data_type, column.is_nullable))
    .collect();
    let observed: Vec<(&str, &str, &str)> = columns
        .iter()
        .map(|(name, data_type, nullable)| (name.as_str(), data_type.as_str(), nullable.as_str()))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("mapping_id", "uuid", "NO"),
            ("platform_id", "uuid", "NO"),
            ("measure_id", "uuid", "NO"),
            ("event_uri", "text", "NO"),
            ("measure_uri", "text", "NO"),
            ("uploader_uri", "text", "NO"),
            ("enabled", "boolean", "NO"),
        ],
        "metric_operas_mapping must carry exactly the seven approved columns"
    );

    // enabled has no database default; mapping_id has exactly the
    // repository-standard Metrics UUID default and nothing else does.
    #[derive(diesel::QueryableByName)]
    struct Default_ {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_default: String,
    }
    let defaults: Vec<(String, String)> = sql_query(
        "SELECT column_name::text, column_default::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_operas_mapping' \
           AND column_default IS NOT NULL \
         ORDER BY ordinal_position",
    )
    .load::<Default_>(&mut connection)
    .expect("Failed to read the metric_operas_mapping defaults")
    .into_iter()
    .map(|row| (row.column_name, row.column_default))
    .collect();
    assert_eq!(
        defaults,
        vec![("mapping_id".to_string(), "uuid_generate_v4()".to_string())],
        "only mapping_id may carry a default, and it must be the \
         repository-standard Metrics UUID default; in particular enabled must \
         have no invented default"
    );
}

#[test]
fn metric_operas_mapping_has_exactly_the_required_text_checks() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed: three required-text checks and nothing
    // else. In particular there is no URI scheme, host, parser, normalization
    // or length rule, and no cross-column rule involving `enabled`.
    assert_eq!(
        check_constraint_names(&pool, "metric_operas_mapping"),
        vec![
            "metric_operas_mapping_event_uri_check",
            "metric_operas_mapping_measure_uri_check",
            "metric_operas_mapping_uploader_uri_check",
        ],
        "metric_operas_mapping must carry exactly the three required-text CHECKs"
    );

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let definitions: Vec<String> = sql_query(
        "SELECT pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = 'public.metric_operas_mapping'::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read the CHECK definitions")
    .into_iter()
    .map(|row| row.definition)
    .collect();
    for definition in &definitions {
        assert!(
            definition.contains("[^[:space:]]"),
            "every URI CHECK must be the existing Metrics required-text idiom \
             and nothing stronger: {definition}"
        );
    }
}

#[test]
fn metric_operas_mapping_has_exactly_the_authorized_non_cascading_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_operas_mapping");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_operas_mapping_platform_id_measure_id_fkey"],
        "metric_operas_mapping must carry exactly one foreign key: no \
         redundant single-column platform or measure key may exist alongside \
         the composite registry key"
    );
    let (name, definition) = &keys[0];
    assert!(
        definition.contains("(platform_id, measure_id)")
            && definition.contains("metric_platform_measure(platform_id, measure_id)"),
        "the mapping key must be the composite registry-pair shape: {definition}"
    );
    assert!(
        !definition.contains("ON DELETE"),
        "{name} must stay non-cascading and use the default restricting \
         behaviour: {definition}"
    );
}

#[test]
fn metric_operas_mapping_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_operas_mapping"),
        vec![
            "metric_operas_mapping_pkey",
            "metric_operas_mapping_platform_id_measure_id_key",
        ],
        "metric_operas_mapping must carry exactly its primary key and the \
         one-mapping-per-pair uniqueness index; no speculative index on \
         enabled, on a URI column or for a future export access path may exist \
         before the WP9 export query and its query-plan evidence are approved"
    );
    assert!(
        index_definition(&pool, "metric_operas_mapping", "metric_operas_mapping_pkey")
            .contains("(mapping_id)"),
        "the primary key must be on mapping_id"
    );
    let unique = index_definition(
        &pool,
        "metric_operas_mapping",
        "metric_operas_mapping_platform_id_measure_id_key",
    );
    assert!(
        unique.contains("UNIQUE") && unique.contains("(platform_id, measure_id)"),
        "the uniqueness index must be unique on the registry pair: {unique}"
    );
}

#[test]
fn no_operas_ledger_reconciliation_or_delivery_object_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // The OPERAS export/import ledgers and the reconciliation tables remain
    // approved future architecture and must not exist yet.
    for table in DEFERRED_OPERAS_TABLES {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_class \
                      WHERE relnamespace = 'public'::regnamespace \
                        AND relkind = 'r' AND relname = '{table}')"
                ),
            ),
            0,
            "MET-WP1-08 must not create the deferred ledger table {table}"
        );
    }

    // No delivery/claim/audit column, and no duplicated registry flag, was
    // smuggled onto the mapping.
    for column in DEFERRED_MAPPING_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_operas_mapping' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-08 must not add the deferred mapping column {column}"
        );
    }

    // direct_collection stays authoritative on the registry, exactly once.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND column_name = 'direct_collection')",
        ),
        1,
        "direct_collection must remain solely on metric_platform_measure"
    );

    // No OPERAS enum vocabulary was created. `typtype = 'e'` restricts the
    // count to enums, because PostgreSQL always creates an implicit composite
    // type named after the table itself.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typtype = 'e' \
                AND (typname LIKE 'metric_operas%' OR typname LIKE 'metric_reconciliation%'))",
        ),
        0,
        "MET-WP1-08 must create no OPERAS or reconciliation enum type: the \
         export/import/reconciliation status vocabularies are deliberately \
         undefined at this stage"
    );

    // No trigger or stored procedure acts on the mapping.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'public.metric_operas_mapping'::regclass \
                AND NOT tgisinternal)",
        ),
        0,
        "MET-WP1-08 must install no trigger on metric_operas_mapping"
    );
}

#[test]
fn reverting_through_the_operas_mapping_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    let count_objects = |connection: &mut PgConnection, query: &str| -> i64 {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
            .get_result(connection)
            .expect("Failed to count schema objects")
    };

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_operas_mapping')",
        ),
        1,
        "the OPERAS mapping table must exist before reverting"
    );

    revert_through_operas_mapping_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_operas_mapping')",
        ),
        0,
        "the downgrade must drop the MET-WP1-08 table"
    );

    // Every predecessor Metrics slice survives, including the MET-WP1-01
    // registry this table references and the supporting pair unique key the
    // composite foreign key depends on.
    assert_eq!(
        count_objects(
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
                                'metric_rollup_delta'))",
        ),
        14,
        "the downgrade must leave the MET-WP1-01..07 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_platform_measure_platform_id_measure_id_key')",
        ),
        1,
        "the downgrade must not drop the MET-WP1-01 supporting unique key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('work', 'publication', 'institution', 'publisher'))",
        ),
        4,
        "the downgrade must not touch the bibliographic schema"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2,
        "the downgrade must leave the MET-WP1-01 measure seeds in place"
    );

    // Reapplication recreates the empty table with exactly its two indexes.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the OPERAS mapping migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_operas_mapping)"
        ),
        0,
        "reapplication must seed no OPERAS mapping row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_operas_mapping')",
        ),
        2,
        "reapplication must restore exactly the two required indexes"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_operas_mapping_platform_id_measure_id_fkey')",
        ),
        1,
        "reapplication must restore the composite registry foreign key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_operas_mapping'::regclass AND contype = 'c')",
        ),
        3,
        "reapplication must restore the three required-text CHECKs"
    );
}
