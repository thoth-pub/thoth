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
use thoth_errors::{
    ThothError, ThothResult, WORK_UPSERT_SUFFIXED_TRIGGER_CODES, WORK_UPSERT_TRIGGER_CODES,
};
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

// ---------------------------------------------------------------------------
// Amendment 3 section 10: the BE-06 error codes and the scoped database-error
// conversion (X2, X5, X6, X7)
// ---------------------------------------------------------------------------

/// The 69 error codes of Amendment 3 section 10.4, each with its variant.
fn be06_error_codes() -> Vec<(ThothError, &'static str)> {
    vec![
        (
            ThothError::WorkUpsertProfileNotImplemented,
            "WORK_UPSERT_PROFILE_NOT_IMPLEMENTED",
        ),
        (
            ThothError::WorkUpsertExecutionProfilesRequired,
            "WORK_UPSERT_EXECUTION_PROFILES_REQUIRED",
        ),
        (
            ThothError::WorkUpsertProfileNotAdmitted,
            "WORK_UPSERT_PROFILE_NOT_ADMITTED",
        ),
        (
            ThothError::WorkUpsertExecutionNotPermitted,
            "WORK_UPSERT_EXECUTION_NOT_PERMITTED",
        ),
        (
            ThothError::WorkUpsertCompletionRequiresFence,
            "WORK_UPSERT_COMPLETION_REQUIRES_FENCE",
        ),
        (
            ThothError::WorkUpsertCompletionRequiresAcceptedPermit,
            "WORK_UPSERT_COMPLETION_REQUIRES_ACCEPTED_PERMIT",
        ),
        (
            ThothError::WorkUpsertCancellationRefusedFencedAttempt,
            "WORK_UPSERT_CANCELLATION_REFUSED_FENCED_ATTEMPT",
        ),
        (
            ThothError::WorkUpsertRecoveryBlocked,
            "WORK_UPSERT_RECOVERY_BLOCKED",
        ),
        (
            ThothError::WorkUpsertCaptureIsMonotone,
            "WORK_UPSERT_CAPTURE_IS_MONOTONE",
        ),
        (
            ThothError::WorkUpsertControlRowIsPermanent,
            "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT",
        ),
        (
            ThothError::WorkUpsertControlKeyImmutable,
            "WORK_UPSERT_CONTROL_KEY_IMMUTABLE",
        ),
        (
            ThothError::WorkUpsertAdmissionImmutable,
            "WORK_UPSERT_ADMISSION_IMMUTABLE",
        ),
        (
            ThothError::WorkUpsertAdmissionDeleteOnlyByPublisherCascade,
            "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE",
        ),
        (
            ThothError::WorkUpsertAdmissionCensusNotEmpty,
            "WORK_UPSERT_ADMISSION_CENSUS_NOT_EMPTY",
        ),
        (
            ThothError::WorkUpsertGenerationOverflow,
            "WORK_UPSERT_GENERATION_OVERFLOW",
        ),
        (
            ThothError::DistributionJobKindNotClaimable,
            "DISTRIBUTION_JOB_KIND_NOT_CLAIMABLE",
        ),
        (
            ThothError::DistributionJobWorkIdentityImmutable,
            "DISTRIBUTION_JOB_WORK_IDENTITY_IMMUTABLE",
        ),
        (
            ThothError::DistributionJobWorkReferenceNotRestorable,
            "DISTRIBUTION_JOB_WORK_REFERENCE_NOT_RESTORABLE",
        ),
        (
            ThothError::WorkDeleteBlockedByFencedAttempt,
            "WORK_DELETE_BLOCKED_BY_FENCED_ATTEMPT",
        ),
        (
            ThothError::WorkDeleteBindingDrift,
            "WORK_DELETE_BINDING_DRIFT",
        ),
        (
            ThothError::WorkDeleteBindingDriftUnresolved,
            "WORK_DELETE_BINDING_DRIFT_UNRESOLVED",
        ),
        (
            ThothError::CrossrefRootWorkNotFound,
            "CROSSREF_ROOT_WORK_NOT_FOUND",
        ),
        (
            ThothError::CrossrefPublisherNotCovered,
            "CROSSREF_PUBLISHER_NOT_COVERED",
        ),
        (
            ThothError::CrossrefBindingMovedRetry,
            "CROSSREF_BINDING_MOVED_RETRY",
        ),
        (ThothError::CrossrefPermitBlocked, "CROSSREF_PERMIT_BLOCKED"),
        (
            ThothError::CrossrefPermitEmptyDoiSet,
            "CROSSREF_PERMIT_EMPTY_DOI_SET",
        ),
        (
            ThothError::CrossrefDoiNotCanonicalisable,
            "CROSSREF_DOI_NOT_CANONICALISABLE",
        ),
        (
            ThothError::CrossrefUnitAlreadyDepositedInJob,
            "CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB",
        ),
        (
            ThothError::CrossrefManualRecoveryRequiresReference,
            "CROSSREF_MANUAL_RECOVERY_REQUIRES_REFERENCE",
        ),
        (
            ThothError::CrossrefPermitClaimStale,
            "CROSSREF_PERMIT_CLAIM_STALE",
        ),
        (
            ThothError::CrossrefPermitInitialStateInvalid,
            "CROSSREF_PERMIT_INITIAL_STATE_INVALID",
        ),
        (
            ThothError::CrossrefPermitEvidenceImmutable,
            "CROSSREF_PERMIT_EVIDENCE_IMMUTABLE",
        ),
        (
            ThothError::CrossrefPermitLinkNotRestorable,
            "CROSSREF_PERMIT_LINK_NOT_RESTORABLE",
        ),
        (
            ThothError::CrossrefPermitWriteOnceField,
            "CROSSREF_PERMIT_WRITE_ONCE_FIELD",
        ),
        (
            ThothError::CrossrefPermitIllegalTransition,
            "CROSSREF_PERMIT_ILLEGAL_TRANSITION",
        ),
        (
            ThothError::CrossrefPermitAuthorizationRequiresManifest,
            "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_MANIFEST",
        ),
        (
            ThothError::CrossrefPermitAuthorizationRequiresFence,
            "CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE",
        ),
        (
            ThothError::CrossrefArtifactSourceChanged,
            "CROSSREF_ARTIFACT_SOURCE_CHANGED",
        ),
        (
            ThothError::CrossrefPermitDeleteRefused,
            "CROSSREF_PERMIT_DELETE_REFUSED",
        ),
        (
            ThothError::CrossrefPermitMembershipImmutable,
            "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE",
        ),
        (
            ThothError::CrossrefPermitMembershipCardinalityMismatch,
            "CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH",
        ),
        (
            ThothError::CrossrefPermitNotFound,
            "CROSSREF_PERMIT_NOT_FOUND",
        ),
        (
            ThothError::CrossrefPermitRequiresReservationToken,
            "CROSSREF_PERMIT_REQUIRES_RESERVATION_TOKEN",
        ),
        (
            ThothError::CrossrefPermitVoidRequiresReserved,
            "CROSSREF_PERMIT_VOID_REQUIRES_RESERVED",
        ),
        (
            ThothError::CrossrefPermitVoidRequiresDetail,
            "CROSSREF_PERMIT_VOID_REQUIRES_DETAIL",
        ),
        (
            ThothError::CrossrefVoidRequiresAuthorizationReference,
            "CROSSREF_VOID_REQUIRES_AUTHORIZATION_REFERENCE",
        ),
        (
            ThothError::CrossrefReconciliationRequiresReference,
            "CROSSREF_RECONCILIATION_REQUIRES_REFERENCE",
        ),
        (
            ThothError::CrossrefTimestampNotIncreasing,
            "CROSSREF_TIMESTAMP_NOT_INCREASING",
        ),
        (
            ThothError::CrossrefTimestampNotDecodable,
            "CROSSREF_TIMESTAMP_NOT_DECODABLE",
        ),
        (
            ThothError::CrossrefTimestampOverflow,
            "CROSSREF_TIMESTAMP_OVERFLOW",
        ),
        (
            ThothError::CrossrefVersionFloorNotDecreasing,
            "CROSSREF_VERSION_FLOOR_NOT_DECREASING",
        ),
        (
            ThothError::CrossrefVersionFloorDomain,
            "CROSSREF_VERSION_FLOOR_DOMAIN",
        ),
        (
            ThothError::CrossrefVersionFloorNotDrained,
            "CROSSREF_VERSION_FLOOR_NOT_DRAINED",
        ),
        (
            ThothError::CrossrefVersionFloorAlreadyAdvanced,
            "CROSSREF_VERSION_FLOOR_ALREADY_ADVANCED",
        ),
        (
            ThothError::CrossrefVersionFloorPermanent,
            "CROSSREF_VERSION_FLOOR_PERMANENT",
        ),
        (
            ThothError::CrossrefVersionFloorAuditAppendOnly,
            "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY",
        ),
        (
            ThothError::AttemptHasAuthorizedPermit,
            "ATTEMPT_HAS_AUTHORIZED_PERMIT",
        ),
        (
            ThothError::AttemptHasOpenReservation,
            "ATTEMPT_HAS_OPEN_RESERVATION",
        ),
        (
            ThothError::OuterAttemptHasOpenPermits,
            "OUTER_ATTEMPT_HAS_OPEN_PERMITS",
        ),
        (
            ThothError::CrossrefVersionFloorTargetInvalid,
            "CROSSREF_VERSION_FLOOR_TARGET_INVALID",
        ),
        (
            ThothError::CrossrefVersionFloorRequiresAuthorizationReference,
            "CROSSREF_VERSION_FLOOR_REQUIRES_AUTHORIZATION_REFERENCE",
        ),
        (
            ThothError::CrossrefVersionFloorRegisterDigestInvalid,
            "CROSSREF_VERSION_FLOOR_REGISTER_DIGEST_INVALID",
        ),
        (
            ThothError::CrossrefVersionFloorBindingMismatch,
            "CROSSREF_VERSION_FLOOR_BINDING_MISMATCH",
        ),
        (
            ThothError::WorkUpsertCaptureNotEnabled,
            "WORK_UPSERT_CAPTURE_NOT_ENABLED",
        ),
        (
            ThothError::WorkUpsertAdmissionRequiresEvidenceReference,
            "WORK_UPSERT_ADMISSION_REQUIRES_EVIDENCE_REFERENCE",
        ),
        (
            ThothError::CrossrefReservationJobKindMismatch,
            "CROSSREF_RESERVATION_JOB_KIND_MISMATCH",
        ),
        (
            ThothError::CrossrefUnitPublisherMismatch,
            "CROSSREF_UNIT_PUBLISHER_MISMATCH",
        ),
        (
            ThothError::CrossrefPermitAttemptAlreadyReserved,
            "CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED",
        ),
        (
            ThothError::CrossrefPayloadDigestInvalid,
            "CROSSREF_PAYLOAD_DIGEST_INVALID",
        ),
    ]
}

#[test]
fn x0_every_be06_code_is_a_variant_with_its_own_arm_and_a_fixed_message() {
    use juniper::IntoFieldError;

    let codes = be06_error_codes();
    assert_eq!(codes.len(), 69, "Amendment 3 section 10.4 counts 69 codes");
    let distinct: BTreeSet<&str> = codes.iter().map(|(_, code)| *code).collect();
    assert_eq!(distinct.len(), 69, "no code is reused");
    let grammar = regex::Regex::new("^[A-Z][A-Z0-9_]*$").expect("grammar");
    let source = include_str!("../../../../thoth-errors/src/lib.rs");
    for (error, code) in codes {
        assert!(grammar.is_match(code) && code.len() <= 64, "{code}");
        assert!(
            source.contains(&format!("\"type\": \"{code}\"")),
            "{code} has an explicit IntoFieldError arm"
        );
        let message = error.to_string();
        assert!(
            !message.is_empty() && !message.contains('{'),
            "{code}: {message}"
        );
        let field_error = error.into_field_error();
        assert_eq!(field_error.message(), message, "{code}");
        assert_eq!(
            field_error.extensions(),
            &juniper::graphql_value!({ "type": code }),
            "{code}"
        );
    }
}

#[test]
fn x0_the_work_upsert_database_failure_is_internal_error_with_the_fixed_message() {
    use juniper::IntoFieldError;

    let error = ThothError::WorkUpsertDatabaseFailure;
    assert_eq!(
        error.to_string(),
        "A work-level distribution database operation failed."
    );
    let field_error = error.into_field_error();
    assert_eq!(
        field_error.message(),
        "A work-level distribution database operation failed."
    );
    assert_eq!(
        field_error.extensions(),
        &juniper::graphql_value!({ "type": "INTERNAL_ERROR" })
    );
}

/// The codes Migration 2 raises: every `RAISE EXCEPTION` literal that is a
/// bare code or a code with an appended value, and every `refuse_truncate`
/// trigger argument.
fn migration_raised_codes() -> (BTreeSet<String>, BTreeSet<String>) {
    let sql = migration_sql(MIGRATION_2, "up.sql");
    let bare = regex::Regex::new(r"RAISE EXCEPTION '([A-Z][A-Z0-9_]*)'").expect("bare");
    let suffixed = regex::Regex::new(r"RAISE EXCEPTION '([A-Z][A-Z0-9_]*): %'").expect("suffixed");
    let truncate =
        regex::Regex::new(r"refuse_truncate\('([A-Z][A-Z0-9_]*)'\)").expect("truncate guard");
    let mut all = BTreeSet::new();
    for pattern in [&bare, &suffixed, &truncate] {
        all.extend(pattern.captures_iter(&sql).map(|c| c[1].to_string()));
    }
    let with_suffix = suffixed
        .captures_iter(&sql)
        .map(|c| c[1].to_string())
        .collect();
    (all, with_suffix)
}

#[test]
fn x2_the_trigger_code_constants_equal_the_migration_literals() {
    let (raised, suffixed) = migration_raised_codes();
    assert_eq!(raised.len(), 29, "Migration 2 raises 29 distinct codes");
    let mut expected: BTreeSet<String> = raised.clone();
    assert!(expected.remove("WORK_UPSERT_TARGET_SET_MISMATCH"));
    let constant: BTreeSet<String> = WORK_UPSERT_TRIGGER_CODES
        .iter()
        .map(|code| code.to_string())
        .collect();
    assert_eq!(WORK_UPSERT_TRIGGER_CODES.len(), 28);
    assert_eq!(constant, expected);
    let constant_suffixed: BTreeSet<String> = WORK_UPSERT_SUFFIXED_TRIGGER_CODES
        .iter()
        .map(|code| code.to_string())
        .collect();
    assert_eq!(
        constant_suffixed,
        BTreeSet::from([
            "CROSSREF_TIMESTAMP_NOT_DECODABLE".to_string(),
            "CROSSREF_TIMESTAMP_OVERFLOW".to_string()
        ])
    );
    assert_eq!(constant_suffixed, suffixed);

    // Every trigger code is one of the 69 codes; the target-set invariant is not.
    let codes: BTreeSet<&str> = be06_error_codes().iter().map(|(_, code)| *code).collect();
    assert!(WORK_UPSERT_TRIGGER_CODES
        .iter()
        .all(|code| codes.contains(code)));
    assert!(!codes.contains("WORK_UPSERT_TARGET_SET_MISMATCH"));

    let sql = migration_sql(MIGRATION_2, "up.sql");
    for (name, _) in EXACT_CONSTRAINTS {
        assert!(sql.contains(name), "Migration 2 defines {name}");
    }
}

/// The six exact database objects of Amendment 3 section 10.3.
const EXACT_CONSTRAINTS: [(&str, &str); 6] = [
    (
        "work_upsert_control_execution_requires_capture_check",
        "WORK_UPSERT_CAPTURE_NOT_ENABLED",
    ),
    (
        "work_upsert_admission_evidence_reference_check",
        "WORK_UPSERT_ADMISSION_REQUIRES_EVIDENCE_REFERENCE",
    ),
    (
        "crossref_write_permit_payload_digest_check",
        "CROSSREF_PAYLOAD_DIGEST_INVALID",
    ),
    (
        "work_crossref_version_floor_domain_check",
        "CROSSREF_VERSION_FLOOR_DOMAIN",
    ),
    (
        "crossref_write_permit_one_per_work_upsert_attempt_idx",
        "CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED",
    ),
    (
        "crossref_version_floor_audit_one_advance_per_g6_attempt_idx",
        "CROSSREF_VERSION_FLOOR_ALREADY_ADVANCED",
    ),
];

struct DatabaseErrorDouble {
    message: &'static str,
    constraint: Option<&'static str>,
}

impl diesel::result::DatabaseErrorInformation for DatabaseErrorDouble {
    fn message(&self) -> &str {
        self.message
    }
    fn details(&self) -> Option<&str> {
        None
    }
    fn hint(&self) -> Option<&str> {
        None
    }
    fn table_name(&self) -> Option<&str> {
        None
    }
    fn column_name(&self) -> Option<&str> {
        None
    }
    fn constraint_name(&self) -> Option<&str> {
        self.constraint
    }
    fn statement_position(&self) -> Option<i32> {
        None
    }
}

fn database_error(
    kind: diesel::result::DatabaseErrorKind,
    message: &'static str,
    constraint: Option<&'static str>,
) -> diesel::result::Error {
    diesel::result::Error::DatabaseError(
        kind,
        Box::new(DatabaseErrorDouble {
            message,
            constraint,
        }),
    )
}

fn code_of(error: ThothError) -> String {
    use juniper::IntoFieldError;
    let message = error.to_string();
    let field_error = error.into_field_error();
    let code = field_error
        .extensions()
        .as_object_value()
        .and_then(|object| object.get_field_value("type"))
        .and_then(|value| value.as_string_value())
        .expect("a type extension")
        .to_string();
    assert_eq!(field_error.message(), message);
    code
}

#[test]
fn x5_the_scoped_conversion_maps_each_branch_exactly() {
    use diesel::result::DatabaseErrorKind as Kind;
    let convert = ThothError::from_work_upsert_database_error;

    // Rule 1 and 2: the exact CHECKs and unique indexes, by kind and name.
    for (name, code) in EXACT_CONSTRAINTS {
        let kind = if name.ends_with("_idx") {
            Kind::UniqueViolation
        } else {
            Kind::CheckViolation
        };
        assert_eq!(
            code_of(convert(database_error(kind, "violates", Some(name)))),
            code,
            "{name}"
        );
        // The right name under the wrong kind is not the exact object.
        let wrong = if name.ends_with("_idx") {
            Kind::CheckViolation
        } else {
            Kind::UniqueViolation
        };
        assert_eq!(
            convert(database_error(wrong, "violates", Some(name))),
            ThothError::WorkUpsertDatabaseFailure,
            "{name} under the wrong kind"
        );
    }

    // Rule 3: a trigger code, byte-equal, with no constraint name, under any kind.
    for code in WORK_UPSERT_TRIGGER_CODES {
        for kind in [Kind::Unknown, Kind::CheckViolation] {
            assert_eq!(code_of(convert(database_error(kind, code, None))), code);
        }
        // A named constraint is never a trigger code.
        assert_eq!(
            convert(database_error(Kind::Unknown, code, Some("any_constraint"))),
            ThothError::WorkUpsertDatabaseFailure
        );
    }
    // The suffix rule: accepted only for the two suffixed codes.
    assert_eq!(
        code_of(convert(database_error(
            Kind::Unknown,
            "CROSSREF_TIMESTAMP_OVERFLOW: 10000-01-01 00:00:00+00",
            None
        ))),
        "CROSSREF_TIMESTAMP_OVERFLOW"
    );
    assert_eq!(
        code_of(convert(database_error(
            Kind::Unknown,
            "CROSSREF_TIMESTAMP_NOT_DECODABLE: 20261301000000000",
            None
        ))),
        "CROSSREF_TIMESTAMP_NOT_DECODABLE"
    );
    let overflow = convert(database_error(
        Kind::Unknown,
        "CROSSREF_TIMESTAMP_OVERFLOW: 10000-01-01 00:00:00+00",
        None,
    ));
    assert!(!overflow.to_string().contains("10000"));
    for refused in [
        "CROSSREF_PERMIT_BLOCKED: x",
        "CROSSREF_TIMESTAMP_OVERFLOWX: 1",
        "CROSSREF_TIMESTAMP_OVERFLOW:1",
        "CROSSREF_TIMESTAMP_OVERFLOW ",
        " CROSSREF_PERMIT_BLOCKED",
        "crossref_permit_blocked",
        "WORK_UPSERT_TARGET_SET_MISMATCH",
        "SOMETHING_ELSE",
        "",
    ] {
        assert_eq!(
            convert(database_error(Kind::Unknown, refused, None)),
            ThothError::WorkUpsertDatabaseFailure,
            "{refused:?}"
        );
    }

    // Rule 4: everything else.
    for (kind, message, constraint) in [
        (
            Kind::CheckViolation,
            "violates",
            Some("crossref_version_floor_audit_shape_check"),
        ),
        (
            Kind::UniqueViolation,
            "duplicate key",
            Some("work_upsert_admission_pkey"),
        ),
        (
            Kind::ForeignKeyViolation,
            "violates foreign key",
            Some("work_upsert_admission_publisher_id_fkey"),
        ),
        (Kind::NotNullViolation, "null value", None),
        (Kind::SerializationFailure, "could not serialize", None),
        (Kind::ClosedConnection, "server closed the connection", None),
        (Kind::ReadOnlyTransaction, "read-only", None),
        (Kind::Unknown, "syntax error", None),
    ] {
        assert_eq!(
            convert(database_error(kind, message, constraint)),
            ThothError::WorkUpsertDatabaseFailure,
            "{message}"
        );
    }
    assert_eq!(
        convert(diesel::result::Error::NotFound),
        ThothError::WorkUpsertDatabaseFailure
    );
    assert_eq!(
        convert(diesel::result::Error::RollbackTransaction),
        ThothError::WorkUpsertDatabaseFailure
    );
    assert_eq!(
        convert(diesel::result::Error::AlreadyInTransaction),
        ThothError::WorkUpsertDatabaseFailure
    );
}

/// Run `statements` in a transaction that is always rolled back, and convert
/// the first failure through the scoped conversion.
fn provoke(connection: &mut PgConnection, statements: &str) -> ThothError {
    let mut failure = None;
    let _ = connection.transaction::<(), diesel::result::Error, _>(|connection| {
        if let Err(error) = connection.batch_execute(statements) {
            failure = Some(ThothError::from_work_upsert_database_error(error));
        }
        Err(diesel::result::Error::RollbackTransaction)
    });
    failure.unwrap_or_else(|| panic!("the provocation succeeded: {statements}"))
}

/// Run `statements` in a transaction that commits, and convert the commit's
/// failure through the scoped conversion.
fn provoke_at_commit(connection: &mut PgConnection, statements: &str) -> ThothError {
    let result = connection.transaction::<(), diesel::result::Error, _>(|connection| {
        connection.batch_execute(statements)
    });
    ThothError::from_work_upsert_database_error(
        result.expect_err("the commit is refused by a deferred trigger"),
    )
}

const PERMIT_COLUMNS: &str = "(route, scope, publisher_id, publisher_identity, root_work_identity, \
     source_generation_witness, doi_set_digest, doi_set_cardinality, crossref_timestamp, doi_batch_id)";

fn legacy_permit_values(publisher_id: Uuid, doi: &str) -> String {
    format!(
        "('LEGACY_SCHEDULED', 'SINGLE_ROOT_WORK', '{publisher_id}', '{publisher_id}', gen_random_uuid(), 0, \
          public.crossref_doi_set_digest(ARRAY['{doi}']), 1, 20260904120000000, 'x6')"
    )
}

#[test]
fn x6_each_exact_provocation_maps_to_its_code_against_a_live_connection() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());
    let mut connection = pool.get().expect("connection");
    let permit = legacy_permit_values(publisher.publisher_id, "https://doi.org/10.12345/x6");

    let cases: Vec<(String, &str)> = vec![
        (
            "UPDATE work_upsert_control SET execution_enabled = true WHERE execution_profile = 'CROSSREF'"
                .to_string(),
            "WORK_UPSERT_CAPTURE_NOT_ENABLED",
        ),
        (
            format!(
                "INSERT INTO work_upsert_admission \
                     (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
                 VALUES ('CROSSREF', '{}', gen_random_uuid(), '   ', 'x6')",
                publisher.publisher_id
            ),
            "WORK_UPSERT_ADMISSION_REQUIRES_EVIDENCE_REFERENCE",
        ),
        (
            format!(
                "ALTER TABLE crossref_write_permit DISABLE TRIGGER USER; \
                 INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES {permit}; \
                 UPDATE crossref_write_permit SET payload_digest = 'NOT-A-DIGEST'"
            ),
            "CROSSREF_PAYLOAD_DIGEST_INVALID",
        ),
        (
            "ALTER TABLE work_crossref_version_floor DISABLE TRIGGER USER; \
             UPDATE work_crossref_version_floor SET floor_value = 5"
                .to_string(),
            "CROSSREF_VERSION_FLOOR_DOMAIN",
        ),
        (
            "UPDATE work_crossref_version_floor SET floor_value = 5".to_string(),
            "CROSSREF_VERSION_FLOOR_DOMAIN",
        ),
        (
            format!(
                "ALTER TABLE crossref_write_permit DISABLE TRIGGER USER; \
                 INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES {permit}, {permit}; \
                 UPDATE crossref_write_permit SET route = 'WORK_UPSERT', permit_generation = 1, \
                        job_identity = gen_random_uuid(), \
                        attempt_identity = '00000000-0000-0000-0000-000000000006'"
            ),
            "CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED",
        ),
        (
            "INSERT INTO crossref_version_floor_audit \
                 (mutation_kind, before_value, after_value, g6_attempt_id, observation_id, \
                  g7_authorization_reference, authorization_register_digest, actor) \
             VALUES ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, '00000000-0000-0000-0000-000000000006', \
                     gen_random_uuid(), 'G7-X6', repeat('a', 64), 'x6'), \
                    ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, '00000000-0000-0000-0000-000000000006', \
                     gen_random_uuid(), 'G7-X6', repeat('a', 64), 'x6')"
                .to_string(),
            "CROSSREF_VERSION_FLOOR_ALREADY_ADVANCED",
        ),
        (
            "SELECT public.crossref_ts_next(99991231235959999)".to_string(),
            "CROSSREF_TIMESTAMP_OVERFLOW",
        ),
        (
            "DELETE FROM work_upsert_control".to_string(),
            "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT",
        ),
        (
            "TRUNCATE work_upsert_admission".to_string(),
            "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE",
        ),
        (
            "UPDATE work_crossref_version_floor SET floor_value = 99999999999999; \
             UPDATE work_crossref_version_floor SET floor_value = 0"
                .to_string(),
            "CROSSREF_VERSION_FLOOR_NOT_DECREASING",
        ),
        (
            format!(
                "INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES {permit}; \
                 DELETE FROM crossref_write_permit"
            ),
            "CROSSREF_PERMIT_DELETE_REFUSED",
        ),
    ];
    for (statements, code) in cases {
        let error = provoke(&mut connection, &statements);
        let message = error.to_string();
        assert_eq!(code_of(error), code, "{statements}");
        assert!(!message.contains("10000"), "{message}");
    }

    // A deferred trigger at COMMIT: membership disagreeing with the digest.
    let error = provoke_at_commit(
        &mut connection,
        &format!("INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES {permit}"),
    );
    assert_eq!(
        code_of(error),
        "CROSSREF_PERMIT_MEMBERSHIP_CARDINALITY_MISMATCH"
    );
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        0
    );
}

#[test]
fn x7_multi_purpose_and_unmapped_provocations_are_the_fixed_internal_failure() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());
    let imprint = test_db::create_imprint(pool.as_ref(), &publisher);
    let work = test_db::create_work(pool.as_ref(), &imprint);
    let mut connection = pool.get().expect("connection");

    let rolled_back = [
        // The multi-purpose audit shape CHECK.
        "INSERT INTO crossref_version_floor_audit \
             (mutation_kind, before_value, after_value, g6_attempt_id, observation_id, \
              g7_authorization_reference, authorization_register_digest, actor) \
         VALUES ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, gen_random_uuid(), gen_random_uuid(), \
                 '   ', repeat('a', 64), 'x7')"
            .to_string(),
        // The multi-purpose manual-route CHECK, with triggers out of the way.
        format!(
            "ALTER TABLE crossref_write_permit DISABLE TRIGGER USER; \
             INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES \
             ('MANUAL_RECOVERY', 'SINGLE_ROOT_WORK', '{0}', '{0}', gen_random_uuid(), 0, \
              public.crossref_doi_set_digest(ARRAY['https://doi.org/10.12345/x7']), 1, \
              20260904120000000, 'x7')",
            publisher.publisher_id
        ),
        // A foreign-key violation.
        "INSERT INTO work_upsert_admission \
             (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
         VALUES ('CROSSREF', gen_random_uuid(), gen_random_uuid(), 'EV-X7', 'x7')"
            .to_string(),
        // A statement error with no constraint name.
        "SELECT no_such_column FROM work_upsert_control".to_string(),
        // A RAISE outside the closed sets, bare and suffixed.
        "DO $$ BEGIN RAISE EXCEPTION 'SOMETHING_ELSE'; END $$".to_string(),
        "DO $$ BEGIN RAISE EXCEPTION 'CROSSREF_PERMIT_BLOCKED: x'; END $$".to_string(),
    ];
    for statements in rolled_back {
        let error = provoke(&mut connection, &statements);
        assert_eq!(error, ThothError::WorkUpsertDatabaseFailure, "{statements}");
        assert_eq!(code_of(error), "INTERNAL_ERROR");
    }

    // The target-set invariant at COMMIT.
    let activation = Uuid::new_v4();
    let error = provoke_at_commit(
        &mut connection,
        &format!(
            "INSERT INTO distribution_job \
                 (kind, publisher_id, work_id, activation_id, status, deduplication_key, \
                  execution_profile, work_identity, created_generation, job_ordinal) \
             VALUES ('WORK_UPSERT', '{p}', '{w}', '{a}', 'PENDING', \
                     'WORK_UPSERT:{p}:{w}:CROSSREF:{a}:1:1', 'CROSSREF', '{w}', 1, 1)",
            p = publisher.publisher_id,
            w = work.work_id,
            a = activation
        ),
    );
    assert_eq!(error, ThothError::WorkUpsertDatabaseFailure);
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM distribution_job"
        ),
        0
    );
}

// ---------------------------------------------------------------------------
// R52B section 7: the work-level execution profile registry
// ---------------------------------------------------------------------------

use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::work_upsert::policy;
use crate::model::work_upsert::registry::{self, FencedRecovery, CROSSREF_PROFILE};
use crate::model::Generation;

fn platform_position(platform: DistributionPlatform) -> usize {
    DistributionPlatform::ALL
        .iter()
        .position(|candidate| *candidate == platform)
        .expect("every platform is in ALL")
}

#[test]
fn the_registry_satisfies_the_section_7_4_invariants() {
    let registered: Vec<_> = DistributionPlatform::ALL
        .into_iter()
        .filter_map(registry::execution_profile)
        .collect();
    assert_eq!(
        registered.len(),
        1,
        "Crossref is the only implemented profile"
    );
    let mut owned = BTreeSet::new();
    for profile in &registered {
        // 1: non-empty, sorted, duplicate-free, contains its key.
        assert!(!profile.targets.is_empty());
        assert!(profile.targets.contains(&profile.key));
        let positions: Vec<usize> = profile
            .targets
            .iter()
            .copied()
            .map(platform_position)
            .collect();
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "sorted and deduplicated"
        );
        // 2: pairwise disjoint.
        for position in positions {
            assert!(owned.insert(position), "target sets are pairwise disjoint");
        }
        // 3: a recovery class and a proof reference.
        assert!(!profile.recovery_proof_reference.trim().is_empty());
        // The registry returns the profile under its own key only.
        assert_eq!(
            registry::execution_profile(profile.key).map(|found| found.key),
            Some(profile.key)
        );
    }
    for platform in DistributionPlatform::ALL {
        assert_eq!(
            registry::execution_profile(platform).is_some(),
            platform == DistributionPlatform::Crossref,
            "{platform}"
        );
    }
}

#[test]
fn the_crossref_profile_is_exactly_the_section_7_3_instance() {
    assert_eq!(CROSSREF_PROFILE.key, DistributionPlatform::Crossref);
    assert_eq!(CROSSREF_PROFILE.targets, &[DistributionPlatform::Crossref]);
    assert_eq!(
        CROSSREF_PROFILE.fenced_recovery,
        FencedRecovery::ReplayBlocked
    );
    assert_eq!(CROSSREF_PROFILE.recovery_proof_reference, "R52B §11.7");
}

#[test]
fn the_registry_match_is_exhaustive_with_no_wildcard_arm() {
    // Invariant 4 is a compile-time property; this pins the source shape that
    // provides it.
    let source = include_str!("registry.rs");
    let body = source
        .split_once("pub fn execution_profile(")
        .expect("the registry function")
        .1
        .split_once("\n}\n")
        .expect("the end of the registry function")
        .0;
    for platform in DistributionPlatform::ALL {
        let arm = format!("DistributionPlatform::{platform:?} =>");
        assert_eq!(body.matches(&arm).count(), 1, "one arm for {platform:?}");
    }
    assert!(!body.contains("_ =>"), "no wildcard arm");
    assert_eq!(body.matches("=>").count(), 17);
}

#[test]
fn the_registry_and_the_database_target_set_arms_agree() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());
    let imprint = test_db::create_imprint(pool.as_ref(), &publisher);
    let work = test_db::create_work(pool.as_ref(), &imprint);
    let mut connection = pool.get().expect("connection");

    let job = |profile: DistributionPlatform, targets: &[&str]| -> String {
        let activation = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let mut sql = format!(
            "INSERT INTO distribution_job \
                 (distribution_job_id, kind, publisher_id, work_id, activation_id, status, \
                  cancellation_reason, completed_at, deduplication_key, execution_profile, \
                  work_identity, created_generation, job_ordinal) \
             VALUES ('{job_id}', 'WORK_UPSERT', '{p}', '{w}', '{activation}', 'CANCELLED', \
                     'ADMINISTRATIVE', now(), 'WORK_UPSERT:{p}:{w}:{profile}:{activation}:1:1', \
                     '{profile}', '{w}', 1, 1);",
            p = publisher.publisher_id,
            w = work.work_id,
        );
        for target in targets {
            sql.push_str(&format!(
                "INSERT INTO distribution_job_target (distribution_job_id, platform) \
                 VALUES ('{job_id}', '{target}');"
            ));
        }
        sql
    };
    let commit_for_real = |connection: &mut PgConnection, sql: &str| {
        connection
            .transaction::<(), diesel::result::Error, _>(|connection| connection.batch_execute(sql))
            .map_err(|error| error_message(&error))
    };

    // The registered profile with exactly its declared target set commits.
    let declared: Vec<String> = CROSSREF_PROFILE
        .targets
        .iter()
        .map(|t| t.to_string())
        .collect();
    let declared: Vec<&str> = declared.iter().map(String::as_str).collect();
    assert_eq!(
        commit_for_real(
            &mut connection,
            &job(DistributionPlatform::Crossref, &declared)
        ),
        Ok(())
    );
    // Missing and extra targets are refused at COMMIT.
    for targets in [vec![], vec!["CROSSREF", "ZENODO"], vec!["ZENODO"]] {
        let refused = commit_for_real(
            &mut connection,
            &job(DistributionPlatform::Crossref, &targets),
        )
        .expect_err("refused at COMMIT");
        assert_eq!(refused, "WORK_UPSERT_TARGET_SET_MISMATCH", "{targets:?}");
    }
    // A duplicate target cannot even be written.
    assert!(commit_for_real(
        &mut connection,
        &job(DistributionPlatform::Crossref, &["CROSSREF", "CROSSREF"])
    )
    .is_err());
    // Every unregistered key is refused at COMMIT.
    for platform in DistributionPlatform::ALL {
        if registry::execution_profile(platform).is_some() {
            continue;
        }
        let name = platform.to_string();
        let refused = commit_for_real(&mut connection, &job(platform, &[name.as_str()]))
            .expect_err("an unregistered profile is refused");
        assert_eq!(refused, "WORK_UPSERT_PROFILE_NOT_IMPLEMENTED", "{platform}");
    }
}

// ---------------------------------------------------------------------------
// Amendment 3 section 7: input validation, code-owned
// ---------------------------------------------------------------------------

#[test]
fn n3_the_blank_rule_is_exactly_the_84_code_points_of_section_7_1() {
    let blank: Vec<u32> = (0..=0x10FFFFu32)
        .filter_map(char::from_u32)
        .filter(|c| policy::is_blank(&c.to_string()))
        .map(u32::from)
        .collect();
    let mut expected: Vec<u32> = (0x00..=0x1F).chain(0x7F..=0x9F).collect();
    expected.extend([0x20, 0xA0, 0x1680, 0x2028, 0x2029, 0x202F, 0x205F, 0x3000]);
    expected.extend(0x2000..=0x200A);
    expected.sort_unstable();
    expected.dedup();
    assert_eq!(expected.len(), 84);
    assert_eq!(blank, expected);
}

#[test]
fn n1_n2_n4_blank_and_non_blank_strings_follow_the_rule() {
    for blank in [
        "",
        " ",
        "\t\n\r\u{0b}\u{0c}",
        "\u{0085}",
        "\u{00A0}",
        "\u{1680}",
        "\u{2003}",
        "\u{2028}",
        "\u{202F}",
        "\u{205F}",
        "\u{3000}",
        "\u{001C}",
        "\u{001F}",
        "\u{0001}",
        "\u{007F}",
        "\u{009F}",
        "\u{00A0}\u{2003}\u{3000}",
    ] {
        assert!(policy::is_blank(blank), "{blank:?} is blank");
    }
    for content in [
        "x",
        " x ",
        "\u{00A0}x",
        "\u{200B}",
        "\u{FEFF}",
        "\u{180E}",
        " G7-AUTH-1 ",
    ] {
        assert!(!policy::is_blank(content), "{content:?} is content");
    }
}

#[test]
fn the_sha256_hex_shape_accepts_only_64_lower_case_hex_bytes() {
    let valid_a = "0123456789abcdef".repeat(4);
    let valid_b = "f".repeat(64);
    for valid in [&valid_a, &valid_b] {
        assert!(policy::is_sha256_lower_hex(valid), "{valid}");
    }
    let mut invalid = vec![
        String::new(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        format!("{}g", "a".repeat(63)),
        format!("{} ", "a".repeat(63)),
        format!("{}\u{FF11}", "a".repeat(63)),
        format!("{}é", "a".repeat(62)),
        format!("{}F", "a".repeat(63)),
    ];
    invalid.push(format!(" {}", "a".repeat(63)));
    for value in invalid {
        assert!(!policy::is_sha256_lower_hex(&value), "{value:?}");
    }
}

#[test]
fn execution_profile_lists_are_validated_without_database_access() {
    use thoth_errors::ThothError;
    assert_eq!(
        policy::registered_profiles(&[]).map(|profiles| profiles.len()),
        Err(ThothError::WorkUpsertExecutionProfilesRequired)
    );
    assert_eq!(
        policy::registered_profiles(&[
            DistributionPlatform::Crossref,
            DistributionPlatform::Zenodo
        ])
        .map(|profiles| profiles.len()),
        Err(ThothError::WorkUpsertProfileNotImplemented)
    );
    let profiles = policy::registered_profiles(&[
        DistributionPlatform::Crossref,
        DistributionPlatform::Crossref,
    ])
    .expect("registered");
    assert_eq!(profiles.len(), 1, "deduplicated");
    assert_eq!(profiles[0].key, DistributionPlatform::Crossref);
}

#[test]
fn limits_follow_the_released_clamp_convention() {
    assert_eq!(policy::clamp_limit(None, 100, 500), 100);
    assert_eq!(policy::clamp_limit(Some(0), 100, 500), 0);
    assert_eq!(policy::clamp_limit(Some(-5), 100, 500), 0);
    assert_eq!(policy::clamp_limit(Some(1), 100, 500), 1);
    assert_eq!(policy::clamp_limit(Some(501), 100, 500), 500);
    assert_eq!(policy::clamp_limit(Some(10_000), 100, 500), 500);
}

// ---------------------------------------------------------------------------
// R52B section 19.3: the Generation wire type
// ---------------------------------------------------------------------------

#[test]
fn generation_is_exactly_the_lossless_decimal_grammar() {
    use std::str::FromStr;
    for (text, value) in [
        ("0", 0i64),
        ("1", 1),
        ("99999999999999", 99_999_999_999_999),
        ("20260914120000000", 20_260_914_120_000_000),
        ("9223372036854775807", i64::MAX),
    ] {
        let generation = Generation::from_str(text).expect(text);
        assert_eq!(generation.value(), value);
        assert_eq!(generation.to_string(), text);
        assert_eq!(Generation::from_i64(value), Some(generation));
    }
    for text in [
        "",
        "-1",
        "+1",
        "01",
        "00",
        " 1",
        "1 ",
        "1.0",
        "1e3",
        "abc",
        "0x1",
        "９",
        "9223372036854775808",
        "99999999999999999999",
    ] {
        assert!(Generation::from_str(text).is_err(), "{text:?}");
    }
    assert_eq!(Generation::from_i64(-1), None);

    use juniper::{DefaultScalarValue, FromInputValue, InputValue};
    let parse = |value: InputValue<DefaultScalarValue>| Generation::from_input_value(&value);
    assert_eq!(
        parse(InputValue::scalar("99999999999999")).map(|g| g.value()),
        Ok(99_999_999_999_999)
    );
    assert!(parse(InputValue::scalar("01")).is_err());
    assert!(parse(InputValue::scalar(1)).is_err(), "never an Int");
    let output: InputValue<DefaultScalarValue> =
        juniper::ToInputValue::to_input_value(&Generation::from_i64(7).expect("7"));
    assert_eq!(output, InputValue::scalar("7"), "a string on the wire");
}

// ---------------------------------------------------------------------------
// Amendment 3 section 10.3: the controlled error boundary (X1, X3, X10)
// ---------------------------------------------------------------------------

use crate::model::work_upsert::{
    work_upsert_transaction, WorkUpsertQueryResultExt, WorkUpsertTxError,
};

/// The five BE-06 model files of Amendment 3 section 10.3's X1 and X3.
const BE06_MODEL_FILES: [&str; 5] = [
    "src/model/work_upsert/mod.rs",
    "src/model/work_upsert/crud.rs",
    "src/model/work_upsert/policy.rs",
    "src/model/crossref_write_permit/mod.rs",
    "src/model/crossref_write_permit/crud.rs",
];

fn source(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn x1_the_boundary_holds_the_only_transaction_and_pool_acquisition() {
    let mut transactions = 0;
    let mut acquisitions = 0;
    for path in BE06_MODEL_FILES {
        let text = source(path);
        transactions += text.matches(".transaction(").count();
        acquisitions += text.matches("db.get()").count();
    }
    assert_eq!(
        transactions, 1,
        "`.transaction(` occurs once, in the boundary"
    );
    assert_eq!(acquisitions, 1, "`db.get()` occurs once, in the boundary");
    let boundary = source("src/model/work_upsert/mod.rs");
    let body = boundary
        .split_once("pub(crate) fn work_upsert_transaction<")
        .expect("the boundary")
        .1
        .split_once("\n}\n")
        .expect("its end")
        .0;
    assert!(body.contains(".transaction("));
    assert!(body.contains("db.get()"));
}

#[test]
fn x3_no_be06_model_file_uses_a_released_conversion() {
    for path in BE06_MODEL_FILES {
        let text = source(path);
        for forbidden in [
            "map_err(Into::into)",
            "ThothError::from(",
            "impl From<diesel",
        ] {
            assert!(!text.contains(forbidden), "{path} contains {forbidden}");
        }
    }
}

#[test]
fn x2_the_scoped_conversion_is_referenced_once_outside_tests() {
    let mut references = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src"
    ))];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).expect("read src") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name == "tests.rs" || name.ends_with("_tests.rs") || !name.ends_with(".rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read");
            for _ in text.matches("from_work_upsert_database_error") {
                references.push(path.display().to_string());
            }
            if text.contains("PgConnection::establish") {
                assert!(
                    path.ends_with("src/db.rs"),
                    "X10: PgConnection::establish outside db.rs in {}",
                    path.display()
                );
            }
        }
    }
    assert_eq!(references.len(), 1, "{references:?}");
    assert!(references[0].ends_with("src/model/work_upsert/mod.rs"));
}

#[test]
fn x10_a_pool_that_cannot_connect_is_the_fixed_internal_failure() {
    let pool = test_db::failing_pool();
    let result = work_upsert_transaction(&pool, |_connection| Ok::<_, WorkUpsertTxError>(()));
    assert_eq!(result, Err(ThothError::WorkUpsertDatabaseFailure));
    let message = result.expect_err("refused").to_string();
    for leak in [
        "invalid",
        "localhost",
        "5432",
        ":1",
        "connection",
        "timed out",
    ] {
        assert!(!message.contains(leak), "{message}");
    }
}

#[test]
fn x10_a_pool_whose_only_connection_is_held_is_the_fixed_internal_failure() {
    let _guard = test_db::test_lock();
    let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(test_db::test_db_url());
    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .connection_timeout(std::time::Duration::from_millis(200))
        .build(manager)
        .expect("pool");
    let _held = pool.get().expect("the only connection");
    let result = work_upsert_transaction(&pool, |_connection| Ok::<_, WorkUpsertTxError>(()));
    assert_eq!(result, Err(ThothError::WorkUpsertDatabaseFailure));
    assert_eq!(
        result.expect_err("refused").to_string(),
        "A work-level distribution database operation failed."
    );
}

#[test]
fn the_boundary_commits_rolls_back_and_converts_exactly() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(pool.as_ref());

    // Commit, at PostgreSQL's default READ COMMITTED.
    let isolation = work_upsert_transaction(pool.as_ref(), |connection| {
        insert_admission(connection, publisher.publisher_id);
        Ok(texts(
            connection,
            "SELECT current_setting('transaction_isolation') AS value",
        ))
    })
    .expect("committed");
    assert_eq!(isolation, vec!["read committed"]);
    let mut connection = pool.get().expect("connection");
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM work_upsert_admission"
        ),
        1
    );

    // A ThothError from the closure passes through unchanged and rolls back.
    let result: ThothResult<()> = work_upsert_transaction(pool.as_ref(), |connection| {
        connection.batch_execute(
            "UPDATE work_upsert_control SET capture_enabled = true WHERE execution_profile = 'CROSSREF'",
        )?;
        Err(ThothError::CrossrefPermitNotFound.into())
    });
    assert_eq!(result, Err(ThothError::CrossrefPermitNotFound));
    assert_eq!(
        texts(
            &mut connection,
            "SELECT capture_enabled::text AS value FROM work_upsert_control"
        ),
        vec!["false"]
    );

    // A statement error inside the closure is converted by the scoped conversion.
    let result: ThothResult<()> = work_upsert_transaction(pool.as_ref(), |connection| {
        connection.batch_execute(
            "UPDATE work_upsert_control SET execution_enabled = true WHERE execution_profile = 'CROSSREF'",
        )?;
        Ok(())
    });
    assert_eq!(result, Err(ThothError::WorkUpsertCaptureNotEnabled));

    // A deferred trigger failing at COMMIT is converted too.
    let result: ThothResult<()> = work_upsert_transaction(pool.as_ref(), |connection| {
        connection.batch_execute(&format!(
            "INSERT INTO crossref_write_permit {PERMIT_COLUMNS} VALUES {}",
            legacy_permit_values(publisher.publisher_id, "https://doi.org/10.12345/boundary")
        ))?;
        Ok(())
    });
    assert_eq!(
        result,
        Err(ThothError::CrossrefPermitMembershipCardinalityMismatch)
    );
    assert_eq!(
        count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        0
    );

    // An unmapped error is the fixed internal failure, never the database text.
    let result: ThothResult<()> = work_upsert_transaction(pool.as_ref(), |connection| {
        connection.batch_execute("SELECT no_such_column FROM work_upsert_control")?;
        Ok(())
    });
    assert_eq!(result, Err(ThothError::WorkUpsertDatabaseFailure));

    // The EB2 extension applies the same conversion.
    let mapped = diesel::sql_query("DELETE FROM work_upsert_control")
        .execute(&mut connection)
        .work_upsert();
    assert_eq!(mapped, Err(ThothError::WorkUpsertControlRowIsPermanent));
    let unmapped = diesel::sql_query("SELECT no_such_column FROM work_upsert_control")
        .execute(&mut connection)
        .work_upsert();
    assert_eq!(unmapped, Err(ThothError::WorkUpsertDatabaseFailure));
}
