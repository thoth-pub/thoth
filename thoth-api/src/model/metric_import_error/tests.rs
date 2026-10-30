//! Focused `MET-WP1-03` database tests for `metric_import_error`: the closed
//! severity enum, the approved field/default contract, the complete
//! authorized CHECK inventory and the non-cascading link to the owning
//! import.
//!
//! Per specification amendment `5455869065` (A3), the publisher/source
//! account/import fixtures used here are the `pub(crate)` helpers defined by
//! the sibling `metric_import` test module in this same task, not helpers
//! added by widening a module outside the write budget.

use std::str::FromStr;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use super::{MetricImportError, MetricImportErrorSeverity};
use crate::db::PgPool;
use crate::model::metric_import::tests::{
    check_constraint_names, fixture_source_account, insert_import_row,
};
use crate::model::metric_platform::tests::{enum_labels, scalar_i64, setup_registry_db};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::Timestamp;
use crate::schema::metric_import_error;

const SEVERITIES: [(MetricImportErrorSeverity, &str); 2] = [
    (MetricImportErrorSeverity::Error, "ERROR"),
    (MetricImportErrorSeverity::Warning, "WARNING"),
];

/// Insert one referenced import and return its id.
fn fixture_import(pool: &PgPool) -> Uuid {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);
    import_id
}

/// Insert one minimal import error, overriding exactly one required text
/// column.
fn insert_error_with_text(
    pool: &PgPool,
    import_id: Uuid,
    column: &str,
    value: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut error_code = "INVALID_ROW";
    let mut message = "The row could not be parsed";
    match column {
        "error_code" => error_code = value,
        "message" => message = value,
        other => panic!("unexpected column {other}"),
    }
    sql_query(
        "INSERT INTO metric_import_error \
             (import_id, error_code, severity, message) \
         VALUES ($1, $2, 'ERROR', $3)",
    )
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Text, _>(error_code)
    .bind::<diesel::sql_types::Text, _>(message)
    .execute(&mut connection)
}

#[test]
fn import_error_severity_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_import_error_severity"),
        vec!["ERROR", "WARNING"],
        "metric_import_error_severity must carry exactly the two approved labels"
    );
}

#[test]
fn severity_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in SEVERITIES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(MetricImportErrorSeverity::from_str(label).unwrap(), variant);
    }
    // No severity is inferred from generic application errors or logging
    // frameworks.
    assert!(MetricImportErrorSeverity::from_str("INFO").is_err());
    assert!(MetricImportErrorSeverity::from_str("FATAL").is_err());
    assert!(MetricImportErrorSeverity::from_str("DEBUG").is_err());
    assert!(MetricImportErrorSeverity::from_str("error").is_err());
}

#[test]
fn both_severities_round_trip_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in SEVERITIES {
        assert_db_enum_roundtrip::<
            MetricImportErrorSeverity,
            crate::schema::sql_types::MetricImportErrorSeverity,
        >(
            pool.as_ref(),
            &format!("'{label}'::metric_import_error_severity"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_import_error_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_error)"),
        0,
        "MET-WP1-03 must not seed any metric_import_error row"
    );
}

#[test]
fn metric_import_error_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let detailed_id: Uuid = diesel::insert_into(metric_import_error::table)
        .values((
            metric_import_error::import_id.eq(import_id),
            metric_import_error::row_number.eq(42),
            metric_import_error::error_code.eq("UNRESOLVED_DOI"),
            metric_import_error::severity.eq(MetricImportErrorSeverity::Error),
            metric_import_error::field_name.eq("doi"),
            metric_import_error::message.eq("The DOI did not resolve to a known publication"),
            metric_import_error::raw_value.eq("10.0000/nonexistent"),
        ))
        .returning(metric_import_error::import_error_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated import-error row");

    let detailed: MetricImportError = metric_import_error::table
        .filter(metric_import_error::import_error_id.eq(detailed_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated import-error row");
    assert_eq!(detailed.import_id, import_id);
    assert_eq!(detailed.row_number, Some(42));
    assert_eq!(detailed.error_code, "UNRESOLVED_DOI");
    assert_eq!(detailed.severity, MetricImportErrorSeverity::Error);
    assert_eq!(detailed.field_name.as_deref(), Some("doi"));
    assert_eq!(
        detailed.message,
        "The DOI did not resolve to a known publication"
    );
    assert_eq!(detailed.raw_value.as_deref(), Some("10.0000/nonexistent"));

    // A finding need not belong to one numbered row or one named field.
    let sparse_id: Uuid = diesel::insert_into(metric_import_error::table)
        .values((
            metric_import_error::import_id.eq(import_id),
            metric_import_error::error_code.eq("TRUNCATED_REPORT"),
            metric_import_error::severity.eq(MetricImportErrorSeverity::Warning),
            metric_import_error::message.eq("The report ended earlier than its declared row count"),
        ))
        .returning(metric_import_error::import_error_id)
        .get_result(&mut connection)
        .expect("Failed to insert the sparse import-error row");
    let sparse: MetricImportError = metric_import_error::table
        .filter(metric_import_error::import_error_id.eq(sparse_id))
        .first(&mut connection)
        .expect("Failed to load the sparse import-error row");
    assert_eq!(sparse.row_number, None);
    assert_eq!(sparse.field_name, None);
    assert_eq!(sparse.raw_value, None);
    assert_eq!(sparse.severity, MetricImportErrorSeverity::Warning);
}

#[test]
fn import_error_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    sql_query(
        "INSERT INTO metric_import_error (import_id, error_code, severity, message) \
         VALUES ($1, 'INVALID_ROW', 'ERROR', 'The row could not be parsed')",
    )
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .execute(&mut connection)
    .expect("Failed to insert the defaulted import-error row");

    let loaded: MetricImportError = metric_import_error::table
        .first(&mut connection)
        .expect("Failed to load the defaulted import-error row");
    assert_ne!(
        loaded.import_error_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate an import_error_id"
    );
    assert_eq!(loaded.row_number, None);
    assert_eq!(loaded.field_name, None);
    assert_eq!(loaded.raw_value, None);
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
}

#[test]
fn blank_required_import_error_text_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    for column in ["error_code", "message"] {
        for blank in ["", " ", "   ", "\t", "\n"] {
            let result = insert_error_with_text(&pool, import_id, column, blank);
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "blank {column} ({blank:?}) must be rejected by a check constraint, got {result:?}"
            );
        }
    }
}

#[test]
fn row_number_is_deliberately_unconstrained() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");
    // Row-number interpretation - zero-based, one-based, or after a header -
    // belongs to the later per-format normalizer contract, so this slice
    // imposes no sign or range rule.
    for row_number in [-1_i64, 0, 1, 9_000_000_000] {
        sql_query(
            "INSERT INTO metric_import_error \
                 (import_id, row_number, error_code, severity, message) \
             VALUES ($1, $2, 'INVALID_ROW', 'ERROR', 'The row could not be parsed')",
        )
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .bind::<diesel::sql_types::BigInt, _>(row_number)
        .execute(&mut connection)
        .unwrap_or_else(|error| panic!("row_number {row_number} must be accepted: {error:?}"));
    }
}

#[test]
fn an_unknown_import_fails_closed() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");
    let result = sql_query(
        "INSERT INTO metric_import_error (import_id, error_code, severity, message) \
         VALUES ($1, 'INVALID_ROW', 'ERROR', 'The row could not be parsed')",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .execute(&mut connection);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown import must be rejected, got {result:?}"
    );
}

#[test]
fn deleting_an_import_with_errors_is_restricted_rather_than_cascaded() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(
            "INSERT INTO metric_import_error (import_id, error_code, severity, message) \
             VALUES ($1, 'INVALID_ROW', 'ERROR', 'The row could not be parsed')",
        )
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .execute(&mut connection)
        .expect("Failed to insert the referencing import-error row");
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    let result = sql_query("DELETE FROM metric_import WHERE import_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(import_id)
        .execute(&mut connection);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a parent import must be restricted so durable row-level \
         evidence is never silently erased, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import_error)"),
        1,
        "the import-error row must survive the restricted deletion"
    );
}

#[test]
fn metric_import_error_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // Specification amendment 5455869065 (A2/A4) closes this set: in
    // particular there is no row_number range or sign check.
    assert_eq!(
        check_constraint_names(&pool, "metric_import_error"),
        vec![
            "metric_import_error_error_code_check",
            "metric_import_error_message_check",
        ],
        "metric_import_error must carry exactly the two authorized CHECK constraints"
    );
}

#[test]
fn metric_import_error_has_exactly_one_non_cascading_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct ForeignKey {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let keys: Vec<(String, String)> = sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = 'public.metric_import_error'::regclass AND c.contype = 'f' \
         ORDER BY c.conname",
    )
    .load::<ForeignKey>(&mut connection)
    .expect("Failed to read the import-error foreign keys")
    .into_iter()
    .map(|key| (key.conname, key.definition))
    .collect();
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_import_error_import_id_fkey"],
        "metric_import_error must carry exactly one foreign key"
    );
    assert!(
        !keys[0].1.contains("ON DELETE"),
        "the owning-import foreign key must stay non-cascading: {}",
        keys[0].1
    );
}

#[test]
fn metric_import_error_has_no_speculative_secondary_index() {
    let (_guard, pool) = setup_registry_db();
    // The complete intended index inventory is exactly the primary key: this
    // slice adds no speculative import-error lookup index.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_import_error')",
        ),
        1,
        "metric_import_error must carry exactly its constraint-derived index"
    );
}
