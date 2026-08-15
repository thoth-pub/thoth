//! `BE-04` database-contract, creation, switch, state-machine, concurrency and
//! cancellation evidence.
//!
//! Every database test here runs against a real disposable PostgreSQL with the
//! migration applied, so the database's own constraints are exercised rather
//! than assumed, and every concurrency test uses **multiple real connections and
//! real transactions**. There are no mocked sequential substitutes.
//!
//! GraphQL-level authorization, error-mapping, SDL, statement-order and
//! statement-count evidence lives in `crate::graphql::distribution_job_tests`.
//!
//! Nothing here performs dissemination, contacts a distribution platform or
//! produces a file, feed or deposit.

use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};

use diesel::connection::SimpleConnection;
use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
use diesel::{
    sql_query, Connection, ExpressionMethods, PgConnection, QueryDsl, QueryableByName, RunQueryDsl,
};
use diesel_migrations::MigrationHarness;

use super::crud::{
    attempts_for_jobs, cancel_distribution_job, claim_distribution_jobs, complete_distribution_job,
    fail_distribution_job, latest_back_catalogue_jobs, sanitize_error_detail, targets_for_jobs,
    validate_error_code,
};
use super::*;
use crate::db::{PgPool, MIGRATIONS};
use crate::model::publisher::ThothPackage;
use crate::model::publisher_distribution_platform::{
    DistributionPlatform, PublisherDistributionPlatform,
};
use crate::model::publisher_service_configuration::crud::replace_publisher_service_configuration;
use crate::model::publisher_service_configuration::{
    PublisherServiceConfigurationSource, ReplacePublisherServiceConfigurationInput,
    ServiceConfigurationWriteContext,
};
use crate::model::tests::db as test_db;
use crate::model::{Crud, Timestamp};
use thoth_errors::ThothError;

const WORKER: &str = "zitadel-dissemination-worker-1";
const ACTOR: &str = "zitadel-superuser-1";

// --------------------------------------------------------------------------
// Catalog and row helpers
// --------------------------------------------------------------------------

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

fn catalog_values(pool: &PgPool, query: &str) -> Vec<String> {
    let mut connection = pool.get().expect("connection");
    sql_query(query)
        .load::<TextRow>(&mut connection)
        .expect("catalog query")
        .into_iter()
        .map(|row| row.value)
        .collect()
}

fn scalar_count(pool: &PgPool, query: &str) -> i64 {
    let mut connection = pool.get().expect("connection");
    sql_query(query)
        .get_result::<CountRow>(&mut connection)
        .expect("count query")
        .count
}

fn write_context(
    job_creation: DistributionJobCreation,
) -> ServiceConfigurationWriteContext<'static> {
    ServiceConfigurationWriteContext {
        source: PublisherServiceConfigurationSource::SuperuserApi,
        actor: ACTOR,
        job_creation,
    }
}

fn backfill_context(
    job_creation: DistributionJobCreation,
) -> ServiceConfigurationWriteContext<'static> {
    ServiceConfigurationWriteContext {
        source: PublisherServiceConfigurationSource::MigrationBackfill,
        actor: "authorized-control-identity",
        job_creation,
    }
}

fn token(pool: &PgPool, publisher_id: Uuid) -> Timestamp {
    crate::model::publisher::Publisher::from_id(pool, &publisher_id)
        .expect("publisher")
        .service_configuration_updated_at
}

/// Commit one configuration replacement through the canonical coordinator.
fn replace(
    pool: &PgPool,
    context: &ServiceConfigurationWriteContext<'_>,
    publisher_id: Uuid,
    package: ThothPackage,
    platforms: &[DistributionPlatform],
) -> ThothResultAlias {
    replace_publisher_service_configuration(
        pool,
        context,
        &ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: package,
            enabled_distribution_platforms: platforms.to_vec(),
            expected_updated_at: token(pool, publisher_id),
        },
    )
    .map(|_| ())
}

type ThothResultAlias = Result<(), ThothError>;

/// Commit a replacement with automatic creation `ON`, which is the ordinary
/// fixture path for tests that need a job to exist.
fn activate(pool: &PgPool, publisher_id: Uuid, platforms: &[DistributionPlatform]) {
    replace(
        pool,
        &write_context(DistributionJobCreation::On),
        publisher_id,
        ThothPackage::Sphinx,
        platforms,
    )
    .expect("activation should commit");
}

fn jobs_of(pool: &PgPool, publisher_id: Uuid) -> Vec<DistributionJob> {
    let mut connection = pool.get().expect("connection");
    crate::schema::distribution_job::table
        .filter(crate::schema::distribution_job::publisher_id.eq(publisher_id))
        .order((
            crate::schema::distribution_job::created_at.asc(),
            crate::schema::distribution_job::distribution_job_id.asc(),
        ))
        .load::<DistributionJob>(&mut connection)
        .expect("jobs")
}

fn only_job(pool: &PgPool, publisher_id: Uuid) -> DistributionJob {
    let mut jobs = jobs_of(pool, publisher_id);
    assert_eq!(jobs.len(), 1, "expected exactly one job");
    jobs.remove(0)
}

fn reload(pool: &PgPool, job_id: Uuid) -> DistributionJob {
    let mut connection = pool.get().expect("connection");
    crate::schema::distribution_job::table
        .filter(crate::schema::distribution_job::distribution_job_id.eq(job_id))
        .first::<DistributionJob>(&mut connection)
        .expect("job")
}

fn targets_of(pool: &PgPool, job_id: Uuid) -> Vec<DistributionPlatform> {
    let mut connection = pool.get().expect("connection");
    targets_for_jobs(&mut connection, &[job_id])
        .expect("targets")
        .into_iter()
        .map(|target| target.platform)
        .collect()
}

fn attempts_of(pool: &PgPool, job_id: Uuid) -> Vec<DistributionJobAttempt> {
    let mut connection = pool.get().expect("connection");
    attempts_for_jobs(&mut connection, &[job_id]).expect("attempts")
}

/// Claim exactly one job as the standard test worker.
fn claim_one(pool: &PgPool) -> ClaimedDistributionJob {
    let mut claimed = claim_distribution_jobs(pool, WORKER, 1, 900, &[]).expect("claim");
    assert_eq!(claimed.len(), 1, "expected exactly one claim");
    claimed.remove(0)
}

/// Force a running job's lease to have expired, without touching anything else.
fn expire_lease(pool: &PgPool, job_id: Uuid) {
    let mut connection = pool.get().expect("connection");
    sql_query(
        "UPDATE distribution_job SET lease_expires_at = CURRENT_TIMESTAMP - interval '1 second' \
         WHERE distribution_job_id = $1",
    )
    .bind::<SqlUuid, _>(job_id)
    .execute(&mut connection)
    .expect("expire lease");
}

/// Make a job due now, regardless of any backoff it carries.
fn make_due(pool: &PgPool, job_id: Uuid) {
    let mut connection = pool.get().expect("connection");
    sql_query(
        "UPDATE distribution_job SET available_at = CURRENT_TIMESTAMP - interval '1 second' \
         WHERE distribution_job_id = $1",
    )
    .bind::<SqlUuid, _>(job_id)
    .execute(&mut connection)
    .expect("make due");
}

/// Run lease recovery **without** letting the recovered job be re-claimed by the
/// same call.
///
/// Recovery is step A of the claim and selection is step B, both inside one
/// transaction, so an ordinary claim call recovers *and* immediately re-claims.
/// Temporarily making the job's target ineligible lets step A be observed in
/// isolation, which is what separates `T5a`'s "attempt_count unchanged" from the
/// increment the very next claim performs.
fn recover_without_reclaim(pool: &PgPool, publisher_id: Uuid) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET enabled = false, disabled_at = now() \
         WHERE publisher_id = '{publisher_id}' AND enabled"
    ))
    .execute(&mut connection)
    .expect("suspend eligibility");
    drop(connection);

    let claimed = claim_distribution_jobs(pool, WORKER, 10, 900, &[]).expect("recovery");
    assert!(
        claimed.is_empty(),
        "selection must find nothing while suspended"
    );

    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET enabled = true, disabled_at = NULL \
         WHERE publisher_id = '{publisher_id}' AND NOT enabled"
    ))
    .execute(&mut connection)
    .expect("restore eligibility");
}

/// A publisher with one activated `ZENODO` job, ready to claim.
fn publisher_with_pending_job(pool: &PgPool) -> (Uuid, Uuid) {
    let publisher = test_db::create_publisher(pool);
    activate(
        pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let job = only_job(pool, publisher.publisher_id);
    (publisher.publisher_id, job.distribution_job_id)
}

// ==========================================================================
// 25.2  Enum and schema-contract tests
// ==========================================================================

#[test]
fn every_enum_label_exists_in_pg_enum_with_the_exact_spelling_and_order() {
    let (_guard, pool) = test_db::setup_test_db();

    let labels = |type_name: &str| {
        catalog_values(
            &pool,
            &format!(
                "SELECT e.enumlabel AS value FROM pg_type t \
                 JOIN pg_enum e ON e.enumtypid = t.oid \
                 WHERE t.typname = '{type_name}' ORDER BY e.enumsortorder"
            ),
        )
    };

    assert_eq!(
        labels("distribution_job_kind"),
        vec!["PUBLISHER_BACK_CATALOGUE"]
    );
    assert_eq!(
        labels("distribution_job_status"),
        vec!["PENDING", "RUNNING", "SUCCEEDED", "FAILED", "CANCELLED"]
    );
    assert_eq!(
        labels("distribution_job_attempt_result"),
        vec!["SUCCEEDED", "FAILED", "CANCELLED", "ABANDONED"]
    );
    assert_eq!(
        labels("distribution_job_cancellation_reason"),
        vec!["ADMINISTRATIVE", "ASSIGNMENT_DISABLED"]
    );
}

#[test]
fn no_enum_carries_a_catch_all_or_unknown_label() {
    let (_guard, pool) = test_db::setup_test_db();
    let suspicious = catalog_values(
        &pool,
        "SELECT e.enumlabel AS value FROM pg_type t \
         JOIN pg_enum e ON e.enumtypid = t.oid \
         WHERE t.typname LIKE 'distribution_job%' \
           AND e.enumlabel IN ('OTHER', 'UNKNOWN', 'NONE', 'NOT_STARTED', \
                               'NOT_APPLICABLE', 'DEFAULT')",
    );
    assert!(
        suspicious.is_empty(),
        "a catch-all label would let an unrecognised value resolve to a nearest one: {suspicious:?}"
    );
}

#[test]
fn every_rust_enum_round_trips_through_the_database() {
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};
    use crate::schema::sql_types;

    let (_guard, pool) = test_db::setup_test_db();

    // `distribution_job_kind` carries exactly one value today.
    assert_db_enum_roundtrip::<DistributionJobKind, sql_types::DistributionJobKind>(
        &pool,
        "'PUBLISHER_BACK_CATALOGUE'::distribution_job_kind",
        DistributionJobKind::PublisherBackCatalogue,
    );
    assert_graphql_enum_roundtrip(DistributionJobKind::PublisherBackCatalogue);

    for (literal, expected) in [
        (
            "'PENDING'::distribution_job_status",
            DistributionJobStatus::Pending,
        ),
        (
            "'RUNNING'::distribution_job_status",
            DistributionJobStatus::Running,
        ),
        (
            "'SUCCEEDED'::distribution_job_status",
            DistributionJobStatus::Succeeded,
        ),
        (
            "'FAILED'::distribution_job_status",
            DistributionJobStatus::Failed,
        ),
        (
            "'CANCELLED'::distribution_job_status",
            DistributionJobStatus::Cancelled,
        ),
    ] {
        assert_db_enum_roundtrip::<DistributionJobStatus, sql_types::DistributionJobStatus>(
            &pool, literal, expected,
        );
        assert_graphql_enum_roundtrip(expected);
    }

    for (literal, expected) in [
        (
            "'SUCCEEDED'::distribution_job_attempt_result",
            DistributionJobAttemptResult::Succeeded,
        ),
        (
            "'FAILED'::distribution_job_attempt_result",
            DistributionJobAttemptResult::Failed,
        ),
        (
            "'CANCELLED'::distribution_job_attempt_result",
            DistributionJobAttemptResult::Cancelled,
        ),
        (
            "'ABANDONED'::distribution_job_attempt_result",
            DistributionJobAttemptResult::Abandoned,
        ),
    ] {
        assert_db_enum_roundtrip::<
            DistributionJobAttemptResult,
            sql_types::DistributionJobAttemptResult,
        >(&pool, literal, expected);
        assert_graphql_enum_roundtrip(expected);
    }

    for (literal, expected) in [
        (
            "'ADMINISTRATIVE'::distribution_job_cancellation_reason",
            DistributionJobCancellationReason::Administrative,
        ),
        (
            "'ASSIGNMENT_DISABLED'::distribution_job_cancellation_reason",
            DistributionJobCancellationReason::AssignmentDisabled,
        ),
    ] {
        assert_db_enum_roundtrip::<
            DistributionJobCancellationReason,
            sql_types::DistributionJobCancellationReason,
        >(&pool, literal, expected);
        assert_graphql_enum_roundtrip(expected);
    }
}

#[test]
fn an_unrecognised_value_fails_rather_than_resolving_to_a_nearest_one() {
    use std::str::FromStr;

    // String / serde.
    assert!(DistributionJobKind::from_str("WORK_UPSERT").is_err());
    assert!(DistributionJobStatus::from_str("UNKNOWN").is_err());
    assert!(DistributionJobStatus::from_str("pending").is_err());
    assert!(DistributionJobAttemptResult::from_str("TIMED_OUT").is_err());
    assert!(DistributionJobCancellationReason::from_str("OTHER").is_err());

    assert!(serde_json::from_str::<DistributionJobStatus>("\"UNKNOWN\"").is_err());
    assert!(serde_json::from_str::<DistributionJobKind>("\"OTHER\"").is_err());
    assert!(serde_json::from_str::<DistributionJobAttemptResult>("\"OTHER\"").is_err());
    assert!(serde_json::from_str::<DistributionJobCancellationReason>("\"UNKNOWN\"").is_err());

    // GraphQL.
    use juniper::{DefaultScalarValue, FromInputValue, InputValue};
    let bogus: InputValue<DefaultScalarValue> = InputValue::scalar("UNKNOWN");
    assert!(DistributionJobStatus::from_input_value(&bogus).is_err());
    assert!(DistributionJobKind::from_input_value(&bogus).is_err());
    assert!(DistributionJobAttemptResult::from_input_value(&bogus).is_err());
    assert!(DistributionJobCancellationReason::from_input_value(&bogus).is_err());

    // The database itself.
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    assert!(
        sql_query("SELECT 'UNKNOWN'::distribution_job_status")
            .execute(&mut connection)
            .is_err(),
        "PostgreSQL must refuse an unknown label"
    );
}

#[test]
fn schema_rs_matches_the_migration_for_all_three_relations() {
    let (_guard, pool) = test_db::setup_test_db();

    // Column name, type and nullability, in ordinal position — the order the
    // `Queryable` derives depend on.
    let columns = |table: &str| {
        catalog_values(
            &pool,
            &format!(
                "SELECT column_name || ' ' || udt_name || ' ' || is_nullable AS value \
                 FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = '{table}' \
                 ORDER BY ordinal_position"
            ),
        )
    };

    assert_eq!(
        columns("distribution_job"),
        vec![
            "distribution_job_id uuid NO",
            "kind distribution_job_kind NO",
            "publisher_id uuid NO",
            "work_id uuid YES",
            "activation_id uuid NO",
            "status distribution_job_status NO",
            "deduplication_key text NO",
            "attempt_count int4 NO",
            "available_at timestamptz NO",
            "claim_token uuid YES",
            "claimed_by text YES",
            "claimed_at timestamptz YES",
            "lease_expires_at timestamptz YES",
            "completed_at timestamptz YES",
            "cancellation_reason distribution_job_cancellation_reason YES",
            "last_error_code text YES",
            "last_error_detail text YES",
            "created_at timestamptz NO",
            "updated_at timestamptz NO",
        ]
    );
    assert_eq!(
        columns("distribution_job_target"),
        vec![
            "distribution_job_id uuid NO",
            "platform distribution_platform NO",
            "created_at timestamptz NO",
        ]
    );
    assert_eq!(
        columns("distribution_job_attempt"),
        vec![
            "distribution_job_attempt_id uuid NO",
            "distribution_job_id uuid NO",
            "attempt_number int4 NO",
            "claim_token uuid NO",
            "claimed_by text NO",
            "started_at timestamptz NO",
            "finished_at timestamptz YES",
            "result distribution_job_attempt_result YES",
            "error_code text YES",
            "error_detail text YES",
        ]
    );

    // The `schema.rs` text itself, so the joinable and same-query entries are
    // pinned rather than merely assumed to exist.
    let schema = include_str!("../../schema.rs");
    for entry in [
        "joinable!(distribution_job -> publisher (publisher_id));",
        "joinable!(distribution_job -> work (work_id));",
        "joinable!(distribution_job_attempt -> distribution_job (distribution_job_id));",
        "joinable!(distribution_job_target -> distribution_job (distribution_job_id));",
    ] {
        assert!(schema.contains(entry), "schema.rs must declare `{entry}`");
    }
    for table in [
        "    distribution_job,\n",
        "    distribution_job_attempt,\n",
        "    distribution_job_target,\n",
    ] {
        assert!(
            schema.contains(table),
            "schema.rs must list `{}` in allow_tables_to_appear_in_same_query!",
            table.trim()
        );
    }
    // ADR-0003 Architecture A: the schema contract is maintained by hand, so no
    // Diesel CLI configuration may appear.
    assert!(
        !std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../diesel.toml")).exists(),
        "no root diesel.toml may be introduced"
    );
}

#[test]
fn only_distribution_job_is_diesel_managed_and_the_indexes_are_exactly_the_specified_three() {
    let (_guard, pool) = test_db::setup_test_db();

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT tgname AS value FROM pg_trigger \
             WHERE tgrelid = 'public.distribution_job'::regclass AND NOT tgisinternal"
        ),
        vec!["set_updated_at"]
    );
    for append_only in ["distribution_job_target", "distribution_job_attempt"] {
        assert!(
            catalog_values(
                &pool,
                &format!(
                    "SELECT tgname AS value FROM pg_trigger \
                     WHERE tgrelid = 'public.{append_only}'::regclass AND NOT tgisinternal"
                )
            )
            .is_empty(),
            "{append_only} is append-only and has no updated_at to manage"
        );
    }

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT indexname AS value FROM pg_indexes \
             WHERE tablename LIKE 'distribution_job%' ORDER BY indexname"
        ),
        vec![
            // Constraint-backed indexes; no separate index is created for them.
            "distribution_job_attempt_claim_token_key",
            "distribution_job_attempt_number_key",
            "distribution_job_attempt_pkey",
            // The three explicit indexes.
            "distribution_job_claimable_idx",
            "distribution_job_deduplication_key_key",
            "distribution_job_lease_idx",
            "distribution_job_pkey",
            "distribution_job_publisher_latest_idx",
            "distribution_job_target_pkey",
        ]
    );
}

#[test]
fn every_named_constraint_of_sections_7_2_to_7_4_exists_in_the_catalog() {
    let (_guard, pool) = test_db::setup_test_db();

    let mut expected = vec![
        "distribution_job_attempt_claim_token_key",
        "distribution_job_attempt_claimed_by_check",
        "distribution_job_attempt_closure_check",
        "distribution_job_attempt_count_check",
        "distribution_job_attempt_distribution_job_id_fkey",
        "distribution_job_attempt_error_code_format_check",
        "distribution_job_attempt_error_detail_length_check",
        "distribution_job_attempt_error_pairing_check",
        "distribution_job_attempt_error_result_check",
        "distribution_job_attempt_interval_check",
        "distribution_job_attempt_number_check",
        "distribution_job_attempt_number_key",
        "distribution_job_attempt_pkey",
        "distribution_job_back_catalogue_work_check",
        "distribution_job_cancellation_reason_check",
        "distribution_job_claim_state_check",
        "distribution_job_claimed_by_check",
        "distribution_job_completed_at_check",
        "distribution_job_deduplication_key_formula_check",
        "distribution_job_deduplication_key_key",
        "distribution_job_deduplication_key_length_check",
        "distribution_job_last_error_check",
        "distribution_job_last_error_code_format_check",
        "distribution_job_last_error_detail_length_check",
        "distribution_job_pkey",
        "distribution_job_publisher_id_fkey",
        "distribution_job_target_distribution_job_id_fkey",
        "distribution_job_target_pkey",
        "distribution_job_work_id_fkey",
    ];
    expected.sort_unstable();

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT conname AS value FROM pg_constraint \
             WHERE conrelid::regclass::text LIKE 'distribution_job%' ORDER BY conname"
        ),
        expected
    );

    // The foreign keys are the specified ones and are not weakened.
    let cascades = catalog_values(
        &pool,
        "SELECT conname || ' ' || confdeltype::text AS value FROM pg_constraint \
         WHERE contype = 'f' AND conrelid::regclass::text LIKE 'distribution_job%' \
         ORDER BY conname",
    );
    assert_eq!(
        cascades,
        vec![
            "distribution_job_attempt_distribution_job_id_fkey c",
            "distribution_job_publisher_id_fkey c",
            "distribution_job_target_distribution_job_id_fkey c",
            "distribution_job_work_id_fkey c",
        ],
        "every foreign key must remain ON DELETE CASCADE and validated"
    );
    assert!(
        catalog_values(
            &pool,
            "SELECT conname AS value FROM pg_constraint \
             WHERE conrelid::regclass::text LIKE 'distribution_job%' AND NOT convalidated"
        )
        .is_empty(),
        "no constraint may be left NOT VALID"
    );
}

#[test]
fn the_attempt_budget_constant_and_the_migration_agree() {
    let (_guard, pool) = test_db::setup_test_db();

    assert_eq!(DISTRIBUTION_JOB_MAX_ATTEMPTS, 5);

    let definition = catalog_values(
        &pool,
        "SELECT pg_get_constraintdef(oid) AS value FROM pg_constraint \
         WHERE conname = 'distribution_job_attempt_count_check'",
    );
    assert_eq!(definition.len(), 1);
    assert!(
        definition[0].contains(&format!("<= {DISTRIBUTION_JOB_MAX_ATTEMPTS}")),
        "the database's upper bound must equal DISTRIBUTION_JOB_MAX_ATTEMPTS; \
         raising the budget is a migration *and* a constant change, reviewed \
         together. Observed: {}",
        definition[0]
    );
    assert!(definition[0].contains(">= 0"));

    // The character bounds are tied the same way.
    for (constant, constraint) in [
        (
            DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS,
            "distribution_job_last_error_code_format_check",
        ),
        (
            DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS,
            "distribution_job_last_error_detail_length_check",
        ),
    ] {
        let definition = catalog_values(
            &pool,
            &format!(
                "SELECT pg_get_constraintdef(oid) AS value FROM pg_constraint \
                 WHERE conname = '{constraint}'"
            ),
        );
        assert!(
            definition[0].contains(&constant.to_string()),
            "{constraint} must carry {constant}: {}",
            definition[0]
        );
    }
}

// ==========================================================================
// 25.4  Constraint and integrity tests
// ==========================================================================

/// Insert a job row directly, bypassing the domain function, so the database's
/// own refusal is what is being observed.
fn raw_insert_job(
    pool: &PgPool,
    columns: &str,
    values: &str,
) -> Result<usize, diesel::result::Error> {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO distribution_job ({columns}) VALUES ({values})"
    ))
    .execute(&mut connection)
}

fn key_for(publisher_id: Uuid, activation_id: Uuid) -> String {
    DistributionJob::back_catalogue_deduplication_key(publisher_id, activation_id)
}

#[test]
fn the_foreign_keys_refuse_orphans_and_cascade_on_publisher_deletion() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let absent = Uuid::new_v4();

    // A job cannot reference an absent publisher.
    assert!(raw_insert_job(
        &pool,
        "kind, publisher_id, activation_id, deduplication_key",
        &format!(
            "'PUBLISHER_BACK_CATALOGUE', '{absent}', '{absent}', '{}'",
            key_for(absent, absent)
        )
    )
    .is_err());

    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let job = only_job(&pool, publisher.publisher_id);
    let claimed = claim_one(&pool);
    assert_eq!(claimed.job.job.distribution_job_id, job.distribution_job_id);

    let mut connection = pool.get().expect("connection");
    // A target cannot reference an absent job.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_target (distribution_job_id, platform) \
         VALUES ('{absent}', 'ZENODO')"
    ))
    .execute(&mut connection)
    .is_err());
    // An attempt cannot reference an absent job.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{absent}', 1, gen_random_uuid(), 'worker')"
    ))
    .execute(&mut connection)
    .is_err());

    // Deleting the publisher cascades to jobs, targets and attempts.
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_target"
        ),
        1
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt"
        ),
        1
    );
    publisher.delete(&pool).expect("delete publisher");
    for table in [
        "distribution_job",
        "distribution_job_target",
        "distribution_job_attempt",
    ] {
        assert_eq!(
            scalar_count(&pool, &format!("SELECT count(*) AS count FROM {table}")),
            0,
            "{table} must cascade with its publisher"
        );
    }
}

#[test]
fn every_uniqueness_rule_is_refused_by_the_database() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );
    let job = only_job(&pool, publisher.publisher_id);
    let claimed = claim_one(&pool);
    let mut connection = pool.get().expect("connection");

    // Duplicate deduplication key.
    assert!(raw_insert_job(
        &pool,
        "kind, publisher_id, activation_id, deduplication_key",
        &format!(
            "'PUBLISHER_BACK_CATALOGUE', '{}', '{}', '{}'",
            publisher.publisher_id, job.activation_id, job.deduplication_key
        )
    )
    .is_err());

    // Duplicate (job, platform) target.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_target (distribution_job_id, platform) \
         VALUES ('{}', 'ZENODO')",
        job.distribution_job_id
    ))
    .execute(&mut connection)
    .is_err());

    // Duplicate (job, attempt_number).
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{}', 1, gen_random_uuid(), 'worker')",
        job.distribution_job_id
    ))
    .execute(&mut connection)
    .is_err());

    // Duplicate claim token across attempts. `UNIQUE (claim_token)` is what
    // binds a token to exactly one attempt row for all time.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{}', 2, '{}', 'worker')",
        job.distribution_job_id, claimed.claim_token
    ))
    .execute(&mut connection)
    .is_err());
}

#[test]
fn every_invalid_job_state_is_refused_by_the_database() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let work_publisher = test_db::create_publisher(&pool);
    let imprint = test_db::create_imprint(&pool, &work_publisher);
    let work = test_db::create_work(&pool, &imprint);
    let publisher_id = publisher.publisher_id;

    let base_columns = "kind, publisher_id, activation_id, deduplication_key";
    let fresh = |extra_columns: &str, extra_values: &str| {
        let activation_id = Uuid::new_v4();
        raw_insert_job(
            &pool,
            &format!("{base_columns}{extra_columns}"),
            &format!(
                "'PUBLISHER_BACK_CATALOGUE', '{publisher_id}', '{activation_id}', '{}'{extra_values}",
                key_for(publisher_id, activation_id)
            ),
        )
    };

    // A control: the shape the domain function writes is accepted.
    assert!(fresh("", "").is_ok());

    // RUNNING with any claim field null.
    assert!(fresh(", status", ", 'RUNNING'").is_err());
    assert!(
        fresh(
            ", status, claim_token, claimed_by, claimed_at",
            ", 'RUNNING', gen_random_uuid(), 'worker', CURRENT_TIMESTAMP"
        )
        .is_err(),
        "a RUNNING row with no lease is refused"
    );
    // Non-RUNNING with any claim field non-null.
    assert!(fresh(", claim_token", ", gen_random_uuid()").is_err());
    assert!(fresh(", claimed_by", ", 'worker'").is_err());
    assert!(fresh(", claimed_at", ", CURRENT_TIMESTAMP").is_err());
    assert!(fresh(", lease_expires_at", ", CURRENT_TIMESTAMP").is_err());

    // Terminal status with completed_at null.
    for terminal in ["SUCCEEDED", "FAILED"] {
        assert!(fresh(", status", &format!(", '{terminal}'")).is_err());
    }
    assert!(fresh(
        ", status, cancellation_reason",
        ", 'CANCELLED', 'ADMINISTRATIVE'"
    )
    .is_err());
    // PENDING / RUNNING with completed_at non-null.
    assert!(fresh(", completed_at", ", CURRENT_TIMESTAMP").is_err());

    // CANCELLED without a reason, and a non-CANCELLED status with one.
    assert!(fresh(", status, completed_at", ", 'CANCELLED', CURRENT_TIMESTAMP").is_err());
    assert!(fresh(", cancellation_reason", ", 'ADMINISTRATIVE'").is_err());

    // A deduplication key that is not the formula.
    let activation_id = Uuid::new_v4();
    assert!(
        raw_insert_job(
            &pool,
            base_columns,
            &format!(
                "'PUBLISHER_BACK_CATALOGUE', '{publisher_id}', '{activation_id}', \
                 'PUBLISHER_BACK_CATALOGUE:{publisher_id}:{}'",
                Uuid::new_v4()
            )
        )
        .is_err(),
        "the formula check is what refuses a wrongly computed key"
    );
    assert!(
        raw_insert_job(
            &pool,
            base_columns,
            &format!("'PUBLISHER_BACK_CATALOGUE', '{publisher_id}', '{activation_id}', ''")
        )
        .is_err(),
        "an empty key is refused by the formula and the length check"
    );

    // A back-catalogue job with a work id.
    assert!(fresh(", work_id", &format!(", '{}'", work.work_id)).is_err());

    // Attempt-count bounds, on INSERT.
    assert!(fresh(", attempt_count", ", -1").is_err());
    assert!(fresh(", attempt_count", ", 6").is_err());
    assert!(fresh(", attempt_count", ", 5").is_ok());

    // Blank or whitespace-only claimed_by.
    assert!(fresh(
        ", status, claim_token, claimed_by, claimed_at, lease_expires_at",
        ", 'RUNNING', gen_random_uuid(), '   ', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP"
    )
    .is_err());

    // Error-field rules.
    assert!(fresh(", last_error_detail", ", 'detail without a code'").is_err());
    assert!(fresh(", last_error_code", ", 'lower_case'").is_err());
    assert!(fresh(", last_error_code", ", '9LEADING_DIGIT'").is_err());
    assert!(fresh(
        ", last_error_code",
        &format!(
            ", '{}'",
            "A".repeat(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS + 1)
        )
    )
    .is_err());
    assert!(fresh(
        ", last_error_code, last_error_detail",
        &format!(
            ", 'TRANSPORT_FAILURE', '{}'",
            "x".repeat(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS + 1)
        )
    )
    .is_err());
    assert!(fresh(
        ", last_error_code, last_error_detail",
        ", 'TRANSPORT_FAILURE', 'a bounded description of what failed'"
    )
    .is_ok());
}

#[test]
fn an_attempt_count_of_six_is_refused_on_update_as_well_as_insert() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_publisher_id, job_id) = publisher_with_pending_job(&pool);
    let mut connection = pool.get().expect("connection");

    for value in [-1, 6, 7, 100] {
        assert!(
            sql_query(format!(
                "UPDATE distribution_job SET attempt_count = {value} \
                 WHERE distribution_job_id = '{job_id}'"
            ))
            .execute(&mut connection)
            .is_err(),
            "the database must refuse attempt_count = {value}"
        );
    }
    assert_eq!(reload(&pool, job_id).attempt_count, 0);
}

#[test]
fn every_invalid_attempt_state_is_refused_by_the_database() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_publisher_id, job_id) = publisher_with_pending_job(&pool);
    let mut connection = pool.get().expect("connection");

    let insert = |extra_columns: &str, extra_values: &str| {
        sql_query(format!(
            "INSERT INTO distribution_job_attempt \
             (distribution_job_id, attempt_number, claim_token, claimed_by{extra_columns}) \
             VALUES ('{job_id}', {}, gen_random_uuid(), 'worker'{extra_values})",
            // A fresh ordinal each time, so the uniqueness rule is not what fails.
            rand_attempt_number()
        ))
        .execute(&mut pool.get().expect("connection"))
    };

    // attempt_number below 1.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{job_id}', 0, gen_random_uuid(), 'worker')"
    ))
    .execute(&mut connection)
    .is_err());

    // Blank claimed_by.
    assert!(sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{job_id}', 1, gen_random_uuid(), '  ')"
    ))
    .execute(&mut connection)
    .is_err());

    // finished_at without result, and result without finished_at.
    assert!(insert(", finished_at", ", CURRENT_TIMESTAMP").is_err());
    assert!(insert(", result", ", 'SUCCEEDED'").is_err());

    // finished_at before started_at.
    assert!(insert(
        ", started_at, finished_at, result",
        ", CURRENT_TIMESTAMP, CURRENT_TIMESTAMP - interval '1 hour', 'SUCCEEDED'"
    )
    .is_err());

    // Error fields on a non-FAILED result.
    assert!(insert(
        ", finished_at, result, error_code",
        ", CURRENT_TIMESTAMP, 'SUCCEEDED', 'TRANSPORT_FAILURE'"
    )
    .is_err());
    assert!(insert(
        ", finished_at, result, error_code",
        ", CURRENT_TIMESTAMP, 'ABANDONED', 'TRANSPORT_FAILURE'"
    )
    .is_err());
    assert!(insert(
        ", finished_at, result, error_code",
        ", CURRENT_TIMESTAMP, 'CANCELLED', 'TRANSPORT_FAILURE'"
    )
    .is_err());
    // Detail without a code.
    assert!(insert(
        ", finished_at, result, error_detail",
        ", CURRENT_TIMESTAMP, 'FAILED', 'a detail'"
    )
    .is_err());
    // Malformed and over-length code, and over-length detail.
    assert!(insert(
        ", finished_at, result, error_code",
        ", CURRENT_TIMESTAMP, 'FAILED', 'lower_case'"
    )
    .is_err());
    assert!(insert(
        ", finished_at, result, error_code",
        &format!(
            ", CURRENT_TIMESTAMP, 'FAILED', '{}'",
            "A".repeat(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS + 1)
        )
    )
    .is_err());
    assert!(insert(
        ", finished_at, result, error_code, error_detail",
        &format!(
            ", CURRENT_TIMESTAMP, 'FAILED', 'TRANSPORT_FAILURE', '{}'",
            "x".repeat(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS + 1)
        )
    )
    .is_err());
    // The conforming shape is accepted.
    assert!(insert(
        ", finished_at, result, error_code, error_detail",
        ", CURRENT_TIMESTAMP, 'FAILED', 'TRANSPORT_FAILURE', 'bounded description'"
    )
    .is_ok());
}

/// A distinct attempt ordinal per call, so uniqueness is never the reason an
/// invalid-state insert fails.
fn rand_attempt_number() -> i32 {
    use std::sync::atomic::{AtomicI32, Ordering};
    static NEXT: AtomicI32 = AtomicI32::new(2);
    NEXT.fetch_add(1, Ordering::SeqCst)
}

const ERROR_RESULT_CHECK: &str = "distribution_job_attempt_error_result_check";

/// The constraint PostgreSQL names when it refuses a write, so a rejection is
/// attributed to the constraint under test rather than to a neighbouring one
/// that happens to fail on the same row.
fn refusing_constraint(error: &diesel::result::Error) -> String {
    match error {
        diesel::result::Error::DatabaseError(_, info) => {
            info.constraint_name().unwrap_or("<unnamed>").to_owned()
        }
        other => panic!("expected a database error, observed {other:?}"),
    }
}

/// Insert one attempt row for `job_id` with the given extra columns/values,
/// under a fresh ordinal so uniqueness never decides the outcome.
fn insert_attempt(
    pool: &PgPool,
    job_id: Uuid,
    extra_columns: &str,
    extra_values: &str,
) -> Result<usize, diesel::result::Error> {
    sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by{extra_columns}) \
         VALUES ('{job_id}', {}, gen_random_uuid(), 'worker'{extra_values})",
        rand_attempt_number()
    ))
    .execute(&mut pool.get().expect("connection"))
}

/// Insert one valid **open** attempt — `finished_at IS NULL`, `result IS NULL`,
/// both error fields null — and return its identifier, so the `UPDATE` half of
/// the truth table starts from the state the claim statement actually creates.
fn open_attempt(pool: &PgPool, job_id: Uuid) -> Uuid {
    #[derive(QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = SqlUuid)]
        distribution_job_attempt_id: Uuid,
    }

    sql_query(format!(
        "INSERT INTO distribution_job_attempt \
         (distribution_job_id, attempt_number, claim_token, claimed_by) \
         VALUES ('{job_id}', {}, gen_random_uuid(), 'worker') \
         RETURNING distribution_job_attempt_id",
        rand_attempt_number()
    ))
    .get_result::<IdRow>(&mut pool.get().expect("connection"))
    .expect("an open attempt with no error fields must be accepted")
    .distribution_job_attempt_id
}

fn update_attempt(
    pool: &PgPool,
    attempt_id: Uuid,
    set_clause: &str,
) -> Result<usize, diesel::result::Error> {
    sql_query(format!(
        "UPDATE distribution_job_attempt SET {set_clause} \
         WHERE distribution_job_attempt_id = '{attempt_id}'"
    ))
    .execute(&mut pool.get().expect("connection"))
}

/// `distribution_job_attempt_error_result_check`, proven as an explicit
/// three-valued truth table on `INSERT` **and** on `UPDATE` (specification
/// sections 7.4 and 25.4).
///
/// The error fields are *closure* fields: they may exist only on an attempt a
/// worker closed with `result = 'FAILED'`. The withdrawn expression
/// `(error_code IS NULL AND error_detail IS NULL) OR result = 'FAILED'`
/// did not enforce that, because on an **open** attempt `result IS NULL` makes
/// the second arm `NULL`, and PostgreSQL admits a row whose `CHECK` evaluates
/// to `UNKNOWN`. The first two rejection cases below are exactly the rows that
/// expression admitted; a suite that omits them does not test this constraint.
///
/// The `UPDATE` half is not redundant: section 11.2's closure statements write
/// the error fields by update, so an insert-only suite would leave the write
/// path that actually sets them unproven.
#[test]
fn the_attempt_error_result_constraint_is_null_safe_on_insert_and_update() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_publisher_id, job_id) = publisher_with_pending_job(&pool);

    // The stored expression itself, so a future edit that reintroduces the
    // three-valued hole is visible in the catalog and not only in behaviour.
    let definition = catalog_values(
        &pool,
        &format!(
            "SELECT pg_get_constraintdef(oid) AS value FROM pg_constraint \
             WHERE conname = '{ERROR_RESULT_CHECK}'"
        ),
    );
    assert_eq!(
        definition.len(),
        1,
        "the constraint must exist exactly once"
    );
    let stored = &definition[0];
    for fragment in [
        "error_code IS NULL",
        "error_detail IS NULL",
        "result IS NOT NULL",
        "FAILED",
    ] {
        assert!(
            stored.contains(fragment),
            "{ERROR_RESULT_CHECK} must remain NULL-safe and contain {fragment:?}. \
             Observed: {stored}"
        );
    }

    // ------------------------------------------------------------------
    // Rejections. Each is expressed twice: as an INSERT of the state, and
    // as an UPDATE of an open attempt into the same state.
    //
    // `error_detail` alone is deliberately not a separate case here: it is
    // refused by distribution_job_attempt_error_pairing_check before this
    // constraint decides, so it would prove nothing about this one.
    // ------------------------------------------------------------------
    let rejections: [(&str, &str, &str, &str); 8] = [
        // 1. open attempt, error_code only - admitted by the old expression.
        (
            "open attempt with error_code",
            ", error_code",
            ", 'TRANSPORT_FAILURE'",
            "error_code = 'TRANSPORT_FAILURE'",
        ),
        // 2. open attempt, both error fields - admitted by the old expression.
        (
            "open attempt with error_code and error_detail",
            ", error_code, error_detail",
            ", 'TRANSPORT_FAILURE', 'bounded description'",
            "error_code = 'TRANSPORT_FAILURE', error_detail = 'bounded description'",
        ),
        // 3. SUCCEEDED with either error field set.
        (
            "SUCCEEDED with error_code",
            ", finished_at, result, error_code",
            ", CURRENT_TIMESTAMP, 'SUCCEEDED', 'TRANSPORT_FAILURE'",
            "finished_at = CURRENT_TIMESTAMP, result = 'SUCCEEDED', \
             error_code = 'TRANSPORT_FAILURE'",
        ),
        (
            "SUCCEEDED with error_code and error_detail",
            ", finished_at, result, error_code, error_detail",
            ", CURRENT_TIMESTAMP, 'SUCCEEDED', 'TRANSPORT_FAILURE', 'bounded description'",
            "finished_at = CURRENT_TIMESTAMP, result = 'SUCCEEDED', \
             error_code = 'TRANSPORT_FAILURE', error_detail = 'bounded description'",
        ),
        // 4. ABANDONED with either error field set.
        (
            "ABANDONED with error_code",
            ", finished_at, result, error_code",
            ", CURRENT_TIMESTAMP, 'ABANDONED', 'TRANSPORT_FAILURE'",
            "finished_at = CURRENT_TIMESTAMP, result = 'ABANDONED', \
             error_code = 'TRANSPORT_FAILURE'",
        ),
        (
            "ABANDONED with error_code and error_detail",
            ", finished_at, result, error_code, error_detail",
            ", CURRENT_TIMESTAMP, 'ABANDONED', 'TRANSPORT_FAILURE', 'bounded description'",
            "finished_at = CURRENT_TIMESTAMP, result = 'ABANDONED', \
             error_code = 'TRANSPORT_FAILURE', error_detail = 'bounded description'",
        ),
        // 5. CANCELLED with either error field set.
        (
            "CANCELLED with error_code",
            ", finished_at, result, error_code",
            ", CURRENT_TIMESTAMP, 'CANCELLED', 'TRANSPORT_FAILURE'",
            "finished_at = CURRENT_TIMESTAMP, result = 'CANCELLED', \
             error_code = 'TRANSPORT_FAILURE'",
        ),
        (
            "CANCELLED with error_code and error_detail",
            ", finished_at, result, error_code, error_detail",
            ", CURRENT_TIMESTAMP, 'CANCELLED', 'TRANSPORT_FAILURE', 'bounded description'",
            "finished_at = CURRENT_TIMESTAMP, result = 'CANCELLED', \
             error_code = 'TRANSPORT_FAILURE', error_detail = 'bounded description'",
        ),
    ];

    for (label, columns, values, set_clause) in rejections {
        let inserted = insert_attempt(&pool, job_id, columns, values);
        let error = inserted.expect_err(&format!(
            "INSERT of {label} must be refused by the database"
        ));
        assert_eq!(
            refusing_constraint(&error),
            ERROR_RESULT_CHECK,
            "INSERT of {label} must be refused by {ERROR_RESULT_CHECK} itself"
        );

        let attempt_id = open_attempt(&pool, job_id);
        let updated = update_attempt(&pool, attempt_id, set_clause);
        let error = updated.expect_err(&format!(
            "UPDATE into {label} must be refused by the database"
        ));
        assert_eq!(
            refusing_constraint(&error),
            ERROR_RESULT_CHECK,
            "UPDATE into {label} must be refused by {ERROR_RESULT_CHECK} itself"
        );
    }

    // ------------------------------------------------------------------
    // Acceptances, each proven to have actually persisted rather than
    // merely to have not errored.
    // ------------------------------------------------------------------

    // 6. An open attempt with both error fields null.
    let open_id = open_attempt(&pool, job_id);
    assert_eq!(
        update_attempt(&pool, open_id, "error_code = NULL, error_detail = NULL")
            .expect("an open attempt with both error fields null must be accepted"),
        1
    );
    assert_eq!(
        scalar_count(
            &pool,
            &format!(
                "SELECT count(*) AS count FROM distribution_job_attempt \
                 WHERE distribution_job_attempt_id = '{open_id}' \
                   AND finished_at IS NULL AND result IS NULL \
                   AND error_code IS NULL AND error_detail IS NULL"
            )
        ),
        1
    );

    // 7. A closed FAILED attempt carrying a valid code and detail.
    assert!(insert_attempt(
        &pool,
        job_id,
        ", finished_at, result, error_code, error_detail",
        ", CURRENT_TIMESTAMP, 'FAILED', 'TRANSPORT_FAILURE', 'bounded description'",
    )
    .is_ok());
    let failed_id = open_attempt(&pool, job_id);
    assert_eq!(
        update_attempt(
            &pool,
            failed_id,
            "finished_at = CURRENT_TIMESTAMP, result = 'FAILED', \
             error_code = 'TRANSPORT_FAILURE', error_detail = 'bounded description'",
        )
        .expect("a closed FAILED attempt with valid error fields must be accepted"),
        1
    );
    assert_eq!(
        scalar_count(
            &pool,
            &format!(
                "SELECT count(*) AS count FROM distribution_job_attempt \
                 WHERE distribution_job_attempt_id = '{failed_id}' \
                   AND result = 'FAILED' AND error_code = 'TRANSPORT_FAILURE'"
            )
        ),
        1
    );

    // 8. A closed FAILED attempt with both error fields null - a legitimate
    //    state, since the error fields are optional even at FAILED closure.
    assert!(insert_attempt(
        &pool,
        job_id,
        ", finished_at, result",
        ", CURRENT_TIMESTAMP, 'FAILED'",
    )
    .is_ok());
    let bare_id = open_attempt(&pool, job_id);
    assert_eq!(
        update_attempt(
            &pool,
            bare_id,
            "finished_at = CURRENT_TIMESTAMP, result = 'FAILED'",
        )
        .expect("a closed FAILED attempt with both error fields null must be accepted"),
        1
    );
    assert_eq!(
        scalar_count(
            &pool,
            &format!(
                "SELECT count(*) AS count FROM distribution_job_attempt \
                 WHERE distribution_job_attempt_id = '{bare_id}' \
                   AND result = 'FAILED' \
                   AND error_code IS NULL AND error_detail IS NULL"
            )
        ),
        1
    );
}

#[test]
fn a_job_with_several_targets_is_readable_in_canonical_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );
    let job = only_job(&pool, publisher.publisher_id);

    assert_eq!(
        targets_of(&pool, job.distribution_job_id),
        vec![DistributionPlatform::Oapen, DistributionPlatform::Doab],
        "the platform column is a PostgreSQL enum, so ordering by it is the \
         canonical declaration order"
    );
}

/// The zero-target invariant, asserted after every creation scenario this suite
/// produces rather than only after one.
fn assert_no_zero_target_job(pool: &PgPool) {
    assert_eq!(
        scalar_count(
            pool,
            "SELECT count(*) AS count FROM distribution_job j \
             WHERE NOT EXISTS (SELECT 1 FROM distribution_job_target t \
                               WHERE t.distribution_job_id = j.distribution_job_id)"
        ),
        0,
        "no logical job may exist with zero targets"
    );
}

// ==========================================================================
// 25.5  Creation and deduplication tests
// ==========================================================================

#[test]
fn one_linked_oapen_doab_activation_produces_one_job_and_two_targets() {
    // All three request shapes: name OAPEN, name DOAB, name both.
    for requested in [
        vec![DistributionPlatform::Oapen],
        vec![DistributionPlatform::Doab],
        vec![DistributionPlatform::Oapen, DistributionPlatform::Doab],
    ] {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        activate(&pool, publisher.publisher_id, &requested);

        let job = only_job(&pool, publisher.publisher_id);
        assert_eq!(job.status, DistributionJobStatus::Pending);
        assert_eq!(job.kind, DistributionJobKind::PublisherBackCatalogue);
        assert_eq!(job.attempt_count, 0);
        assert!(job.work_id.is_none());
        assert!(job.claim_token.is_none());
        assert!(job.completed_at.is_none());

        assert_eq!(
            targets_of(&pool, job.distribution_job_id),
            vec![DistributionPlatform::Oapen, DistributionPlatform::Doab],
            "one activation, one job, one target per AutomaticPush member, in \
             canonical order — requested {requested:?}"
        );

        // The one activation the two members share is the one the key derives
        // from.
        let assignments =
            PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
                .expect("assignments");
        let activations: HashSet<Uuid> = assignments
            .iter()
            .filter(|row| {
                matches!(
                    row.platform,
                    DistributionPlatform::Oapen | DistributionPlatform::Doab
                )
            })
            .map(|row| row.activation_id)
            .collect();
        assert_eq!(activations.len(), 1, "one shared activation identity");
        assert_eq!(job.activation_id, *activations.iter().next().unwrap());
        assert_eq!(
            job.deduplication_key,
            key_for(publisher.publisher_id, job.activation_id)
        );
        assert_no_zero_target_job(&pool);
    }
}

#[test]
fn a_repeated_observation_of_one_activation_creates_no_second_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );
    let first = only_job(&pool, publisher.publisher_id);

    // A second replacement naming the same linked group over already-normalized
    // state: the lifecycle reports `Unchanged`, so nothing is even attempted.
    activate(&pool, publisher.publisher_id, &[DistributionPlatform::Doab]);
    assert_eq!(jobs_of(&pool, publisher.publisher_id).len(), 1);
    assert_eq!(
        only_job(&pool, publisher.publisher_id).distribution_job_id,
        first.distribution_job_id
    );
}

#[test]
fn an_independent_activation_creates_its_own_job_with_its_own_single_target() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Zenodo],
    );

    let job = only_job(&pool, publisher.publisher_id);
    assert_eq!(
        targets_of(&pool, job.distribution_job_id),
        vec![DistributionPlatform::Zenodo]
    );
    assert_no_zero_target_job(&pool);
}

#[test]
fn activating_a_linked_group_and_an_independent_destination_creates_exactly_two_jobs() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
    );

    let jobs = jobs_of(&pool, publisher.publisher_id);
    assert_eq!(jobs.len(), 2, "one job per qualifying activated group");
    // Rendered as labels, because `DistributionPlatform` is deliberately not
    // `Ord`: its canonical order is the enum's declaration order, not a
    // comparison.
    let mut all_targets: Vec<Vec<String>> = jobs
        .iter()
        .map(|job| {
            targets_of(&pool, job.distribution_job_id)
                .into_iter()
                .map(|platform| platform.to_string())
                .collect()
        })
        .collect();
    all_targets.sort();
    let mut expected = vec![
        vec!["OAPEN".to_string(), "DOAB".to_string()],
        vec!["ZENODO".to_string()],
    ];
    expected.sort();
    assert_eq!(all_targets, expected);
    // Distinct activations therefore distinct keys.
    let keys: HashSet<&str> = jobs
        .iter()
        .map(|job| job.deduplication_key.as_str())
        .collect();
    assert_eq!(keys.len(), 2);
    assert_no_zero_target_job(&pool);
}

#[test]
fn pull_feed_and_manual_activations_create_no_uploader_job() {
    for platforms in [
        vec![DistributionPlatform::OclcKb],
        vec![DistributionPlatform::ExLibrisKb],
        vec![
            DistributionPlatform::OclcKb,
            DistributionPlatform::ExLibrisKb,
        ],
        vec![DistributionPlatform::ScienceOpen],
    ] {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        activate(&pool, publisher.publisher_id, &platforms);

        assert!(
            jobs_of(&pool, publisher.publisher_id).is_empty(),
            "{platforms:?} never creates an uploader job"
        );
        // The activation itself still committed.
        assert!(
            !PublisherDistributionPlatform::enabled_assignments(&pool, publisher.publisher_id)
                .expect("assignments")
                .is_empty()
        );
    }
}

#[test]
fn package_only_no_op_and_stale_changes_create_no_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;

    // Package-only: no group is Activated, and no assignment row is written.
    replace(
        &pool,
        &write_context(DistributionJobCreation::On),
        publisher_id,
        ThothPackage::Obelisk,
        &[],
    )
    .expect("package-only change");
    assert!(jobs_of(&pool, publisher_id).is_empty());
    assert!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher_id)
            .expect("assignments")
            .is_empty()
    );

    // True no-op: the coordinator returns before any write.
    let before = token(&pool, publisher_id);
    replace(
        &pool,
        &write_context(DistributionJobCreation::On),
        publisher_id,
        ThothPackage::Obelisk,
        &[],
    )
    .expect("no-op");
    assert_eq!(token(&pool, publisher_id), before);
    assert!(jobs_of(&pool, publisher_id).is_empty());

    // Stale: rejected before any write, so no job.
    let stale = replace_publisher_service_configuration(
        &pool,
        &write_context(DistributionJobCreation::On),
        &ReplacePublisherServiceConfigurationInput {
            publisher_id,
            subscription_package: ThothPackage::Sphinx,
            enabled_distribution_platforms: vec![DistributionPlatform::Zenodo],
            expected_updated_at: Timestamp::default(),
        },
    );
    assert!(matches!(
        stale,
        Err(ThothError::StalePublisherServiceConfiguration)
    ));
    assert!(jobs_of(&pool, publisher_id).is_empty());
    assert_eq!(token(&pool, publisher_id), before);
}

#[test]
fn a_linked_state_repair_creates_no_job_and_infers_no_delivery() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;

    // Enable the linked pair with creation OFF is impossible (fail-closed), so
    // seed the enabled pair the way a pre-BE-04 publisher would have it: write
    // the rows directly, with **no** job.
    let mut connection = pool.get().expect("connection");
    let activation_id = Uuid::new_v4();
    for platform in ["OAPEN", "DOAB"] {
        sql_query(format!(
            "INSERT INTO publisher_distribution_platform \
             (publisher_id, platform, enabled, activation_id, enabled_at) \
             VALUES ('{publisher_id}', '{platform}', true, '{activation_id}', now())"
        ))
        .execute(&mut connection)
        .expect("seed assignment");
    }
    // Split the pair's activation so the group is no longer normalized.
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET activation_id = '{}' \
         WHERE publisher_id = '{publisher_id}' AND platform = 'DOAB'",
        Uuid::new_v4()
    ))
    .execute(&mut connection)
    .expect("split activation");
    drop(connection);

    assert!(jobs_of(&pool, publisher_id).is_empty());

    // The repair commits — under `ON` *and* under `OFF`, because a repair never
    // qualifies.
    for creation in [DistributionJobCreation::On, DistributionJobCreation::Off] {
        let before = token(&pool, publisher_id);
        let outcome = replace(
            &pool,
            &write_context(creation),
            publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen],
        );
        if creation == DistributionJobCreation::On {
            outcome.expect("a repair must commit");
            assert!(
                token(&pool, publisher_id) > before,
                "a repair moves the token"
            );
        } else {
            // By now the group is already normalized, so this second call is a
            // no-op — which also commits under OFF.
            outcome.expect("a normalized no-op must commit under OFF");
        }
        assert!(
            jobs_of(&pool, publisher_id).is_empty(),
            "a repair creates no automatic job, because it is not a new \
             zero-enabled-to-enabled activation — and for no other reason"
        );
    }

    // The durable record is "no job", and nothing anywhere fabricates a status.
    let mut connection = pool.get().expect("connection");
    assert!(
        latest_back_catalogue_jobs(&mut connection, &[publisher_id])
            .expect("latest job")
            .is_empty(),
        "a repaired group with no job stays no job"
    );
}

#[test]
fn migration_backfill_creates_no_job_under_either_switch_position() {
    for creation in [DistributionJobCreation::Off, DistributionJobCreation::On] {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let publisher_id = publisher.publisher_id;

        replace(
            &pool,
            &backfill_context(creation),
            publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
        )
        .expect("MIGRATION_BACKFILL commits normally and never fails under this rule");

        assert!(
            jobs_of(&pool, publisher_id).is_empty(),
            "an imported existing assignment must create zero onboarding jobs"
        );
        // The assignments themselves were written.
        assert_eq!(
            PublisherDistributionPlatform::enabled_assignments(&pool, publisher_id)
                .expect("assignments")
                .len(),
            3
        );
    }
}

#[test]
fn jisc_nbk_is_rejected_before_any_write_and_never_becomes_a_target() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    let before = token(&pool, publisher_id);

    let outcome = replace(
        &pool,
        &write_context(DistributionJobCreation::On),
        publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Zenodo, DistributionPlatform::JiscNbk],
    );
    assert!(matches!(
        outcome,
        Err(ThothError::DistributionPlatformNotAssignable(ref platform)) if platform == "JISC_NBK"
    ));

    assert_eq!(token(&pool, publisher_id), before, "no write occurred");
    assert!(jobs_of(&pool, publisher_id).is_empty());
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_target WHERE platform = 'JISC_NBK'"
        ),
        0
    );
}

#[test]
fn a_disable_then_re_enable_cycle_creates_a_second_legitimate_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;

    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let first = only_job(&pool, publisher_id);

    // Disable: no new job, and the pending one is withdrawn.
    activate(&pool, publisher_id, &[]);
    assert_eq!(
        reload(&pool, first.distribution_job_id).status,
        DistributionJobStatus::Cancelled
    );

    // Re-enable: `enable_on` sees zero enabled members, mints a **new**
    // activation, so the key differs and a new job is legitimately created.
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let jobs = jobs_of(&pool, publisher_id);
    assert_eq!(jobs.len(), 2);
    assert_ne!(jobs[0].activation_id, jobs[1].activation_id);
    assert_ne!(jobs[0].deduplication_key, jobs[1].deduplication_key);
    assert_eq!(jobs[1].status, DistributionJobStatus::Pending);
    assert_no_zero_target_job(&pool);
}

#[test]
fn two_real_connections_creating_one_activation_produce_exactly_one_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    let activation_id = Uuid::new_v4();

    // Both connections attempt the creation for the *same* logical activation,
    // bypassing the publisher row lock entirely, so it is the unique constraint
    // and `ON CONFLICT DO NOTHING` alone that is under test — the guarantee that
    // survives any future creation path.
    let outcomes: Vec<Option<Uuid>> = (0..2)
        .map(|_| {
            let pool = pool.clone();
            thread::spawn(move || {
                let mut connection = pool.get().expect("connection");
                connection
                    .transaction(|connection| {
                        super::crud::create_back_catalogue_job_on(
                            connection,
                            publisher_id,
                            activation_id,
                            &[DistributionPlatform::Zenodo],
                        )
                    })
                    .expect("creation must not error, even when it creates nothing")
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().expect("thread"))
        .collect();

    assert_eq!(
        outcomes.iter().filter(|created| created.is_some()).count(),
        1,
        "exactly one observation created the job"
    );
    assert_eq!(jobs_of(&pool, publisher_id).len(), 1);
    assert_eq!(
        targets_of(&pool, jobs_of(&pool, publisher_id)[0].distribution_job_id).len(),
        1
    );
    assert_no_zero_target_job(&pool);
}

// ==========================================================================
// 25.6  Switch, fail-closed and sweep tests
// ==========================================================================

/// Everything the `OFF` rollback must leave untouched, captured as one value.
#[derive(Debug, PartialEq)]
struct CommittedSnapshot {
    assignments: Vec<String>,
    configuration_token: Timestamp,
    publisher_row: String,
    audit_rows: i64,
    jobs: i64,
    targets: i64,
    work_freshness: Vec<String>,
}

fn snapshot(pool: &PgPool, publisher_id: Uuid) -> CommittedSnapshot {
    CommittedSnapshot {
        assignments: catalog_values(
            pool,
            &format!(
                "SELECT platform::text || ' ' || enabled::text || ' ' || activation_id::text \
                 || ' ' || enabled_at::text || ' ' || coalesce(disabled_at::text, '-') \
                 || ' ' || updated_at::text AS value \
                 FROM publisher_distribution_platform WHERE publisher_id = '{publisher_id}' \
                 ORDER BY platform"
            ),
        ),
        configuration_token: token(pool, publisher_id),
        publisher_row: catalog_values(
            pool,
            &format!(
                "SELECT publisher_name || ' ' || subscription_package::text || ' ' \
                 || updated_at::text AS value FROM publisher WHERE publisher_id = '{publisher_id}'"
            ),
        )
        .remove(0),
        audit_rows: scalar_count(
            pool,
            &format!(
                "SELECT count(*) AS count FROM publisher_service_configuration_history \
                 WHERE publisher_id = '{publisher_id}'"
            ),
        ),
        jobs: scalar_count(pool, "SELECT count(*) AS count FROM distribution_job"),
        targets: scalar_count(
            pool,
            "SELECT count(*) AS count FROM distribution_job_target",
        ),
        work_freshness: catalog_values(
            pool,
            &format!(
                "SELECT w.updated_at_with_relations::text AS value FROM work w \
                 JOIN imprint i ON i.imprint_id = w.imprint_id \
                 WHERE i.publisher_id = '{publisher_id}' ORDER BY w.work_id"
            ),
        ),
    }
}

#[test]
fn with_creation_off_a_qualifying_activation_fails_and_rolls_back_in_full() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    let imprint = test_db::create_imprint(&pool, &publisher);
    for _ in 0..3 {
        test_db::create_work(&pool, &imprint);
    }
    // Some pre-existing non-qualifying state, so the snapshot is not trivially
    // empty.
    replace(
        &pool,
        &write_context(DistributionJobCreation::Off),
        publisher_id,
        ThothPackage::Obelisk,
        &[DistributionPlatform::OclcKb],
    )
    .expect("a PullFeed activation commits normally under OFF");

    let before = snapshot(&pool, publisher_id);

    let outcome = replace(
        &pool,
        &write_context(DistributionJobCreation::Off),
        publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::OclcKb, DistributionPlatform::Oapen],
    );
    assert!(
        matches!(outcome, Err(ThothError::DistributionJobCreationDisabled)),
        "a qualifying AutomaticPush activation must fail closed under OFF, got {outcome:?}"
    );

    let after = snapshot(&pool, publisher_id);
    assert_eq!(
        before, after,
        "the whole transaction must roll back: assignment rows, activation ids, \
         the configuration token, the publisher row, the audit table, the job \
         tables and the works' freshness signal are all unchanged"
    );
    // Stated individually as well, so a failure names the property that broke.
    assert_eq!(before.assignments, after.assignments);
    assert_eq!(before.configuration_token, after.configuration_token);
    assert_eq!(after.audit_rows, before.audit_rows);
    assert_eq!(after.jobs, 0);
    assert_eq!(after.targets, 0);
    assert_eq!(before.work_freshness, after.work_freshness);
    assert!(
        !after.assignments.iter().any(|row| row.starts_with("OAPEN")),
        "no new activation may be committed"
    );
}

#[test]
fn the_same_activation_retried_with_creation_on_creates_exactly_one_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;

    // The switch is the **only** difference between these two calls.
    let refused = replace(
        &pool,
        &write_context(DistributionJobCreation::Off),
        publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    );
    assert!(matches!(
        refused,
        Err(ThothError::DistributionJobCreationDisabled)
    ));

    replace(
        &pool,
        &write_context(DistributionJobCreation::On),
        publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    )
    .expect("the same desired activation must succeed under ON");

    let job = only_job(&pool, publisher_id);
    assert_eq!(
        targets_of(&pool, job.distribution_job_id),
        vec![DistributionPlatform::Oapen, DistributionPlatform::Doab],
        "the earlier refusal lost nothing and left no partial state that would \
         deduplicate the retry away"
    );
    assert_eq!(
        scalar_count(
            &pool,
            &format!(
                "SELECT count(*) AS count FROM publisher_service_configuration_history \
                 WHERE publisher_id = '{publisher_id}'"
            )
        ),
        1,
        "exactly one audit row, from the one committed change"
    );
}

#[test]
fn creation_off_permits_every_non_qualifying_change() {
    let (_guard, pool) = test_db::setup_test_db();
    let off = write_context(DistributionJobCreation::Off);

    // PullFeed only.
    let pull = test_db::create_publisher(&pool);
    replace(
        &pool,
        &off,
        pull.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::OclcKb],
    )
    .expect("PullFeed activation commits under OFF");
    // Manual only.
    let manual = test_db::create_publisher(&pool);
    replace(
        &pool,
        &off,
        manual.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::ScienceOpen],
    )
    .expect("Manual activation commits under OFF");
    // Package-only.
    let package = test_db::create_publisher(&pool);
    replace(
        &pool,
        &off,
        package.publisher_id,
        ThothPackage::Obelisk,
        &[],
    )
    .expect("package-only change commits under OFF");
    // True no-op.
    replace(
        &pool,
        &off,
        package.publisher_id,
        ThothPackage::Obelisk,
        &[],
    )
    .expect("no-op commits under OFF");
    // Disable.
    replace(&pool, &off, pull.publisher_id, ThothPackage::Sphinx, &[])
        .expect("disable commits under OFF");
    // Repair: seed an enabled split pair directly, then normalize it.
    let repair = test_db::create_publisher(&pool);
    let mut connection = pool.get().expect("connection");
    for (platform, activation) in [("OAPEN", Uuid::new_v4()), ("DOAB", Uuid::new_v4())] {
        sql_query(format!(
            "INSERT INTO publisher_distribution_platform \
             (publisher_id, platform, enabled, activation_id, enabled_at) \
             VALUES ('{}', '{platform}', true, '{activation}', now())",
            repair.publisher_id
        ))
        .execute(&mut connection)
        .expect("seed split pair");
    }
    drop(connection);
    replace(
        &pool,
        &off,
        repair.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::Oapen],
    )
    .expect("a repair commits under OFF");

    assert_eq!(
        scalar_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
        0,
        "none of these is a qualifying activation, so none creates a job"
    );
}

#[test]
fn enabling_the_switch_performs_no_retroactive_sweep() {
    let (_guard, pool) = test_db::setup_test_db();

    // Several publishers with pre-existing enabled AutomaticPush assignments,
    // written the way a pre-BE-04 database holds them: no job.
    let mut connection = pool.get().expect("connection");
    let publishers: Vec<Uuid> = (0..3)
        .map(|_| {
            let publisher = test_db::create_publisher(&pool);
            sql_query(format!(
                "INSERT INTO publisher_distribution_platform \
                 (publisher_id, platform, enabled, activation_id, enabled_at) \
                 VALUES ('{}', 'ZENODO', true, '{}', now())",
                publisher.publisher_id,
                Uuid::new_v4()
            ))
            .execute(&mut connection)
            .expect("seed assignment");
            publisher.publisher_id
        })
        .collect();
    drop(connection);

    assert_eq!(
        scalar_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
        0
    );

    // Turning the switch on executes no code over existing rows: there is no
    // sweep, backfill, startup scan, reconciliation pass or lazy creation.
    let on = write_context(DistributionJobCreation::On);
    for publisher_id in &publishers {
        // An unrelated no-op replacement still creates nothing.
        replace(
            &pool,
            &on,
            *publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
        )
        .expect("no-op");
    }
    assert_eq!(
        scalar_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
        0,
        "no historical assignment is enqueued retroactively"
    );

    // Only a genuinely fresh activation creates one.
    replace(&pool, &on, publishers[0], ThothPackage::Sphinx, &[]).expect("disable");
    replace(
        &pool,
        &on,
        publishers[0],
        ThothPackage::Sphinx,
        &[DistributionPlatform::Zenodo],
    )
    .expect("re-enable");
    assert_eq!(jobs_of(&pool, publishers[0]).len(), 1);
    assert!(jobs_of(&pool, publishers[1]).is_empty());
    assert!(jobs_of(&pool, publishers[2]).is_empty());
}

#[test]
fn the_merged_default_is_off() {
    assert_eq!(
        DistributionJobCreation::default(),
        DistributionJobCreation::Off,
        "every construction path that omits the switch is inactive"
    );
    assert_eq!(
        write_context(DistributionJobCreation::default()).job_creation,
        DistributionJobCreation::Off
    );
}

// ==========================================================================
// 25.8  Concurrency, claim, state-machine and retry tests
//
// These use multiple real connections and real transactions. There are no
// mocked sequential substitutes anywhere in this section.
// ==========================================================================

#[test]
fn a_claim_returns_exactly_the_jobs_it_claimed_at_zero_one_and_many() {
    let (_guard, pool) = test_db::setup_test_db();

    // Zero: no due job at all.
    let none = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
    assert!(
        none.is_empty(),
        "zero claims returns zero rows, not an error"
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt"
        ),
        0
    );

    // One.
    let (_publisher_id, job_id) = publisher_with_pending_job(&pool);
    let claimed = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
    assert_eq!(claimed.len(), 1);
    let one = &claimed[0];
    assert_eq!(one.job.job.distribution_job_id, job_id);
    assert_eq!(one.job.job.status, DistributionJobStatus::Running);
    assert_eq!(one.job.job.attempt_count, 1);
    assert_eq!(one.attempt_number, 1);
    assert_eq!(one.job.job.claimed_by.as_deref(), Some(WORKER));
    assert_eq!(one.job.job.claim_token, Some(one.claim_token));
    assert_eq!(one.job.job.lease_expires_at, Some(one.lease_expires_at));
    let attempts = attempts_of(&pool, job_id);
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].attempt_number, 1);
    assert_eq!(attempts[0].claim_token, one.claim_token);
    assert_eq!(attempts[0].claimed_by, WORKER);
    assert!(attempts[0].finished_at.is_none());
    assert!(attempts[0].result.is_none());

    // Many: M due jobs, batch of M.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let mut expected: Vec<Uuid> = Vec::new();
    for _ in 0..7 {
        let (_p, job_id) = publisher_with_pending_job(&pool);
        expected.push(job_id);
    }
    let claimed = claim_distribution_jobs(&pool, WORKER, 7, 900, &[]).expect("claim");
    assert_eq!(claimed.len(), 7);
    let tokens: HashSet<Uuid> = claimed.iter().map(|claim| claim.claim_token).collect();
    assert_eq!(
        tokens.len(),
        7,
        "each claimed job receives its own distinct token"
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt"
        ),
        7,
        "exactly one attempt row per claimed job"
    );
    for claim in &claimed {
        assert_eq!(claim.attempt_number, 1);
        assert_eq!(claim.job.job.attempt_count, 1);
    }
}

#[test]
fn claims_are_returned_in_the_deterministic_total_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let mut jobs: Vec<(Timestamp, Uuid)> = Vec::new();
    for offset in [30, 10, 20, 40, 10] {
        let (_p, job_id) = publisher_with_pending_job(&pool);
        sql_query(format!(
            "UPDATE distribution_job \
             SET available_at = CURRENT_TIMESTAMP - interval '{offset} minutes' \
             WHERE distribution_job_id = '{job_id}'"
        ))
        .execute(&mut connection)
        .expect("stagger availability");
        jobs.push((reload(&pool, job_id).available_at, job_id));
    }
    drop(connection);

    let mut expected: Vec<(Timestamp, Uuid)> = jobs.clone();
    expected.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

    let claimed = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
    let observed: Vec<Uuid> = claimed
        .iter()
        .map(|claim| claim.job.job.distribution_job_id)
        .collect();
    assert_eq!(
        observed,
        expected.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
        "claim order is available_at ASC, distribution_job_id ASC — a total order"
    );
}

#[test]
fn concurrent_workers_receive_disjoint_claim_sets_and_never_block_each_other() {
    let (_guard, pool) = test_db::setup_test_db();
    const JOBS: usize = 24;
    const WORKERS: usize = 4;

    for _ in 0..JOBS {
        publisher_with_pending_job(&pool);
    }

    let started = Instant::now();
    let handles: Vec<_> = (0..WORKERS)
        .map(|index| {
            let pool = pool.clone();
            thread::spawn(move || {
                let worker = format!("worker-{index}");
                claim_distribution_jobs(&pool, &worker, 10, 900, &[]).expect("claim")
            })
        })
        .collect();
    let results: Vec<Vec<ClaimedDistributionJob>> = handles
        .into_iter()
        .map(|handle| handle.join().expect("worker thread"))
        .collect();
    let elapsed = started.elapsed();

    // Disjoint, and together a partition of what was claimed.
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut tokens: HashSet<Uuid> = HashSet::new();
    for batch in &results {
        for claim in batch {
            assert!(
                seen.insert(claim.job.job.distribution_job_id),
                "a job was claimed twice"
            );
            assert!(tokens.insert(claim.claim_token), "a token was issued twice");
        }
    }
    assert_eq!(seen.len(), JOBS, "every due job was claimed exactly once");

    // No worker observed another's job or token: each batch's members carry that
    // worker's own identity.
    for (index, batch) in results.iter().enumerate() {
        let worker = format!("worker-{index}");
        for claim in batch {
            assert_eq!(claim.job.job.claimed_by.as_deref(), Some(worker.as_str()));
        }
    }

    // Exactly one open attempt per claimed job.
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt WHERE finished_at IS NULL"
        ),
        JOBS as i64
    );
    // `SKIP LOCKED` means no worker waits on another's locked rows. This is a
    // generous bound: it is evidence of non-blocking, not a performance target.
    assert!(
        elapsed < Duration::from_secs(20),
        "workers must not serialize behind one another: {elapsed:?}"
    );
}

#[test]
fn claim_bounds_are_clamped_rather_than_rejected() {
    let (_guard, pool) = test_db::setup_test_db();
    for _ in 0..(DISTRIBUTION_JOB_CLAIM_MAX_BATCH + 5) {
        publisher_with_pending_job(&pool);
    }

    // `limit <= 0` claims nothing and is not an error.
    for limit in [0, -1, -100] {
        assert!(claim_distribution_jobs(&pool, WORKER, limit, 900, &[])
            .expect("claim")
            .is_empty());
    }
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt"
        ),
        0
    );

    // Above the maximum clamps to the maximum.
    let claimed = claim_distribution_jobs(&pool, WORKER, 5_000, 900, &[]).expect("claim");
    assert_eq!(claimed.len() as i32, DISTRIBUTION_JOB_CLAIM_MAX_BATCH);

    // Lease seconds clamp to the nearer bound.
    test_db::reset_db(&pool).expect("reset the disposable database");
    publisher_with_pending_job(&pool);
    let short = claim_distribution_jobs(&pool, WORKER, 1, 1, &[]).expect("claim");
    let granted = short[0].lease_expires_at.to_rfc3339();
    let lower_bound = scalar_count(
        &pool,
        &format!(
            "SELECT count(*) AS count FROM distribution_job \
             WHERE lease_expires_at >= claimed_at + interval '{DISTRIBUTION_JOB_LEASE_MIN_SECONDS} seconds' \
               AND lease_expires_at < claimed_at + interval '{} seconds'",
            DISTRIBUTION_JOB_LEASE_MIN_SECONDS + 5
        ),
    );
    assert_eq!(
        lower_bound, 1,
        "a too-short lease clamps up to the minimum ({granted})"
    );

    test_db::reset_db(&pool).expect("reset the disposable database");
    publisher_with_pending_job(&pool);
    claim_distribution_jobs(&pool, WORKER, 1, 100_000, &[]).expect("claim");
    let upper_bound = scalar_count(
        &pool,
        &format!(
            "SELECT count(*) AS count FROM distribution_job \
             WHERE lease_expires_at <= claimed_at + interval '{DISTRIBUTION_JOB_LEASE_MAX_SECONDS} seconds' \
               AND lease_expires_at > claimed_at + interval '{} seconds'",
            DISTRIBUTION_JOB_LEASE_MAX_SECONDS - 5
        ),
    );
    assert_eq!(
        upper_bound, 1,
        "a too-long lease clamps down to the maximum"
    );
}

#[test]
fn the_kinds_filter_selects_correctly() {
    let (_guard, pool) = test_db::setup_test_db();
    publisher_with_pending_job(&pool);

    assert_eq!(
        claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .len(),
        1,
        "an empty kinds list claims any kind"
    );
    test_db::reset_db(&pool).expect("reset the disposable database");
    publisher_with_pending_job(&pool);
    assert_eq!(
        claim_distribution_jobs(
            &pool,
            WORKER,
            10,
            900,
            &[DistributionJobKind::PublisherBackCatalogue]
        )
        .expect("claim")
        .len(),
        1
    );
}

#[test]
fn eligibility_refuses_a_disabled_target_a_different_activation_and_an_exhausted_budget() {
    // A target disabled while its job was pending.
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let job = only_job(&pool, publisher_id);
    // Disable the assignment directly, leaving the job PENDING — the state the
    // fail-closed backstop exists for, reached by a route other than the
    // coordinator's own deterministic cancellation.
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET enabled = false, disabled_at = now() \
         WHERE publisher_id = '{publisher_id}' AND platform = 'ZENODO'"
    ))
    .execute(&mut connection)
    .expect("disable directly");
    drop(connection);
    assert!(
        claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .is_empty(),
        "a job whose target is no longer enabled is never claimed"
    );
    // It is neither deleted nor mutated: it stays visible for explicit
    // cancellation.
    assert_eq!(
        reload(&pool, job.distribution_job_id).status,
        DistributionJobStatus::Pending
    );

    // A target enabled again under a *different* activation.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let first = only_job(&pool, publisher_id);
    activate(&pool, publisher_id, &[]);
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    // Undo the withdrawal cancellation so only the activation check can exclude
    // the old job.
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE distribution_job SET status = 'PENDING', completed_at = NULL, \
         cancellation_reason = NULL WHERE distribution_job_id = '{}'",
        first.distribution_job_id
    ))
    .execute(&mut connection)
    .expect("restore the old job to PENDING");
    drop(connection);
    let claimed = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
    assert_eq!(
        claimed.len(),
        1,
        "only the current activation's job is claimable"
    );
    assert_ne!(
        claimed[0].job.job.distribution_job_id, first.distribution_job_id,
        "an old job must not become claimable alongside its successor: the same \
         back catalogue would otherwise be pushed twice"
    );

    // An exhausted PENDING row — the malformed/legacy shape the independent
    // eligibility clause exists for.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE distribution_job SET attempt_count = {DISTRIBUTION_JOB_MAX_ATTEMPTS} \
         WHERE distribution_job_id = '{job_id}'"
    ))
    .execute(&mut connection)
    .expect("write an exhausted PENDING row directly");
    drop(connection);
    assert!(
        claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .is_empty(),
        "an exhausted PENDING row is never claimed, whatever produced it"
    );
    assert_eq!(
        reload(&pool, job_id).attempt_count,
        DISTRIBUTION_JOB_MAX_ATTEMPTS
    );
}

#[test]
fn every_target_of_a_linked_job_must_qualify_not_merely_one() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    activate(&pool, publisher_id, &[DistributionPlatform::Oapen]);

    // Disable exactly one member directly, leaving the other enabled under the
    // job's own activation.
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET enabled = false, disabled_at = now() \
         WHERE publisher_id = '{publisher_id}' AND platform = 'DOAB'"
    ))
    .execute(&mut connection)
    .expect("disable one member");
    drop(connection);

    assert!(
        claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .is_empty(),
        "partial delivery to a subset of a linked group is not a state BE-04 authorizes"
    );
}

#[test]
fn a_current_token_completes_the_job_and_clears_the_last_error() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);

    // A retryable failure first, so there is a last error to clear.
    let first = claim_one(&pool);
    fail_distribution_job(
        &pool,
        job_id,
        first.claim_token,
        "TRANSPORT_FAILURE",
        Some("SFTP handshake rejected by remote host after 3 retries"),
        true,
    )
    .expect("retryable failure");
    assert_eq!(
        reload(&pool, job_id).last_error_code.as_deref(),
        Some("TRANSPORT_FAILURE")
    );

    make_due(&pool, job_id);
    let second = claim_one(&pool);
    let completed = complete_distribution_job(&pool, job_id, second.claim_token).expect("complete");

    assert_eq!(completed.status, DistributionJobStatus::Succeeded);
    assert!(completed.completed_at.is_some());
    assert!(completed.claim_token.is_none());
    assert!(completed.claimed_by.is_none());
    assert!(completed.claimed_at.is_none());
    assert!(completed.lease_expires_at.is_none());
    assert!(
        completed.last_error_code.is_none(),
        "T2 clears last_error_*"
    );
    assert!(completed.last_error_detail.is_none());

    let attempts = attempts_of(&pool, job_id);
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].attempt_number, 2);
    assert_eq!(
        attempts[0].result,
        Some(DistributionJobAttemptResult::Succeeded)
    );
    assert!(attempts[0].error_code.is_none());
    assert_eq!(
        attempts[1].result,
        Some(DistributionJobAttemptResult::Failed)
    );
    assert_eq!(attempts[1].error_code.as_deref(), Some("TRANSPORT_FAILURE"));
}

#[test]
fn a_retryable_failure_returns_the_job_to_pending_with_the_computed_backoff() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);

    for attempt in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        assert_eq!(claim.attempt_number, attempt);
        let job = fail_distribution_job(
            &pool,
            job_id,
            claim.claim_token,
            "TRANSPORT_FAILURE",
            Some("bounded description"),
            true,
        )
        .expect("retryable failure");

        assert_eq!(
            job.status,
            DistributionJobStatus::Pending,
            "T3 at attempt {attempt}"
        );
        assert!(job.completed_at.is_none());
        assert!(job.claim_token.is_none());
        assert_eq!(job.attempt_count, attempt);

        // backoff(n) = min(BASE * 2^(n-1), CAP), as an absolute timestamp.
        let expected_seconds = std::cmp::min(
            DISTRIBUTION_JOB_RETRY_BASE_SECONDS * 2_i64.pow(u32::try_from(attempt - 1).unwrap()),
            DISTRIBUTION_JOB_RETRY_MAX_SECONDS,
        );
        let matched = scalar_count(
            &pool,
            &format!(
                "SELECT count(*) AS count FROM distribution_job \
                 WHERE distribution_job_id = '{job_id}' \
                   AND available_at BETWEEN \
                       CURRENT_TIMESTAMP + interval '{expected_seconds} seconds' - interval '30 seconds' \
                   AND CURRENT_TIMESTAMP + interval '{expected_seconds} seconds' + interval '30 seconds'"
            ),
        );
        assert_eq!(
            matched, 1,
            "attempt {attempt} must schedule the retry {expected_seconds}s ahead"
        );

        assert_eq!(attempts_of(&pool, job_id).len() as i32, attempt);
        make_due(&pool, job_id);
    }

    // The fifth reported failure terminalizes through T4.
    let claim = claim_one(&pool);
    assert_eq!(claim.attempt_number, DISTRIBUTION_JOB_MAX_ATTEMPTS);
    let job = fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "TRANSPORT_FAILURE",
        None,
        true,
    )
    .expect("failure at the budget");
    assert_eq!(
        job.status,
        DistributionJobStatus::Failed,
        "fail(retryable = true) at attempt 5 terminalizes: five means five"
    );
    assert!(job.completed_at.is_some());
    assert_eq!(job.attempt_count, DISTRIBUTION_JOB_MAX_ATTEMPTS);
    assert_eq!(
        attempts_of(&pool, job_id).len() as i32,
        DISTRIBUTION_JOB_MAX_ATTEMPTS
    );
    assert!(claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
        .expect("claim")
        .is_empty());
}

#[test]
fn a_non_retryable_failure_terminalizes_immediately() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);

    let job = fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "PERMANENT_REJECTION",
        Some("destination rejected the deposit as malformed"),
        false,
    )
    .expect("non-retryable failure");

    assert_eq!(job.status, DistributionJobStatus::Failed);
    assert!(job.completed_at.is_some());
    assert_eq!(job.attempt_count, 1);
    assert_eq!(job.last_error_code.as_deref(), Some("PERMANENT_REJECTION"));
    let attempts = attempts_of(&pool, job_id);
    assert_eq!(
        attempts[0].result,
        Some(DistributionJobAttemptResult::Failed)
    );
    assert_eq!(
        attempts[0].error_detail.as_deref(),
        Some("destination rejected the deposit as malformed"),
        "the job's values equal the closing attempt row's values for T3 and T4"
    );
}

#[test]
fn lease_expiry_within_budget_recovers_to_pending_without_moving_the_attempt_count() {
    for attempt_to_reach in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let (_guard, pool) = test_db::setup_test_db();
        let (publisher_id, job_id) = publisher_with_pending_job(&pool);

        // Consume attempts up to the one whose lease will expire.
        for _ in 1..attempt_to_reach {
            let claim = claim_one(&pool);
            fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
                .expect("retryable failure");
            make_due(&pool, job_id);
        }
        let claim = claim_one(&pool);
        assert_eq!(claim.attempt_number, attempt_to_reach);
        expire_lease(&pool, job_id);

        // Recovery happens as step A of the next claim call, observed here in
        // isolation from step B's increment.
        recover_without_reclaim(&pool, publisher_id);

        let job = reload(&pool, job_id);
        assert_eq!(
            job.status,
            DistributionJobStatus::Pending,
            "T5a at attempt {attempt_to_reach}"
        );
        assert_eq!(
            job.attempt_count, attempt_to_reach,
            "T5a neither decrements nor increments the attempt count"
        );
        assert!(job.completed_at.is_none(), "T5a leaves completed_at null");
        assert!(job.claim_token.is_none());
        assert!(job.claimed_by.is_none());
        assert!(job.claimed_at.is_none());
        assert!(job.lease_expires_at.is_none());

        let attempts = attempts_of(&pool, job_id);
        let abandoned = attempts
            .iter()
            .find(|attempt| attempt.attempt_number == attempt_to_reach)
            .expect("the expired attempt");
        assert_eq!(
            abandoned.result,
            Some(DistributionJobAttemptResult::Abandoned),
            "the expired attempt is closed ABANDONED"
        );
        assert!(abandoned.finished_at.is_some());
        assert!(
            abandoned.error_code.is_none(),
            "abandonment reports no worker error"
        );

        // And it is immediately available again: no backoff is applied to work
        // that was orphaned rather than reported failed.
        let reclaimed = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
        assert_eq!(reclaimed.len(), 1, "the recovered job is claimable again");
        assert_eq!(reclaimed[0].attempt_number, attempt_to_reach + 1);
    }
}

#[test]
fn lease_expiry_at_the_budget_terminalizes_and_is_never_claimable_again() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);

    // Reach attempt 5 and let its lease expire.
    for _ in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
            .expect("retryable failure");
        make_due(&pool, job_id);
    }
    let last = claim_one(&pool);
    assert_eq!(last.attempt_number, DISTRIBUTION_JOB_MAX_ATTEMPTS);
    expire_lease(&pool, job_id);

    let claimed = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("claim");
    assert!(
        claimed.is_empty(),
        "T5b terminalizes rather than returning the job to PENDING"
    );

    let job = reload(&pool, job_id);
    assert_eq!(job.status, DistributionJobStatus::Failed);
    assert!(job.completed_at.is_some());
    assert!(job.claim_token.is_none());
    assert!(job.claimed_by.is_none());
    assert!(job.lease_expires_at.is_none());
    assert_eq!(
        job.attempt_count, DISTRIBUTION_JOB_MAX_ATTEMPTS,
        "the count is unchanged"
    );

    let attempts = attempts_of(&pool, job_id);
    assert_eq!(
        attempts[0].result,
        Some(DistributionJobAttemptResult::Abandoned)
    );
    assert!(attempts.iter().all(|attempt| attempt.finished_at.is_some()));

    // Never PENDING, never claimable, on any later call.
    for _ in 0..3 {
        assert!(claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .is_empty());
        assert_eq!(reload(&pool, job_id).status, DistributionJobStatus::Failed);
    }
}

#[test]
fn no_sixth_attempt_exists_on_any_path_including_mixed_failure_and_expiry() {
    // Every combination of a reported failure and a lease expiry, driving one
    // job from attempt 1 through attempt 5, then terminalizing it.
    for pattern in [
        [true, true, true, true],     // fail, fail, fail, fail
        [false, false, false, false], // expire, expire, expire, expire
        [true, false, true, false],
        [false, true, false, true],
    ] {
        let (_guard, pool) = test_db::setup_test_db();
        let (publisher_id, job_id) = publisher_with_pending_job(&pool);

        let mut claim = claim_one(&pool);
        assert_eq!(claim.attempt_number, 1);

        for (index, reported_failure) in pattern.into_iter().enumerate() {
            if reported_failure {
                fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
                    .expect("retryable failure");
                make_due(&pool, job_id);
            } else {
                expire_lease(&pool, job_id);
                recover_without_reclaim(&pool, publisher_id);
                assert_eq!(
                    reload(&pool, job_id).attempt_count as usize,
                    index + 1,
                    "recovery consumes no attempt of its own"
                );
            }
            claim = claim_one(&pool);
            assert_eq!(claim.attempt_number as usize, index + 2);
        }

        // Attempt 5 is running. Terminalize it by lease expiry, which is T5b.
        assert_eq!(claim.attempt_number, DISTRIBUTION_JOB_MAX_ATTEMPTS);
        expire_lease(&pool, job_id);
        let after = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery");
        assert!(after.is_empty(), "T5b never returns the job to PENDING");

        let job = reload(&pool, job_id);
        assert_eq!(
            job.attempt_count, DISTRIBUTION_JOB_MAX_ATTEMPTS,
            "pattern {pattern:?}"
        );
        assert_eq!(job.status, DistributionJobStatus::Failed);

        let max_attempt = scalar_count(
            &pool,
            &format!(
                "SELECT coalesce(max(attempt_number), 0)::bigint AS count FROM distribution_job_attempt \
                 WHERE distribution_job_id = '{job_id}'"
            ),
        );
        assert_eq!(
            max_attempt,
            i64::from(DISTRIBUTION_JOB_MAX_ATTEMPTS),
            "no attempt_number = 6 exists on any route"
        );
        assert_eq!(
            attempts_of(&pool, job_id).len() as i32,
            DISTRIBUTION_JOB_MAX_ATTEMPTS
        );

        // And the database itself refuses a sixth, whatever the application did.
        let mut connection = pool.get().expect("connection");
        assert!(sql_query(format!(
            "UPDATE distribution_job SET attempt_count = 6 WHERE distribution_job_id = '{job_id}'"
        ))
        .execute(&mut connection)
        .is_err());
    }
}

#[test]
fn two_workers_racing_recovery_preserve_exactly_one_transition() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    expire_lease(&pool, job_id);

    let handles: Vec<_> = (0..4)
        .map(|index| {
            let pool = pool.clone();
            thread::spawn(move || {
                claim_distribution_jobs(&pool, &format!("racer-{index}"), 10, 900, &[])
                    .expect("claim")
            })
        })
        .collect();
    let claims: usize = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread").len())
        .sum();

    assert!(claims <= 1, "the recovered job may be claimed at most once");
    let attempts = attempts_of(&pool, job_id);
    assert_eq!(
        attempts
            .iter()
            .filter(|attempt| attempt.result == Some(DistributionJobAttemptResult::Abandoned))
            .count(),
        1,
        "exactly one closed abandoned attempt, and no duplicate attempt row"
    );
    assert_eq!(
        attempts
            .iter()
            .filter(|attempt| attempt.claim_token == claim.claim_token)
            .count(),
        1
    );
    let numbers: HashSet<i32> = attempts
        .iter()
        .map(|attempt| attempt.attempt_number)
        .collect();
    assert_eq!(numbers.len(), attempts.len(), "attempt ordinals are unique");
}

// --------------------------------------------------------------------------
// 12.6  Stale claim tokens
// --------------------------------------------------------------------------

#[test]
fn a_stale_token_can_neither_complete_nor_fail_nor_retry_a_newer_attempt() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);

    let stale = claim_one(&pool);
    expire_lease(&pool, job_id);
    let fresh = claim_distribution_jobs(&pool, "second-worker", 10, 900, &[])
        .expect("recovery + re-claim")
        .remove(0);
    assert_ne!(stale.claim_token, fresh.claim_token);

    let before = reload(&pool, job_id);
    let before_attempts = attempts_of(&pool, job_id);

    // complete, fail (terminal) and fail (retryable) are all refused.
    assert!(matches!(
        complete_distribution_job(&pool, job_id, stale.claim_token),
        Err(ThothError::StaleDistributionJobClaim)
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, stale.claim_token, "STALE", None, false),
        Err(ThothError::StaleDistributionJobClaim)
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, stale.claim_token, "STALE", None, true),
        Err(ThothError::StaleDistributionJobClaim)
    ));

    let after = reload(&pool, job_id);
    assert_eq!(before, after, "the newer attempt's job row is untouched");
    assert_eq!(
        after.claim_token,
        Some(fresh.claim_token),
        "worker identity and lease are only ever written by the claim statement"
    );
    assert_eq!(after.claimed_by.as_deref(), Some("second-worker"));
    assert_eq!(before_attempts, attempts_of(&pool, job_id));

    // The current holder still succeeds.
    complete_distribution_job(&pool, job_id, fresh.claim_token).expect("the live claim works");

    // A terminal row cannot be mutated by any token.
    assert!(matches!(
        complete_distribution_job(&pool, job_id, fresh.claim_token),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "SUCCEEDED"
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, stale.claim_token, "STALE", None, true),
        Err(ThothError::DistributionJobAlreadyTerminal(_))
    ));
}

#[test]
fn the_token_of_an_abandoned_attempt_is_stale_in_both_recovery_branches() {
    // T5a: recovered to PENDING within budget.
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    expire_lease(&pool, job_id);
    // One call recovers the orphaned attempt and re-claims the job, so the old
    // token now addresses a job whose current token is a different one.
    let recovered = claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery + claim");
    assert_eq!(recovered.len(), 1);
    assert_ne!(recovered[0].claim_token, claim.claim_token);
    assert!(matches!(
        complete_distribution_job(&pool, job_id, claim.claim_token),
        Err(ThothError::StaleDistributionJobClaim)
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, claim.claim_token, "STALE", None, true),
        Err(ThothError::StaleDistributionJobClaim)
    ));

    // T5b: terminalized at the budget.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    for _ in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
            .expect("retryable failure");
        make_due(&pool, job_id);
    }
    let last = claim_one(&pool);
    expire_lease(&pool, job_id);
    claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery");
    assert_eq!(reload(&pool, job_id).status, DistributionJobStatus::Failed);
    assert!(matches!(
        complete_distribution_job(&pool, job_id, last.claim_token),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "FAILED"
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, last.claim_token, "STALE", None, true),
        Err(ThothError::DistributionJobAlreadyTerminal(_))
    ));
}

#[test]
fn the_repeat_call_matrix_of_section_13_3_holds() {
    // complete then complete.
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    complete_distribution_job(&pool, job_id, claim.claim_token).expect("first complete");
    let before = reload(&pool, job_id);
    assert!(matches!(
        complete_distribution_job(&pool, job_id, claim.claim_token),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "SUCCEEDED"
    ));
    // complete then fail.
    assert!(matches!(
        fail_distribution_job(&pool, job_id, claim.claim_token, "LATE", None, true),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "SUCCEEDED"
    ));
    assert_eq!(before, reload(&pool, job_id), "no state change");
    assert_eq!(attempts_of(&pool, job_id).len(), 1, "no second attempt row");

    // fail(retryable = false) then fail.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    fail_distribution_job(&pool, job_id, claim.claim_token, "PERMANENT", None, false)
        .expect("terminal failure");
    assert!(matches!(
        fail_distribution_job(&pool, job_id, claim.claim_token, "PERMANENT", None, false),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "FAILED"
    ));

    // fail(retryable = true) then fail: the job is PENDING with a null token.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
        .expect("retryable failure");
    let before = reload(&pool, job_id);
    assert_eq!(before.status, DistributionJobStatus::Pending);
    assert!(before.claim_token.is_none());
    assert!(matches!(
        fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true),
        Err(ThothError::StaleDistributionJobClaim)
    ));
    assert_eq!(before, reload(&pool, job_id));
    assert_eq!(attempts_of(&pool, job_id).len(), 1);
}

#[test]
fn operating_on_an_absent_job_reports_entity_not_found() {
    let (_guard, pool) = test_db::setup_test_db();
    let absent = Uuid::new_v4();
    assert!(matches!(
        complete_distribution_job(&pool, absent, Uuid::new_v4()),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        fail_distribution_job(&pool, absent, Uuid::new_v4(), "GONE", None, true),
        Err(ThothError::EntityNotFound)
    ));
    assert!(matches!(
        cancel_distribution_job(&pool, absent),
        Err(ThothError::EntityNotFound)
    ));
}

#[test]
fn completing_or_failing_a_pending_job_reports_a_stale_claim() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    // `PENDING` and "held by another worker" deliberately produce the same
    // error: distinguishing them would tell a caller whether someone else holds
    // the job.
    assert!(matches!(
        complete_distribution_job(&pool, job_id, Uuid::new_v4()),
        Err(ThothError::StaleDistributionJobClaim)
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, Uuid::new_v4(), "NOPE", None, true),
        Err(ThothError::StaleDistributionJobClaim)
    ));
}

// --------------------------------------------------------------------------
// 11.2  `last_error_*` semantics, as the six fixed cases
// --------------------------------------------------------------------------

#[test]
fn last_error_holds_the_most_recent_worker_reported_failure_and_nothing_else() {
    // 1. T5a after a previous worker-reported failure: unchanged.
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let first = claim_one(&pool);
    fail_distribution_job(
        &pool,
        job_id,
        first.claim_token,
        "TRANSPORT_FAILURE",
        Some("earlier reported failure"),
        true,
    )
    .expect("retryable failure");
    make_due(&pool, job_id);
    let second = claim_one(&pool);
    assert_eq!(second.attempt_number, 2);
    expire_lease(&pool, job_id);
    claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery + claim");
    let job = reload(&pool, job_id);
    assert_eq!(job.last_error_code.as_deref(), Some("TRANSPORT_FAILURE"));
    assert_eq!(
        job.last_error_detail.as_deref(),
        Some("earlier reported failure")
    );
    let abandoned = attempts_of(&pool, job_id)
        .into_iter()
        .find(|attempt| attempt.attempt_number == 2)
        .expect("attempt 2");
    assert_eq!(
        abandoned.result,
        Some(DistributionJobAttemptResult::Abandoned)
    );
    assert!(
        abandoned.error_code.is_none(),
        "an abandoned attempt has no error of its own"
    );

    // 2. T5a with no previous reported failure: still null.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    claim_one(&pool);
    expire_lease(&pool, job_id);
    claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery + claim");
    let job = reload(&pool, job_id);
    assert!(job.last_error_code.is_none());
    assert!(job.last_error_detail.is_none());

    // 3. T5b after a previous reported failure: the earlier value is retained,
    //    and it is demonstrably not the cause of terminalization.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    for index in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        fail_distribution_job(
            &pool,
            job_id,
            claim.claim_token,
            "TRANSPORT_FAILURE",
            Some(&format!("reported at attempt {index}")),
            true,
        )
        .expect("retryable failure");
        make_due(&pool, job_id);
    }
    claim_one(&pool);
    expire_lease(&pool, job_id);
    claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery");
    let job = reload(&pool, job_id);
    assert_eq!(job.status, DistributionJobStatus::Failed);
    assert_eq!(job.last_error_code.as_deref(), Some("TRANSPORT_FAILURE"));
    assert_eq!(
        job.last_error_detail.as_deref(),
        Some("reported at attempt 4"),
        "the last thing a worker actually reported — not the reason the final \
         attempt was abandoned"
    );
    assert_eq!(
        attempts_of(&pool, job_id)[0].result,
        Some(DistributionJobAttemptResult::Abandoned),
        "attempt history is the authoritative record of how the job ended"
    );

    // 4. T5b with no previous reported failure: null on a FAILED job, and
    //    nothing synthesizes a placeholder.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (publisher_id, job_id) = publisher_with_pending_job(&pool);
    for expected in 1..=DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        assert_eq!(claim.attempt_number, expected);
        expire_lease(&pool, job_id);
        recover_without_reclaim(&pool, publisher_id);
    }
    let job = reload(&pool, job_id);
    assert_eq!(job.status, DistributionJobStatus::Failed);
    assert!(
        job.last_error_code.is_none() && job.last_error_detail.is_none(),
        "a FAILED job no worker ever reported a failure for legitimately has a \
         null lastError; that is correct, not missing data"
    );
    assert!(attempts_of(&pool, job_id)
        .iter()
        .all(|attempt| attempt.result == Some(DistributionJobAttemptResult::Abandoned)));

    // 5 and 6 (success clears; cancellation leaves alone) are covered by
    // `a_current_token_completes_the_job_and_clears_the_last_error` and by the
    // cancellation tests below.
}

// ==========================================================================
// 25.9  Cancellation and assignment-interaction tests
// ==========================================================================

#[test]
fn cancellation_from_every_state_of_the_section_14_2_table() {
    // PENDING, never claimed (T6).
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let cancelled = cancel_distribution_job(&pool, job_id).expect("cancel a pending job");
    assert_eq!(cancelled.status, DistributionJobStatus::Cancelled);
    assert!(cancelled.completed_at.is_some());
    assert_eq!(
        cancelled.cancellation_reason,
        Some(DistributionJobCancellationReason::Administrative)
    );
    assert!(
        attempts_of(&pool, job_id).is_empty(),
        "no open attempt existed to close"
    );
    assert_eq!(
        targets_of(&pool, job_id).len(),
        1,
        "target rows are immutable"
    );

    // PENDING after a retry (T6): earlier closed attempts are untouched.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "TRANSIENT",
        Some("earlier"),
        true,
    )
    .expect("retryable failure");
    let before = attempts_of(&pool, job_id);
    let cancelled = cancel_distribution_job(&pool, job_id).expect("cancel after a retry");
    assert_eq!(cancelled.status, DistributionJobStatus::Cancelled);
    assert_eq!(
        before,
        attempts_of(&pool, job_id),
        "earlier attempts untouched"
    );
    assert_eq!(
        cancelled.last_error_code.as_deref(),
        Some("TRANSIENT"),
        "cancellation neither sets nor clears last_error_*"
    );
    assert_eq!(cancelled.last_error_detail.as_deref(), Some("earlier"));

    // RUNNING with a live lease (T7).
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    let cancelled = cancel_distribution_job(&pool, job_id).expect("cancel a running job");
    assert_eq!(cancelled.status, DistributionJobStatus::Cancelled);
    assert!(
        cancelled.claim_token.is_none(),
        "the holder's token is invalidated"
    );
    assert!(cancelled.claimed_by.is_none());
    assert!(cancelled.lease_expires_at.is_none());
    assert!(
        cancelled.last_error_code.is_none(),
        "a job that never failed keeps nulls"
    );
    let attempts = attempts_of(&pool, job_id);
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        attempts[0].result,
        Some(DistributionJobAttemptResult::Cancelled),
        "the attempt's result names the event that closed it"
    );
    // The in-flight worker's next call is rejected as stale, and it is not
    // notified.
    assert!(matches!(
        complete_distribution_job(&pool, job_id, claim.claim_token),
        Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == "CANCELLED"
    ));
    assert!(matches!(
        fail_distribution_job(&pool, job_id, claim.claim_token, "LATE", None, true),
        Err(ThothError::DistributionJobAlreadyTerminal(_))
    ));

    // RUNNING with an expired lease (T7): identical handling, and the attempt is
    // closed CANCELLED rather than ABANDONED, because cancellation is what
    // closed it.
    test_db::reset_db(&pool).expect("reset the disposable database");
    let (_p, job_id) = publisher_with_pending_job(&pool);
    claim_one(&pool);
    expire_lease(&pool, job_id);
    let cancelled = cancel_distribution_job(&pool, job_id).expect("cancel an expired running job");
    assert_eq!(cancelled.status, DistributionJobStatus::Cancelled);
    assert_eq!(
        attempts_of(&pool, job_id)[0].result,
        Some(DistributionJobAttemptResult::Cancelled)
    );

    // Every terminal state is rejected, including an already-CANCELLED job.
    for terminal in ["SUCCEEDED", "FAILED", "CANCELLED"] {
        test_db::reset_db(&pool).expect("reset the disposable database");
        let (_p, job_id) = publisher_with_pending_job(&pool);
        match terminal {
            "SUCCEEDED" => {
                let claim = claim_one(&pool);
                complete_distribution_job(&pool, job_id, claim.claim_token).expect("complete");
            }
            "FAILED" => {
                let claim = claim_one(&pool);
                fail_distribution_job(&pool, job_id, claim.claim_token, "PERMANENT", None, false)
                    .expect("fail");
            }
            _ => {
                cancel_distribution_job(&pool, job_id).expect("cancel");
            }
        }
        let before = reload(&pool, job_id);
        let before_attempts = attempts_of(&pool, job_id);
        assert!(
            matches!(
                cancel_distribution_job(&pool, job_id),
                Err(ThothError::DistributionJobAlreadyTerminal(ref status)) if status == terminal
            ),
            "cancelling a {terminal} job must fail closed rather than report a \
             comfortable idempotent success"
        );
        assert_eq!(before, reload(&pool, job_id), "nothing changed");
        assert_eq!(before_attempts, attempts_of(&pool, job_id));
    }
}

#[test]
fn a_cancelled_job_cannot_be_reopened_retried_or_re_claimed_and_loses_no_history() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);
    fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
        .expect("retryable failure");
    make_due(&pool, job_id);
    let second = claim_one(&pool);
    cancel_distribution_job(&pool, job_id).expect("cancel");

    make_due(&pool, job_id);
    assert!(
        claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
            .expect("claim")
            .is_empty(),
        "a cancelled job is never claimed again"
    );
    assert!(matches!(
        fail_distribution_job(&pool, job_id, second.claim_token, "LATE", None, true),
        Err(ThothError::DistributionJobAlreadyTerminal(_))
    ));

    assert_eq!(
        attempts_of(&pool, job_id).len(),
        2,
        "attempt history survives"
    );
    assert_eq!(targets_of(&pool, job_id).len(), 1, "target rows survive");
    assert_eq!(
        scalar_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
        1,
        "cancellation deletes no job row"
    );
}

#[test]
fn disabling_an_assignment_cancels_its_pending_jobs_and_leaves_others_alone() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    let other = test_db::create_publisher(&pool);

    activate(
        &pool,
        publisher_id,
        &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
    );
    activate(&pool, other.publisher_id, &[DistributionPlatform::Zenodo]);

    let jobs = jobs_of(&pool, publisher_id);
    assert_eq!(jobs.len(), 2);
    let linked = jobs
        .iter()
        .find(|job| {
            targets_of(&pool, job.distribution_job_id).contains(&DistributionPlatform::Oapen)
        })
        .expect("the linked job")
        .clone();
    let independent = jobs
        .iter()
        .find(|job| job.distribution_job_id != linked.distribution_job_id)
        .expect("the independent job")
        .clone();
    let other_job = only_job(&pool, other.publisher_id);

    // Disable only the linked group, in the same transaction as the withdrawal.
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);

    let linked_after = reload(&pool, linked.distribution_job_id);
    assert_eq!(linked_after.status, DistributionJobStatus::Cancelled);
    assert_eq!(
        linked_after.cancellation_reason,
        Some(DistributionJobCancellationReason::AssignmentDisabled),
        "ASSIGNMENT_DISABLED distinguishes this from an operator's decision"
    );
    assert!(linked_after.completed_at.is_some());
    assert!(
        linked_after.last_error_code.is_none(),
        "T8 leaves last_error_* alone"
    );

    assert_eq!(
        reload(&pool, independent.distribution_job_id).status,
        DistributionJobStatus::Pending,
        "a PENDING job for a different group of the same publisher is untouched"
    );
    assert_eq!(
        reload(&pool, other_job.distribution_job_id).status,
        DistributionJobStatus::Pending,
        "another publisher's jobs are never touched"
    );
}

#[test]
fn disabling_an_assignment_leaves_a_running_job_alone_but_unclaimable_after_expiry() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let job_id = only_job(&pool, publisher_id).distribution_job_id;
    let claim = claim_one(&pool);

    // Withdraw the assignment while the job is RUNNING.
    activate(&pool, publisher_id, &[]);
    let running = reload(&pool, job_id);
    assert_eq!(
        running.status,
        DistributionJobStatus::Running,
        "a RUNNING job is not touched: external work may be in flight, and \
         cancelling cannot undo an upload"
    );
    assert_eq!(running.claim_token, Some(claim.claim_token));

    // The claiming worker can still terminalize its own attempt.
    fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
        .expect("the holder may still report");
    assert_eq!(reload(&pool, job_id).status, DistributionJobStatus::Pending);

    // Back in PENDING, eligibility makes it unclaimable — it waits visibly for
    // an operator rather than silently resuming.
    make_due(&pool, job_id);
    assert!(claim_distribution_jobs(&pool, WORKER, 10, 900, &[])
        .expect("claim")
        .is_empty());
    assert_eq!(reload(&pool, job_id).status, DistributionJobStatus::Pending);

    // And a superuser can still cancel it explicitly.
    let cancelled = cancel_distribution_job(&pool, job_id).expect("explicit cancellation");
    assert_eq!(
        cancelled.cancellation_reason,
        Some(DistributionJobCancellationReason::Administrative)
    );
}

#[test]
fn a_running_job_with_an_exhausted_budget_terminalizes_after_a_withdrawal() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let publisher_id = publisher.publisher_id;
    activate(&pool, publisher_id, &[DistributionPlatform::Zenodo]);
    let job_id = only_job(&pool, publisher_id).distribution_job_id;

    for _ in 1..DISTRIBUTION_JOB_MAX_ATTEMPTS {
        let claim = claim_one(&pool);
        fail_distribution_job(&pool, job_id, claim.claim_token, "TRANSIENT", None, true)
            .expect("retryable failure");
        make_due(&pool, job_id);
    }
    claim_one(&pool);
    activate(&pool, publisher_id, &[]);
    assert_eq!(reload(&pool, job_id).status, DistributionJobStatus::Running);

    expire_lease(&pool, job_id);
    claim_distribution_jobs(&pool, WORKER, 10, 900, &[]).expect("recovery");
    assert_eq!(
        reload(&pool, job_id).status,
        DistributionJobStatus::Failed,
        "an exhausted budget terminalizes directly through T5b"
    );
}

// ==========================================================================
// 25.10  Error storage and sanitization
// ==========================================================================

#[test]
fn an_over_length_detail_is_truncated_on_a_character_boundary() {
    let long = "é".repeat(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS + 500);
    let sanitized = sanitize_error_detail(&long).expect("detail");
    assert_eq!(
        sanitized.chars().count(),
        DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS,
        "exactly the first 2048 Unicode scalar values"
    );
    assert!(
        std::str::from_utf8(sanitized.as_bytes()).is_ok(),
        "never a byte slice, so no partial UTF-8 sequence can be produced"
    );
    assert!(sanitized.chars().all(|character| character == 'é'));

    // A mixed multi-byte input truncates safely too.
    let mixed: String = "aé漢🙂".chars().cycle().take(5_000).collect();
    let sanitized = sanitize_error_detail(&mixed).expect("detail");
    assert_eq!(
        sanitized.chars().count(),
        DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS
    );
    assert!(mixed.starts_with(&sanitized));
}

#[test]
fn control_characters_are_removed_and_whitespace_trimmed() {
    let raw = "  \u{0}line one\nline\ttwo\u{7}\u{1b}[31mred\u{1b}[0m  \r\n";
    let sanitized = sanitize_error_detail(raw).expect("detail");
    assert_eq!(sanitized, "line one\nline\ttwo[31mred[0m");
    assert!(sanitized.contains('\n'), "newline is retained");
    assert!(sanitized.contains('\t'), "tab is retained");
    assert!(!sanitized.contains('\u{0}'));
    assert!(!sanitized.contains('\u{7}'));
    assert!(!sanitized.contains('\u{1b}'));
    assert!(!sanitized.contains('\r'));

    // A detail that sanitizes to nothing is stored as nothing.
    assert!(sanitize_error_detail("   \u{0}\u{7}  ").is_none());
    assert!(sanitize_error_detail("").is_none());
}

#[test]
fn error_codes_are_validated_rather_than_truncated() {
    for valid in [
        "TRANSPORT_FAILURE",
        "A",
        "A1",
        "PERMANENT_REJECTION_2",
        &"A".repeat(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS),
    ] {
        assert!(
            validate_error_code(valid).is_ok(),
            "`{valid}` must be accepted"
        );
    }
    for invalid in [
        "",
        "lower_case",
        "_LEADING_UNDERSCORE",
        "9LEADING_DIGIT",
        "HAS SPACE",
        "HAS-HYPHEN",
        "HAS.DOT",
        "TRAILING ",
        "ÉACCENT",
        &"A".repeat(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS + 1),
    ] {
        assert!(
            matches!(
                validate_error_code(invalid),
                Err(ThothError::InvalidDistributionJobErrorCode)
            ),
            "`{invalid}` must be rejected"
        );
    }
}

#[test]
fn a_rejected_error_code_changes_no_state_and_leaves_the_claim_token_valid() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);

    let before = reload(&pool, job_id);
    let before_attempts = attempts_of(&pool, job_id);

    for invalid in [
        "not a code",
        &"A".repeat(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS + 1),
    ] {
        let outcome = fail_distribution_job(
            &pool,
            job_id,
            claim.claim_token,
            invalid,
            Some("a detail"),
            true,
        );
        assert!(matches!(
            outcome,
            Err(ThothError::InvalidDistributionJobErrorCode)
        ));
        // The public message is a fixed string that echoes no part of the value.
        let message = ThothError::InvalidDistributionJobErrorCode.to_string();
        assert_eq!(
            message,
            "The supplied distribution job error code is not a valid classification code."
        );
        assert!(!message.contains(invalid));
        assert!(!message.contains(&invalid.len().to_string()));
        assert!(!message.contains("A-Z"));

        assert_eq!(before, reload(&pool, job_id), "no job state changed");
        assert_eq!(
            before_attempts,
            attempts_of(&pool, job_id),
            "no attempt state changed"
        );
        assert!(
            attempts_of(&pool, job_id)[0].finished_at.is_none(),
            "the attempt stays open"
        );
    }

    // The claim token is still valid: a conforming resubmission succeeds under
    // the same token.
    let job = fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "TRANSPORT_FAILURE",
        Some("a detail"),
        true,
    )
    .expect("a conforming code succeeds under the same token");
    assert_eq!(job.last_error_code.as_deref(), Some("TRANSPORT_FAILURE"));
}

#[test]
fn a_worker_reported_detail_is_sanitized_before_storage() {
    let (_guard, pool) = test_db::setup_test_db();
    let (_p, job_id) = publisher_with_pending_job(&pool);
    let claim = claim_one(&pool);

    let raw = format!(
        "  \u{0}SFTP handshake rejected\u{7}\n{}  ",
        "é".repeat(4_000)
    );
    let job = fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "TRANSPORT_FAILURE",
        Some(&raw),
        false,
    )
    .expect("failure");

    let stored = job.last_error_detail.expect("detail");
    assert_eq!(
        stored.chars().count(),
        DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS
    );
    assert!(!stored.contains('\u{0}'));
    assert!(stored.starts_with("SFTP handshake rejected\n"));
    // The database check still holds for the stored value.
    assert_eq!(
        attempts_of(&pool, job_id)[0].error_detail.as_deref(),
        Some(stored.as_str()),
        "the attempt row carries the same sanitized value"
    );
}

// ==========================================================================
// 25.3  Migration tests, including observed locking on the referenced tables
// ==========================================================================

/// The exact `up.sql` of the `BE-04` migration, so the locking and contention
/// fixtures run the migration's own DDL rather than an approximation of it.
const MIGRATION_UP_SQL: &str = include_str!("../../../migrations/20260814_v1.7.0/up.sql");
const MIGRATION_VERSION: &str = "20260814";

/// A throwaway database, so migration tests never disturb the shared test
/// database and never point at anything shared.
struct TempMigrationDb {
    admin_url: String,
    name: String,
}

impl TempMigrationDb {
    fn new() -> Self {
        let admin_url = test_db::test_db_url();
        let name = format!("thoth_be04_{}", Uuid::new_v4().simple());
        let mut admin = PgConnection::establish(&admin_url).expect("admin connection");
        admin
            .batch_execute(&format!("CREATE DATABASE \"{name}\""))
            .expect("create temp db");
        TempMigrationDb { admin_url, name }
    }

    fn url(&self) -> String {
        let (prefix, _) = self.admin_url.rsplit_once('/').expect("db url has a path");
        format!("{prefix}/{}", self.name)
    }

    fn conn(&self) -> PgConnection {
        PgConnection::establish(&self.url()).expect("temp db connection")
    }

    fn pool(&self) -> PgPool {
        diesel::r2d2::Pool::builder()
            .max_size(4)
            .build(diesel::r2d2::ConnectionManager::<PgConnection>::new(
                self.url(),
            ))
            .expect("temp pool")
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

/// Revert migrations until the `BE-04` migration itself has been reverted.
fn revert_through_be04(connection: &mut PgConnection) {
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("revert migration");
        if reverted.to_string() == MIGRATION_VERSION {
            return;
        }
    }
}

/// Publishers with imprints and works, enabled and disabled assignments
/// including a linked OAPEN/DOAB pair, and configuration audit rows.
fn seed_representative_state(pool: &PgPool) {
    let first = test_db::create_publisher(pool);
    let imprint = test_db::create_imprint(pool, &first);
    for _ in 0..5 {
        test_db::create_work(pool, &imprint);
    }
    let second = test_db::create_publisher(pool);
    let second_imprint = test_db::create_imprint(pool, &second);
    test_db::create_work(pool, &second_imprint);

    // A linked pair, an independent destination, and a retained disabled row.
    replace(
        pool,
        &write_context(DistributionJobCreation::Off),
        first.publisher_id,
        ThothPackage::Sphinx,
        &[DistributionPlatform::OclcKb],
    )
    .expect("a PullFeed activation commits under OFF");
    let mut connection = pool.get().expect("connection");
    let activation = Uuid::new_v4();
    for platform in ["OAPEN", "DOAB"] {
        sql_query(format!(
            "INSERT INTO publisher_distribution_platform \
             (publisher_id, platform, enabled, activation_id, enabled_at) \
             VALUES ('{}', '{platform}', true, '{activation}', now())",
            first.publisher_id
        ))
        .execute(&mut connection)
        .expect("seed linked pair");
    }
    sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at, disabled_at) \
         VALUES ('{}', 'ZENODO', false, '{}', now(), now())",
        second.publisher_id,
        Uuid::new_v4()
    ))
    .execute(&mut connection)
    .expect("seed retained disabled row");
}

fn relfilenodes(connection: &mut PgConnection) -> Vec<String> {
    sql_query(
        "SELECT relname || ' ' || relfilenode::text AS value FROM pg_class \
         WHERE relname IN ('publisher', 'publisher_distribution_platform', \
                           'publisher_service_configuration_history', 'work') \
         ORDER BY relname",
    )
    .load::<TextRow>(connection)
    .expect("relfilenodes")
    .into_iter()
    .map(|row| row.value)
    .collect()
}

#[test]
fn the_migration_directory_sorts_after_every_existing_one() {
    let migrations = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"));
    let mut names: Vec<String> = std::fs::read_dir(migrations)
        .expect("migrations directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();

    let ours = names
        .iter()
        .find(|name| name.starts_with(MIGRATION_VERSION))
        .expect("the BE-04 migration directory")
        .clone();
    assert_eq!(
        names.last().expect("at least one migration"),
        &ours,
        "the embedded runner applies directories in lexicographic name order, so \
         this migration must sort after every existing one. `make migration` \
         derives the version from Cargo.toml rather than from the existing \
         directories, which is why this is checked rather than assumed."
    );
    assert_eq!(ours, "20260814_v1.7.0");
}

#[test]
fn the_migration_applies_reverts_and_re_applies_on_an_empty_database() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations apply on an empty database");
    let pool = db.pool();
    assert_eq!(
        catalog_values(
            &pool,
            "SELECT table_name AS value FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'distribution_job%' \
             ORDER BY table_name"
        ),
        vec![
            "distribution_job",
            "distribution_job_attempt",
            "distribution_job_target"
        ]
    );

    revert_through_be04(&mut connection);
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'distribution_job%'"
        ),
        0,
        "the down migration drops all three relations"
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM pg_type WHERE typname LIKE 'distribution_job%'"
        ),
        0,
        "and all four enum types"
    );

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations re-apply cleanly");
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'distribution_job%'"
        ),
        3
    );
}

#[test]
fn the_migration_changes_no_existing_row_and_rewrites_no_existing_table() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations");
    let pool = db.pool();
    seed_representative_state(&pool);

    // Revert only BE-04, so the forward run under observation is BE-04's alone.
    revert_through_be04(&mut connection);

    let assignments_before = catalog_values(
        &pool,
        "SELECT publisher_id::text || ' ' || platform::text || ' ' || enabled::text \
         || ' ' || activation_id::text || ' ' || enabled_at::text AS value \
         FROM publisher_distribution_platform ORDER BY publisher_id, platform",
    );
    let publishers_before = catalog_values(
        &pool,
        "SELECT publisher_id::text || ' ' || subscription_package::text || ' ' \
         || service_configuration_updated_at::text AS value FROM publisher ORDER BY publisher_id",
    );
    let audit_before = scalar_count(
        &pool,
        "SELECT count(*) AS count FROM publisher_service_configuration_history",
    );
    let works_before = catalog_values(
        &pool,
        "SELECT work_id::text || ' ' || updated_at_with_relations::text AS value \
         FROM work ORDER BY work_id",
    );
    let relfilenodes_before = relfilenodes(&mut connection);

    let started = Instant::now();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("forward migration on a representative populated database");
    let elapsed = started.elapsed();

    // Zero rows created by the migration itself.
    for table in [
        "distribution_job",
        "distribution_job_target",
        "distribution_job_attempt",
    ] {
        assert_eq!(
            scalar_count(&pool, &format!("SELECT count(*) AS count FROM {table}")),
            0,
            "{table} must be created empty: a job exists only because a future \
             activation created it"
        );
    }

    // Every existing row is unchanged.
    assert_eq!(
        assignments_before,
        catalog_values(
            &pool,
            "SELECT publisher_id::text || ' ' || platform::text || ' ' || enabled::text \
             || ' ' || activation_id::text || ' ' || enabled_at::text AS value \
             FROM publisher_distribution_platform ORDER BY publisher_id, platform"
        )
    );
    assert_eq!(
        publishers_before,
        catalog_values(
            &pool,
            "SELECT publisher_id::text || ' ' || subscription_package::text || ' ' \
             || service_configuration_updated_at::text AS value FROM publisher ORDER BY publisher_id"
        )
    );
    assert_eq!(
        audit_before,
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM publisher_service_configuration_history"
        )
    );
    assert_eq!(
        works_before,
        catalog_values(
            &pool,
            "SELECT work_id::text || ' ' || updated_at_with_relations::text AS value \
             FROM work ORDER BY work_id"
        )
    );

    // No rewrite of any referenced or neighbouring table, despite the locks the
    // two foreign keys take.
    assert_eq!(
        relfilenodes_before,
        relfilenodes(&mut connection),
        "pg_class.relfilenode must be unchanged for publisher, \
         publisher_distribution_platform, publisher_service_configuration_history \
         and work"
    );

    // Recorded as a disposable-environment measurement. It is **not** a
    // production duration prediction and none may be extrapolated from it.
    println!(
        "BE-04 migration forward duration on a representative populated \
         disposable database: {elapsed:?} (disposable-environment measurement only)"
    );
    assert!(
        elapsed < Duration::from_secs(60),
        "observed duration {elapsed:?}"
    );
}

#[test]
fn the_migration_takes_share_row_exclusive_locks_on_publisher_and_work() {
    use std::sync::mpsc;

    let db = TempMigrationDb::new();
    let mut setup = db.conn();
    setup
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations");
    let pool = db.pool();
    seed_representative_state(&pool);
    revert_through_be04(&mut setup);

    let (ready_tx, ready_rx) = mpsc::channel::<i32>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let url = db.url();

    // Session A runs the migration's own DDL inside a transaction it holds open,
    // so the lock set is observable rather than raced for.
    let migrator = thread::spawn(move || {
        let mut connection = PgConnection::establish(&url).expect("migration connection");
        connection.batch_execute("BEGIN").expect("begin");
        let pid = sql_query("SELECT pg_backend_pid() AS value")
            .get_result::<PidRow>(&mut connection)
            .expect("backend pid")
            .value;
        connection
            .batch_execute(MIGRATION_UP_SQL)
            .expect("migration DDL");
        ready_tx.send(pid).expect("signal ready");
        release_rx.recv().expect("await release");
        connection.batch_execute("COMMIT").expect("commit");
    });

    let pid = ready_rx.recv().expect("migration pid");

    // Session B observes `pg_locks` joined to `pg_class` while the migration
    // transaction is still open.
    let mut observer = db.conn();
    let observed: Vec<String> = sql_query(format!(
        "SELECT c.relname || ' ' || l.mode AS value \
         FROM pg_locks l JOIN pg_class c ON c.oid = l.relation \
         WHERE l.pid = {pid} AND l.locktype = 'relation' \
           AND c.relname IN ('publisher', 'work') \
         ORDER BY c.relname, l.mode"
    ))
    .load::<TextRow>(&mut observer)
    .expect("pg_locks")
    .into_iter()
    .map(|row| row.value)
    .collect();

    release_tx.send(()).expect("release");
    migrator.join().expect("migration thread");

    println!("BE-04 migration observed pg_locks on referenced tables: {observed:?}");
    assert!(
        observed
            .iter()
            .any(|entry| entry == "publisher ShareRowExclusiveLock"),
        "establishing distribution_job_publisher_id_fkey must be observed taking \
         SHARE ROW EXCLUSIVE on public.publisher. Observed: {observed:?}"
    );
    assert!(
        observed
            .iter()
            .any(|entry| entry == "work ShareRowExclusiveLock"),
        "establishing distribution_job_work_id_fkey must be observed taking \
         SHARE ROW EXCLUSIVE on public.work. Observed: {observed:?}"
    );
    assert!(
        !observed
            .iter()
            .any(|entry| entry.ends_with("AccessExclusiveLock")),
        "the migration must not take ACCESS EXCLUSIVE on a referenced table, \
         which would block readers as well as writers. Observed: {observed:?}"
    );
}

#[derive(QueryableByName)]
struct PidRow {
    #[diesel(sql_type = Integer)]
    value: i32,
}

#[test]
fn the_migration_waits_behind_a_conflicting_writer_and_fails_cleanly_under_a_lock_timeout() {
    use std::sync::mpsc;

    let db = TempMigrationDb::new();
    let mut setup = db.conn();
    setup
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations");
    let pool = db.pool();
    seed_representative_state(&pool);
    revert_through_be04(&mut setup);

    // A deterministic fixture rather than a race: one session holds an open
    // transaction that has UPDATEd a publisher row, taking ROW EXCLUSIVE on
    // `public.publisher`, which conflicts with SHARE ROW EXCLUSIVE.
    let (writing_tx, writing_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let url = db.url();
    let writer = thread::spawn(move || {
        let mut connection = PgConnection::establish(&url).expect("writer connection");
        connection.batch_execute("BEGIN").expect("begin");
        connection
            .batch_execute(
                "UPDATE publisher SET publisher_name = publisher_name \
                 WHERE publisher_id = (SELECT publisher_id FROM publisher LIMIT 1)",
            )
            .expect("conflicting write");
        writing_tx.send(()).expect("signal writing");
        release_rx.recv().expect("await release");
        connection.batch_execute("COMMIT").expect("commit");
    });
    writing_rx.recv().expect("writer holds its lock");

    // With a short `lock_timeout` in force the migration fails cleanly rather
    // than waiting indefinitely.
    let mut blocked = db.conn();
    blocked
        .batch_execute("SET lock_timeout = '750ms'")
        .expect("set lock_timeout");
    let started = Instant::now();
    let timed_out = blocked.batch_execute(&format!("BEGIN; {MIGRATION_UP_SQL} COMMIT;"));
    let waited = started.elapsed();
    let _ = blocked.batch_execute("ROLLBACK");
    assert!(
        timed_out.is_err(),
        "the migration must fail cleanly behind a conflicting writer when a \
         lock_timeout is set, rather than proceeding"
    );
    println!(
        "BE-04 migration under contention with lock_timeout = 750ms: waited {waited:?} \
         then failed cleanly (disposable-environment measurement only)"
    );
    assert!(
        waited >= Duration::from_millis(500),
        "it genuinely waited: {waited:?}"
    );

    // With no timeout it waits, and completes once the writer commits.
    let url = db.url();
    let (migrated_tx, migrated_rx) = mpsc::channel::<Result<(), String>>();
    let waiting = thread::spawn(move || {
        let mut connection = PgConnection::establish(&url).expect("waiting connection");
        let outcome = connection
            .batch_execute(&format!("BEGIN; {MIGRATION_UP_SQL} COMMIT;"))
            .map_err(|error| error.to_string());
        migrated_tx.send(outcome).expect("signal outcome");
    });
    // It must still be waiting while the writer holds its lock.
    assert!(
        migrated_rx
            .recv_timeout(Duration::from_millis(750))
            .is_err(),
        "the migration must wait for a conflicting writer rather than skipping it"
    );
    release_tx.send(()).expect("release the writer");
    writer.join().expect("writer thread");
    migrated_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("the migration completes once the writer commits")
        .expect("the migration succeeds once the lock is free");
    waiting.join().expect("waiting thread");

    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'distribution_job%'"
        ),
        3
    );
}

#[test]
fn the_down_migration_is_exercised_on_a_populated_database() {
    let db = TempMigrationDb::new();
    let mut connection = db.conn();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations");
    let pool = db.pool();
    seed_representative_state(&pool);

    // Populate the job relations themselves, so the down migration is exercised
    // against real job, target and attempt rows rather than empty tables.
    let publisher = test_db::create_publisher(&pool);
    activate(
        &pool,
        publisher.publisher_id,
        &[DistributionPlatform::Oapen],
    );
    let job_id = only_job(&pool, publisher.publisher_id).distribution_job_id;
    let claim = claim_distribution_jobs(&pool, WORKER, 1, 900, &[])
        .expect("claim")
        .remove(0);
    fail_distribution_job(
        &pool,
        job_id,
        claim.claim_token,
        "TRANSIENT",
        Some("detail"),
        true,
    )
    .expect("failure");
    assert_eq!(
        scalar_count(&pool, "SELECT count(*) AS count FROM distribution_job"),
        1
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_target"
        ),
        2
    );
    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM distribution_job_attempt"
        ),
        1
    );

    let assignments_before = catalog_values(
        &pool,
        "SELECT publisher_id::text || ' ' || platform::text || ' ' || enabled::text AS value \
         FROM publisher_distribution_platform ORDER BY publisher_id, platform",
    );
    let audit_before = scalar_count(
        &pool,
        "SELECT count(*) AS count FROM publisher_service_configuration_history",
    );

    revert_through_be04(&mut connection);

    assert_eq!(
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'distribution_job%'"
        ),
        0
    );
    assert_eq!(
        assignments_before,
        catalog_values(
            &pool,
            "SELECT publisher_id::text || ' ' || platform::text || ' ' || enabled::text AS value \
             FROM publisher_distribution_platform ORDER BY publisher_id, platform"
        ),
        "the down migration touches no desired-state row"
    );
    assert_eq!(
        audit_before,
        scalar_count(
            &pool,
            "SELECT count(*) AS count FROM publisher_service_configuration_history"
        ),
        "and no configuration audit row"
    );

    // Reversibility evidence only. Dropping a populated job relation in a
    // deployed environment destroys operational audit evidence of what was
    // attempted against external platforms and requires separate explicit
    // authorization.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("re-apply after a populated revert");
}

// ==========================================================================
// Pure-Rust contract tests
// ==========================================================================

#[test]
fn back_catalogue_deduplication_key_matches_the_specified_formula() {
    let publisher_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    let activation_id = Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap();

    let key = DistributionJob::back_catalogue_deduplication_key(publisher_id, activation_id);

    assert_eq!(
        key,
        "PUBLISHER_BACK_CATALOGUE:11111111-1111-4111-8111-111111111111:22222222-2222-4222-8222-222222222222"
    );
    // Fixed at 98 characters, well inside the 256-character check.
    assert_eq!(key.chars().count(), 98);
    // The kind prefix is what guarantees two kinds can never collide in the one
    // unique index.
    assert!(key.starts_with(&format!("{}:", DistributionJobKind::PublisherBackCatalogue)));
}

#[test]
fn distribution_job_creation_parses_exactly_off_and_on() {
    assert_eq!(
        "OFF".parse::<DistributionJobCreation>().unwrap(),
        DistributionJobCreation::Off
    );
    assert_eq!(
        "ON".parse::<DistributionJobCreation>().unwrap(),
        DistributionJobCreation::On
    );
    for invalid in ["off", "on", "On", "TRUE", "1", "", "ENABLED", "OFF "] {
        assert!(
            invalid.parse::<DistributionJobCreation>().is_err(),
            "`{invalid}` must not parse: an unparseable value is never silently \
             resolved to a nearest value"
        );
    }
    for value in [DistributionJobCreation::Off, DistributionJobCreation::On] {
        assert_eq!(
            value
                .to_string()
                .parse::<DistributionJobCreation>()
                .unwrap(),
            value
        );
    }
}

#[test]
fn terminal_statuses_are_exactly_the_three_completed_ones() {
    assert!(!DistributionJobStatus::Pending.is_terminal());
    assert!(!DistributionJobStatus::Running.is_terminal());
    assert!(DistributionJobStatus::Succeeded.is_terminal());
    assert!(DistributionJobStatus::Failed.is_terminal());
    assert!(DistributionJobStatus::Cancelled.is_terminal());
}

#[test]
fn code_owned_bounds_are_the_specified_values() {
    assert_eq!(DISTRIBUTION_JOB_MAX_ATTEMPTS, 5);
    assert_eq!(DISTRIBUTION_JOB_LEASE_DEFAULT_SECONDS, 900);
    assert_eq!(DISTRIBUTION_JOB_LEASE_MIN_SECONDS, 60);
    assert_eq!(DISTRIBUTION_JOB_LEASE_MAX_SECONDS, 3600);
    assert_eq!(DISTRIBUTION_JOB_CLAIM_DEFAULT_BATCH, 10);
    assert_eq!(DISTRIBUTION_JOB_CLAIM_MAX_BATCH, 50);
    assert_eq!(DISTRIBUTION_JOB_LEASE_RECOVERY_BATCH, 50);
    assert_eq!(DISTRIBUTION_JOB_RETRY_BASE_SECONDS, 300);
    assert_eq!(DISTRIBUTION_JOB_RETRY_MAX_SECONDS, 21_600);
    assert_eq!(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS, 64);
    assert_eq!(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS, 2048);

    // The backoff curve the constants describe: 5, 10, 20, 40 minutes for
    // attempts 1 to 4. Attempt 5 is terminal, so the cap is not reached by the
    // current budget and exists so the curve stays correct if it is ever raised.
    let backoff = |n: u32| {
        std::cmp::min(
            DISTRIBUTION_JOB_RETRY_BASE_SECONDS * 2_i64.pow(n - 1),
            DISTRIBUTION_JOB_RETRY_MAX_SECONDS,
        )
    };
    assert_eq!(
        (1..=4).map(backoff).collect::<Vec<_>>(),
        vec![300, 600, 1_200, 2_400]
    );
    assert_eq!(backoff(10), DISTRIBUTION_JOB_RETRY_MAX_SECONDS);
}
