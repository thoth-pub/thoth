//! Focused `MET-WP1-02` database tests for the `metric_source_checkpoint`
//! durable checkpoint/lease storage: identity, non-cascading account foreign
//! key, nullable JSONB cursor, progress/lease/error round-trips, the
//! repository-standard `updated_at` trigger and the exact index inventory.
//!
//! Deliberately absent: any claim, lease-acquisition, `FOR UPDATE SKIP
//! LOCKED` or stale-lease test. The operation-level concurrency protocol is
//! outside this slice and must not be pretend-tested here.
//!
//! `MET-WP2-03` adds the closed `thoth-period-manifest-cursor/1` codec
//! evidence: SQL `NULL` as the only empty history, the exact schema version
//! and member sets, canonical dates and digests, strict ordering, the 64-entry
//! bound, deterministic encoding, insertion, replacement and oldest-period
//! eviction, and that the database column itself stays generic JSONB.

use chrono::{Days, NaiveDate};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::{
    MetricPeriodManifestCursor, MetricPeriodManifestCursorError, MetricSourceCheckpoint,
    PERIOD_MANIFEST_CURSOR_MAX_ENTRIES, PERIOD_MANIFEST_CURSOR_SCHEMA,
};
use crate::db::PgPool;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_source_account::tests::{fixture_source_and_platform, insert_account_row};
use crate::model::Timestamp;
use crate::schema::metric_source_checkpoint;

/// Insert one referenced source/platform/account chain for checkpoint tests.
fn fixture_account(pool: &PgPool) -> Uuid {
    let (source_id, platform_id) = fixture_source_and_platform(pool);
    let source_account_id = Uuid::new_v4();
    insert_account_row(pool, source_account_id, source_id, platform_id, "account-1");
    source_account_id
}

fn insert_checkpoint_raw(
    pool: &PgPool,
    source_account_id: Uuid,
    partition_key: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_source_checkpoint (source_account_id, partition_key) \
         VALUES ($1, $2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(source_account_id)
    .bind::<diesel::sql_types::Text, _>(partition_key)
    .execute(&mut connection)
}

#[test]
fn migration_seeds_no_checkpoint_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_source_checkpoint)"),
        0,
        "MET-WP1-02 must not seed any metric_source_checkpoint row"
    );
}

#[test]
fn checkpoint_deliberately_has_no_created_at_column() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' \
                AND table_name = 'metric_source_checkpoint' \
                AND column_name = 'created_at')",
        ),
        0,
        "the approved design specifies no created_at column for checkpoints"
    );
}

#[test]
fn blank_partition_key_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_checkpoint_raw(&pool, source_account_id, blank);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "blank partition key {blank:?} must fail the check constraint: {result:?}"
        );
    }
}

#[test]
fn checkpoint_identity_is_unique_per_account_and_partition_key() {
    let (_guard, pool) = setup_registry_db();
    let (source_id, platform_id) = fixture_source_and_platform(&pool);
    let source_account_id = Uuid::new_v4();
    insert_account_row(
        &pool,
        source_account_id,
        source_id,
        platform_id,
        "account-1",
    );
    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("First insert must pass");
    let duplicate = insert_checkpoint_raw(&pool, source_account_id, "2026-07");
    assert!(
        matches!(
            duplicate,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a duplicate (source_account_id, partition_key) pair must fail the unique \
         constraint: {duplicate:?}"
    );

    // The same partition key under another account is a different identity.
    let other_account_id = Uuid::new_v4();
    insert_account_row(&pool, other_account_id, source_id, platform_id, "account-2");
    insert_checkpoint_raw(&pool, other_account_id, "2026-07")
        .expect("The same partition key under another account must pass");
}

#[test]
fn checkpoint_foreign_key_requires_an_existing_account_and_restricts_deletion() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);

    let unknown_account = insert_checkpoint_raw(&pool, Uuid::new_v4(), "2026-07");
    assert!(
        matches!(
            unknown_account,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a checkpoint referencing an unknown account must fail the foreign key: \
         {unknown_account:?}"
    );

    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("Insert must pass");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let delete_account =
        sql_query("DELETE FROM metric_source_account WHERE source_account_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(source_account_id)
            .execute(&mut connection);
    assert!(
        matches!(
            delete_account,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting an account still referenced by a checkpoint must be restricted, \
         not cascaded: {delete_account:?}"
    );
}

#[test]
fn metric_source_checkpoint_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let cursor = json!({
        "schemaVersion": "thoth-period-manifest-cursor/1",
        "entries": [{"periodStart": "2026-07-30", "manifestDigest": "ab".repeat(32)}],
    });
    let discovered = Timestamp::parse_from_rfc3339("2026-08-01T12:00:00Z")
        .expect("Failed to parse the discovery timestamp");
    let completed = Timestamp::parse_from_rfc3339("2026-08-01T12:05:00Z")
        .expect("Failed to parse the completion timestamp");
    let lease_expiry = Timestamp::parse_from_rfc3339("2026-08-01T12:10:00Z")
        .expect("Failed to parse the lease expiry timestamp");
    let period_end = NaiveDate::from_ymd_opt(2026, 7, 31).expect("Failed to build the period end");

    let progressed_id: Uuid = diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("2026-07"),
            metric_source_checkpoint::cursor.eq(cursor.clone()),
            metric_source_checkpoint::last_discovered_at.eq(discovered),
            metric_source_checkpoint::last_completed_at.eq(completed),
            metric_source_checkpoint::last_successful_period_end.eq(period_end),
            metric_source_checkpoint::lease_owner.eq("sphinx-worker-1"),
            metric_source_checkpoint::lease_expires_at.eq(lease_expiry),
            metric_source_checkpoint::last_error.eq("upstream returned HTTP 503"),
        ))
        .returning(metric_source_checkpoint::source_checkpoint_id)
        .get_result(&mut connection)
        .expect("Failed to insert progressed checkpoint row");
    diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("2026-08"),
        ))
        .execute(&mut connection)
        .expect("Failed to insert fresh checkpoint row");

    let progressed: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load progressed checkpoint row");
    assert_eq!(progressed.source_checkpoint_id, progressed_id);
    assert_eq!(progressed.source_account_id, source_account_id);
    assert_eq!(progressed.partition_key, "2026-07");
    assert_eq!(progressed.cursor, Some(cursor));
    assert_eq!(progressed.last_discovered_at, Some(discovered));
    assert_eq!(progressed.last_completed_at, Some(completed));
    assert_eq!(progressed.last_successful_period_end, Some(period_end));
    assert_eq!(progressed.lease_owner.as_deref(), Some("sphinx-worker-1"));
    assert_eq!(progressed.lease_expires_at, Some(lease_expiry));
    assert_eq!(
        progressed.last_error.as_deref(),
        Some("upstream returned HTTP 503")
    );

    let fresh: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-08"))
        .first(&mut connection)
        .expect("Failed to load fresh checkpoint row");
    assert_eq!(fresh.cursor, None);
    assert_eq!(fresh.last_discovered_at, None);
    assert_eq!(fresh.last_completed_at, None);
    assert_eq!(fresh.last_successful_period_end, None);
    assert_eq!(fresh.lease_owner, None);
    assert_eq!(fresh.lease_expires_at, None);
    assert_eq!(fresh.last_error, None);
}

#[test]
fn checkpoint_updated_at_is_maintained_by_the_repository_standard_trigger() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    insert_checkpoint_raw(&pool, source_account_id, "2026-07").expect("Insert must pass");
    let mut connection = pool.get().expect("Failed to get DB connection");

    let initial: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load the fresh checkpoint row");

    sql_query(
        "UPDATE metric_source_checkpoint SET last_error = 'transient failure' \
         WHERE partition_key = '2026-07'",
    )
    .execute(&mut connection)
    .expect("Failed to update the checkpoint row");

    let updated: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("2026-07"))
        .first(&mut connection)
        .expect("Failed to load the updated checkpoint row");
    assert!(
        updated.updated_at > initial.updated_at,
        "the set_updated_at trigger must advance updated_at on update \
         ({:?} -> {:?})",
        initial.updated_at,
        updated.updated_at
    );
}

#[test]
fn lease_expiry_has_the_required_operational_index() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes \
              WHERE schemaname = 'public' \
                AND tablename = 'metric_source_checkpoint' \
                AND indexname = 'metric_source_checkpoint_lease_expires_at_idx' \
                AND indexdef LIKE '%(lease_expires_at)%')",
        ),
        1,
        "the design-required operational index on lease_expires_at must exist"
    );
}

#[test]
fn source_state_tables_have_no_speculative_secondary_index() {
    let (_guard, pool) = setup_registry_db();
    // The complete intended index inventory is exactly: three primary keys,
    // metric_source(code) UNIQUE, the two composite identity UNIQUEs, the
    // MET-WP2-01A metric_source_account(code) UNIQUE, and the single
    // operational lease-expiry index.
    for (table, expected) in [
        ("metric_source", 2),
        ("metric_source_account", 3),
        ("metric_source_checkpoint", 3),
    ] {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM pg_indexes \
                      WHERE schemaname = 'public' AND tablename = '{table}')"
                ),
            ),
            expected,
            "{table} must carry exactly its constraint-derived indexes \
             (plus, for checkpoints, the lease-expiry index)"
        );
    }
}

// --------------------------------------------------------------------------
// MET-WP2-03: the closed `thoth-period-manifest-cursor/1` codec
// --------------------------------------------------------------------------
//
// These tests exercise the codec directly. The claim-time decode, the
// server-derived advancement and every lifecycle consequence are proven in
// `crate::model::metric_ingestion_lifecycle::tests`.

fn period(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

/// A distinct valid manifest digest: 64 lowercase hexadecimal characters.
fn manifest_digest(seed: u64) -> String {
    format!("{seed:064x}")
}

fn stored_entry(period_start: &str, digest: &str) -> JsonValue {
    json!({"periodStart": period_start, "manifestDigest": digest})
}

fn stored_cursor(entries: Vec<JsonValue>) -> JsonValue {
    json!({"schemaVersion": "thoth-period-manifest-cursor/1", "entries": entries})
}

/// `count` canonical consecutive daily entries starting at 2026-01-01.
fn consecutive_entries(count: u64) -> Vec<JsonValue> {
    (0..count)
        .map(|offset| {
            stored_entry(
                &period(2026, 1, 1)
                    .checked_add_days(Days::new(offset))
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string(),
                &manifest_digest(offset),
            )
        })
        .collect()
}

fn decoded(stored: &JsonValue) -> MetricPeriodManifestCursor {
    MetricPeriodManifestCursor::decode(Some(stored))
        .expect("a supported cursor decodes")
        .expect("a stored cursor is not SQL NULL")
}

fn periods(cursor: &MetricPeriodManifestCursor) -> Vec<(NaiveDate, String)> {
    cursor
        .retained_entries()
        .iter()
        .map(|entry| (entry.period(), entry.digest().to_string()))
        .collect()
}

fn assert_unsupported(label: &str, stored: JsonValue) {
    assert_eq!(
        MetricPeriodManifestCursor::decode(Some(&stored)),
        Err(MetricPeriodManifestCursorError),
        "{label} must fail closed: {stored}"
    );
}

#[test]
fn the_shared_constants_are_the_approved_schema_and_bound() {
    assert_eq!(
        PERIOD_MANIFEST_CURSOR_SCHEMA,
        "thoth-period-manifest-cursor/1"
    );
    assert_eq!(PERIOD_MANIFEST_CURSOR_MAX_ENTRIES, 64);
}

#[test]
fn sql_null_is_the_only_empty_history() {
    assert_eq!(MetricPeriodManifestCursor::decode(None), Ok(None));
    assert_unsupported("a JSON null", JsonValue::Null);
    assert_unsupported("zero entries", stored_cursor(vec![]));
    assert_unsupported("an empty object", json!({}));
}

#[test]
fn a_supported_cursor_round_trips_exactly_at_one_and_sixty_four_entries() {
    for count in [1, 2, 64] {
        let stored = stored_cursor(consecutive_entries(count));
        let cursor = decoded(&stored);
        assert_eq!(cursor.retained_entries().len() as u64, count);
        assert_eq!(cursor.encode(), stored, "{count} entries");
    }
    let cursor = decoded(&stored_cursor(vec![
        stored_entry("2024-02-29", &manifest_digest(1)),
        stored_entry("2026-03-01", &manifest_digest(2)),
    ]));
    assert_eq!(
        periods(&cursor),
        [
            (period(2024, 2, 29), manifest_digest(1)),
            (period(2026, 3, 1), manifest_digest(2)),
        ]
    );
}

#[test]
fn only_the_exact_schema_version_and_key_sets_are_supported() {
    let entry = || stored_entry("2026-03-01", &manifest_digest(1));
    for version in [
        json!("thoth-period-manifest-cursor/2"),
        json!("thoth-period-manifest-cursor/1 "),
        json!("THOTH-PERIOD-MANIFEST-CURSOR/1"),
        json!("thoth-period-manifest-cursor"),
        json!(""),
        json!(1),
        JsonValue::Null,
    ] {
        assert_unsupported(
            "a schema version",
            json!({"schemaVersion": version, "entries": [entry()]}),
        );
    }
    assert_unsupported("no schema version", json!({"entries": [entry()]}));
    assert_unsupported(
        "no entries member",
        json!({"schemaVersion": "thoth-period-manifest-cursor/1"}),
    );
    assert_unsupported(
        "entries that are not an array",
        json!({"schemaVersion": "thoth-period-manifest-cursor/1", "entries": entry()}),
    );
    for extension in ["objects", "cursor", "lastModified", "schemaVersion2"] {
        let mut stored = stored_cursor(vec![entry()]);
        stored
            .as_object_mut()
            .unwrap()
            .insert(extension.to_string(), json!(["cf/a.gz"]));
        assert_unsupported(extension, stored);
    }
    for extension in ["objectKey", "requestId", "etag", "size"] {
        let mut member = entry();
        member
            .as_object_mut()
            .unwrap()
            .insert(extension.to_string(), json!("x"));
        assert_unsupported(extension, stored_cursor(vec![member]));
    }
    assert_unsupported(
        "an entry without a digest",
        stored_cursor(vec![json!({"periodStart": "2026-03-01"})]),
    );
    assert_unsupported(
        "an entry without a period",
        stored_cursor(vec![json!({"manifestDigest": manifest_digest(1)})]),
    );
    for not_an_entry in [json!("2026-03-01"), json!(["2026-03-01"]), JsonValue::Null] {
        assert_unsupported("a non-object entry", stored_cursor(vec![not_an_entry]));
    }
    for not_a_cursor in [json!([]), json!("cursor"), json!(64), json!(true)] {
        assert_unsupported("a non-object cursor", not_a_cursor);
    }
}

#[test]
fn a_period_start_must_round_trip_as_one_canonical_date() {
    for canonical in ["2026-03-01", "2024-02-29", "0001-01-01", "9999-12-31"] {
        let cursor = decoded(&stored_cursor(vec![stored_entry(
            canonical,
            &manifest_digest(1),
        )]));
        assert_eq!(
            cursor.retained_entries()[0]
                .period()
                .format("%Y-%m-%d")
                .to_string(),
            canonical
        );
    }
    for non_canonical in [
        "2026-3-01",
        "2026-03-1",
        "2026-02-30",
        "2023-02-29",
        "2026-13-01",
        "26-03-01",
        "+2026-03-01",
        "02026-03-01",
        "10000-01-01",
        "-001-01-01",
        " 2026-03-01",
        "2026-03-01 ",
        "2026-03-01T00:00:00",
        "2026-03-01Z",
        "2026/03/01",
        "20260301",
        "",
    ] {
        assert_unsupported(
            non_canonical,
            stored_cursor(vec![stored_entry(non_canonical, &manifest_digest(1))]),
        );
    }
    for not_a_string in [json!(20260301), JsonValue::Null, json!(["2026-03-01"])] {
        assert_unsupported(
            "a non-string period",
            stored_cursor(vec![
                json!({"periodStart": not_a_string, "manifestDigest": manifest_digest(1)}),
            ]),
        );
    }
}

#[test]
fn a_manifest_digest_must_be_64_lowercase_hexadecimal_characters() {
    let valid = manifest_digest(u64::MAX);
    for accepted in [valid.clone(), "0".repeat(64), "abcdef0123456789".repeat(4)] {
        decoded(&stored_cursor(vec![stored_entry("2026-03-01", &accepted)]));
    }
    for rejected in [
        valid.to_uppercase(),
        format!("A{}", &valid[1..]),
        valid[1..].to_string(),
        format!("{valid}0"),
        format!("g{}", &valid[1..]),
        format!(" {}", &valid[1..]),
        format!("{}\n", &valid[1..]),
        "é".repeat(32),
        String::new(),
    ] {
        assert_unsupported(
            &rejected,
            stored_cursor(vec![stored_entry("2026-03-01", &rejected)]),
        );
    }
    for not_a_string in [json!(1), JsonValue::Null, json!([valid.clone()])] {
        assert_unsupported(
            "a non-string digest",
            stored_cursor(vec![
                json!({"periodStart": "2026-03-01", "manifestDigest": not_a_string}),
            ]),
        );
    }
}

#[test]
fn entries_must_be_strictly_ascending_and_at_most_sixty_four() {
    let digest = manifest_digest(1);
    assert_unsupported(
        "a descending pair",
        stored_cursor(vec![
            stored_entry("2026-03-02", &digest),
            stored_entry("2026-03-01", &digest),
        ]),
    );
    assert_unsupported(
        "a duplicated period with one digest",
        stored_cursor(vec![
            stored_entry("2026-03-01", &digest),
            stored_entry("2026-03-01", &digest),
        ]),
    );
    assert_unsupported(
        "a duplicated period with two digests",
        stored_cursor(vec![
            stored_entry("2026-03-01", &digest),
            stored_entry("2026-03-01", &manifest_digest(2)),
        ]),
    );
    let mut unsorted = consecutive_entries(64);
    unsorted.swap(10, 11);
    assert_unsupported("one swapped pair among 64", stored_cursor(unsorted));
    assert_unsupported("65 entries", stored_cursor(consecutive_entries(65)));
    assert_unsupported("1000 entries", stored_cursor(consecutive_entries(1000)));
}

#[test]
fn encoding_is_canonical_and_deterministic() {
    let stored = stored_cursor(consecutive_entries(3));
    let cursor = decoded(&stored);
    let encoded = cursor.encode();
    assert_eq!(encoded, stored);
    assert_eq!(
        serde_json::to_string(&encoded).unwrap(),
        serde_json::to_string(&cursor.encode()).unwrap()
    );
    let object = encoded.as_object().unwrap();
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["entries", "schemaVersion"]);
    for entry in object["entries"].as_array().unwrap() {
        let mut keys: Vec<&str> = entry
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["manifestDigest", "periodStart"]);
    }
    // Decoding what was encoded is the identity.
    assert_eq!(decoded(&encoded), cursor);
}

#[test]
fn recording_inserts_or_replaces_one_period_and_evicts_only_the_oldest() {
    // No history becomes a one-entry cursor.
    let one = MetricPeriodManifestCursor::record_accepted_manifest(
        None,
        period(2026, 3, 5),
        &manifest_digest(5),
    )
    .unwrap();
    assert_eq!(periods(&one), [(period(2026, 3, 5), manifest_digest(5))]);
    assert_eq!(
        one.encode(),
        stored_cursor(vec![stored_entry("2026-03-05", &manifest_digest(5))])
    );

    // Another period is inserted in chronological order, not insertion order.
    let two = MetricPeriodManifestCursor::record_accepted_manifest(
        Some(&one),
        period(2026, 3, 2),
        &manifest_digest(2),
    )
    .unwrap();
    assert_eq!(
        periods(&two),
        [
            (period(2026, 3, 2), manifest_digest(2)),
            (period(2026, 3, 5), manifest_digest(5)),
        ]
    );
    // The same period and digest is the same value; a changed digest replaces
    // exactly that period.
    assert_eq!(
        MetricPeriodManifestCursor::record_accepted_manifest(
            Some(&two),
            period(2026, 3, 5),
            &manifest_digest(5)
        )
        .unwrap(),
        two
    );
    let replaced = MetricPeriodManifestCursor::record_accepted_manifest(
        Some(&two),
        period(2026, 3, 5),
        &manifest_digest(55),
    )
    .unwrap();
    assert_eq!(
        periods(&replaced),
        [
            (period(2026, 3, 2), manifest_digest(2)),
            (period(2026, 3, 5), manifest_digest(55)),
        ]
    );

    // At the bound, a newer period evicts only the chronologically oldest.
    let full = decoded(&stored_cursor(consecutive_entries(64)));
    let newest = period(2026, 1, 1).checked_add_days(Days::new(64)).unwrap();
    let advanced = MetricPeriodManifestCursor::record_accepted_manifest(
        Some(&full),
        newest,
        &manifest_digest(64),
    )
    .unwrap();
    let mut expected = periods(&full);
    expected.remove(0);
    expected.push((newest, manifest_digest(64)));
    assert_eq!(periods(&advanced), expected);
    assert_eq!(advanced.retained_entries().len(), 64);

    // A period in the middle of a full cursor still evicts the oldest only.
    let gap = decoded(&stored_cursor(
        consecutive_entries(65)
            .into_iter()
            .enumerate()
            .filter(|(index, _)| *index != 30)
            .map(|(_, entry)| entry)
            .collect(),
    ));
    let middle = period(2026, 1, 1).checked_add_days(Days::new(30)).unwrap();
    let filled = MetricPeriodManifestCursor::record_accepted_manifest(
        Some(&gap),
        middle,
        &manifest_digest(30),
    )
    .unwrap();
    assert_eq!(filled.retained_entries().len(), 64);
    assert_eq!(filled.retained_entries()[0].period(), period(2026, 1, 2));
    assert_eq!(filled.retained_entries()[29].period(), middle);

    // A period older than all 64 retained ones is inserted and evicted at once,
    // leaving the cursor unchanged.
    let older = MetricPeriodManifestCursor::record_accepted_manifest(
        Some(&full),
        period(2025, 12, 31),
        &manifest_digest(999),
    )
    .unwrap();
    assert_eq!(older, full);
}

#[test]
fn recording_never_produces_a_cursor_the_decoder_would_refuse() {
    let one = decoded(&stored_cursor(consecutive_entries(1)));
    for digest in [
        manifest_digest(0xabcdef).to_uppercase(),
        manifest_digest(1)[1..].to_string(),
        "z".repeat(64),
    ] {
        assert_eq!(
            MetricPeriodManifestCursor::record_accepted_manifest(
                Some(&one),
                period(2026, 3, 1),
                &digest
            ),
            Err(MetricPeriodManifestCursorError),
            "{digest}"
        );
    }
    for unrenderable in [
        NaiveDate::from_ymd_opt(10000, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(-1, 12, 31).unwrap(),
    ] {
        assert_eq!(
            MetricPeriodManifestCursor::record_accepted_manifest(
                None,
                unrenderable,
                &manifest_digest(1)
            ),
            Err(MetricPeriodManifestCursorError),
            "{unrenderable}"
        );
    }
}

#[test]
fn a_decoding_failure_reveals_nothing_of_the_stored_value() {
    let error = MetricPeriodManifestCursor::decode(Some(&json!({
        "schemaVersion": "thoth-period-manifest-cursor/1",
        "entries": [{"periodStart": "2026-03-01", "manifestDigest": "leak-sentinel"}],
    })))
    .unwrap_err();
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("leak-sentinel"), "{rendered}");
    assert!(!rendered.contains("2026-03-01"), "{rendered}");
}

#[test]
fn the_database_column_stays_generic_and_the_codec_is_the_gate() {
    let (_guard, pool) = setup_registry_db();
    let source_account_id = fixture_account(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    // A canonical cursor survives the JSONB round trip unchanged.
    let canonical = decoded(&stored_cursor(consecutive_entries(64)));
    diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("canonical"),
            metric_source_checkpoint::cursor.eq(canonical.encode()),
        ))
        .execute(&mut connection)
        .expect("Failed to insert a canonical cursor");
    let stored: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("canonical"))
        .first(&mut connection)
        .expect("Failed to load the canonical cursor");
    assert_eq!(
        MetricPeriodManifestCursor::decode(stored.cursor.as_ref()),
        Ok(Some(canonical))
    );

    // No database constraint was added: an unsupported value is still
    // storable, and only the codec refuses it.
    diesel::insert_into(metric_source_checkpoint::table)
        .values((
            metric_source_checkpoint::source_account_id.eq(source_account_id),
            metric_source_checkpoint::partition_key.eq("unsupported"),
            metric_source_checkpoint::cursor.eq(json!({"page": 3})),
        ))
        .execute(&mut connection)
        .expect("the column accepts any JSONB value");
    let stored: MetricSourceCheckpoint = metric_source_checkpoint::table
        .filter(metric_source_checkpoint::partition_key.eq("unsupported"))
        .first(&mut connection)
        .expect("Failed to load the unsupported cursor");
    assert_eq!(
        MetricPeriodManifestCursor::decode(stored.cursor.as_ref()),
        Err(MetricPeriodManifestCursorError)
    );
}
