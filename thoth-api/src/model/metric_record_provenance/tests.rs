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
//! Extended by `MET-WP2-01A` with the exact `REJECTED` nullable-hash truth
//! table and the ordered bounded-batch linkage that makes one committed
//! batch's per-row outcomes replayable in `batch_row_index` order.
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

/// Insert one provenance row whose hashes may each be NULL.
///
/// The `MET-WP2-01A` truth table is about absence, so it cannot be exercised
/// through the non-null helper above.
fn insert_provenance_hashes(
    pool: &PgPool,
    import_id: Uuid,
    identity_hash: Option<&str>,
    content_hash: Option<&str>,
    classification: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_record_provenance \
             (import_id, identity_hash, content_hash, classification) \
         VALUES ($1, $2, $3, '{classification}')"
    ))
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(identity_hash)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(content_hash)
    .execute(&mut connection)
}

/// Insert one provenance row linked to a batch position.
fn insert_provenance_batch_link(
    pool: &PgPool,
    import_id: Uuid,
    identity_hash: &str,
    import_batch_id: Option<Uuid>,
    batch_row_index: Option<i64>,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_record_provenance \
             (import_id, identity_hash, content_hash, classification, \
              import_batch_id, batch_row_index) \
         VALUES ($1, $2, $3, 'WINNER', $4, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Text, _>(identity_hash)
    .bind::<diesel::sql_types::Text, _>(format!("content-for-{identity_hash}"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(import_batch_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(batch_row_index)
    .execute(&mut connection)
}

/// Insert one bounded batch under `import_id` and return its id.
fn insert_batch(pool: &PgPool, import_id: Uuid, batch_key: &str) -> Uuid {
    let mut connection = pool.get().expect("Failed to get DB connection");
    diesel::insert_into(crate::schema::metric_import_batch::table)
        .values((
            crate::schema::metric_import_batch::import_id.eq(import_id),
            crate::schema::metric_import_batch::batch_key.eq(batch_key),
            crate::schema::metric_import_batch::request_hash.eq(format!("hash-{batch_key}")),
        ))
        .returning(crate::schema::metric_import_batch::import_batch_id)
        .get_result(&mut connection)
        .expect("Failed to insert the batch fixture row")
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
    assert_eq!(loaded.identity_hash.as_deref(), Some("identity-a"));
    assert_eq!(loaded.content_hash.as_deref(), Some("content-a"));
    assert_eq!(loaded.import_batch_id, None);
    assert_eq!(loaded.batch_row_index, None);
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
        loaded.import_batch_id, None,
        "an unbatched row must keep the MET-WP2-01A linkage null"
    );
    assert_eq!(loaded.batch_row_index, None);
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
            "metric_record_provenance_batch_link_check",
            "metric_record_provenance_batch_row_index_check",
            "metric_record_provenance_classification_hash_check",
            "metric_record_provenance_content_hash_check",
            "metric_record_provenance_identity_hash_check",
        ],
        "metric_record_provenance must carry exactly the two original nonblank \
         CHECK constraints plus the three authorized MET-WP2-01A constraints"
    );
}

#[test]
fn metric_record_provenance_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_record_provenance");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_record_provenance_import_batch_fkey",
            "metric_record_provenance_import_id_fkey",
            "metric_record_provenance_record_id_fkey",
        ],
        "metric_record_provenance must carry exactly the two original foreign keys \
         plus the MET-WP2-01A composite batch key"
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
            "metric_record_provenance_import_batch_id_batch_row_index_key",
            "metric_record_provenance_import_id_idx",
            "metric_record_provenance_pkey",
            "metric_record_provenance_record_id_idx",
        ],
        "metric_record_provenance must carry exactly its primary key, the three \
         design-required audit indexes and the one MET-WP2-01A batch-position \
         unique key"
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

// ---------------------------------------------------------------------------
// MET-WP2-01A: the exact REJECTED hash truth table.
// ---------------------------------------------------------------------------

#[test]
fn rejected_provenance_accepts_every_approved_hash_combination() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    // A row can be refused before canonical identity exists, after identity
    // but before valid content exists, or after both were constructible.
    for (index, (identity, content)) in [
        (None, None),
        (Some("identity"), None),
        (Some("identity"), Some("content")),
    ]
    .into_iter()
    .enumerate()
    {
        let identity = identity.map(|value| format!("{value}-{index}"));
        let content = content.map(|value| format!("{value}-{index}"));
        insert_provenance_hashes(
            &pool,
            fixture.import_id,
            identity.as_deref(),
            content.as_deref(),
            "REJECTED",
        )
        .unwrap_or_else(|error| {
            panic!("REJECTED ({identity:?}, {content:?}) must be storable: {error:?}")
        });
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        3
    );
}

#[test]
fn content_without_identity_is_refused_under_every_classification() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    // Content is only meaningful once canonical identity has been formed, so
    // this state is nonsensical even for REJECTED.
    for (_, label) in CLASSIFICATIONS {
        let result =
            insert_provenance_hashes(&pool, fixture.import_id, None, Some("content-a"), label);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "{label} with a content hash but no identity hash must be refused: {result:?}"
        );
    }
}

#[test]
fn only_rejected_provenance_may_omit_a_hash() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    // Every other classification describes a row that reached canonical
    // comparison, so both hashes must exist.
    for (_, label) in CLASSIFICATIONS {
        if label == "REJECTED" {
            continue;
        }
        for (identity, content) in [(None, None), (Some("identity-a"), None)] {
            let result =
                insert_provenance_hashes(&pool, fixture.import_id, identity, content, label);
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "{label} must require both hashes, but ({identity:?}, {content:?}) \
                 was accepted: {result:?}"
            );
        }
    }
}

#[test]
fn a_present_hash_must_still_be_nonblank() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    // Nullability was relaxed; blankness was not. A NULL check expression is
    // satisfied, so the original nonblank rules still bite whenever a value
    // is actually present.
    for blank in ["", " ", "   ", "\t", "\n"] {
        for (identity, content) in [
            (Some(blank), None),
            (Some(blank), Some("content-a")),
            (Some("identity-a"), Some(blank)),
        ] {
            let result =
                insert_provenance_hashes(&pool, fixture.import_id, identity, content, "REJECTED");
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "a blank hash ({blank:?}) must still be refused even where NULL is \
                 allowed: {result:?}"
            );
        }
    }
}

#[test]
fn null_hashes_round_trip_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let provenance_id: Uuid = diesel::insert_into(metric_record_provenance::table)
        .values((
            metric_record_provenance::import_id.eq(fixture.import_id),
            metric_record_provenance::classification
                .eq(MetricRecordProvenanceClassification::Rejected),
        ))
        .returning(metric_record_provenance::record_provenance_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully unhashed rejection row");

    let loaded: MetricRecordProvenance = metric_record_provenance::table
        .filter(metric_record_provenance::record_provenance_id.eq(provenance_id))
        .first(&mut connection)
        .expect("Failed to load the fully unhashed rejection row");
    assert_eq!(loaded.identity_hash, None);
    assert_eq!(loaded.content_hash, None);
    assert_eq!(loaded.record_id, None);
    assert_eq!(
        loaded.classification,
        MetricRecordProvenanceClassification::Rejected
    );
}

// ---------------------------------------------------------------------------
// MET-WP2-01A: ordered bounded-batch linkage.
// ---------------------------------------------------------------------------

#[test]
fn pre_batch_provenance_rows_remain_valid_without_any_linkage() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id) = fixture_record(&pool, "identity-a");
    // Every provenance row written before MET-WP2-01A carries neither batch
    // field. The migration must leave them valid, and new unbatched rows must
    // stay insertable.
    insert_provenance_row(
        &pool,
        Some(record_id),
        fixture.import_id,
        "identity-a",
        "content-a",
        "WINNER",
    )
    .expect("an unbatched provenance row must remain valid");
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_record_provenance \
              WHERE import_batch_id IS NULL AND batch_row_index IS NULL)",
        ),
        1
    );
}

#[test]
fn the_two_batch_fields_are_null_or_present_together() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let import_batch_id = insert_batch(&pool, fixture.import_id, "batch-1");

    // An index without a batch has no ordering context; a batch link without
    // an index cannot be replayed deterministically.
    for (index, (batch, row_index)) in [(Some(import_batch_id), None), (None, Some(0_i64))]
        .into_iter()
        .enumerate()
    {
        let result = insert_provenance_batch_link(
            &pool,
            fixture.import_id,
            &format!("identity-{index}"),
            batch,
            row_index,
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a half-populated batch linkage ({batch:?}, {row_index:?}) must be \
             refused: {result:?}"
        );
    }

    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-ok",
        Some(import_batch_id),
        Some(0),
    )
    .expect("a complete batch linkage must be accepted");
}

#[test]
fn a_negative_batch_row_index_is_refused() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let import_batch_id = insert_batch(&pool, fixture.import_id, "batch-1");
    // Unlike source_row_number, which mirrors whatever an upstream format
    // counted, batch_row_index is Thoth's own replay ordering.
    for row_index in [i64::MIN, -1] {
        let result = insert_provenance_batch_link(
            &pool,
            fixture.import_id,
            &format!("identity-{row_index}"),
            Some(import_batch_id),
            Some(row_index),
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "batch row index {row_index} must be refused: {result:?}"
        );
    }
    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-zero",
        Some(import_batch_id),
        Some(0),
    )
    .expect("a zero-based batch row index must be accepted");
}

#[test]
fn a_batch_position_holds_exactly_one_provenance_row() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let first_batch = insert_batch(&pool, fixture.import_id, "batch-1");
    let second_batch = insert_batch(&pool, fixture.import_id, "batch-2");

    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-a",
        Some(first_batch),
        Some(0),
    )
    .expect("the first row of the first batch must be accepted");

    let duplicate = insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-b",
        Some(first_batch),
        Some(0),
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "two rows cannot occupy one position in one batch, or replay would be \
         ambiguous: {duplicate:?}"
    );

    // The same position in a different batch is a different position.
    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-c",
        Some(second_batch),
        Some(0),
    )
    .expect("the same row index in another batch must be accepted");

    // And many unlinked rows coexist, because NULLs compare as distinct.
    for identity in ["identity-d", "identity-e"] {
        insert_provenance_batch_link(&pool, fixture.import_id, identity, None, None)
            .expect("unlinked rows must not collide with each other");
    }
}

#[test]
fn provenance_cannot_reference_a_batch_from_another_import() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let other_import_id = Uuid::new_v4();
    crate::model::metric_import::tests::insert_import_row(
        &pool,
        other_import_id,
        fixture.source_account_id,
    );
    let foreign_batch = insert_batch(&pool, other_import_id, "batch-1");

    // The composite key carries import_id precisely so this cannot happen.
    let result = insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-a",
        Some(foreign_batch),
        Some(0),
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "provenance under one import must not reference a batch committed under \
         another: {result:?}"
    );

    // An unknown batch is refused for the same reason.
    let unknown = insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-b",
        Some(Uuid::new_v4()),
        Some(0),
    );
    assert!(
        matches!(
            unknown,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown batch must be refused: {unknown:?}"
    );
}

#[test]
fn deleting_a_referenced_batch_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let import_batch_id = insert_batch(&pool, fixture.import_id, "batch-1");
    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-a",
        Some(import_batch_id),
        Some(0),
    )
    .expect("the linked provenance row must be accepted");

    let result = delete_row(
        &pool,
        "metric_import_batch",
        "import_batch_id",
        import_batch_id,
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced batch must be restricted, not cascade away durable \
         provenance evidence: {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_provenance)"),
        1,
        "the provenance row must survive the restricted deletion"
    );
}

#[test]
fn one_committed_batch_is_readable_in_deterministic_row_order() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-a");
    let batch = insert_batch(&pool, fixture.import_id, "batch-1");
    let other_batch = insert_batch(&pool, fixture.import_id, "batch-2");

    // Insert out of order, and interleave a second batch plus an unlinked
    // row, so ordering cannot accidentally come from insertion order.
    for row_index in [2_i64, 0, 3, 1] {
        insert_provenance_batch_link(
            &pool,
            fixture.import_id,
            &format!("identity-{row_index}"),
            Some(batch),
            Some(row_index),
        )
        .expect("the batch row must be accepted");
    }
    insert_provenance_batch_link(
        &pool,
        fixture.import_id,
        "identity-other",
        Some(other_batch),
        Some(0),
    )
    .expect("the other batch row must be accepted");
    insert_provenance_batch_link(&pool, fixture.import_id, "identity-unlinked", None, None)
        .expect("the unlinked row must be accepted");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let ordered: Vec<MetricRecordProvenance> = metric_record_provenance::table
        .filter(metric_record_provenance::import_batch_id.eq(batch))
        .order(metric_record_provenance::batch_row_index)
        .load(&mut connection)
        .expect("Failed to load the batch provenance rows in order");
    assert_eq!(
        ordered
            .iter()
            .map(|row| row.batch_row_index)
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1), Some(2), Some(3)],
        "one committed batch must be replayable in exact batch_row_index order"
    );
    assert_eq!(
        ordered
            .iter()
            .map(|row| row.identity_hash.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("identity-0"),
            Some("identity-1"),
            Some("identity-2"),
            Some("identity-3"),
        ],
        "each replayed position must carry its own evidence"
    );
    assert!(
        ordered.iter().all(|row| row.import_batch_id == Some(batch)),
        "the ordered read must be scoped to exactly one batch"
    );
}
