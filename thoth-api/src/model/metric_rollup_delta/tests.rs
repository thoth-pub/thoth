//! Focused `MET-WP1-07` database tests for `metric_rollup_delta`: the approved
//! field/default contract, signed positive/zero/negative delta values,
//! unconstrained `status` text, nullable `applied_at`, same-record/revision
//! composite referential integrity, one-delta-per-revision uniqueness,
//! restricted (non-cascading) deletion, the exact index inventory and the
//! targeted revert/reapply of the rollup-delta migration.
//!
//! The canonical record/revision fixtures are the existing `pub(crate)`
//! helpers defined by `metric_record/tests.rs` and
//! `metric_record_revision/tests.rs`, consumed as-is: no other model's test
//! module is widened, because this task's write budget contains only this
//! file.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no delta at runtime, applies none, and implements no claiming,
//! lease, retry, stale-claim recovery, rebuild, generation, watermark or
//! rollup projection, and nothing here pretends otherwise.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricRollupDelta;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{
    delete_row, foreign_keys, index_definition, index_names, RecordFixture,
};
use crate::model::metric_record_revision::tests::{
    fixture_record, insert_revision_row, insert_second_record,
};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_rollup_delta;

/// The Diesel migration version of `thoth-api/migrations/20260903_v1.9.0`.
const MET_WP1_07_MIGRATION_VERSION: &str = "20260903";

/// The four rebuildable work-level rollup projection tables named by the
/// approved design. They remain approved *future* architecture: MET-WP1-07
/// must not create any of them.
const DEFERRED_PROJECTION_TABLES: [&str; 4] = [
    "metric_rollup_work_country_month",
    "metric_rollup_work_day",
    "metric_rollup_work_institution_month",
    "metric_rollup_work_month",
];

/// Column names that would betray a claim, lease, retry, failure-detail,
/// rebuild-generation or watermark protocol having been smuggled into this
/// persistence-only slice.
const DEFERRED_CLAIM_COLUMNS: [&str; 10] = [
    "attempt_count",
    "claim_token",
    "claimed_at",
    "claimed_by",
    "error_detail",
    "generation",
    "lease_expires_at",
    "retry_count",
    "updated_at",
    "watermark",
];

/// Revert migrations until the `MET-WP1-07` rollup-delta migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_record_schema_migration` and
/// its predecessors: a bare `revert_last_migration` would only mean "the
/// rollup-delta migration" while it happens to be the newest applied
/// migration. Reverting down to and including the target keeps the meaning
/// under any later migration order, and no future migration name is assumed
/// or hard-coded.
fn revert_through_rollup_delta_migration(connection: &mut PgConnection) {
    let rollup_delta_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_07_MIGRATION_VERSION);
    assert!(
        rollup_delta_migration_applied,
        "the MET-WP1-07 rollup-delta migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_07_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one canonical record plus one canonical revision of it, and return
/// the fixture, the record id and the revision id.
fn fixture_record_revision(pool: &PgPool, identity_hash: &str) -> (RecordFixture, Uuid, Uuid) {
    let (fixture, record_id) = fixture_record(pool, identity_hash);
    let revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(revision_id),
        record_id,
        1,
        fixture.import_id,
        100,
        &format!("content-{identity_hash}"),
        "CURRENT",
        None,
    )
    .expect("Failed to insert the fixture revision row");
    (fixture, record_id, revision_id)
}

/// Insert one rollup delta through raw SQL so the database defaults are
/// exercised rather than restated by a Diesel fixture.
fn insert_delta_row(
    pool: &PgPool,
    delta_id: Option<Uuid>,
    record_id: Uuid,
    revision_id: Uuid,
    delta_value: i64,
    status: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match delta_id {
        Some(delta_id) => sql_query(
            "INSERT INTO metric_rollup_delta \
                 (delta_id, record_id, revision_id, delta_value, status) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind::<diesel::sql_types::Uuid, _>(delta_id)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .bind::<diesel::sql_types::BigInt, _>(delta_value)
        .bind::<diesel::sql_types::Text, _>(status)
        .execute(&mut connection),
        None => sql_query(
            "INSERT INTO metric_rollup_delta \
                 (record_id, revision_id, delta_value, status) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .bind::<diesel::sql_types::BigInt, _>(delta_value)
        .bind::<diesel::sql_types::Text, _>(status)
        .execute(&mut connection),
    }
}

/// One additional canonical revision of an existing record.
fn insert_extra_revision(
    pool: &PgPool,
    fixture: &RecordFixture,
    record_id: Uuid,
    revision_number: i32,
) -> Uuid {
    let revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(revision_id),
        record_id,
        revision_number,
        fixture.import_id,
        i64::from(revision_number),
        &format!("content-extra-{revision_number}-{record_id}"),
        "SUPERSEDED",
        None,
    )
    .expect("Failed to insert the extra revision row");
    revision_id
}

#[test]
fn migration_seeds_no_rollup_delta_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0,
        "MET-WP1-07 must not seed any metric_rollup_delta row"
    );
}

#[test]
fn a_rollup_delta_round_trips_through_diesel_with_a_null_applied_at() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let delta_id: Uuid = diesel::insert_into(metric_rollup_delta::table)
        .values((
            metric_rollup_delta::record_id.eq(record_id),
            metric_rollup_delta::revision_id.eq(revision_id),
            metric_rollup_delta::delta_value.eq(1_200_i64),
            metric_rollup_delta::status.eq("PENDING"),
        ))
        .returning(metric_rollup_delta::delta_id)
        .get_result(&mut connection)
        .expect("Failed to insert the unapplied rollup delta");

    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .filter(metric_rollup_delta::delta_id.eq(delta_id))
        .first(&mut connection)
        .expect("Failed to load the unapplied rollup delta");
    assert_eq!(loaded.delta_id, delta_id);
    assert_eq!(loaded.record_id, record_id);
    assert_eq!(loaded.revision_id, revision_id);
    assert_eq!(loaded.delta_value, 1_200);
    assert_eq!(loaded.status, "PENDING");
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
    assert_eq!(
        loaded.applied_at, None,
        "applied_at must stay NULL until later WP4 work applies the delta"
    );
}

#[test]
fn a_rollup_delta_round_trips_through_diesel_with_a_non_null_applied_at() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let applied_at = Timestamp::parse_from_rfc3339("2026-09-03T11:22:33Z")
        .expect("Failed to parse the fixture applied_at timestamp");
    let delta_id: Uuid = diesel::insert_into(metric_rollup_delta::table)
        .values((
            metric_rollup_delta::record_id.eq(record_id),
            metric_rollup_delta::revision_id.eq(revision_id),
            metric_rollup_delta::delta_value.eq(-7_i64),
            metric_rollup_delta::status.eq("APPLIED"),
            metric_rollup_delta::applied_at.eq(applied_at),
        ))
        .returning(metric_rollup_delta::delta_id)
        .get_result(&mut connection)
        .expect("Failed to insert the applied rollup delta");

    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .filter(metric_rollup_delta::delta_id.eq(delta_id))
        .first(&mut connection)
        .expect("Failed to load the applied rollup delta");
    assert_eq!(loaded.delta_value, -7);
    assert_eq!(loaded.status, "APPLIED");
    assert_eq!(loaded.applied_at, Some(applied_at));
}

#[test]
fn rollup_delta_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    insert_delta_row(&pool, None, record_id, revision_id, 0, "PENDING")
        .expect("Failed to insert the defaulted rollup delta");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .first(&mut connection)
        .expect("Failed to load the defaulted rollup delta");
    assert_ne!(
        loaded.delta_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a delta_id"
    );
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
    assert_eq!(
        loaded.applied_at, None,
        "applied_at must have no invented default"
    );
}

#[test]
fn positive_zero_and_negative_delta_values_are_all_accepted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, first_revision_id) = fixture_record_revision(&pool, "identity-a");

    // A revision contributes the signed difference `new - old` and a
    // retraction subtracts the previously applied value, so there is
    // deliberately no blanket delta_value >= 0 constraint. The extremes prove
    // the column is a full signed BIGINT.
    let values = [1_200_i64, 0, -45, i64::MIN, i64::MAX];
    let mut revision_ids = vec![first_revision_id];
    for number in 2..=(values.len() as i32) {
        revision_ids.push(insert_extra_revision(&pool, &fixture, record_id, number));
    }
    for (revision_id, value) in revision_ids.iter().zip(values) {
        insert_delta_row(&pool, None, record_id, *revision_id, value, "PENDING").unwrap_or_else(
            |error| panic!("a signed BIGINT delta_value {value} must be accepted: {error:?}"),
        );
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut stored: Vec<i64> = metric_rollup_delta::table
        .select(metric_rollup_delta::delta_value)
        .load(&mut connection)
        .expect("Failed to load the stored delta values");
    stored.sort_unstable();
    let mut expected = values.to_vec();
    expected.sort_unstable();
    assert_eq!(stored, expected);
}

#[test]
fn arbitrary_non_empty_status_text_persists_without_a_closed_vocabulary() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, first_revision_id) = fixture_record_revision(&pool, "identity-a");

    // The approved design names `status` but defines no closed vocabulary,
    // so this foundation must accept any text. Nothing here asserts that any
    // of these strings is a real state: the WP4 claim/application
    // specification owns that decision.
    let statuses = [
        "PENDING",
        "anything at all",
        "  padded  ",
        "état-appliqué",
        "x",
    ];
    let mut revision_ids = vec![first_revision_id];
    for number in 2..=(statuses.len() as i32) {
        revision_ids.push(insert_extra_revision(&pool, &fixture, record_id, number));
    }
    for (revision_id, status) in revision_ids.iter().zip(statuses) {
        insert_delta_row(&pool, None, record_id, *revision_id, 1, status)
            .unwrap_or_else(|error| panic!("status {status:?} must be accepted: {error:?}"));
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut stored: Vec<String> = metric_rollup_delta::table
        .select(metric_rollup_delta::status)
        .load(&mut connection)
        .expect("Failed to load the stored statuses");
    stored.sort();
    let mut expected: Vec<String> = statuses.iter().map(|status| status.to_string()).collect();
    expected.sort();
    assert_eq!(stored, expected);
}

#[test]
fn an_unknown_record_or_revision_reference_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");

    for (label, delta_record_id, delta_revision_id) in [
        ("an unknown record", Uuid::new_v4(), revision_id),
        ("an unknown revision", record_id, Uuid::new_v4()),
        ("both unknown", Uuid::new_v4(), Uuid::new_v4()),
    ] {
        let result = insert_delta_row(
            &pool,
            None,
            delta_record_id,
            delta_revision_id,
            10,
            "PENDING",
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "{label} must be rejected by the composite foreign key, got {result:?}"
        );
    }
}

#[test]
fn a_revision_belonging_to_another_record_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");
    let foreign_revision_id = insert_extra_revision(&pool, &fixture, other_record_id, 1);

    // Both ids exist and both are individually valid, but they do not name the
    // same canonical pair. Only the composite key catches this.
    let result = insert_delta_row(&pool, None, record_id, foreign_revision_id, 10, "PENDING");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a delta must not pair a record with a revision owned by another \
         record, got {result:?}"
    );

    // The same-record pairings remain valid.
    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("a delta naming its own record's revision must be accepted");
    insert_delta_row(
        &pool,
        None,
        other_record_id,
        foreign_revision_id,
        20,
        "PENDING",
    )
    .expect("the other record's own delta must be accepted");
}

#[test]
fn at_most_one_delta_is_permitted_per_canonical_revision() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");

    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("the first delta for a revision must be accepted");

    let duplicate = insert_delta_row(&pool, None, record_id, revision_id, -10, "APPLIED");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a second delta for the same canonical revision must be rejected so \
         later rollup application cannot double count, got {duplicate:?}"
    );

    // Uniqueness is per revision, not per record: a second revision of the
    // same record gets its own delta.
    let second_revision_id = insert_extra_revision(&pool, &fixture, record_id, 2);
    insert_delta_row(&pool, None, record_id, second_revision_id, 5, "PENDING")
        .expect("a delta for a different revision of the same record must be accepted");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        2,
    );
}

#[test]
fn deleting_a_referenced_revision_or_record_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("the referencing delta must be accepted");

    for (table, id_column, id) in [
        ("metric_record_revision", "record_revision_id", revision_id),
        ("metric_record", "record_id", record_id),
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
            "deleting a referenced {table} must be restricted, not cascade away \
             durable delta evidence, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1,
        "the delta row must survive the restricted deletions"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1,
        "the referenced revision must survive the restricted deletion"
    );
}

#[test]
fn rollup_delta_not_null_columns_are_enforced() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    // delta_value and status carry no default, so omitting either must fail
    // rather than silently resolve to zero or an invented state.
    for (label, statement) in [
        (
            "delta_value",
            "INSERT INTO metric_rollup_delta (record_id, revision_id, status) \
             VALUES ($1, $2, 'PENDING')",
        ),
        (
            "status",
            "INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value) \
             VALUES ($1, $2, 10)",
        ),
    ] {
        let result = sql_query(statement)
            .bind::<diesel::sql_types::Uuid, _>(record_id)
            .bind::<diesel::sql_types::Uuid, _>(revision_id)
            .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "{label} must be NOT NULL with no default, got {result:?}"
        );
    }
}

#[test]
fn metric_rollup_delta_carries_no_check_constraint() {
    let (_guard, pool) = setup_registry_db();
    // The set is empty and closed. In particular there is no
    // `delta_value >= 0` rule, because retraction and correction deltas are
    // negative, and no cross-column rule tying `applied_at` to `status`,
    // because the status vocabulary is deliberately undefined at this stage.
    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_delta"),
        Vec::<String>::new(),
        "metric_rollup_delta must carry no CHECK constraint"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_authorized_non_cascading_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_rollup_delta");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_rollup_delta_record_id_revision_id_fkey"],
        "metric_rollup_delta must carry exactly one foreign key"
    );
    let (name, definition) = &keys[0];
    assert!(
        definition.contains("(record_id, revision_id)")
            && definition.contains("metric_record_revision(record_id, record_revision_id)"),
        "the delta key must be the same-record composite shape: {definition}"
    );
    assert!(
        !definition.contains("ON DELETE"),
        "{name} must stay non-cascading and use the default restricting \
         behaviour: {definition}"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_rollup_delta"),
        vec![
            "metric_rollup_delta_pkey",
            "metric_rollup_delta_revision_id_key",
        ],
        "metric_rollup_delta must carry exactly its primary key and the \
         one-delta-per-revision uniqueness index; no speculative claim index \
         may exist before the WP4 claim protocol is approved"
    );
    assert!(
        index_definition(&pool, "metric_rollup_delta", "metric_rollup_delta_pkey")
            .contains("(delta_id)"),
        "the primary key must be on delta_id"
    );
    let unique = index_definition(
        &pool,
        "metric_rollup_delta",
        "metric_rollup_delta_revision_id_key",
    );
    assert!(
        unique.contains("UNIQUE") && unique.contains("(revision_id)"),
        "the uniqueness index must be unique on revision_id alone: {unique}"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_approved_columns() {
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
         WHERE table_schema = 'public' AND table_name = 'metric_rollup_delta' \
         ORDER BY ordinal_position",
    )
    .load::<Column>(&mut connection)
    .expect("Failed to read the metric_rollup_delta columns")
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
            ("delta_id", "uuid", "NO"),
            ("record_id", "uuid", "NO"),
            ("revision_id", "uuid", "NO"),
            ("delta_value", "bigint", "NO"),
            ("status", "text", "NO"),
            ("created_at", "timestamp with time zone", "NO"),
            ("applied_at", "timestamp with time zone", "YES"),
        ],
        "metric_rollup_delta must carry exactly the seven approved columns, \
         with signed BIGINT delta_value, TEXT status and a nullable applied_at"
    );
}

#[test]
fn no_claim_rebuild_or_projection_object_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // The four rebuildable work-level projections remain approved future
    // architecture and must not exist yet.
    for table in DEFERRED_PROJECTION_TABLES {
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
            "MET-WP1-07 must not create the deferred projection table {table}"
        );
    }

    // No claim/lease/retry/rebuild/watermark column was smuggled in.
    for column in DEFERRED_CLAIM_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_rollup_delta' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-07 must not add the deferred claim/rebuild column {column}"
        );
    }

    // status stays free TEXT: no PostgreSQL enum type was created for it.
    // `typtype = 'e'` restricts the count to enums, because PostgreSQL always
    // creates an implicit composite type named after the table itself.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typtype = 'e' \
                AND typname LIKE 'metric_rollup%')",
        ),
        0,
        "MET-WP1-07 must create no rollup enum type: the status vocabulary is \
         deliberately undefined at this stage"
    );

    // No trigger or stored procedure moves a delta between states.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'public.metric_rollup_delta'::regclass \
                AND NOT tgisinternal)",
        ),
        0,
        "MET-WP1-07 must install no trigger on metric_rollup_delta"
    );
}

#[test]
fn reverting_through_the_rollup_delta_migration_removes_it_and_reapplication_restores_it() {
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
                AND relname = 'metric_rollup_delta')",
        ),
        1,
        "the rollup-delta table must exist before reverting"
    );

    revert_through_rollup_delta_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_rollup_delta')",
        ),
        0,
        "the downgrade must drop the MET-WP1-07 table"
    );

    // Every predecessor Metrics slice survives, including the MET-WP1-04
    // record/revision schema this table references and the supporting
    // same-record unique key the composite foreign key depends on.
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
                                'metric_publisher_platform_approval'))",
        ),
        13,
        "the downgrade must leave the MET-WP1-01..06 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_record_revision_record_id_record_revision_id_key')",
        ),
        1,
        "the downgrade must not drop the MET-WP1-04 supporting unique key"
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
        .expect("Failed to reapply migrations from the rollup-delta migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_rollup_delta)"
        ),
        0,
        "reapplication must seed no rollup-delta row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_rollup_delta')",
        ),
        2,
        "reapplication must restore exactly the two required indexes"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_rollup_delta_record_id_revision_id_fkey')",
        ),
        1,
        "reapplication must restore the composite foreign key"
    );
}
