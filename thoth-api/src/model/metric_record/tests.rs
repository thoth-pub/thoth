//! Focused `MET-WP1-04` database tests for `metric_record`: the approved
//! field/default contract, the complete authorized CHECK and foreign-key
//! inventory, the separate Metrics alpha-2 country representation, half-open
//! period ordering, same-record current-revision integrity and the four
//! design-required access indexes.
//!
//! The bibliographic `publisher -> imprint -> work -> publication` and
//! `institution` fixtures this slice needs are defined here rather than by
//! widening another model's test module: no existing `pub(crate)` helper
//! supplies them, and `thoth-api/src/model/tests.rs` is outside this task's
//! write budget. They are `pub(crate)` so the revision and provenance test
//! modules reuse them. The registry, source-account and import fixtures are
//! the existing `pub(crate)` helpers, consumed as-is.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! implements no identity or content hashing, no first-arrival, duplicate,
//! revision, conflict or retraction transaction and no period-overlap
//! detection, and nothing here pretends otherwise.

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::MetricRecord;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_import::tests::{fixture_source_account, insert_import_row};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_platform_measure::MetricReportingGrain;
use crate::model::Timestamp;
use crate::schema::metric_record;

/// The Diesel migration version of `thoth-api/migrations/20260831_v1.9.0`.
pub(crate) const MET_WP1_04_MIGRATION_VERSION: &str = "20260831";

/// Revert migrations until the `MET-WP1-04` record-schema migration itself
/// has been reverted.
///
/// The same durable pattern as `revert_through_registry_migration`,
/// `revert_through_source_state_migration` and
/// `revert_through_import_state_migration`: a single `revert_last_migration`
/// would only mean "the record-schema migration" while it happens to be the
/// newest applied migration. Reverting down to and including the target keeps
/// the meaning under any later migration order, and no future migration name
/// is assumed or hard-coded.
pub(crate) fn revert_through_record_schema_migration(connection: &mut PgConnection) {
    let record_migration_applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP1_04_MIGRATION_VERSION);
    assert!(
        record_migration_applied,
        "the MET-WP1-04 record-schema migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP1_04_MIGRATION_VERSION {
            return;
        }
    }
}

/// The canonical Thoth entities one metric record must resolve to.
pub(crate) struct RecordFixture {
    pub(crate) work_id: Uuid,
    pub(crate) publication_id: Uuid,
    pub(crate) institution_id: Uuid,
    pub(crate) platform_id: Uuid,
    pub(crate) measure_id: Uuid,
    pub(crate) source_account_id: Uuid,
    pub(crate) import_id: Uuid,
}

/// Insert one `publisher -> imprint -> work -> publication` chain plus one
/// `institution`, and return them with the registry/source/import ids a
/// canonical record row needs.
///
/// `work_status` is `forthcoming` and `edition` is supplied so the existing
/// `work_active_publication_date_check` and `work_non_chapter_has_edition`
/// constraints are satisfied without inventing publication dates.
pub(crate) fn insert_record_fixture(pool: &PgPool) -> RecordFixture {
    let source_account_id = fixture_source_account(pool);
    let import_id = Uuid::new_v4();
    insert_import_row(pool, import_id, source_account_id);

    let publisher_id = Uuid::new_v4();
    let imprint_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let publication_id = Uuid::new_v4();
    let institution_id = Uuid::new_v4();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("INSERT INTO publisher (publisher_id, publisher_name) VALUES ($1, 'Test publisher')")
        .bind::<diesel::sql_types::Uuid, _>(publisher_id)
        .execute(&mut connection)
        .expect("Failed to insert publisher fixture row");
    sql_query(
        "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) \
         VALUES ($1, $2, 'Test imprint')",
    )
    .bind::<diesel::sql_types::Uuid, _>(imprint_id)
    .bind::<diesel::sql_types::Uuid, _>(publisher_id)
    .execute(&mut connection)
    .expect("Failed to insert imprint fixture row");
    sql_query(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
         VALUES ($1, 'monograph', 'forthcoming', $2, 1)",
    )
    .bind::<diesel::sql_types::Uuid, _>(work_id)
    .bind::<diesel::sql_types::Uuid, _>(imprint_id)
    .execute(&mut connection)
    .expect("Failed to insert work fixture row");
    sql_query(
        "INSERT INTO publication (publication_id, publication_type, work_id) \
         VALUES ($1, 'PDF', $2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(publication_id)
    .bind::<diesel::sql_types::Uuid, _>(work_id)
    .execute(&mut connection)
    .expect("Failed to insert publication fixture row");
    sql_query("INSERT INTO institution (institution_id, institution_name) VALUES ($1, 'Test institution')")
        .bind::<diesel::sql_types::Uuid, _>(institution_id)
        .execute(&mut connection)
        .expect("Failed to insert institution fixture row");

    let (platform_id, measure_id) = registry_ids(&mut connection);
    RecordFixture {
        work_id,
        publication_id,
        institution_id,
        platform_id,
        measure_id,
        source_account_id,
        import_id,
    }
}

/// The platform routed by the fixture source account and one seeded measure.
fn registry_ids(connection: &mut PgConnection) -> (Uuid, Uuid) {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        platform_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        measure_id: Uuid,
    }
    sql_query(
        "SELECT (SELECT platform_id FROM metric_platform ORDER BY code LIMIT 1) AS platform_id, \
                (SELECT measure_id FROM metric_measure WHERE code = 'title_sessions') AS measure_id",
    )
    .get_result::<Row>(connection)
    .map(|row| (row.platform_id, row.measure_id))
    .expect("Failed to read the fixture registry ids")
}

/// Insert one minimal canonical record through raw SQL so the database
/// defaults are exercised rather than restated by a Diesel fixture.
pub(crate) fn insert_record_row(
    pool: &PgPool,
    fixture: &RecordFixture,
    record_id: Option<Uuid>,
    identity_hash: &str,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    match record_id {
        Some(record_id) => sql_query(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, platform_id, measure_id, \
                  period_start, period_end, reporting_grain, winning_source_account_id) \
             VALUES ($1, $2, $3, $4, $5, DATE '2026-07-01', DATE '2026-08-01', 'MONTH', $6)",
        )
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .bind::<diesel::sql_types::Text, _>(identity_hash)
        .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .execute(&mut connection),
        None => sql_query(
            "INSERT INTO metric_record \
                 (identity_hash, work_id, platform_id, measure_id, \
                  period_start, period_end, reporting_grain, winning_source_account_id) \
             VALUES ($1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', 'MONTH', $5)",
        )
        .bind::<diesel::sql_types::Text, _>(identity_hash)
        .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .execute(&mut connection),
    }
}

/// Insert one canonical record overriding only the country representation.
fn insert_record_with_country(
    pool: &PgPool,
    fixture: &RecordFixture,
    identity_hash: &str,
    country_code: Option<&str>,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_record \
             (identity_hash, work_id, platform_id, measure_id, period_start, period_end, \
              reporting_grain, country_code, winning_source_account_id) \
         VALUES ($1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', 'MONTH', $5, $6)",
    )
    .bind::<diesel::sql_types::Text, _>(identity_hash)
    .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(country_code)
    .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
    .execute(&mut connection)
}

/// Insert one canonical record overriding only the reporting period.
fn insert_record_with_period(
    pool: &PgPool,
    fixture: &RecordFixture,
    identity_hash: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_record \
             (identity_hash, work_id, platform_id, measure_id, period_start, period_end, \
              reporting_grain, winning_source_account_id) \
         VALUES ($1, $2, $3, $4, $5, $6, 'MONTH', $7)",
    )
    .bind::<diesel::sql_types::Text, _>(identity_hash)
    .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
    .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
    .bind::<diesel::sql_types::Date, _>(period_start)
    .bind::<diesel::sql_types::Date, _>(period_end)
    .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
    .execute(&mut connection)
}

pub(crate) fn delete_row(
    pool: &PgPool,
    table: &str,
    id_column: &str,
    id: Uuid,
) -> Result<usize, DieselError> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!("DELETE FROM {table} WHERE {id_column} = $1"))
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut connection)
}

/// The sorted names of one table's foreign-key constraints, with definitions.
pub(crate) fn foreign_keys(pool: &PgPool, table: &str) -> Vec<(String, String)> {
    #[derive(diesel::QueryableByName)]
    struct ForeignKey {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = $1::regclass AND c.contype = 'f' \
         ORDER BY c.conname",
    )
    .bind::<diesel::sql_types::Text, _>(format!("public.{table}"))
    .load::<ForeignKey>(&mut connection)
    .expect("Failed to read foreign keys")
    .into_iter()
    .map(|key| (key.conname, key.definition))
    .collect()
}

/// The sorted index names of one table.
pub(crate) fn index_names(pool: &PgPool, table: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        indexname: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT indexname::text AS indexname FROM pg_indexes \
         WHERE schemaname = 'public' AND tablename = $1 ORDER BY indexname",
    )
    .bind::<diesel::sql_types::Text, _>(table)
    .load::<Row>(&mut connection)
    .expect("Failed to read index names")
    .into_iter()
    .map(|row| row.indexname)
    .collect()
}

/// The definition of one named index.
pub(crate) fn index_definition(pool: &PgPool, table: &str, index: &str) -> String {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        indexdef: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT indexdef FROM pg_indexes \
         WHERE schemaname = 'public' AND tablename = $1 AND indexname = $2",
    )
    .bind::<diesel::sql_types::Text, _>(table)
    .bind::<diesel::sql_types::Text, _>(index)
    .get_result::<Row>(&mut connection)
    .unwrap_or_else(|error| panic!("index {index} on {table} must exist: {error:?}"))
    .indexdef
}

#[test]
fn migration_seeds_no_record_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        0,
        "MET-WP1-04 must not seed any metric_record row"
    );
}

#[test]
fn metric_record_rows_map_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");

    let record_id: Uuid = diesel::insert_into(metric_record::table)
        .values((
            metric_record::identity_hash.eq("identity-a"),
            metric_record::work_id.eq(fixture.work_id),
            metric_record::publication_id.eq(fixture.publication_id),
            metric_record::platform_id.eq(fixture.platform_id),
            metric_record::measure_id.eq(fixture.measure_id),
            metric_record::period_start.eq(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
            metric_record::period_end.eq(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
            metric_record::reporting_grain.eq(MetricReportingGrain::Month),
            metric_record::country_code.eq("GB"),
            metric_record::institution_id.eq(fixture.institution_id),
            metric_record::winning_source_account_id.eq(fixture.source_account_id),
        ))
        .returning(metric_record::record_id)
        .get_result(&mut connection)
        .expect("Failed to insert the fully populated record row");

    let loaded: MetricRecord = metric_record::table
        .filter(metric_record::record_id.eq(record_id))
        .first(&mut connection)
        .expect("Failed to load the fully populated record row");
    assert_eq!(loaded.record_id, record_id);
    assert_eq!(loaded.identity_hash, "identity-a");
    assert_eq!(loaded.work_id, fixture.work_id);
    assert_eq!(loaded.publication_id, Some(fixture.publication_id));
    assert_eq!(loaded.platform_id, fixture.platform_id);
    assert_eq!(loaded.measure_id, fixture.measure_id);
    assert_eq!(
        loaded.period_start,
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
    );
    assert_eq!(
        loaded.period_end,
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()
    );
    assert_eq!(loaded.reporting_grain, MetricReportingGrain::Month);
    assert_eq!(loaded.country_code.as_deref(), Some("GB"));
    assert_eq!(loaded.institution_id, Some(fixture.institution_id));
    assert_eq!(loaded.winning_source_account_id, fixture.source_account_id);
    assert_eq!(
        loaded.current_revision_id, None,
        "a record is created before its first revision exists"
    );
}

#[test]
fn every_reporting_grain_is_reused_from_the_existing_registry_enum() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");
    // MET-WP1-04 must reuse MET-WP1-01's metric_reporting_grain rather than
    // duplicate it, so every existing grain must remain storable here.
    for (index, grain) in [
        MetricReportingGrain::Day,
        MetricReportingGrain::Month,
        MetricReportingGrain::ReportingPeriod,
    ]
    .into_iter()
    .enumerate()
    {
        let record_id: Uuid = diesel::insert_into(metric_record::table)
            .values((
                metric_record::identity_hash.eq(format!("identity-grain-{index}")),
                metric_record::work_id.eq(fixture.work_id),
                metric_record::platform_id.eq(fixture.platform_id),
                metric_record::measure_id.eq(fixture.measure_id),
                metric_record::period_start.eq(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
                metric_record::period_end.eq(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
                metric_record::reporting_grain.eq(grain),
                metric_record::winning_source_account_id.eq(fixture.source_account_id),
            ))
            .returning(metric_record::record_id)
            .get_result(&mut connection)
            .unwrap_or_else(|error| panic!("{grain:?} must be storable: {error:?}"));
        let loaded: MetricRecord = metric_record::table
            .filter(metric_record::record_id.eq(record_id))
            .first(&mut connection)
            .expect("Failed to reload the record row");
        assert_eq!(loaded.reporting_grain, grain);
    }
    // typtype = 'e' filters out the array type PostgreSQL auto-creates
    // alongside every enum, which MET-WP1-01 already established.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_type WHERE typnamespace = 'public'::regnamespace \
              AND typtype = 'e' AND typname LIKE '%reporting_grain%')",
        ),
        1,
        "MET-WP1-04 must not create a duplicate reporting-grain enum"
    );
}

#[test]
fn record_database_defaults_are_applied_without_explicit_values() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    insert_record_row(&pool, &fixture, None, "identity-defaults")
        .expect("Failed to insert the defaulted record row");

    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRecord = metric_record::table
        .first(&mut connection)
        .expect("Failed to load the defaulted record row");
    assert_ne!(
        loaded.record_id,
        Uuid::nil(),
        "the repository-standard UUID default must generate a record_id"
    );
    assert_eq!(loaded.publication_id, None);
    assert_eq!(loaded.country_code, None);
    assert_eq!(loaded.institution_id, None);
    assert_eq!(
        loaded.current_revision_id, None,
        "current_revision_id must stay unset until a WP2 transaction sets it"
    );
    assert!(
        loaded.first_received_at > Timestamp::default(),
        "the repository-standard current-time default must populate first_received_at"
    );
    assert!(
        loaded.updated_at > Timestamp::default(),
        "the repository-standard current-time default must populate updated_at"
    );
}

#[test]
fn blank_record_identity_hash_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    for blank in ["", " ", "   ", "\t", "\n"] {
        let result = insert_record_row(&pool, &fixture, None, blank);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "a blank identity hash ({blank:?}) must be rejected by a check constraint, \
             got {result:?}"
        );
    }
}

#[test]
fn duplicate_record_identity_hash_is_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    insert_record_row(&pool, &fixture, None, "identity-a")
        .expect("the first canonical record must be accepted");
    let result = insert_record_row(&pool, &fixture, None, "identity-a");
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        ),
        "a repeated identity hash must be rejected, got {result:?}"
    );
}

#[test]
fn the_identity_hash_carries_no_algorithm_encoding_or_length_rule() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    // This slice deliberately fixes no hash algorithm. Any non-blank value
    // must remain storable so WP2 can choose the algorithm later.
    for (index, hash) in [
        "a",
        "0123456789abcdef",
        "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        "a-very-long-identity-hash-representation-that-no-schema-rule-bounds",
    ]
    .into_iter()
    .enumerate()
    {
        insert_record_row(&pool, &fixture, None, hash)
            .unwrap_or_else(|error| panic!("identity hash {index} must be storable: {error:?}"));
    }
}

#[test]
fn a_null_country_representation_is_accepted() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    insert_record_with_country(&pool, &fixture, "identity-null-country", None)
        .expect("country is an optional dimension and NULL must be accepted");
    let mut connection = pool.get().expect("Failed to get DB connection");
    let loaded: MetricRecord = metric_record::table
        .first(&mut connection)
        .expect("Failed to load the record row");
    assert_eq!(loaded.country_code, None);
}

#[test]
fn exactly_two_uppercase_ascii_letters_are_accepted_as_a_country() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    for (index, country) in ["GB", "US", "DE", "ZW", "AA"].into_iter().enumerate() {
        insert_record_with_country(
            &pool,
            &fixture,
            &format!("identity-ok-{index}"),
            Some(country),
        )
        .unwrap_or_else(|error| panic!("{country} must be accepted: {error:?}"));
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let stored: Vec<Option<String>> = metric_record::table
        .order(metric_record::identity_hash)
        .select(metric_record::country_code)
        .load(&mut connection)
        .expect("Failed to load the stored country representations");
    assert_eq!(
        stored,
        vec![
            Some("GB".to_string()),
            Some("US".to_string()),
            Some("DE".to_string()),
            Some("ZW".to_string()),
            Some("AA".to_string()),
        ],
        "an accepted alpha-2 representation must round-trip exactly"
    );
    // Shape only: this foundation deliberately performs no ISO 3166-1 alpha-2
    // membership validation, which is why the unassigned code AA is stored.
    // Semantic membership belongs to later WP2 normalized-observation
    // validation.
}

#[test]
fn invalid_country_representations_are_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    // Wrong case, non-letters, and anything that is not exactly two
    // characters. A one-character value is blank-padded by CHAR(2) and still
    // fails the two-uppercase-letter shape.
    for (index, country) in ["gb", "Gb", "gB", "G1", "12", "G-", "G", "", " ", "  ", " G"]
        .into_iter()
        .enumerate()
    {
        let result = insert_record_with_country(
            &pool,
            &fixture,
            &format!("identity-bad-{index}"),
            Some(country),
        );
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "country {country:?} must be rejected by a check constraint, got {result:?}"
        );
    }
    // A value longer than two characters cannot even be stored in CHAR(2).
    let too_long = insert_record_with_country(&pool, &fixture, "identity-bad-long", Some("GBR"));
    assert!(
        too_long.is_err(),
        "a three-character country representation must be rejected, got {too_long:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        0,
        "no invalid country representation may be stored"
    );
}

#[test]
fn the_metrics_country_representation_does_not_reuse_the_bibliographic_alpha_3_enum() {
    let (_guard, pool) = setup_registry_db();
    // The existing bibliographic country_code enum must be neither modified
    // nor reused: metric_record.country_code is a separate CHAR(2) column.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_enum e JOIN pg_type t ON t.oid = e.enumtypid \
              WHERE t.typname = 'country_code')",
        ),
        249,
        "the existing alpha-3 country_code enum must keep all its labels"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_enum e JOIN pg_type t ON t.oid = e.enumtypid \
              WHERE t.typname = 'country_code' AND length(e.enumlabel::text) <> 3)",
        ),
        0,
        "the existing country_code domain must stay three-letter"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM information_schema.columns \
              WHERE table_schema = 'public' AND table_name = 'metric_record' \
                AND column_name = 'country_code' \
                AND data_type = 'character' AND character_maximum_length = 2)",
        ),
        1,
        "metric_record.country_code must be a separate CHAR(2) representation"
    );
}

#[test]
fn record_period_ordering_is_enforced() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    let july = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let august = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

    insert_record_with_period(&pool, &fixture, "identity-ordered", july, august)
        .expect("a correctly ordered half-open period must be accepted");

    for (label, start, end) in [("inverted", august, july), ("empty", july, july)] {
        let result =
            insert_record_with_period(&pool, &fixture, &format!("identity-{label}"), start, end);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::CheckViolation,
                    _
                ))
            ),
            "an {label} period must be rejected because period_end > period_start, got {result:?}"
        );
    }
}

#[test]
fn overlapping_periods_are_deliberately_not_rejected() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    // Period-overlap enforcement is explicitly deferred to the WP2 ingestion
    // transaction slice. This foundation must therefore accept two records
    // whose periods overlap, and must not have silently chosen an exclusion
    // constraint, btree_gist or an advisory-lock protocol.
    insert_record_with_period(
        &pool,
        &fixture,
        "identity-overlap-a",
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
    )
    .expect("the first record must be accepted");
    insert_record_with_period(
        &pool,
        &fixture,
        "identity-overlap-b",
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
    )
    .expect("overlap detection belongs to WP2 and must not be enforced here");
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_constraint \
              WHERE contype = 'x' AND conrelid::regclass::text LIKE 'metric_record%')",
        ),
        0,
        "MET-WP1-04 must introduce no exclusion constraint"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_extension WHERE extname = 'btree_gist')",
        ),
        0,
        "MET-WP1-04 must not install btree_gist"
    );
}

#[test]
fn invalid_record_foreign_keys_fail_closed() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    let mut connection = pool.get().expect("Failed to get DB connection");
    let unknown = Uuid::new_v4();

    for (label, work_id, publication_id, platform_id, measure_id, institution_id, account_id) in [
        (
            "work",
            unknown,
            None,
            fixture.platform_id,
            fixture.measure_id,
            None,
            fixture.source_account_id,
        ),
        (
            "publication",
            fixture.work_id,
            Some(unknown),
            fixture.platform_id,
            fixture.measure_id,
            None,
            fixture.source_account_id,
        ),
        (
            "platform",
            fixture.work_id,
            None,
            unknown,
            fixture.measure_id,
            None,
            fixture.source_account_id,
        ),
        (
            "measure",
            fixture.work_id,
            None,
            fixture.platform_id,
            unknown,
            None,
            fixture.source_account_id,
        ),
        (
            "institution",
            fixture.work_id,
            None,
            fixture.platform_id,
            fixture.measure_id,
            Some(unknown),
            fixture.source_account_id,
        ),
        (
            "winning source account",
            fixture.work_id,
            None,
            fixture.platform_id,
            fixture.measure_id,
            None,
            unknown,
        ),
    ] {
        let result = sql_query(
            "INSERT INTO metric_record \
                 (identity_hash, work_id, publication_id, platform_id, measure_id, \
                  period_start, period_end, reporting_grain, institution_id, \
                  winning_source_account_id) \
             VALUES ($1, $2, $3, $4, $5, DATE '2026-07-01', DATE '2026-08-01', 'MONTH', $6, $7)",
        )
        .bind::<diesel::sql_types::Text, _>(format!("identity-fk-{label}"))
        .bind::<diesel::sql_types::Uuid, _>(work_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(publication_id)
        .bind::<diesel::sql_types::Uuid, _>(platform_id)
        .bind::<diesel::sql_types::Uuid, _>(measure_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(institution_id)
        .bind::<diesel::sql_types::Uuid, _>(account_id)
        .execute(&mut connection);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "an unknown {label} must be rejected, got {result:?}"
        );
    }
}

#[test]
fn deleting_a_referenced_canonical_entity_is_restricted() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(
            "INSERT INTO metric_record \
                 (identity_hash, work_id, publication_id, platform_id, measure_id, \
                  period_start, period_end, reporting_grain, institution_id, \
                  winning_source_account_id) \
             VALUES ('identity-a', $1, $2, $3, $4, DATE '2026-07-01', DATE '2026-08-01', \
                     'MONTH', $5, $6)",
        )
        .bind::<diesel::sql_types::Uuid, _>(fixture.work_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.publication_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.platform_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.measure_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.institution_id)
        .bind::<diesel::sql_types::Uuid, _>(fixture.source_account_id)
        .execute(&mut connection)
        .expect("Failed to insert the referencing record row");
    }

    for (table, id_column, id) in [
        ("publication", "publication_id", fixture.publication_id),
        ("work", "work_id", fixture.work_id),
        ("metric_platform", "platform_id", fixture.platform_id),
        ("metric_measure", "measure_id", fixture.measure_id),
        ("institution", "institution_id", fixture.institution_id),
        (
            "metric_source_account",
            "source_account_id",
            fixture.source_account_id,
        ),
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
             canonical history, got {result:?}"
        );
    }
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        1,
        "the canonical record must survive the restricted deletions"
    );
}

#[test]
fn metric_record_has_exactly_the_authorized_check_constraints() {
    let (_guard, pool) = setup_registry_db();
    // The set is closed: no publisher/imprint/series attribution rule, no
    // publication-belongs-to-work rule, no institution-has-ROR rule and no
    // overlap rule may exist.
    assert_eq!(
        crate::model::metric_import::tests::check_constraint_names(&pool, "metric_record"),
        vec![
            "metric_record_country_code_check",
            "metric_record_identity_hash_check",
            "metric_record_period_check",
        ],
        "metric_record must carry exactly the three authorized CHECK constraints"
    );
}

#[test]
fn metric_record_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_record");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_record_current_revision_id_fkey",
            "metric_record_institution_id_fkey",
            "metric_record_measure_id_fkey",
            "metric_record_platform_id_fkey",
            "metric_record_publication_id_fkey",
            "metric_record_winning_source_account_id_fkey",
            "metric_record_work_id_fkey",
        ],
        "metric_record must carry exactly the seven authorized foreign keys"
    );
    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour: {definition}"
        );
    }
    assert!(
        keys.iter().any(
            |(name, definition)| name == "metric_record_current_revision_id_fkey"
                && definition.contains("(record_id, current_revision_id)")
                && definition.contains("metric_record_revision(record_id, record_revision_id)")
        ),
        "the current-revision key must be the composite same-record shape: {keys:?}"
    );
}

#[test]
fn metric_record_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_record"),
        vec![
            "metric_record_identity_hash_key",
            "metric_record_measure_id_idx",
            "metric_record_period_start_idx",
            "metric_record_pkey",
            "metric_record_platform_id_idx",
            "metric_record_work_id_idx",
        ],
        "metric_record must carry exactly its primary key, the unique identity \
         hash and the four design-required access indexes, with no speculative \
         dashboard composite"
    );
    assert!(
        index_definition(&pool, "metric_record", "metric_record_identity_hash_key")
            .contains("UNIQUE"),
        "the identity-hash index must be unique"
    );
    for (index, column) in [
        ("metric_record_work_id_idx", "work_id"),
        ("metric_record_platform_id_idx", "platform_id"),
        ("metric_record_measure_id_idx", "measure_id"),
        ("metric_record_period_start_idx", "period_start"),
    ] {
        let definition = index_definition(&pool, "metric_record", index);
        assert!(
            definition.contains(&format!("({column})")),
            "{index} must begin with {column}: {definition}"
        );
    }
}

#[test]
fn no_hashing_or_arbitration_behaviour_is_implemented_by_this_slice() {
    let (_guard, pool) = setup_registry_db();
    let fixture = insert_record_fixture(&pool);
    // The identity hash is supplied by the caller and is never derived here:
    // two records whose canonical dimensions are identical but whose supplied
    // identity hashes differ are both accepted, because no schema rule
    // computes identity from the dimensions. First-arrival arbitration,
    // duplicate detection and conflict resolution are WP2 behaviour.
    insert_record_row(&pool, &fixture, None, "identity-x")
        .expect("the first record must be accepted");
    insert_record_row(&pool, &fixture, None, "identity-y")
        .expect("this slice implements no identity derivation or arbitration");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        2,
        "no first-arrival or duplicate-detection algorithm exists in this slice"
    );
}

#[test]
fn metric_record_carries_only_the_repository_standard_updated_at_trigger() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        tgname: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let triggers: Vec<String> = sql_query(
        "SELECT t.tgname::text AS tgname FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
         WHERE NOT t.tgisinternal AND c.relname LIKE 'metric_record%' ORDER BY c.relname, t.tgname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read triggers")
    .into_iter()
    .map(|row| row.tgname)
    .collect();
    // The only permitted trigger is the repository-standard
    // diesel_manage_updated_at helper that every other Metrics table with an
    // updated_at column already uses. No same-record integrity trigger and no
    // revision state machine may exist: same-record integrity is carried by
    // composite foreign keys, and WP2 owns revision transitions.
    assert_eq!(
        triggers,
        vec!["set_updated_at"],
        "metric_record* tables must carry only the repository-standard \
         updated-at trigger"
    );
}
