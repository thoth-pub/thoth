//! `BE-06` generic substrate evidence: migration and object inventory (G), the
//! R-8 permanence guards (U), and the substrate operations added in later units.
//!
//! Every database test runs against a real disposable PostgreSQL. Migration
//! tests use their own throwaway database and never touch the shared test
//! database's schema.

use std::collections::BTreeSet;

use diesel::connection::SimpleConnection;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use diesel::{Connection, PgConnection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use crate::db::MIGRATIONS;
use crate::model::tests::db as test_db;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

fn texts(connection: &mut PgConnection, sql: &str) -> Vec<String> {
    diesel::sql_query(sql)
        .load::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("query `{sql}` failed: {error}"))
        .into_iter()
        .map(|row| row.value)
        .collect()
}

fn count(connection: &mut PgConnection, sql: &str) -> i64 {
    diesel::sql_query(sql)
        .get_result::<CountRow>(connection)
        .unwrap_or_else(|error| panic!("query `{sql}` failed: {error}"))
        .count
}

fn error_message(error: &diesel::result::Error) -> String {
    match error {
        diesel::result::Error::DatabaseError(_, info) => info.message().to_string(),
        other => other.to_string(),
    }
}

/// Run `sql` in a transaction that is always rolled back and return the error
/// it raised. Panics if the statement succeeded.
fn refusal(connection: &mut PgConnection, sql: &str) -> String {
    let outcome = connection.transaction::<(), diesel::result::Error, _>(|connection| {
        connection.batch_execute(sql)?;
        Err(diesel::result::Error::RollbackTransaction)
    });
    match outcome {
        Err(diesel::result::Error::RollbackTransaction) => {
            panic!("`{sql}` was expected to be refused but succeeded")
        }
        Err(error) => error_message(&error),
        Ok(()) => unreachable!("the closure never commits"),
    }
}

// ---------------------------------------------------------------------------
// Throwaway migration databases
// ---------------------------------------------------------------------------

/// The last released migration, after which BE-06's two migrations apply.
const RELEASED_LAST_VERSION: &str = "20260814";
const MIGRATION_1: &str = "20260910_v1.10.0";
const MIGRATION_2: &str = "20260911_v1.10.0";

struct TempMigrationDb {
    admin_url: String,
    name: String,
}

impl TempMigrationDb {
    fn new() -> Self {
        let admin_url = test_db::test_db_url();
        let name = format!("thoth_be06_{}", Uuid::new_v4().simple());
        let mut admin = PgConnection::establish(&admin_url).expect("admin connection");
        admin
            .batch_execute(&format!(
                "CREATE DATABASE \"{name}\" WITH ENCODING 'UTF8' TEMPLATE template0"
            ))
            .expect("create temp db");
        TempMigrationDb { admin_url, name }
    }

    fn conn(&self) -> PgConnection {
        let (prefix, _) = self.admin_url.rsplit_once('/').expect("db url has a path");
        PgConnection::establish(&format!("{prefix}/{}", self.name)).expect("temp db connection")
    }
}

impl Drop for TempMigrationDb {
    fn drop(&mut self) {
        if let Ok(mut admin) = PgConnection::establish(&self.admin_url) {
            let _ = admin.batch_execute(&format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
                 WHERE datname = '{}' AND pid <> pg_backend_pid()",
                self.name
            ));
            let _ = admin.batch_execute(&format!("DROP DATABASE IF EXISTS \"{}\"", self.name));
        }
    }
}

/// Apply released migrations one at a time until the last released one has
/// run, leaving BE-06's two migrations pending.
fn migrate_to_released(connection: &mut PgConnection) {
    loop {
        let version = connection
            .run_next_migration(MIGRATIONS)
            .expect("run the next released migration");
        if version.to_string() == RELEASED_LAST_VERSION {
            return;
        }
        assert!(
            version.to_string().as_str() < RELEASED_LAST_VERSION,
            "a BE-06 migration ran before the released baseline was complete"
        );
    }
}

fn migration_sql(directory: &str, file: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/migrations/{directory}/{file}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|error| panic!("read {directory}/{file}: {error}"))
}

/// Every schema object by name, class-qualified, with its definition where the
/// definition is what a rollback must restore.
fn catalog(connection: &mut PgConnection) -> BTreeSet<String> {
    texts(
        connection,
        "SELECT 'table:' || tablename AS value FROM pg_tables WHERE schemaname = 'public' \
         UNION ALL SELECT 'column:' || table_name || '.' || column_name || ':' || data_type || ':' \
                  || is_nullable || ':' || coalesce(column_default, '') \
           FROM information_schema.columns WHERE table_schema = 'public' \
         UNION ALL SELECT 'constraint:' || conrelid::regclass::text || '.' || conname || ':' \
                  || pg_get_constraintdef(c.oid) \
           FROM pg_constraint c JOIN pg_namespace n ON n.oid = c.connamespace WHERE n.nspname = 'public' \
         UNION ALL SELECT 'index:' || indexname || ':' || indexdef FROM pg_indexes WHERE schemaname = 'public' \
         UNION ALL SELECT 'trigger:' || c.relname || '.' || t.tgname || ':' || pg_get_triggerdef(t.oid) \
           FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace \
          WHERE n.nspname = 'public' AND NOT t.tgisinternal \
         UNION ALL SELECT 'function:' || p.oid::regprocedure::text \
           FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'public' \
         UNION ALL SELECT 'enum:' || t.typname \
           FROM pg_type t JOIN pg_namespace n ON n.oid = t.typnamespace \
          WHERE n.nspname = 'public' AND t.typtype = 'e' \
         UNION ALL SELECT 'label:' || t.typname || '.' || e.enumlabel \
           FROM pg_enum e JOIN pg_type t ON t.oid = e.enumtypid JOIN pg_namespace n ON n.oid = t.typnamespace \
          WHERE n.nspname = 'public' \
         UNION ALL SELECT 'sequence:' || sequencename FROM pg_sequences WHERE schemaname = 'public'",
    )
    .into_iter()
    .collect()
}

struct Inventory {
    tables: i64,
    user_triggers: i64,
    indexes: i64,
    functions: i64,
    enum_types: i64,
    partial_indexes: i64,
}

fn inventory(connection: &mut PgConnection) -> Inventory {
    Inventory {
        tables: count(
            connection,
            "SELECT count(*) AS count FROM pg_tables WHERE schemaname = 'public'",
        ),
        user_triggers: count(
            connection,
            "SELECT count(*) AS count FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = 'public' AND NOT t.tgisinternal",
        ),
        indexes: count(
            connection,
            "SELECT count(*) AS count FROM pg_indexes WHERE schemaname = 'public'",
        ),
        functions: count(
            connection,
            "SELECT count(*) AS count FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace \
             WHERE n.nspname = 'public'",
        ),
        enum_types: count(
            connection,
            "SELECT count(*) AS count FROM pg_type t JOIN pg_namespace n ON n.oid = t.typnamespace \
             WHERE n.nspname = 'public' AND t.typtype = 'e'",
        ),
        partial_indexes: count(
            connection,
            "SELECT count(*) AS count FROM pg_indexes WHERE schemaname = 'public' \
             AND indexdef LIKE '% WHERE %'",
        ),
    }
}

/// The exact BE-06 trigger manifest (Amendment 3 section 3.3): 24 generic and
/// 11 Crossref profile triggers, as `table.trigger`.
const BE06_TRIGGERS: [&str; 35] = [
    // capture (16)
    "abstract.work_upsert_capture",
    "affiliation.work_upsert_capture",
    "contribution.work_upsert_capture",
    "contributor.work_upsert_capture",
    "funding.work_upsert_capture",
    "imprint.work_upsert_capture",
    "institution.work_upsert_capture",
    "issue.work_upsert_capture",
    "location.work_upsert_capture",
    "publication.work_upsert_capture",
    "publisher.work_upsert_capture",
    "reference.work_upsert_capture",
    "series.work_upsert_capture",
    "title.work_upsert_capture",
    "work.work_upsert_capture",
    "work_relation.work_upsert_capture",
    // flush (1)
    "work_upsert_capture_queue.work_upsert_capture_flush",
    // target set (2)
    "distribution_job.work_upsert_target_set_job",
    "distribution_job_target.work_upsert_target_set_target",
    // job guard (1)
    "distribution_job.distribution_job_work_reference_guard",
    // control (2)
    "work_upsert_control.work_upsert_control_guard",
    "work_upsert_control.work_upsert_control_no_truncate",
    // admission (2)
    "work_upsert_admission.work_upsert_admission_guard",
    "work_upsert_admission.work_upsert_admission_no_truncate",
    // permit (4)
    "crossref_write_permit.crossref_permit_membership_agreement_p",
    "crossref_write_permit.crossref_write_permit_fsm",
    "crossref_write_permit.crossref_write_permit_insert_guard",
    "crossref_write_permit.crossref_write_permit_no_truncate",
    // membership (3)
    "crossref_write_permit_doi.crossref_permit_membership_agreement_d",
    "crossref_write_permit_doi.crossref_write_permit_doi_immutable",
    "crossref_write_permit_doi.crossref_write_permit_doi_no_truncate",
    // floor (2)
    "work_crossref_version_floor.work_crossref_version_floor_guard",
    "work_crossref_version_floor.work_crossref_version_floor_no_truncate",
    // audit (2)
    "crossref_version_floor_audit.crossref_version_floor_audit_append_only",
    "crossref_version_floor_audit.crossref_version_floor_audit_no_truncate",
];

const BE06_TABLES: [&str; 8] = [
    "crossref_version_floor_audit",
    "crossref_write_permit",
    "crossref_write_permit_doi",
    "work_crossref_version_floor",
    "work_upsert_admission",
    "work_upsert_capture_queue",
    "work_upsert_control",
    "work_upsert_generation",
];

const BE06_FUNCTIONS: [&str; 27] = [
    "crossref_allocate_timestamp(bigint,bigint,bigint)",
    "crossref_blocking_write_permit_count()",
    "crossref_canonical_doi(text)",
    "crossref_deposit_membership(uuid)",
    "crossref_doi_set_digest(text[])",
    "crossref_is_blocking_write_permit(crossref_write_permit_state,crossref_reconciliation_state)",
    "crossref_is_drained()",
    "crossref_permit_membership_agreement()",
    "crossref_refuse_truncate()",
    "crossref_roots(uuid)",
    "crossref_ts_decode(bigint)",
    "crossref_ts_encode(timestamp with time zone)",
    "crossref_ts_next(bigint)",
    "crossref_ts_now()",
    "crossref_version_floor_audit_append_only()",
    "crossref_write_permit_doi_immutable()",
    "crossref_write_permit_fsm()",
    "crossref_write_permit_insert_guard()",
    "distribution_job_work_reference_guard()",
    "work_crossref_version_floor_guard()",
    "work_upsert_admission_guard()",
    "work_upsert_capture()",
    "work_upsert_capture_flush()",
    "work_upsert_control_guard()",
    "work_upsert_refuse_truncate()",
    "work_upsert_resolution(uuid,distribution_platform)",
    "work_upsert_target_set_check()",
];

/// The five new enum types with their exact members, in declaration order.
const BE06_ENUMS: [(&str, &[&str]); 5] = [
    (
        "crossref_reconciliation_state",
        &[
            "RECONCILED",
            "RECONCILIATION_REQUIRED",
            "RECONCILIATION_IMPOSSIBLE",
        ],
    ),
    (
        "crossref_void_reason",
        &[
            "SOURCE_CHANGED_DURING_PREPARATION",
            "DOI_MEMBERSHIP_CHANGED",
            "ARTIFACT_DOI_SET_MISMATCH",
            "ARTIFACT_BATCH_ID_MISMATCH",
            "ARTIFACT_TIMESTAMP_MISMATCH",
            "EXECUTION_NOT_PERMITTED",
            "PROFILE_NOT_ADMITTED",
            "INELIGIBLE",
            "BINDING_SUPERSEDED",
            "ASSIGNMENT_DISABLED",
            "NO_WORK",
            "OWNER_ABANDONED",
            "OPERATOR_CLEANUP",
        ],
    ),
    (
        "crossref_write_permit_state",
        &[
            "RESERVED",
            "AUTHORIZED",
            "INDETERMINATE",
            "ACCEPTED",
            "NONE_ATTEMPTED",
            "VOIDED",
        ],
    ),
    (
        "crossref_write_route",
        &[
            "WORK_UPSERT",
            "PUBLISHER_BACK_CATALOGUE",
            "LEGACY_SCHEDULED",
            "MANUAL_RECOVERY",
        ],
    ),
    ("crossref_write_scope", &["SINGLE_ROOT_WORK"]),
];

fn be06_names(
    snapshot: &BTreeSet<String>,
    baseline: &BTreeSet<String>,
    class: &str,
) -> Vec<String> {
    snapshot
        .difference(baseline)
        .filter_map(|entry| entry.strip_prefix(&format!("{class}:")))
        .map(|rest| rest.split(':').next().unwrap_or(rest).to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// G: migration and inventory (Amendment 3 section 11.2; R52B section 25.15)
// ---------------------------------------------------------------------------

#[test]
fn g1_up_m1_up_m2_down_m2_up_m2_down_m2_restores_the_released_catalog_exactly() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    migrate_to_released(&mut connection);
    let released = catalog(&mut connection);

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("up M1, up M2");
    let migrated = catalog(&mut connection);

    let reverted = connection
        .revert_last_migration(MIGRATIONS)
        .expect("down M2");
    assert_eq!(reverted.to_string(), "20260911");
    let after_first_down = catalog(&mut connection);

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("up M2 again");
    assert_eq!(
        catalog(&mut connection),
        migrated,
        "re-applying Migration 2 reproduces the migrated catalog exactly"
    );

    let reverted = connection
        .revert_last_migration(MIGRATIONS)
        .expect("down M2 again");
    assert_eq!(reverted.to_string(), "20260911");
    let after_second_down = catalog(&mut connection);
    assert_eq!(after_first_down, after_second_down);

    let labels: BTreeSet<String> = [
        "label:distribution_job_cancellation_reason.BINDING_SUPERSEDED",
        "label:distribution_job_cancellation_reason.WORK_DELETED",
        "label:distribution_job_kind.WORK_UPSERT",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let added: BTreeSet<String> = after_first_down.difference(&released).cloned().collect();
    let removed: BTreeSet<String> = released.difference(&after_first_down).cloned().collect();
    assert_eq!(
        added, labels,
        "after down M2 the catalog differs from the released one by exactly Migration 1's three labels"
    );
    assert!(
        removed.is_empty(),
        "down M2 removed a released object: {removed:?}"
    );
    assert!(released.iter().any(|entry| entry
        .starts_with("constraint:distribution_job.distribution_job_work_id_fkey:")
        && entry.ends_with("ON DELETE CASCADE")));
    assert!(migrated.iter().any(|entry| entry
        .starts_with("constraint:distribution_job.distribution_job_work_id_fkey:")
        && entry.ends_with("ON DELETE SET NULL")));
}

#[test]
fn g2_the_migrated_and_released_inventories_are_exactly_the_specified_counts() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    migrate_to_released(&mut connection);
    let released = inventory(&mut connection);
    assert_eq!(
        (
            released.tables,
            released.user_triggers,
            released.indexes,
            released.functions,
            released.enum_types,
            released.partial_indexes
        ),
        (60, 57, 173, 30, 28, 20)
    );

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("up M1, up M2");
    let migrated = inventory(&mut connection);
    assert_eq!(
        (
            migrated.tables,
            migrated.user_triggers,
            migrated.indexes,
            migrated.functions,
            migrated.enum_types,
            migrated.partial_indexes
        ),
        (68, 92, 191, 57, 33, 24)
    );
}

#[test]
fn g3_the_be06_trigger_set_is_exactly_the_35_name_manifest() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    migrate_to_released(&mut connection);
    let released = catalog(&mut connection);
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("up M1, up M2");
    let migrated = catalog(&mut connection);

    let mut triggers = be06_names(&migrated, &released, "trigger");
    triggers.sort();
    let mut manifest: Vec<String> = BE06_TRIGGERS.iter().map(|name| name.to_string()).collect();
    manifest.sort();
    assert_eq!(triggers, manifest);

    // The reset manifest of Amendment 3 section 5.2 is a 15-member subset of it.
    let reset_manifest = [
        "crossref_write_permit.crossref_write_permit_insert_guard",
        "crossref_write_permit.crossref_write_permit_fsm",
        "crossref_write_permit.crossref_write_permit_no_truncate",
        "crossref_write_permit.crossref_permit_membership_agreement_p",
        "crossref_write_permit_doi.crossref_write_permit_doi_immutable",
        "crossref_write_permit_doi.crossref_write_permit_doi_no_truncate",
        "crossref_write_permit_doi.crossref_permit_membership_agreement_d",
        "work_crossref_version_floor.work_crossref_version_floor_guard",
        "work_crossref_version_floor.work_crossref_version_floor_no_truncate",
        "crossref_version_floor_audit.crossref_version_floor_audit_append_only",
        "crossref_version_floor_audit.crossref_version_floor_audit_no_truncate",
        "work_upsert_control.work_upsert_control_guard",
        "work_upsert_control.work_upsert_control_no_truncate",
        "work_upsert_admission.work_upsert_admission_guard",
        "work_upsert_admission.work_upsert_admission_no_truncate",
    ];
    assert_eq!(reset_manifest.len(), 15);
    for trigger in reset_manifest {
        assert!(BE06_TRIGGERS.contains(&trigger), "{trigger}");
        assert!(
            test_db::TEST_RESET_SQL.contains(&format!(
                "('{}', '{}')",
                trigger.split_once('.').unwrap().0,
                trigger.split_once('.').unwrap().1
            )),
            "the reset statement names {trigger}"
        );
    }
}

#[test]
fn g4_the_be06_tables_enums_and_functions_exist_and_no_removed_object_does() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    migrate_to_released(&mut connection);
    let released = catalog(&mut connection);
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("up M1, up M2");
    let migrated = catalog(&mut connection);

    let mut tables = be06_names(&migrated, &released, "table");
    tables.sort();
    assert_eq!(tables, BE06_TABLES);

    let mut functions = be06_names(&migrated, &released, "function");
    functions.sort();
    let mut expected_functions: Vec<String> =
        BE06_FUNCTIONS.iter().map(|name| name.to_string()).collect();
    expected_functions.sort();
    assert_eq!(functions, expected_functions);

    let mut enums = be06_names(&migrated, &released, "enum");
    enums.sort();
    let expected_enums: Vec<String> = BE06_ENUMS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    assert_eq!(enums, expected_enums);
    for (name, members) in BE06_ENUMS {
        assert_eq!(
            texts(
                &mut connection,
                &format!(
                    "SELECT e.enumlabel AS value FROM pg_enum e JOIN pg_type t ON t.oid = e.enumtypid \
                     WHERE t.typname = '{name}' ORDER BY e.enumsortorder"
                )
            ),
            members.to_vec(),
            "{name}"
        );
    }

    // The objects the CTO R-3 decision removed (Amendment 3 section 3.1).
    for removed in [
        "work_crossref_baseline",
        "crossref_baseline_residual_class",
        "PARSER_FAILURES",
        "DELETED_WORK_IDENTITY",
        "DESTROYED_RELATION_TOPOLOGY",
        "UNKNOWN_UNQUANTIFIED",
        "residual_count",
    ] {
        assert!(
            migrated.iter().all(|entry| !entry.contains(removed)),
            "the migrated catalog contains the removed object {removed}"
        );
        for directory in [MIGRATION_1, MIGRATION_2] {
            for file in ["up.sql", "down.sql"] {
                assert!(
                    !migration_sql(directory, file).contains(removed),
                    "{directory}/{file} names the removed object {removed}"
                );
            }
        }
    }
}

#[test]
fn g5_the_two_migrations_in_one_transaction_fail_with_unsafe_use_of_new_value() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    migrate_to_released(&mut connection);
    let migration_1 = migration_sql(MIGRATION_1, "up.sql");
    let migration_2 = migration_sql(MIGRATION_2, "up.sql");

    let outcome = connection.transaction::<(), diesel::result::Error, _>(|connection| {
        connection.batch_execute(&migration_1)?;
        connection.batch_execute(&migration_2)?;
        Err(diesel::result::Error::RollbackTransaction)
    });
    let message = match outcome {
        Err(diesel::result::Error::RollbackTransaction) => {
            panic!("the single-transaction variant must fail")
        }
        Err(error) => error_message(&error),
        Ok(()) => unreachable!(),
    };
    assert!(
        message.contains("unsafe use of new value"),
        "expected `unsafe use of new value`, got: {message}"
    );
}

/// Diesel's version for a migration directory: the text before the first
/// underscore, with `-` removed (`migrations_internals::version_from_string`).
fn diesel_version(directory: &str) -> String {
    directory
        .split_once('_')
        .map(|(version, _)| version)
        .unwrap_or(directory)
        .replace('-', "")
}

#[test]
fn g6_the_two_migration_directories_are_exact_and_every_diesel_version_is_distinct() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"));
    let mut directories: Vec<String> = std::fs::read_dir(root)
        .expect("migrations directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    directories.sort();

    for directory in [MIGRATION_1, MIGRATION_2] {
        assert!(
            directories.iter().any(|name| name == directory),
            "thoth-api/migrations/{directory} exists"
        );
        for file in ["up.sql", "down.sql"] {
            assert!(
                root.join(directory).join(file).is_file(),
                "{directory}/{file}"
            );
        }
    }

    let mut versions = BTreeSet::new();
    for directory in &directories {
        let version = diesel_version(directory);
        assert!(
            versions.insert(version.clone()),
            "two directories share the Diesel version {version}"
        );
        if version == "20260910" || version == "20260911" {
            assert!(
                directory == MIGRATION_1 || directory == MIGRATION_2,
                "{directory} consumes a BE-06 reserved Diesel version"
            );
        } else {
            assert!(
                version.as_str() <= RELEASED_LAST_VERSION,
                "{directory} is a migration directory BE-06 did not add and the released tree does not contain"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// U: the R-8 permanence guards (Amendment 3 sections 3.2 and 11.2)
// ---------------------------------------------------------------------------

fn insert_admission(connection: &mut PgConnection, publisher_id: Uuid) {
    diesel::sql_query(
        "INSERT INTO work_upsert_admission \
             (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
         VALUES ('CROSSREF', $1, gen_random_uuid(), 'EV-U', 'u-actor')",
    )
    .bind::<SqlUuid, _>(publisher_id)
    .execute(connection)
    .expect("insert an admission row");
}

#[test]
fn u1_truncating_the_control_table_is_refused_with_and_without_cascade() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    for statement in [
        "TRUNCATE work_upsert_control",
        "TRUNCATE work_upsert_control CASCADE",
    ] {
        let message = refusal(&mut connection, statement);
        assert!(
            message.contains("WORK_UPSERT_CONTROL_ROW_IS_PERMANENT"),
            "{statement}: {message}"
        );
    }
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM work_upsert_control"
        ),
        1
    );
}

#[test]
fn u2_truncating_the_admission_table_or_the_publisher_table_is_refused() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());
    let mut connection = pool.get().expect("connection");
    insert_admission(&mut connection, publisher.publisher_id);

    let message = refusal(&mut connection, "TRUNCATE work_upsert_admission");
    assert!(
        message.contains("WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE"),
        "{message}"
    );
    let message = refusal(&mut connection, "TRUNCATE publisher CASCADE");
    assert!(
        message.contains("WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE")
            || message.contains("CROSSREF_PERMIT_DELETE_REFUSED"),
        "a publisher TRUNCATE CASCADE reaches a guarded table and is refused: {message}"
    );
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM work_upsert_admission"
        ),
        1
    );
}

#[test]
fn u3_deleting_a_publisher_cascades_only_its_admission_rows_and_nulls_its_permit_links() {
    let (_guard, pool) = test_db::setup_test_db();
    let doomed = test_db::create_publisher(pool.as_ref());
    let survivor = test_db::create_publisher(pool.as_ref());
    let mut connection = pool.get().expect("connection");
    insert_admission(&mut connection, doomed.publisher_id);
    insert_admission(&mut connection, survivor.publisher_id);
    connection
        .transaction::<(), diesel::result::Error, _>(|connection| {
            diesel::sql_query(
                "INSERT INTO crossref_write_permit \
                     (route, scope, publisher_id, publisher_identity, root_work_identity, \
                      source_generation_witness, doi_set_digest, doi_set_cardinality, \
                      crossref_timestamp, doi_batch_id) \
                 VALUES ('LEGACY_SCHEDULED', 'SINGLE_ROOT_WORK', $1, $1, gen_random_uuid(), 0, \
                         public.crossref_doi_set_digest(ARRAY['https://doi.org/10.12345/u3']), \
                         1, 20260904120000000, 'u3')",
            )
            .bind::<SqlUuid, _>(doomed.publisher_id)
            .execute(connection)?;
            connection.batch_execute(
                "INSERT INTO crossref_write_permit_doi (permit_id, doi) \
                 SELECT permit_id, 'https://doi.org/10.12345/u3' FROM crossref_write_permit",
            )
        })
        .expect("a permit bound to the doomed publisher");

    diesel::sql_query("DELETE FROM publisher WHERE publisher_id = $1")
        .bind::<SqlUuid, _>(doomed.publisher_id)
        .execute(&mut connection)
        .expect("a direct publisher deletion commits");

    assert_eq!(
        texts(
            &mut connection,
            "SELECT publisher_id::text AS value FROM work_upsert_admission"
        ),
        vec![survivor.publisher_id.to_string()],
        "only the deleted publisher's admission rows cascade"
    );
    assert_eq!(
        texts(
            &mut connection,
            "SELECT coalesce(publisher_id::text, 'NULL') || '|' || publisher_identity::text || '|' \
                 || state::text AS value FROM crossref_write_permit"
        ),
        vec![format!("NULL|{}|RESERVED", doomed.publisher_id)],
        "the permit survives with its link NULL and its identity kept"
    );
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit_doi"
        ),
        1
    );
    // A direct delete while the publisher exists is still refused.
    let message = refusal(&mut connection, "DELETE FROM work_upsert_admission");
    assert!(
        message.contains("WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE"),
        "{message}"
    );
}

#[test]
fn u4_the_two_guards_are_before_truncate_statement_triggers_on_exactly_their_tables() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    // tgtype bits: 1 row-level, 2 BEFORE, 32 TRUNCATE.
    assert_eq!(
        texts(
            &mut connection,
            "SELECT c.relname || '.' || t.tgname || ':' || (t.tgtype & 1)::text || ':' \
                 || (t.tgtype & 2)::text || ':' || (t.tgtype & 32)::text AS value \
               FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
               JOIN pg_proc p ON p.oid = t.tgfoid \
              WHERE p.proname = 'work_upsert_refuse_truncate' \
              ORDER BY c.relname, t.tgname"
        ),
        vec![
            "work_upsert_admission.work_upsert_admission_no_truncate:0:2:32",
            "work_upsert_control.work_upsert_control_no_truncate:0:2:32",
        ]
    );
}

#[test]
fn u5_the_test_reset_still_truncates_the_control_and_admission_tables() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());
    let mut connection = pool.get().expect("connection");
    insert_admission(&mut connection, publisher.publisher_id);
    connection
        .batch_execute(
            "UPDATE work_upsert_control SET capture_enabled = true WHERE execution_profile = 'CROSSREF'",
        )
        .expect("enable capture");

    test_db::reset_db(pool.as_ref()).expect("reset");
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM work_upsert_admission"
        ),
        0
    );
    assert_eq!(
        texts(
            &mut connection,
            "SELECT capture_enabled::text || execution_enabled::text AS value FROM work_upsert_control"
        ),
        vec!["falsefalse"]
    );
}
