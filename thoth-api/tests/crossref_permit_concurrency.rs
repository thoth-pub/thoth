#![cfg(feature = "backend")]

//! `BE-06` Crossref write-permit concurrency and the integration harness reset.
//!
//! Every test runs against a real disposable PostgreSQL with both BE-06
//! migrations applied, on real pooled connections. Nothing here contacts
//! Crossref.

#[allow(dead_code)]
mod support;

use diesel::connection::SimpleConnection;
use diesel::sql_types::Text;
use diesel::{PgConnection, QueryableByName, RunQueryDsl};

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

fn text(connection: &mut PgConnection, query: &str) -> String {
    diesel::sql_query(query)
        .get_result::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("{query}: {error}"))
        .value
}

/// One row of residue in every BE-06 table of Amendment 3 section 3.3, written
/// through ordinary SQL on the protected schema.
fn seed_residue(connection: &mut PgConnection) {
    connection
        .batch_execute(
            "BEGIN; \
             INSERT INTO publisher (publisher_name) VALUES ('BE-06 H8 residue publisher'); \
             INSERT INTO crossref_write_permit \
                 (route, scope, publisher_id, publisher_identity, root_work_identity, \
                  source_generation_witness, doi_set_digest, doi_set_cardinality, \
                  crossref_timestamp, doi_batch_id) \
             SELECT 'LEGACY_SCHEDULED', 'SINGLE_ROOT_WORK', publisher_id, publisher_id, \
                    gen_random_uuid(), 0, \
                    public.crossref_doi_set_digest(ARRAY['https://doi.org/10.12345/be06-h8']), \
                    1, 20260904120000000, 'be06-h8' \
               FROM publisher; \
             INSERT INTO crossref_write_permit_doi (permit_id, doi) \
             SELECT permit_id, 'https://doi.org/10.12345/be06-h8' FROM crossref_write_permit; \
             COMMIT; \
             UPDATE work_crossref_version_floor SET floor_value = 99999999999999; \
             INSERT INTO crossref_version_floor_audit \
                 (mutation_kind, before_value, after_value, g6_attempt_id, observation_id, \
                  g7_authorization_reference, authorization_register_digest, actor) \
             VALUES ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, gen_random_uuid(), \
                     gen_random_uuid(), 'G7-AUTH-H8', repeat('a', 64), 'be06-h8'); \
             UPDATE work_upsert_control SET capture_enabled = true, execution_enabled = true \
              WHERE execution_profile = 'CROSSREF'; \
             INSERT INTO work_upsert_generation (work_id, execution_profile, source_generation) \
             VALUES (gen_random_uuid(), 'CROSSREF', 3); \
             INSERT INTO work_upsert_capture_queue (entry_kind, work_ids) \
             VALUES ('OWNERS', ARRAY[gen_random_uuid()]); \
             INSERT INTO work_upsert_admission \
                 (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
             SELECT 'CROSSREF', publisher_id, gen_random_uuid(), 'EV-H8', 'be06-h8' \
               FROM publisher;",
        )
        .expect("seed BE-06 residue");
}

fn residue_fingerprint(connection: &mut PgConnection) -> String {
    text(
        connection,
        "SELECT concat_ws('|', \
             (SELECT count(*) FROM crossref_write_permit), \
             (SELECT count(*) FROM crossref_write_permit_doi), \
             (SELECT floor_value FROM work_crossref_version_floor), \
             (SELECT count(*) FROM crossref_version_floor_audit), \
             (SELECT string_agg(execution_profile::text || ':' || capture_enabled::text \
                                || ':' || execution_enabled::text, ',') FROM work_upsert_control), \
             (SELECT count(*) FROM work_upsert_generation), \
             (SELECT count(*) FROM work_upsert_admission), \
             (SELECT count(*) FROM work_upsert_capture_queue), \
             (SELECT count(*) FROM publisher), \
             current_setting('session_replication_role'), \
             (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
                JOIN pg_namespace n ON n.oid = c.relnamespace \
               WHERE n.nspname = 'public' AND NOT t.tgisinternal AND t.tgenabled <> 'O')) AS value",
    )
}

/// Amendment 3 section 11, H8: `support::reset_db` over BE-06 residue leaves
/// H1's state — no residue, the two seed rows restored, protections active.
#[test]
fn h8_the_integration_reset_over_be06_residue_leaves_the_clean_seeded_state() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("initial reset");

    let mut connection = pool.get().expect("connection");
    seed_residue(&mut connection);
    assert_eq!(
        residue_fingerprint(&mut connection),
        "1|1|99999999999999|1|CROSSREF:true:true|1|1|1|1|origin|0"
    );

    support::reset_db(pool.as_ref()).expect("the integration reset succeeds over BE-06 residue");

    let clean = "0|0|0|0|CROSSREF:false:false|0|0|0|0|origin|0";
    assert_eq!(residue_fingerprint(&mut connection), clean);
    let mut other = pool.get().expect("another pooled connection");
    assert_eq!(residue_fingerprint(&mut other), clean);
    assert_eq!(
        diesel::sql_query("DELETE FROM work_upsert_control")
            .execute(&mut other)
            .expect_err("the control row is permanent after the reset")
            .to_string()
            .split(':')
            .next(),
        Some("WORK_UPSERT_CONTROL_ROW_IS_PERMANENT")
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// Amendment 3 section 11.1 F13: the floor advance against reservations and itself, on real sessions, waits read from
// pg_locks, no 40P01.
// ---------------------------------------------------------------------------------------------------------------------

use std::sync::Arc;

use diesel::{Connection, PgConnection as Session};
use thoth_api::db::PgPool;
use thoth_api::model::crossref_write_permit::crud as permit_crud;
use thoth_errors::ThothError;
use uuid::Uuid;

const PAUSE_NAMESPACE: i64 = 1_948_579_100;

fn session() -> Session {
    Session::establish(&support::test_db_url()).expect("session")
}

/// A one-shot pause: the first transaction firing `timing` on `table` blocks on an advisory lock held by this guard.
struct Pause {
    controller: Session,
    name: String,
    table: String,
    held: bool,
}

impl Pause {
    fn install(name: &str, timing: &str, table: &str) -> Self {
        let mut controller = session();
        controller
            .batch_execute(&format!(
                "CREATE SCHEMA IF NOT EXISTS be06_test;
                 CREATE TABLE IF NOT EXISTS be06_test.integration_pause_arm (point text PRIMARY KEY);
                 CREATE OR REPLACE FUNCTION be06_test.integration_pause() RETURNS trigger LANGUAGE plpgsql AS $$
                 DECLARE armed boolean;
                 BEGIN
                     SELECT true INTO armed FROM be06_test.integration_pause_arm WHERE point = TG_ARGV[0] FOR UPDATE SKIP LOCKED;
                     IF armed THEN
                         DELETE FROM be06_test.integration_pause_arm WHERE point = TG_ARGV[0];
                         PERFORM pg_advisory_xact_lock({PAUSE_NAMESPACE}, hashtext(TG_ARGV[0]));
                     END IF;
                     IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
                     RETURN NEW;
                 END $$;
                 SELECT pg_advisory_lock({PAUSE_NAMESPACE}, hashtext('{name}'));
                 INSERT INTO be06_test.integration_pause_arm (point) VALUES ('{name}') ON CONFLICT DO NOTHING;
                 CREATE TRIGGER be06_test_ipause_{name} {timing} ON public.{table}
                     FOR EACH ROW EXECUTE FUNCTION be06_test.integration_pause('{name}');"
            ))
            .expect("install");
        Pause {
            controller,
            name: name.to_string(),
            table: table.to_string(),
            held: true,
        }
    }

    fn release(&mut self) {
        if self.held {
            self.controller
                .batch_execute(&format!(
                    "SELECT pg_advisory_unlock({PAUSE_NAMESPACE}, hashtext('{}'))",
                    self.name
                ))
                .expect("release");
            self.held = false;
        }
    }
}

impl Drop for Pause {
    fn drop(&mut self) {
        self.release();
        let _ = self.controller.batch_execute(&format!(
            "DROP TRIGGER IF EXISTS be06_test_ipause_{n} ON public.{t}; DELETE FROM be06_test.integration_pause_arm WHERE point = '{n}';",
            n = self.name,
            t = self.table
        ));
    }
}

fn wait_until(mut condition: impl FnMut() -> bool) {
    for _ in 0..500 {
        if condition() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the expected wait was not observed");
}

fn paused(observer: &mut Session) -> bool {
    text(observer, &format!(
        "SELECT count(*)::text AS value FROM pg_locks WHERE locktype = 'advisory' AND NOT granted AND classid = {PAUSE_NAMESPACE}"
    )) == "1"
}

fn other_waits(observer: &mut Session) -> String {
    text(observer, &format!(
        "SELECT coalesce(string_agg(locktype || ':' || mode, ',' ORDER BY locktype, mode), '') AS value FROM pg_locks \
         WHERE NOT granted AND NOT (locktype = 'advisory' AND classid = {PAUSE_NAMESPACE})"
    ))
}

fn covered_root(connection: &mut Session) -> Uuid {
    let (publisher, imprint, work) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    connection
        .batch_execute(&format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{publisher}', 'F13 {publisher}');
             INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint}', '{publisher}', 'F13 {imprint}');
             INSERT INTO publisher_distribution_platform (publisher_id, platform, enabled, activation_id, enabled_at)
             VALUES ('{publisher}', 'CROSSREF', true, gen_random_uuid(), now());
             INSERT INTO work (work_id, work_type, work_status, edition, imprint_id, doi, publication_date, landing_page)
             VALUES ('{work}', 'monograph', 'active', 1, '{imprint}', 'https://doi.org/10.12345/f13-{s}', '2026-01-01', 'https://example.org/{s}');
             INSERT INTO title (work_id, locale_code, full_title, title, canonical) VALUES ('{work}', 'en', 'F13', 'F13', true);
             INSERT INTO publication (publication_type, work_id, isbn) VALUES ('Paperback', '{work}', '978-3-16-148410-0');",
            s = work.simple()
        ))
        .expect("fixture");
    work
}

fn advance(pool: &PgPool, attempt: Uuid) -> Result<i64, ThothError> {
    permit_crud::advance_crossref_version_floor(
        pool,
        &permit_crud::AdvanceCrossrefVersionFloor {
            target_value: 99_999_999_999_999,
            g6_attempt_id: attempt,
            observation_id: Uuid::new_v4(),
            authorization_reference: "G7-F13".to_string(),
            authorization_register_digest: "a".repeat(64),
        },
        "superuser",
    )
    .map(|advance| advance.after_value)
}

/// Run `first` to its pause, start `second`, observe `second` wait, release; return both and the wait transcript.
fn schedule<A: Send + 'static, B: Send + 'static>(
    pool: &Arc<PgPool>,
    mut pause: Pause,
    first: impl FnOnce(&PgPool) -> A + Send + 'static,
    second: impl FnOnce(&PgPool) -> B + Send + 'static,
) -> (A, B, String) {
    let mut observer = session();
    let p = pool.clone();
    let first = std::thread::spawn(move || first(p.as_ref()));
    wait_until(|| paused(&mut observer));
    let p = pool.clone();
    let second = std::thread::spawn(move || second(p.as_ref()));
    wait_until(|| !other_waits(&mut observer).is_empty());
    let transcript = other_waits(&mut observer);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(!second.is_finished(), "the second session waits");
    pause.release();
    (
        first.join().expect("first"),
        second.join().expect("second"),
        transcript,
    )
}

#[test]
fn f13_the_floor_advance_against_reservations_and_itself() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    let mut connection = pool.get().expect("connection");

    // A reservation holding F FOR SHARE commits while the advance waits: NOT_DRAINED.
    let root = covered_root(&mut connection);
    let (reserved, advanced, transcript) = schedule(
        &pool,
        Pause::install("f13a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, root),
        |pool| advance(pool, Uuid::new_v4()),
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(advanced, Err(ThothError::CrossrefVersionFloorNotDrained));
    assert_eq!(transcript, "transactionid:ShareLock");
    permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        reserved.permit_id,
        "f13",
        "INC-F13",
    )
    .expect("void");

    // An advance holding F FOR UPDATE commits while a reservation waits: a 17-digit allocation above the floor.
    let attempt = Uuid::new_v4();
    let root = covered_root(&mut connection);
    let (advanced, reserved, transcript) = schedule(
        &pool,
        Pause::install("f13b", "BEFORE INSERT", "crossref_version_floor_audit"),
        move |pool| advance(pool, attempt),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, root),
    );
    assert_eq!(advanced, Ok(99_999_999_999_999));
    let reserved = reserved.expect("reserved");
    assert!(reserved.crossref_timestamp > 99_999_999_999_999);
    assert_eq!(reserved.crossref_timestamp.to_string().len(), 17);
    assert_eq!(transcript, "transactionid:ShareLock");
    permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        reserved.permit_id,
        "f13",
        "INC-F13",
    )
    .expect("void");

    // Two advances with one attempt, and with different attempts, against a fresh floor.
    connection
        .batch_execute(
            "BEGIN; SET LOCAL session_replication_role = replica; \
             UPDATE work_crossref_version_floor SET floor_value = 0; DELETE FROM crossref_version_floor_audit; COMMIT;",
        )
        .expect("fresh floor");
    let attempt = Uuid::new_v4();
    let (first, second, transcript) = schedule(
        &pool,
        Pause::install("f13c", "BEFORE INSERT", "crossref_version_floor_audit"),
        move |pool| advance(pool, attempt),
        move |pool| advance(pool, attempt),
    );
    assert_eq!(first, Ok(99_999_999_999_999));
    assert_eq!(second, Err(ThothError::CrossrefVersionFloorAlreadyAdvanced));
    assert_eq!(transcript, "transactionid:ShareLock");
    connection
        .batch_execute(
            "BEGIN; SET LOCAL session_replication_role = replica; \
             UPDATE work_crossref_version_floor SET floor_value = 0; DELETE FROM crossref_version_floor_audit; COMMIT;",
        )
        .expect("fresh floor");
    let (first, second, _) = schedule(
        &pool,
        Pause::install("f13d", "BEFORE INSERT", "crossref_version_floor_audit"),
        |pool| advance(pool, Uuid::new_v4()),
        |pool| advance(pool, Uuid::new_v4()),
    );
    assert_eq!(first, Ok(99_999_999_999_999));
    assert_eq!(second, Err(ThothError::CrossrefVersionFloorBindingMismatch));
}

// ---------------------------------------------------------------------------------------------------------------------
// Amendment 3 section 10.3 X9 over the live provocations: no EB1 or EB2 error rendered for GraphQL carries database,
// driver, host, port or role text.
// ---------------------------------------------------------------------------------------------------------------------

const LEAKS: [&str; 20] = [
    "violates",
    "duplicate key",
    "constraint",
    "relation",
    "sqlstate",
    "23505",
    "23514",
    "postgres",
    "diesel",
    "127.0.0.1",
    "localhost",
    "54411",
    "pq:",
    "work_upsert_control",
    "work_upsert_admission",
    "crossref_write_permit",
    "work_crossref_version_floor",
    "crossref_version_floor_audit",
    "_check",
    "_idx",
];

/// The rendered GraphQL error: its message, and its `type` code, which must be an upper-case code with no other
/// extension.
fn rendered(error: ThothError) -> String {
    use juniper::IntoFieldError;
    let field_error: juniper::FieldError = error.into_field_error();
    let extensions = format!("{:?}", field_error.extensions());
    let code = regex::Regex::new(r#"String\("([A-Z0-9_]+)"\)"#)
        .expect("regex")
        .captures(&extensions)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| panic!("an upper-case type code: {extensions}"));
    assert_eq!(
        extensions.matches("String(").count(),
        1,
        "the type is the only extension: {extensions}"
    );
    let _ = code;
    field_error.message().to_lowercase()
}

fn assert_no_leak(context: &str, rendered: &str) {
    for leak in LEAKS {
        assert!(!rendered.contains(leak), "{context}: {leak} in {rendered}");
    }
}

#[test]
fn x9_no_live_provocation_renders_database_or_connection_text() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    let mut connection = pool.get().expect("connection");
    let publisher = Uuid::new_v4();
    connection
        .batch_execute(&format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{publisher}', 'X9')"
        ))
        .expect("publisher");
    let provocations = [
        "UPDATE work_upsert_control SET execution_enabled = true WHERE execution_profile = 'CROSSREF'".to_string(),
        format!(
            "INSERT INTO work_upsert_admission (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
             VALUES ('CROSSREF', '{publisher}', gen_random_uuid(), '   ', 'x9')"
        ),
        "UPDATE work_crossref_version_floor SET floor_value = 5".to_string(),
        "SELECT public.crossref_ts_next(99991231235959999)".to_string(),
        "DELETE FROM work_upsert_control".to_string(),
        "TRUNCATE work_upsert_admission".to_string(),
        "INSERT INTO work_upsert_admission (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
         VALUES ('CROSSREF', gen_random_uuid(), gen_random_uuid(), 'EV', 'x9')"
            .to_string(),
        "SELECT no_such_column FROM work_upsert_control".to_string(),
        "DO $$ BEGIN RAISE EXCEPTION 'SOMETHING_ELSE'; END $$".to_string(),
        "DO $$ BEGIN RAISE EXCEPTION 'CROSSREF_TIMESTAMP_OVERFLOW: 10000-01-01 00:00:00+00'; END $$".to_string(),
        "INSERT INTO crossref_version_floor_audit (mutation_kind, before_value, after_value, g6_attempt_id, observation_id, \
             g7_authorization_reference, authorization_register_digest, actor) \
         VALUES ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, gen_random_uuid(), gen_random_uuid(), '   ', repeat('a', 64), 'x9')"
            .to_string(),
    ];
    for statement in provocations {
        let mut error = None;
        let _ = connection.transaction::<(), diesel::result::Error, _>(|connection| {
            if let Err(failure) = connection.batch_execute(&statement) {
                error = Some(ThothError::from_work_upsert_database_error(failure));
            }
            Err(diesel::result::Error::RollbackTransaction)
        });
        let error = error.unwrap_or_else(|| panic!("the provocation succeeded: {statement}"));
        assert_no_leak(&statement, &rendered(error));
    }

    // EB1 through GraphQL on a pool whose connections cannot be established.
    let unreachable: PgPool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .connection_timeout(std::time::Duration::from_millis(200))
        .build_unchecked(diesel::r2d2::ConnectionManager::<Session>::new(
            "postgres://nobody_x9:secret_x9@127.0.0.1:1/nowhere_x9",
        ));
    let response = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(support::execute_graphql(
            Arc::new(unreachable),
            Some(support::superuser("x9")),
            "{ crossrefVersionFloor { floorValue } }",
            None,
        ));
    assert_eq!(
        support::first_error_type(&response),
        Some("INTERNAL_ERROR"),
        "{response}"
    );
    let text = response.to_string().to_lowercase();
    for leak in [
        "nobody_x9",
        "secret_x9",
        "nowhere_x9",
        "127.0.0.1",
        ":1/",
        "connection refused",
        "timed out",
    ] {
        assert!(!text.contains(leak), "{leak} in {text}");
    }
}
