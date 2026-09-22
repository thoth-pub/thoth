//! Focused `MET-WP7-PREREQ-02` database tests for `metric_identifier_quarantine`:
//! the exact approved column contract, the nonblank, period and uniqueness
//! constraints, the four non-cascading foreign keys, the deliberate absence of
//! any DOI-format rule or request-identity column, and the Diesel mapping.
//!
//! This module also owns the migration evidence for the slice: application
//! from the exact predecessor schema, an empty rollback that restores that
//! schema exactly, a populated rollback that refuses before dropping anything
//! (including against a concurrent insert), and apply/revert/reapply through
//! the embedded Diesel runner.
//!
//! Coordinator and checkpoint behaviour is proven in
//! `crate::model::metric_ingestion::tests` and
//! `crate::model::metric_ingestion_lifecycle::tests`.

use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricIdentifierQuarantine;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::{
    check_constraint_names, fixture_source_account, insert_import_row,
};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_names};
use crate::model::tests::db::test_db_url;
use crate::model::Doi;
use crate::schema::metric_identifier_quarantine;

/// The Diesel migration version of `thoth-api/migrations/20260921_v1.9.0`.
pub(crate) const MET_WP7_PREREQ_02_MIGRATION_VERSION: &str = "20260921";

/// Every column of the approved quarantine contract, with its exact type,
/// nullability and default, as `(name, type, not_null, default)`.
const QUARANTINE_COLUMNS: [(&str, &str, bool, Option<&str>); 14] = [
    (
        "identifier_quarantine_id",
        "uuid",
        true,
        Some("uuid_generate_v4()"),
    ),
    ("record_provenance_id", "uuid", true, None),
    ("source_account_id", "uuid", true, None),
    ("platform_id", "uuid", true, None),
    ("measure_id", "uuid", true, None),
    ("schema_version", "text", true, None),
    ("work_doi", "text", true, None),
    ("period_start", "date", true, None),
    ("period_end", "date", true, None),
    ("reporting_grain", "metric_reporting_grain", true, None),
    ("country_code", "text", false, None),
    ("value", "bigint", true, None),
    ("methodology_version", "text", true, None),
    (
        "created_at",
        "timestamp with time zone",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
];

/// Column names that would betray request identity, raw source evidence, a
/// second classification store or resolution state smuggled into this slice.
const FORBIDDEN_COLUMNS: [&str; 22] = [
    "classification",
    "client_ip",
    "cookie",
    "cookies",
    "credential",
    "credentials",
    "import_id",
    "ip",
    "ip_address",
    "query_string",
    "raw",
    "raw_row",
    "raw_value",
    "reason_code",
    "referer",
    "referrer",
    "request_id",
    "resolved_at",
    "session_id",
    "status",
    "updated_at",
    "user_agent",
];

/// Every whitespace-only value the nonblank rule must refuse, including code
/// points a C-locale `[:space:]` class would miss.
const BLANK_VALUES: [&str; 6] = [
    "",
    " ",
    "\t\n\r",
    "\u{00A0}",
    "\u{2003}",
    "\u{3000} \u{202F}",
];

/// Revert every migration newer than the quarantine migration, leaving the
/// historical MET-WP7-PREREQ-02 schema head applied.
fn revert_newer_than_quarantine_migration(connection: &mut PgConnection) {
    assert!(
        is_applied(connection),
        "the MET-WP7-PREREQ-02 quarantine migration must be applied before isolating it"
    );
    loop {
        let next = connection
            .applied_migrations()
            .expect("Failed to read applied migrations")
            .first()
            .expect("the quarantine migration must still be applied")
            .to_string();
        if next == MET_WP7_PREREQ_02_MIGRATION_VERSION {
            return;
        }
        connection
            .revert_last_migration(MIGRATIONS)
            .unwrap_or_else(|error| panic!("Failed to revert later migration {next}: {error}"));
    }
}

/// Revert migrations until the quarantine migration itself has been reverted.
fn revert_through_quarantine_migration(connection: &mut PgConnection) {
    assert!(
        is_applied(connection),
        "the MET-WP7-PREREQ-02 quarantine migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP7_PREREQ_02_MIGRATION_VERSION {
            return;
        }
    }
}

/// Attempt to revert the quarantine migration, peeling any newer migration off
/// first, and return its error. A newer migration refusing its own rollback is
/// reported as that failure, never mistaken for this guard.
fn try_revert_quarantine_migration(connection: &mut PgConnection) -> String {
    assert!(is_applied(connection));
    loop {
        let next = connection
            .applied_migrations()
            .expect("Failed to read applied migrations")
            .first()
            .expect("the quarantine migration must still be applied")
            .to_string();
        if next == MET_WP7_PREREQ_02_MIGRATION_VERSION {
            return match connection.revert_last_migration(MIGRATIONS) {
                Ok(version) => panic!(
                    "the rollback must have failed closed, but it reverted {version} \
                     and discarded the quarantine evidence"
                ),
                Err(error) => error.to_string(),
            };
        }
        connection
            .revert_last_migration(MIGRATIONS)
            .unwrap_or_else(|error| panic!("Failed to revert {next}: {error}"));
    }
}

fn is_applied(connection: &mut PgConnection) -> bool {
    connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP7_PREREQ_02_MIGRATION_VERSION)
}

fn establish() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database")
}

/// One scalar `BIGINT` result on a caller-owned connection, for migration tests
/// whose table shapes change underneath any pooled connection.
fn on_connection(connection: &mut PgConnection, query: &str) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
        .get_result(connection)
        .expect("Failed to run scalar query")
}

/// A sorted catalogue of every public-schema object shape: relations, columns
/// with types, nullability and defaults, constraints, indexes, enum labels,
/// triggers and functions. Two equal fingerprints are the same schema.
fn schema_fingerprint(connection: &mut PgConnection) -> BTreeSet<String> {
    #[derive(diesel::QueryableByName)]
    struct Line {
        #[diesel(sql_type = diesel::sql_types::Text)]
        line: String,
    }
    sql_query(
        "SELECT 'relation ' || c.relname || ' ' || c.relkind::text AS line \
           FROM pg_class c \
          WHERE c.relnamespace = 'public'::regnamespace \
            AND c.relkind IN ('r', 'i', 'S', 'v', 'm', 'p') \
         UNION ALL \
         SELECT 'column ' || c.relname || '.' || a.attname || ' ' \
                || format_type(a.atttypid, a.atttypmod) || ' notnull=' || a.attnotnull \
                || ' default=' || COALESCE(pg_get_expr(d.adbin, d.adrelid), '') \
           FROM pg_attribute a \
           JOIN pg_class c ON c.oid = a.attrelid \
           LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
          WHERE c.relnamespace = 'public'::regnamespace AND c.relkind IN ('r', 'p') \
            AND a.attnum > 0 AND NOT a.attisdropped \
         UNION ALL \
         SELECT 'constraint ' || c.relname || '.' || con.conname || ' ' \
                || pg_get_constraintdef(con.oid) \
           FROM pg_constraint con \
           JOIN pg_class c ON c.oid = con.conrelid \
          WHERE c.relnamespace = 'public'::regnamespace \
         UNION ALL \
         SELECT 'index ' || indexname || ' ' || indexdef \
           FROM pg_indexes WHERE schemaname = 'public' \
         UNION ALL \
         SELECT 'enum ' || t.typname || ' ' \
                || (SELECT string_agg(e.enumlabel, ',' ORDER BY e.enumsortorder) \
                      FROM pg_enum e WHERE e.enumtypid = t.oid) \
           FROM pg_type t \
          WHERE t.typnamespace = 'public'::regnamespace AND t.typtype = 'e' \
         UNION ALL \
         SELECT 'trigger ' || c.relname || '.' || tg.tgname || ' ' || pg_get_triggerdef(tg.oid) \
           FROM pg_trigger tg \
           JOIN pg_class c ON c.oid = tg.tgrelid \
          WHERE c.relnamespace = 'public'::regnamespace AND NOT tg.tgisinternal \
         UNION ALL \
         SELECT 'function ' || p.proname || ' ' || md5(pg_get_functiondef(p.oid)) \
           FROM pg_proc p \
          WHERE p.pronamespace = 'public'::regnamespace AND p.prokind = 'f'",
    )
    .load::<Line>(connection)
    .expect("Failed to read the schema fingerprint")
    .into_iter()
    .map(|line| line.line)
    .collect()
}

/// The canonical rows one quarantine row references: a provenance row under an
/// import of a source account, that account's platform, and a seeded measure.
#[derive(Clone, Copy)]
struct QuarantineRefs {
    record_provenance_id: Uuid,
    source_account_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    import_id: Uuid,
}

fn uuid_of(pool: &PgPool, query: &str) -> Uuid {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let text: String = diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(query))
        .get_result(&mut connection)
        .expect("Failed to run uuid query");
    Uuid::parse_str(&text).expect("a uuid")
}

/// Insert one `REJECTED` / `UNKNOWN_DOI` provenance row under `import_id`.
fn insert_rejected_provenance(pool: &PgPool, import_id: Uuid) -> Uuid {
    let record_provenance_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_record_provenance \
             (record_provenance_id, import_id, classification, details) \
         VALUES ($1, $2, 'REJECTED', \
                 '{\"schema\": \"thoth-metric-provenance-details/1\", \
                   \"reason_code\": \"UNKNOWN_DOI\", \"reporting_grain\": \"DAY\"}'::jsonb)",
    )
    .bind::<diesel::sql_types::Uuid, _>(record_provenance_id)
    .bind::<diesel::sql_types::Uuid, _>(import_id)
    .execute(&mut connection)
    .expect("Failed to insert provenance fixture row");
    record_provenance_id
}

/// The same references with a fresh `REJECTED` provenance row, so one
/// fixture account can carry any number of independent quarantine rows.
fn fresh(pool: &PgPool, refs: &QuarantineRefs) -> QuarantineRefs {
    QuarantineRefs {
        record_provenance_id: insert_rejected_provenance(pool, refs.import_id),
        ..*refs
    }
}

/// The fixture account, import and first provenance row. The account fixture
/// uses fixed codes, so this runs once per test; [`fresh`] adds more rows.
fn fixture_refs(pool: &PgPool) -> QuarantineRefs {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);
    QuarantineRefs {
        record_provenance_id: insert_rejected_provenance(pool, import_id),
        source_account_id,
        platform_id: uuid_of(
            pool,
            &format!("(SELECT platform_id::text FROM metric_source_account WHERE source_account_id = '{source_account_id}')"),
        ),
        measure_id: uuid_of(
            pool,
            "(SELECT measure_id::text FROM metric_measure WHERE code = 'title_sessions')",
        ),
        import_id,
    }
}

/// Insert one quarantine row through raw SQL, overriding any column by name
/// with a raw SQL expression, so database defaults and constraints are
/// exercised rather than restated.
fn insert_quarantine(
    pool: &PgPool,
    refs: &QuarantineRefs,
    overrides: &[(&str, String)],
) -> Result<usize, DieselError> {
    let mut columns: Vec<(&str, String)> = vec![
        (
            "record_provenance_id",
            format!("'{}'", refs.record_provenance_id),
        ),
        ("source_account_id", format!("'{}'", refs.source_account_id)),
        ("platform_id", format!("'{}'", refs.platform_id)),
        ("measure_id", format!("'{}'", refs.measure_id)),
        ("schema_version", "'thoth-normalized-metrics/1'".into()),
        ("work_doi", "'https://doi.org/10.12345/unresolved'".into()),
        ("period_start", "DATE '2026-03-01'".into()),
        ("period_end", "DATE '2026-03-02'".into()),
        ("reporting_grain", "'DAY'".into()),
        ("country_code", "NULL".into()),
        ("value", "7".into()),
        ("methodology_version", "'cloudfront-title-session/2'".into()),
    ];
    for (name, value) in overrides {
        match columns.iter_mut().find(|column| column.0 == *name) {
            Some(column) => column.1 = value.clone(),
            None => columns.push((name, value.clone())),
        }
    }
    let names: Vec<&str> = columns.iter().map(|column| column.0).collect();
    let values: Vec<&str> = columns.iter().map(|column| column.1.as_str()).collect();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_identifier_quarantine ({}) VALUES ({})",
        names.join(", "),
        values.join(", ")
    ))
    .execute(&mut connection)
}

/// A SQL string literal for `value`, doubling any single quote.
fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn assert_violation(result: Result<usize, DieselError>, kind: &str, constraint: &str) {
    match result {
        Err(DieselError::DatabaseError(error_kind, info)) => {
            let matches_kind = match kind {
                "check" => matches!(error_kind, DatabaseErrorKind::CheckViolation),
                "unique" => matches!(error_kind, DatabaseErrorKind::UniqueViolation),
                "foreign" => matches!(error_kind, DatabaseErrorKind::ForeignKeyViolation),
                "not_null" => matches!(error_kind, DatabaseErrorKind::NotNullViolation),
                other => panic!("unknown violation kind {other}"),
            };
            assert!(
                matches_kind,
                "expected a {kind} violation, got {error_kind:?}"
            );
            if !constraint.is_empty() {
                assert_eq!(info.constraint_name(), Some(constraint));
            }
        }
        other => panic!("expected a {kind} violation of {constraint}, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Schema contract
// ---------------------------------------------------------------------------

#[test]
fn migration_seeds_no_quarantine_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_identifier_quarantine)"),
        0,
        "MET-WP7-PREREQ-02 must neither seed nor backfill any quarantine row"
    );
}

#[test]
fn the_quarantine_table_has_exactly_the_approved_column_contract() {
    let (_guard, pool) = setup_registry_db();
    for (name, sql_type, not_null, default) in QUARANTINE_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute a \
                       LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
                      WHERE a.attrelid = 'metric_identifier_quarantine'::regclass \
                        AND a.attnum > 0 AND NOT a.attisdropped \
                        AND a.attname = '{name}' \
                        AND format_type(a.atttypid, a.atttypmod) = '{sql_type}' \
                        AND a.attnotnull = {not_null} \
                        AND pg_get_expr(d.adbin, d.adrelid) IS NOT DISTINCT FROM {})",
                    match default {
                        Some(expression) => format!("'{expression}'"),
                        None => "NULL".to_string(),
                    }
                ),
            ),
            1,
            "column `{name}` must be {sql_type}, not_null={not_null}, default={default:?}"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_attribute \
              WHERE attrelid = 'metric_identifier_quarantine'::regclass \
                AND attnum > 0 AND NOT attisdropped)",
        ),
        QUARANTINE_COLUMNS.len() as i64,
        "the quarantine table must carry exactly the approved columns and no others"
    );
    for column in FORBIDDEN_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_attribute \
                      WHERE attrelid = 'metric_identifier_quarantine'::regclass \
                        AND attname = '{column}' AND NOT attisdropped)"
                ),
            ),
            0,
            "`{column}` must not exist: quarantine stores no request identity, raw \
             evidence, classification or resolution state"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_attribute a \
              WHERE a.attrelid = 'metric_identifier_quarantine'::regclass \
                AND a.attnum > 0 AND NOT a.attisdropped \
                AND format_type(a.atttypid, a.atttypmod) IN ('json', 'jsonb', 'bytea', 'inet'))",
        ),
        0,
        "the quarantine table must carry no JSON, binary or network-address column"
    );
}

#[test]
fn the_quarantine_table_has_exactly_the_approved_constraints_and_no_doi_rule() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        check_constraint_names(&pool, "metric_identifier_quarantine"),
        vec![
            "metric_identifier_quarantine_methodology_version_check",
            "metric_identifier_quarantine_period_check",
            "metric_identifier_quarantine_schema_version_check",
            "metric_identifier_quarantine_work_doi_check",
        ],
        "exactly the three nonblank CHECKs and the period CHECK"
    );
    assert_eq!(
        index_names(&pool, "metric_identifier_quarantine"),
        vec![
            "metric_identifier_quarantine_created_id_idx",
            "metric_identifier_quarantine_pkey",
            "metric_identifier_quarantine_record_provenance_id_key",
        ],
        "the original primary/unique keys plus exactly the later MET-WP7-PREREQ-03 scheduling index"
    );
    // The provenance uniqueness covers exactly that one column.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_identifier_quarantine'::regclass \
                AND conname = 'metric_identifier_quarantine_record_provenance_id_key' \
                AND contype = 'u' \
                AND (SELECT array_agg(attname::text) FROM pg_attribute \
                      WHERE attrelid = conrelid AND attnum = ANY(conkey)) \
                    = ARRAY['record_provenance_id'])",
        ),
        1
    );
    // The work_doi CHECK is a nonblank rule only: no DOI shape is imposed.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'metric_identifier_quarantine'::regclass \
                AND contype = 'c' \
                AND (pg_get_constraintdef(oid) ILIKE '%doi.org%' \
                     OR pg_get_constraintdef(oid) LIKE '%10\\.%'))",
        ),
        0,
        "no DOI-format CHECK may exist"
    );

    let keys = foreign_keys(&pool, "metric_identifier_quarantine");
    let expected = [
        (
            "metric_identifier_quarantine_measure_id_fkey",
            "FOREIGN KEY (measure_id) REFERENCES metric_measure(measure_id)",
        ),
        (
            "metric_identifier_quarantine_platform_id_fkey",
            "FOREIGN KEY (platform_id) REFERENCES metric_platform(platform_id)",
        ),
        (
            "metric_identifier_quarantine_record_provenance_id_fkey",
            "FOREIGN KEY (record_provenance_id) REFERENCES metric_record_provenance(record_provenance_id)",
        ),
        (
            "metric_identifier_quarantine_source_account_id_fkey",
            "FOREIGN KEY (source_account_id) REFERENCES metric_source_account(source_account_id)",
        ),
    ];
    assert_eq!(
        keys,
        expected
            .iter()
            .map(|(name, definition)| (name.to_string(), definition.to_string()))
            .collect::<Vec<_>>(),
        "exactly the four approved foreign keys, none with an ON DELETE or ON UPDATE action"
    );
}

#[test]
fn database_defaults_apply_and_every_required_column_is_not_null() {
    let (_guard, pool) = setup_registry_db();
    let refs = fixture_refs(&pool);
    insert_quarantine(&pool, &refs, &[]).expect("a valid quarantine row");
    assert_eq!(
        scalar_i64(
            &pool,
            &format!(
                "(SELECT COUNT(*) FROM metric_identifier_quarantine \
                  WHERE record_provenance_id = '{}' \
                    AND identifier_quarantine_id IS NOT NULL \
                    AND created_at IS NOT NULL AND country_code IS NULL)",
                refs.record_provenance_id
            )
        ),
        1
    );
    for (name, _, not_null, _) in QUARANTINE_COLUMNS {
        if !not_null || matches!(name, "identifier_quarantine_id" | "created_at") {
            continue;
        }
        let other = fresh(&pool, &refs);
        assert_violation(
            insert_quarantine(&pool, &other, &[(name, "NULL".into())]),
            "not_null",
            "",
        );
    }
}

#[test]
fn blank_schema_versions_dois_and_methodologies_are_rejected_locale_independently() {
    let (_guard, pool) = setup_registry_db();
    let base = fixture_refs(&pool);
    for column in ["schema_version", "work_doi", "methodology_version"] {
        let constraint = format!("metric_identifier_quarantine_{column}_check");
        for blank in BLANK_VALUES {
            let refs = fresh(&pool, &base);
            assert_violation(
                insert_quarantine(&pool, &refs, &[(column, literal(blank))]),
                "check",
                &constraint,
            );
        }
        // Surrounding whitespace is not blankness: values are stored as supplied.
        let refs = fresh(&pool, &base);
        insert_quarantine(&pool, &refs, &[(column, literal("\u{3000}x "))])
            .unwrap_or_else(|error| panic!("{column}: {error}"));
    }
}

#[test]
fn the_period_must_be_ordered_and_half_open() {
    let (_guard, pool) = setup_registry_db();
    let base = fixture_refs(&pool);
    for (start, end) in [("2026-03-02", "2026-03-02"), ("2026-03-02", "2026-03-01")] {
        let refs = fresh(&pool, &base);
        assert_violation(
            insert_quarantine(
                &pool,
                &refs,
                &[
                    ("period_start", format!("DATE '{start}'")),
                    ("period_end", format!("DATE '{end}'")),
                ],
            ),
            "check",
            "metric_identifier_quarantine_period_check",
        );
    }
    let refs = fresh(&pool, &base);
    insert_quarantine(
        &pool,
        &refs,
        &[
            ("period_start", "DATE '2026-03-01'".into()),
            ("period_end", "DATE '2026-04-01'".into()),
            ("reporting_grain", "'MONTH'".into()),
            ("country_code", "'GB'".into()),
            ("value", "-9223372036854775808".into()),
        ],
    )
    .expect("a MONTH row with a country and a signed 64-bit value");
}

#[test]
fn exactly_one_quarantine_row_may_reference_one_provenance_row() {
    let (_guard, pool) = setup_registry_db();
    let refs = fixture_refs(&pool);
    insert_quarantine(&pool, &refs, &[]).expect("the first quarantine row");
    assert_violation(
        insert_quarantine(&pool, &refs, &[("value", "8".into())]),
        "unique",
        "metric_identifier_quarantine_record_provenance_id_key",
    );
    // Another provenance row of the same import may be quarantined separately.
    let second = insert_rejected_provenance(&pool, refs.import_id);
    insert_quarantine(
        &pool,
        &refs,
        &[("record_provenance_id", format!("'{second}'"))],
    )
    .expect("a second provenance row is a second quarantine row");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_identifier_quarantine)"),
        2
    );
}

#[test]
fn every_reference_must_exist_and_no_parent_deletion_cascades() {
    let (_guard, pool) = setup_registry_db();
    let base = fixture_refs(&pool);
    for (column, constraint) in [
        (
            "record_provenance_id",
            "metric_identifier_quarantine_record_provenance_id_fkey",
        ),
        (
            "source_account_id",
            "metric_identifier_quarantine_source_account_id_fkey",
        ),
        (
            "platform_id",
            "metric_identifier_quarantine_platform_id_fkey",
        ),
        ("measure_id", "metric_identifier_quarantine_measure_id_fkey"),
    ] {
        let refs = fresh(&pool, &base);
        assert_violation(
            insert_quarantine(&pool, &refs, &[(column, format!("'{}'", Uuid::new_v4()))]),
            "foreign",
            constraint,
        );
    }

    let refs = fresh(&pool, &base);
    insert_quarantine(&pool, &refs, &[]).expect("a valid quarantine row");
    // A parent with quarantine evidence cannot be deleted, and the attempt
    // leaves the evidence in place.
    for (table, id_column, id, constraint) in [
        (
            "metric_record_provenance",
            "record_provenance_id",
            refs.record_provenance_id,
            "metric_identifier_quarantine_record_provenance_id_fkey",
        ),
        (
            "metric_source_account",
            "source_account_id",
            refs.source_account_id,
            "",
        ),
        // The account and platform are also referenced by the fixture import
        // and account, so only the violation kind is asserted for them; the
        // provenance row and the measure are referenced by quarantine alone.
        ("metric_platform", "platform_id", refs.platform_id, ""),
        (
            "metric_measure",
            "measure_id",
            refs.measure_id,
            "metric_identifier_quarantine_measure_id_fkey",
        ),
    ] {
        assert_violation(
            delete_row(&pool, table, id_column, id),
            "foreign",
            constraint,
        );
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM metric_identifier_quarantine \
                      WHERE record_provenance_id = '{}')",
                    refs.record_provenance_id
                )
            ),
            1,
            "deleting {table} must not cascade to quarantine evidence"
        );
    }
}

#[test]
fn every_application_valid_doi_form_is_stored_verbatim_with_no_database_doi_rule() {
    let (_guard, pool) = setup_registry_db();
    // Each form is accepted by the application's own `Doi::from_str`, so the
    // database must accept and preserve it byte for byte.
    let forms = [
        "10.12345/unresolved",
        "https://doi.org/10.12345/Unresolved",
        "http://doi.org/10.12345/MIXED-case",
        "HTTPS://WWW.DX.DOI.ORG/10.12345/upper-prefix",
        "dx.doi.org/10.123456789/a(b)c;d<e>f+g[h]i:j/k",
        "10.\u{0661}\u{0662}\u{0663}\u{0664}/unicode-registrant-digits",
    ];
    let base = fixture_refs(&pool);
    for form in forms {
        assert!(Doi::from_str(form).is_ok(), "{form} is application-valid");
        let refs = fresh(&pool, &base);
        insert_quarantine(&pool, &refs, &[("work_doi", literal(form))])
            .unwrap_or_else(|error| panic!("{form}: {error}"));
        let mut connection = pool.get().unwrap();
        let stored: String = metric_identifier_quarantine::table
            .filter(diesel::ExpressionMethods::eq(
                metric_identifier_quarantine::record_provenance_id,
                refs.record_provenance_id,
            ))
            .select(metric_identifier_quarantine::work_doi)
            .first(&mut connection)
            .unwrap();
        assert_eq!(stored, form, "stored exactly as supplied");
    }
}

#[test]
fn quarantine_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let refs = fixture_refs(&pool);
    insert_quarantine(
        &pool,
        &refs,
        &[
            ("country_code", "'GB'".into()),
            ("value", "42".into()),
            ("work_doi", "'10.12345/Mapped'".into()),
        ],
    )
    .unwrap();
    let mut connection = pool.get().unwrap();
    let row: MetricIdentifierQuarantine = metric_identifier_quarantine::table
        .first(&mut connection)
        .expect("the row maps through the schema contract");
    assert_eq!(row.record_provenance_id, refs.record_provenance_id);
    assert_eq!(row.source_account_id, refs.source_account_id);
    assert_eq!(row.platform_id, refs.platform_id);
    assert_eq!(row.measure_id, refs.measure_id);
    assert_eq!(row.schema_version, "thoth-normalized-metrics/1");
    assert_eq!(row.work_doi, "10.12345/Mapped");
    assert_eq!(
        row.period_start,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    );
    assert_eq!(row.period_end, NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
    assert_eq!(row.reporting_grain, MetricReportingGrain::Day);
    assert_eq!(row.country_code.as_deref(), Some("GB"));
    assert_eq!(row.value, 42);
    assert_eq!(row.methodology_version, "cloudfront-title-session/2");
}

// ---------------------------------------------------------------------------
// Migration evidence
// ---------------------------------------------------------------------------

#[test]
fn the_migration_applies_from_its_exact_predecessor_and_an_empty_rollback_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let mut connection = establish();
    let full_head = schema_fingerprint(&mut connection);

    // Later approved migrations may add objects that reference the quarantine
    // table. Isolate the historical MET-WP7-PREREQ-02 head before measuring
    // this migration's own delta.
    revert_newer_than_quarantine_migration(&mut connection);
    let quarantine_head = schema_fingerprint(&mut connection);

    revert_through_quarantine_migration(&mut connection);
    assert!(!is_applied(&mut connection));
    let predecessor = schema_fingerprint(&mut connection);
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace \
               AND relname LIKE 'metric_identifier_quarantine%')",
        ),
        0,
        "the predecessor schema has no quarantine object"
    );
    // The migration is purely additive: everything it adds names the new
    // table, and it changes or removes nothing that existed before it.
    assert!(
        predecessor.is_subset(&quarantine_head),
        "the migration must not alter a predecessor object: {:?}",
        predecessor.difference(&quarantine_head).collect::<Vec<_>>()
    );
    let added: Vec<&String> = quarantine_head.difference(&predecessor).collect();
    assert!(!added.is_empty());
    assert!(
        added
            .iter()
            .all(|line| line.contains("metric_identifier_quarantine")),
        "every added object belongs to the quarantine table: {added:?}"
    );

    // Up from the exact predecessor reproduces the head schema.
    let applied = connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply the quarantine migration")
        .iter()
        .map(|version| version.to_string())
        .collect::<Vec<_>>();
    assert!(applied.contains(&MET_WP7_PREREQ_02_MIGRATION_VERSION.to_string()));
    assert_eq!(schema_fingerprint(&mut connection), full_head);

    // An empty rollback through the later migrations and the quarantine
    // migration restores the exact historical predecessor, and reapplying the
    // complete current migration set is deterministic.
    revert_through_quarantine_migration(&mut connection);
    assert_eq!(schema_fingerprint(&mut connection), predecessor);
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply the migration stack");
    assert_eq!(schema_fingerprint(&mut connection), full_head);
}

#[test]
fn a_populated_quarantine_refuses_rollback_before_dropping_anything() {
    let (_guard, pool) = setup_registry_db();
    let refs = fixture_refs(&pool);
    insert_quarantine(&pool, &refs, &[]).unwrap();
    drop(pool);

    let mut connection = establish();
    let before = schema_fingerprint(&mut connection);
    let error = try_revert_quarantine_migration(&mut connection);
    assert!(
        error.contains("metric_identifier_quarantine holds 1 row(s)"),
        "the rollback must refuse to discard quarantine evidence: {error}"
    );
    assert!(is_applied(&mut connection), "the quarantine migration stays applied");
    assert_eq!(
        on_connection(
            &mut connection,
            &format!(
                "(SELECT COUNT(*) FROM metric_identifier_quarantine \
                  WHERE record_provenance_id = '{}' AND value = 7)",
                refs.record_provenance_id
            ),
        ),
        1,
        "the refused rollback preserved the evidence row"
    );

    // The helper may have peeled later migrations before reaching the
    // quarantine rollback guard. Reapply them and prove the complete schema
    // returns byte-for-byte to the original head with the evidence intact.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to restore later migrations after refused rollback");
    assert_eq!(
        schema_fingerprint(&mut connection),
        before,
        "the refused rollback plus reapplication preserves the complete schema"
    );

    // Once the evidence is gone the same rollback succeeds, so it was the
    // guard, and nothing else, that refused.
    sql_query("DELETE FROM metric_identifier_quarantine")
        .execute(&mut connection)
        .unwrap();
    revert_through_quarantine_migration(&mut connection);
    assert!(!is_applied(&mut connection));
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply the quarantine migration");
    assert_eq!(schema_fingerprint(&mut connection), before);
}

#[test]
fn a_rollback_racing_a_concurrent_insert_waits_for_it_and_then_refuses() {
    let (_guard, pool) = setup_registry_db();
    let refs = fixture_refs(&pool);
    let before = {
        let mut connection = establish();
        schema_fingerprint(&mut connection)
    };

    // An uncommitted insert holds its row lock on the table.
    let (insert_done, wait_insert) = channel();
    let (commit, wait_commit) = channel::<()>();
    let provenance = refs.record_provenance_id;
    let account = refs.source_account_id;
    let platform = refs.platform_id;
    let measure = refs.measure_id;
    let inserter = thread::spawn(move || {
        let mut connection = establish();
        connection
            .transaction::<_, DieselError, _>(|connection| {
                sql_query(format!(
                    "INSERT INTO metric_identifier_quarantine \
                         (record_provenance_id, source_account_id, platform_id, measure_id, \
                          schema_version, work_doi, period_start, period_end, reporting_grain, \
                          value, methodology_version) \
                     VALUES ('{provenance}', '{account}', '{platform}', '{measure}', \
                             'thoth-normalized-metrics/1', '10.12345/racing', \
                             DATE '2026-03-01', DATE '2026-03-02', 'DAY', 1, 'm/1')"
                ))
                .execute(connection)?;
                insert_done.send(()).unwrap();
                wait_commit.recv().unwrap();
                Ok(())
            })
            .expect("the racing insert commits");
    });
    wait_insert.recv().unwrap();

    // The rollback starts while the insert is uncommitted: its ACCESS
    // EXCLUSIVE lock must wait rather than inspect a table that looks empty.
    let (pid_tx, pid_rx) = channel();
    let reverter = thread::spawn(move || {
        let mut connection = establish();
        let pid: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
            "pg_backend_pid()",
        ))
        .get_result(&mut connection)
        .unwrap();
        pid_tx.send(pid).unwrap();
        let error = try_revert_quarantine_migration(&mut connection);
        (error, is_applied(&mut connection))
    });
    let pid = pid_rx.recv().unwrap();
    let started = Instant::now();
    loop {
        let waiting = scalar_i64(
            &pool,
            &format!(
                "(SELECT COUNT(*) FROM pg_stat_activity \
                  WHERE pid = {pid} AND wait_event_type = 'Lock' \
                    AND query ILIKE '%LOCK TABLE public.metric_identifier_quarantine%')"
            ),
        );
        if waiting == 1 {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "the rollback never waited on the quarantine table lock"
        );
        thread::sleep(Duration::from_millis(20));
    }
    commit.send(()).unwrap();
    inserter.join().unwrap();

    let (error, still_applied) = reverter.join().unwrap();
    assert!(
        error.contains("metric_identifier_quarantine holds 1 row(s)"),
        "the rollback saw the committed row and refused: {error}"
    );
    assert!(still_applied);
    let mut connection = establish();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to restore later migrations after refused racing rollback");
    assert_eq!(schema_fingerprint(&mut connection), before);
    assert_eq!(
        on_connection(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_identifier_quarantine WHERE work_doi = '10.12345/racing')",
        ),
        1
    );
}
