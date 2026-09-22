//! PostgreSQL acceptance evidence for unresolved-DOI quarantine reconciliation.
//!
//! These tests use the existing disposable managed-DRIVER fixture and the real
//! ingestion coordinator to create immutable UNKNOWN_DOI quarantine evidence.
//! Reconciliation itself always runs through the production coordinator.

use std::sync::{Arc, Barrier};
use std::thread;

use chrono::NaiveDate;
use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, QueryableByName, RunQueryDsl};
use uuid::Uuid;

use super::{
    reconcile_metric_identifier_quarantine, AttemptBucket,
    MetricIdentifierQuarantineReconciliationState as State,
};
use crate::model::metric_import::MetricImportStatus;
use crate::model::metric_ingestion_lifecycle::tests::{
    begin_input, complete_day, day, observation, setup, Fixture,
};
use crate::model::metric_ingestion_lifecycle::{
    begin_metric_import, complete_metric_import, ingest_metric_batch_under_claim,
    CompleteMetricImportInput, IngestMetricBatchInput, NormalizedMetricObservationInput,
};

const ACTOR: &str = "metrics-ingest-service-reconciliation-test";
const DOI: &str = "https://doi.org/10.12345/reconciliation-target";

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    identifier_quarantine_id: Uuid,
}

#[derive(QueryableByName)]
struct StateRow {
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Integer)]
    attempt_count: i32,
    #[diesel(sql_type = BigInt)]
    retry_seconds: i64,
}

fn scalar_i64(f: &Fixture, expression: &str) -> i64 {
    let mut connection = f.pool.get().expect("connection");
    diesel::select(diesel::dsl::sql::<BigInt>(expression))
        .get_result(&mut connection)
        .expect("scalar")
}

fn scalar_text(f: &Fixture, expression: &str) -> String {
    let mut connection = f.pool.get().expect("connection");
    diesel::select(diesel::dsl::sql::<Text>(expression))
        .get_result(&mut connection)
        .expect("scalar")
}

fn reconciliation_state(f: &Fixture, id: Uuid) -> Option<String> {
    let mut connection = f.pool.get().expect("connection");
    sql_query(
        "SELECT state::text AS state FROM metric_identifier_quarantine_reconciliation WHERE identifier_quarantine_id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<OnlyState>(&mut connection)
    .optional()
    .expect("state")
    .map(|row| row.state)
}

#[derive(QueryableByName)]
struct OnlyState {
    #[diesel(sql_type = Text)]
    state: String,
}

use diesel::OptionalExtension;

fn retry_row(f: &Fixture, id: Uuid) -> StateRow {
    let mut connection = f.pool.get().expect("connection");
    sql_query(
        "SELECT state::text AS state, attempt_count,          EXTRACT(EPOCH FROM (next_attempt_at - last_attempt_at))::bigint AS retry_seconds          FROM metric_identifier_quarantine_reconciliation WHERE identifier_quarantine_id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result(&mut connection)
    .expect("retry row")
}

fn history_snapshot(f: &Fixture, import_id: Uuid) -> String {
    scalar_text(
        f,
        &format!(
            "(SELECT concat_ws('|',                 (SELECT row_to_json(i)::text FROM metric_import i WHERE import_id = '{import_id}'),                 (SELECT COALESCE(string_agg(row_to_json(p)::text, ';' ORDER BY p.record_provenance_id), '')                    FROM metric_record_provenance p WHERE p.import_id = '{import_id}'),                 (SELECT COALESCE(string_agg(row_to_json(q)::text, ';' ORDER BY q.identifier_quarantine_id), '')                    FROM metric_identifier_quarantine q JOIN metric_record_provenance p USING (record_provenance_id)                   WHERE p.import_id = '{import_id}'),                 (SELECT COALESCE(string_agg(row_to_json(c)::text, ';' ORDER BY c.coverage_id), '')                    FROM metric_coverage c WHERE c.import_id = '{import_id}'),                 (SELECT row_to_json(s)::text FROM metric_source_checkpoint s                   WHERE s.source_account_id = (SELECT source_account_id FROM metric_import WHERE import_id = '{import_id}'))             ))"
        ),
    )
}

struct Seed {
    import_id: Uuid,
    quarantine_ids: Vec<Uuid>,
}

fn seed(f: &Fixture, doi: &str, start: NaiveDate, values: &[&str]) -> Seed {
    let claim = f.claim_a();
    let upstream = format!("reconcile-{}", Uuid::new_v4());
    let input = crate::model::metric_ingestion_lifecycle::BeginMetricImportInput {
        expected_batch_keys: vec!["only".into()],
        ..begin_input(claim.lease_token, &upstream, start)
    };
    let import = begin_metric_import(&f.pool, ACTOR, &input).expect("begin");
    let observations = values
        .iter()
        .map(|value| NormalizedMetricObservationInput {
            work_doi: doi.into(),
            source_record_id: None,
            source_row_number: None,
            ..observation(value, start)
        })
        .collect();
    ingest_metric_batch_under_claim(
        &f.pool,
        &IngestMetricBatchInput {
            import_id: import.import_id,
            lease_token: claim.lease_token,
            batch_key: "only".into(),
            schema_version: "thoth-normalized-metrics/1".into(),
            observations,
            coverage: vec![complete_day(start)],
        },
    )
    .expect("ingest");
    let import = complete_metric_import(
        &f.pool,
        &CompleteMetricImportInput {
            import_id: import.import_id,
            lease_token: claim.lease_token,
        },
    )
    .expect("complete");
    assert_eq!(import.status, MetricImportStatus::CompletedWithErrors);
    // Completion intentionally keeps the producer lease. Test fixtures need
    // further historical imports, so release only the disposable checkpoint
    // lease without recording progress.
    f.sql(&format!(
        "UPDATE metric_source_checkpoint SET lease_owner = NULL, lease_expires_at = NULL WHERE source_account_id = '{}'",
        f.account_a
    ));

    let mut connection = f.pool.get().expect("connection");
    let rows: Vec<IdRow> = sql_query(
        "SELECT q.identifier_quarantine_id          FROM metric_identifier_quarantine q          JOIN metric_record_provenance p USING (record_provenance_id)          WHERE p.import_id = $1          ORDER BY q.created_at, q.identifier_quarantine_id",
    )
    .bind::<SqlUuid, _>(import.import_id)
    .load(&mut connection)
    .expect("quarantine ids");
    assert_eq!(rows.len(), values.len());

    Seed {
        import_id: import.import_id,
        quarantine_ids: rows
            .into_iter()
            .map(|row| row.identifier_quarantine_id)
            .collect(),
    }
}

fn make_work(f: &Fixture, doi: &str) -> Uuid {
    let work_id = Uuid::new_v4();
    f.sql(&format!(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi)          SELECT '{work_id}', 'monograph', 'forthcoming', imprint_id, 1, '{doi}'          FROM imprint WHERE publisher_id = '{}' ORDER BY imprint_id LIMIT 1",
        f.publisher_id
    ));
    work_id
}

fn make_foreign_work(f: &Fixture, doi: &str) -> Uuid {
    let publisher = Uuid::new_v4();
    let imprint = Uuid::new_v4();
    let work = Uuid::new_v4();
    f.sql(&format!(
        "INSERT INTO publisher (publisher_id, publisher_name, subscription_package)          VALUES ('{publisher}', 'Foreign reconciliation publisher', 'OBELISK');          INSERT INTO imprint (imprint_id, publisher_id, imprint_name)          VALUES ('{imprint}', '{publisher}', 'Foreign reconciliation imprint');          INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi)          VALUES ('{work}', 'monograph', 'forthcoming', '{imprint}', 1, '{doi}')"
    ));
    work
}

fn make_due(f: &Fixture, id: Uuid) {
    f.sql(&format!(
        "UPDATE metric_identifier_quarantine_reconciliation          SET next_attempt_at = transaction_timestamp() - interval '1 second'          WHERE identifier_quarantine_id = '{id}'"
    ));
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
fn unresolved_rows_follow_the_exact_capped_retry_schedule() {
    let (_guard, f) = setup();
    let seeded = seed(&f, DOI, day(1), &["1"]);
    let id = seeded.quarantine_ids[0];

    for (attempt, expected_seconds) in [
        (1, 3600),
        (2, 7200),
        (3, 14400),
        (4, 28800),
        (5, 57600),
        (6, 86400),
        (7, 86400),
    ] {
        if attempt > 1 {
            make_due(&f, id);
        }
        let batch =
            reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("reconcile pending");
        assert_eq!((batch.attempted, batch.pending), (1, 1));
        let row = retry_row(&f, id);
        assert_eq!(row.state, "PENDING_UNKNOWN_DOI");
        assert_eq!(row.attempt_count, attempt);
        assert_eq!(row.retry_seconds, expected_seconds);
    }
}

#[test]
fn newly_resolvable_winner_is_terminal_creates_one_delta_and_preserves_history() {
    let (_guard, f) = setup();
    let seeded = seed(&f, DOI, day(1), &["10"]);
    let id = seeded.quarantine_ids[0];
    let before = history_snapshot(&f, seeded.import_id);
    make_work(&f, DOI);

    let batch = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("winner");
    assert_eq!((batch.attempted, batch.resolved), (1, 1));
    assert_eq!(
        reconciliation_state(&f, id).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(scalar_i64(&f, "(SELECT COUNT(*) FROM metric_record)"), 1);
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
    assert_eq!(history_snapshot(&f, seeded.import_id), before);

    let replay = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("terminal skip");
    assert_eq!(replay.attempted, 0, "terminal state can never re-apply");
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
}

#[test]
fn identical_other_source_is_duplicate_but_different_content_is_source_conflict() {
    let (_guard, f) = setup();
    let first = seed(&f, DOI, day(1), &["10"]);
    let duplicate = seed(&f, DOI, day(1), &["10"]);
    let conflict = seed(&f, DOI, day(1), &["11"]);

    for seeded in [&duplicate, &conflict] {
        f.sql(&format!(
            "UPDATE metric_import SET source_account_id = '{}' WHERE import_id = '{}';              UPDATE metric_identifier_quarantine q SET source_account_id = '{}'              FROM metric_record_provenance p              WHERE q.record_provenance_id = p.record_provenance_id AND p.import_id = '{}'",
            f.account_b, seeded.import_id, f.account_b, seeded.import_id
        ));
    }
    make_work(&f, DOI);

    let batch = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 3).expect("three outcomes");
    assert_eq!((batch.attempted, batch.resolved, batch.blocked), (3, 2, 1));
    assert_eq!(
        reconciliation_state(&f, first.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(
        reconciliation_state(&f, duplicate.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_DUPLICATE")
    );
    assert_eq!(
        reconciliation_state(&f, conflict.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_SOURCE_CONFLICT")
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
}

#[test]
fn historical_order_distinguishes_revision_superseded_equal_time_and_same_import() {
    let (_guard, f) = setup();
    let older = seed(&f, DOI, day(1), &["10"]);
    let newer = seed(&f, DOI, day(1), &["20"]);
    f.sql(&format!(
        "UPDATE metric_import SET created_at = '2026-01-01 00:00:00+00' WHERE import_id = '{}';          UPDATE metric_import SET created_at = '2026-01-02 00:00:00+00' WHERE import_id = '{}'",
        older.import_id, newer.import_id
    ));
    make_work(&f, DOI);
    let batch = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 2).expect("revision");
    assert_eq!(batch.resolved, 2);
    assert_eq!(
        reconciliation_state(&f, older.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(
        reconciliation_state(&f, newer.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_REVISION")
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        2
    );

    let (_guard, f) = setup();
    let old = seed(&f, DOI, day(1), &["10"]);
    let current = seed(&f, DOI, day(1), &["20"]);
    let equal = seed(&f, DOI, day(1), &["30"]);
    f.sql(&format!(
        "UPDATE metric_import SET created_at = '2026-01-01 00:00:00+00' WHERE import_id = '{}';          UPDATE metric_import SET created_at = '2026-01-03 00:00:00+00' WHERE import_id = '{}';          UPDATE metric_import SET created_at = '2026-01-03 00:00:00+00' WHERE import_id = '{}';          UPDATE metric_identifier_quarantine SET created_at = '2026-01-02 00:00:00+00' WHERE identifier_quarantine_id = '{}';          UPDATE metric_identifier_quarantine SET created_at = '2026-01-01 00:00:00+00' WHERE identifier_quarantine_id = '{}';          UPDATE metric_identifier_quarantine SET created_at = '2026-01-03 00:00:00+00' WHERE identifier_quarantine_id = '{}'",
        old.import_id,
        current.import_id,
        equal.import_id,
        old.quarantine_ids[0],
        current.quarantine_ids[0],
        equal.quarantine_ids[0],
    ));
    make_work(&f, DOI);
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 3).expect("historical ordering");
    assert_eq!(
        reconciliation_state(&f, current.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(
        reconciliation_state(&f, old.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_SUPERSEDED")
    );
    assert_eq!(
        reconciliation_state(&f, equal.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_IMPORT_ORDER_AMBIGUOUS")
    );

    let (_guard, f) = setup();
    let same = seed(&f, DOI, day(1), &["10", "20"]);
    make_work(&f, DOI);
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 2).expect("same import");
    let states: Vec<String> = same
        .quarantine_ids
        .iter()
        .map(|id| reconciliation_state(&f, *id).expect("state"))
        .collect();
    assert!(states.contains(&"RESOLVED_WINNER".to_string()));
    assert!(states.contains(&"BLOCKED_SAME_IMPORT_ORDER".to_string()));
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
}

#[test]
fn publisher_mismatch_overlap_and_delta_overflow_are_blocked_without_extra_delta() {
    let (_guard, f) = setup();
    let mismatch = seed(&f, DOI, day(1), &["10"]);
    make_foreign_work(&f, DOI);
    let batch = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("mismatch");
    assert_eq!((batch.attempted, batch.blocked), (1, 1));
    assert_eq!(
        reconciliation_state(&f, mismatch.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_PUBLISHER_SCOPE_MISMATCH")
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0
    );

    let (_guard, f) = setup();
    let first = seed(&f, DOI, day(1), &["10"]);
    let overlap = seed(&f, DOI, day(1), &["20"]);
    f.sql(&format!(
        "UPDATE metric_identifier_quarantine          SET reporting_grain = 'MONTH', period_start = '2026-03-01', period_end = '2026-04-01'          WHERE identifier_quarantine_id = '{}'",
        overlap.quarantine_ids[0]
    ));
    make_work(&f, DOI);
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 2).expect("overlap");
    assert_eq!(
        reconciliation_state(&f, first.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(
        reconciliation_state(&f, overlap.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_OVERLAPPING_PERIOD")
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );

    let (_guard, f) = setup();
    f.sql("UPDATE metric_measure SET allow_negative = TRUE WHERE code = 'title_sessions'");
    let low_value = i64::MIN.to_string();
    let high_value = i64::MAX.to_string();
    let low = seed(&f, DOI, day(1), &[low_value.as_str()]);
    let high = seed(&f, DOI, day(1), &[high_value.as_str()]);
    f.sql(&format!(
        "UPDATE metric_import SET created_at = '2026-01-01 00:00:00+00' WHERE import_id = '{}';          UPDATE metric_import SET created_at = '2026-01-02 00:00:00+00' WHERE import_id = '{}'",
        low.import_id, high.import_id
    ));
    make_work(&f, DOI);
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 2).expect("overflow");
    assert_eq!(
        reconciliation_state(&f, low.quarantine_ids[0]).as_deref(),
        Some("RESOLVED_WINNER")
    );
    assert_eq!(
        reconciliation_state(&f, high.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_DELTA_OVERFLOW")
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
}

#[test]
fn inconsistent_oldest_row_commits_only_blocked_state_and_does_not_starve_later_due_work() {
    let (_guard, f) = setup();
    let bad = seed(&f, DOI, day(1), &["10"]);
    let pending = seed(
        &f,
        "https://doi.org/10.12345/reconciliation-second",
        day(2),
        &["20"],
    );
    f.sql(&format!(
        "UPDATE metric_record_provenance p          SET details = jsonb_set(details, '{{reason_code}}', '"AMBIGUOUS_DOI"', true)          FROM metric_identifier_quarantine q          WHERE q.record_provenance_id = p.record_provenance_id            AND q.identifier_quarantine_id = '{}'",
        bad.quarantine_ids[0]
    ));
    let bad_history = history_snapshot(&f, bad.import_id);
    let pending_history = history_snapshot(&f, pending.import_id);

    let batch = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 2).expect("continue");
    assert_eq!((batch.attempted, batch.blocked, batch.pending), (2, 1, 1));
    assert_eq!(
        reconciliation_state(&f, bad.quarantine_ids[0]).as_deref(),
        Some("BLOCKED_INCONSISTENT_EVIDENCE")
    );
    assert_eq!(
        reconciliation_state(&f, pending.quarantine_ids[0]).as_deref(),
        Some("PENDING_UNKNOWN_DOI")
    );
    assert_eq!(history_snapshot(&f, bad.import_id), bad_history);
    assert_eq!(history_snapshot(&f, pending.import_id), pending_history);
    assert_eq!(scalar_i64(&f, "(SELECT COUNT(*) FROM metric_record)"), 0);
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0
    );
}

#[test]
fn unexpected_internal_failure_rolls_back_the_attempt_row() {
    let (_guard, f) = setup();
    let seeded = seed(&f, DOI, day(1), &["10", "20"]);
    make_work(&f, DOI);
    reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("first winner");
    let winner = seeded
        .quarantine_ids
        .iter()
        .copied()
        .find(|id| reconciliation_state(&f, *id).as_deref() == Some("RESOLVED_WINNER"))
        .expect("winner");
    let remaining = seeded
        .quarantine_ids
        .iter()
        .copied()
        .find(|id| *id != winner)
        .expect("remaining");

    f.sql("UPDATE metric_record SET current_revision_id = NULL");
    assert!(
        reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).is_err(),
        "corrupt canonical state must fail closed"
    );
    assert_eq!(
        reconciliation_state(&f, remaining),
        None,
        "the failed row transaction must not persist an attempt"
    );
    assert_eq!(
        scalar_i64(&f, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );
}

#[test]
fn skip_locked_excludes_a_row_held_by_another_worker_without_blocking() {
    let (_guard, f) = setup();
    let seeded = seed(&f, DOI, day(1), &["1"]);
    let id = seeded.quarantine_ids[0];

    let acquired = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let pool = Arc::clone(&f.pool);
    let acquired_worker = Arc::clone(&acquired);
    let release_worker = Arc::clone(&release);
    let handle = thread::spawn(move || {
        let mut connection = pool.get().expect("worker connection");
        connection
            .transaction::<(), diesel::result::Error, _>(|connection| {
                sql_query(
                    "UPDATE metric_identifier_quarantine                      SET created_at = created_at WHERE identifier_quarantine_id = $1",
                )
                .bind::<SqlUuid, _>(id)
                .execute(connection)?;
                acquired_worker.wait();
                release_worker.wait();
                Ok(())
            })
            .expect("worker transaction");
    });

    acquired.wait();
    let skipped = reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("skip locked");
    assert_eq!(skipped.attempted, 0);
    release.wait();
    handle.join().expect("worker");

    let attempted =
        reconcile_metric_identifier_quarantine(&f.pool, ACTOR, 1).expect("after release");
    assert_eq!((attempted.attempted, attempted.pending), (1, 1));
}
