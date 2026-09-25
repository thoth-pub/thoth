//! Acceptance evidence for automatic unresolved-DOI reconciliation.
//!
//! These tests use the real migrated PostgreSQL schema and the existing
//! managed-CloudFront fixture. They exercise the production coordinator,
//! canonical application authority, row transactions and lock behavior.

use std::sync::mpsc;
use std::thread;

use diesel::result::Error as DieselError;
use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, RunQueryDsl};
use uuid::Uuid;

use super::{
    reconcile_metric_identifier_quarantine, AttemptBucket,
    MetricIdentifierQuarantineReconciliationState as State,
};
use crate::db::PgPool;
use crate::model::metric_ingestion_lifecycle::tests::{setup, Fixture};
use crate::model::metric_platform::tests::scalar_i64;

const ACTOR: &str = "metrics-reconciler";
const DOI: &str = "https://doi.org/10.12345/reconcile-me";

#[derive(Clone, Copy)]
struct Evidence {
    import_id: Uuid,
    provenance_id: Uuid,
    quarantine_id: Uuid,
    account_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct LockedRow {
    #[diesel(sql_type = SqlUuid)]
    identifier_quarantine_id: Uuid,
}

fn text(pool: &PgPool, expression: &str) -> String {
    let mut connection = pool.get().expect("database connection");
    diesel::select(diesel::dsl::sql::<Text>(expression))
        .get_result(&mut connection)
        .unwrap_or_else(|error| panic!("{expression}: {error}"))
}

fn uuid_of(pool: &PgPool, expression: &str) -> Uuid {
    text(pool, expression).parse().expect("uuid expression")
}

fn state(f: &Fixture, quarantine_id: Uuid) -> String {
    text(
        &f.pool,
        &format!(
            "(SELECT state::text FROM metric_identifier_quarantine_reconciliation              WHERE identifier_quarantine_id = '{quarantine_id}')"
        ),
    )
}

fn attempt_count(f: &Fixture, quarantine_id: Uuid) -> i64 {
    scalar_i64(
        &f.pool,
        &format!(
            "(SELECT attempt_count::bigint FROM metric_identifier_quarantine_reconciliation              WHERE identifier_quarantine_id = '{quarantine_id}')"
        ),
    )
}

fn retry_seconds(f: &Fixture, quarantine_id: Uuid) -> i64 {
    scalar_i64(
        &f.pool,
        &format!(
            "(SELECT EXTRACT(EPOCH FROM (next_attempt_at - last_attempt_at))::bigint              FROM metric_identifier_quarantine_reconciliation              WHERE identifier_quarantine_id = '{quarantine_id}')"
        ),
    )
}

fn mark_due(f: &Fixture, quarantine_id: Uuid) {
    // Preserve the table's temporal invariant while making the row due for
    // the next deterministic retry inside this disposable test fixture.
    f.sql(&format!(
        "UPDATE metric_identifier_quarantine_reconciliation \
         SET first_attempt_at = LEAST(first_attempt_at, transaction_timestamp() - interval '2 seconds'), \
             last_attempt_at = transaction_timestamp() - interval '2 seconds', \
             next_attempt_at = transaction_timestamp() - interval '1 second' \
         WHERE identifier_quarantine_id = '{quarantine_id}'"
    ));
}

fn measure_id(f: &Fixture) -> Uuid {
    uuid_of(
        &f.pool,
        "(SELECT measure_id::text FROM metric_measure WHERE code = 'title_sessions')",
    )
}

fn imprint_id(f: &Fixture) -> Uuid {
    uuid_of(
        &f.pool,
        &format!(
            "(SELECT imprint_id::text FROM imprint WHERE publisher_id = '{}' ORDER BY imprint_id LIMIT 1)",
            f.publisher_id
        ),
    )
}

fn insert_work(f: &Fixture, doi: &str) -> Uuid {
    let work_id = Uuid::new_v4();
    let imprint_id = imprint_id(f);
    f.sql(&format!(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi)          VALUES ('{work_id}', 'monograph', 'forthcoming', '{imprint_id}', 1, '{doi}')"
    ));
    work_id
}

fn insert_other_publisher_work(f: &Fixture, doi: &str) {
    let publisher_id = Uuid::new_v4();
    let imprint_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    f.sql(&format!(
        "INSERT INTO publisher (publisher_id, publisher_name, subscription_package)          VALUES ('{publisher_id}', 'Other publisher', 'OBELISK')"
    ));
    f.sql(&format!(
        "INSERT INTO imprint (imprint_id, publisher_id, imprint_name)          VALUES ('{imprint_id}', '{publisher_id}', 'Other imprint')"
    ));
    f.sql(&format!(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi)          VALUES ('{work_id}', 'monograph', 'forthcoming', '{imprint_id}', 1, '{doi}')"
    ));
}

#[allow(clippy::too_many_arguments)]
fn append_evidence(
    f: &Fixture,
    import_id: Uuid,
    account_id: Uuid,
    doi: &str,
    value: i64,
    period_start: &str,
    period_end: &str,
    grain: &str,
    quarantine_created_at: &str,
) -> Evidence {
    let batch_id = Uuid::new_v4();
    let provenance_id = Uuid::new_v4();
    let quarantine_id = Uuid::new_v4();
    let measure_id = measure_id(f);

    f.sql(&format!(
        "INSERT INTO metric_import_batch (import_batch_id, import_id, batch_key, request_hash)          VALUES ('{batch_id}', '{import_id}', '{quarantine_id}', 'hash-{quarantine_id}')"
    ));
    f.sql(&format!(
        "INSERT INTO metric_record_provenance              (record_provenance_id, import_id, classification, details, import_batch_id, batch_row_index)          VALUES ('{provenance_id}', '{import_id}', 'REJECTED',                  '{{\"schema\":\"thoth-metric-provenance-details/1\",                    \"reason_code\":\"UNKNOWN_DOI\",\"reporting_grain\":\"{grain}\"}}'::jsonb,                  '{batch_id}', 0)"
    ));
    f.sql(&format!(
        "INSERT INTO metric_identifier_quarantine              (identifier_quarantine_id, record_provenance_id, source_account_id, platform_id,               measure_id, schema_version, work_doi, period_start, period_end, reporting_grain,               country_code, value, methodology_version, created_at)          VALUES ('{quarantine_id}', '{provenance_id}', '{account_id}', '{}', '{measure_id}',                  'thoth-normalized-metrics/1', '{doi}', DATE '{period_start}', DATE '{period_end}',                  '{grain}', NULL, {value}, 'cloudfront-title-session/2', TIMESTAMPTZ '{quarantine_created_at}')",
        f.platform_id
    ));

    Evidence {
        import_id,
        provenance_id,
        quarantine_id,
        account_id,
    }
}

#[allow(clippy::too_many_arguments)]
fn seed_evidence(
    f: &Fixture,
    account_id: Uuid,
    doi: &str,
    value: i64,
    import_created_at: &str,
    quarantine_created_at: &str,
    period_start: &str,
    period_end: &str,
    grain: &str,
) -> Evidence {
    let import_id = Uuid::new_v4();
    f.sql(&format!(
        "INSERT INTO metric_import              (import_id, source_account_id, publisher_id, format_code, format_version, status,               received_count, invalid_count, normalizer_version, created_by, created_at, completed_at)          VALUES ('{import_id}', '{account_id}', '{}', 'cloudfront-legacy-s3', '1',                  'COMPLETED_WITH_ERRORS', 1, 1, 'sphinx-cloudfront/1', 'fixture',                  TIMESTAMPTZ '{import_created_at}', TIMESTAMPTZ '{import_created_at}')",
        f.publisher_id
    ));
    append_evidence(
        f,
        import_id,
        account_id,
        doi,
        value,
        period_start,
        period_end,
        grain,
        quarantine_created_at,
    )
}

fn historical_snapshot(f: &Fixture, evidence: Evidence) -> String {
    text(
        &f.pool,
        &format!(
            "(SELECT jsonb_build_object(                 'import', (SELECT to_jsonb(i) FROM metric_import i WHERE import_id = '{}'),                 'provenance', (SELECT to_jsonb(p) FROM metric_record_provenance p WHERE record_provenance_id = '{}'),                 'quarantine', (SELECT to_jsonb(q) FROM metric_identifier_quarantine q WHERE identifier_quarantine_id = '{}'),                 'coverage', (SELECT COALESCE(jsonb_agg(to_jsonb(c) ORDER BY coverage_id), '[]'::jsonb)                                FROM metric_coverage c WHERE import_id = '{}'),                 'checkpoint', (SELECT COALESCE(jsonb_agg(to_jsonb(s) ORDER BY source_account_id), '[]'::jsonb)                                  FROM metric_source_checkpoint s WHERE source_account_id = '{}')              )::text)",
            evidence.import_id,
            evidence.provenance_id,
            evidence.quarantine_id,
            evidence.import_id,
            evidence.account_id
        ),
    )
}

fn reconcile(f: &Fixture, limit: i32) -> super::MetricIdentifierQuarantineReconciliationBatch {
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, limit).expect("reconcile")
}

fn set_provenance_details(f: &Fixture, evidence: Evidence, details_json: &str) {
    f.sql(&format!(
        "UPDATE metric_record_provenance \
         SET details = '{details_json}'::jsonb \
         WHERE record_provenance_id = '{}'",
        evidence.provenance_id
    ));
}

fn assert_resolved_at_is_null(f: &Fixture, quarantine_id: Uuid) {
    assert_eq!(
        scalar_i64(
            &f.pool,
            &format!(
                "(SELECT COUNT(*) FROM metric_identifier_quarantine_reconciliation \
                  WHERE identifier_quarantine_id = '{quarantine_id}' \
                    AND resolved_at IS NULL)"
            ),
        ),
        1
    );
}

fn assert_inconsistent_details_vector(name: &str, details_json: &str) {
    let (_guard, f) = setup();
    insert_work(&f, DOI);
    let evidence = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    set_provenance_details(&f, evidence, details_json);
    let historical = historical_snapshot(&f, evidence);

    let result = reconcile(&f, 1);
    assert_eq!(
        (
            result.attempted,
            result.resolved,
            result.pending,
            result.blocked
        ),
        (1, 0, 0, 1),
        "{name}"
    );
    assert_eq!(
        state(&f, evidence.quarantine_id),
        "BLOCKED_INCONSISTENT_EVIDENCE",
        "{name}"
    );
    assert_eq!(attempt_count(&f, evidence.quarantine_id), 1, "{name}");
    assert_eq!(retry_seconds(&f, evidence.quarantine_id), 3600, "{name}");
    assert_resolved_at_is_null(&f, evidence.quarantine_id);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        0,
        "{name}"
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        0,
        "{name}"
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0,
        "{name}"
    );
    assert_eq!(historical_snapshot(&f, evidence), historical, "{name}");
}

#[test]
fn closed_state_partition_is_exhaustive_and_stable() {
    let cases = [
        (State::PendingUnknownDoi, false, AttemptBucket::Pending),
        (State::BlockedAmbiguousDoi, false, AttemptBucket::Blocked),
        (
            State::BlockedPublisherScopeMismatch,
            false,
            AttemptBucket::Blocked,
        ),
        (State::BlockedSourceConflict, false, AttemptBucket::Blocked),
        (
            State::BlockedOverlappingPeriod,
            false,
            AttemptBucket::Blocked,
        ),
        (State::BlockedSameImportOrder, false, AttemptBucket::Blocked),
        (
            State::BlockedImportOrderAmbiguous,
            false,
            AttemptBucket::Blocked,
        ),
        (State::BlockedDeltaOverflow, false, AttemptBucket::Blocked),
        (
            State::BlockedInconsistentEvidence,
            false,
            AttemptBucket::Blocked,
        ),
        (State::ResolvedWinner, true, AttemptBucket::Resolved),
        (State::ResolvedDuplicate, true, AttemptBucket::Resolved),
        (State::ResolvedRevision, true, AttemptBucket::Resolved),
        (State::ResolvedSuperseded, true, AttemptBucket::Resolved),
    ];

    for (state, terminal, bucket) in cases {
        assert_eq!(state.is_terminal(), terminal);
        assert_eq!(state.bucket(), bucket);
        assert_eq!(state.to_string().parse::<State>(), Ok(state));
    }
}

#[test]
fn unresolved_rows_retry_on_the_exact_capped_schedule() {
    let (_guard, f) = setup();
    let evidence = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );

    for (index, hours) in [1_i64, 2, 4, 8, 16, 24, 24].into_iter().enumerate() {
        let result = reconcile(&f, 1);
        assert_eq!((result.attempted, result.pending), (1, 1));
        assert_eq!(state(&f, evidence.quarantine_id), "PENDING_UNKNOWN_DOI");
        assert_eq!(attempt_count(&f, evidence.quarantine_id), index as i64 + 1);
        assert_eq!(retry_seconds(&f, evidence.quarantine_id), hours * 3600);
        if index < 6 {
            mark_due(&f, evidence.quarantine_id);
        }
    }
}

#[test]
fn inconsistent_evidence_is_retryable_and_does_not_starve_later_due_work() {
    let (_guard, f) = setup();
    let first = seed_evidence(
        &f,
        f.account_a,
        "https://doi.org/10.12345/inconsistent",
        1,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    let second = seed_evidence(
        &f,
        f.account_a,
        "https://doi.org/10.12345/later",
        2,
        "2026-09-22 08:01:00+00",
        "2026-09-22 08:01:00+00",
        "2026-03-02",
        "2026-03-03",
        "DAY",
    );
    f.sql(&format!(
        "UPDATE metric_record_provenance          SET details = jsonb_set(details, '{{reason_code}}', '\"INVALID_DOI\"'::jsonb)          WHERE record_provenance_id = '{}'",
        first.provenance_id
    ));

    let result = reconcile(&f, 2);
    assert_eq!(
        (
            result.attempted,
            result.resolved,
            result.pending,
            result.blocked
        ),
        (2, 0, 1, 1)
    );
    assert_eq!(
        state(&f, first.quarantine_id),
        "BLOCKED_INCONSISTENT_EVIDENCE"
    );
    assert_eq!(state(&f, second.quarantine_id), "PENDING_UNKNOWN_DOI");
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        0
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0
    );
}

#[test]
fn malformed_provenance_details_fail_closed_before_canonical_application() {
    let cases = [
        (
            "reporting grain absent",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI"}"#,
        ),
        (
            "reporting grain null",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":null}"#,
        ),
        (
            "reporting grain non-string",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":1}"#,
        ),
        (
            "reporting grain unsupported",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":"WEEK"}"#,
        ),
        (
            "reporting grain non-canonical",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":"day"}"#,
        ),
        (
            "reporting grain mismatched",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":"MONTH"}"#,
        ),
        (
            "schema absent",
            r#"{"reason_code":"UNKNOWN_DOI","reporting_grain":"DAY"}"#,
        ),
        (
            "schema non-string",
            r#"{"schema":null,"reason_code":"UNKNOWN_DOI","reporting_grain":"DAY"}"#,
        ),
        (
            "schema wrong",
            r#"{"schema":"thoth-metric-provenance-details/2","reason_code":"UNKNOWN_DOI","reporting_grain":"DAY"}"#,
        ),
        (
            "reporting grain enum-shaped object",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":{"DAY":null}}"#,
        ),
        (
            "reason code enum-shaped object",
            r#"{"schema":"thoth-metric-provenance-details/1","reason_code":{"UNKNOWN_DOI":null},"reporting_grain":"DAY"}"#,
        ),
        (
            "details positional array",
            r#"["thoth-metric-provenance-details/1","UNKNOWN_DOI","DAY"]"#,
        ),
    ];

    for (name, details_json) in cases {
        assert_inconsistent_details_vector(name, details_json);
    }
}

#[test]
fn valid_matching_provenance_details_reach_the_exact_winner_control() {
    let (_guard, f) = setup();
    insert_work(&f, DOI);
    let evidence = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    set_provenance_details(
        &f,
        evidence,
        r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":"DAY","unrelated":{"ignored":true}}"#,
    );
    let historical = historical_snapshot(&f, evidence);

    let result = reconcile(&f, 1);
    assert_eq!(
        (
            result.attempted,
            result.resolved,
            result.pending,
            result.blocked
        ),
        (1, 1, 0, 0)
    );
    assert_eq!(state(&f, evidence.quarantine_id), "RESOLVED_WINNER");
    assert_eq!(historical_snapshot(&f, evidence), historical);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        1
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1
    );
    assert_eq!(
        scalar_i64(
            &f.pool,
            "(SELECT COUNT(*) FROM metric_rollup_delta WHERE status = 'PENDING')"
        ),
        1
    );
}

#[test]
fn reporting_grain_inconsistency_retries_exactly_and_keeps_later_due_work_live() {
    let (_guard, f) = setup();
    insert_work(&f, DOI);
    let inconsistent = seed_evidence(
        &f,
        f.account_a,
        DOI,
        1,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    let later = seed_evidence(
        &f,
        f.account_a,
        "https://doi.org/10.12345/later-strict",
        2,
        "2026-09-22 08:01:00+00",
        "2026-09-22 08:01:00+00",
        "2026-03-02",
        "2026-03-03",
        "DAY",
    );
    set_provenance_details(
        &f,
        inconsistent,
        r#"{"schema":"thoth-metric-provenance-details/1","reason_code":"UNKNOWN_DOI","reporting_grain":"MONTH"}"#,
    );
    let historical = historical_snapshot(&f, inconsistent);

    let first = reconcile(&f, 2);
    assert_eq!(
        (
            first.attempted,
            first.resolved,
            first.pending,
            first.blocked
        ),
        (2, 0, 1, 1)
    );
    assert_eq!(
        state(&f, inconsistent.quarantine_id),
        "BLOCKED_INCONSISTENT_EVIDENCE"
    );
    assert_eq!(attempt_count(&f, inconsistent.quarantine_id), 1);
    assert_eq!(retry_seconds(&f, inconsistent.quarantine_id), 3600);
    assert_resolved_at_is_null(&f, inconsistent.quarantine_id);
    assert_eq!(state(&f, later.quarantine_id), "PENDING_UNKNOWN_DOI");
    assert_eq!(attempt_count(&f, later.quarantine_id), 1);
    assert_eq!(historical_snapshot(&f, inconsistent), historical);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        0
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        0
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0
    );

    mark_due(&f, inconsistent.quarantine_id);
    let second = reconcile(&f, 1);
    assert_eq!(
        (
            second.attempted,
            second.resolved,
            second.pending,
            second.blocked
        ),
        (1, 0, 0, 1)
    );
    assert_eq!(
        state(&f, inconsistent.quarantine_id),
        "BLOCKED_INCONSISTENT_EVIDENCE"
    );
    assert_eq!(attempt_count(&f, inconsistent.quarantine_id), 2);
    assert_eq!(retry_seconds(&f, inconsistent.quarantine_id), 7200);
    assert_resolved_at_is_null(&f, inconsistent.quarantine_id);
    assert_eq!(historical_snapshot(&f, inconsistent), historical);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        0
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        0
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0
    );
}

#[test]
fn skip_locked_never_double_attempts_a_row_owned_by_another_worker() {
    let (_guard, f) = setup();
    let evidence = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );

    let pool = std::sync::Arc::clone(&f.pool);
    let quarantine_id = evidence.quarantine_id;
    let (locked_tx, locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let holder = thread::spawn(move || {
        let mut connection = pool.get().expect("holder connection");
        connection
            .transaction::<(), DieselError, _>(|connection| {
                let locked: LockedRow = sql_query(format!(
                    "SELECT identifier_quarantine_id                      FROM metric_identifier_quarantine                      WHERE identifier_quarantine_id = '{quarantine_id}' FOR UPDATE"
                ))
                .get_result(connection)?;
                assert_eq!(locked.identifier_quarantine_id, quarantine_id);
                locked_tx.send(()).expect("announce lock");
                release_rx.recv().expect("release lock");
                Ok(())
            })
            .expect("holder transaction");
    });

    locked_rx.recv().expect("row locked");
    let skipped = reconcile(&f, 1);
    assert_eq!(
        skipped.attempted, 0,
        "locked row must be skipped, not waited on"
    );
    assert_eq!(
        scalar_i64(
            &f.pool,
            "(SELECT COUNT(*) FROM metric_identifier_quarantine_reconciliation)"
        ),
        0
    );

    release_tx.send(()).expect("release");
    holder.join().expect("holder");
    let attempted = reconcile(&f, 1);
    assert_eq!((attempted.attempted, attempted.pending), (1, 1));
    assert_eq!(attempt_count(&f, evidence.quarantine_id), 1);
}

#[test]
fn unexpected_database_failure_rolls_the_row_transaction_back() {
    let (_guard, f) = setup();
    let evidence = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    let before = historical_snapshot(&f, evidence);
    f.sql(
        "ALTER TABLE metric_identifier_quarantine_reconciliation          ADD CONSTRAINT test_reconciliation_actor CHECK (last_attempted_by <> 'explode')",
    );

    assert!(
        reconcile_metric_identifier_quarantine(&f.pool, "explode", 1).is_err(),
        "unexpected database errors must fail the invocation"
    );
    assert_eq!(
        scalar_i64(
            &f.pool,
            "(SELECT COUNT(*) FROM metric_identifier_quarantine_reconciliation)"
        ),
        0,
        "failed attempt state must roll back"
    );
    assert_eq!(historical_snapshot(&f, evidence), before);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        0
    );
}

#[test]
fn winner_duplicate_and_source_conflict_share_one_canonical_authority() {
    let (_guard, f) = setup();
    insert_work(&f, DOI);

    let winner = seed_evidence(
        &f,
        f.account_a,
        DOI,
        7,
        "2026-09-22 08:00:00+00",
        "2026-09-22 08:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    let historical = historical_snapshot(&f, winner);
    let result = reconcile(&f, 1);
    assert_eq!((result.attempted, result.resolved), (1, 1));
    assert_eq!(state(&f, winner.quarantine_id), "RESOLVED_WINNER");
    assert_eq!(historical_snapshot(&f, winner), historical);
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
        1
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );

    let duplicate = seed_evidence(
        &f,
        f.account_b,
        DOI,
        7,
        "2026-09-22 09:00:00+00",
        "2026-09-22 09:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    assert_eq!(reconcile(&f, 1).resolved, 1);
    assert_eq!(state(&f, duplicate.quarantine_id), "RESOLVED_DUPLICATE");
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1,
        "duplicates create no delta"
    );

    let conflict = seed_evidence(
        &f,
        f.account_b,
        DOI,
        8,
        "2026-09-22 10:00:00+00",
        "2026-09-22 10:00:00+00",
        "2026-03-01",
        "2026-03-02",
        "DAY",
    );
    assert_eq!(reconcile(&f, 1).blocked, 1);
    assert_eq!(state(&f, conflict.quarantine_id), "BLOCKED_SOURCE_CONFLICT");
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1
    );
    assert_eq!(
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1,
        "conflicts create no delta"
    );

    let settled = (
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
    );
    assert_eq!(
        reconcile(&f, 50).attempted,
        0,
        "terminal rows never re-apply"
    );
    assert_eq!(
        (
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)")
        ),
        settled
    );
}

#[test]
fn historical_order_distinguishes_revision_superseded_same_import_and_equal_time() {
    {
        let (_guard, f) = setup();
        insert_work(&f, DOI);
        let winner = seed_evidence(
            &f,
            f.account_a,
            DOI,
            10,
            "2026-09-22 10:00:00+00",
            "2026-09-22 10:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, winner), "RESOLVED_WINNER");

        let newer = seed_evidence(
            &f,
            f.account_a,
            DOI,
            20,
            "2026-09-22 11:00:00+00",
            "2026-09-22 11:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, newer), "RESOLVED_REVISION");
        assert_eq!(
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
            2
        );

        let older = seed_evidence(
            &f,
            f.account_a,
            DOI,
            5,
            "2026-09-22 09:00:00+00",
            "2026-09-22 12:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, older), "RESOLVED_SUPERSEDED");
        assert_eq!(
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
            2,
            "superseded historical evidence creates no delta"
        );
    }

    {
        let (_guard, f) = setup();
        insert_work(&f, DOI);
        let first = seed_evidence(
            &f,
            f.account_a,
            DOI,
            10,
            "2026-09-22 10:00:00+00",
            "2026-09-22 10:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        let same_import = append_evidence(
            &f,
            first.import_id,
            f.account_a,
            DOI,
            11,
            "2026-03-01",
            "2026-03-02",
            "DAY",
            "2026-09-22 10:01:00+00",
        );
        f.sql(&format!(
            "UPDATE metric_import SET received_count = 2, invalid_count = 2 WHERE import_id = '{}'",
            first.import_id
        ));
        assert_eq!(state_after(&f, first), "RESOLVED_WINNER");
        assert_eq!(state_after(&f, same_import), "BLOCKED_SAME_IMPORT_ORDER");
    }

    {
        let (_guard, f) = setup();
        insert_work(&f, DOI);
        let first = seed_evidence(
            &f,
            f.account_a,
            DOI,
            10,
            "2026-09-22 10:00:00+00",
            "2026-09-22 10:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, first), "RESOLVED_WINNER");
        let equal_time = seed_evidence(
            &f,
            f.account_a,
            DOI,
            12,
            "2026-09-22 10:00:00+00",
            "2026-09-22 10:02:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(
            state_after(&f, equal_time),
            "BLOCKED_IMPORT_ORDER_AMBIGUOUS"
        );
    }
}

fn state_after(f: &Fixture, evidence: Evidence) -> String {
    let result = reconcile(f, 1);
    assert_eq!(result.attempted, 1);
    state(f, evidence.quarantine_id)
}

#[test]
fn publisher_overlap_and_delta_overflow_fail_closed_without_canonical_effect() {
    {
        let (_guard, f) = setup();
        insert_other_publisher_work(&f, DOI);
        let mismatch = seed_evidence(
            &f,
            f.account_a,
            DOI,
            7,
            "2026-09-22 08:00:00+00",
            "2026-09-22 08:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(
            state_after(&f, mismatch),
            "BLOCKED_PUBLISHER_SCOPE_MISMATCH"
        );
        assert_eq!(
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
            0
        );
    }

    {
        let (_guard, f) = setup();
        insert_work(&f, DOI);
        let first = seed_evidence(
            &f,
            f.account_a,
            DOI,
            3,
            "2026-09-22 08:00:00+00",
            "2026-09-22 08:00:00+00",
            "2026-01-01",
            "2026-03-01",
            "REPORTING_PERIOD",
        );
        assert_eq!(state_after(&f, first), "RESOLVED_WINNER");
        let overlapping = seed_evidence(
            &f,
            f.account_a,
            DOI,
            4,
            "2026-09-22 09:00:00+00",
            "2026-09-22 09:00:00+00",
            "2026-02-01",
            "2026-04-01",
            "REPORTING_PERIOD",
        );
        assert_eq!(state_after(&f, overlapping), "BLOCKED_OVERLAPPING_PERIOD");
        assert_eq!(
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)"),
            1
        );
    }

    {
        let (_guard, f) = setup();
        f.sql("UPDATE metric_measure SET allow_negative = TRUE WHERE code = 'title_sessions'");
        insert_work(&f, DOI);
        let first = seed_evidence(
            &f,
            f.account_a,
            DOI,
            i64::MIN,
            "2026-09-22 08:00:00+00",
            "2026-09-22 08:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, first), "RESOLVED_WINNER");
        let before = (
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
            scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        );
        let overflow = seed_evidence(
            &f,
            f.account_a,
            DOI,
            i64::MAX,
            "2026-09-22 09:00:00+00",
            "2026-09-22 09:00:00+00",
            "2026-03-01",
            "2026-03-02",
            "DAY",
        );
        assert_eq!(state_after(&f, overflow), "BLOCKED_DELTA_OVERFLOW");
        assert_eq!(
            (
                scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
                scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_rollup_delta)")
            ),
            before,
            "overflow must commit only reconciliation state"
        );
    }
}
