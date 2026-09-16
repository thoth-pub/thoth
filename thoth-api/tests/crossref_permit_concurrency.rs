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
