//! Focused `MET-WP1-10` database tests for `metric_operas_import`: the
//! approved six-field/default contract, the composite
//! `(remote_instance, remote_event_id)` remote identity, the four opaque
//! required-text columns, deliberate payload-hash cardinality, nullable and
//! non-unique canonical import linkage, restricted (non-cascading) deletion,
//! the exact column/check/foreign-key/index/default inventory and the targeted
//! revert/reapply of the OPERAS import migration.
//!
//! The canonical import fixture reuses the existing `pub(crate)` helpers
//! `fixture_source_account` and `insert_import_row` from
//! `metric_import/tests.rs`, exactly as the approved specification expects. No
//! other model's test module is widened, because this task's write budget does
//! not permit it.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no inbound-ledger row at runtime and implements no OPERAS network
//! or API access, discovery cursor, rolling scan, snapshot import, remote
//! polling, normalization, canonical ingestion, automatic `metric_import`
//! creation or completion, `direct_collection` eligibility, configured-uploader
//! matching, export echo matching or skipping, loop prevention,
//! payload-divergence handling, reconciliation, inbound status vocabulary or
//! state machine, worker claim, lease or retry logic, and nothing here pretends
//! otherwise. In particular, an accepted `remote_instance`, `remote_event_id`,
//! `payload_hash` or `status` string is evidence of nonblank-text storage only,
//! never of a recognised remote instance, a valid remote event identifier, a
//! computed payload hash or a recognised state.
//!
//! Inbound-completeness boundary (section 15.5, reviewed): the existence of
//! this ledger does not imply guaranteed inbound discovery. Guaranteed
//! completeness remains externally blocked without an adequate cursor or
//! created-at stream, replication, a complete snapshot/export or an equivalent
//! reliable incremental mechanism. These tests therefore assert the *absence*
//! of any cursor, remote-created-at, discovery, scan, snapshot, normalized-at,
//! updated-at, export-linkage or completeness column, and no test here may be
//! read as evidence that inbound discovery is complete. WP9 owns discovery
//! modes, loop prevention, reconciliation and completeness reporting.
//!
//! Indexing boundary (section 14.4, reviewed): the design's generic import
//! status/creation-time index requirement is already satisfied by the merged
//! `metric_import_status_created_at_idx` on `metric_import`, so there is no
//! outstanding OPERAS-import operational index requirement and the index
//! inventory below is exactly the composite primary key.

use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricOperasImport;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::{
    check_constraint_names, fixture_source_account, insert_import_row,
};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_definition, index_names};
use crate::model::tests::db::test_db_url;
use crate::model::Timestamp;
use crate::schema::metric_operas_import;

/// The Diesel migration version of `thoth-api/migrations/20260906_v1.9.0`.
const MET_WP1_10_MIGRATION_VERSION: &str = "20260906";

/// The reconciliation ledgers named by the approved design. They stay approved
/// *future* architecture: `MET-WP1-10` creates the inbound OPERAS import
/// ledger only, and WP9 owns reconciliation runs and issues.
const DEFERRED_RECONCILIATION_TABLES: [&str; 2] =
    ["metric_reconciliation_issue", "metric_reconciliation_run"];

/// Column names that would betray an invented inbound discovery, completeness,
/// normalization or export-linkage protocol having been smuggled into this
/// persistence-only slice. The approved design's section 6.14 names none of
/// them, and section 15.5 leaves guaranteed inbound discovery externally
/// blocked and WP9-owned.
const DEFERRED_IMPORT_COLUMNS: [&str; 16] = [
    "completeness",
    "cursor",
    "discovered_at",
    "export_id",
    "is_complete",
    "last_seen_at",
    "normalized_at",
    "remote_created_at",
    "remote_updated_at",
    "scan_id",
    "snapshot_id",
    "sync_cursor",
    "updated_at",
    "uploader_uri",
    "mapping_id",
    "platform_id",
];

/// Revert migrations until the `MET-WP1-10` OPERAS import migration itself has
/// been reverted.
///
/// The same durable pattern as `revert_through_operas_export_migration` and its
/// predecessors: a bare `revert_last_migration` would only mean "the OPERAS
/// import migration" while it happens to be the newest applied migration.
/// Reverting down to and including the target keeps the meaning under any later
/// migration order, and no future migration name is assumed or hard-coded.
fn revert_through_operas_import_migration(connection: &mut PgConnection) {
    let operas_import_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_10_MIGRATION_VERSION);
    assert!(
        operas_import_migration_applied,
        "the MET-WP1-10 OPERAS import migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_10_MIGRATION_VERSION {
            return;
        }
    }
}

/// One canonical `MET-WP1-03` import row, the only referential target this
/// ledger has.
fn fixture_import(pool: &PgPool) -> Uuid {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);
    import_id
}

/// The column values one raw-SQL inbound-ledger insert supplies.
///
/// `created_at` is deliberately never supplied, so the database creation-time
/// default is exercised rather than restated by the fixture.
struct ImportRow<'a> {
    remote_instance: &'a str,
    remote_event_id: &'a str,
    payload_hash: &'a str,
    import_id: Option<Uuid>,
    status: &'a str,
}

impl ImportRow<'_> {
    /// The minimal valid row: an arbitrary nonblank fixture remote identity and
    /// payload hash, no canonical import linked yet and an arbitrary nonblank
    /// fixture status. None of these values is approved OPERAS data.
    fn minimal() -> Self {
        Self {
            remote_instance: "fixture-remote-instance",
            remote_event_id: "fixture-remote-event-id",
            payload_hash: "fixture-payload-hash",
            import_id: None,
            status: "fixture-status",
        }
    }
}

/// Insert one inbound-ledger row through raw SQL.
fn insert_operas_import_row(pool: &PgPool, row: ImportRow<'_>) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_operas_import \
             (remote_instance, remote_event_id, payload_hash, import_id, status) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind::<diesel::sql_types::Text, _>(row.remote_instance)
    .bind::<diesel::sql_types::Text, _>(row.remote_event_id)
    .bind::<diesel::sql_types::Text, _>(row.payload_hash)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(row.import_id)
    .bind::<diesel::sql_types::Text, _>(row.status)
    .execute(&mut connection)
}

/// The single stored inbound-ledger row.
fn only_import(pool: &PgPool) -> MetricOperasImport {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_operas_import::table
        .first(&mut connection)
        .expect("Failed to load the stored OPERAS import row")
}

#[test]
fn migration_seeds_no_operas_import_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0,
        "MET-WP1-10 must not seed any metric_operas_import row: this slice \
         observes no remote OPERAS instance and discovers no remote event"
    );
}

#[test]
fn a_complete_operas_import_row_round_trips_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Fixture text only. None of these values is a real OPERAS instance, a
    // real remote event identifier, a computed payload hash or a recognised
    // inbound state.
    diesel::insert_into(metric_operas_import::table)
        .values((
            metric_operas_import::remote_instance.eq("fixture-remote-instance"),
            metric_operas_import::remote_event_id.eq("fixture-remote-event-id"),
            metric_operas_import::payload_hash.eq("fixture-payload-hash"),
            metric_operas_import::import_id.eq(import_id),
            metric_operas_import::status.eq("fixture-status"),
        ))
        .execute(&mut connection)
        .expect("Failed to insert the complete OPERAS import row");

    let loaded: MetricOperasImport = metric_operas_import::table
        .filter(metric_operas_import::remote_instance.eq("fixture-remote-instance"))
        .filter(metric_operas_import::remote_event_id.eq("fixture-remote-event-id"))
        .first(&mut connection)
        .expect("Failed to load the complete OPERAS import row");
    assert_eq!(loaded.remote_instance, "fixture-remote-instance");
    assert_eq!(loaded.remote_event_id, "fixture-remote-event-id");
    assert_eq!(loaded.payload_hash, "fixture-payload-hash");
    assert_eq!(loaded.import_id, Some(import_id));
    assert_eq!(loaded.status, "fixture-status");
    assert!(
        loaded.created_at > Timestamp::default(),
        "the repository-standard current-time default must populate created_at \
         when the insert omits it"
    );
}

#[test]
fn a_remote_event_is_accepted_before_any_canonical_import_exists() {
    let (_guard, pool) = setup_registry_db();

    // The approved design requires the remote event ID and payload hash to be
    // stored *before* normalization, and permits a linked or skipped event
    // never to need a canonical import of its own. A NULL import_id is
    // therefore a first-class state, not a degraded one.
    insert_operas_import_row(&pool, ImportRow::minimal())
        .expect("a remote event with no canonical import linked must be accepted");

    let loaded = only_import(&pool);
    assert_eq!(
        loaded.import_id, None,
        "import_id must remain NULL until a later WP9 path links a canonical import"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import)"),
        0,
        "no metric_import may be created automatically by storing remote-event evidence"
    );
}

#[test]
fn the_same_remote_event_on_one_instance_is_rejected_as_a_duplicate() {
    let (_guard, pool) = setup_registry_db();
    insert_operas_import_row(&pool, ImportRow::minimal())
        .expect("the first observation of a remote event must be accepted");

    // A differing payload hash, status and import linkage must not create a
    // second durable row: one remote event observed repeatedly resolves to the
    // same durable remote-event evidence. What a *changed* payload for a known
    // remote identity means is a WP9 divergence/reconciliation outcome, not a
    // second row.
    let import_id = fixture_import(&pool);
    let result = insert_operas_import_row(
        &pool,
        ImportRow {
            payload_hash: "a-different-fixture-payload-hash",
            status: "a-different-fixture-status",
            import_id: Some(import_id),
            ..ImportRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "re-observing the same remote event on the same remote instance must be \
         rejected by the composite primary key, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        1,
        "exactly one durable row may exist per (remote_instance, remote_event_id)"
    );
}

#[test]
fn one_remote_event_id_is_accepted_under_two_different_remote_instances() {
    let (_guard, pool) = setup_registry_db();
    insert_operas_import_row(&pool, ImportRow::minimal())
        .expect("the first remote instance's event must be accepted");

    // The approved design deliberately carries remote_instance alongside
    // remote_event_id, so a bare remote event identifier is not established as
    // globally unique.
    insert_operas_import_row(
        &pool,
        ImportRow {
            remote_instance: "a-second-fixture-remote-instance",
            ..ImportRow::minimal()
        },
    )
    .expect(
        "the same remote_event_id under a different remote_instance must be \
         accepted: no global event-ID uniqueness is authorized",
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        2
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT remote_event_id) FROM metric_operas_import)"
        ),
        1,
        "both rows must genuinely share one remote_event_id"
    );
}

#[test]
fn two_remote_event_ids_are_accepted_under_one_remote_instance() {
    let (_guard, pool) = setup_registry_db();
    insert_operas_import_row(&pool, ImportRow::minimal())
        .expect("the first remote event must be accepted");
    insert_operas_import_row(
        &pool,
        ImportRow {
            remote_event_id: "a-second-fixture-remote-event-id",
            ..ImportRow::minimal()
        },
    )
    .expect("a second distinct remote event on one remote instance must be accepted");

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        2
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT remote_instance) FROM metric_operas_import)"
        ),
        1,
        "both rows must genuinely share one remote_instance"
    );
}

#[test]
fn two_different_remote_events_may_share_one_payload_hash() {
    let (_guard, pool) = setup_registry_db();
    insert_operas_import_row(&pool, ImportRow::minimal())
        .expect("the first remote event must be accepted");

    // Identical payload content across two genuinely different remote events is
    // ordinary, so no uniqueness rule may forbid it. Canonical remote identity
    // is the composite key alone; the payload hash is evidence of payload
    // identity for later WP9 divergence handling.
    insert_operas_import_row(
        &pool,
        ImportRow {
            remote_event_id: "a-second-fixture-remote-event-id",
            ..ImportRow::minimal()
        },
    )
    .expect("a duplicate payload_hash across two remote events must be accepted");
    insert_operas_import_row(
        &pool,
        ImportRow {
            remote_instance: "a-second-fixture-remote-instance",
            ..ImportRow::minimal()
        },
    )
    .expect("a duplicate payload_hash across two remote instances must be accepted");

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        3
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT payload_hash) FROM metric_operas_import)"
        ),
        1,
        "all three rows must genuinely share one payload_hash"
    );
}

#[test]
fn arbitrary_nonblank_text_round_trips_in_every_required_column() {
    let (_guard, pool) = setup_registry_db();

    // Deliberately mixed and deliberately not a vocabulary: accepting all of
    // these is the point. The approved design fixes no remote-instance syntax,
    // no remote event-ID syntax, no hash algorithm or encoding and no status
    // labels, so each column stores nonblank text and nothing more. The padded
    // values must round-trip byte-for-byte, proving nothing trims, normalizes
    // or case-folds them.
    let values = [
        "fixture-value",
        "https://example.invalid/operas",
        "not a uri at all",
        "a",
        "  padded fixture value  ",
        "ünïcödé välüé",
        "PENDING",
        "0123456789abcdef",
    ];

    for (index, value) in values.into_iter().enumerate() {
        // Vary the remote event ID so each case gets its own durable row, then
        // put the value under test in each required column in turn.
        let event_id = format!("event-instance-{index}");
        insert_operas_import_row(
            &pool,
            ImportRow {
                remote_instance: value,
                remote_event_id: &event_id,
                ..ImportRow::minimal()
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank remote_instance {value:?} must be accepted: {error:?}")
        });

        let event_id = format!("event-event-{index}");
        insert_operas_import_row(
            &pool,
            ImportRow {
                remote_event_id: value,
                remote_instance: &event_id,
                ..ImportRow::minimal()
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank remote_event_id {value:?} must be accepted: {error:?}")
        });

        let event_id = format!("event-hash-{index}");
        insert_operas_import_row(
            &pool,
            ImportRow {
                payload_hash: value,
                remote_event_id: &event_id,
                ..ImportRow::minimal()
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank payload_hash {value:?} must be accepted: {error:?}")
        });

        let event_id = format!("event-status-{index}");
        insert_operas_import_row(
            &pool,
            ImportRow {
                status: value,
                remote_event_id: &event_id,
                ..ImportRow::minimal()
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank status {value:?} must be accepted: {error:?}")
        });

        let mut connection = pool.get().expect("Failed to get DB connection");
        let stored_instance: String = metric_operas_import::table
            .filter(metric_operas_import::remote_event_id.eq(format!("event-instance-{index}")))
            .select(metric_operas_import::remote_instance)
            .first(&mut connection)
            .expect("Failed to load the stored remote_instance");
        let stored_event_id: String = metric_operas_import::table
            .filter(metric_operas_import::remote_instance.eq(format!("event-event-{index}")))
            .select(metric_operas_import::remote_event_id)
            .first(&mut connection)
            .expect("Failed to load the stored remote_event_id");
        let stored_hash: String = metric_operas_import::table
            .filter(metric_operas_import::remote_event_id.eq(format!("event-hash-{index}")))
            .select(metric_operas_import::payload_hash)
            .first(&mut connection)
            .expect("Failed to load the stored payload_hash");
        let stored_status: String = metric_operas_import::table
            .filter(metric_operas_import::remote_event_id.eq(format!("event-status-{index}")))
            .select(metric_operas_import::status)
            .first(&mut connection)
            .expect("Failed to load the stored status");
        assert_eq!(
            (
                stored_instance.as_str(),
                stored_event_id.as_str(),
                stored_hash.as_str(),
                stored_status.as_str()
            ),
            (value, value, value, value),
            "{value:?} must round-trip unchanged in every required column; \
             storing it is not a claim that it is a recognised remote instance, \
             remote event identifier, computed payload hash or inbound state"
        );
    }
}

#[test]
fn blank_and_whitespace_only_text_is_rejected_in_every_required_column() {
    let (_guard, pool) = setup_registry_db();

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let cases = [
            (
                "remote_instance",
                ImportRow {
                    remote_instance: blank,
                    ..ImportRow::minimal()
                },
            ),
            (
                "remote_event_id",
                ImportRow {
                    remote_event_id: blank,
                    ..ImportRow::minimal()
                },
            ),
            (
                "payload_hash",
                ImportRow {
                    payload_hash: blank,
                    ..ImportRow::minimal()
                },
            ),
            (
                "status",
                ImportRow {
                    status: blank,
                    ..ImportRow::minimal()
                },
            ),
        ];
        for (column, row) in cases {
            let result = insert_operas_import_row(&pool, row);
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "blank/whitespace-only {column} variant {index} ({blank:?}) must be \
                 rejected by the required-text CHECK, got {result:?}"
            );
        }
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0,
        "no rejected inbound-ledger row may have been stored"
    );
}

#[test]
fn import_not_null_columns_are_enforced() {
    let (_guard, pool) = setup_registry_db();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // import_id is deliberately absent from this list: it is the one nullable
    // column, and created_at is omitted because its default supplies a value.
    for column in [
        "remote_instance",
        "remote_event_id",
        "payload_hash",
        "status",
    ] {
        let columns = [
            "remote_instance",
            "remote_event_id",
            "payload_hash",
            "status",
        ];
        let values: Vec<String> = columns
            .iter()
            .map(|name| {
                if *name == column {
                    "NULL".to_string()
                } else {
                    format!("'fixture-{name}'")
                }
            })
            .collect();
        let result = sql_query(format!(
            "INSERT INTO metric_operas_import ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        ))
        .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "a NULL {column} must be rejected, got {result:?}"
        );
    }

    // created_at is NOT NULL too, but is normally supplied by its default; an
    // explicit NULL must still be rejected.
    let result = sql_query(
        "INSERT INTO metric_operas_import \
             (remote_instance, remote_event_id, payload_hash, status, created_at) \
         VALUES ('i', 'e', 'h', 's', NULL)",
    )
    .execute(&mut connection);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicit NULL created_at must be rejected, got {result:?}"
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0
    );
}

#[test]
fn a_nonexistent_import_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let result = insert_operas_import_row(
        &pool,
        ImportRow {
            import_id: Some(Uuid::new_v4()),
            ..ImportRow::minimal()
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "an import_id naming no canonical import must be rejected, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0
    );
}

#[test]
fn several_remote_events_may_reference_one_canonical_import() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);

    // One metric_import may represent an API response or batch containing many
    // distinct remote events, so import_id must be non-unique.
    for index in 0..3 {
        let event_id = format!("fixture-remote-event-{index}");
        insert_operas_import_row(
            &pool,
            ImportRow {
                remote_event_id: &event_id,
                import_id: Some(import_id),
                ..ImportRow::minimal()
            },
        )
        .expect("several remote events must be able to reference one canonical import");
    }

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_operas_import WHERE import_id IS NOT NULL)"
        ),
        3
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT import_id) FROM metric_operas_import)"
        ),
        1,
        "all three rows must genuinely reference the same canonical import"
    );

    // No uniqueness machinery may exist on the referencing side.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_index i \
              JOIN pg_class c ON c.oid = i.indrelid \
              WHERE c.relname = 'metric_operas_import' AND i.indisunique \
                AND 'import_id' = ANY ( \
                    SELECT a.attname FROM pg_attribute a \
                    WHERE a.attrelid = c.oid AND a.attnum = ANY (i.indkey)))",
        ),
        0,
        "no unique index may cover import_id: one canonical import may carry \
         many distinct remote events"
    );
}

#[test]
fn deleting_a_referenced_canonical_import_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let import_id = fixture_import(&pool);
    insert_operas_import_row(
        &pool,
        ImportRow {
            import_id: Some(import_id),
            ..ImportRow::minimal()
        },
    )
    .expect("Failed to insert the referencing inbound-ledger row");

    let result = delete_row(&pool, "metric_import", "import_id", import_id);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a canonical import while durable remote-event evidence \
         references it must fail rather than cascade, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        1,
        "the durable remote-event evidence must survive the refused deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_import)"),
        1,
        "the referenced canonical import must survive the refused deletion"
    );
}

#[test]
fn metric_operas_import_has_exactly_the_approved_columns() {
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
         WHERE table_schema = 'public' AND table_name = 'metric_operas_import' \
         ORDER BY ordinal_position",
    )
    .load::<Column>(&mut connection)
    .expect("Failed to read the metric_operas_import columns")
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
            ("remote_instance", "text", "NO"),
            ("remote_event_id", "text", "NO"),
            ("payload_hash", "text", "NO"),
            ("import_id", "uuid", "YES"),
            ("status", "text", "NO"),
            ("created_at", "timestamp with time zone", "NO"),
        ],
        "metric_operas_import must carry exactly the six approved design \
         fields, in the approved order and nullability, with no surrogate \
         inbound identity and no cursor, remote-created-at, discovery, scan, \
         snapshot, normalized-at, updated-at, export-linkage or completeness \
         column"
    );

    // Exactly one default: the repository-standard current-time creation stamp.
    // In particular there is no surrogate UUID identity default and no status
    // default, because this slice defines no initial inbound state.
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
         WHERE table_schema = 'public' AND table_name = 'metric_operas_import' \
           AND column_default IS NOT NULL \
         ORDER BY ordinal_position",
    )
    .load::<Default_>(&mut connection)
    .expect("Failed to read the metric_operas_import defaults")
    .into_iter()
    .map(|row| (row.column_name, row.column_default))
    .collect();
    assert_eq!(
        defaults,
        vec![("created_at".to_string(), "CURRENT_TIMESTAMP".to_string())],
        "only created_at may carry a default; in particular status must have \
         none, because this slice defines no initial inbound state, and no \
         surrogate identity default exists because the identity is composite"
    );
}

#[test]
fn metric_operas_import_has_exactly_the_approved_checks() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed: four nonblank required-text rules. In
    // particular there is no CHECK enumerating status values, no URI, hostname,
    // UUID, hash-algorithm or length rule, and no cross-column rule tying
    // status to import_id or payload_hash.
    assert_eq!(
        check_constraint_names(&pool, "metric_operas_import"),
        vec![
            "metric_operas_import_payload_hash_check",
            "metric_operas_import_remote_event_id_check",
            "metric_operas_import_remote_instance_check",
            "metric_operas_import_status_check",
        ],
        "metric_operas_import must carry exactly the four approved CHECKs"
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
         WHERE c.conrelid = 'public.metric_operas_import'::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read the CHECK definitions")
    .into_iter()
    .map(|row| (row.conname, row.definition))
    .collect();
    for (name, definition) in &definitions {
        assert!(
            definition.contains("[^[:space:]]"),
            "{name} must be the existing Metrics required-text idiom and \
             nothing stronger — no vocabulary, syntax, algorithm, encoding, \
             case or length rule: {definition}"
        );
        assert!(
            !definition.contains("IS NULL"),
            "{name} guards a required column, so it needs no nullable escape: \
             {definition}"
        );
    }
}

#[test]
fn metric_operas_import_has_exactly_the_authorized_non_cascading_foreign_key() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_operas_import");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec!["metric_operas_import_import_id_fkey"],
        "metric_operas_import must carry exactly one foreign key: no key to \
         metric_operas_export, metric_operas_mapping, metric_platform or \
         metric_measure may exist, because loop prevention is WP9 runtime and \
         reconciliation logic rather than a stored relational identity"
    );

    let (name, definition) = &keys[0];
    assert!(
        !definition.contains("ON DELETE"),
        "{name} must stay non-cascading and use the default restricting \
         behaviour, so durable remote-event evidence cannot be erased through \
         parent deletion: {definition}"
    );
    assert!(
        definition.contains("(import_id)") && definition.contains("metric_import(import_id)"),
        "the import key must reference the MET-WP1-03 canonical import \
         identity: {definition}"
    );
}

#[test]
fn metric_operas_import_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_operas_import"),
        vec!["metric_operas_import_pkey"],
        "metric_operas_import must carry exactly its composite primary-key \
         index; no status, created_at, import_id, payload_hash, bare \
         remote_event_id, cursor or scan index may exist. The approved design's \
         generic import status/creation-time index requirement is already \
         satisfied by the merged metric_import_status_created_at_idx on \
         metric_import, so there is no outstanding OPERAS-import operational \
         index requirement, and WP9 may add one only from actual query-plan \
         evidence"
    );
    let primary_key = index_definition(&pool, "metric_operas_import", "metric_operas_import_pkey");
    assert!(
        primary_key.contains("UNIQUE")
            && primary_key.contains("(remote_instance, remote_event_id)"),
        "the primary key must be the composite remote identity in the approved \
         column order: {primary_key}"
    );
}

#[test]
fn no_discovery_reconciliation_export_linkage_enum_or_trigger_was_introduced() {
    let (_guard, pool) = setup_registry_db();

    // The reconciliation tables remain approved future architecture and must
    // not exist yet. The outbound `metric_operas_export` ledger is deliberately
    // not asserted absent: it is MET-WP1-09-owned and created by migration
    // `20260905_v1.9.0`.
    for table in DEFERRED_RECONCILIATION_TABLES {
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
            "MET-WP1-10 must not create the deferred reconciliation table {table}"
        );
    }

    // No discovery, completeness, normalization or export-linkage column was
    // smuggled onto the inbound row. Section 15.5 leaves guaranteed inbound
    // discovery externally blocked and WP9-owned, and this table must not imply
    // otherwise.
    for column in DEFERRED_IMPORT_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_operas_import' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-10 must not add the deferred inbound column {column}"
        );
    }

    // No inbound OPERAS status vocabulary was created. `typtype = 'e'`
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
        "MET-WP1-10 must create no OPERAS or reconciliation enum type: the \
         inbound status vocabulary and transition graph are deliberately \
         undefined at this stage"
    );

    // status stays plain text on the inbound row.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_operas_import' \
                AND column_name = 'status' AND data_type = 'text')",
        ),
        1,
        "status must remain unconstrained TEXT, not a PostgreSQL enum"
    );

    // No trigger or stored procedure implements a state machine, a loop-
    // prevention rule or an automatic metric_import lifecycle.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'public.metric_operas_import'::regclass \
                AND NOT tgisinternal)",
        ),
        0,
        "MET-WP1-10 must install no trigger on metric_operas_import"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_proc \
              WHERE pronamespace = 'public'::regnamespace \
                AND proname LIKE '%operas_import%')",
        ),
        0,
        "MET-WP1-10 must create no stored procedure for the inbound ledger"
    );
}

#[test]
fn reverting_through_the_operas_import_migration_removes_it_and_reapplication_restores_it() {
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
                AND relname = 'metric_operas_import')",
        ),
        1,
        "the OPERAS import table must exist before reverting"
    );

    revert_through_operas_import_migration(&mut connection);

    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_class \
              WHERE relnamespace = 'public'::regnamespace \
                AND relname = 'metric_operas_import')",
        ),
        0,
        "the downgrade must drop the MET-WP1-10 table"
    );

    // Every predecessor Metrics slice survives, including the table this ledger
    // references and the outbound export ledger it is deliberately unrelated to.
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
                                'metric_rollup_delta', 'metric_operas_mapping', \
                                'metric_operas_export'))",
        ),
        16,
        "the downgrade must leave the MET-WP1-01..09 schema in place"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint WHERE conname = 'metric_import_pkey')",
        ),
        1,
        "the downgrade must not drop the referenced predecessor primary key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND indexname = 'metric_import_status_created_at_idx')",
        ),
        1,
        "the downgrade must leave the merged import status/creation-time index \
         in place: it is what already satisfies the design's generic indexing \
         requirement"
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

    // Reapplication recreates the empty ledger with exactly its one index, one
    // foreign key and four CHECKs.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply migrations from the OPERAS import migration onward");
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM metric_operas_import)"
        ),
        0,
        "reapplication must seed no OPERAS import row"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' AND tablename = 'metric_operas_import')",
        ),
        1,
        "reapplication must restore exactly the composite primary-key index"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_operas_import'::regclass AND contype = 'f')",
        ),
        1,
        "reapplication must restore the single non-cascading foreign key"
    );
    assert_eq!(
        count_objects(
            &mut connection,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE conrelid = 'public.metric_operas_import'::regclass AND contype = 'c')",
        ),
        4,
        "reapplication must restore the four approved CHECKs"
    );
}
