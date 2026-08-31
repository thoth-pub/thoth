//! Focused `MET-WP1-03` database tests for `metric_import`: the closed
//! lifecycle enum, the approved field/default contract, the complete
//! authorized CHECK and foreign-key inventory, the two mutually exclusive
//! idempotency paths and the single operational index.
//!
//! Per specification amendment `5455869065` (A3), every publisher/source
//! account/import fixture this slice needs is defined here rather than by
//! widening another Metrics test module: `insert_publisher_row` and
//! `delete_row` in `metric_source_account/tests.rs` are private at the
//! reviewed baseline. The `pub(crate)` helpers `fixture_source_and_platform`
//! and `insert_account_row` are consumed as-is, because their existing
//! visibility already permits it.
//!
//! These tests deliberately assert **schema** behaviour only. No status
//! transition, worker claim, lease, retry or idempotent-return runtime
//! behaviour exists in this slice, and nothing here pretends otherwise.

use std::str::FromStr;

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use serde_json::json;
use uuid::Uuid;

use super::{MetricImport, MetricImportStatus};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{
    enum_labels, insert_platform_row, scalar_i64, setup_registry_db,
};
use crate::model::metric_source::tests::insert_source_row;
use crate::model::metric_source_account::tests::{fixture_source_and_platform, insert_account_row};
use crate::model::tests::assert_db_enum_roundtrip;
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_import;

/// The Diesel migration version of `thoth-api/migrations/20260828_v1.9.0`.
pub(crate) const MET_WP1_03_MIGRATION_VERSION: &str = "20260828";

/// Revert migrations until the `MET-WP1-03` import-state migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_registry_migration` and
/// `revert_through_source_state_migration`: a single `revert_last_migration`
/// would only mean "the import-state migration" while it happens to be the
/// newest applied migration. Reverting down to and including the target keeps
/// the meaning under any later migration order, and no future migration name
/// is assumed or hard-coded.
pub(crate) fn revert_through_import_state_migration(connection: &mut PgConnection) {
    let import_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_03_MIGRATION_VERSION);
    assert!(
        import_migration_applied,
        "the MET-WP1-03 import-state migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_03_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one `publisher` row for the optional import publisher FK.
///
/// Defined locally under amendment A3: the equivalent helper in
/// `metric_source_account/tests.rs` is private and that file is outside this
/// task's write budget.
pub(crate) fn insert_publisher_row(pool: &PgPool, publisher_id: Uuid) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("INSERT INTO publisher (publisher_id, publisher_name) VALUES ($1, 'Test publisher')")
        .bind::<diesel::sql_types::Uuid, _>(publisher_id)
        .execute(&mut connection)
        .expect("Failed to insert publisher fixture row");
}

/// Insert one referenced source account and return its id.
pub(crate) fn fixture_source_account(pool: &PgPool) -> Uuid {
    let (source_id, platform_id) = fixture_source_and_platform(pool);
    let source_account_id = Uuid::new_v4();
    insert_account_row(pool, source_account_id, source_id, platform_id, "account_a");
    source_account_id
}

/// Insert a second referenced source account under a fresh source.
fn fixture_second_source_account(pool: &PgPool) -> Uuid {
    let source_id = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    insert_source_row(pool, source_id, "test_source_b");
    insert_platform_row(pool, platform_id, "test_platform_b");
    let source_account_id = Uuid::new_v4();
    insert_account_row(pool, source_account_id, source_id, platform_id, "account_b");
    source_account_id
}

/// Insert one `metric_import` row with an explicit id through raw SQL, for
/// dependent `metric_import_error` fixtures.
pub(crate) fn insert_import_row(pool: &PgPool, import_id: Uuid, source_account_id: Uuid) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_import \
             (import_id, source_account_id, format_code, format_version, status, \
              normalizer_version, created_by) \
         VALUES ($1, $2, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test')",
    )
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .execute(&mut connection)
    .expect("Failed to insert metric_import fixture row");
}

/// Insert one minimal import, overriding exactly one required text column.
fn insert_import_with_text(pool: &PgPool, column: &str, value: &str) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let source_account_id = single_source_account(&mut connection);
    let mut columns = [
        ("format_code", "thoth_csv"),
        ("format_version", "1"),
        ("normalizer_version", "normalizer/1"),
        ("created_by", "test"),
    ];
    for entry in columns.iter_mut() {
        if entry.0 == column {
            entry.1 = value;
        }
    }
    assert!(
        columns.iter().any(|entry| entry.1 == value),
        "insert_import_with_text was given an unknown column {column}"
    );
    sql_query(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by) \
         VALUES ($1, $2, $3, 'UPLOADED', $4, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Text, _>(columns[0].1)
    .bind::<diesel::sql_types::Text, _>(columns[1].1)
    .bind::<diesel::sql_types::Text, _>(columns[2].1)
    .bind::<diesel::sql_types::Text, _>(columns[3].1)
    .execute(&mut connection)
}

/// The only source account present in a single-account fixture database.
fn single_source_account(connection: &mut PgConnection) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        source_account_id: Uuid,
    }
    sql_query("SELECT source_account_id FROM metric_source_account ORDER BY external_key LIMIT 1")
        .get_result::<Row>(connection)
        .expect("Failed to read the fixture source account")
        .source_account_id
}

/// Insert one minimal import, overriding exactly one counter column.
fn insert_import_with_counter(
    pool: &PgPool,
    source_account_id: Uuid,
    column: &str,
    value: i64,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by, {column}) \
         VALUES ($1, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test', $2)"
    ))
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::BigInt, _>(value)
    .execute(&mut connection)
}

/// Insert one import carrying explicit idempotency evidence.
fn insert_import_with_evidence(
    pool: &PgPool,
    source_account_id: Uuid,
    upstream_report_id: Option<&str>,
    raw_sha256: Option<&str>,
    format_version: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by, upstream_report_id, raw_sha256) \
         VALUES ($1, 'thoth_csv', $2, 'UPLOADED', 'normalizer/1', 'test', $3, $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Text, _>(format_version)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(upstream_report_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(raw_sha256)
    .execute(&mut connection)
}

fn delete_row(pool: &PgPool, table: &str, id_column: &str, id: Uuid) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!("DELETE FROM {table} WHERE {id_column} = $1"))
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut connection)
}

/// The sorted names of one table's CHECK constraints.
pub(crate) fn check_constraint_names(pool: &PgPool, table: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT c.conname::text AS conname \
         FROM pg_constraint c \
         WHERE c.conrelid = $1::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .bind::<diesel::sql_types::Text, _>(format!("public.{table}"))
    .load::<Row>(&mut connection)
    .expect("Failed to read check constraints")
    .into_iter()
    .map(|row| row.conname)
    .collect()
}

const IMPORT_STATUSES: [(MetricImportStatus, &str); 6] = [
    (MetricImportStatus::Uploaded, "UPLOADED"),
    (MetricImportStatus::Queued, "QUEUED"),
    (MetricImportStatus::Processing, "PROCESSING"),
    (MetricImportStatus::Completed, "COMPLETED"),
    (
        MetricImportStatus::CompletedWithErrors,
        "COMPLETED_WITH_ERRORS",
    ),
    (MetricImportStatus::Failed, "FAILED"),
];

#[test]
fn import_status_enum_has_exactly_the_approved_labels() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        enum_labels(&pool, "metric_import_status"),
        vec![
            "UPLOADED",
            "QUEUED",
            "PROCESSING",
            "COMPLETED",
            "COMPLETED_WITH_ERRORS",
            "FAILED",
        ],
        "metric_import_status must carry exactly the six approved lifecycle labels"
    );
}

#[test]
fn import_status_string_conversion_round_trips_and_rejects_unknown_values() {
    for (variant, label) in IMPORT_STATUSES {
        assert_eq!(variant.to_string(), label);
        assert_eq!(MetricImportStatus::from_str(label).unwrap(), variant);
    }
    assert!(MetricImportStatus::from_str("OTHER").is_err());
    assert!(MetricImportStatus::from_str("CANCELLED").is_err());
    assert!(MetricImportStatus::from_str("uploaded").is_err());
}

#[test]
fn every_import_status_round_trips_through_postgres() {
    let (_guard, pool) = setup_registry_db();
    for (variant, label) in IMPORT_STATUSES {
        assert_db_enum_roundtrip::<MetricImportStatus, crate::schema::sql_types::MetricImportStatus>(
            pool.as_ref(),
            &format!("'{label}'::metric_import_status"),
            variant,
        );
    }
}

#[test]
fn migration_seeds_no_import_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import)"),
        0,
        "MET-WP1-03 must not seed any metric_import row"
    );
}

#[test]
fn metric_import_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let completed_at = Timestamp::parse_from_rfc3339("2026-08-01T09:30:00Z").unwrap();
    let import_id: Uuid = diesel::insert_into(metric_import::table)
        .values((
            metric_import::source_account_id.eq(source_account_id),
            metric_import::publisher_id.eq(publisher_id),
            metric_import::format_code.eq("thoth_csv"),
            metric_import::format_version.eq("2"),
            metric_import::raw_object_key.eq("imports/2026/07/report.csv"),
            metric_import::raw_sha256.eq("abc123"),
            metric_import::upstream_report_id.eq("upstream-42"),
            metric_import::period_start.eq(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
            metric_import::period_end.eq(NaiveDate::from_ymd_opt(2026, 7, 31).unwrap()),
            metric_import::status.eq(MetricImportStatus::CompletedWithErrors),
            metric_import::received_count.eq(100),
            metric_import::accepted_count.eq(80),
            metric_import::duplicate_count.eq(9),
            metric_import::revision_count.eq(5),
            metric_import::conflict_count.eq(3),
            metric_import::invalid_count.eq(3),
            metric_import::normalizer_version.eq("normalizer/7"),
            metric_import::manifest.eq(json!({"rows": 100, "source_file": "report.csv"})),
            metric_import::created_by.eq("publisher_upload_service"),
            metric_import::completed_at.eq(completed_at),
        ))
        .returning(metric_import::import_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated import row");

    let loaded: MetricImport = metric_import::table
        .filter(metric_import::import_id.eq(import_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated import row");
    assert_eq!(loaded.import_id, import_id);
    assert_eq!(loaded.source_account_id, source_account_id);
    assert_eq!(loaded.publisher_id, Some(publisher_id));
    assert_eq!(loaded.format_code, "thoth_csv");
    assert_eq!(loaded.format_version, "2");
    assert_eq!(
        loaded.raw_object_key.as_deref(),
        Some("imports/2026/07/report.csv")
    );
    assert_eq!(loaded.raw_sha256.as_deref(), Some("abc123"));
    assert_eq!(loaded.upstream_report_id.as_deref(), Some("upstream-42"));
    assert_eq!(
        loaded.period_start,
        Some(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
    );
    assert_eq!(
        loaded.period_end,
        Some(NaiveDate::from_ymd_opt(2026, 7, 31).unwrap())
    );
    assert_eq!(loaded.status, MetricImportStatus::CompletedWithErrors);
    assert_eq!(loaded.received_count, 100);
    assert_eq!(loaded.accepted_count, 80);
    assert_eq!(loaded.duplicate_count, 9);
    assert_eq!(loaded.revision_count, 5);
    assert_eq!(loaded.conflict_count, 3);
    assert_eq!(loaded.invalid_count, 3);
    assert_eq!(loaded.normalizer_version, "normalizer/7");
    assert_eq!(
        loaded.manifest,
        json!({"rows": 100, "source_file": "report.csv"})
    );
    assert_eq!(loaded.created_by, "publisher_upload_service");
    assert_eq!(loaded.completed_at, Some(completed_at));
}

#[test]
fn import_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Deliberately supply only the non-defaulted required columns through raw
    // SQL so the database defaults are exercised rather than restated by a
    // Diesel fixture.
    sql_query(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by) \
         VALUES ($1, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test')",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .execute(&mut connection)
    .expect("Failed to insert the defaulted import row");

    let loaded: MetricImport = metric_import::table
        .first(&mut connection)
        .expect("Failed to load the defaulted import row");
    assert_ne!(
        loaded.import_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate an import_id"
    );
    assert_eq!(loaded.publisher_id, None);
    assert_eq!(loaded.raw_object_key, None);
    assert_eq!(loaded.raw_sha256, None);
    assert_eq!(loaded.upstream_report_id, None);
    assert_eq!(loaded.period_start, None);
    assert_eq!(loaded.period_end, None);
    assert_eq!(loaded.received_count, 0);
    assert_eq!(loaded.accepted_count, 0);
    assert_eq!(loaded.duplicate_count, 0);
    assert_eq!(loaded.revision_count, 0);
    assert_eq!(loaded.conflict_count, 0);
    assert_eq!(loaded.invalid_count, 0);
    assert_eq!(
        loaded.manifest,
        json!({}),
        "manifest must default to an empty JSON object"
    );
    assert_eq!(
        loaded.completed_at, None,
        "completed_at must stay unset until later completion behaviour exists"
    );
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
}

#[test]
fn blank_required_import_text_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    fixture_source_account(&pool);
    for column in [
        "format_code",
        "format_version",
        "normalizer_version",
        "created_by",
    ] {
        for blank in ["", " ", "   ", "\t", "\n"] {
            let result = insert_import_with_text(&pool, column, blank);
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
fn negative_import_counters_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    for column in [
        "received_count",
        "accepted_count",
        "duplicate_count",
        "revision_count",
        "conflict_count",
        "invalid_count",
    ] {
        let result = insert_import_with_counter(&pool, source_account_id, column, -1);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a negative {column} must be rejected by a check constraint, got {result:?}"
        );
    }
}

#[test]
fn zero_and_positive_import_counters_are_accepted() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    for column in [
        "received_count",
        "accepted_count",
        "duplicate_count",
        "revision_count",
        "conflict_count",
        "invalid_count",
    ] {
        for value in [0, 1, 9_000_000_000] {
            insert_import_with_counter(&pool, source_account_id, column, value)
                .unwrap_or_else(|error| panic!("{column} = {value} must be accepted: {error:?}"));
        }
    }
}

#[test]
fn duplicate_upstream_report_id_is_rejected_within_a_source_account() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    insert_import_with_evidence(&pool, source_account_id, Some("report-1"), None, "1")
        .expect("the first upstream-identified import must be accepted");
    let result = insert_import_with_evidence(&pool, source_account_id, Some("report-1"), None, "1");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a repeated (source_account_id, upstream_report_id) must be rejected, got {result:?}"
    );
}

#[test]
fn upstream_report_uniqueness_ignores_a_differing_format_version_and_raw_hash() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    insert_import_with_evidence(
        &pool,
        source_account_id,
        Some("report-1"),
        Some("hash-a"),
        "1",
    )
    .expect("the first upstream-identified import must be accepted");
    // The upstream-report path is keyed only on (source_account_id,
    // upstream_report_id): a different format version or raw hash must not
    // create a second logical job for the same upstream report.
    let result = insert_import_with_evidence(
        &pool,
        source_account_id,
        Some("report-1"),
        Some("hash-b"),
        "2",
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "the upstream-report path must not be widened by format_version/raw_sha256, got {result:?}"
    );
}

#[test]
fn duplicate_raw_hash_and_format_version_is_rejected_when_no_upstream_report_id_is_supplied() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    insert_import_with_evidence(&pool, source_account_id, None, Some("hash-a"), "1")
        .expect("the first hash-identified import must be accepted");
    let result = insert_import_with_evidence(&pool, source_account_id, None, Some("hash-a"), "1");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a repeated (source_account_id, raw_sha256, format_version) must be rejected, got {result:?}"
    );
}

#[test]
fn the_raw_hash_fallback_is_scoped_to_its_format_version() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    insert_import_with_evidence(&pool, source_account_id, None, Some("hash-a"), "1")
        .expect("the first hash-identified import must be accepted");
    insert_import_with_evidence(&pool, source_account_id, None, Some("hash-a"), "2")
        .expect("the same raw hash under a different format version is a distinct logical import");
}

#[test]
fn the_raw_hash_fallback_does_not_apply_when_an_upstream_report_id_is_present() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    // Identical (source_account_id, raw_sha256, format_version), but distinct
    // upstream report IDs: the fallback path is deliberately inapplicable, so
    // these are two distinct upstream reports rather than one duplicate.
    insert_import_with_evidence(
        &pool,
        source_account_id,
        Some("report-1"),
        Some("hash-a"),
        "1",
    )
    .expect("the first upstream-identified import must be accepted");
    insert_import_with_evidence(
        &pool,
        source_account_id,
        Some("report-2"),
        Some("hash-a"),
        "1",
    )
    .expect("the raw-hash fallback must not replace or broaden the upstream-report-id path");
}

#[test]
fn different_source_accounts_do_not_collide_on_the_same_evidence() {
    let (_guard, pool) = setup_registry_db();
    let first_account = fixture_source_account(&pool);
    let second_account = fixture_second_source_account(&pool);
    insert_import_with_evidence(&pool, first_account, Some("report-1"), None, "1")
        .expect("the first account's upstream-identified import must be accepted");
    insert_import_with_evidence(&pool, second_account, Some("report-1"), None, "1")
        .expect("import idempotency is source-account scoped, not global");
    insert_import_with_evidence(&pool, first_account, None, Some("hash-a"), "1")
        .expect("the first account's hash-identified import must be accepted");
    insert_import_with_evidence(&pool, second_account, None, Some("hash-a"), "1")
        .expect("the raw-hash fallback is also source-account scoped");
}

#[test]
fn imports_without_idempotency_evidence_are_permitted() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    // The approved model keeps both idempotency columns nullable: later
    // upload/claim APIs own the rule for when sufficient evidence is required
    // before queueing or processing. No constraint may require it at row
    // creation, so repeated evidence-free rows must remain insertable.
    for _ in 0..3 {
        insert_import_with_evidence(&pool, source_account_id, None, None, "1")
            .expect("an import carrying no idempotency evidence must be permitted");
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import)"),
        3,
        "no partial unique index may collapse evidence-free imports"
    );
}

#[test]
fn import_period_ordering_is_deliberately_unconstrained() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");
    // The approved design places period ordering on metric_record, not on
    // metric_import: malformed source/report period evidence must remain
    // representable at the import/error layer.
    sql_query(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by, period_start, period_end) \
         VALUES ($1, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test', \
                 DATE '2026-07-31', DATE '2026-07-01')",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .execute(&mut connection)
    .expect("an inverted import period must remain representable");
}

#[test]
fn invalid_import_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let unknown_account = sql_query(
        "INSERT INTO metric_import \
             (source_account_id, format_code, format_version, status, \
              normalizer_version, created_by) \
         VALUES ($1, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test')",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .execute(&mut connection);
    assert!(
        matches!(
            unknown_account,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown source account must be rejected, got {unknown_account:?}"
    );

    let unknown_publisher = sql_query(
        "INSERT INTO metric_import \
             (source_account_id, publisher_id, format_code, format_version, status, \
              normalizer_version, created_by) \
         VALUES ($1, $2, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test')",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .execute(&mut connection);
    assert!(
        matches!(
            unknown_publisher,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown publisher must be rejected, got {unknown_publisher:?}"
    );
}

#[test]
fn deleting_a_referenced_source_account_or_publisher_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_source_account(&pool);
    let publisher_id = Uuid::new_v4();
    insert_publisher_row(&pool, publisher_id);
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(
            "INSERT INTO metric_import \
                 (source_account_id, publisher_id, format_code, format_version, status, \
                  normalizer_version, created_by) \
             VALUES ($1, $2, 'thoth_csv', '1', 'UPLOADED', 'normalizer/1', 'test')",
        )
        .bind::<diesel::sql_types::Uuid, _>(source_account_id)
        .bind::<diesel::sql_types::Uuid, _>(publisher_id)
        .execute(&mut connection)
        .expect("Failed to insert the referencing import row");
    }

    for (table, id_column, id) in [
        (
            "metric_source_account",
            "source_account_id",
            source_account_id,
        ),
        ("publisher", "publisher_id", publisher_id),
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
             durable import evidence, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import)"),
        1,
        "the import row must survive the restricted deletions"
    );
}

#[test]
fn status_and_creation_time_have_the_required_operational_index() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND tablename = 'metric_import' \
                AND indexname = 'metric_import_status_created_at_idx' \
                AND indexdef LIKE '%(status, created_at)%')",
        ),
        1,
        "the design-required operational index on (status, created_at) must exist"
    );
}

#[test]
fn the_two_idempotency_indexes_are_partial_and_mutually_exclusive() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct Index {
        #[diesel(sql_type = diesel::sql_types::Text)]
        indexdef: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let definitions: Vec<String> = sql_query(
        "SELECT indexdef FROM pg_indexes \
         WHERE schemaname = 'public' AND tablename = 'metric_import' \
           AND indexdef LIKE '%WHERE%' ORDER BY indexname",
    )
    .load::<Index>(&mut connection)
    .expect("Failed to read the partial index definitions")
    .into_iter()
    .map(|index| index.indexdef)
    .collect();
    assert_eq!(
        definitions.len(),
        2,
        "exactly two partial idempotency indexes must exist, found {definitions:?}"
    );
    let upstream = definitions
        .iter()
        .find(|definition| definition.contains("upstream_report_id, "))
        .or_else(|| {
            definitions
                .iter()
                .find(|definition| definition.contains("(source_account_id, upstream_report_id)"))
        })
        .expect("the upstream-report idempotency index must exist");
    assert!(
        upstream.contains("UNIQUE") && upstream.contains("(upstream_report_id IS NOT NULL)"),
        "the upstream-report index must be unique and apply only when an \
         upstream report ID is supplied: {upstream}"
    );
    let fallback = definitions
        .iter()
        .find(|definition| definition.contains("raw_sha256"))
        .expect("the raw-hash fallback idempotency index must exist");
    assert!(
        fallback.contains("UNIQUE")
            && fallback.contains("(source_account_id, raw_sha256, format_version)")
            && fallback.contains("upstream_report_id IS NULL")
            && fallback.contains("raw_sha256 IS NOT NULL"),
        "the fallback index must be unique on (source_account_id, raw_sha256, \
         format_version) and apply only when no upstream report ID is supplied \
         and a raw hash is: {fallback}"
    );
}

#[test]
fn metric_import_has_no_speculative_secondary_index() {
    let (_guard, pool) = setup_registry_db();
    // The complete intended index inventory is exactly: the primary key, the
    // two partial unique idempotency indexes and the single operational
    // (status, created_at) index.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_import')",
        ),
        4,
        "metric_import must carry exactly its primary key, two idempotency \
         indexes and one operational index"
    );
}

#[test]
fn metric_import_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // Specification amendment 5455869065 (A1/A4) closes this set: no import
    // period-ordering check, no counter-relationship check and no other
    // speculative integrity rule may exist.
    assert_eq!(
        check_constraint_names(&pool, "metric_import"),
        vec![
            "metric_import_accepted_count_check",
            "metric_import_conflict_count_check",
            "metric_import_created_by_check",
            "metric_import_duplicate_count_check",
            "metric_import_format_code_check",
            "metric_import_format_version_check",
            "metric_import_invalid_count_check",
            "metric_import_normalizer_version_check",
            "metric_import_received_count_check",
            "metric_import_revision_count_check",
        ],
        "metric_import must carry exactly the ten authorized CHECK constraints"
    );
}

#[test]
fn metric_import_has_exactly_the_two_authorized_foreign_keys() {
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
         WHERE c.conrelid = 'public.metric_import'::regclass AND c.contype = 'f' \
         ORDER BY c.conname",
    )
    .load::<ForeignKey>(&mut connection)
    .expect("Failed to read the import foreign keys")
    .into_iter()
    .map(|key| (key.conname, key.definition))
    .collect();
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_import_publisher_id_fkey",
            "metric_import_source_account_id_fkey",
        ],
        "metric_import must carry exactly the two authorized foreign keys; in \
         particular created_by must not become an identity/account FK"
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
fn reverting_through_the_import_state_migration_removes_it_and_reapplication_restores_it() {
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

    revert_through_import_state_migration(&mut connection);

    // The MET-WP1-03 tables and both enum types are gone...
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_import', 'metric_import_error'))",
        ),
        0,
        "the downgrade must drop metric_import_error and metric_import"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname IN ('metric_import_status', 'metric_import_error_severity'))",
        ),
        0,
        "the downgrade must drop both metric_import_status and \
         metric_import_error_severity"
    );

    // ...while the MET-WP1-01 registry and MET-WP1-02 source-state schema and
    // the exact measure seeds survive.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint'))",
        ),
        6,
        "the downgrade must leave the MET-WP1-01/02 schema in place"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "the downgrade must leave the measure seeds byte-identical"
    );

    // Reapplication recreates both enums and both empty tables.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the import-state migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typname IN ('metric_import_status', 'metric_import_error_severity'))",
        ),
        2,
        "reapplication must recreate both enum types cleanly"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "((SELECT COUNT(*) FROM metric_import) \
              + (SELECT COUNT(*) FROM metric_import_error))",
        ),
        0,
        "reapplication must seed no import or import-error row"
    );
    assert_eq!(
        measure_seeds(&mut connection),
        seeds_before,
        "reapplication must leave the measure seeds byte-identical"
    );
}
