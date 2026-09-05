//! Focused `MET-WP1-09` database tests for `metric_operas_export`: the
//! approved ten-field/default contract, database-generated and explicitly
//! supplied export identity, zero and positive attempt counts, unconstrained
//! nonblank `status` text, the four nullable delivery-result columns in both
//! their null and populated states, revision/mapping referential integrity,
//! one-export-row-per-canonical-revision uniqueness, restricted
//! (non-cascading) deletion, the exact column/check/foreign-key/index
//! inventory and the targeted revert/reapply of the OPERAS export migration.
//!
//! The canonical record/revision fixtures are the existing `pub(crate)`
//! helpers defined by `metric_record/tests.rs` and
//! `metric_record_revision/tests.rs`, consumed as-is. The `metric_measure`,
//! `metric_platform_measure` and `metric_operas_mapping` fixtures below are
//! local raw-SQL helpers rather than reuses of the private equivalents in
//! `metric_platform_measure/tests.rs` and `metric_operas_mapping/tests.rs`: no
//! other model's test module is widened, because this task's write budget
//! contains only this file.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no export row at runtime and implements no outbound eligibility,
//! source finalization, `METRICS_OPERAS_EXPORT` capability enforcement, status
//! vocabulary or transition graph, claiming, lease, retry scheduling, backoff,
//! stale-claim recovery, payload construction, hashing, remote delivery,
//! remote idempotency, inbound synchronization or reconciliation, and nothing
//! here pretends otherwise. In particular, an accepted `status`,
//! `remote_event_id` or `request_hash` string is evidence of nonblank-text
//! storage only, never of a recognised state, a valid remote event identifier
//! or a computed payload hash.
//!
//! Retry-time boundary (reviewed): the approved design's section 6.14 names
//! the ten fields asserted here and contains **no** retry-time column, while
//! its section 14.4 refers to OPERAS export indexes on status and retry time.
//! The independently approved decision is a deliberate deferral to WP9, which
//! owns the claim/retry representation and its query/index contract. These
//! tests therefore assert the *absence* of any retry-time, claim or lease
//! column and of any speculative status/retry index.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricOperasExport;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{insert_platform_row, scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{
    delete_row, foreign_keys, index_definition, index_names, RecordFixture,
};
use crate::model::metric_record_revision::tests::{
    fixture_record, insert_revision_row, insert_second_record,
};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_operas_export;

/// The Diesel migration version of `thoth-api/migrations/20260905_v1.9.0`.
const MET_WP1_09_MIGRATION_VERSION: &str = "20260905";

/// The remaining reconciliation ledgers named by the approved design. They stay
/// approved *future* architecture: `MET-WP1-09` creates the outbound export
/// ledger only.
///
/// `metric_operas_import` is deliberately absent from this list: the inbound
/// import ledger is now owned by MET-WP1-10 (issue #888), which creates it in
/// migration `20260906_v1.9.0`. MET-WP1-09 still does not create it — the
/// export migration adds only `metric_operas_export` — so this constant
/// narrows to the ledgers that are genuinely still deferred rather than
/// asserting that a later authorized slice never landed.
const DEFERRED_LEDGER_TABLES: [&str; 2] =
    ["metric_reconciliation_issue", "metric_reconciliation_run"];

/// Column names that would betray an invented retry-time representation or
/// claim/lease protocol having been smuggled into this persistence-only
/// slice. The approved design fixes none of them, and WP9 owns the eventual
/// claim/retry contract.
const DEFERRED_EXPORT_COLUMNS: [&str; 12] = [
    "backoff_seconds",
    "claim_token",
    "claim_until",
    "claimed_at",
    "claimed_by",
    "lease_expires_at",
    "lease_until",
    "next_attempt_at",
    "next_retry",
    "retry_after",
    "retry_at",
    "updated_at",
];

/// Revert migrations until the `MET-WP1-09` OPERAS export migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_operas_mapping_migration` and
/// its predecessors: a bare `revert_last_migration` would only mean "the
/// OPERAS export migration" while it happens to be the newest applied
/// migration. Reverting down to and including the target keeps the meaning
/// under any later migration order, and no future migration name is assumed or
/// hard-coded.
fn revert_through_operas_export_migration(connection: &mut PgConnection) {
    let operas_export_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_09_MIGRATION_VERSION);
    assert!(
        operas_export_migration_applied,
        "the MET-WP1-09 OPERAS export migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_09_MIGRATION_VERSION {
            return;
        }
    }
}

/// Insert one `metric_measure` row with an explicit id through raw SQL.
///
/// Local to this module: the equivalent helper in
/// `metric_platform_measure/tests.rs` is private, and this task's write budget
/// does not permit widening that file.
fn insert_measure_row(pool: &PgPool, measure_id: Uuid, code: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_measure \
             (measure_id, code, display_name, category, unit, allow_negative, \
              additive_across_time, additive_across_works, definition, enabled) \
         VALUES ($1, $2, $3, 'USAGE', 'COUNT', FALSE, TRUE, TRUE, 'Some definition.', TRUE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .bind::<diesel::sql_types::Text, _>(code)
    .bind::<diesel::sql_types::Text, _>(format!("Measure {code}"))
    .execute(&mut connection)
    .expect("Failed to insert metric_measure fixture row");
}

/// Register one `metric_platform_measure` pair through raw SQL and return its
/// primary key.
fn insert_platform_measure_row(pool: &PgPool, platform_id: Uuid, measure_id: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        platform_measure_id: Uuid,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_platform_measure \
             (platform_id, measure_id, supported_grains, supports_country, \
              supports_institution, supports_publication, direct_collection, enabled) \
         VALUES ($1, $2, ARRAY['MONTH']::public.metric_reporting_grain[], \
                 TRUE, FALSE, FALSE, FALSE, TRUE) \
         RETURNING platform_measure_id",
    )
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .get_result::<Row>(&mut connection)
    .expect("Failed to insert metric_platform_measure fixture row")
    .platform_measure_id
}

/// Insert one `MET-WP1-08` OPERAS mapping for a registered pair and return its
/// `mapping_id`.
///
/// The URI strings are fixture text. They are not approved OPERAS values, and
/// the merged mapping table applies no URI semantics to them.
fn insert_operas_mapping_row(pool: &PgPool, platform_id: Uuid, measure_id: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        mapping_id: Uuid,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_operas_mapping \
             (platform_id, measure_id, event_uri, measure_uri, uploader_uri, enabled) \
         VALUES ($1, $2, 'fixture-event-uri', 'fixture-measure-uri', \
                 'fixture-uploader-uri', TRUE) \
         RETURNING mapping_id",
    )
    .bind::<diesel::sql_types::Uuid, _>(platform_id)
    .bind::<diesel::sql_types::Uuid, _>(measure_id)
    .get_result::<Row>(&mut connection)
    .expect("Failed to insert metric_operas_mapping fixture row")
    .mapping_id
}

/// One canonical revision plus the OPERAS mapping registered for that
/// revision's own platform/measure pair — the shape a later WP9 enqueue path
/// would select. Nothing in this slice performs that selection.
struct ExportFixture {
    record: RecordFixture,
    record_id: Uuid,
    record_revision_id: Uuid,
    mapping_id: Uuid,
    platform_measure_id: Uuid,
}

/// Build the canonical revision and its corresponding OPERAS mapping.
fn insert_export_fixture(pool: &PgPool) -> ExportFixture {
    let (record, record_id) = fixture_record(pool, "identity-hash-a");
    let record_revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(record_revision_id),
        record_id,
        1,
        record.import_id,
        1_200,
        "content-hash-a",
        "CURRENT",
        None,
    )
    .expect("Failed to insert the fixture canonical revision");
    let platform_measure_id =
        insert_platform_measure_row(pool, record.platform_id, record.measure_id);
    let mapping_id = insert_operas_mapping_row(pool, record.platform_id, record.measure_id);
    ExportFixture {
        record,
        record_id,
        record_revision_id,
        mapping_id,
        platform_measure_id,
    }
}

/// Add a second canonical revision, superseded so the single-`CURRENT`
/// partial unique index stays satisfied.
fn insert_second_revision(pool: &PgPool, fixture: &ExportFixture) -> Uuid {
    let record_revision_id = Uuid::new_v4();
    insert_revision_row(
        pool,
        Some(record_revision_id),
        fixture.record_id,
        2,
        fixture.record.import_id,
        1_300,
        "content-hash-b",
        "SUPERSEDED",
        None,
    )
    .expect("Failed to insert the second fixture canonical revision");
    record_revision_id
}

/// Register a second platform/measure pair and its OPERAS mapping.
///
/// The registry `code` values deliberately sort after the `test_platform`
/// fixture code, so `metric_record/tests.rs`'s "first platform by code"
/// fixture lookup keeps resolving to the original platform.
fn insert_second_mapping(pool: &PgPool) -> Uuid {
    let platform_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    insert_platform_row(pool, platform_id, "zz_second_platform");
    insert_measure_row(pool, measure_id, "zz_second_measure");
    insert_platform_measure_row(pool, platform_id, measure_id);
    insert_operas_mapping_row(pool, platform_id, measure_id)
}

/// The column values one raw-SQL export insert supplies.
///
/// `export_id` and `created_at` are deliberately never supplied together with
/// the rest, so the database identity and creation-time defaults are exercised
/// rather than restated by the fixture.
struct ExportRow<'a> {
    export_id: Option<Uuid>,
    record_revision_id: Uuid,
    mapping_id: Uuid,
    status: &'a str,
    attempt_count: i32,
    remote_event_id: Option<&'a str>,
    request_hash: Option<&'a str>,
    last_error: Option<&'a str>,
}

impl<'a> ExportRow<'a> {
    /// The minimal valid row: a database-generated identity, an arbitrary
    /// nonblank fixture status, no attempts yet and every nullable delivery
    /// result still unknown.
    fn minimal(record_revision_id: Uuid, mapping_id: Uuid) -> Self {
        Self {
            export_id: None,
            record_revision_id,
            mapping_id,
            status: "fixture-status",
            attempt_count: 0,
            remote_event_id: None,
            request_hash: None,
            last_error: None,
        }
    }
}

/// Insert one export row through raw SQL.
fn insert_export_row(pool: &PgPool, row: ExportRow<'_>) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match row.export_id {
        Some(export_id) => sql_query(
            "INSERT INTO metric_operas_export \
                 (export_id, record_revision_id, mapping_id, status, attempt_count, \
                  remote_event_id, request_hash, last_error) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<diesel::sql_types::Uuid, _>(export_id)
        .bind::<diesel::sql_types::Uuid, _>(row.record_revision_id)
        .bind::<diesel::sql_types::Uuid, _>(row.mapping_id)
        .bind::<diesel::sql_types::Text, _>(row.status)
        .bind::<diesel::sql_types::Integer, _>(row.attempt_count)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.remote_event_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.request_hash)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.last_error)
        .execute(&mut connection),
        None => sql_query(
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, status, attempt_count, \
                  remote_event_id, request_hash, last_error) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(row.record_revision_id)
        .bind::<diesel::sql_types::Uuid, _>(row.mapping_id)
        .bind::<diesel::sql_types::Text, _>(row.status)
        .bind::<diesel::sql_types::Integer, _>(row.attempt_count)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.remote_event_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.request_hash)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(row.last_error)
        .execute(&mut connection),
    }
}

/// The single stored export row.
fn only_export(pool: &PgPool) -> MetricOperasExport {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_operas_export::table
        .first(&mut connection)
        .expect("Failed to load the stored OPERAS export row")
}

#[test]
fn migration_seeds_no_operas_export_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        0,
        "MET-WP1-09 must not seed any metric_operas_export row: this slice \
         creates no export work and decides no revision's eligibility"
    );
}

#[test]
fn a_complete_operas_export_row_round_trips_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Fixture text only. None of these values is a recognised status, a real
    // OPERAS event identifier or a computed payload hash.
    let completed_at = Timestamp::parse_from_rfc3339("2026-09-05T11:22:33Z")
        .expect("Failed to parse the fixture completed_at timestamp");
    let export_id: Uuid = diesel::insert_into(metric_operas_export::table)
        .values((
            metric_operas_export::record_revision_id.eq(fixture.record_revision_id),
            metric_operas_export::mapping_id.eq(fixture.mapping_id),
            metric_operas_export::status.eq("fixture-status"),
            metric_operas_export::attempt_count.eq(3_i32),
            metric_operas_export::remote_event_id.eq("fixture-remote-event-id"),
            metric_operas_export::request_hash.eq("fixture-request-hash"),
            metric_operas_export::last_error.eq("fixture last error detail"),
            metric_operas_export::completed_at.eq(completed_at),
        ))
        .returning(metric_operas_export::export_id)
        .get_result(&mut connection)
        .expect("Failed to insert the complete OPERAS export row");

    let loaded: MetricOperasExport = metric_operas_export::table
        .filter(metric_operas_export::export_id.eq(export_id))
        .first(&mut connection)
        .expect("Failed to load the complete OPERAS export row");
    assert_eq!(loaded.export_id, export_id);
    assert_eq!(loaded.record_revision_id, fixture.record_revision_id);
    assert_eq!(loaded.mapping_id, fixture.mapping_id);
    assert_eq!(loaded.status, "fixture-status");
    assert_eq!(loaded.attempt_count, 3);
    assert_eq!(
        loaded.remote_event_id,
        Some("fixture-remote-event-id".to_string())
    );
    assert_eq!(
        loaded.request_hash,
        Some("fixture-request-hash".to_string())
    );
    assert_eq!(
        loaded.last_error,
        Some("fixture last error detail".to_string())
    );
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
    assert_eq!(loaded.completed_at, Some(completed_at));
}

#[test]
fn every_nullable_export_column_accepts_null() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .expect("a row with every nullable column left NULL must be accepted");

    let loaded = only_export(&pool);
    assert_eq!(
        (
            loaded.remote_event_id,
            loaded.request_hash,
            loaded.last_error,
            loaded.completed_at
        ),
        (None, None, None, None),
        "remote_event_id, request_hash, last_error and completed_at must all \
         stay NULL: the export row exists before any delivery attempt, and \
         this slice attempts none"
    );
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at"
    );
}

#[test]
fn export_id_is_generated_when_omitted_and_honoured_when_supplied() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .expect("Failed to insert the defaulted OPERAS export row");
    let generated = only_export(&pool);
    assert_ne!(
        generated.export_id,
        Uuid::nil(),
        "the repository-standard Metrics UUID default must generate an export_id"
    );

    // A second canonical revision, because UNIQUE(record_revision_id) permits
    // exactly one export row per revision.
    let second_revision_id = insert_second_revision(&pool, &fixture);
    let explicit_id = Uuid::new_v4();
    insert_export_row(
        &pool,
        ExportRow {
            export_id: Some(explicit_id),
            ..ExportRow::minimal(second_revision_id, fixture.mapping_id)
        },
    )
    .expect("Failed to insert the explicitly identified OPERAS export row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let stored: Vec<Uuid> = metric_operas_export::table
        .filter(metric_operas_export::export_id.eq(explicit_id))
        .select(metric_operas_export::export_id)
        .load(&mut connection)
        .expect("Failed to load the explicitly identified export row");
    assert_eq!(
        stored,
        vec![explicit_id],
        "an explicitly supplied export_id must be stored as given"
    );
    assert_ne!(
        generated.export_id, explicit_id,
        "the generated identity must be independent of the explicit one"
    );
}

#[test]
fn zero_and_positive_attempt_counts_round_trip() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    let second_revision_id = insert_second_revision(&pool, &fixture);

    for (revision_id, attempt_count) in [
        (fixture.record_revision_id, 0_i32),
        (second_revision_id, 7_i32),
    ] {
        insert_export_row(
            &pool,
            ExportRow {
                attempt_count,
                ..ExportRow::minimal(revision_id, fixture.mapping_id)
            },
        )
        .unwrap_or_else(|error| {
            panic!("attempt_count = {attempt_count} must be accepted: {error:?}")
        });

        let mut connection = pool.get().expect("Failed to get DB connection");
        let stored: i32 = metric_operas_export::table
            .filter(metric_operas_export::record_revision_id.eq(revision_id))
            .select(metric_operas_export::attempt_count)
            .first(&mut connection)
            .expect("Failed to load the stored attempt_count");
        assert_eq!(
            stored, attempt_count,
            "attempt_count = {attempt_count} must round-trip exactly"
        );
    }

    // Both values had to be stated explicitly; the NOT NULL test below proves
    // the column genuinely has no default to fall back on. Nothing here
    // defines when an attempt starts or increments — that is WP9's contract.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        2
    );
}

#[test]
fn a_negative_attempt_count_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    for attempt_count in [-1_i32, -7, i32::MIN] {
        let result = insert_export_row(
            &pool,
            ExportRow {
                attempt_count,
                ..ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id)
            },
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "attempt_count = {attempt_count} must be rejected by the \
             non-negative CHECK, got {result:?}"
        );
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        0,
        "no rejected export row may have been stored"
    );
}

#[test]
fn arbitrary_nonblank_status_text_round_trips() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    // Deliberately mixed and deliberately not a vocabulary: accepting all of
    // these is the point. The approved design fixes no export status labels,
    // so the column stores nonblank text and nothing more. The padded value
    // must round-trip byte-for-byte, proving nothing trims or normalizes it.
    let values = [
        "fixture-status",
        "PENDING",
        "a",
        "  padded fixture status  ",
        "ünïcödé status",
    ];

    for (index, value) in values.into_iter().enumerate() {
        let (record, record_id) = if index == 0 {
            (None, fixture.record_id)
        } else {
            let record_id = insert_second_record(
                &pool,
                &fixture.record,
                &format!("identity-hash-status-{index}"),
            );
            (Some(record_id), record_id)
        };
        let _ = record;
        let revision_id = Uuid::new_v4();
        let revision_number = if index == 0 { 2 } else { 1 };
        insert_revision_row(
            &pool,
            Some(revision_id),
            record_id,
            revision_number,
            fixture.record.import_id,
            10,
            &format!("content-hash-status-{index}"),
            "SUPERSEDED",
            None,
        )
        .expect("Failed to insert the status fixture revision");

        insert_export_row(
            &pool,
            ExportRow {
                status: value,
                ..ExportRow::minimal(revision_id, fixture.mapping_id)
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank status {value:?} must be accepted: {error:?}")
        });

        let mut connection = pool.get().expect("Failed to get DB connection");
        let stored: String = metric_operas_export::table
            .filter(metric_operas_export::record_revision_id.eq(revision_id))
            .select(metric_operas_export::status)
            .first(&mut connection)
            .expect("Failed to load the stored status");
        assert_eq!(
            stored, value,
            "status {value:?} must round-trip unchanged; storing it is not a \
             claim that it is a recognised export state"
        );
    }
}

#[test]
fn blank_and_whitespace_only_status_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let result = insert_export_row(
            &pool,
            ExportRow {
                status: blank,
                ..ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id)
            },
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank/whitespace-only status variant {index} ({blank:?}) must be \
             rejected by the required-text CHECK, got {result:?}"
        );
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        0,
        "no rejected export row may have been stored"
    );
}

#[test]
fn arbitrary_nonblank_remote_event_id_and_request_hash_round_trip() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    // Neither column has an approved syntax, algorithm, encoding, length or
    // uniqueness rule, so a UUID-shaped identifier, a URI, a hex digest and
    // free text are all equally acceptable opaque values.
    let remote_event_id = "https://metrics.operas-eu.org/fixture/event/1";
    let request_hash = "  0123456789abcdef  ";
    insert_export_row(
        &pool,
        ExportRow {
            remote_event_id: Some(remote_event_id),
            request_hash: Some(request_hash),
            last_error: Some("fixture last error detail"),
            ..ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id)
        },
    )
    .expect("arbitrary nonblank delivery-result text must be accepted");

    let loaded = only_export(&pool);
    assert_eq!(
        loaded.remote_event_id,
        Some(remote_event_id.to_string()),
        "remote_event_id must round-trip unchanged; storing it is not a claim \
         that it is a valid or unique OPERAS event identifier"
    );
    assert_eq!(
        loaded.request_hash,
        Some(request_hash.to_string()),
        "request_hash must round-trip unchanged, untrimmed and unnormalized; \
         storing it is not a claim about any hash algorithm or payload"
    );
    assert_eq!(
        loaded.last_error,
        Some("fixture last error detail".to_string())
    );
    assert_eq!(
        loaded.completed_at, None,
        "no cross-column rule ties completed_at to a populated delivery result"
    );
}

#[test]
fn blank_and_whitespace_only_remote_event_id_is_rejected() {
    assert_blank_nullable_text_rejected("remote_event_id");
}

#[test]
fn blank_and_whitespace_only_request_hash_is_rejected() {
    assert_blank_nullable_text_rejected("request_hash");
}

/// Every blank/whitespace-only *non-null* value of one nullable delivery-result
/// column must be refused, while NULL itself stays valid.
fn assert_blank_nullable_text_rejected(column: &str) {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let base = ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id);
        let row = match column {
            "remote_event_id" => ExportRow {
                remote_event_id: Some(blank),
                ..base
            },
            _ => ExportRow {
                request_hash: Some(blank),
                ..base
            },
        };
        let result = insert_export_row(&pool, row);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank/whitespace-only {column} variant {index} ({blank:?}) must be \
             rejected by the nullable required-text CHECK, got {result:?}"
        );
    }

    // NULL remains valid: the column is unknown until a later WP9 delivery
    // path records it.
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .unwrap_or_else(|error| panic!("a NULL {column} must stay acceptable: {error:?}"));
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        1,
        "only the NULL row may have been stored"
    );
}

#[test]
fn export_not_null_columns_are_enforced() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // record_revision_id, mapping_id, status and attempt_count carry no
    // default at all, so omitting any of them must fail rather than silently
    // resolve to an invented value; export_id and created_at have the
    // repository-standard defaults, so they are probed with an explicit NULL.
    let statements = [
        (
            "export_id",
            "INSERT INTO metric_operas_export \
                 (export_id, record_revision_id, mapping_id, status, attempt_count) \
             VALUES (NULL, $1, $2, 'fixture-status', 0)",
        ),
        (
            "record_revision_id",
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, status, attempt_count) \
             VALUES (NULL, $2, 'fixture-status', 0)",
        ),
        (
            "mapping_id",
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, status, attempt_count) \
             VALUES ($1, NULL, 'fixture-status', 0)",
        ),
        (
            "status",
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, attempt_count) \
             VALUES ($1, $2, 0)",
        ),
        (
            "attempt_count",
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, status) \
             VALUES ($1, $2, 'fixture-status')",
        ),
        (
            "created_at",
            "INSERT INTO metric_operas_export \
                 (record_revision_id, mapping_id, status, attempt_count, created_at) \
             VALUES ($1, $2, 'fixture-status', 0, NULL)",
        ),
    ];

    for (label, statement) in statements {
        let result = sql_query(statement)
            .bind::<diesel::sql_types::Uuid, _>(fixture.record_revision_id)
            .bind::<diesel::sql_types::Uuid, _>(fixture.mapping_id)
            .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "{label} must be NOT NULL with no invented default, got {result:?}"
        );
    }
}

#[test]
fn a_nonexistent_record_revision_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    let result = insert_export_row(
        &pool,
        ExportRow::minimal(Uuid::new_v4(), fixture.mapping_id),
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an export row must name an existing canonical revision, got {result:?}"
    );
}

#[test]
fn a_nonexistent_mapping_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);

    let result = insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, Uuid::new_v4()),
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an export row must name an existing OPERAS mapping, got {result:?}"
    );
}

#[test]
fn at_most_one_export_row_is_permitted_per_canonical_revision() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .expect("the first export row for a canonical revision must be accepted");

    // A different export_id, a different mapping, a different status and a
    // different attempt count still name the same revision, so duplicate
    // outbound delivery stays unrepresentable.
    let other_mapping_id = insert_second_mapping(&pool);
    let duplicate = insert_export_row(
        &pool,
        ExportRow {
            export_id: Some(Uuid::new_v4()),
            mapping_id: other_mapping_id,
            status: "other-fixture-status",
            attempt_count: 5,
            ..ExportRow::minimal(fixture.record_revision_id, other_mapping_id)
        },
    );
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a second export row for the same canonical revision must be rejected \
         so duplicate-delivery prevention stays unambiguous, got {duplicate:?}"
    );

    // Uniqueness is per revision: another revision gets its own export row.
    let second_revision_id = insert_second_revision(&pool, &fixture);
    insert_export_row(
        &pool,
        ExportRow::minimal(second_revision_id, fixture.mapping_id),
    )
    .expect("an export row for a different canonical revision must be accepted");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        2
    );
}

#[test]
fn deleting_a_referenced_record_revision_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .expect("the referencing export row must be accepted");

    let result = delete_row(
        &pool,
        "metric_record_revision",
        "record_revision_id",
        fixture.record_revision_id,
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced metric_record_revision row must be restricted, \
         not cascade away durable outbound export evidence, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        1,
        "the export row must survive the restricted deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record_revision)"),
        1,
        "the referenced canonical revision must survive the restricted deletion"
    );
}

#[test]
fn deleting_a_referenced_operas_mapping_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, fixture.mapping_id),
    )
    .expect("the referencing export row must be accepted");

    let result = delete_row(
        &pool,
        "metric_operas_mapping",
        "mapping_id",
        fixture.mapping_id,
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a referenced metric_operas_mapping row must be restricted, \
         not cascade away durable outbound export evidence, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        1,
        "the export row must survive the restricted deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_mapping)"),
        1,
        "the referenced OPERAS mapping must survive the restricted deletion"
    );

    // The registry pair the mapping configures is likewise untouched.
    assert_eq!(
        scalar_i64(
            &pool,
            &format!(
                "(SELECT COUNT(*) FROM metric_platform_measure \
                  WHERE platform_measure_id = '{}')",
                fixture.platform_measure_id
            ),
        ),
        1
    );
}

#[test]
fn mapping_to_revision_correspondence_is_not_enforced_by_the_database() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_export_fixture(&pool);
    let unrelated_mapping_id = insert_second_mapping(&pool);

    // Both foreign keys are individually satisfied, so the row is
    // representable at raw database level even though the mapping configures a
    // different platform/measure pair than the revision's canonical record.
    // This is the approved WP1/WP9 boundary, not an oversight: the export row
    // deliberately does not duplicate record_id, platform_id or measure_id,
    // and no trigger is added to assert the correspondence. The later
    // export-enqueue/eligibility path must select the mapping from the
    // revision's own canonical record and fail closed otherwise. No such path
    // exists in this slice, so nothing here creates such a row at runtime.
    insert_export_row(
        &pool,
        ExportRow::minimal(fixture.record_revision_id, unrelated_mapping_id),
    )
    .expect(
        "a mismatched-but-individually-valid mapping stays representable: \
         correspondence is a later fail-closed enqueue validation",
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_export)"),
        1
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'public.metric_operas_export'::regclass \
                AND NOT tgisinternal)",
        ),
        0,
        "MET-WP1-09 must install no correspondence trigger on \
         metric_operas_export"
    );
}

#[test]
fn metric_operas_export_has_exactly_the_approved_columns() {
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
         WHERE table_schema = 'public' AND table_name = 'metric_operas_export' \
         ORDER BY ordinal_position",
    )
    .load::<Column>(&mut connection)
    .expect("Failed to read the metric_operas_export columns")
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
            ("export_id", "uuid", "NO"),
            ("record_revision_id", "uuid", "NO"),
            ("mapping_id", "uuid", "NO"),
            ("status", "text", "NO"),
            ("attempt_count", "integer", "NO"),
            ("remote_event_id", "text", "YES"),
            ("request_hash", "text", "YES"),
            ("last_error", "text", "YES"),
            ("created_at", "timestamp with time zone", "NO"),
            ("completed_at", "timestamp with time zone", "YES"),
        ],
        "metric_operas_export must carry exactly the ten approved design \
         fields, in the approved order and nullability, with no retry-time, \
         claim, lease or duplicated identity column"
    );

    // Exactly two defaults: the repository-standard Metrics UUID identity and
    // the repository-standard current-time creation stamp. status,
    // attempt_count and every nullable delivery result must have none.
    #[derive(diesel::QueryableByName)]
    struct Default_ {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_default: String,
    }
    let defaults: Vec<(String, String)> = sql_query(
        "SELECT column_name::text, column_default::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_operas_export' \
           AND column_default IS NOT NULL \
         ORDER BY ordinal_position",
    )
    .load::<Default_>(&mut connection)
    .expect("Failed to read the metric_operas_export defaults")
    .into_iter()
    .map(|row| (row.column_name, row.column_default))
    .collect();
    assert_eq!(
        defaults,
        vec![
            ("export_id".to_string(), "uuid_generate_v4()".to_string()),
            ("created_at".to_string(), "CURRENT_TIMESTAMP".to_string()),
        ],
        "only export_id and created_at may carry defaults; in particular \
         status and attempt_count must have none, because this slice defines \
         no initial export state and no attempt semantics"
    );
}

#[test]
fn metric_operas_export_has_exactly_the_approved_checks() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed: one nonblank status rule, one non-negative
    // attempt rule and two nullable-nonblank delivery-result rules. In
    // particular there is no CHECK enumerating status values and no
    // cross-column rule tying status to completed_at, remote_event_id,
    // request_hash, last_error or attempt_count.
    assert_eq!(
        check_constraint_names(&pool, "metric_operas_export"),
        vec![
            "metric_operas_export_attempt_count_check",
            "metric_operas_export_remote_event_id_check",
            "metric_operas_export_request_hash_check",
            "metric_operas_export_status_check",
        ],
        "metric_operas_export must carry exactly the four approved CHECKs"
    );

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let definitions: Vec<(String, String)> = sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = 'public.metric_operas_export'::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read the CHECK definitions")
    .into_iter()
    .map(|row| (row.conname, row.definition))
    .collect();
    for (name, definition) in &definitions {
        match name.as_str() {
            "metric_operas_export_attempt_count_check" => assert!(
                definition.contains("attempt_count >= 0"),
                "the attempt CHECK must be the non-negative rule and nothing \
                 stronger: {definition}"
            ),
            _ => assert!(
                definition.contains("[^[:space:]]"),
                "every text CHECK must be the existing Metrics required-text \
                 idiom and nothing stronger — no vocabulary, syntax, algorithm \
                 or length rule: {definition}"
            ),
        }
    }
    for name in [
        "metric_operas_export_remote_event_id_check",
        "metric_operas_export_request_hash_check",
    ] {
        let definition = definitions
            .iter()
            .find(|(conname, _)| conname == name)
            .map(|(_, definition)| definition.as_str())
            .unwrap_or_else(|| panic!("{name} must exist"));
        assert!(
            definition.contains("IS NULL"),
            "{name} must stay satisfied by a NULL value: {definition}"
        );
    }
}

#[test]
fn metric_operas_export_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_operas_export");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_operas_export_mapping_id_fkey",
            "metric_operas_export_record_revision_id_fkey",
        ],
        "metric_operas_export must carry exactly the two approved \
         single-column foreign keys: no redundant record, platform or measure \
         key encoding the later runtime correspondence check may exist"
    );

    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour, so durable export evidence cannot be erased through \
             parent deletion: {definition}"
        );
    }
    let mapping_key = &keys[0].1;
    assert!(
        mapping_key.contains("(mapping_id)")
            && mapping_key.contains("metric_operas_mapping(mapping_id)"),
        "the mapping key must reference the MET-WP1-08 surrogate mapping \
         identity: {mapping_key}"
    );
    let revision_key = &keys[1].1;
    assert!(
        revision_key.contains("(record_revision_id)")
            && revision_key.contains("metric_record_revision(record_revision_id)"),
        "the revision key must reference the MET-WP1-04 canonical revision \
         identity: {revision_key}"
    );
}

#[test]
fn metric_operas_export_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_operas_export"),
        vec![
            "metric_operas_export_pkey",
            "metric_operas_export_record_revision_id_key",
        ],
        "metric_operas_export must carry exactly its primary key and the \
         one-export-row-per-revision uniqueness index; no status index, retry \
         index, mapping index, (status, created_at) index or claim index may \
         exist before the WP9 claim/retry protocol, its actual query and its \
         query-plan evidence are approved"
    );
    assert!(
        index_definition(&pool, "metric_operas_export", "metric_operas_export_pkey")
            .contains("(export_id)"),
        "the primary key must be on export_id"
    );
    let unique = index_definition(
        &pool,
        "metric_operas_export",
        "metric_operas_export_record_revision_id_key",
    );
    assert!(
        unique.contains("UNIQUE") && unique.contains("(record_revision_id)"),
        "the uniqueness index must be unique on the canonical revision: {unique}"
    );
}

#[test]
fn no_retry_claim_status_enum_or_reconciliation_object_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // The reconciliation tables remain approved future architecture and must
    // not exist yet. The inbound `metric_operas_import` ledger is no longer
    // asserted absent here: it is MET-WP1-10-owned and created by migration
    // `20260906_v1.9.0`.
    for table in DEFERRED_LEDGER_TABLES {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_class \
                      WHERE relnamespace = 'public'::regnamespace \
                        AND relkind = 'r' AND relname = '{table}')"
                ),
            ),
            0,
            "MET-WP1-09 must not create the deferred ledger table {table}"
        );
    }

    // No retry-time, claim or lease column was smuggled onto the export row.
    // The approved design's section 6.14 names no retry-time field, and the
    // reviewed decision defers the section 14.4 status/retry index — and the
    // representation it would need — to WP9.
    for column in DEFERRED_EXPORT_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_operas_export' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-09 must not add the deferred export column {column}"
        );
    }

    // The export row must not duplicate the revision's canonical dimensions
    // merely to encode the later runtime mapping-correspondence check.
    for column in ["record_id", "platform_id", "measure_id"] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_operas_export' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-09 must not duplicate {column} on the export row"
        );
    }

    // No OPERAS export status vocabulary was created. `typtype = 'e'`
    // restricts the count to enums, because PostgreSQL always creates an
    // implicit composite type named after the table itself.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type \
              WHERE typnamespace = 'public'::regnamespace \
                AND typtype = 'e' \
                AND (typname LIKE 'metric_operas%' OR typname LIKE 'metric_reconciliation%'))",
        ),
        0,
        "MET-WP1-09 must create no OPERAS or reconciliation enum type: the \
         export status vocabulary and transition graph are deliberately \
         undefined at this stage"
    );

    // status stays plain text on the export row.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_operas_export' \
                AND column_name = 'status' AND data_type = 'text')",
        ),
        1,
        "status must remain unconstrained TEXT, not a PostgreSQL enum"
    );
}

#[test]
fn reverting_through_the_operas_export_migration_removes_it_and_reapplication_restores_it() {
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
                AND relname = 'metric_operas_export')",
        ),
        1,
        "the OPERAS export table must exist before reverting"
    );

    revert_through_operas_export_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_operas_export')",
        ),
        0,
        "the downgrade must drop the MET-WP1-09 table"
    );

    // Every predecessor Metrics slice survives, including both tables this
    // ledger references.
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
                                'metric_publisher_platform_approval', \
                                'metric_rollup_delta', 'metric_operas_mapping'))",
        ),
        15,
        "the downgrade must leave the MET-WP1-01..08 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conname IN ('metric_record_revision_pkey', \
                                'metric_operas_mapping_pkey'))",
        ),
        2,
        "the downgrade must not drop the referenced predecessor primary keys"
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

    // Reapplication recreates the empty table with exactly its two indexes,
    // two foreign keys and four CHECKs.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the OPERAS export migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_operas_export)"
        ),
        0,
        "reapplication must seed no OPERAS export row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_operas_export')",
        ),
        2,
        "reapplication must restore exactly the two required indexes"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_operas_export'::regclass AND contype = 'f')",
        ),
        2,
        "reapplication must restore both non-cascading foreign keys"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_operas_export'::regclass AND contype = 'c')",
        ),
        4,
        "reapplication must restore the four approved CHECKs"
    );
}
