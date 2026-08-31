//! Focused `MET-WP1-04` database tests for `metric_record_provenance`: the
//! closed classification enum, the approved field/default contract, the
//! intentionally nullable record link that carries rejected-row evidence, the
//! complete authorized CHECK and foreign-key inventory and the three audit
//! indexes.
//!
//! The canonical fixtures are the `pub(crate)` helpers defined by
//! `metric_record/tests.rs` and `metric_record_revision/tests.rs`, consumed
//! as-is.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! stores classifications but implements no algorithm that assigns them, and
//! nothing here pretends otherwise.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

use super::{MetricRecordProvenance, MetricRecordProvenanceClassification};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_definition, index_names};
use crate::model::metric_record_revision::tests::fixture_record;
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::Timestamp;
use crate::schema::metric_record_provenance;

const CLASSIFICATIONS: [(MetricRecordProvenanceClassification, &str); 5] = [
    (MetricRecordProvenanceClassification::Winner, "WINNER"),
    (MetricRecordProvenanceClassification::Duplicate, "DUPLICATE"),
    (MetricRecordProvenanceClassification::Revision, "REVISION"),
    (MetricRecordProvenanceClassification::Conflict, "CONFLICT"),
    (MetricRecordProvenanceClassification::Rejected, "REJECTED"),
];

/// Insert one provenance row through raw SQL so database defaults are
/// exercised rather than restated by a Diesel fixture.
fn insert_provenance_row(
    pool: &PgPool,
    record_id: Option<Uuid>,
    import_id: Uuid,
    identity_hash: &str,
    content_hash: &str,
    classification: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_record_provenance \
             (record_id, import_id, identity_hash, content_hash, classification) \
         VALUES ($1, $2, $3, $4, '{classification}')"
    ))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(record_id)
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Text, _>(identity_hash)
    .bind::<diesel::sql_types::Text, _>(content_hash)
    .execute(&mut connection)
}

#[test]
fn provenance_classification_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_record_provenance_classification"),
        vec!["WINNER", "DUPLICATE", "REVISION", "CONFLICT", "REJECTED"],
        "metric_record_provenance_classification must carry exactly the five approved labels"
    );
}

#[test]
fn provenance_classification_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in CLASSIFICATIONS {
        assert_eq!(variant.to_string(), label);
        assert_eq!(
            MetricRecordProvenanceClassification::from_str(label).unwrap(),
            variant
        );
    }
    assert!(MetricRecordProvenanceClassification::from_str("OTHER").is_err());
    assert!(MetricRecordProvenanceClassification::from_str("ACCEPTED").is_err());
    assert!(MetricRecordProvenanceClassification::from_str("winner").is_err());
}

#[test]
fn every_provenance_classification_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in CLASSIFICATIONS {
        assert_db_enum_roundtrip::<
            MetricRecordProvenanceClassification,
            crate::schema::sql_types::MetricRecordProvenanceClassification,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_record_provenance_classification"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_provenance_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        0,
        "MET-WP1-04 must not seed any metric_record_provenance row"
    );
}

#[test]
fn metric_record_provenance_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let provenance_id: Uuid = diesel::insert_into(metric_record_provenance::table)
        .values((
            metric_record_provenance::record_id.eq(record_id),
            metric_record_provenance::import_id.eq(fixture.import_id),
            metric_record_provenance::source_record_id.eq("source-row-key-42"),
            metric_record_provenance::source_row_number.eq(9_000_000_000_i64),
            metric_record_provenance::identity_hash.eq("identity-a"),
            metric_record_provenance::content_hash.eq("content-a"),
            metric_record_provenance::classification
                .eq(MetricRecordProvenanceClassification::Winner),
            metric_record_provenance::details
                .eq(json!({"normalizer": "thoth_csv/1", "notes": ["first arrival"]})),
        ))
        .returning(metric_record_provenance::record_provenance_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated provenance row");

    let loaded: MetricRecordProvenance = metric_record_provenance::table
        .filter(metric_record_provenance::record_provenance_id.eq(provenance_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated provenance row");
    assert_eq!(loaded.record_provenance_id, provenance_id);
    assert_eq!(loaded.record_id, Some(record_id));
    assert_eq!(loaded.import_id, fixture.import_id);
    assert_eq!(
        loaded.source_record_id.as_deref(),
        Some("source-row-key-42")
    );
    assert_eq!(loaded.source_row_number, Some(9_000_000_000));
    assert_eq!(loaded.identity_hash, "identity-a");
    assert_eq!(loaded.content_hash, "content-a");
    assert_eq!(
        loaded.classification,
        MetricRecordProvenanceClassification::Winner
    );
    assert_eq!(
        loaded.details,
        json!({"normalizer": "thoth_csv/1", "notes": ["first arrival"]})
    );
    assert!(loaded.received_at > Timestamp::default());
}

#[test]
fn provenance_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    insert_provenance_row(
        &pool,
        Some(record_id),
        fixture.import_id,
        "identity-a",
        "content-a",
        "WINNER",
    )
    .expect("Failed to insert the defaulted provenance row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRecordProvenance = metric_record_provenance::table
        .first(&mut connection)
        .expect("Failed to load the defaulted provenance row");
    assert_ne!(
        loaded.record_provenance_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a record_provenance_id"
    );
    assert_eq!(loaded.source_record_id, None);
    assert_eq!(loaded.source_row_number, None);
    assert_eq!(
        loaded.details,
        json!({}),
        "details must default to an empty JSON object"
    );
    assert!(
        loaded.received_at > Timestamp::default(),
        "the repository-standard current-time default must populate received_at"
    );
}

#[test]
fn every_classification_is_storable_and_reloadable() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    for (index, (_, label)) in CLASSIFICATIONS.into_iter().enumerate() {
        // A rejected or conflicting row need not resolve to a canonical
        // record, so those two are deliberately stored without one.
        let linked_record = match label {
            "REJECTED" | "CONFLICT" => None,
            _ => Some(record_id),
        };
        insert_provenance_row(
            &pool,
            linked_record,
            fixture.import_id,
            &format!("identity-{index}"),
            &format!("content-{index}"),
            label,
        )
        .unwrap_or_else(|error| panic!("classification {label} must be storable: {error:?}"));
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: Vec<MetricRecordProvenance> = metric_record_provenance::table
        .order(metric_record_provenance::identity_hash)
        .load(&mut connection)
        .expect("Failed to load the provenance rows");
    assert_eq!(loaded.len(), 5);
    assert_eq!(
        loaded
            .iter()
            .map(|row| row.classification)
            .collect::<Vec<_>>(),
        CLASSIFICATIONS
            .into_iter()
            .map(|(variant, _)| variant)
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_nullable_record_link_supports_rejected_row_evidence() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    // Provenance exists for every normalized row, including rows that produced
    // no canonical record. Nothing here may invent a record for them.
    insert_provenance_row(
        &pool,
        None,
        fixture.import_id,
        "identity-rejected",
        "content-rejected",
        "REJECTED",
    )
    .expect("rejected-row evidence must be recordable without a canonical record");
    insert_provenance_row(
        &pool,
        None,
        fixture.import_id,
        "identity-conflict",
        "content-conflict",
        "CONFLICT",
    )
    .expect("conflicting-row evidence must be recordable without a canonical record");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let unlinked: i64 = metric_record_provenance::table
        .filter(metric_record_provenance::record_id.is_null())
        .count()
        .get_result(&mut connection)
        .expect("Failed to count the unlinked provenance rows");
    assert_eq!(unlinked, 2);
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        1,
        "recording rejected or conflicting evidence must create no canonical record"
    );
}

#[test]
fn blank_provenance_hashes_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    for blank in ["", " ", "   ", "\t", "\n"] {
        for (identity, content) in [(blank, "content-a"), ("identity-a", blank)] {
            let result = insert_provenance_row(
                &pool,
                Some(record_id),
                fixture.import_id,
                identity,
                content,
                "WINNER",
            );
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "a blank hash ({blank:?}) must be rejected by a check constraint, got {result:?}"
            );
        }
    }
}

#[test]
fn repeated_provenance_hashes_are_deliberately_permitted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    // Provenance is append-only evidence, not an identity table: the same
    // identity and content hash may legitimately appear in several imports,
    // for example as a WINNER and then as a DUPLICATE. No unique index may
    // collapse that history.
    for classification in ["WINNER", "DUPLICATE", "DUPLICATE"] {
        insert_provenance_row(
            &pool,
            Some(record_id),
            fixture.import_id,
            "identity-a",
            "content-a",
            classification,
        )
        .expect("repeated provenance evidence must be preserved");
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        3
    );
}

#[test]
fn provenance_details_accept_arbitrary_json_without_a_source_specific_schema() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");
    for (index, details) in [
        json!({}),
        json!({"reason": "unresolvable DOI"}),
        json!({"nested": {"rows": [1, 2, 3], "ok": false}}),
    ]
    .into_iter()
    .enumerate()
    {
        let provenance_id: Uuid = diesel::insert_into(metric_record_provenance::table)
            .values((
                metric_record_provenance::record_id.eq(record_id),
                metric_record_provenance::import_id.eq(fixture.import_id),
                metric_record_provenance::identity_hash.eq(format!("identity-{index}")),
                metric_record_provenance::content_hash.eq(format!("content-{index}")),
                metric_record_provenance::classification
                    .eq(MetricRecordProvenanceClassification::Rejected),
                metric_record_provenance::details.eq(details.clone()),
            ))
            .returning(metric_record_provenance::record_provenance_id)
            .get_result(&mut connection)
            .unwrap_or_else(|error| panic!("details {details} must be storable: {error:?}"));
        let loaded: MetricRecordProvenance = metric_record_provenance::table
            .filter(metric_record_provenance::record_provenance_id.eq(provenance_id))
            .first(&mut connection)
            .expect("Failed to reload the provenance row");
        assert_eq!(loaded.details, details);
    }
}

#[test]
fn source_row_origin_is_deliberately_unconstrained() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");
    // Whether rows are counted from zero, from one, or after a header belongs
    // to the later per-format normalizer contract, so no sign, lower-bound or
    // upper-bound rule is imposed here.
    for (index, row_number) in [i64::MIN, -1, 0, 1, i64::MAX].into_iter().enumerate() {
        diesel::insert_into(metric_record_provenance::table)
            .values((
                metric_record_provenance::record_id.eq(record_id),
                metric_record_provenance::import_id.eq(fixture.import_id),
                metric_record_provenance::source_row_number.eq(row_number),
                metric_record_provenance::identity_hash.eq(format!("identity-{index}")),
                metric_record_provenance::content_hash.eq(format!("content-{index}")),
                metric_record_provenance::classification
                    .eq(MetricRecordProvenanceClassification::Winner),
            ))
            .execute(&mut connection)
            .unwrap_or_else(|error| panic!("row number {row_number} must be storable: {error:?}"));
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        5
    );
}

#[test]
fn invalid_provenance_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");

    let unknown_record = insert_provenance_row(
        &pool,
        Some(Uuid::new_v4()),
        fixture.import_id,
        "identity-a",
        "content-a",
        "WINNER",
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

    let unknown_import = insert_provenance_row(
        &pool,
        None,
        Uuid::new_v4(),
        "identity-a",
        "content-a",
        "REJECTED",
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
    insert_provenance_row(
        &pool,
        Some(record_id),
        fixture.import_id,
        "identity-a",
        "content-a",
        "WINNER",
    )
    .expect("the referencing provenance row must be accepted");

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
             durable provenance evidence, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        1,
        "the provenance row must survive the restricted deletions"
    );
}

#[test]
fn metric_record_provenance_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(
            &pool,
            "metric_record_provenance"
        ),
        vec![
            "metric_record_provenance_content_hash_check",
            "metric_record_provenance_identity_hash_check",
        ],
        "metric_record_provenance must carry exactly the two authorized CHECK constraints"
    );
}

#[test]
fn metric_record_provenance_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_record_provenance");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_record_provenance_import_id_fkey",
            "metric_record_provenance_record_id_fkey",
        ],
        "metric_record_provenance must carry exactly the two authorized foreign keys"
    );
    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour: {definition}"
        );
    }
}

#[test]
fn metric_record_provenance_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_record_provenance"),
        vec![
            "metric_record_provenance_identity_hash_idx",
            "metric_record_provenance_import_id_idx",
            "metric_record_provenance_pkey",
            "metric_record_provenance_record_id_idx",
        ],
        "metric_record_provenance must carry exactly its primary key and the \
         three design-required audit indexes"
    );
    for (index, column) in [
        ("metric_record_provenance_import_id_idx", "import_id"),
        ("metric_record_provenance_record_id_idx", "record_id"),
        (
            "metric_record_provenance_identity_hash_idx",
            "identity_hash",
        ),
    ] {
        let definition = index_definition(&pool, "metric_record_provenance", index);
        assert!(
            definition.contains(&format!("({column})")) && !definition.contains("UNIQUE"),
            "{index} must be a plain audit index on {column}: {definition}"
        );
    }
}
