//! Focused `MET-WP1-04` database tests for `metric_record_revision`: the
//! closed status enum, the approved field/default contract, positive and
//! per-record-unique revision numbers, signed values, same-record supersedes
//! integrity, the single-`CURRENT` partial unique index and the targeted
//! revert/reapply of the record-schema migration.
//!
//! The canonical fixtures are the `pub(crate)` helpers defined by
//! `metric_record/tests.rs`, consumed as-is.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! implements no revision state machine, no retraction command, no
//! managed-source revision authorization and no rollup-delta generation, and
//! nothing here pretends otherwise.

use std::str::FromStr;

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::{MetricRecordRevision, MetricRecordRevisionStatus};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{
    delete_row, foreign_keys, index_definition, index_names, insert_record_fixture,
    insert_record_row, revert_through_record_schema_migration, RecordFixture,
};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_record_revision;

const REVISION_STATUSES: [(MetricRecordRevisionStatus, &str); 3] = [
    (MetricRecordRevisionStatus::Current, "CURRENT"),
    (MetricRecordRevisionStatus::Superseded, "SUPERSEDED"),
    (MetricRecordRevisionStatus::Retracted, "RETRACTED"),
];

/// Insert one canonical record and return its id alongside the fixture.
pub(crate) fn fixture_record(pool: &PgPool, identity_hash: &str) -> (RecordFixture, Uuid) {
    let fixture = insert_record_fixture(pool);
    let record_id = Uuid::new_v4();
    insert_record_row(pool, &fixture, Some(record_id), identity_hash)
        .expect("Failed to insert the fixture record row");
    (fixture, record_id)
}

/// Insert a second canonical record under the same fixture entities.
pub(crate) fn insert_second_record(
    pool: &PgPool,
    fixture: &RecordFixture,
    identity_hash: &str,
) -> Uuid {
    let record_id = Uuid::new_v4();
    insert_record_row(pool, fixture, Some(record_id), identity_hash)
        .expect("Failed to insert the second fixture record row");
    record_id
}

/// Insert one revision through raw SQL so database defaults are exercised.
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_revision_row(
    pool: &PgPool,
    record_revision_id: Option<Uuid>,
    record_id: Uuid,
    revision_number: i32,
    import_id: Uuid,
    value: i64,
    content_hash: &str,
    status: &str,
    supersedes_revision_id: Option<Uuid>,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match record_revision_id {
        Some(id) => sql_query(format!(
            "INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status, supersedes_revision_id) \
             VALUES ($1, $2, $3, $4, $5, $6, '{status}', $7)"
        ))
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Integer, _>(revision_number)
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .bind::<diesel::sql_types::BigInt, _>(value)
        .bind::<diesel::sql_types::Text, _>(content_hash)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(supersedes_revision_id)
        .execute(&mut connection),
        None => sql_query(format!(
            "INSERT INTO metric_record_revision \
                 (record_id, revision_number, import_id, value, content_hash, status, \
                  supersedes_revision_id) \
             VALUES ($1, $2, $3, $4, $5, '{status}', $6)"
        ))
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Integer, _>(revision_number)
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .bind::<diesel::sql_types::BigInt, _>(value)
        .bind::<diesel::sql_types::Text, _>(content_hash)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(supersedes_revision_id)
        .execute(&mut connection),
    }
}

#[test]
fn revision_status_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_record_revision_status"),
        vec!["CURRENT", "SUPERSEDED", "RETRACTED"],
        "metric_record_revision_status must carry exactly the three approved labels"
    );
}

#[test]
fn revision_status_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in REVISION_STATUSES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(
            MetricRecordRevisionStatus::from_str(label).unwrap(),
            variant
        );
    }
    assert!(MetricRecordRevisionStatus::from_str("OTHER").is_err());
    assert!(MetricRecordRevisionStatus::from_str("WITHDRAWN").is_err());
    assert!(MetricRecordRevisionStatus::from_str("current").is_err());
}

#[test]
fn every_revision_status_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in REVISION_STATUSES {
        assert_db_enum_roundtrip::<
            MetricRecordRevisionStatus,
            crate::schema::sql_types::MetricRecordRevisionStatus,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_record_revision_status"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_revision_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        0,
        "MET-WP1-04 must not seed any metric_record_revision row"
    );
}

#[test]
fn metric_record_revision_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let first_id: Uuid = diesel::insert_into(metric_record_revision::table)
        .values((
            metric_record_revision::record_id.eq(record_id),
            metric_record_revision::revision_number.eq(1),
            metric_record_revision::import_id.eq(fixture.import_id),
            metric_record_revision::value.eq(1_200_i64),
            metric_record_revision::content_hash.eq("content-1"),
            metric_record_revision::status.eq(MetricRecordRevisionStatus::Superseded),
        ))
        .returning(metric_record_revision::record_revision_id)
        .get_result(&mut connection)
        .expect("Failed to insert the first revision row");

    // A signed negative value must remain representable: sales measures may
    // report refunds or returns as negative net units, and measure-specific
    // validation belongs to WP2 rather than to a blanket schema rule.
    let second_id: Uuid = diesel::insert_into(metric_record_revision::table)
        .values((
            metric_record_revision::record_id.eq(record_id),
            metric_record_revision::revision_number.eq(2),
            metric_record_revision::import_id.eq(fixture.import_id),
            metric_record_revision::value.eq(-45_i64),
            metric_record_revision::content_hash.eq("content-2"),
            metric_record_revision::status.eq(MetricRecordRevisionStatus::Current),
            metric_record_revision::supersedes_revision_id.eq(first_id),
        ))
        .returning(metric_record_revision::record_revision_id)
        .get_result(&mut connection)
        .expect("Failed to insert the superseding revision row");

    let loaded: MetricRecordRevision = metric_record_revision::table
        .filter(metric_record_revision::record_revision_id.eq(second_id))
        .first(&mut connection)
        .expect("Failed to load the superseding revision row");
    assert_eq!(loaded.record_revision_id, second_id);
    assert_eq!(loaded.record_id, record_id);
    assert_eq!(loaded.revision_number, 2);
    assert_eq!(loaded.import_id, fixture.import_id);
    assert_eq!(loaded.value, -45);
    assert_eq!(loaded.content_hash, "content-2");
    assert_eq!(loaded.status, MetricRecordRevisionStatus::Current);
    assert_eq!(loaded.supersedes_revision_id, Some(first_id));
    assert!(loaded.created_at > Timestamp::default());

    let first: MetricRecordRevision = metric_record_revision::table
        .filter(metric_record_revision::record_revision_id.eq(first_id))
        .first(&mut connection)
        .expect("Failed to load the first revision row");
    assert_eq!(first.value, 1_200);
    assert_eq!(first.status, MetricRecordRevisionStatus::Superseded);
    assert_eq!(
        first.supersedes_revision_id, None,
        "an initial revision needs no predecessor"
    );
}

#[test]
fn revision_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        0,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("Failed to insert the defaulted revision row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRecordRevision = metric_record_revision::table
        .first(&mut connection)
        .expect("Failed to load the defaulted revision row");
    assert_ne!(
        loaded.record_revision_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a record_revision_id"
    );
    assert_eq!(loaded.supersedes_revision_id, None);
    assert_eq!(loaded.value, 0);
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
}

#[test]
fn signed_revision_values_remain_representable() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    // There is deliberately no blanket value >= 0 constraint.
    for (number, value) in [(1_i32, i64::MIN), (2, -1), (3, 0), (4, i64::MAX)] {
        insert_revision_row(
            &pool,
            None,
            record_id,
            number,
            fixture.import_id,
            value,
            &format!("content-{number}"),
            "SUPERSEDED",
            None,
        )
        .unwrap_or_else(|error| {
            panic!("a signed BIGINT value {value} must be accepted: {error:?}")
        });
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let values: Vec<i64> = metric_record_revision::table
        .order(metric_record_revision::revision_number)
        .select(metric_record_revision::value)
        .load(&mut connection)
        .expect("Failed to load the stored revision values");
    assert_eq!(values, vec![i64::MIN, -1, 0, i64::MAX]);
}

#[test]
fn blank_revision_content_hash_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_revision_row(
            &pool,
            None,
            record_id,
            1,
            fixture.import_id,
            10,
            blank,
            "CURRENT",
            None,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a blank content hash ({blank:?}) must be rejected by a check constraint, \
             got {result:?}"
        );
    }
}

#[test]
fn non_positive_revision_numbers_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    for number in [0_i32, -1, i32::MIN] {
        let result = insert_revision_row(
            &pool,
            None,
            record_id,
            number,
            fixture.import_id,
            10,
            "content-1",
            "CURRENT",
            None,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "revision_number {number} must be rejected by a check constraint, got {result:?}"
        );
    }
    insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("revision_number 1 must be accepted");
}

#[test]
fn revision_numbers_are_unique_per_record_but_not_globally() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");

    insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("the first revision must be accepted");

    let duplicate = insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        20,
        "content-2",
        "SUPERSEDED",
        None,
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a repeated (record_id, revision_number) must be rejected, got {duplicate:?}"
    );

    insert_revision_row(
        &pool,
        None,
        other_record_id,
        1,
        fixture.import_id,
        30,
        "content-3",
        "CURRENT",
        None,
    )
    .expect("revision numbering is per record, not global");
}

#[test]
fn at_most_one_current_revision_is_permitted_per_record() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");

    insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("the first CURRENT revision must be accepted");

    let second_current = insert_revision_row(
        &pool,
        None,
        record_id,
        2,
        fixture.import_id,
        20,
        "content-2",
        "CURRENT",
        None,
    );
    assert!(
        matches!(
            second_current,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a second CURRENT revision for one record must be rejected, got {second_current:?}"
    );

    // The partial index constrains CURRENT only: full history is retained, so
    // any number of SUPERSEDED and RETRACTED revisions may coexist.
    for (number, status) in [
        (2_i32, "SUPERSEDED"),
        (3, "SUPERSEDED"),
        (4, "RETRACTED"),
        (5, "RETRACTED"),
    ] {
        insert_revision_row(
            &pool,
            None,
            record_id,
            number,
            fixture.import_id,
            i64::from(number),
            &format!("content-{number}"),
            status,
            None,
        )
        .unwrap_or_else(|error| panic!("a {status} revision must be accepted: {error:?}"));
    }

    insert_revision_row(
        &pool,
        None,
        other_record_id,
        1,
        fixture.import_id,
        10,
        "content-other",
        "CURRENT",
        None,
    )
    .expect("the single-CURRENT rule is per record, not global");
}

#[test]
fn a_supersedes_reference_to_another_records_revision_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");

    let foreign_revision_id = Uuid::new_v4();
    insert_revision_row(
        &pool,
        Some(foreign_revision_id),
        other_record_id,
        1,
        fixture.import_id,
        10,
        "content-other",
        "CURRENT",
        None,
    )
    .expect("the other record's revision must be accepted");

    let result = insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        20,
        "content-1",
        "SUPERSEDED",
        Some(foreign_revision_id),
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a revision must not supersede a revision owned by another record, got {result:?}"
    );

    // An unknown revision id must fail the same way.
    let unknown = insert_revision_row(
        &pool,
        None,
        record_id,
        2,
        fixture.import_id,
        20,
        "content-2",
        "SUPERSEDED",
        Some(Uuid::new_v4()),
    );
    assert!(
        matches!(
            unknown,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown superseded revision must be rejected, got {unknown:?}"
    );

    // The same-record case remains valid.
    let own_revision_id = Uuid::new_v4();
    insert_revision_row(
        &pool,
        Some(own_revision_id),
        record_id,
        3,
        fixture.import_id,
        30,
        "content-3",
        "SUPERSEDED",
        None,
    )
    .expect("an initial revision for this record must be accepted");
    insert_revision_row(
        &pool,
        None,
        record_id,
        4,
        fixture.import_id,
        40,
        "content-4",
        "CURRENT",
        Some(own_revision_id),
    )
    .expect("superseding a revision of the same record must be accepted");
}

#[test]
fn a_current_revision_pointer_to_another_records_revision_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");

    let own_revision_id = Uuid::new_v4();
    let foreign_revision_id = Uuid::new_v4();
    insert_revision_row(
        &pool,
        Some(own_revision_id),
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("this record's revision must be accepted");
    insert_revision_row(
        &pool,
        Some(foreign_revision_id),
        other_record_id,
        1,
        fixture.import_id,
        20,
        "content-other",
        "CURRENT",
        None,
    )
    .expect("the other record's revision must be accepted");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let update_pointer = |connection: &mut PgConnection, revision_id: Uuid| {
        sql_query("UPDATE metric_record SET current_revision_id = $1 WHERE record_id = $2")
            .bind::<diesel::sql_types::Uuid, _>(revision_id)
            .bind::<diesel::sql_types::Uuid, _>(record_id)
            .execute(connection)
    };

    update_pointer(&mut connection, own_revision_id)
        .expect("pointing at this record's own revision must be accepted");

    let cross_record = update_pointer(&mut connection, foreign_revision_id);
    assert!(
        matches!(
            cross_record,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a record must not name a revision owned by another record, got {cross_record:?}"
    );

    let unknown = update_pointer(&mut connection, Uuid::new_v4());
    assert!(
        matches!(
            unknown,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a record must not name a revision that does not exist, got {unknown:?}"
    );
}

#[test]
fn the_current_revision_pointer_is_not_tied_to_revision_status_by_this_slice() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let retracted_id = Uuid::new_v4();
    insert_revision_row(
        &pool,
        Some(retracted_id),
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "RETRACTED",
        None,
    )
    .expect("a RETRACTED revision must be storable");

    let mut connection = pool.get().expect("Failed to get DB connection");
    // WP2 owns the transaction that keeps status and the record pointer
    // consistent. This slice deliberately installs no trigger or constraint
    // enforcing that relationship, so a pointer at a non-CURRENT revision
    // must still be accepted here. Nothing in WP1 may pretend otherwise.
    sql_query("UPDATE metric_record SET current_revision_id = $1 WHERE record_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(retracted_id)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .execute(&mut connection)
        .expect("no WP1 constraint ties current_revision_id to revision status");
}

#[test]
fn invalid_revision_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");

    let unknown_record = insert_revision_row(
        &pool,
        None,
        Uuid::new_v4(),
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    );
    assert!(
        matches!(
            unknown_record,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown record must be rejected, got {unknown_record:?}"
    );

    let unknown_import = insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        Uuid::new_v4(),
        10,
        "content-1",
        "CURRENT",
        None,
    );
    assert!(
        matches!(
            unknown_import,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown import must be rejected, got {unknown_import:?}"
    );
}

#[test]
fn deleting_a_referenced_record_or_import_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    insert_revision_row(
        &pool,
        None,
        record_id,
        1,
        fixture.import_id,
        10,
        "content-1",
        "CURRENT",
        None,
    )
    .expect("the referencing revision must be accepted");

    for (table, id_column, id) in [
        ("metric_record", "record_id", record_id),
        ("metric_import", "import_id", fixture.import_id),
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
             canonical history, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1,
        "the revision row must survive the restricted deletions"
    );
}

#[test]
fn metric_record_revision_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // The set is closed: in particular no blanket value >= 0 constraint may
    // exist, because sales measures may report signed net units.
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(&pool, "metric_record_revision"),
        vec![
            "metric_record_revision_content_hash_check",
            "metric_record_revision_revision_number_check",
        ],
        "metric_record_revision must carry exactly the two authorized CHECK constraints"
    );
}

#[test]
fn metric_record_revision_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_record_revision");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_record_revision_import_id_fkey",
            "metric_record_revision_record_id_fkey",
            "metric_record_revision_supersedes_revision_id_fkey",
        ],
        "metric_record_revision must carry exactly the three authorized foreign keys"
    );
    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour: {definition}"
        );
    }
    assert!(
        keys.iter().any(|(name, definition)| name
            == "metric_record_revision_supersedes_revision_id_fkey"
            && definition.contains("(record_id, supersedes_revision_id)")
            && definition.contains("metric_record_revision(record_id, record_revision_id)")),
        "the supersedes key must be the self-referential same-record composite \
         shape: {keys:?}"
    );
}

#[test]
fn metric_record_revision_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_record_revision"),
        vec![
            "metric_record_revision_pkey",
            "metric_record_revision_record_id_current_idx",
            "metric_record_revision_record_id_record_revision_id_key",
            "metric_record_revision_record_id_revision_number_key",
        ],
        "metric_record_revision must carry exactly its primary key, the \
         per-record revision-number unique key, the same-record integrity \
         unique key and the partial current-revision index"
    );
    let current = index_definition(
        &pool,
        "metric_record_revision",
        "metric_record_revision_record_id_current_idx",
    );
    assert!(
        current.contains("UNIQUE")
            && current.contains("(record_id)")
            && current.contains("WHERE (status = 'CURRENT'"),
        "the current-revision index must be unique on record_id and partial on \
         status = 'CURRENT': {current}"
    );
    assert!(
        index_definition(
            &pool,
            "metric_record_revision",
            "metric_record_revision_record_id_revision_number_key",
        )
        .contains("(record_id, revision_number)"),
        "the per-record revision-number key must be preserved"
    );
}

#[test]
fn reverting_through_the_record_schema_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    let count_objects = |connection: &mut PgConnection, query: &str| -> i64 {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
            .get_result(connection)
            .expect("Failed to count schema objects")
    };
    let measure_seeds = |connection: &mut PgConnection| -> Vec<String> {
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
    let seeds_before = measure_seeds(&mut connection);
    assert_eq!(seeds_before.len(), 2, "both measure seeds must exist");

    revert_through_record_schema_migration(&mut connection);

    // The MET-WP1-04 tables and both enum types are gone. The down migration
    // must have dropped the circular current-revision constraint explicitly
    // before dropping either participating table.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_record', 'metric_record_revision', \
                                'metric_record_provenance'))",
        ),
        0,
        "the downgrade must drop all three MET-WP1-04 tables"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname IN ('metric_record_revision_status', \
                                'metric_record_provenance_classification'))",
        ),
        0,
        "the downgrade must drop both MET-WP1-04 enum types"
    );

    // ...while every earlier Metrics slice and the shared reporting-grain enum
    // survive, together with the exact measure seeds.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint', \
                                'metric_import', 'metric_import_error'))",
        ),
        8,
        "the downgrade must leave the MET-WP1-01/02/03 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname = 'metric_reporting_grain')",
        ),
        1,
        "the downgrade must not drop the MET-WP1-01 reporting-grain enum"
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
        measure_seeds(&mut connection),
        seeds_before,
        "the downgrade must leave the measure seeds byte-identical"
    );

    // Reapplication recreates both enums and all three empty tables.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the record-schema migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname IN ('metric_record_revision_status', \
                                'metric_record_provenance_classification'))",
        ),
        2,
        "reapplication must recreate both enum types cleanly"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "((SELECT COUNT(*) FROM metric_record) \
              + (SELECT COUNT(*) FROM metric_record_revision) \
              + (SELECT COUNT(*) FROM metric_record_provenance))",
        ),
        0,
        "reapplication must seed no record, revision or provenance row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_record_current_revision_id_fkey')",
        ),
        1,
        "reapplication must restore the circular current-revision constraint"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "reapplication must leave the measure seeds byte-identical"
    );
}
