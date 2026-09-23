//! Focused database tests for the rollup delta, its MOM-1 work-day progress
//! ordering, the claim/lease protocol, the `metric_rollup_work_day`
//! projection and the durable watermark.
//!
//! The `MET-WP1-07` half asserts the persistence contract: the approved
//! field/default shape, signed positive/zero/negative delta values, nullable
//! `applied_at`, same-record/revision composite referential integrity,
//! one-delta-per-revision uniqueness, restricted (non-cascading) deletion and
//! the exact index inventory.
//!
//! The `MET-WP4-01` half asserts the application contract: transactional
//! gap-free sequence allocation, the closed `PENDING`/`CLAIMED`/`APPLIED`
//! lifecycle, strict-frontier claiming under two workers, lease expiry and
//! reclaim, whole-batch atomic application, checked signed-i64 arithmetic,
//! failure-injection rollback, idempotent replay, watermark monotonicity,
//! deterministic rebuild equivalence, and the populated-database migration
//! and guarded rollback behaviour.
//!
//! The canonical record/revision fixtures are the existing `pub(crate)`
//! helpers defined by `metric_record/tests.rs` and
//! `metric_record_revision/tests.rs`, consumed as-is: no other model's test
//! module is widened, because this task's write budget contains only this
//! file. Those helpers create `MONTH`-grain records, so the work-day tests
//! below insert their own one-day `DAY` records through raw SQL over the same
//! fixture entities rather than changing a helper another module owns.
//!
//! The `MET-WP4-03A` half asserts the derived monthly serving layer beneath
//! the same completion: the four-table schema, day-first resolution (the
//! Amendment 6 total rule and the unique-least country/institution rule),
//! additive publication identity, OR-ed dependency flags, sparse ambiguity,
//! exact row watermarks, the fixed nine-statement set-wise maintenance,
//! atomic rollback, read-only replay, monthly overflow, a fixed-seed
//! differential against an independent per-day oracle, incremental/rebuild
//! equality and the populated migration round trip.
//!
//! GraphQL authorization, resolver wiring and SDL evidence live in
//! `crate::graphql::metric_rollup_tests`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate};
use diesel::connection::InstrumentationEvent;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use thoth_errors::ThothError;
use uuid::Uuid;

use super::crud::{
    claim_metric_rollup_deltas, complete_metric_rollup_deltas, rebuild_month_projections,
    recompute_month_projections, MonthKey, MONTH_MAINTENANCE_STATEMENTS,
    MONTH_MAINTENANCE_STATEMENT_COUNT,
};
use super::{
    MetricRollupDelta, MetricRollupWorkCountryMonth, MetricRollupWorkDay, MetricRollupWorkDayState,
    MetricRollupWorkInstitutionMonth, MetricRollupWorkMonth, MetricRollupWorkMonthAmbiguity,
    METRIC_ROLLUP_CLAIM_MAX_BATCH, METRIC_ROLLUP_LEASE_SECONDS,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{
    delete_row, foreign_keys, index_definition, index_names, RecordFixture,
};
use crate::model::metric_record_revision::tests::{
    fixture_record, insert_revision_row, insert_second_record,
};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::{
    metric_rollup_delta, metric_rollup_work_country_month, metric_rollup_work_day,
    metric_rollup_work_day_state, metric_rollup_work_institution_month, metric_rollup_work_month,
    metric_rollup_work_month_ambiguity,
};

/// The Diesel migration version of `thoth-api/migrations/20260903_v1.9.0`.
const MET_WP1_07_MIGRATION_VERSION: &str = "20260903";

/// The four derived monthly serving tables `MET-WP4-03A` delivers beneath
/// the MOM-1 work-day projection, in name order.
const MONTH_TABLES: [&str; 4] = [
    "metric_rollup_work_country_month",
    "metric_rollup_work_institution_month",
    "metric_rollup_work_month",
    "metric_rollup_work_month_ambiguity",
];

/// Serving objects the accepted `MET-WP4-03-BENCH-01` evidence rejected and
/// the approved `MET-WP4-03A` specification prohibits. None may exist.
const REJECTED_SERVING_OBJECTS: [&str; 2] =
    ["metric_rollup_work_month_state", "metric_work_dimension"];

/// Column names that would betray a retry/backoff, failure-detail or
/// rebuild-generation protocol having been smuggled into the delta table.
///
/// `MET-WP4-01` deliberately adds none of them. A poison frontier blocks and
/// waits for separately authorized repair, so there is nothing to count
/// attempts of or to schedule a next attempt for, and rebuild generations
/// belong to a rebuild surface that stays deferred.
const DEFERRED_CLAIM_COLUMNS: [&str; 6] = [
    "attempt_count",
    "error_detail",
    "generation",
    "next_attempt_at",
    "retry_count",
    "updated_at",
];

/// Revert migrations until the `MET-WP1-07` rollup-delta migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_record_schema_migration` and
/// its predecessors: a bare `revert_last_migration` would only mean "the
/// rollup-delta migration" while it happens to be the newest applied
/// migration. Reverting down to and including the target keeps the meaning
/// under any later migration order, and no future migration name is assumed
/// or hard-coded.
fn revert_through_rollup_delta_migration(connection: &mut PgConnection) {
    let rollup_delta_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_07_MIGRATION_VERSION);
    assert!(
        rollup_delta_migration_applied,
        "the MET-WP1-07 rollup-delta migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_07_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one canonical record plus one canonical revision of it, and return
/// the fixture, the record id and the revision id.
fn fixture_record_revision(pool: &PgPool, identity_hash: &str) -> (RecordFixture, Uuid, Uuid) {
    let (fixture, record_id) = fixture_record(pool, identity_hash);
    let revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(revision_id),
        record_id,
        1,
        fixture.import_id,
        100,
        &format!("content-{identity_hash}"),
        "CURRENT",
        None,
    )
    .expect("Failed to insert the fixture revision row");
    (fixture, record_id, revision_id)
}

/// Insert one rollup delta through raw SQL so the database defaults are
/// exercised rather than restated by a Diesel fixture.
fn insert_delta_row(
    pool: &PgPool,
    delta_id: Option<Uuid>,
    record_id: Uuid,
    revision_id: Uuid,
    delta_value: i64,
    status: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match delta_id {
        Some(delta_id) => sql_query(
            "INSERT INTO metric_rollup_delta \
                 (delta_id, record_id, revision_id, delta_value, status) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind::<diesel::sql_types::Uuid, _>(delta_id)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .bind::<diesel::sql_types::BigInt, _>(delta_value)
        .bind::<diesel::sql_types::Text, _>(status)
        .execute(&mut connection),
        None => sql_query(
            "INSERT INTO metric_rollup_delta \
                 (record_id, revision_id, delta_value, status) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .bind::<diesel::sql_types::BigInt, _>(delta_value)
        .bind::<diesel::sql_types::Text, _>(status)
        .execute(&mut connection),
    }
}

/// One additional canonical revision of an existing record.
fn insert_extra_revision(
    pool: &PgPool,
    fixture: &RecordFixture,
    record_id: Uuid,
    revision_number: i32,
) -> Uuid {
    let revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(revision_id),
        record_id,
        revision_number,
        fixture.import_id,
        i64::from(revision_number),
        &format!("content-extra-{revision_number}-{record_id}"),
        "SUPERSEDED",
        None,
    )
    .expect("Failed to insert the extra revision row");
    revision_id
}

#[test]
fn migration_seeds_no_rollup_delta_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        0,
        "MET-WP1-07 must not seed any metric_rollup_delta row"
    );
}

#[test]
fn a_rollup_delta_round_trips_through_diesel_with_a_null_applied_at() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let delta_id: Uuid = diesel::insert_into(metric_rollup_delta::table)
        .values((
            metric_rollup_delta::record_id.eq(record_id),
            metric_rollup_delta::revision_id.eq(revision_id),
            metric_rollup_delta::delta_value.eq(1_200_i64),
            metric_rollup_delta::status.eq("PENDING"),
        ))
        .returning(metric_rollup_delta::delta_id)
        .get_result(&mut connection)
        .expect("Failed to insert the unapplied rollup delta");

    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .filter(metric_rollup_delta::delta_id.eq(delta_id))
        .first(&mut connection)
        .expect("Failed to load the unapplied rollup delta");
    assert_eq!(loaded.delta_id, delta_id);
    assert_eq!(loaded.record_id, record_id);
    assert_eq!(loaded.revision_id, revision_id);
    assert_eq!(loaded.delta_value, 1_200);
    assert_eq!(loaded.status, "PENDING");
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
    assert_eq!(
        loaded.applied_at, None,
        "applied_at must stay NULL until the delta is applied"
    );
    // The fixture record is MONTH grain, so it is outside the MOM-1 work-day
    // progress stream and must receive no position at all.
    assert_eq!(
        loaded.work_day_sequence, None,
        "a non-day delta must receive no work-day sequence"
    );
    assert_eq!(loaded.claim_token, None);
    assert_eq!(loaded.claimed_by, None);
    assert_eq!(loaded.claimed_at, None);
    assert_eq!(loaded.lease_expires_at, None);
}

#[test]
fn an_applied_rollup_delta_round_trips_with_its_terminal_claim_evidence() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = day_record_revision(&pool, "identity-a", DAY_ONE);
    insert_delta_row(&pool, None, record_id, revision_id, -7, "PENDING")
        .expect("Failed to insert the pending rollup delta");
    let claimed_at = Timestamp::parse_from_rfc3339("2026-09-13T11:22:33Z")
        .expect("Failed to parse the fixture claimed_at timestamp");
    let applied_at = Timestamp::parse_from_rfc3339("2026-09-13T11:24:33Z")
        .expect("Failed to parse the fixture applied_at timestamp");
    let claim_token = Uuid::new_v4();

    let mut connection = pool.get().expect("Failed to get DB connection");
    let delta_id: Uuid = diesel::update(metric_rollup_delta::table)
        .set((
            metric_rollup_delta::status.eq("APPLIED"),
            metric_rollup_delta::claim_token.eq(claim_token),
            metric_rollup_delta::claimed_by.eq("metrics-ingest-1"),
            metric_rollup_delta::claimed_at.eq(claimed_at),
            metric_rollup_delta::lease_expires_at.eq(None::<Timestamp>),
            metric_rollup_delta::applied_at.eq(applied_at),
        ))
        .returning(metric_rollup_delta::delta_id)
        .get_result(&mut connection)
        .expect("Failed to terminalize the rollup delta");

    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .filter(metric_rollup_delta::delta_id.eq(delta_id))
        .first(&mut connection)
        .expect("Failed to load the applied rollup delta");
    assert_eq!(loaded.delta_value, -7);
    assert_eq!(loaded.status, "APPLIED");
    assert_eq!(loaded.applied_at, Some(applied_at));
    // The terminal row keeps who applied it and under which batch: the lease
    // is closed by nulling its expiry, not by erasing the evidence, and that
    // retained token is what makes an identical replay recognisable.
    assert_eq!(loaded.claim_token, Some(claim_token));
    assert_eq!(loaded.claimed_by.as_deref(), Some("metrics-ingest-1"));
    assert_eq!(loaded.claimed_at, Some(claimed_at));
    assert_eq!(loaded.lease_expires_at, None);
    assert_eq!(loaded.work_day_sequence, Some(1));
}

#[test]
fn rollup_delta_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    insert_delta_row(&pool, None, record_id, revision_id, 0, "PENDING")
        .expect("Failed to insert the defaulted rollup delta");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRollupDelta = metric_rollup_delta::table
        .first(&mut connection)
        .expect("Failed to load the defaulted rollup delta");
    assert_ne!(
        loaded.delta_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a delta_id"
    );
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
    assert_eq!(
        loaded.applied_at, None,
        "applied_at must have no invented default"
    );
}

#[test]
fn positive_zero_and_negative_delta_values_are_all_accepted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, first_revision_id) = fixture_record_revision(&pool, "identity-a");

    // A revision contributes the signed difference `new - old` and a
    // retraction subtracts the previously applied value, so there is
    // deliberately no blanket delta_value >= 0 constraint. The extremes prove
    // the column is a full signed BIGINT.
    let values = [1_200_i64, 0, -45, i64::MIN, i64::MAX];
    let mut revision_ids = vec![first_revision_id];
    for number in 2..=(values.len() as i32) {
        revision_ids.push(insert_extra_revision(&pool, &fixture, record_id, number));
    }
    for (revision_id, value) in revision_ids.iter().zip(values) {
        insert_delta_row(&pool, None, record_id, *revision_id, value, "PENDING").unwrap_or_else(
            |error| panic!("a signed BIGINT delta_value {value} must be accepted: {error:?}"),
        );
    }

    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut stored: Vec<i64> = metric_rollup_delta::table
        .select(metric_rollup_delta::delta_value)
        .load(&mut connection)
        .expect("Failed to load the stored delta values");
    stored.sort_unstable();
    let mut expected = values.to_vec();
    expected.sort_unstable();
    assert_eq!(stored, expected);
}

#[test]
fn the_status_vocabulary_is_closed_to_the_three_approved_states() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, first_revision_id) = fixture_record_revision(&pool, "identity-a");

    // MET-WP1-07 deliberately left `status` as open TEXT because the
    // claim/application state machine was undecided. MET-WP4-01 decides it,
    // so anything outside the three approved states is now a database error
    // rather than a silently stored string.
    let rejected = [
        "anything at all",
        "  padded  ",
        "état-appliqué",
        "pending",
        "APPLYING",
        "",
    ];
    let mut revision_ids = vec![first_revision_id];
    for number in 2..=(rejected.len() as i32 + 1) {
        revision_ids.push(insert_extra_revision(&pool, &fixture, record_id, number));
    }
    for (revision_id, status) in revision_ids.iter().zip(rejected) {
        let result = insert_delta_row(&pool, None, record_id, *revision_id, 1, status);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "status {status:?} must be rejected by the closed lifecycle \
             constraint, got {result:?}"
        );
    }

    // Only PENDING is directly insertable: CLAIMED and APPLIED additionally
    // require claim evidence that only the protocol can produce, which is
    // asserted by the lifecycle-shape test below.
    insert_delta_row(&pool, None, record_id, first_revision_id, 1, "PENDING")
        .expect("PENDING must be accepted");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let stored: Vec<String> = metric_rollup_delta::table
        .select(metric_rollup_delta::status)
        .load(&mut connection)
        .expect("Failed to load the stored statuses");
    assert_eq!(stored, vec!["PENDING".to_string()]);
}

#[test]
fn an_unknown_record_or_revision_reference_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");

    for (label, delta_record_id, delta_revision_id) in [
        ("an unknown record", Uuid::new_v4(), revision_id),
        ("an unknown revision", record_id, Uuid::new_v4()),
        ("both unknown", Uuid::new_v4(), Uuid::new_v4()),
    ] {
        let result = insert_delta_row(
            &pool,
            None,
            delta_record_id,
            delta_revision_id,
            10,
            "PENDING",
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "{label} must be rejected by the composite foreign key, got {result:?}"
        );
    }
}

#[test]
fn a_revision_belonging_to_another_record_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let other_record_id = insert_second_record(&pool, &fixture, "identity-b");
    let foreign_revision_id = insert_extra_revision(&pool, &fixture, other_record_id, 1);

    // Both ids exist and both are individually valid, but they do not name the
    // same canonical pair. Only the composite key catches this.
    let result = insert_delta_row(&pool, None, record_id, foreign_revision_id, 10, "PENDING");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a delta must not pair a record with a revision owned by another \
         record, got {result:?}"
    );

    // The same-record pairings remain valid.
    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("a delta naming its own record's revision must be accepted");
    insert_delta_row(
        &pool,
        None,
        other_record_id,
        foreign_revision_id,
        20,
        "PENDING",
    )
    .expect("the other record's own delta must be accepted");
}

#[test]
fn at_most_one_delta_is_permitted_per_canonical_revision() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");

    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("the first delta for a revision must be accepted");

    let duplicate = insert_delta_row(&pool, None, record_id, revision_id, -10, "PENDING");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a second delta for the same canonical revision must be rejected so \
         later rollup application cannot double count, got {duplicate:?}"
    );

    // Uniqueness is per revision, not per record: a second revision of the
    // same record gets its own delta.
    let second_revision_id = insert_extra_revision(&pool, &fixture, record_id, 2);
    insert_delta_row(&pool, None, record_id, second_revision_id, 5, "PENDING")
        .expect("a delta for a different revision of the same record must be accepted");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        2,
    );
}

#[test]
fn deleting_a_referenced_revision_or_record_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING")
        .expect("the referencing delta must be accepted");

    for (table, id_column, id) in [
        ("metric_record_revision", "record_revision_id", revision_id),
        ("metric_record", "record_id", record_id),
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
             durable delta evidence, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1,
        "the delta row must survive the restricted deletions"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1,
        "the referenced revision must survive the restricted deletion"
    );
}

#[test]
fn rollup_delta_not_null_columns_are_enforced() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = fixture_record_revision(&pool, "identity-a");
    let mut connection = pool.get().expect("Failed to get DB connection");

    // delta_value and status carry no default, so omitting either must fail
    // rather than silently resolve to zero or an invented state.
    for (label, statement) in [
        (
            "delta_value",
            "INSERT INTO metric_rollup_delta (record_id, revision_id, status) \
             VALUES ($1, $2, 'PENDING')",
        ),
        (
            "status",
            "INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value) \
             VALUES ($1, $2, 10)",
        ),
    ] {
        let result = sql_query(statement)
            .bind::<diesel::sql_types::Uuid, _>(record_id)
            .bind::<diesel::sql_types::Uuid, _>(revision_id)
            .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "{label} must be NOT NULL with no default, got {result:?}"
        );
    }
}

#[test]
fn metric_rollup_delta_carries_exactly_the_two_approved_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed. In particular there is still no
    // `delta_value >= 0` rule, because retraction and correction deltas are
    // negative; what MET-WP4-01 adds is the positive-sequence rule and the
    // closed per-status shape, and nothing else.
    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_delta"),
        vec![
            "metric_rollup_delta_claim_state_check".to_string(),
            "metric_rollup_delta_work_day_sequence_check".to_string(),
        ],
        "metric_rollup_delta must carry exactly the two approved CHECK constraints"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_authorized_non_cascading_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_rollup_delta");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_rollup_delta_record_id_revision_id_fkey"],
        "metric_rollup_delta must carry exactly one foreign key"
    );
    let (name, definition) = &keys[0];
    assert!(
        definition.contains("(record_id, revision_id)")
            && definition.contains("metric_record_revision(record_id, record_revision_id)"),
        "the delta key must be the same-record composite shape: {definition}"
    );
    assert!(
        !definition.contains("ON DELETE"),
        "{name} must stay non-cascading and use the default restricting \
         behaviour: {definition}"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_rollup_delta"),
        vec![
            "metric_rollup_delta_claim_token_idx",
            "metric_rollup_delta_frontier_idx",
            "metric_rollup_delta_pkey",
            "metric_rollup_delta_revision_id_key",
            "metric_rollup_delta_work_day_sequence_idx",
        ],
        "metric_rollup_delta must carry exactly its primary key, the \
         one-delta-per-revision uniqueness index and the three approved \
         MET-WP4-01 claim indexes; no speculative access path may be added \
         without query-plan evidence"
    );
    assert!(
        index_definition(&pool, "metric_rollup_delta", "metric_rollup_delta_pkey")
            .contains("(delta_id)"),
        "the primary key must be on delta_id"
    );
    let unique = index_definition(
        &pool,
        "metric_rollup_delta",
        "metric_rollup_delta_revision_id_key",
    );
    assert!(
        unique.contains("UNIQUE") && unique.contains("(revision_id)"),
        "the uniqueness index must be unique on revision_id alone: {unique}"
    );

    // The work-day position index is what makes a position unrepeatable. It
    // is partial because every non-day delta shares a NULL sequence, and an
    // ordinary unique index would then be satisfied trivially by all of them
    // while still admitting a duplicate real position.
    let sequence = index_definition(
        &pool,
        "metric_rollup_delta",
        "metric_rollup_delta_work_day_sequence_idx",
    );
    assert!(
        sequence.contains("UNIQUE")
            && sequence.contains("(work_day_sequence)")
            && sequence.contains("WHERE (work_day_sequence IS NOT NULL)"),
        "the sequence index must be partially unique on work_day_sequence: {sequence}"
    );

    let frontier = index_definition(
        &pool,
        "metric_rollup_delta",
        "metric_rollup_delta_frontier_idx",
    );
    assert!(
        frontier.contains("(work_day_sequence)") && frontier.contains("status <> 'APPLIED'"),
        "the frontier index must cover only non-applied work-day rows: {frontier}"
    );

    let token = index_definition(
        &pool,
        "metric_rollup_delta",
        "metric_rollup_delta_claim_token_idx",
    );
    assert!(
        !token.contains("UNIQUE")
            && token.contains("(claim_token)")
            && token.contains("WHERE (claim_token IS NOT NULL)"),
        "the claim-token index must be partial and non-unique, because one \
         token addresses a whole batch: {token}"
    );
}

#[test]
fn metric_rollup_delta_has_exactly_the_approved_columns() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct Column {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let columns: Vec<(String, String, String)> = sql_query(
        "SELECT column_name::text, data_type::text, is_nullable::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_rollup_delta' \
         ORDER BY ordinal_position",
    )
    .load::<Column>(&mut connection)
    .expect("Failed to read the metric_rollup_delta columns")
    .into_iter()
    .map(|column| (column.column_name, column.data_type, column.is_nullable))
    .collect();
    let observed: Vec<(&str, &str, &str)> = columns
        .iter()
        .map(|(name, data_type, nullable)| (name.as_str(), data_type.as_str(), nullable.as_str()))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("delta_id", "uuid", "NO"),
            ("record_id", "uuid", "NO"),
            ("revision_id", "uuid", "NO"),
            ("delta_value", "bigint", "NO"),
            ("status", "text", "NO"),
            ("created_at", "timestamp with time zone", "NO"),
            ("applied_at", "timestamp with time zone", "YES"),
            ("work_day_sequence", "bigint", "YES"),
            ("claim_token", "uuid", "YES"),
            ("claimed_by", "text", "YES"),
            ("claimed_at", "timestamp with time zone", "YES"),
            ("lease_expires_at", "timestamp with time zone", "YES"),
        ],
        "metric_rollup_delta must carry exactly the twelve approved columns. \
         All five MET-WP4-01 additions are nullable because a PENDING \
         non-day delta legitimately carries none of them; the per-status \
         shape is enforced by the closed lifecycle CHECK, which NOT NULL \
         could not express"
    );
}

#[test]
fn no_deferred_rebuild_retry_or_projection_object_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // The four monthly serving tables are delivered by MET-WP4-03A; the
    // serving objects its benchmark rejected must not exist.
    for table in MONTH_TABLES {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_class \
                      WHERE relnamespace = 'public'::regnamespace \
                        AND relkind = 'r' AND relname = '{table}')"
                ),
            ),
            1,
            "MET-WP4-03A must create the derived monthly table {table}"
        );
    }
    for object in REJECTED_SERVING_OBJECTS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_class \
                      WHERE relnamespace = 'public'::regnamespace \
                        AND relname = '{object}')"
                ),
            ),
            0,
            "the rejected serving object {object} must not exist"
        );
    }

    // No retry/backoff/rebuild-generation column was smuggled in.
    for column in DEFERRED_CLAIM_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_rollup_delta' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP4-01 must not add the deferred retry/rebuild column {column}"
        );
    }

    // The closed vocabulary is carried by a CHECK, not by a PostgreSQL enum:
    // an enum type would make a later vocabulary change a type migration
    // across every dependent object. `typtype = 'e'` restricts the count to
    // enums, because PostgreSQL always creates an implicit composite type
    // named after the table itself.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typtype = 'e' \
                AND typname LIKE 'metric_rollup%')",
        ),
        0,
        "MOM-1 must create no rollup enum type"
    );

    // Exactly one trigger exists on the delta table, and it allocates the
    // work-day position and nothing else. No trigger or stored procedure
    // moves a delta between statuses: every transition is an explicit,
    // authorized statement in the coordinator's own transaction.
    let triggers = trigger_names(&pool, "metric_rollup_delta");
    assert_eq!(
        triggers,
        vec!["metric_rollup_delta_assign_work_day_sequence".to_string()],
        "metric_rollup_delta must carry exactly the work-day allocation trigger"
    );
    for table in [
        "metric_rollup_work_day",
        "metric_rollup_work_day_state",
        "metric_rollup_work_month",
        "metric_rollup_work_country_month",
        "metric_rollup_work_institution_month",
        "metric_rollup_work_month_ambiguity",
    ] {
        assert_eq!(
            trigger_names(&pool, table),
            Vec::<String>::new(),
            "{table} must carry no trigger: its timestamps and values are \
             written explicitly by the completion transaction"
        );
    }
}

#[test]
fn reverting_through_the_rollup_delta_migration_removes_it_and_reapplication_restores_it() {
    let (_guard, _pool) = setup_registry_db();
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    let count_objects = |connection: &mut PgConnection, query: &str| -> i64 {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(query))
            .get_result(connection)
            .expect("Failed to count schema objects")
    };

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_rollup_delta')",
        ),
        1,
        "the rollup-delta table must exist before reverting"
    );

    revert_through_rollup_delta_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_rollup_delta')",
        ),
        0,
        "the downgrade must drop the MET-WP1-07 table"
    );

    // Every predecessor Metrics slice survives, including the MET-WP1-04
    // record/revision schema this table references and the supporting
    // same-record unique key the composite foreign key depends on.
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('metric_platform', 'metric_measure', \
                                'metric_platform_measure', 'metric_source', \
                                'metric_source_account', 'metric_source_checkpoint', \
                                'metric_import', 'metric_import_error', \
                                'metric_record', 'metric_record_revision', \
                                'metric_record_provenance', 'metric_coverage', \
                                'metric_publisher_platform_approval'))",
        ),
        13,
        "the downgrade must leave the MET-WP1-01..06 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_record_revision_record_id_record_revision_id_key')",
        ),
        1,
        "the downgrade must not drop the MET-WP1-04 supporting unique key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname IN ('work', 'publication', 'institution', 'publisher'))",
        ),
        4,
        "the downgrade must not touch the bibliographic schema"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_measure WHERE code IN ('title_sessions', 'net_units'))",
        ),
        2,
        "the downgrade must leave the MET-WP1-01 measure seeds in place"
    );

    // Reapplication recreates the empty table with exactly its two indexes.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the rollup-delta migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_rollup_delta)"
        ),
        0,
        "reapplication must seed no rollup-delta row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_rollup_delta')",
        ),
        5,
        "reapplication must restore exactly the five required indexes"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname = 'metric_rollup_delta_record_id_revision_id_fkey')",
        ),
        1,
        "reapplication must restore the composite foreign key"
    );
}

// ===========================================================================
// MET-WP4-01: work-day ordering, claim/lease protocol, application, watermark
// ===========================================================================

/// The Diesel migration version of `thoth-api/migrations/20260913_v1.9.0`.
const MET_WP4_01_MIGRATION_VERSION: &str = "20260913";

/// The authenticated machine principal these tests act as.
const CLAIMANT: &str = "metrics-ingest-service-1";
/// A different authenticated machine principal.
const OTHER_CLAIMANT: &str = "metrics-ingest-service-2";

/// One calendar day inside the fixture window.
pub(crate) const DAY_ONE: (i32, u32, u32) = (2026, 3, 1);
/// The next calendar day.
pub(crate) const DAY_TWO: (i32, u32, u32) = (2026, 3, 2);

pub(crate) fn day(parts: (i32, u32, u32)) -> NaiveDate {
    NaiveDate::from_ymd_opt(parts.0, parts.1, parts.2).expect("a valid fixture date")
}

/// The optional dimensions of one canonical work-day record.
#[derive(Clone, Copy, Default)]
pub(crate) struct DayDimensions {
    pub(crate) publication: bool,
    pub(crate) country_code: Option<&'static str>,
    pub(crate) institution: bool,
}

/// Revert migrations until the `MET-WP4-01` rollup-application migration
/// itself has been reverted.
///
/// The same durable pattern as [`revert_through_rollup_delta_migration`]: a
/// bare `revert_last_migration` would only mean "the rollup-application
/// migration" while it happens to be the newest applied one.
fn revert_through_rollup_application_migration(
    connection: &mut PgConnection,
) -> Result<(), String> {
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .map_err(|error| error.to_string())?;
        if reverted.to_string() == MET_WP4_01_MIGRATION_VERSION {
            return Ok(());
        }
    }
}

/// The non-internal trigger names on one table, in name order.
fn trigger_names(pool: &PgPool, table: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        tgname: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "SELECT t.tgname::text AS tgname FROM pg_trigger t \
         JOIN pg_class c ON c.oid = t.tgrelid \
         WHERE NOT t.tgisinternal AND c.relname = '{table}' \
         ORDER BY t.tgname"
    ))
    .load::<Row>(&mut connection)
    .expect("Failed to read triggers")
    .into_iter()
    .map(|row| row.tgname)
    .collect()
}

/// Insert one canonical one-day `DAY` record over the shared fixture
/// entities.
///
/// The record/revision helpers this module already consumes create
/// `MONTH`-grain records, and they belong to other modules' test budgets, so
/// the work-day shape is written here through raw SQL instead of by widening
/// a helper this task does not own.
fn insert_day_record(
    pool: &PgPool,
    fixture: &RecordFixture,
    identity_hash: &str,
    on: NaiveDate,
    dimensions: DayDimensions,
) -> Uuid {
    let record_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_record \
             (record_id, identity_hash, work_id, publication_id, platform_id, measure_id, \
              period_start, period_end, reporting_grain, country_code, institution_id, \
              winning_source_account_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7 + 1, 'DAY', $8, $9, $10)",
    )
    .bind::<diesel::sql_types::Uuid, _>(record_id)
    .bind::<diesel::sql_types::Text, _>(identity_hash)
    .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(
        dimensions.publication.then_some(fixture.publication_id),
    )
    .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
    .bind::<diesel::sql_types::Date, _>(on)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(dimensions.country_code)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(
        dimensions.institution.then_some(fixture.institution_id),
    )
    .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
    .execute(&mut connection)
    .expect("Failed to insert the fixture work-day record");
    record_id
}

/// Insert one canonical revision, supersede its predecessor and point the
/// record's `current_revision_id` at it, so a rebuild can be derived from
/// current canonical state alone.
///
/// Superseding first is what the merged ingestion coordinator does inside its
/// own transaction, and it is required here too: MET-WP1-04 enforces at most
/// one `CURRENT` revision per record.
fn insert_current_revision(
    pool: &PgPool,
    fixture: &RecordFixture,
    record_id: Uuid,
    revision_number: i32,
    value: i64,
) -> Uuid {
    exec(
        pool,
        &format!(
            "UPDATE metric_record_revision SET status = 'SUPERSEDED' \
             WHERE record_id = '{record_id}' AND status = 'CURRENT'"
        ),
    );
    let revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(revision_id),
        record_id,
        revision_number,
        fixture.import_id,
        value,
        &format!("content-{record_id}-{revision_number}"),
        "CURRENT",
        None,
    )
    .expect("Failed to insert the fixture revision row");
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("UPDATE metric_record SET current_revision_id = $1 WHERE record_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .execute(&mut connection)
        .expect("Failed to point the record at its current revision");
    revision_id
}

/// One canonical one-day `DAY` record with one current revision.
fn day_record_revision(
    pool: &PgPool,
    identity_hash: &str,
    on: (i32, u32, u32),
) -> (RecordFixture, Uuid, Uuid) {
    let (fixture, _month_record_id) = fixture_record(pool, &format!("{identity_hash}-month"));
    let record_id = insert_day_record(
        pool,
        &fixture,
        identity_hash,
        day(on),
        DayDimensions::default(),
    );
    let revision_id = insert_current_revision(pool, &fixture, record_id, 1, 100);
    (fixture, record_id, revision_id)
}

/// Commit one `PENDING` work-day delta of `value` over the shared fixture,
/// and return its canonical record id.
pub(crate) fn commit_work_day_delta(
    pool: &PgPool,
    fixture: &RecordFixture,
    identity_hash: &str,
    on: (i32, u32, u32),
    dimensions: DayDimensions,
    value: i64,
) -> Uuid {
    let record_id = insert_day_record(pool, fixture, identity_hash, day(on), dimensions);
    let revision_id = insert_current_revision(pool, fixture, record_id, 1, value);
    insert_delta_row(pool, None, record_id, revision_id, value, "PENDING")
        .expect("Failed to insert the fixture work-day delta");
    record_id
}

/// Commit one further revision of an existing record, with its signed
/// difference delta, exactly as canonical ingestion would.
fn commit_revision_delta(
    pool: &PgPool,
    fixture: &RecordFixture,
    record_id: Uuid,
    revision_number: i32,
    new_value: i64,
    previous_value: i64,
) {
    let revision_id = insert_current_revision(pool, fixture, record_id, revision_number, new_value);
    insert_delta_row(
        pool,
        None,
        record_id,
        revision_id,
        new_value - previous_value,
        "PENDING",
    )
    .expect("Failed to insert the fixture revision delta");
}

/// The singleton progress row.
pub(crate) fn state(pool: &PgPool) -> MetricRollupWorkDayState {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_rollup_work_day_state::table
        .first(&mut connection)
        .expect("Failed to load the rollup progress state")
}

/// Every projection row, in a deterministic order.
pub(crate) fn projection(pool: &PgPool) -> Vec<MetricRollupWorkDay> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupWorkDay> = metric_rollup_work_day::table
        .load(&mut connection)
        .expect("Failed to load the work-day projection");
    rows.sort_by_key(|row| (row.day, row.value, row.watermark));
    rows
}

/// Every delta, ordered by work-day position with non-day deltas last.
pub(crate) fn deltas(pool: &PgPool) -> Vec<MetricRollupDelta> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupDelta> = metric_rollup_delta::table
        .load(&mut connection)
        .expect("Failed to load the rollup deltas");
    rows.sort_by_key(|row| (row.work_day_sequence.unwrap_or(i64::MAX), row.delta_id));
    rows
}

/// Run a statement that is expected to succeed.
fn exec(pool: &PgPool, statement: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(statement)
        .execute(&mut connection)
        .unwrap_or_else(|error| panic!("statement failed: {statement}: {error:?}"));
}

/// Age one batch's lease so the expiry predicate treats it as expired.
///
/// The alternative would be waiting 900 seconds. This writes the same durable
/// column the protocol reads, so the real expiry predicate is exercised
/// rather than simulated.
fn expire_lease(pool: &PgPool, claim_token: Uuid) {
    exec(
        pool,
        &format!(
            "UPDATE metric_rollup_delta \
             SET lease_expires_at = transaction_timestamp() - interval '1 second' \
             WHERE claim_token = '{claim_token}'"
        ),
    );
}

/// A trigger that raises, so the statement it guards fails.
///
/// Dropped on scope exit, so an injected failure cannot leak into another
/// test even when the assertion in between panics.
struct FailingTrigger {
    pool: Arc<PgPool>,
    name: String,
    table: String,
}

impl FailingTrigger {
    fn install(
        pool: &Arc<PgPool>,
        timing_and_event: &str,
        table: &str,
        when: Option<&str>,
    ) -> Self {
        let name = format!("thoth_wp4_test_{}", Uuid::new_v4().simple());
        exec(
            pool,
            &format!(
                "CREATE FUNCTION {name}() RETURNS trigger LANGUAGE plpgsql AS \
                 $$ BEGIN RAISE EXCEPTION 'thoth test failure injection'; RETURN NULL; END $$"
            ),
        );
        let when = when.map(|w| format!(" WHEN ({w})")).unwrap_or_default();
        exec(
            pool,
            &format!(
                "CREATE TRIGGER {name} {timing_and_event} ON {table} \
                 FOR EACH ROW{when} EXECUTE FUNCTION {name}()"
            ),
        );
        FailingTrigger {
            pool: Arc::clone(pool),
            name,
            table: table.to_string(),
        }
    }
}

impl Drop for FailingTrigger {
    fn drop(&mut self) {
        let Ok(mut connection) = self.pool.get() else {
            return;
        };
        let _ = sql_query(format!(
            "DROP TRIGGER IF EXISTS {} ON {}",
            self.name, self.table
        ))
        .execute(&mut connection);
        let _ =
            sql_query(format!("DROP FUNCTION IF EXISTS {}()", self.name)).execute(&mut connection);
    }
}

/// Run two closures at the same time, released together by a barrier.
fn run_concurrently<F1, F2, R1, R2>(first: F1, second: F2) -> (R1, R2)
where
    F1: FnOnce() -> R1 + Send + 'static,
    F2: FnOnce() -> R2 + Send + 'static,
    R1: Send + 'static,
    R2: Send + 'static,
{
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let barrier_one = Arc::clone(&barrier);
    let one: JoinHandle<R1> = thread::spawn(move || {
        barrier_one.wait();
        first()
    });
    let two: JoinHandle<R2> = thread::spawn(move || {
        barrier.wait();
        second()
    });
    (one.join().unwrap(), two.join().unwrap())
}

/// The message of a rejected rollup operation.
fn rejection(error: &ThothError) -> String {
    match error {
        ThothError::DatabaseConstraintError(message) => message.to_string(),
        other => panic!("expected a bounded rollup rejection, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Work-day progress ordering
// ---------------------------------------------------------------------------

#[test]
fn only_valid_one_day_deltas_join_the_work_day_ordering() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, month_record_id) = fixture_record(&pool, "identity-month");
    let month_revision_id = insert_current_revision(&pool, &fixture, month_record_id, 1, 5);

    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    // A MONTH-grain delta is durable and PENDING, but outside the MOM-1
    // work-day stream: it gets no position and must never be able to hold up
    // the work-day frontier.
    insert_delta_row(
        &pool,
        None,
        month_record_id,
        month_revision_id,
        5,
        "PENDING",
    )
    .expect("a non-day delta must still be accepted");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );

    let rows = deltas(&pool);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.iter()
            .map(|row| row.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), None],
        "positions are contiguous from 1 across work-day deltas only"
    );
    assert_eq!(state(&pool).next_sequence, 3);

    // The frontier claim reaches both work-day deltas: the interleaved
    // non-day delta is not a hole in the ordering, because it was never in it.
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 10).expect("claim");
    assert_eq!(
        claims
            .iter()
            .map(|claim| claim.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn a_rolled_back_delta_insertion_leaves_no_gap_in_the_ordering() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    assert_eq!(state(&pool).next_sequence, 2);

    // A PostgreSQL sequence would have burned a position here and left a
    // permanent hole the contiguous frontier could never close. The counter
    // row is updated inside the caller's own transaction, so the rollback
    // takes the allocation with it.
    let record_id = insert_day_record(
        &pool,
        &fixture,
        "identity-day-doomed",
        day(DAY_TWO),
        DayDimensions::default(),
    );
    let revision_id = insert_current_revision(&pool, &fixture, record_id, 1, 7);
    let mut connection = pool.get().expect("Failed to get DB connection");
    let outcome: Result<(), DieselError> = connection.transaction(|connection| {
        sql_query(
            "INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value, status) \
             VALUES ($1, $2, 7, 'PENDING')",
        )
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Uuid, _>(revision_id)
        .execute(connection)?;
        Err(DieselError::RollbackTransaction)
    });
    assert!(matches!(outcome, Err(DieselError::RollbackTransaction)));
    drop(connection);

    assert_eq!(
        state(&pool).next_sequence,
        2,
        "a rolled-back ingestion must roll back its allocation too"
    );
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );
    assert_eq!(
        deltas(&pool)
            .iter()
            .filter_map(|row| row.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2],
        "committed positions stay contiguous from 1"
    );
}

#[test]
fn a_caller_supplied_work_day_sequence_is_refused() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = day_record_revision(&pool, "identity-a", DAY_ONE);

    let mut connection = pool.get().expect("Failed to get DB connection");
    let result = sql_query(
        "INSERT INTO metric_rollup_delta \
             (record_id, revision_id, delta_value, status, work_day_sequence) \
         VALUES ($1, $2, 10, 'PENDING', 99)",
    )
    .bind::<diesel::sql_types::Uuid, _>(record_id)
    .bind::<diesel::sql_types::Uuid, _>(revision_id)
    .execute(&mut connection);
    assert!(
        result.is_err(),
        "a writer must not be able to choose its own position in the ordering \
         the exactly-once frontier depends on, got {result:?}"
    );
    assert_eq!(state(&pool).next_sequence, 1, "and no position is burned");
}

#[test]
fn concurrent_first_insertions_receive_distinct_contiguous_positions() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let first = insert_day_record(
        &pool,
        &fixture,
        "identity-day-1",
        day(DAY_ONE),
        DayDimensions::default(),
    );
    let first_revision = insert_current_revision(&pool, &fixture, first, 1, 10);
    let second = insert_day_record(
        &pool,
        &fixture,
        "identity-day-2",
        day(DAY_TWO),
        DayDimensions::default(),
    );
    let second_revision = insert_current_revision(&pool, &fixture, second, 1, 20);

    let one = Arc::clone(&pool);
    let two = Arc::clone(&pool);
    let (a, b) = run_concurrently(
        move || insert_delta_row(&one, None, first, first_revision, 10, "PENDING"),
        move || insert_delta_row(&two, None, second, second_revision, 20, "PENDING"),
    );
    a.expect("the first concurrent delta must commit");
    b.expect("the second concurrent delta must commit");

    let mut positions: Vec<i64> = deltas(&pool)
        .iter()
        .filter_map(|row| row.work_day_sequence)
        .collect();
    positions.sort_unstable();
    assert_eq!(
        positions,
        vec![1, 2],
        "two concurrent allocations must take distinct contiguous positions"
    );
    assert_eq!(state(&pool).next_sequence, 3);
}

// ---------------------------------------------------------------------------
// Claim protocol
// ---------------------------------------------------------------------------

#[test]
fn a_fresh_claim_takes_the_contiguous_run_from_the_frontier() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for index in 0..4 {
        commit_work_day_delta(
            &pool,
            &fixture,
            &format!("identity-day-{index}"),
            DAY_ONE,
            DayDimensions::default(),
            10,
        );
    }

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 3).expect("claim");
    assert_eq!(
        claims
            .iter()
            .map(|claim| claim.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3],
        "a claim starts at applied_through_sequence + 1 and is contiguous"
    );
    // One token and one lease identify the whole batch.
    let token = claims[0].claim_token;
    assert!(claims.iter().all(|claim| claim.claim_token == token));
    let lease = claims[0].lease_expires_at;
    assert!(claims.iter().all(|claim| claim.lease_expires_at == lease));

    let rows = deltas(&pool);
    assert_eq!(
        rows.iter()
            .map(|row| row.status.as_str())
            .collect::<Vec<_>>(),
        vec!["CLAIMED", "CLAIMED", "CLAIMED", "PENDING"]
    );
    assert!(rows[..3]
        .iter()
        .all(|row| row.claimed_by.as_deref() == Some(CLAIMANT)));
    assert_eq!(
        state(&pool).applied_through_sequence,
        0,
        "claiming alone must not move the durable watermark"
    );
}

#[test]
fn a_claim_limit_outside_the_approved_bounds_is_rejected_and_claims_nothing() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );

    for limit in [i32::MIN, -1, 0, METRIC_ROLLUP_CLAIM_MAX_BATCH + 1, i32::MAX] {
        let error = claim_metric_rollup_deltas(&pool, CLAIMANT, limit)
            .expect_err("an out-of-range limit must be rejected, not clamped");
        assert_eq!(
            rejection(&error),
            "A rollup delta claim limit must be between 1 and 50 inclusive."
        );
    }
    assert_eq!(
        deltas(&pool)[0].status,
        "PENDING",
        "a rejected limit must perform no database work at all"
    );

    // The bounds themselves are inclusive.
    assert_eq!(
        claim_metric_rollup_deltas(&pool, CLAIMANT, 1)
            .expect("limit 1 is accepted")
            .len(),
        1
    );
}

#[test]
fn two_concurrent_workers_never_hold_the_same_frontier_work() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for index in 0..4 {
        commit_work_day_delta(
            &pool,
            &fixture,
            &format!("identity-day-{index}"),
            DAY_ONE,
            DayDimensions::default(),
            10,
        );
    }

    let one = Arc::clone(&pool);
    let two = Arc::clone(&pool);
    let (first, second) = run_concurrently(
        move || claim_metric_rollup_deltas(&one, CLAIMANT, 2),
        move || claim_metric_rollup_deltas(&two, OTHER_CLAIMANT, 2),
    );
    let first = first.expect("the first claim must succeed");
    let second = second.expect("the second claim must succeed");

    // Exactly one worker holds the frontier; the other correctly learns there
    // is nothing it may take. An empty result is the right answer, not an
    // error, and specifically not "skip ahead to sequence 3".
    let (held, empty) = if first.is_empty() {
        (second, first)
    } else {
        (first, second)
    };
    assert!(
        empty.is_empty(),
        "two workers must not both hold frontier work: {held:?} and {empty:?}"
    );
    assert_eq!(
        held.iter()
            .map(|claim| claim.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let rows = deltas(&pool);
    assert_eq!(
        rows.iter()
            .map(|row| row.status.as_str())
            .collect::<Vec<_>>(),
        vec!["CLAIMED", "CLAIMED", "PENDING", "PENDING"],
        "no delta beyond the held run may be claimed"
    );
}

#[test]
fn an_unexpired_frontier_claim_is_never_skipped() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for index in 0..3 {
        commit_work_day_delta(
            &pool,
            &fixture,
            &format!("identity-day-{index}"),
            DAY_ONE,
            DayDimensions::default(),
            10,
        );
    }
    let held = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    assert_eq!(held[0].work_day_sequence, 1);

    // Sequences 2 and 3 are PENDING and would be claimable in any
    // skip-locked queue. They are not claimable here: applying a difference
    // out of order would leave the projection wrong until 1 caught up, and
    // the contiguous watermark could not describe the result.
    let blocked = claim_metric_rollup_deltas(&pool, OTHER_CLAIMANT, 50).expect("claim");
    assert!(
        blocked.is_empty(),
        "a live claim at the frontier must block later work, got {blocked:?}"
    );
    assert_eq!(
        deltas(&pool)
            .iter()
            .map(|row| row.status.as_str())
            .collect::<Vec<_>>(),
        vec!["CLAIMED", "PENDING", "PENDING"]
    );
}

#[test]
fn an_expired_frontier_claim_is_reclaimable_with_a_fresh_token() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for index in 0..2 {
        commit_work_day_delta(
            &pool,
            &fixture,
            &format!("identity-day-{index}"),
            DAY_ONE,
            DayDimensions::default(),
            10,
        );
    }
    let abandoned = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let stale_token = abandoned[0].claim_token;
    // A worker that crashed holding the frontier: the lease is what makes
    // that recoverable without an operator.
    expire_lease(&pool, stale_token);

    let reclaimed = claim_metric_rollup_deltas(&pool, OTHER_CLAIMANT, 2).expect("claim");
    assert_eq!(
        reclaimed
            .iter()
            .map(|claim| claim.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2],
        "the expired frontier row is reclaimed, together with the run behind it"
    );
    assert_ne!(
        reclaimed[0].claim_token, stale_token,
        "a reclaim must issue a fresh token"
    );
    let rows = deltas(&pool);
    assert!(rows
        .iter()
        .all(|row| row.claimed_by.as_deref() == Some(OTHER_CLAIMANT)));
}

// ---------------------------------------------------------------------------
// Token rejection
// ---------------------------------------------------------------------------

#[test]
fn a_stale_token_cannot_complete_after_a_reclaim() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );

    let abandoned = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let stale_token = abandoned[0].claim_token;
    expire_lease(&pool, stale_token);
    let reclaimed = claim_metric_rollup_deltas(&pool, OTHER_CLAIMANT, 1).expect("claim");

    let error = complete_metric_rollup_deltas(&pool, CLAIMANT, stale_token)
        .expect_err("a superseded token must not apply anything");
    assert!(
        rejection(&error).starts_with("This rollup claim token names no claimed delta."),
        "unexpected rejection: {error:?}"
    );
    assert!(projection(&pool).is_empty(), "and it must write nothing");
    assert_eq!(state(&pool).applied_through_sequence, 0);

    // The live claimant is unaffected and still completes normally.
    let watermark = complete_metric_rollup_deltas(&pool, OTHER_CLAIMANT, reclaimed[0].claim_token)
        .expect("the live claim completes");
    assert_eq!(watermark.applied_through_sequence, 1);
}

#[test]
fn a_foreign_claimant_cannot_complete_another_workers_batch() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");

    // Holding the role is not holding the claim: `claimed_by` is derived from
    // the authenticated principal, so a second ingest-service identity that
    // somehow learned the token still cannot apply it.
    let error = complete_metric_rollup_deltas(&pool, OTHER_CLAIMANT, claims[0].claim_token)
        .expect_err("a foreign claimant must be rejected");
    assert_eq!(
        rejection(&error),
        "This rollup claim token belongs to a different claimant."
    );
    assert!(projection(&pool).is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 0);
    assert_eq!(deltas(&pool)[0].status, "CLAIMED");
}

#[test]
fn an_unknown_token_and_an_expired_lease_both_fail_without_mutation() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );

    let error = complete_metric_rollup_deltas(&pool, CLAIMANT, Uuid::new_v4())
        .expect_err("a token that was never issued must be rejected");
    assert!(rejection(&error).starts_with("This rollup claim token names no claimed delta."));

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    expire_lease(&pool, claims[0].claim_token);
    let error = complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
        .expect_err("an expired lease must be rejected");
    assert!(
        rejection(&error).starts_with("This rollup claim lease has expired."),
        "unexpected rejection: {error:?}"
    );
    assert!(projection(&pool).is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 0);
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

#[test]
fn a_whole_batch_completion_applies_every_delta_and_advances_the_watermark() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );
    let before = state(&pool);

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    let watermark =
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");

    assert_eq!(watermark.applied_through_sequence, 2);
    assert!(watermark.watermark_at >= before.watermark_at);
    let rows = projection(&pool);
    assert_eq!(
        rows.iter()
            .map(|row| (row.day, row.value, row.watermark))
            .collect::<Vec<_>>(),
        vec![(day(DAY_ONE), 10, 1), (day(DAY_TWO), 20, 2)]
    );
    // Terminal rows keep their claim evidence and close only the lease.
    let deltas = deltas(&pool);
    assert!(deltas.iter().all(|row| row.status == "APPLIED"
        && row.applied_at.is_some()
        && row.lease_expires_at.is_none()
        && row.claim_token == Some(claims[0].claim_token)
        && row.claimed_by.as_deref() == Some(CLAIMANT)
        && row.claimed_at.is_some()));
    assert_eq!(state(&pool).applied_through_sequence, 2);
}

#[test]
fn a_first_arrival_and_its_later_revisions_net_out_in_one_projection_row() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let record_id = commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        100,
    );
    // A source corrects the figure upwards, then downwards. The second
    // correction is a negative delta, which is exactly why the projection
    // value has no non-negative rule.
    commit_revision_delta(&pool, &fixture, record_id, 2, 130, 100);
    commit_revision_delta(&pool, &fixture, record_id, 3, 70, 130);

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    assert_eq!(claims.len(), 3);
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");

    let rows = projection(&pool);
    assert_eq!(rows.len(), 1, "every delta targets one logical aggregate");
    assert_eq!(
        rows[0].value, 70,
        "100 + 30 - 60 is the current canonical value"
    );
    assert_eq!(
        rows[0].watermark, 3,
        "the row records the latest position that changed it"
    );
}

#[test]
fn absent_optional_dimensions_share_one_logical_aggregate_row() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // Two canonical records on the same day with no publication, no country
    // and no institution. Ordinary SQL uniqueness treats each NULL as
    // distinct, so without NULLS NOT DISTINCT these would silently become two
    // projection rows and split the total.
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-a",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-b",
        DAY_ONE,
        DayDimensions::default(),
        5,
    );
    // A country-scoped record on the same day is a genuinely different
    // aggregate and must not be merged into it.
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-c",
        DAY_ONE,
        DayDimensions {
            country_code: Some("GB"),
            ..DayDimensions::default()
        },
        7,
    );

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");

    let rows = projection(&pool);
    assert_eq!(rows.len(), 2, "exactly two logical aggregates: {rows:?}");
    let unscoped = rows
        .iter()
        .find(|row| row.country_code.is_none())
        .expect("the unscoped aggregate");
    assert_eq!(
        unscoped.value, 15,
        "both absent-dimension deltas landed on one row"
    );
    let scoped = rows
        .iter()
        .find(|row| row.country_code.as_deref() == Some("GB"))
        .expect("the country aggregate");
    assert_eq!(scoped.value, 7);
}

#[test]
fn signed_i64_overflow_and_underflow_fail_closed_and_change_nothing() {
    for (extreme, nudge) in [(i64::MAX, 1_i64), (i64::MIN, -1_i64)] {
        let (_guard, pool) = setup_registry_db();
        let (fixture, _record_id) = fixture_record(&pool, "identity-base");
        let record_id = commit_work_day_delta(
            &pool,
            &fixture,
            "identity-day-1",
            DAY_ONE,
            DayDimensions::default(),
            extreme,
        );
        let first = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
        complete_metric_rollup_deltas(&pool, CLAIMANT, first[0].claim_token).expect("completion");
        assert_eq!(projection(&pool)[0].value, extreme);

        // One more unit in the same direction cannot be represented. It must
        // abort the whole batch rather than wrap into the opposite sign.
        commit_revision_delta(&pool, &fixture, record_id, 2, 0, -nudge);
        let second = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
        let error = complete_metric_rollup_deltas(&pool, CLAIMANT, second[0].claim_token)
            .expect_err("an overflowing application must fail closed");
        assert!(
            rejection(&error).starts_with("Applying this rollup delta would overflow"),
            "unexpected rejection: {error:?}"
        );

        assert_eq!(
            projection(&pool)[0].value,
            extreme,
            "the projected total must be unchanged"
        );
        assert_eq!(
            state(&pool).applied_through_sequence,
            1,
            "and the frontier must stay blocked at the last safe position"
        );
        assert_eq!(
            deltas(&pool)[1].status,
            "CLAIMED",
            "the poison delta is not applied"
        );
    }
}

// ---------------------------------------------------------------------------
// Failure atomicity
// ---------------------------------------------------------------------------

#[test]
fn a_projection_failure_rolls_back_the_whole_batch() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");

    {
        let _injected =
            FailingTrigger::install(&pool, "BEFORE INSERT", "metric_rollup_work_day", None);
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
            .expect_err("a failing projection write must fail the completion");
    }

    assert!(
        projection(&pool).is_empty(),
        "no projection row may survive a rolled-back batch"
    );
    assert_eq!(
        state(&pool).applied_through_sequence,
        0,
        "the watermark must not advance"
    );
    let rows = deltas(&pool);
    assert!(
        rows.iter()
            .all(|row| row.status == "CLAIMED" && row.applied_at.is_none()),
        "and every delta stays claimed and retryable: {rows:?}"
    );

    // With the injection removed, the same token still applies the batch: the
    // failure left it live rather than consuming it.
    let watermark = complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
        .expect("the retry succeeds");
    assert_eq!(watermark.applied_through_sequence, 2);
    assert_eq!(projection(&pool).len(), 2);
}

#[test]
fn a_terminalization_failure_rolls_back_the_projection_and_the_watermark() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");

    {
        let _injected = FailingTrigger::install(
            &pool,
            "BEFORE UPDATE",
            "metric_rollup_delta",
            Some("NEW.status = 'APPLIED'"),
        );
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
            .expect_err("a failing terminalization must fail the completion");
    }

    // The forbidden outcome this guards: a projection changed while its delta
    // remains safely retryable, which would double count on the retry.
    assert!(projection(&pool).is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 0);
    assert_eq!(deltas(&pool)[0].status, "CLAIMED");
}

#[test]
fn a_watermark_failure_rolls_back_the_projection_and_the_terminalization() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");

    {
        let _injected = FailingTrigger::install(
            &pool,
            "BEFORE UPDATE",
            "metric_rollup_work_day_state",
            Some("NEW.applied_through_sequence <> OLD.applied_through_sequence"),
        );
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
            .expect_err("a failing watermark advance must fail the completion");
    }

    // The other forbidden outcome: a delta marked APPLIED whose effect the
    // watermark does not account for.
    assert!(projection(&pool).is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 0);
    assert_eq!(deltas(&pool)[0].status, "CLAIMED");
}

// ---------------------------------------------------------------------------
// Idempotency and watermark safety
// ---------------------------------------------------------------------------

#[test]
fn a_timeout_after_commit_replay_is_read_only_and_returns_the_same_watermark() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let token = claims[0].claim_token;

    let first = complete_metric_rollup_deltas(&pool, CLAIMANT, token).expect("completion");
    let applied_at = deltas(&pool)[0].applied_at;

    // The completion committed but its response never arrived, so the
    // claimant retried with the same token. Twice, to prove the answer is
    // stable rather than merely tolerated once.
    for _ in 0..2 {
        let replay = complete_metric_rollup_deltas(&pool, CLAIMANT, token).expect("replay");
        assert_eq!(
            replay, first,
            "a replay must return the same durable watermark"
        );
    }
    assert_eq!(
        projection(&pool)[0].value,
        10,
        "a replay must not apply the delta a second time"
    );
    assert_eq!(
        deltas(&pool)[0].applied_at,
        applied_at,
        "and must not rewrite the terminal row"
    );

    // A foreign principal replaying the same applied token is still refused:
    // the ownership check precedes the replay decision.
    let error = complete_metric_rollup_deltas(&pool, OTHER_CLAIMANT, token)
        .expect_err("a foreign replay must be rejected");
    assert_eq!(
        rejection(&error),
        "This rollup claim token belongs to a different claimant."
    );
}

#[test]
fn the_watermark_never_advances_across_a_gap() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    for index in 0..3 {
        commit_work_day_delta(
            &pool,
            &fixture,
            &format!("identity-day-{index}"),
            DAY_ONE,
            DayDimensions::default(),
            10,
        );
    }
    // Damage the ordering the way only an out-of-band operation could: remove
    // the delta at position 2. Positions 1 and 3 remain, so a queue that
    // simply took "the oldest unapplied rows" would apply 3 and report a
    // watermark that silently skipped 2.
    exec(
        &pool,
        "DELETE FROM metric_rollup_delta WHERE work_day_sequence = 2",
    );

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    assert_eq!(
        claims
            .iter()
            .map(|claim| claim.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1],
        "the claim must stop at the hole, not reach across it"
    );
    let watermark =
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");
    assert_eq!(watermark.applied_through_sequence, 1);

    // And the frontier stays blocked: position 3 is never claimable while 2
    // is missing, so no read path can be told that its data is applied.
    assert!(claim_metric_rollup_deltas(&pool, CLAIMANT, 50)
        .expect("claim")
        .is_empty());
    assert_eq!(state(&pool).applied_through_sequence, 1);
    assert_eq!(projection(&pool).len(), 1);
}

#[test]
fn deterministic_rebuild_from_canonical_state_matches_the_forward_applied_projection() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let corrected = commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        100,
    );
    commit_revision_delta(&pool, &fixture, corrected, 2, 130, 100);
    commit_revision_delta(&pool, &fixture, corrected, 3, 45, 130);
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions {
            country_code: Some("GB"),
            ..DayDimensions::default()
        },
        20,
    );

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");
    let snapshot = state(&pool).applied_through_sequence;

    // The rebuild: the totals implied by the canonical current revisions
    // alone, with no reference to any delta or projection row.
    #[derive(diesel::QueryableByName, Debug, PartialEq, Eq)]
    struct Rebuilt {
        #[diesel(sql_type = diesel::sql_types::Date)]
        day: NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        country_code: Option<String>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        value: i64,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rebuilt: Vec<Rebuilt> = sql_query(
        "SELECT r.period_start AS day, r.country_code::text AS country_code, \
                SUM(rev.value)::bigint AS value \
         FROM metric_record r \
         JOIN metric_record_revision rev ON rev.record_revision_id = r.current_revision_id \
         WHERE r.reporting_grain = 'DAY' AND r.period_end = r.period_start + 1 \
         GROUP BY r.work_id, r.publication_id, r.platform_id, r.measure_id, \
                  r.period_start, r.country_code, r.institution_id \
         ORDER BY r.period_start",
    )
    .load(&mut connection)
    .expect("Failed to rebuild from canonical state");
    rebuilt.sort_by_key(|row| (row.day, row.value));
    drop(connection);

    let forward: Vec<Rebuilt> = projection(&pool)
        .into_iter()
        .map(|row| Rebuilt {
            day: row.day,
            country_code: row.country_code,
            value: row.value,
        })
        .collect();
    assert_eq!(
        forward, rebuilt,
        "the forward-applied projection must equal a clean rebuild from canonical state"
    );

    // An ingestion that commits after the snapshot takes a position beyond it
    // and stays outside it: it is neither silently treated as rebuilt nor
    // silently counted as applied.
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-3",
        DAY_TWO,
        DayDimensions::default(),
        9,
    );
    let latest = deltas(&pool);
    let newest = latest.last().expect("the newest delta");
    assert!(newest.work_day_sequence.unwrap() > snapshot);
    assert_eq!(newest.status, "PENDING");
    assert_eq!(
        state(&pool).applied_through_sequence,
        snapshot,
        "the durable watermark is not advanced by a rebuild comparison"
    );
    assert_eq!(
        projection(&pool).len(),
        forward.len(),
        "and the new canonical row is not yet projected"
    );
}

// ---------------------------------------------------------------------------
// Lifecycle shape
// ---------------------------------------------------------------------------

#[test]
fn the_durable_claim_state_shapes_are_closed() {
    let (_guard, pool) = setup_registry_db();
    let (_fixture, record_id, revision_id) = day_record_revision(&pool, "identity-a", DAY_ONE);
    insert_delta_row(&pool, None, record_id, revision_id, 10, "PENDING").expect("pending delta");
    let delta_id = deltas(&pool)[0].delta_id;

    // Each of these is one field away from a legal shape, and each is a way
    // the exactly-once protocol could be undermined: a claim without an
    // owner, a claim without a lease, a claim that is already applied, an
    // applied row whose lease is still open, an applied row with no applied
    // time, and an applied row whose claim evidence has been erased.
    let invalid = [
        (
            "CLAIMED without an owner",
            "status = 'CLAIMED', claim_token = gen_random_uuid(), claimed_at = now(), \
             lease_expires_at = now() + interval '900 seconds'",
        ),
        (
            "CLAIMED without a lease",
            "status = 'CLAIMED', claim_token = gen_random_uuid(), claimed_by = 'w', \
             claimed_at = now()",
        ),
        (
            "CLAIMED with a blank owner",
            "status = 'CLAIMED', claim_token = gen_random_uuid(), claimed_by = '   ', \
             claimed_at = now(), lease_expires_at = now() + interval '900 seconds'",
        ),
        (
            "CLAIMED and already applied",
            "status = 'CLAIMED', claim_token = gen_random_uuid(), claimed_by = 'w', \
             claimed_at = now(), lease_expires_at = now() + interval '900 seconds', \
             applied_at = now()",
        ),
        (
            "APPLIED with an open lease",
            "status = 'APPLIED', claim_token = gen_random_uuid(), claimed_by = 'w', \
             claimed_at = now(), lease_expires_at = now() + interval '900 seconds', \
             applied_at = now()",
        ),
        (
            "APPLIED without an applied time",
            "status = 'APPLIED', claim_token = gen_random_uuid(), claimed_by = 'w', \
             claimed_at = now()",
        ),
        (
            "APPLIED without its terminal claim evidence",
            "status = 'APPLIED', applied_at = now()",
        ),
        (
            "PENDING carrying claim evidence",
            "claim_token = gen_random_uuid()",
        ),
        ("a non-positive position", "work_day_sequence = 0"),
    ];
    for (label, assignment) in invalid {
        let mut connection = pool.get().expect("Failed to get DB connection");
        let result = sql_query(format!(
            "UPDATE metric_rollup_delta SET {assignment} WHERE delta_id = '{delta_id}'"
        ))
        .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "{label} must be rejected by the closed lifecycle constraint, got {result:?}"
        );
    }
    assert_eq!(deltas(&pool)[0].status, "PENDING");
}

#[test]
fn the_singleton_progress_row_is_singular_and_ordered() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    for (label, statement) in [
        (
            "a second state row",
            "INSERT INTO metric_rollup_work_day_state \
                 (state_id, next_sequence, applied_through_sequence, watermark_at, updated_at) \
             VALUES (2, 1, 0, now(), now())",
        ),
        (
            "a frontier past the allocated end",
            "UPDATE metric_rollup_work_day_state SET applied_through_sequence = next_sequence",
        ),
        (
            "a negative frontier",
            "UPDATE metric_rollup_work_day_state SET applied_through_sequence = -1",
        ),
        (
            "a zero next position",
            "UPDATE metric_rollup_work_day_state SET next_sequence = 0",
        ),
    ] {
        let result = sql_query(statement).execute(&mut connection);
        assert!(result.is_err(), "{label} must be rejected, got {result:?}");
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_day_state)"),
        1
    );
}

#[test]
fn the_lease_duration_and_batch_bound_are_the_approved_values() {
    // The two numbers the protocol is specified in terms of. They are asserted
    // rather than left implicit because a silent change to either would
    // change how long a crashed claimant blocks the single global frontier,
    // and how far one completion can advance it.
    assert_eq!(METRIC_ROLLUP_LEASE_SECONDS, 900);
    assert_eq!(METRIC_ROLLUP_CLAIM_MAX_BATCH, 50);
}

#[test]
fn a_claim_lease_expires_fifteen_minutes_after_it_is_granted() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );

    let granted = Instant::now();
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let remaining = scalar_i64(
        &pool,
        &format!(
            "(SELECT EXTRACT(EPOCH FROM (lease_expires_at - transaction_timestamp()))::bigint \
              FROM metric_rollup_delta WHERE claim_token = '{}')",
            claims[0].claim_token
        ),
    );
    let elapsed = granted.elapsed();
    assert!(
        remaining > i64::from(METRIC_ROLLUP_LEASE_SECONDS) - 60 - elapsed.as_secs() as i64
            && remaining <= i64::from(METRIC_ROLLUP_LEASE_SECONDS),
        "the server-fixed lease must be about {METRIC_ROLLUP_LEASE_SECONDS} seconds, got {remaining}"
    );
}

// ---------------------------------------------------------------------------
// Migration: populated data, fail-closed guards and guarded rollback
// ---------------------------------------------------------------------------

/// Revert the rollup-application migration, run `work` against the pre-WP4
/// schema, then reapply it and return the reapplication result.
fn with_pre_wp4_schema<F>(pool: &PgPool, work: F) -> Result<(), String>
where
    F: FnOnce(&PgPool),
{
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    revert_through_rollup_application_migration(&mut connection)
        .expect("the rollup-application migration must revert on clean state");
    assert_eq!(
        scalar_i64(
            pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_rollup_delta' \
                AND column_name = 'work_day_sequence')"
        ),
        0,
        "the revert must remove the MET-WP4-01 columns"
    );

    work(pool);

    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[test]
fn the_migration_backfills_existing_pending_deltas_deterministically() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");

    with_pre_wp4_schema(&pool, |pool| {
        // Three work-day deltas and one MONTH-grain delta, committed under the
        // pre-WP4 schema exactly as the merged ingestion coordinator writes
        // them. `created_at` is set explicitly so the deterministic ordering
        // is being asserted rather than accidentally matching insertion order.
        for (index, minute) in [(0, 30), (1, 10), (2, 20)] {
            let record_id = insert_day_record(
                pool,
                &fixture,
                &format!("identity-day-{index}"),
                day(DAY_ONE),
                DayDimensions::default(),
            );
            let revision_id = insert_current_revision(pool, &fixture, record_id, 1, 10);
            insert_delta_row(pool, None, record_id, revision_id, 10, "PENDING")
                .expect("pre-WP4 delta");
            exec(
                pool,
                &format!(
                    "UPDATE metric_rollup_delta \
                     SET created_at = TIMESTAMPTZ '2026-09-01T10:{minute}:00Z' \
                     WHERE record_id = '{record_id}'"
                ),
            );
        }
        // A MONTH-grain record under the same fixture entities: a second
        // `fixture_record` would collide on the shared registry codes.
        let month_record_id = insert_second_record(pool, &fixture, "identity-month");
        let month_revision_id = insert_current_revision(pool, &fixture, month_record_id, 1, 5);
        insert_delta_row(pool, None, month_record_id, month_revision_id, 5, "PENDING")
            .expect("pre-WP4 non-day delta");
    })
    .expect("the migration must apply over populated pre-WP4 data");

    // Positions follow `(created_at, delta_id)`, so the middle-inserted delta
    // with the earliest timestamp is position 1.
    let rows = deltas(&pool);
    let ordered: Vec<(Option<i64>, Timestamp)> = rows
        .iter()
        .map(|row| (row.work_day_sequence, row.created_at))
        .collect();
    let positions: Vec<Option<i64>> = ordered.iter().map(|(position, _)| *position).collect();
    assert_eq!(
        positions,
        vec![Some(1), Some(2), Some(3), None],
        "exactly the three work-day deltas are numbered, in created_at order"
    );
    let mut timestamps: Vec<Timestamp> = ordered[..3].iter().map(|(_, at)| *at).collect();
    let sorted = {
        let mut copy = timestamps.clone();
        copy.sort();
        copy
    };
    timestamps.sort();
    assert_eq!(timestamps, sorted);

    let state = state(&pool);
    assert_eq!(
        state.next_sequence, 4,
        "allocation continues after the backfill"
    );
    assert_eq!(
        state.applied_through_sequence, 0,
        "the migration applies nothing, so the frontier starts at zero"
    );
    assert!(
        rows.iter()
            .all(|row| row.status == "PENDING" && row.applied_at.is_none()),
        "and no delta is marked applied by the migration"
    );
    assert!(
        projection(&pool).is_empty(),
        "and the projection is not populated by the migration"
    );
}

#[test]
fn the_migration_refuses_unexpected_pre_wp4_applied_state() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");

    let outcome = with_pre_wp4_schema(&pool, |pool| {
        let record_id = insert_day_record(
            pool,
            &fixture,
            "identity-day-1",
            day(DAY_ONE),
            DayDimensions::default(),
        );
        let revision_id = insert_current_revision(pool, &fixture, record_id, 1, 10);
        // No approved pre-WP4 runtime can produce this. It is evidence of
        // something unreviewed having written the table, so the migration
        // must refuse rather than guess what it meant.
        insert_delta_row(pool, None, record_id, revision_id, 10, "APPLIED").expect("pre-WP4 delta");
        exec(
            pool,
            "UPDATE metric_rollup_delta SET applied_at = now() WHERE status = 'APPLIED'",
        );
    });
    let message = outcome.expect_err("the migration must fail closed");
    assert!(
        message.contains("already non-PENDING or carry applied_at"),
        "unexpected migration failure: {message}"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_rollup_delta' \
                AND column_name = 'work_day_sequence')"
        ),
        0,
        "a refused migration must leave the schema exactly as it was"
    );

    // Resolve the unexpected state and prove the same migration then applies.
    exec(
        &pool,
        "UPDATE metric_rollup_delta SET status = 'PENDING', applied_at = NULL",
    );
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the migration applies once the state is resolved");
    assert_eq!(deltas(&pool)[0].work_day_sequence, Some(1));
}

#[test]
fn the_migration_refuses_a_day_record_that_is_not_exactly_one_day() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");

    let outcome = with_pre_wp4_schema(&pool, |pool| {
        let record_id = insert_day_record(
            pool,
            &fixture,
            "identity-day-1",
            day(DAY_ONE),
            DayDimensions::default(),
        );
        let revision_id = insert_current_revision(pool, &fixture, record_id, 1, 10);
        insert_delta_row(pool, None, record_id, revision_id, 10, "PENDING").expect("pre-WP4 delta");
        // The projection keys on `day = period_start`, so silently accepting
        // this would attribute a three-day total to a single day.
        exec(
            pool,
            &format!(
                "UPDATE metric_record SET period_end = period_start + 3 \
                 WHERE record_id = '{record_id}'"
            ),
        );
    });
    let message = outcome.expect_err("the migration must fail closed");
    assert!(
        message.contains("not exactly one calendar day"),
        "unexpected migration failure: {message}"
    );

    exec(
        &pool,
        "UPDATE metric_record SET period_end = period_start + 1 WHERE reporting_grain = 'DAY'",
    );
    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the migration applies once the canonical shape is resolved");
}

#[test]
fn the_down_migration_and_reapplication_round_trip_before_activation() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    let delta_value_before = deltas(&pool)[0].delta_value;

    let reapplied = with_pre_wp4_schema(&pool, |pool| {
        // The durable PENDING row survives the rollback in its original
        // MET-WP1-07 shape: discarding an unapplied position is safe
        // precisely because nothing depends on it yet.
        assert_eq!(
            scalar_i64(pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
            1
        );
        for gone in ["metric_rollup_work_day", "metric_rollup_work_day_state"] {
            assert_eq!(
                scalar_i64(
                    pool,
                    &format!(
                        "(SELECT COUNT(*) FROM pg_class \
                          WHERE relnamespace = 'public'::regnamespace \
                            AND relkind = 'r' AND relname = '{gone}')"
                    )
                ),
                0,
                "{gone} must be dropped by the rollback"
            );
        }
        assert_eq!(
            scalar_i64(
                pool,
                "(SELECT COUNT(*) FROM pg_trigger \
                  WHERE tgrelid = 'public.metric_rollup_delta'::regclass \
                    AND NOT tgisinternal)"
            ),
            0,
            "the allocation trigger must be dropped by the rollback"
        );
    });
    reapplied.expect("the migration must reapply after a clean rollback");

    let rows = deltas(&pool);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_value, delta_value_before);
    assert_eq!(
        rows[0].work_day_sequence,
        Some(1),
        "reapplication renumbers the surviving PENDING row deterministically"
    );
    assert_eq!(state(&pool).next_sequence, 2);
    assert_eq!(trigger_names(&pool, "metric_rollup_delta").len(), 1);
}

#[test]
fn the_down_migration_refuses_to_erase_claim_or_application_evidence() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-2",
        DAY_TWO,
        DayDimensions::default(),
        20,
    );
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");

    let database_url = test_db_url();
    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to the test database");

    // An outstanding claim alone is enough: dropping the columns would erase
    // who holds the frontier while leaving the delta looking untouched.
    let refused = revert_through_rollup_application_migration(&mut connection)
        .expect_err("a rollback over an outstanding claim must be refused");
    assert!(
        refused.contains("carry claim or application evidence"),
        "unexpected rollback failure: {refused}"
    );
    // Reverting through MET-WP4-01 peels off the newer additive MET-WP4-03A
    // monthly migration first; it is restored here so the completion below
    // can maintain the monthly projections it now owns.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the newer additive migrations reapply after the refusal");

    // Once applied, the refusal is about the watermark and the projection:
    // dropping them would not undo the projection, only the record of it.
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");
    assert_eq!(projection(&pool).len(), 1);
    let refused = revert_through_rollup_application_migration(&mut connection)
        .expect_err("a rollback over applied progress must be refused");
    assert!(
        refused.contains("durable watermark has advanced"),
        "unexpected rollback failure: {refused}"
    );
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the newer additive migrations reapply after the refusal");

    // The schema, the applied progress and the projection all survive the
    // refusal untouched: a refused rollback is a no-op, not a partial one.
    assert_eq!(state(&pool).applied_through_sequence, 1);
    assert_eq!(projection(&pool).len(), 1);
    assert_eq!(deltas(&pool)[0].status, "APPLIED");
    assert_eq!(trigger_names(&pool, "metric_rollup_delta").len(), 1);
}

/// A held transaction that keeps its statements' locks until released.
struct HeldTransaction {
    release: Sender<()>,
    handle: JoinHandle<()>,
}

impl HeldTransaction {
    fn start(statement: String) -> Self {
        let (release, wait): (Sender<()>, Receiver<()>) = channel();
        let (ready_tx, ready_rx) = channel();
        let handle = thread::spawn(move || {
            let mut connection = PgConnection::establish(&test_db_url())
                .expect("Failed to connect to the test database");
            connection
                .transaction::<_, DieselError, _>(|connection| {
                    sql_query(&statement).execute(connection)?;
                    ready_tx.send(()).expect("ready signal");
                    wait.recv().expect("release signal");
                    Ok(())
                })
                .expect("the held transaction must commit");
        });
        ready_rx.recv().expect("the held transaction must start");
        HeldTransaction { release, handle }
    }

    fn commit(self) {
        self.release.send(()).expect("release");
        self.handle.join().expect("join");
    }
}

#[test]
fn an_allocating_ingestion_waits_behind_an_in_flight_claim() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    commit_work_day_delta(
        &pool,
        &fixture,
        "identity-day-1",
        DAY_ONE,
        DayDimensions::default(),
        10,
    );

    // Hold the singleton progress row exactly as a claim or completion does.
    let held = HeldTransaction::start(
        "SELECT 1 FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE".to_string(),
    );

    let record_id = insert_day_record(
        &pool,
        &fixture,
        "identity-day-2",
        day(DAY_TWO),
        DayDimensions::default(),
    );
    let revision_id = insert_current_revision(&pool, &fixture, record_id, 1, 20);
    let ingest_pool = Arc::clone(&pool);
    let ingestion = thread::spawn(move || {
        insert_delta_row(&ingest_pool, None, record_id, revision_id, 20, "PENDING")
    });

    // The allocation is blocked on the row lock, not spinning or failing.
    thread::sleep(Duration::from_millis(250));
    assert!(
        !ingestion.is_finished(),
        "the allocation must wait for the lock"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_delta)"),
        1
    );

    held.commit();
    ingestion
        .join()
        .expect("join")
        .expect("the allocation completes once the lock is released");
    assert_eq!(
        deltas(&pool)
            .iter()
            .filter_map(|row| row.work_day_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2],
        "and it takes the next contiguous position"
    );
}

// ===========================================================================
// MET-WP4-03A: derived monthly serving projections
// ===========================================================================

/// The Diesel migration version of `thoth-api/migrations/20260923_v1.9.0`.
const MET_WP4_03A_MIGRATION_VERSION: &str = "20260923";

/// The fixed seed of the differential fixture, and the cardinality it
/// generates. Both are pinned so a change to either the generator or the
/// seed is a visible change to the evidence.
const DIFFERENTIAL_SEED: u64 = 20_260_923;

/// The third calendar day of the fixture month.
const DAY_THREE: (i32, u32, u32) = (2026, 3, 3);
/// The first day of the month after the fixture month.
const APRIL_ONE: (i32, u32, u32) = (2026, 4, 1);

fn month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, 1).expect("a valid fixture month")
}

fn first_of_month(day: NaiveDate) -> NaiveDate {
    day.with_day(1).expect("the first day of a month")
}

/// The first day of the `n`th month from January 2021, for keys that must
/// fall in distinct months.
fn distinct_month(n: u32) -> NaiveDate {
    month(2021 + (n / 12) as i32, n % 12 + 1)
}

fn ymd(day: NaiveDate) -> (i32, u32, u32) {
    (day.year(), day.month(), day.day())
}

// ---------------------------------------------------------------------------
// Dimension shorthands over the shared fixture
// ---------------------------------------------------------------------------

const AGG: DayDimensions = DayDimensions {
    publication: false,
    country_code: None,
    institution: false,
};
const GB: DayDimensions = DayDimensions {
    publication: false,
    country_code: Some("GB"),
    institution: false,
};
const US: DayDimensions = DayDimensions {
    publication: false,
    country_code: Some("US"),
    institution: false,
};
const INST: DayDimensions = DayDimensions {
    publication: false,
    country_code: None,
    institution: true,
};
const PUB: DayDimensions = DayDimensions {
    publication: true,
    country_code: None,
    institution: false,
};
const GB_INST: DayDimensions = DayDimensions {
    publication: false,
    country_code: Some("GB"),
    institution: true,
};
const PUB_GB: DayDimensions = DayDimensions {
    publication: true,
    country_code: Some("GB"),
    institution: false,
};
const PUB_INST: DayDimensions = DayDimensions {
    publication: true,
    country_code: None,
    institution: true,
};
const PUB_GB_INST: DayDimensions = DayDimensions {
    publication: true,
    country_code: Some("GB"),
    institution: true,
};

/// Commit one `PENDING` work-day delta over the shared fixture under a
/// fresh identity, and return its canonical record id.
fn day_delta(
    pool: &PgPool,
    fixture: &RecordFixture,
    on: (i32, u32, u32),
    dimensions: DayDimensions,
    value: i64,
) -> Uuid {
    commit_work_day_delta(
        pool,
        fixture,
        &format!("identity-{}", Uuid::new_v4().simple()),
        on,
        dimensions,
        value,
    )
}

/// Claim and complete until the frontier is exhausted, in batches of the
/// maximum size, and return the durable watermark.
fn apply_everything(pool: &PgPool) -> i64 {
    loop {
        let claims = claim_metric_rollup_deltas(pool, CLAIMANT, METRIC_ROLLUP_CLAIM_MAX_BATCH)
            .expect("claim");
        let Some(first) = claims.first() else {
            return state(pool).applied_through_sequence;
        };
        complete_metric_rollup_deltas(pool, CLAIMANT, first.claim_token).expect("completion");
    }
}

// ---------------------------------------------------------------------------
// Monthly row readers
// ---------------------------------------------------------------------------

/// Every resolved monthly total, in a deterministic order.
pub(crate) fn month_rows(pool: &PgPool) -> Vec<MetricRollupWorkMonth> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupWorkMonth> = metric_rollup_work_month::table
        .load(&mut connection)
        .expect("Failed to load the monthly projection");
    rows.sort_by_key(|row| (row.month_start, row.work_id, row.publication_id, row.value));
    rows
}

fn country_rows(pool: &PgPool) -> Vec<MetricRollupWorkCountryMonth> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupWorkCountryMonth> = metric_rollup_work_country_month::table
        .load(&mut connection)
        .expect("Failed to load the monthly country projection");
    rows.sort_by_key(|row| {
        (
            row.month_start,
            row.work_id,
            row.publication_id,
            row.country_code.clone(),
            row.value,
        )
    });
    rows
}

fn institution_rows(pool: &PgPool) -> Vec<MetricRollupWorkInstitutionMonth> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupWorkInstitutionMonth> =
        metric_rollup_work_institution_month::table
            .load(&mut connection)
            .expect("Failed to load the monthly institution projection");
    rows.sort_by_key(|row| {
        (
            row.month_start,
            row.work_id,
            row.publication_id,
            row.institution_id,
            row.value,
        )
    });
    rows
}

fn ambiguity_rows(pool: &PgPool) -> Vec<MetricRollupWorkMonthAmbiguity> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut rows: Vec<MetricRollupWorkMonthAmbiguity> = metric_rollup_work_month_ambiguity::table
        .load(&mut connection)
        .expect("Failed to load the monthly ambiguity state");
    rows.sort_by_key(|row| (row.month_start, row.work_id));
    rows
}

/// `(month_start, publication_id, value, requires_country, requires_institution, watermark)`.
type TotalFact = (NaiveDate, Option<Uuid>, i64, bool, bool, i64);
/// `(month_start, publication_id, country_code, value, requires_institution, watermark)`.
type CountryFact = (NaiveDate, Option<Uuid>, String, i64, bool, i64);
/// `(month_start, publication_id, institution_id, value, requires_country, watermark)`.
type InstitutionFact = (NaiveDate, Option<Uuid>, Uuid, i64, bool, i64);
/// `(month_start, total_ambiguous, country_ambiguous, institution_ambiguous, watermark)`.
type AmbiguityFact = (NaiveDate, bool, bool, bool, i64);

fn totals(pool: &PgPool) -> Vec<TotalFact> {
    month_rows(pool)
        .into_iter()
        .map(|row| {
            (
                row.month_start,
                row.publication_id,
                row.value,
                row.requires_country_coverage,
                row.requires_institution_coverage,
                row.watermark,
            )
        })
        .collect()
}

fn countries(pool: &PgPool) -> Vec<CountryFact> {
    country_rows(pool)
        .into_iter()
        .map(|row| {
            (
                row.month_start,
                row.publication_id,
                row.country_code,
                row.value,
                row.requires_institution_coverage,
                row.watermark,
            )
        })
        .collect()
}

fn institutions(pool: &PgPool) -> Vec<InstitutionFact> {
    institution_rows(pool)
        .into_iter()
        .map(|row| {
            (
                row.month_start,
                row.publication_id,
                row.institution_id,
                row.value,
                row.requires_country_coverage,
                row.watermark,
            )
        })
        .collect()
}

fn ambiguities(pool: &PgPool) -> Vec<AmbiguityFact> {
    ambiguity_rows(pool)
        .into_iter()
        .map(|row| {
            (
                row.month_start,
                row.total_ambiguous,
                row.country_ambiguous,
                row.institution_ambiguous,
                row.watermark,
            )
        })
        .collect()
}

fn country(code: &str) -> String {
    code.to_string()
}

// ---------------------------------------------------------------------------
// Logical monthly state and the independent per-day oracle
// ---------------------------------------------------------------------------

/// `(work_id, publication_id, platform_id, measure_id, month_start)`.
type TotalIdentity = (Uuid, Option<Uuid>, Uuid, Uuid, NaiveDate);
/// `(value, requires_country_coverage, requires_institution_coverage, watermark)`.
type TotalValue = (i64, bool, bool, i64);
/// The total identity plus `country_code`.
type CountryIdentity = (Uuid, Option<Uuid>, Uuid, Uuid, NaiveDate, String);
/// The total identity plus `institution_id`.
type InstitutionIdentity = (Uuid, Option<Uuid>, Uuid, Uuid, NaiveDate, Uuid);
/// `(value, requires_<other dimension>_coverage, watermark)`.
type DimensionValue = (i64, bool, i64);
/// `(work_id, platform_id, measure_id, month_start)`.
type AmbiguityIdentity = (Uuid, Uuid, Uuid, NaiveDate);
/// `(total_ambiguous, country_ambiguous, institution_ambiguous, watermark)`.
type AmbiguityValue = (bool, bool, bool, i64);

/// The four monthly datasets by logical identity, without surrogate ids, so
/// incrementally maintained state, rebuilt state and the oracle compare
/// exactly on values, dependency flags, ambiguity flags and watermarks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MonthState {
    totals: BTreeMap<TotalIdentity, TotalValue>,
    countries: BTreeMap<CountryIdentity, DimensionValue>,
    institutions: BTreeMap<InstitutionIdentity, DimensionValue>,
    ambiguity: BTreeMap<AmbiguityIdentity, AmbiguityValue>,
}

/// The persisted monthly state.
fn month_state(pool: &PgPool) -> MonthState {
    MonthState {
        totals: month_rows(pool)
            .into_iter()
            .map(|row| {
                (
                    (
                        row.work_id,
                        row.publication_id,
                        row.platform_id,
                        row.measure_id,
                        row.month_start,
                    ),
                    (
                        row.value,
                        row.requires_country_coverage,
                        row.requires_institution_coverage,
                        row.watermark,
                    ),
                )
            })
            .collect(),
        countries: country_rows(pool)
            .into_iter()
            .map(|row| {
                (
                    (
                        row.work_id,
                        row.publication_id,
                        row.platform_id,
                        row.measure_id,
                        row.month_start,
                        row.country_code,
                    ),
                    (row.value, row.requires_institution_coverage, row.watermark),
                )
            })
            .collect(),
        institutions: institution_rows(pool)
            .into_iter()
            .map(|row| {
                (
                    (
                        row.work_id,
                        row.publication_id,
                        row.platform_id,
                        row.measure_id,
                        row.month_start,
                        row.institution_id,
                    ),
                    (row.value, row.requires_country_coverage, row.watermark),
                )
            })
            .collect(),
        ambiguity: ambiguity_rows(pool)
            .into_iter()
            .map(|row| {
                (
                    (
                        row.work_id,
                        row.platform_id,
                        row.measure_id,
                        row.month_start,
                    ),
                    (
                        row.total_ambiguous,
                        row.country_ambiguous,
                        row.institution_ambiguous,
                        row.watermark,
                    ),
                )
            })
            .collect(),
    }
}

/// The `MET-WP4-02` presence mask: publication = 4, country = 2,
/// institution = 1.
fn presence_mask(row: &MetricRollupWorkDay) -> u8 {
    (u8::from(row.publication_id.is_some()) << 2)
        | (u8::from(row.country_code.is_some()) << 1)
        | u8::from(row.institution_id.is_some())
}

/// The unique least represented mask containing `target` under set
/// inclusion: `Ok(None)` when no represented mask contains the target,
/// `Ok(Some(mask))` when exactly one minimal mask exists, and `Err(minimal)`
/// naming the incomparable minimal masks otherwise.
///
/// This is the generic rule, computed by minimality over the represented
/// set, rather than the fixed case table the SQL uses; that is what makes it
/// an independent oracle.
fn least_representation(represented: &BTreeSet<u8>, target: u8) -> Result<Option<u8>, Vec<u8>> {
    let candidates: Vec<u8> = represented
        .iter()
        .copied()
        .filter(|mask| mask & target != 0)
        .collect();
    let minimal: Vec<u8> = candidates
        .iter()
        .copied()
        .filter(|mask| {
            !candidates
                .iter()
                .any(|other| other != mask && other & mask == *other)
        })
        .collect();
    match minimal.as_slice() {
        [] => Ok(None),
        [only] => Ok(Some(*only)),
        _ => Err(minimal),
    }
}

/// The monthly state the approved semantics imply for the given work-day
/// rows, derived per base cell in Rust and independently of the SQL.
fn oracle_month_state(days: &[MetricRollupWorkDay]) -> MonthState {
    let mut cells: BTreeMap<(Uuid, Uuid, Uuid, NaiveDate), Vec<&MetricRollupWorkDay>> =
        BTreeMap::new();
    for row in days {
        cells
            .entry((row.work_id, row.platform_id, row.measure_id, row.day))
            .or_default()
            .push(row);
    }

    let mut state = MonthState::default();
    for ((work_id, platform_id, measure_id, day), rows) in cells {
        let month_start = first_of_month(day);
        let represented: BTreeSet<u8> = rows.iter().map(|row| presence_mask(row)).collect();
        let mut flags = (false, false, false);
        let mut ambiguity_watermark = 0_i64;

        // Totals: Amendment 6 exactly.
        let total = if represented.contains(&0) {
            Some(0)
        } else if represented.len() == 1 {
            represented.first().copied()
        } else {
            None
        };
        match total {
            Some(mask) => {
                for row in rows.iter().filter(|row| presence_mask(row) == mask) {
                    let entry = state
                        .totals
                        .entry((
                            work_id,
                            row.publication_id,
                            platform_id,
                            measure_id,
                            month_start,
                        ))
                        .or_insert((0, false, false, 0));
                    entry.0 = entry.0.checked_add(row.value).expect("oracle overflow");
                    entry.1 |= mask & 2 != 0;
                    entry.2 |= mask & 1 != 0;
                    entry.3 = entry.3.max(row.watermark);
                }
            }
            None => {
                flags.0 = true;
                ambiguity_watermark = rows.iter().map(|row| row.watermark).max().unwrap_or(0);
            }
        }

        // Countries: unique least mask containing country.
        match least_representation(&represented, 2) {
            Ok(Some(mask)) => {
                for row in rows.iter().filter(|row| presence_mask(row) == mask) {
                    let entry = state
                        .countries
                        .entry((
                            work_id,
                            row.publication_id,
                            platform_id,
                            measure_id,
                            month_start,
                            row.country_code.clone().expect("a country row"),
                        ))
                        .or_insert((0, false, 0));
                    entry.0 = entry.0.checked_add(row.value).expect("oracle overflow");
                    entry.1 |= mask & 1 != 0;
                    entry.2 = entry.2.max(row.watermark);
                }
            }
            Ok(None) => {}
            Err(minimal) => {
                flags.1 = true;
                ambiguity_watermark = ambiguity_watermark.max(
                    rows.iter()
                        .filter(|row| minimal.contains(&presence_mask(row)))
                        .map(|row| row.watermark)
                        .max()
                        .unwrap_or(0),
                );
            }
        }

        // Institutions: unique least mask containing institution.
        match least_representation(&represented, 1) {
            Ok(Some(mask)) => {
                for row in rows.iter().filter(|row| presence_mask(row) == mask) {
                    let entry = state
                        .institutions
                        .entry((
                            work_id,
                            row.publication_id,
                            platform_id,
                            measure_id,
                            month_start,
                            row.institution_id.expect("an institution row"),
                        ))
                        .or_insert((0, false, 0));
                    entry.0 = entry.0.checked_add(row.value).expect("oracle overflow");
                    entry.1 |= mask & 2 != 0;
                    entry.2 = entry.2.max(row.watermark);
                }
            }
            Ok(None) => {}
            Err(minimal) => {
                flags.2 = true;
                ambiguity_watermark = ambiguity_watermark.max(
                    rows.iter()
                        .filter(|row| minimal.contains(&presence_mask(row)))
                        .map(|row| row.watermark)
                        .max()
                        .unwrap_or(0),
                );
            }
        }

        if flags.0 || flags.1 || flags.2 {
            let entry = state
                .ambiguity
                .entry((work_id, platform_id, measure_id, month_start))
                .or_insert((false, false, false, 0));
            entry.0 |= flags.0;
            entry.1 |= flags.1;
            entry.2 |= flags.2;
            entry.3 = entry.3.max(ambiguity_watermark);
        }
    }
    state
}

/// Every monthly watermark must sit at or below the durable frontier.
fn assert_watermarks_within(state: &MonthState, frontier: i64) {
    let watermarks = state
        .totals
        .values()
        .map(|value| value.3)
        .chain(state.countries.values().map(|value| value.2))
        .chain(state.institutions.values().map(|value| value.2))
        .chain(state.ambiguity.values().map(|value| value.3));
    for watermark in watermarks {
        assert!(
            watermark > 0 && watermark <= frontier,
            "a monthly watermark {watermark} must be positive and at most the frontier {frontier}"
        );
    }
}

// ---------------------------------------------------------------------------
// Extra fixture entities and explicit-dimension records
// ---------------------------------------------------------------------------

/// A second work under the fixture imprint.
fn insert_extra_work(pool: &PgPool, fixture: &RecordFixture) -> Uuid {
    let work_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
         SELECT $1, 'monograph', 'forthcoming', imprint_id, 1 FROM work WHERE work_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(work_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
    .execute(&mut connection)
    .expect("Failed to insert the extra work");
    work_id
}

/// A further publication of `work_id`, of a type the work does not have yet.
fn insert_extra_publication(pool: &PgPool, work_id: Uuid, publication_type: &str) -> Uuid {
    let publication_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO publication (publication_id, publication_type, work_id) \
         VALUES ($1, '{publication_type}', $2)"
    ))
    .bind::<diesel::sql_types::Uuid, _>(publication_id)
    .bind::<diesel::sql_types::Uuid, _>(work_id)
    .execute(&mut connection)
    .expect("Failed to insert the extra publication");
    publication_id
}

fn insert_extra_institution(pool: &PgPool) -> Uuid {
    let institution_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("INSERT INTO institution (institution_id, institution_name) VALUES ($1, 'Other')")
        .bind::<diesel::sql_types::Uuid, _>(institution_id)
        .execute(&mut connection)
        .expect("Failed to insert the extra institution");
    institution_id
}

/// The optional dimensions of one generated record, with explicit entity
/// ids so several publications and institutions can be represented.
#[derive(Clone, Copy)]
struct CellDimensions {
    publication_id: Option<Uuid>,
    country_code: Option<&'static str>,
    institution_id: Option<Uuid>,
}

/// Commit one `PENDING` work-day delta of `value` for `work_id` with
/// explicit dimensions, and return its canonical record id.
fn commit_cell_delta(
    pool: &PgPool,
    fixture: &RecordFixture,
    work_id: Uuid,
    on: NaiveDate,
    dimensions: CellDimensions,
    value: i64,
) -> Uuid {
    let record_id = Uuid::new_v4();
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, publication_id, platform_id, measure_id, \
                  period_start, period_end, reporting_grain, country_code, institution_id, \
                  winning_source_account_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7 + 1, 'DAY', $8, $9, $10)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Text, _>(format!("identity-{}", record_id.simple()))
        .bind::<diesel::sql_types::Uuid, _>(work_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(dimensions.publication_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .bind::<diesel::sql_types::Date, _>(on)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(dimensions.country_code)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(dimensions.institution_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .execute(&mut connection)
        .expect("Failed to insert the generated work-day record");
    }
    let revision_id = insert_current_revision(pool, fixture, record_id, 1, value);
    insert_delta_row(pool, None, record_id, revision_id, value, "PENDING")
        .expect("Failed to insert the generated work-day delta");
    record_id
}

/// A small deterministic generator (xorshift64*), so the differential
/// fixture is reproducible from its seed alone.
struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

// ---------------------------------------------------------------------------
// Statement capture through Diesel instrumentation
// ---------------------------------------------------------------------------

/// Records the text of every statement a pooled connection starts.
#[derive(Debug)]
struct StatementLog(Arc<Mutex<Vec<String>>>);

impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for StatementLog {
    fn on_acquire(&self, connection: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        let log = Arc::clone(&self.0);
        connection.set_instrumentation(move |event: InstrumentationEvent<'_>| {
            if let InstrumentationEvent::StartQuery { query, .. } = event {
                log.lock().expect("statement log").push(query.to_string());
            }
        });
        Ok(())
    }
}

/// A one-connection pool whose statements are captured.
fn logging_pool() -> (Arc<PgPool>, Arc<Mutex<Vec<String>>>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(StatementLog(Arc::clone(&log))))
        .build(ConnectionManager::<PgConnection>::new(test_db_url()))
        .expect("Failed to create the logging pool");
    (Arc::new(pool), log)
}

/// The positions, within the fixed monthly statement list, of the monthly
/// statements captured so far, in execution order.
fn captured_month_statements(log: &Mutex<Vec<String>>) -> Vec<usize> {
    log.lock()
        .expect("statement log")
        .iter()
        .filter_map(|text| {
            MONTH_MAINTENANCE_STATEMENTS
                .iter()
                .position(|statement| text.starts_with(statement))
        })
        .collect()
}

/// The number of captured statements that touch the work-day projection
/// row by row: the per-delta `SELECT ... FOR UPDATE` and upsert pairs.
fn captured_day_statements(log: &Mutex<Vec<String>>) -> usize {
    log.lock()
        .expect("statement log")
        .iter()
        .filter(|text| {
            text.starts_with("SELECT value \\n         FROM public.metric_rollup_work_day")
                || text.starts_with("SELECT value FROM public.metric_rollup_work_day")
                || text.starts_with("INSERT INTO public.metric_rollup_work_day ")
        })
        .count()
}

// ---------------------------------------------------------------------------
// Resolve day first: total, country and institution representation
// ---------------------------------------------------------------------------

#[test]
fn an_undimensioned_day_row_is_authoritative_over_country_alternatives_for_the_total() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    day_delta(&pool, &fixture, DAY_ONE, US, 4);
    apply_everything(&pool);

    // The aggregate is the total; the country rows are not added to it.
    // The total's watermark is the aggregate row's own position, 1, even
    // though positions 2 and 3 touched the same month: ignored
    // representations do not advance a row they do not feed.
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, false, false, 1)]
    );
    assert_eq!(
        countries(&pool),
        vec![
            (month(2026, 3), None, country("GB"), 6, false, 2),
            (month(2026, 3), None, country("US"), 4, false, 3),
        ]
    );
    assert!(institutions(&pool).is_empty());
    assert!(ambiguities(&pool).is_empty());
}

#[test]
fn one_consistent_dimensioned_mask_sums_into_the_total_and_marks_its_dependency() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    day_delta(&pool, &fixture, DAY_ONE, US, 4);
    apply_everything(&pool);

    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, true, false, 2)],
        "one consistent country mask sums, and the total depends on country coverage"
    );
    assert_eq!(
        countries(&pool),
        vec![
            (month(2026, 3), None, country("GB"), 6, false, 1),
            (month(2026, 3), None, country("US"), 4, false, 2),
        ]
    );
    assert!(ambiguities(&pool).is_empty());
}

#[test]
fn incompatible_total_masks_are_recorded_as_total_ambiguous_without_blocking_progress() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    day_delta(&pool, &fixture, DAY_ONE, INST, 4);
    let frontier = apply_everything(&pool);

    assert_eq!(frontier, 2, "ambiguity never stops the frontier");
    assert!(
        totals(&pool).is_empty(),
        "nothing is guessed or summed across masks"
    );
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), true, false, false, 2)],
        "the ambiguity watermark spans every row of the ambiguous cell"
    );
    // Each target still resolves on its own: country from the one country
    // mask, institution from the one institution mask.
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 6, false, 1)]
    );
    assert_eq!(
        institutions(&pool),
        vec![(month(2026, 3), None, fixture.institution_id, 4, false, 2)]
    );
}

#[test]
fn the_country_projection_selects_the_unique_least_country_representation() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // March: {C,I}, {P,C,I} and {C} are all represented. {C} is the unique
    // least country mask, so only it contributes and it depends on nothing.
    day_delta(&pool, &fixture, DAY_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, DAY_ONE, PUB_GB_INST, 2);
    day_delta(&pool, &fixture, DAY_ONE, GB, 5);
    // April: only {C,I} and {P,C,I}. {C,I} is least: institutions are summed
    // away, the publication is not retained, and institution coverage is
    // now a dependency.
    day_delta(&pool, &fixture, APRIL_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, APRIL_ONE, PUB_GB_INST, 2);
    apply_everything(&pool);

    assert_eq!(
        countries(&pool),
        vec![
            (month(2026, 3), None, country("GB"), 5, false, 3),
            (month(2026, 4), None, country("GB"), 3, true, 4),
        ]
    );
    // Institution: {C,I} is the least institution mask in both months.
    assert_eq!(
        institutions(&pool),
        vec![
            (month(2026, 3), None, fixture.institution_id, 3, true, 1),
            (month(2026, 4), None, fixture.institution_id, 3, true, 4),
        ]
    );
    // Neither month has an undimensioned row or a single mask, so both
    // totals are ambiguous; nothing else is.
    assert!(totals(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![
            (month(2026, 3), true, false, false, 3),
            (month(2026, 4), true, false, false, 5),
        ]
    );
}

#[test]
fn the_institution_projection_selects_the_unique_least_institution_representation() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // March: {P,I}, {P,C,I} and {I}. {I} is least.
    day_delta(&pool, &fixture, DAY_ONE, PUB_INST, 3);
    day_delta(&pool, &fixture, DAY_ONE, PUB_GB_INST, 2);
    day_delta(&pool, &fixture, DAY_ONE, INST, 5);
    // April: {C,I} and {P,C,I}. {C,I} is least.
    day_delta(&pool, &fixture, APRIL_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, APRIL_ONE, PUB_GB_INST, 2);
    apply_everything(&pool);

    assert_eq!(
        institutions(&pool),
        vec![
            (month(2026, 3), None, fixture.institution_id, 5, false, 3),
            (month(2026, 4), None, fixture.institution_id, 3, true, 4),
        ]
    );
    // Country in March: only {P,C,I} carries a country, so it is the least
    // and the publication is retained.
    assert_eq!(
        countries(&pool),
        vec![
            (
                month(2026, 3),
                Some(fixture.publication_id),
                country("GB"),
                2,
                true,
                2
            ),
            (month(2026, 4), None, country("GB"), 3, true, 4),
        ]
    );
    assert!(totals(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![
            (month(2026, 3), true, false, false, 3),
            (month(2026, 4), true, false, false, 5),
        ]
    );
}

#[test]
fn incomparable_country_masks_are_country_ambiguous_while_the_total_stays_valid() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, DAY_ONE, PUB_GB, 2);
    apply_everything(&pool);

    // The aggregate keeps the total valid and independent of any dimension.
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, false, false, 1)]
    );
    // {C,I} and {P,C} are incomparable minima: no country row, and the
    // ambiguity watermark spans exactly those two rows.
    assert!(countries(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), false, true, false, 3)]
    );
    // Institution is unaffected: {C,I} is the only institution mask.
    assert_eq!(
        institutions(&pool),
        vec![(month(2026, 3), None, fixture.institution_id, 3, true, 2)]
    );
}

#[test]
fn incomparable_institution_masks_are_institution_ambiguous_while_the_total_stays_valid() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, DAY_ONE, PUB_INST, 2);
    apply_everything(&pool);

    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, false, false, 1)]
    );
    assert!(institutions(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), false, false, true, 3)]
    );
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 3, true, 2)]
    );
}

#[test]
fn a_month_can_carry_ambiguity_with_no_value_row_in_any_section() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // {C,I}, {P,C} and {P,I}: no aggregate and three masks (total
    // ambiguous), incomparable country minima {C,I}/{P,C}, incomparable
    // institution minima {C,I}/{P,I}.
    day_delta(&pool, &fixture, DAY_ONE, GB_INST, 3);
    day_delta(&pool, &fixture, DAY_ONE, PUB_GB, 2);
    day_delta(&pool, &fixture, DAY_ONE, PUB_INST, 1);
    let frontier = apply_everything(&pool);

    assert_eq!(frontier, 3);
    assert!(totals(&pool).is_empty());
    assert!(countries(&pool).is_empty());
    assert!(institutions(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), true, true, true, 3)],
        "sparse ambiguity state exists on its own, for the later reader to honour"
    );
}

#[test]
fn different_daily_masks_in_one_month_are_resolved_per_day_and_then_summed() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // The raw-month design was falsified on exactly this shape: day one is
    // an aggregate 10, day two is country-only 6 + 4. Raw compaction would
    // give 10; the resolved month is 20.
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_TWO, GB, 6);
    day_delta(&pool, &fixture, DAY_TWO, US, 4);
    apply_everything(&pool);

    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 20, true, false, 3)],
        "day two's country representation is a dependency of the whole monthly row"
    );
    assert_eq!(
        countries(&pool),
        vec![
            (month(2026, 3), None, country("GB"), 6, false, 2),
            (month(2026, 3), None, country("US"), 4, false, 3),
        ],
        "day one has no country representation and contributes no country value"
    );
    assert!(ambiguities(&pool).is_empty());
}

#[test]
fn null_and_publication_specific_monthly_contributions_coexist_and_are_additive() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_TWO, PUB, 7);
    day_delta(&pool, &fixture, DAY_THREE, PUB, 3);
    apply_everything(&pool);

    // Two monthly rows, one per resolved publication identity. No
    // month-level precedence is re-run between them, and the
    // publication-specific row sums its two days.
    assert_eq!(
        totals(&pool),
        vec![
            (month(2026, 3), None, 10, false, false, 1),
            (
                month(2026, 3),
                Some(fixture.publication_id),
                10,
                false,
                false,
                3
            ),
        ]
    );
    assert!(countries(&pool).is_empty());
    assert!(ambiguities(&pool).is_empty());
}

// ---------------------------------------------------------------------------
// Transitions, revisions and retractions
// ---------------------------------------------------------------------------

#[test]
fn a_country_only_day_later_gains_an_aggregate_and_the_total_switches_to_it() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 6, true, false, 1)]
    );
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 6, false, 1)]
    );

    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    apply_everything(&pool);

    // The aggregate now feeds the total, so its value, dependency and
    // watermark all move; the country row is fed by the same row as before,
    // so its watermark stays at 1.
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, false, false, 2)]
    );
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 6, false, 1)]
    );
}

#[test]
fn an_aggregate_revision_and_retraction_flow_through_to_the_month() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let record_id = day_delta(&pool, &fixture, DAY_ONE, AGG, 100);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 100, false, false, 1)]
    );

    commit_revision_delta(&pool, &fixture, record_id, 2, 130, 100);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 130, false, false, 2)]
    );

    // A retraction subtracts the whole applied value. The day row and the
    // monthly row are both retained at zero, because "counted, and zero" is
    // different from "never counted".
    commit_revision_delta(&pool, &fixture, record_id, 3, 0, 130);
    apply_everything(&pool);
    assert_eq!(projection(&pool)[0].value, 0);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 0, false, false, 3)]
    );
}

#[test]
fn a_country_day_gaining_a_country_institution_row_becomes_ambiguous_until_an_aggregate_arrives() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 6, true, false, 1)]
    );

    // {C} and {C,I} without an aggregate: the total is ambiguous; the
    // country row stays on {C} (least) and its watermark stays at 1; the
    // institution projection appears from {C,I}.
    day_delta(&pool, &fixture, DAY_ONE, GB_INST, 3);
    apply_everything(&pool);
    assert!(totals(&pool).is_empty());
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), true, false, false, 2)]
    );
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 6, false, 1)]
    );
    assert_eq!(
        institutions(&pool),
        vec![(month(2026, 3), None, fixture.institution_id, 3, true, 2)]
    );

    // An aggregate resolves the total, and a cleared ambiguity row is
    // removed rather than kept for its old watermark.
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 10, false, false, 3)]
    );
    assert!(ambiguities(&pool).is_empty());
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 6, false, 1)]
    );
}

#[test]
fn a_publication_specific_revision_moves_only_its_own_monthly_row() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    let record_id = day_delta(&pool, &fixture, DAY_TWO, PUB, 7);
    apply_everything(&pool);

    commit_revision_delta(&pool, &fixture, record_id, 2, 9, 7);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![
            (month(2026, 3), None, 10, false, false, 1),
            (
                month(2026, 3),
                Some(fixture.publication_id),
                9,
                false,
                false,
                3
            ),
        ]
    );
}

#[test]
fn a_signed_downward_revision_reduces_the_month_without_wrapping() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let record_id = day_delta(&pool, &fixture, DAY_ONE, AGG, 100);
    day_delta(&pool, &fixture, DAY_TWO, AGG, 5);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 105, false, false, 2)]
    );

    commit_revision_delta(&pool, &fixture, record_id, 2, 70, 100);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 75, false, false, 3)]
    );
}

#[test]
fn a_zero_valued_resolved_day_still_contributes_its_representation_and_watermark() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_TWO, GB, 4);
    let record_id = day_delta(&pool, &fixture, DAY_ONE, AGG, 5);
    commit_revision_delta(&pool, &fixture, record_id, 2, 0, 5);
    apply_everything(&pool);

    // Day one nets to zero within the batch and is retained. It still
    // resolves (aggregate), still contributes to the month, and its position
    // 3 is the monthly watermark: dropping zero rows would report 1.
    assert_eq!(
        projection(&pool)
            .iter()
            .map(|row| (row.day, row.value))
            .collect::<Vec<_>>(),
        vec![(day(DAY_ONE), 0), (day(DAY_TWO), 4)]
    );
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 4, true, false, 3)]
    );
}

#[test]
fn several_deltas_on_one_base_cell_in_one_batch_resolve_once() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let record_id = day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    commit_revision_delta(&pool, &fixture, record_id, 2, 15, 10);
    day_delta(&pool, &fixture, DAY_ONE, AGG, 3);

    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    assert_eq!(claims.len(), 3);
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");

    assert_eq!(projection(&pool).len(), 1);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 18, false, false, 3)]
    );
}

// ---------------------------------------------------------------------------
// Row watermarks never exceed the frontier
// ---------------------------------------------------------------------------

#[test]
fn a_day_row_watermarked_above_the_frontier_fails_the_completion_closed() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    apply_everything(&pool);
    let before = month_state(&pool);

    // Out-of-band damage: a day row claims a position that was never
    // applied. Deriving a monthly watermark from it would put derived
    // evidence above the frontier, so the completion refuses.
    exec(&pool, "UPDATE metric_rollup_work_day SET watermark = 999");
    day_delta(&pool, &fixture, DAY_TWO, AGG, 1);
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let error = complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
        .expect_err("a day row above the frontier must block the completion");
    assert!(
        matches!(&error, ThothError::InternalError(message)
            if message.contains("watermarked above the frontier")),
        "unexpected failure: {error:?}"
    );
    assert_eq!(state(&pool).applied_through_sequence, 1);
    assert_eq!(deltas(&pool)[1].status, "CLAIMED");
    assert_eq!(projection(&pool).len(), 1, "the day update rolled back");
    assert_eq!(
        month_state(&pool),
        before,
        "and so did the monthly recomputation"
    );

    // Repairing the damage lets the same token complete.
    exec(&pool, "UPDATE metric_rollup_work_day SET watermark = 1");
    complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token).expect("completion");
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 11, false, false, 2)]
    );
}

// ---------------------------------------------------------------------------
// Fixed-statement set-wise maintenance
// ---------------------------------------------------------------------------

#[test]
fn the_monthly_maintenance_issues_the_same_nine_statements_for_1_10_and_50_keys() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let (logged, log) = logging_pool();
    assert_eq!(MONTH_MAINTENANCE_STATEMENT_COUNT, 9);
    assert_eq!(MONTH_MAINTENANCE_STATEMENTS.len(), 9);

    let mut next_month = 0;
    for keys in [1_u32, 10, 50] {
        for _ in 0..keys {
            day_delta(
                &pool,
                &fixture,
                ymd(distinct_month(next_month)),
                AGG,
                i64::from(next_month) + 1,
            );
            next_month += 1;
        }
        log.lock().expect("statement log").clear();
        let claims = claim_metric_rollup_deltas(&logged, CLAIMANT, 50).expect("claim");
        assert_eq!(claims.len(), keys as usize);
        complete_metric_rollup_deltas(&logged, CLAIMANT, claims[0].claim_token)
            .expect("completion");

        // Exactly the nine fixed statements, once each, in order, whatever
        // the number of distinct affected month keys; the day layer's
        // per-delta pair grows with the batch as approved.
        assert_eq!(
            captured_month_statements(&log),
            (0..MONTH_MAINTENANCE_STATEMENT_COUNT).collect::<Vec<_>>(),
            "{keys} affected keys must issue exactly the nine monthly statements"
        );
        assert_eq!(captured_day_statements(&log), 2 * keys as usize);
    }
    assert_eq!(month_rows(&pool).len(), 61);
    assert_eq!(state(&pool).applied_through_sequence, 61);
}

#[test]
fn fifty_deltas_into_one_month_key_recompute_it_once() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let (logged, log) = logging_pool();
    for index in 0..50 {
        day_delta(
            &pool,
            &fixture,
            (2026, 3, index % 25 + 1),
            if index % 2 == 0 { AGG } else { GB },
            i64::from(index) + 1,
        );
    }

    let claims = claim_metric_rollup_deltas(&logged, CLAIMANT, 50).expect("claim");
    assert_eq!(claims.len(), 50);
    complete_metric_rollup_deltas(&logged, CLAIMANT, claims[0].claim_token).expect("completion");

    assert_eq!(
        captured_month_statements(&log),
        (0..MONTH_MAINTENANCE_STATEMENT_COUNT).collect::<Vec<_>>()
    );
    // Every day holds an aggregate and a country row, so the aggregate is
    // authoritative on each: the month is the sum of the odd positions'
    // values, 1 + 3 + ... + 49 = 625, and the country rows sum the rest.
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, 625, false, false, 49)]
    );
    assert_eq!(
        countries(&pool),
        vec![(month(2026, 3), None, country("GB"), 650, false, 50)]
    );
    assert_eq!(state(&pool).applied_through_sequence, 50);
}

// ---------------------------------------------------------------------------
// Atomicity, replay and overflow
// ---------------------------------------------------------------------------

#[test]
fn a_monthly_recomputation_failure_rolls_back_day_month_delta_and_frontier_state() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    // One batch that feeds all four monthly tables: a country day, an
    // institution day, and a day whose two masks are total-ambiguous.
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    day_delta(&pool, &fixture, DAY_TWO, INST, 4);
    day_delta(&pool, &fixture, DAY_THREE, GB, 1);
    day_delta(&pool, &fixture, DAY_THREE, INST, 1);
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    let token = claims[0].claim_token;

    for table in MONTH_TABLES {
        {
            let _injected = FailingTrigger::install(&pool, "BEFORE INSERT", table, None);
            complete_metric_rollup_deltas(&pool, CLAIMANT, token)
                .expect_err("a failing monthly write must fail the completion");
        }
        assert!(
            projection(&pool).is_empty(),
            "{table}: the day updates must roll back with the monthly failure"
        );
        assert_eq!(month_state(&pool), MonthState::default(), "{table}");
        assert_eq!(state(&pool).applied_through_sequence, 0, "{table}");
        assert!(
            deltas(&pool)
                .iter()
                .all(|row| row.status == "CLAIMED" && row.applied_at.is_none()),
            "{table}: every delta stays claimed and retryable"
        );
    }

    // With the injections removed, the same token applies the batch.
    complete_metric_rollup_deltas(&pool, CLAIMANT, token).expect("the retry succeeds");
    assert_eq!(projection(&pool).len(), 4);
    let settled = month_state(&pool);
    assert_eq!(settled.totals.len(), 1);
    assert_eq!(settled.countries.len(), 1);
    assert_eq!(settled.institutions.len(), 1);
    assert_eq!(settled.ambiguity.len(), 1);
    assert_eq!(state(&pool).applied_through_sequence, 4);

    // A later batch failing in the keyed delete leaves the earlier monthly
    // state, the day rows and the frontier exactly as they were.
    let day_rows = projection(&pool);
    day_delta(&pool, &fixture, DAY_ONE, AGG, 20);
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    {
        let _injected =
            FailingTrigger::install(&pool, "BEFORE DELETE", "metric_rollup_work_month", None);
        complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
            .expect_err("a failing monthly delete must fail the completion");
    }
    assert_eq!(projection(&pool), day_rows);
    assert_eq!(month_state(&pool), settled);
    assert_eq!(state(&pool).applied_through_sequence, 4);
    assert_eq!(deltas(&pool)[4].status, "CLAIMED");
}

#[test]
fn a_timeout_after_commit_replay_rewrites_no_monthly_row() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_ONE, GB, 6);
    day_delta(&pool, &fixture, DAY_TWO, GB_INST, 3);
    day_delta(&pool, &fixture, DAY_TWO, PUB_GB, 2);
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
    let token = claims[0].claim_token;
    let first = complete_metric_rollup_deltas(&pool, CLAIMANT, token).expect("completion");

    // The surrogate ids are part of the snapshot: a replay that deleted and
    // reinserted the same logical rows would change them.
    let days = projection(&pool);
    let months = month_rows(&pool);
    let countries_before = country_rows(&pool);
    let institutions_before = institution_rows(&pool);
    let ambiguity_before = ambiguity_rows(&pool);
    assert_eq!(months.len(), 1);
    assert_eq!(ambiguity_before.len(), 1);

    for _ in 0..2 {
        let replay = complete_metric_rollup_deltas(&pool, CLAIMANT, token).expect("replay");
        assert_eq!(replay, first);
    }
    assert_eq!(projection(&pool), days);
    assert_eq!(month_rows(&pool), months);
    assert_eq!(country_rows(&pool), countries_before);
    assert_eq!(institution_rows(&pool), institutions_before);
    assert_eq!(ambiguity_rows(&pool), ambiguity_before);
    assert_eq!(state(&pool).applied_through_sequence, 4);
}

#[test]
fn a_monthly_sum_that_overflows_a_bigint_fails_closed_and_changes_nothing() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, i64::MAX);
    apply_everything(&pool);
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, i64::MAX, false, false, 1)]
    );

    // A second day of the same month is fine for the day layer, whose
    // checked arithmetic only sees its own row, and overflows only when the
    // month is summed. That overflow surfaces from PostgreSQL's own cast
    // and aborts the whole batch.
    day_delta(&pool, &fixture, DAY_TWO, AGG, 1);
    let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 1).expect("claim");
    let error = complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
        .expect_err("an overflowing monthly sum must fail closed");
    assert!(
        rejection(&error)
            .starts_with("Recomputing the monthly projections for this batch would overflow"),
        "unexpected rejection: {error:?}"
    );
    assert_eq!(projection(&pool).len(), 1, "the day row is not applied");
    assert_eq!(
        totals(&pool),
        vec![(month(2026, 3), None, i64::MAX, false, false, 1)]
    );
    assert_eq!(state(&pool).applied_through_sequence, 1);
    assert_eq!(deltas(&pool)[1].status, "CLAIMED");
}

// ---------------------------------------------------------------------------
// Fixed-seed differential and incremental/rebuild equality
// ---------------------------------------------------------------------------

#[test]
fn a_fixed_seed_differential_matches_the_per_day_oracle_and_a_fresh_rebuild() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let other_work = insert_extra_work(&pool, &fixture);
    let works = [
        (
            fixture.work_id,
            [
                fixture.publication_id,
                insert_extra_publication(&pool, fixture.work_id, "Paperback"),
            ],
        ),
        (
            other_work,
            [
                insert_extra_publication(&pool, other_work, "PDF"),
                insert_extra_publication(&pool, other_work, "Paperback"),
            ],
        ),
    ];
    let countries = ["GB", "US", "DE"];
    let institutions = [fixture.institution_id, insert_extra_institution(&pool)];

    // Bounded generated data: two works, three months, fourteen days each,
    // zero to three rows per base cell over every one of the eight masks,
    // then a revision (possibly to zero) on about a quarter of the records.
    let mut generator = Generator(DIFFERENTIAL_SEED);
    let mut records: Vec<(Uuid, i64)> = Vec::new();
    for (work_id, publications) in works {
        for month_index in 1..=3 {
            for day_of_month in 1..=14 {
                let on = NaiveDate::from_ymd_opt(2026, month_index, day_of_month).expect("a day");
                let rows = match generator.below(20) {
                    0..=4 => 0,
                    5..=12 => 1,
                    13..=17 => 2,
                    _ => 3,
                };
                for _ in 0..rows {
                    let mask = generator.below(8) as u8;
                    let dimensions = CellDimensions {
                        publication_id: (mask & 4 != 0)
                            .then(|| publications[generator.below(2) as usize]),
                        country_code: (mask & 2 != 0)
                            .then(|| countries[generator.below(3) as usize]),
                        institution_id: (mask & 1 != 0)
                            .then(|| institutions[generator.below(2) as usize]),
                    };
                    let value = 1 + generator.below(100) as i64;
                    let record_id =
                        commit_cell_delta(&pool, &fixture, work_id, on, dimensions, value);
                    records.push((record_id, value));
                }
            }
        }
    }
    let mut revisions = 0;
    for (record_id, value) in records.clone() {
        if generator.below(4) == 0 {
            let new_value = generator.below(121) as i64;
            commit_revision_delta(&pool, &fixture, record_id, 2, new_value, value);
            revisions += 1;
        }
    }
    let delta_count = deltas(&pool).len();
    let cardinality = format!(
        "seed {DIFFERENTIAL_SEED}: {} records, {revisions} revisions, {delta_count} deltas",
        records.len()
    );
    // Pinned so a change to the generator or the seed is visible.
    assert_eq!(
        (records.len(), revisions, delta_count),
        (120, 33, 153),
        "{cardinality}"
    );

    let frontier = apply_everything(&pool);
    assert_eq!(frontier, delta_count as i64);
    let days = projection(&pool);
    let expected = oracle_month_state(&days);
    let incremental = month_state(&pool);
    assert_eq!(
        incremental, expected,
        "{cardinality}: incremental state vs oracle"
    );
    assert_watermarks_within(&incremental, frontier);
    assert!(
        !incremental.ambiguity.is_empty() && !incremental.countries.is_empty(),
        "{cardinality}: the fixture must exercise ambiguity and country rows"
    );

    // A fresh rebuild from the work-day projection alone reproduces the
    // incrementally maintained state exactly, watermarks included, and
    // returns the frontier it corresponds to.
    let rebuilt_frontier = rebuild_month_projections(&pool).expect("rebuild");
    assert_eq!(rebuilt_frontier, frontier);
    let rebuilt = month_state(&pool);
    assert_eq!(
        rebuilt, incremental,
        "{cardinality}: rebuild vs incremental"
    );
    assert_eq!(rebuilt, expected, "{cardinality}: rebuild vs oracle");
    assert_eq!(
        projection(&pool),
        days,
        "the rebuild reads and never writes day rows"
    );
    assert_eq!(state(&pool).applied_through_sequence, frontier);
}

#[test]
fn a_rebuild_refuses_a_day_row_watermarked_above_the_durable_frontier() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    apply_everything(&pool);
    let before = month_state(&pool);

    exec(&pool, "UPDATE metric_rollup_work_day SET watermark = 5");
    let error = rebuild_month_projections(&pool).expect_err("the rebuild must refuse");
    assert!(
        matches!(&error, ThothError::InternalError(message)
            if message.contains("watermarked above the durable frontier")),
        "unexpected failure: {error:?}"
    );
    assert_eq!(month_state(&pool), before, "a refused rebuild is a no-op");
}

// ---------------------------------------------------------------------------
// Migration: schema, constraints, populated forward, revert and reapply
// ---------------------------------------------------------------------------

/// Revert migrations until the `MET-WP4-03A` monthly migration itself has
/// been reverted.
fn revert_through_month_migration(connection: &mut PgConnection) -> Result<(), String> {
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .map_err(|error| error.to_string())?;
        if reverted.to_string() == MET_WP4_03A_MIGRATION_VERSION {
            return Ok(());
        }
    }
}

fn table_exists(pool: &PgPool, table: &str) -> bool {
    scalar_i64(
        pool,
        &format!(
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relkind = 'r' AND relname = '{table}')"
        ),
    ) == 1
}

/// Revert the monthly migration, run `work` against the pre-03A schema,
/// then reapply it and return the reapplication result.
fn with_pre_03a_schema<F>(pool: &PgPool, work: F) -> Result<(), String>
where
    F: FnOnce(&PgPool),
{
    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database");
    revert_through_month_migration(&mut connection)
        .expect("the monthly migration must revert on derived-only state");
    for table in MONTH_TABLES {
        assert!(
            !table_exists(pool, table),
            "{table} must be dropped by the revert"
        );
    }
    work(pool);
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn columns(pool: &PgPool, table: &str) -> Vec<(String, String, String)> {
    #[derive(diesel::QueryableByName)]
    struct Column {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT column_name::text, data_type::text, is_nullable::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = $1 \
         ORDER BY ordinal_position",
    )
    .bind::<diesel::sql_types::Text, _>(table)
    .load::<Column>(&mut connection)
    .expect("Failed to read the columns")
    .into_iter()
    .map(|column| (column.column_name, column.data_type, column.is_nullable))
    .collect()
}

fn owned(shape: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    shape
        .iter()
        .map(|(name, data_type, nullable)| {
            (
                name.to_string(),
                data_type.to_string(),
                nullable.to_string(),
            )
        })
        .collect()
}

#[test]
fn the_month_migration_creates_exactly_the_four_empty_tables_with_the_approved_columns() {
    let (_guard, pool) = setup_registry_db();
    for table in MONTH_TABLES {
        assert!(table_exists(&pool, table));
        assert_eq!(
            scalar_i64(&pool, &format!("(SELECT COUNT(*) FROM {table})")),
            0,
            "the migration must populate nothing in {table}"
        );
    }
    assert_eq!(
        columns(&pool, "metric_rollup_work_month"),
        owned(&[
            ("rollup_work_month_id", "uuid", "NO"),
            ("work_id", "uuid", "NO"),
            ("publication_id", "uuid", "YES"),
            ("platform_id", "uuid", "NO"),
            ("measure_id", "uuid", "NO"),
            ("month_start", "date", "NO"),
            ("value", "bigint", "NO"),
            ("requires_country_coverage", "boolean", "NO"),
            ("requires_institution_coverage", "boolean", "NO"),
            ("watermark", "bigint", "NO"),
        ])
    );
    assert_eq!(
        columns(&pool, "metric_rollup_work_country_month"),
        owned(&[
            ("rollup_work_country_month_id", "uuid", "NO"),
            ("work_id", "uuid", "NO"),
            ("publication_id", "uuid", "YES"),
            ("platform_id", "uuid", "NO"),
            ("measure_id", "uuid", "NO"),
            ("month_start", "date", "NO"),
            ("country_code", "character", "NO"),
            ("value", "bigint", "NO"),
            ("requires_institution_coverage", "boolean", "NO"),
            ("watermark", "bigint", "NO"),
        ])
    );
    assert_eq!(
        columns(&pool, "metric_rollup_work_institution_month"),
        owned(&[
            ("rollup_work_institution_month_id", "uuid", "NO"),
            ("work_id", "uuid", "NO"),
            ("publication_id", "uuid", "YES"),
            ("platform_id", "uuid", "NO"),
            ("measure_id", "uuid", "NO"),
            ("month_start", "date", "NO"),
            ("institution_id", "uuid", "NO"),
            ("value", "bigint", "NO"),
            ("requires_country_coverage", "boolean", "NO"),
            ("watermark", "bigint", "NO"),
        ])
    );
    assert_eq!(
        columns(&pool, "metric_rollup_work_month_ambiguity"),
        owned(&[
            ("rollup_work_month_ambiguity_id", "uuid", "NO"),
            ("work_id", "uuid", "NO"),
            ("platform_id", "uuid", "NO"),
            ("measure_id", "uuid", "NO"),
            ("month_start", "date", "NO"),
            ("total_ambiguous", "boolean", "NO"),
            ("country_ambiguous", "boolean", "NO"),
            ("institution_ambiguous", "boolean", "NO"),
            ("watermark", "bigint", "NO"),
        ])
    );
}

#[test]
fn the_month_tables_carry_exactly_the_approved_constraints_and_no_secondary_index() {
    let (_guard, pool) = setup_registry_db();

    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_work_month"),
        vec![
            "metric_rollup_work_month_month_start_check".to_string(),
            "metric_rollup_work_month_watermark_check".to_string(),
        ]
    );
    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_work_country_month"),
        vec![
            "metric_rollup_work_country_month_country_code_check".to_string(),
            "metric_rollup_work_country_month_month_start_check".to_string(),
            "metric_rollup_work_country_month_watermark_check".to_string(),
        ]
    );
    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_work_institution_month"),
        vec![
            "metric_rollup_work_institution_month_month_start_check".to_string(),
            "metric_rollup_work_institution_month_watermark_check".to_string(),
        ]
    );
    assert_eq!(
        check_constraint_names(&pool, "metric_rollup_work_month_ambiguity"),
        vec![
            "metric_rollup_work_month_ambiguity_flags_check".to_string(),
            "metric_rollup_work_month_ambiguity_month_start_check".to_string(),
            "metric_rollup_work_month_ambiguity_watermark_check".to_string(),
        ]
    );

    // Non-cascading foreign keys to exactly the represented entities.
    for (table, parents) in [
        (
            "metric_rollup_work_month",
            vec!["metric_measure", "metric_platform", "publication", "work"],
        ),
        (
            "metric_rollup_work_country_month",
            vec!["metric_measure", "metric_platform", "publication", "work"],
        ),
        (
            "metric_rollup_work_institution_month",
            vec![
                "institution",
                "metric_measure",
                "metric_platform",
                "publication",
                "work",
            ],
        ),
        (
            "metric_rollup_work_month_ambiguity",
            vec!["metric_measure", "metric_platform", "work"],
        ),
    ] {
        let keys = foreign_keys(&pool, table);
        let mut referenced: Vec<String> = keys
            .iter()
            .map(|(_, definition)| {
                definition
                    .split("REFERENCES ")
                    .nth(1)
                    .and_then(|rest| rest.split('(').next())
                    .expect("a referenced table")
                    .to_string()
            })
            .collect();
        referenced.sort();
        assert_eq!(referenced, parents, "{table} foreign keys: {keys:?}");
        for (name, definition) in &keys {
            assert!(
                !definition.contains("ON DELETE"),
                "{name} must stay non-cascading: {definition}"
            );
        }
    }

    // Exactly the primary-key index and the logical identity index; no
    // secondary performance index, per the accepted benchmark.
    for (table, identity) in [
        (
            "metric_rollup_work_month",
            "metric_rollup_work_month_identity_key",
        ),
        (
            "metric_rollup_work_country_month",
            "metric_rollup_work_country_month_identity_key",
        ),
        (
            "metric_rollup_work_institution_month",
            "metric_rollup_work_institution_month_identity_key",
        ),
        (
            "metric_rollup_work_month_ambiguity",
            "metric_rollup_work_month_ambiguity_identity_key",
        ),
    ] {
        assert_eq!(
            index_names(&pool, table),
            vec![identity.to_string(), format!("{table}_pkey")],
            "{table} must carry exactly its primary-key and identity indexes"
        );
        let definition = index_definition(&pool, table, identity);
        assert!(definition.contains("UNIQUE"), "{identity}: {definition}");
        if table != "metric_rollup_work_month_ambiguity" {
            assert!(
                definition.contains("NULLS NOT DISTINCT"),
                "{identity} must treat an absent publication as a value: {definition}"
            );
        }
    }
}

#[test]
fn the_month_identities_and_constraints_reject_malformed_and_duplicate_rows() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let (work, platform, measure, institution) = (
        fixture.work_id,
        fixture.platform_id,
        fixture.measure_id,
        fixture.institution_id,
    );

    let attempt = |statement: String| -> Result<usize, DieselError> {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(statement).execute(&mut connection)
    };
    let total = |month_start: &str, value: i64, watermark: i64| {
        format!(
            "INSERT INTO metric_rollup_work_month (work_id, publication_id, platform_id, \
                 measure_id, month_start, value, requires_country_coverage, \
                 requires_institution_coverage, watermark) \
             VALUES ('{work}', NULL, '{platform}', '{measure}', '{month_start}', {value}, \
                     false, false, {watermark})"
        )
    };
    let country_row = |code: &str| {
        format!(
            "INSERT INTO metric_rollup_work_country_month (work_id, publication_id, \
                 platform_id, measure_id, month_start, country_code, value, \
                 requires_institution_coverage, watermark) \
             VALUES ('{work}', NULL, '{platform}', '{measure}', '2026-03-01', '{code}', 1, \
                     false, 1)"
        )
    };
    let institution_row = format!(
        "INSERT INTO metric_rollup_work_institution_month (work_id, publication_id, \
             platform_id, measure_id, month_start, institution_id, value, \
             requires_country_coverage, watermark) \
         VALUES ('{work}', NULL, '{platform}', '{measure}', '2026-03-01', '{institution}', 1, \
                 false, 1)"
    );
    let ambiguity_row = |flags: &str| {
        format!(
            "INSERT INTO metric_rollup_work_month_ambiguity (work_id, platform_id, measure_id, \
                 month_start, total_ambiguous, country_ambiguous, institution_ambiguous, \
                 watermark) \
             VALUES ('{work}', '{platform}', '{measure}', '2026-03-01', {flags}, 1)"
        )
    };

    attempt(total("2026-03-01", 1, 1)).expect("a well-formed total row");
    attempt(country_row("GB")).expect("a well-formed country row");
    attempt(institution_row.clone()).expect("a well-formed institution row");
    attempt(ambiguity_row("true, false, false")).expect("a well-formed ambiguity row");

    // Logical identities, with an absent publication treated as a value.
    for (label, statement) in [
        (
            "a duplicate NULL-publication total",
            total("2026-03-01", 2, 2),
        ),
        ("a duplicate country row", country_row("GB")),
        ("a duplicate institution row", institution_row.clone()),
        (
            "a duplicate ambiguity row",
            ambiguity_row("false, true, false"),
        ),
    ] {
        let result = attempt(statement);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _
                ))
            ),
            "{label} must be rejected, got {result:?}"
        );
    }
    // CHECK constraints.
    for (label, statement) in [
        (
            "a month_start that is not the first of its month",
            total("2026-04-02", 1, 1),
        ),
        ("a zero watermark", total("2026-04-01", 1, 0)),
        ("a negative watermark", total("2026-05-01", 1, -1)),
        ("a lowercase country code", country_row("gb")),
        ("a one-letter country code", country_row("G")),
        ("a digit in a country code", country_row("G1")),
        ("an ambiguity row with no true flag", {
            let mut statement = ambiguity_row("false, false, false");
            statement = statement.replace("'2026-03-01'", "'2026-04-01'");
            statement
        }),
    ] {
        let result = attempt(statement);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "{label} must be rejected, got {result:?}"
        );
    }
    // A three-letter code never reaches the CHECK: the `character(2)` column
    // itself refuses it.
    let result = attempt(country_row("GBR"));
    assert!(
        matches!(&result, Err(DieselError::DatabaseError(_, info))
            if info.message().contains("too long for type character(2)")),
        "a three-letter country code must be rejected, got {result:?}"
    );
    // A signed value is accepted; a NULL publication and a specific one are
    // distinct identities.
    attempt(total("2026-06-01", -5, 3)).expect("a negative monthly value");
    attempt(format!(
        "INSERT INTO metric_rollup_work_month (work_id, publication_id, platform_id, \
             measure_id, month_start, value, requires_country_coverage, \
             requires_institution_coverage, watermark) \
         VALUES ('{work}', '{}', '{platform}', '{measure}', '2026-03-01', 1, false, false, 1)",
        fixture.publication_id
    ))
    .expect("a publication-specific row beside the NULL-publication row");

    // Foreign keys: unknown parents are rejected, and a parent still
    // referenced cannot be deleted.
    let result = attempt(format!(
        "INSERT INTO metric_rollup_work_month (work_id, publication_id, platform_id, \
             measure_id, month_start, value, requires_country_coverage, \
             requires_institution_coverage, watermark) \
         VALUES ('{}', NULL, '{platform}', '{measure}', '2026-07-01', 1, false, false, 1)",
        Uuid::new_v4()
    ));
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an unknown work must be rejected, got {result:?}"
    );
    let result = delete_row(&pool, "institution", "institution_id", institution);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced institution must be restricted, got {result:?}"
    );
}

#[test]
fn the_month_migration_applies_over_populated_pre_03a_state_and_a_rebuild_populates_it() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");

    // Applied work-day history as it would exist before MET-WP4-03A: day
    // rows and an advanced frontier, written directly as derived state.
    let seeded = |pool: &PgPool| {
        let (work, publication, platform, measure, institution) = (
            fixture.work_id,
            fixture.publication_id,
            fixture.platform_id,
            fixture.measure_id,
            fixture.institution_id,
        );
        for (day_of_month, publication_id, country_code, institution_id, value, watermark) in [
            (
                1,
                "NULL".to_string(),
                "NULL".to_string(),
                "NULL".to_string(),
                10,
                1,
            ),
            (
                1,
                "NULL".to_string(),
                "'GB'".to_string(),
                "NULL".to_string(),
                6,
                2,
            ),
            (
                2,
                format!("'{publication}'"),
                "'US'".to_string(),
                "NULL".to_string(),
                4,
                3,
            ),
            (
                3,
                "NULL".to_string(),
                "'GB'".to_string(),
                format!("'{institution}'"),
                2,
                4,
            ),
            (
                3,
                "NULL".to_string(),
                "NULL".to_string(),
                format!("'{institution}'"),
                1,
                5,
            ),
        ] {
            exec(
                pool,
                &format!(
                    "INSERT INTO metric_rollup_work_day (work_id, publication_id, platform_id, \
                         measure_id, day, country_code, institution_id, value, watermark) \
                     VALUES ('{work}', {publication_id}, '{platform}', '{measure}', \
                             '2026-03-0{day_of_month}', {country_code}, {institution_id}, \
                             {value}, {watermark})"
                ),
            );
        }
        exec(
            pool,
            "UPDATE metric_rollup_work_day_state \
             SET next_sequence = 6, applied_through_sequence = 5",
        );
    };
    with_pre_03a_schema(&pool, seeded).expect("the migration must apply over populated state");

    // The forward migration touched no day row, no state and created empty
    // monthly tables.
    let days = projection(&pool);
    assert_eq!(days.len(), 5);
    assert_eq!(state(&pool).applied_through_sequence, 5);
    assert_eq!(state(&pool).next_sequence, 6);
    assert_eq!(month_state(&pool), MonthState::default());

    // The separately authorized historical rebuild is what populates them,
    // from the work-day projection alone, to exactly the oracle's state.
    let frontier = rebuild_month_projections(&pool).expect("rebuild");
    assert_eq!(frontier, 5);
    let rebuilt = month_state(&pool);
    assert_eq!(rebuilt, oracle_month_state(&days));
    // Day one: the aggregate is the total. Day two: one {P,C} mask, so a
    // publication-specific total that depends on country coverage. Day
    // three: {C,I} beside {I} is total-ambiguous.
    assert_eq!(
        totals(&pool),
        vec![
            (month(2026, 3), None, 10, false, false, 1),
            (
                month(2026, 3),
                Some(fixture.publication_id),
                4,
                true,
                false,
                3
            ),
        ]
    );
    // Country: day one's {C} row and day three's {C,I} row (its least
    // country mask) are additive into one GB row that depends on
    // institution coverage; day two's {P,C} row keeps its publication.
    assert_eq!(
        countries(&pool),
        vec![
            (month(2026, 3), None, country("GB"), 8, true, 4),
            (
                month(2026, 3),
                Some(fixture.publication_id),
                country("US"),
                4,
                false,
                3
            ),
        ]
    );
    assert_eq!(
        institutions(&pool),
        vec![(month(2026, 3), None, fixture.institution_id, 1, false, 5)],
        "day three's {{I}} row is the least institution mask"
    );
    assert_eq!(
        ambiguities(&pool),
        vec![(month(2026, 3), true, false, false, 5)]
    );
    assert_eq!(projection(&pool), days);
}

#[test]
fn the_month_migration_reverts_only_the_derived_tables_and_reapplies_empty() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    day_delta(&pool, &fixture, DAY_ONE, AGG, 10);
    day_delta(&pool, &fixture, DAY_TWO, GB, 6);
    day_delta(&pool, &fixture, DAY_TWO, INST, 4);
    apply_everything(&pool);
    let days = projection(&pool);
    let rollup_deltas = deltas(&pool);
    let frontier = state(&pool);
    let incremental = month_state(&pool);
    assert!(!incremental.totals.is_empty() && !incremental.ambiguity.is_empty());

    // apply -> revert -> apply, then once more, with derived monthly rows
    // present: the downgrade drops only the four derived tables, leaves the
    // work-day projection, the deltas and the frontier untouched, and the
    // reapplication recreates the tables empty.
    for _ in 0..2 {
        with_pre_03a_schema(&pool, |pool| {
            assert_eq!(projection(pool), days);
            assert_eq!(deltas(pool), rollup_deltas);
            assert_eq!(state(pool), frontier);
            assert!(table_exists(pool, "metric_rollup_work_day"));
            assert!(table_exists(pool, "metric_rollup_work_day_state"));
            assert_eq!(
                scalar_i64(
                    pool,
                    "(SELECT COUNT(*) FROM pg_trigger \
                      WHERE tgrelid = 'public.metric_rollup_delta'::regclass \
                        AND NOT tgisinternal)"
                ),
                1,
                "the MET-WP4-01 allocation trigger survives the revert"
            );
        })
        .expect("the monthly migration must reapply");
        for table in MONTH_TABLES {
            assert!(table_exists(&pool, table));
        }
        assert_eq!(month_state(&pool), MonthState::default());
        assert_eq!(projection(&pool), days);
        assert_eq!(state(&pool), frontier);
    }

    // And a rebuild restores exactly the state incremental maintenance had
    // produced before the round trip.
    rebuild_month_projections(&pool).expect("rebuild");
    assert_eq!(month_state(&pool), incremental);
}

// ---------------------------------------------------------------------------
// Performance, query-plan and locking evidence (run explicitly)
// ---------------------------------------------------------------------------

/// The recorded duration percentiles of one sample set, in microseconds.
fn percentiles(samples: &mut [Duration]) -> (u128, u128, u128) {
    samples.sort();
    let at = |fraction: f64| {
        let index = ((samples.len() - 1) as f64 * fraction).round() as usize;
        samples[index].as_micros()
    };
    (at(0.5), at(0.95), samples[samples.len() - 1].as_micros())
}

/// Production-shaped synthetic work-day rows for evidence: `works` works
/// under the fixture imprint, one year of days each, and a random mix of
/// masks per day. Values, country choices and mask draws come from a seeded
/// `random()`; institution choices hash generated UUIDs, so the institution
/// row count varies slightly between runs while the shape does not.
fn seed_production_shaped_days(pool: &PgPool, fixture: &RecordFixture, works: i64) -> i64 {
    exec(pool, "SELECT setseed(0.20260923)");
    exec(
        pool,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT gen_random_uuid(), 'monograph', 'forthcoming', imprint_id, 1 \
             FROM work, generate_series(1, {works}) \
             WHERE work_id = '{}'",
            fixture.work_id
        ),
    );
    exec(
        pool,
        "INSERT INTO institution (institution_id, institution_name) \
         SELECT gen_random_uuid(), 'Institution ' || g FROM generate_series(1, 20) g",
    );
    // Every work-day gets an aggregate row; 40% also get a country row and
    // 15% an institution row, drawn from 12 countries and 20 institutions,
    // with a monotone watermark so the frontier can be set above them.
    exec(
        pool,
        &format!(
            "WITH days AS (SELECT generate_series(DATE '2025-01-01', DATE '2025-12-31', \
                                                  interval '1 day')::date AS day), \
                  cells AS (SELECT w.work_id, d.day, random() AS r1, random() AS r2, \
                                   random() AS r3, random() AS r4 \
                            FROM work w CROSS JOIN days d \
                            WHERE w.work_id <> '{}'), \
                  rows AS ( \
                      SELECT work_id, day, NULL::char(2) AS country_code, \
                             NULL::uuid AS institution_id, (1 + r3 * 50)::bigint AS value \
                      FROM cells \
                      UNION ALL \
                      SELECT work_id, day, \
                             (ARRAY['GB','US','DE','FR','ES','IT','NL','SE','CA','AU','JP','BR']) \
                                 [1 + (r2 * 11)::int], \
                             NULL, (1 + r4 * 20)::bigint \
                      FROM cells WHERE r1 < 0.40 \
                      UNION ALL \
                      SELECT work_id, day, NULL, \
                             (SELECT institution_id FROM institution \
                              ORDER BY md5(institution_id::text || c.day::text || c.work_id::text) \
                              LIMIT 1), \
                             (1 + r4 * 10)::bigint \
                      FROM cells c WHERE r1 >= 0.85) \
             INSERT INTO metric_rollup_work_day (work_id, publication_id, platform_id, \
                 measure_id, day, country_code, institution_id, value, watermark) \
             SELECT work_id, NULL, '{}', '{}', day, country_code, institution_id, value, \
                    row_number() OVER (ORDER BY day, work_id) \
             FROM rows",
            fixture.work_id, fixture.platform_id, fixture.measure_id
        ),
    );
    let rows = scalar_i64(pool, "(SELECT COUNT(*) FROM metric_rollup_work_day)");
    exec(
        pool,
        &format!(
            "UPDATE metric_rollup_work_day_state \
             SET next_sequence = {}, applied_through_sequence = {rows}",
            rows + 1
        ),
    );
    exec(pool, "ANALYZE metric_rollup_work_day");
    rows
}

/// Bounded local evidence for the implementation report: the monthly
/// maintenance's cost for 1, 10 and 50 affected keys over a
/// production-shaped work-day table, the full rebuild, the query plans of
/// the nine statements with fifty keys, and the relation locks the
/// completion critical section holds. Run explicitly with `--ignored
/// --nocapture`; the numbers are host-dependent and are reported, not
/// asserted.
#[test]
#[ignore = "bounded local performance evidence; run explicitly"]
fn evidence_month_maintenance_cost_plans_and_locks() {
    let (_guard, pool) = setup_registry_db();
    let (fixture, _record_id) = fixture_record(&pool, "identity-base");
    let works: i64 = std::env::var("THOTH_EVIDENCE_WORKS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2_000);
    let started = Instant::now();
    let rows = seed_production_shaped_days(&pool, &fixture, works);
    println!(
        "seeded {rows} work-day rows over {works} works x 365 days in {:?}",
        started.elapsed()
    );

    // Full rebuild, twice: fresh state, then over the truncated state.
    for round in 1..=2 {
        let started = Instant::now();
        let frontier = rebuild_month_projections(&pool).expect("rebuild");
        println!(
            "rebuild {round}: {:?} at frontier {frontier}; totals {} country {} institution {} ambiguity {}",
            started.elapsed(),
            scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_month)"),
            scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_country_month)"),
            scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_institution_month)"),
            scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_month_ambiguity)"),
        );
    }
    let frontier = state(&pool).applied_through_sequence;

    // Month keys drawn deterministically from the seeded works.
    #[derive(diesel::QueryableByName)]
    struct KeyRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        work_id: Uuid,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let work_ids: Vec<Uuid> =
        sql_query("SELECT work_id FROM work WHERE work_id <> $1 ORDER BY work_id LIMIT 50")
            .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
            .load::<KeyRow>(&mut connection)
            .expect("seeded works")
            .into_iter()
            .map(|row| row.work_id)
            .collect();
    let keys_of = |count: usize| -> BTreeSet<MonthKey> {
        work_ids[..count]
            .iter()
            .enumerate()
            .map(|(index, work_id)| {
                (
                    *work_id,
                    fixture.platform_id,
                    fixture.measure_id,
                    month(2025, (index % 12) as u32 + 1),
                )
            })
            .collect()
    };

    // The monthly portion alone, under the same state-row lock a completion
    // holds: warm-up, then repeated samples. Each sample deletes and
    // reinserts the keys' rows exactly as a completion does.
    for count in [1_usize, 10, 50] {
        let keys = keys_of(count);
        let mut samples = Vec::with_capacity(40);
        for sample in 0..45 {
            let started = Instant::now();
            connection
                .transaction::<_, ThothError, _>(|connection| {
                    sql_query(
                        "SELECT 1 FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE",
                    )
                    .execute(connection)?;
                    recompute_month_projections(connection, &keys, frontier)
                })
                .expect("recompute");
            if sample >= 5 {
                samples.push(started.elapsed());
            }
        }
        let (p50, p95, max) = percentiles(&mut samples);
        println!(
            "monthly maintenance, {count} affected keys: p50 {p50} us, p95 {p95} us, max {max} us over {} samples",
            samples.len()
        );
    }

    // Query plans of the nine statements with fifty keys.
    #[derive(diesel::QueryableByName)]
    struct PlanLine {
        #[diesel(sql_type = diesel::sql_types::Text, column_name = "QUERY PLAN")]
        line: String,
    }
    let keys = keys_of(50);
    let work_ids: Vec<Uuid> = keys.iter().map(|key| key.0).collect();
    let platform_ids: Vec<Uuid> = keys.iter().map(|key| key.1).collect();
    let measure_ids: Vec<Uuid> = keys.iter().map(|key| key.2).collect();
    let month_starts: Vec<NaiveDate> = keys.iter().map(|key| key.3).collect();
    connection
        .transaction::<(), ThothError, _>(|connection| {
            sql_query("SELECT 1 FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE")
                .execute(connection)?;
            for (index, statement) in MONTH_MAINTENANCE_STATEMENTS.iter().enumerate() {
                let plan: Vec<PlanLine> =
                    sql_query(format!("EXPLAIN (ANALYZE, BUFFERS, COSTS OFF) {statement}"))
                        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&work_ids)
                        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&platform_ids)
                        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&measure_ids)
                        .bind::<diesel::sql_types::Array<diesel::sql_types::Date>, _>(&month_starts)
                        .load(connection)?;
                println!("--- plan of monthly statement {} (50 keys):", index + 1);
                for line in plan {
                    println!("{}", line.line);
                }
            }
            // The relation locks this critical section holds after the
            // monthly maintenance ran.
            #[derive(diesel::QueryableByName)]
            struct LockRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                relation: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                mode: String,
            }
            let locks: Vec<LockRow> = sql_query(
                "SELECT relation::regclass::text AS relation, mode::text AS mode \
                 FROM pg_locks \
                 WHERE pid = pg_backend_pid() AND locktype = 'relation' \
                   AND relation::regclass::text NOT LIKE 'pg_%' \
                 ORDER BY 1, 2",
            )
            .load(connection)?;
            println!("--- relation locks held by the completion critical section:");
            for lock in locks {
                println!("{} {}", lock.relation, lock.mode);
            }
            Err(ThothError::InternalError("evidence only; roll back".into()))
        })
        .expect_err("evidence transaction rolls back");
    drop(connection);

    // The whole completion, end to end, for 1, 10 and 50 keys with one delta
    // per key, and for 50 deltas into one key, with real claims.
    for (label, keys, deltas_per_key) in [
        ("1 key, 1 delta", 1_u32, 1_u32),
        ("10 keys, 10 deltas", 10, 1),
        ("50 keys, 50 deltas", 50, 1),
        ("1 key, 50 deltas", 1, 50),
    ] {
        let mut samples = Vec::new();
        for sample in 0..12 {
            for key in 0..keys {
                for delta in 0..deltas_per_key {
                    day_delta(
                        &pool,
                        &fixture,
                        (2026, key % 12 + 1, delta % 28 + 1),
                        if delta % 3 == 0 { GB } else { AGG },
                        i64::from(delta) + 1,
                    );
                }
            }
            let claims = claim_metric_rollup_deltas(&pool, CLAIMANT, 50).expect("claim");
            let started = Instant::now();
            complete_metric_rollup_deltas(&pool, CLAIMANT, claims[0].claim_token)
                .expect("completion");
            if sample >= 2 {
                samples.push(started.elapsed());
            }
        }
        let (p50, p95, max) = percentiles(&mut samples);
        println!(
            "whole completion, {label}: p50 {p50} us, p95 {p95} us, max {max} us over {} samples",
            samples.len()
        );
    }
}
