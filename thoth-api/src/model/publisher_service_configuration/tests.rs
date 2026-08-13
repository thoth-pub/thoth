//! `BE-03` coordinator, audit, concurrency, linked-platform, trigger-cascade
//! and staff-report evidence.
//!
//! Every test here runs against a real disposable PostgreSQL database with the
//! migration applied, so both publisher `UPDATE` triggers actually execute and
//! the database's own constraints are exercised rather than assumed.
//!
//! GraphQL-level authorization, capability exposure, SDL and query-count
//! evidence lives in `crate::graphql::service_configuration_tests`.

use std::collections::HashSet;
use std::sync::Arc;

use diesel::sql_query;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use super::crud::{normalize_requested_platforms, replace_publisher_service_configuration};
use super::*;
use crate::db::PgPool;
use crate::model::publisher::{Publisher, PublisherField, PublisherOrderBy, ThothPackage};
use crate::model::publisher_distribution_platform::{
    DistributionPlatform, PublisherDistributionPlatform,
};
use crate::model::tests::db as test_db;
use crate::model::{Crud, Timestamp};
use crate::schema::{publisher, publisher_history, publisher_service_configuration_history, work};
use thoth_errors::{ThothError, ThothResult};

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

const ACTOR: &str = "zitadel-superuser-1";

fn superuser_context() -> ServiceConfigurationWriteContext<'static> {
    ServiceConfigurationWriteContext {
        source: PublisherServiceConfigurationSource::SuperuserApi,
        actor: ACTOR,
    }
}

fn input(
    publisher_id: Uuid,
    package: ThothPackage,
    platforms: &[DistributionPlatform],
    expected_updated_at: Timestamp,
) -> ReplacePublisherServiceConfigurationInput {
    ReplacePublisherServiceConfigurationInput {
        publisher_id,
        subscription_package: package,
        enabled_distribution_platforms: platforms.to_vec(),
        expected_updated_at,
    }
}

/// Call the canonical coordinator exactly as the GraphQL mutation does, with a
/// superuser write context.
fn replace(
    pool: &PgPool,
    data: &ReplacePublisherServiceConfigurationInput,
) -> ThothResult<PublisherServiceConfiguration> {
    replace_publisher_service_configuration(pool, &superuser_context(), data)
}

fn publisher_row(pool: &PgPool, publisher_id: Uuid) -> Publisher {
    Publisher::from_id(pool, &publisher_id).expect("publisher row")
}

fn token(pool: &PgPool, publisher_id: Uuid) -> Timestamp {
    publisher_row(pool, publisher_id).service_configuration_updated_at
}

fn enabled(pool: &PgPool, publisher_id: Uuid) -> Vec<DistributionPlatform> {
    PublisherDistributionPlatform::enabled_assignments(pool, publisher_id)
        .expect("enabled assignments")
        .into_iter()
        .map(|assignment| assignment.platform)
        .collect()
}

fn audit_rows(pool: &PgPool, publisher_id: Uuid) -> Vec<PublisherServiceConfigurationHistory> {
    let mut connection = pool.get().expect("connection");
    publisher_service_configuration_history::table
        .filter(publisher_service_configuration_history::publisher_id.eq(publisher_id))
        .order(publisher_service_configuration_history::created_at.asc())
        .load::<PublisherServiceConfigurationHistory>(&mut connection)
        .expect("audit rows")
}

fn only_audit_row(pool: &PgPool, publisher_id: Uuid) -> PublisherServiceConfigurationHistory {
    let mut rows = audit_rows(pool, publisher_id);
    assert_eq!(rows.len(), 1, "expected exactly one audit row");
    rows.pop().expect("audit row")
}

fn publisher_history_count(pool: &PgPool, publisher_id: Uuid) -> i64 {
    let mut connection = pool.get().expect("connection");
    publisher_history::table
        .filter(publisher_history::publisher_id.eq(publisher_id))
        .count()
        .get_result::<i64>(&mut connection)
        .expect("publisher history count")
}

fn all_rows(pool: &PgPool, publisher_id: Uuid) -> Vec<PublisherDistributionPlatform> {
    PublisherDistributionPlatform::all_for_publisher(pool, publisher_id).expect("assignment rows")
}

/// Force a linked state the supported domain path cannot produce, so repair
/// behaviour can be proven. This mirrors `BE-02`'s own raw-write fixture.
fn write_raw_assignment(
    pool: &PgPool,
    publisher_id: Uuid,
    platform: &str,
    activation_id: Uuid,
    enabled_at_sql: &str,
) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at, disabled_at) \
         VALUES ('{publisher_id}', '{platform}', true, '{activation_id}', {enabled_at_sql}, NULL) \
         ON CONFLICT (publisher_id, platform) DO UPDATE SET \
         enabled = true, activation_id = EXCLUDED.activation_id, \
         enabled_at = EXCLUDED.enabled_at, disabled_at = NULL"
    ))
    .execute(&mut connection)
    .expect("raw assignment write");
}

/// Set a publisher's package outside the coordinator, so a package fixture can
/// be established without moving the configuration token.
fn set_package_directly(pool: &PgPool, publisher_id: Uuid, package: ThothPackage) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher SET subscription_package = '{package}' \
         WHERE publisher_id = '{publisher_id}'"
    ))
    .execute(&mut connection)
    .expect("package fixture");
}

fn state_keys(state: &JsonValue) -> Vec<String> {
    let mut keys: Vec<String> = state
        .as_object()
        .expect("audit state must be a JSON object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

fn platforms_in(state: &JsonValue) -> Vec<String> {
    state["enabledDistributionPlatforms"]
        .as_array()
        .expect("platform array")
        .iter()
        .map(|value| value.as_str().expect("platform code").to_string())
        .collect()
}

/// A guard that removes an injected failure trigger even if the test panics.
struct InjectedFailure {
    pool: Arc<PgPool>,
}

impl InjectedFailure {
    /// Fail every insert into the configuration audit table, which happens
    /// after the package update, after the lifecycle calls and after the token
    /// bump, but before the coordinator's transaction commits.
    fn on_audit_insert(pool: &Arc<PgPool>) -> Self {
        let mut connection = pool.get().expect("connection");
        sql_query(
            "CREATE OR REPLACE FUNCTION be03_reject_audit() RETURNS trigger AS $$ \
             BEGIN RAISE EXCEPTION 'injected pre-commit failure'; END; $$ LANGUAGE plpgsql",
        )
        .execute(&mut connection)
        .expect("create injection function");
        sql_query(
            "CREATE TRIGGER be03_reject_audit BEFORE INSERT \
             ON publisher_service_configuration_history \
             FOR EACH ROW EXECUTE FUNCTION be03_reject_audit()",
        )
        .execute(&mut connection)
        .expect("create injection trigger");
        Self {
            pool: Arc::clone(pool),
        }
    }
}

impl Drop for InjectedFailure {
    fn drop(&mut self) {
        let Ok(mut connection) = self.pool.get() else {
            return;
        };
        let _ = sql_query(
            "DROP TRIGGER IF EXISTS be03_reject_audit ON publisher_service_configuration_history",
        )
        .execute(&mut connection);
        let _ = sql_query("DROP FUNCTION IF EXISTS be03_reject_audit()").execute(&mut connection);
    }
}

// --------------------------------------------------------------------------
// Normalization vocabulary
// --------------------------------------------------------------------------

#[test]
fn requested_platforms_are_deduplicated_and_closed_under_linked_membership() {
    use DistributionPlatform::{Doab, Oapen, OclcKb, Zenodo};

    assert_eq!(normalize_requested_platforms(&[]), Vec::new());
    assert_eq!(normalize_requested_platforms(&[Oapen]), vec![Oapen, Doab]);
    assert_eq!(normalize_requested_platforms(&[Doab]), vec![Oapen, Doab]);
    assert_eq!(
        normalize_requested_platforms(&[Doab, Oapen, Doab]),
        vec![Oapen, Doab]
    );
    // Canonical `DistributionPlatform::ALL` order, not request order.
    assert_eq!(
        normalize_requested_platforms(&[Zenodo, OclcKb, Zenodo]),
        vec![Zenodo, OclcKb]
    );
}

// --------------------------------------------------------------------------
// Committed change, token and audit
// --------------------------------------------------------------------------

#[test]
fn a_committed_change_moves_the_token_and_writes_exactly_one_audit_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let before = token(&pool, publisher.publisher_id);

    let configuration = replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo, DistributionPlatform::Oapen],
            before,
        ),
    )
    .expect("replace");

    let after = token(&pool, publisher.publisher_id);
    assert!(after > before, "the token must move");
    assert_eq!(
        configuration.publisher.service_configuration_updated_at,
        after
    );
    assert_eq!(
        configuration.publisher.subscription_package,
        ThothPackage::Sphinx
    );
    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![
            DistributionPlatform::Oapen,
            DistributionPlatform::Doab,
            DistributionPlatform::Zenodo
        ]
    );

    let row = only_audit_row(&pool, publisher.publisher_id);
    assert_eq!(row.actor, ACTOR);
    assert_eq!(
        row.source,
        PublisherServiceConfigurationSource::SuperuserApi
    );
}

#[test]
fn the_audit_json_key_set_is_exactly_the_three_canonical_keys() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let before = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Oapen],
            before,
        ),
    )
    .expect("replace");

    let row = only_audit_row(&pool, publisher.publisher_id);
    let expected = vec![
        "configurationVersion".to_string(),
        "enabledDistributionPlatforms".to_string(),
        "subscriptionPackage".to_string(),
    ];
    // This fails if any key is ever added, including an activation identifier,
    // a per-row timestamp, a capability list or any publisher metadata.
    assert_eq!(state_keys(&row.before_state), expected);
    assert_eq!(state_keys(&row.after_state), expected);

    let serialized = format!("{}{}", row.before_state, row.after_state);
    for forbidden in [
        "activation",
        "enabledAt",
        "disabledAt",
        "capabilit",
        "zitadel",
        "publisherName",
        "credential",
        "token",
        "endpoint",
        "bucket",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "audit JSON must not contain `{forbidden}`: {serialized}"
        );
    }
}

#[test]
fn audit_states_record_the_canonical_before_and_after_configuration() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let first_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Doab],
            first_token,
        ),
    )
    .expect("first replace");
    let second_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Pyramid,
            &[
                DistributionPlatform::Doab,
                DistributionPlatform::InternetArchive,
            ],
            second_token,
        ),
    )
    .expect("second replace");

    let rows = audit_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 2);
    let second = &rows[1];

    assert_eq!(second.before_state["subscriptionPackage"], "OBELISK");
    assert_eq!(second.after_state["subscriptionPackage"], "PYRAMID");
    // Canonical `DistributionPlatform::ALL` order: INTERNET_ARCHIVE precedes
    // OAPEN, which precedes DOAB.
    assert_eq!(platforms_in(&second.before_state), vec!["OAPEN", "DOAB"]);
    assert_eq!(
        platforms_in(&second.after_state),
        vec!["INTERNET_ARCHIVE", "OAPEN", "DOAB"]
    );
    assert_eq!(
        second.before_state["configurationVersion"],
        serde_json::to_value(second_token).expect("token json")
    );
    assert_eq!(
        second.after_state["configurationVersion"],
        serde_json::to_value(token(&pool, publisher.publisher_id)).expect("token json")
    );
}

#[test]
fn one_request_touching_several_groups_writes_one_audit_row_and_one_token_bump() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let before = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[
                DistributionPlatform::Oapen,
                DistributionPlatform::OclcKb,
                DistributionPlatform::ExLibrisKb,
                DistributionPlatform::Crossref,
            ],
            before,
        ),
    )
    .expect("replace");

    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![
            DistributionPlatform::Oapen,
            DistributionPlatform::Doab,
            DistributionPlatform::Crossref,
            DistributionPlatform::OclcKb,
            DistributionPlatform::ExLibrisKb,
        ]
    );
    assert!(token(&pool, publisher.publisher_id) > before);
}

#[test]
fn the_mutation_writes_no_publisher_history_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let before = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            before,
        ),
    )
    .expect("replace");

    // The coordinator writes `subscription_package` directly and never through
    // the shared `Crud::update` macro, so the generic entity history is not
    // touched and the configuration audit is the only history BE-03 writes.
    assert_eq!(publisher_history_count(&pool, publisher.publisher_id), 0);
}

#[test]
fn the_database_rejects_an_actor_with_no_non_whitespace_character() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let mut connection = pool.get().expect("connection");

    let insert = |connection: &mut diesel::PgConnection, actor: &str| {
        diesel::insert_into(publisher_service_configuration_history::table)
            .values(&NewPublisherServiceConfigurationHistory {
                publisher_id: publisher.publisher_id,
                actor: actor.to_string(),
                source: PublisherServiceConfigurationSource::SuperuserApi,
                before_state: serde_json::json!({}),
                after_state: serde_json::json!({}),
            })
            .execute(connection)
    };

    // The invariant is that an audit actor must contain at least one
    // non-whitespace character, enforced by
    // `CHECK (actor ~ '[^[:space:]]')`. The POSIX `[[:space:]]` class covers
    // every case below, so each is rejected by the database itself. A narrower
    // `btrim(actor) <> ''` predicate would accept everything from the tab case
    // downwards, because one-argument `btrim` trims spaces only.
    for (name, actor) in [
        ("empty", ""),
        ("single space", " "),
        ("three spaces", "   "),
        ("tab", "\t"),
        ("newline", "\n"),
        ("carriage return", "\r"),
        ("vertical tab", "\u{0b}"),
        ("form feed", "\u{0c}"),
        ("mixed whitespace", " \t\n\r\u{0b}\u{0c} "),
    ] {
        let outcome = insert(&mut connection, actor);
        assert!(
            outcome.is_err(),
            "the actor check must reject the {name} case {actor:?}"
        );
    }

    // An actor carrying a real identifier is accepted even when it is
    // surrounded by whitespace: the invariant is presence of a non-whitespace
    // character, not absence of whitespace.
    for (name, actor) in [
        ("space padded", "  real-actor-42  "),
        ("tab and newline padded", "\t\nreal-actor-42\r\n"),
    ] {
        assert!(
            insert(&mut connection, actor).is_ok(),
            "the actor check must accept the {name} case {actor:?}"
        );
    }

    // Only the two accepted rows exist, and no BE-03 write path can produce a
    // whitespace-only actor in any case: the only production writer takes it
    // from `PolicyContext::user_id()`.
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 2);
}

#[test]
fn no_be03_path_writes_a_migration_backfill_source() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let mut current = token(&pool, publisher.publisher_id);

    for (package, platforms) in [
        (ThothPackage::Obelisk, vec![DistributionPlatform::Oapen]),
        (ThothPackage::Sphinx, vec![DistributionPlatform::Oapen]),
        (ThothPackage::Sphinx, vec![]),
    ] {
        replace(
            &pool,
            &input(publisher.publisher_id, package, &platforms, current),
        )
        .expect("replace");
        current = token(&pool, publisher.publisher_id);
    }

    let rows = audit_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 3);
    assert!(rows
        .iter()
        .all(|row| row.source == PublisherServiceConfigurationSource::SuperuserApi));
    assert!(rows
        .iter()
        .all(|row| row.source != PublisherServiceConfigurationSource::MigrationBackfill));
}

#[test]
fn an_unknown_publisher_yields_entity_not_found_and_writes_nothing() {
    let (_guard, pool) = test_db::setup_test_db();
    let unknown = Uuid::new_v4();

    let outcome = replace(
        &pool,
        &input(
            unknown,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            Timestamp::default(),
        ),
    );

    assert!(matches!(outcome, Err(ThothError::EntityNotFound)));
    assert!(audit_rows(&pool, unknown).is_empty());
}

// --------------------------------------------------------------------------
// True no-op, staleness and rollback
// --------------------------------------------------------------------------

#[test]
fn a_true_no_op_moves_no_token_and_writes_no_audit_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let first = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Oapen],
            first,
        ),
    )
    .expect("seed replace");

    let seeded = publisher_row(&pool, publisher.publisher_id);
    let rows_before = all_rows(&pool, publisher.publisher_id);

    // Same package, same normalized membership, group already fully normalized.
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Doab],
            seeded.service_configuration_updated_at,
        ),
    )
    .expect("no-op replace");

    let after = publisher_row(&pool, publisher.publisher_id);
    assert_eq!(
        after.service_configuration_updated_at,
        seeded.service_configuration_updated_at
    );
    assert_eq!(after.updated_at, seeded.updated_at);
    assert_eq!(all_rows(&pool, publisher.publisher_id), rows_before);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn a_stale_request_fails_and_writes_nothing() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stale = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            stale,
        ),
    )
    .expect("first replace");
    let committed = publisher_row(&pool, publisher.publisher_id);

    let outcome = replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Pyramid,
            &[DistributionPlatform::Crossref],
            stale,
        ),
    );

    assert!(matches!(
        outcome,
        Err(ThothError::StalePublisherServiceConfiguration)
    ));
    let after = publisher_row(&pool, publisher.publisher_id);
    assert_eq!(after, committed);
    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![DistributionPlatform::Zenodo]
    );
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn a_stale_request_that_would_have_been_a_true_no_op_still_fails() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stale = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            stale,
        ),
    )
    .expect("first replace");
    let committed = publisher_row(&pool, publisher.publisher_id);

    // Semantically identical to the committed state, but with the superseded
    // token: the version check precedes everything.
    let outcome = replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            stale,
        ),
    );

    assert!(matches!(
        outcome,
        Err(ThothError::StalePublisherServiceConfiguration)
    ));
    assert_eq!(publisher_row(&pool, publisher.publisher_id), committed);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn a_stale_request_that_would_have_repaired_a_split_pair_still_fails() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let stale = token(&pool, publisher.publisher_id);

    // Move the token so the request below is stale, without touching the pair.
    replace(
        &pool,
        &input(publisher.publisher_id, ThothPackage::Obelisk, &[], stale),
    )
    .expect("token move");

    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        Uuid::new_v4(),
        "now()",
    );
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "DOAB",
        Uuid::new_v4(),
        "now() - interval '1 hour'",
    );
    let split = all_rows(&pool, publisher.publisher_id);
    let committed_token = token(&pool, publisher.publisher_id);

    let outcome = replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Oapen],
            stale,
        ),
    );

    assert!(matches!(
        outcome,
        Err(ThothError::StalePublisherServiceConfiguration)
    ));
    assert_eq!(
        all_rows(&pool, publisher.publisher_id),
        split,
        "the split pair must survive a stale request exactly as it was"
    );
    assert_eq!(token(&pool, publisher.publisher_id), committed_token);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn an_injected_pre_commit_failure_rolls_the_whole_change_back() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            start,
        ),
    )
    .expect("seed replace");
    let committed = publisher_row(&pool, publisher.publisher_id);
    let committed_rows = all_rows(&pool, publisher.publisher_id);

    let outcome = {
        let _injection = InjectedFailure::on_audit_insert(&pool);
        replace(
            &pool,
            &input(
                publisher.publisher_id,
                ThothPackage::Pyramid,
                &[DistributionPlatform::Oapen],
                committed.service_configuration_updated_at,
            ),
        )
    };

    assert!(outcome.is_err(), "the injected failure must propagate");
    let after = publisher_row(&pool, publisher.publisher_id);
    assert_eq!(
        after, committed,
        "package, token and publisher.updated_at all roll back together"
    );
    assert_eq!(all_rows(&pool, publisher.publisher_id), committed_rows);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

// --------------------------------------------------------------------------
// Independence and linked platforms
// --------------------------------------------------------------------------

#[test]
fn a_package_only_change_leaves_every_assignment_row_byte_identical() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
            start,
        ),
    )
    .expect("seed replace");
    let rows_before = all_rows(&pool, publisher.publisher_id);
    let seeded_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
            seeded_token,
        ),
    )
    .expect("package-only replace");

    assert_eq!(all_rows(&pool, publisher.publisher_id), rows_before);
    assert_eq!(
        publisher_row(&pool, publisher.publisher_id).subscription_package,
        ThothPackage::Sphinx
    );
    assert!(token(&pool, publisher.publisher_id) > seeded_token);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 2);
}

#[test]
fn a_platform_only_change_leaves_the_package_unchanged() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    set_package_directly(&pool, publisher.publisher_id, ThothPackage::Obelisk);
    let start = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Jstor],
            start,
        ),
    )
    .expect("platform-only replace");

    assert_eq!(
        publisher_row(&pool, publisher.publisher_id).subscription_package,
        ThothPackage::Obelisk
    );
    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![DistributionPlatform::Jstor]
    );
}

#[test]
fn requesting_either_linked_member_enables_both_with_one_shared_activation() {
    let (_guard, pool) = test_db::setup_test_db();
    for platform in [DistributionPlatform::Oapen, DistributionPlatform::Doab] {
        let publisher = test_db::create_publisher(&pool);
        let start = token(&pool, publisher.publisher_id);
        replace(
            &pool,
            &input(
                publisher.publisher_id,
                ThothPackage::Oasis,
                &[platform],
                start,
            ),
        )
        .expect("linked replace");

        let rows = all_rows(&pool, publisher.publisher_id);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.enabled));
        assert_eq!(rows[0].activation_id, rows[1].activation_id);
        assert_eq!(rows[0].enabled_at, rows[1].enabled_at);
    }
}

#[test]
fn omitting_both_linked_members_disables_both() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            start,
        ),
    )
    .expect("seed replace");
    let seeded_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[],
            seeded_token,
        ),
    )
    .expect("disable replace");

    assert!(enabled(&pool, publisher.publisher_id).is_empty());
    let rows = all_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 2, "disabled rows are retained, never deleted");
    assert!(rows
        .iter()
        .all(|row| !row.enabled && row.disabled_at.is_some()));
}

#[test]
fn an_empty_request_disables_everything_and_is_never_read_as_all() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[
                DistributionPlatform::Zenodo,
                DistributionPlatform::Oapen,
                DistributionPlatform::OclcKb,
            ],
            start,
        ),
    )
    .expect("seed replace");
    let seeded_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[],
            seeded_token,
        ),
    )
    .expect("empty replace");

    assert!(enabled(&pool, publisher.publisher_id).is_empty());
}

#[test]
fn duplicates_in_the_requested_list_are_deduplicated_with_no_error() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[
                DistributionPlatform::Zenodo,
                DistributionPlatform::Zenodo,
                DistributionPlatform::Oapen,
                DistributionPlatform::Doab,
            ],
            start,
        ),
    )
    .expect("duplicate replace");

    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![
            DistributionPlatform::Oapen,
            DistributionPlatform::Doab,
            DistributionPlatform::Zenodo
        ]
    );
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

/// Every membership-equal split state must still be repaired, must bump the
/// token and must write exactly one audit row whose two states differ only in
/// `configurationVersion`.
fn assert_membership_equal_repair(seed: impl Fn(&PgPool, Uuid), requested: DistributionPlatform) {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    seed(&pool, publisher.publisher_id);
    let before_token = token(&pool, publisher.publisher_id);
    let membership_before = enabled(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[requested],
            before_token,
        ),
    )
    .expect("repair replace");

    let rows = all_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .all(|row| row.enabled && row.disabled_at.is_none()));
    assert_eq!(rows[0].activation_id, rows[1].activation_id);
    assert_eq!(rows[0].enabled_at, rows[1].enabled_at);

    let after_token = token(&pool, publisher.publisher_id);
    assert!(after_token > before_token, "a repair is a committed change");

    let row = only_audit_row(&pool, publisher.publisher_id);
    assert_eq!(
        row.before_state["subscriptionPackage"],
        row.after_state["subscriptionPackage"]
    );
    if membership_before.len() == 2 {
        assert_eq!(
            platforms_in(&row.before_state),
            platforms_in(&row.after_state),
            "a membership-equal repair differs only in configurationVersion"
        );
    }
    assert_ne!(
        row.before_state["configurationVersion"],
        row.after_state["configurationVersion"]
    );
    assert_eq!(
        row.before_state["configurationVersion"],
        serde_json::to_value(before_token).expect("token json")
    );
    assert_eq!(
        row.after_state["configurationVersion"],
        serde_json::to_value(after_token).expect("token json")
    );
}

#[test]
fn a_split_activation_pair_is_repaired_although_membership_is_unchanged() {
    assert_membership_equal_repair(
        |pool, publisher_id| {
            let enabled_at = "now()";
            write_raw_assignment(pool, publisher_id, "OAPEN", Uuid::new_v4(), enabled_at);
            write_raw_assignment(pool, publisher_id, "DOAB", Uuid::new_v4(), enabled_at);
        },
        DistributionPlatform::Oapen,
    );
}

#[test]
fn a_split_enabled_at_pair_is_repaired_although_membership_is_unchanged() {
    assert_membership_equal_repair(
        |pool, publisher_id| {
            let activation = Uuid::new_v4();
            write_raw_assignment(pool, publisher_id, "OAPEN", activation, "now()");
            write_raw_assignment(
                pool,
                publisher_id,
                "DOAB",
                activation,
                "now() - interval '2 hours'",
            );
        },
        DistributionPlatform::Doab,
    );
}

#[test]
fn a_one_sided_pair_is_repaired_whichever_member_the_request_names() {
    for requested in [DistributionPlatform::Oapen, DistributionPlatform::Doab] {
        assert_membership_equal_repair(
            |pool, publisher_id| {
                write_raw_assignment(pool, publisher_id, "OAPEN", Uuid::new_v4(), "now()");
            },
            requested,
        );
    }
}

#[test]
fn a_fully_normalized_pair_with_the_same_request_is_a_true_no_op() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            start,
        ),
    )
    .expect("seed replace");
    let rows_before = all_rows(&pool, publisher.publisher_id);
    let seeded = publisher_row(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            seeded.service_configuration_updated_at,
        ),
    )
    .expect("no-op replace");

    assert_eq!(all_rows(&pool, publisher.publisher_id), rows_before);
    assert_eq!(publisher_row(&pool, publisher.publisher_id), seeded);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn a_package_change_over_a_split_group_writes_the_package_and_repairs_the_group() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        Uuid::new_v4(),
        "now()",
    );
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "DOAB",
        Uuid::new_v4(),
        "now()",
    );
    let before = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen],
            before,
        ),
    )
    .expect("replace");

    let rows = all_rows(&pool, publisher.publisher_id);
    assert_eq!(rows[0].activation_id, rows[1].activation_id);
    assert_eq!(
        publisher_row(&pool, publisher.publisher_id).subscription_package,
        ThothPackage::Sphinx
    );
    assert!(token(&pool, publisher.publisher_id) > before);
    assert_eq!(
        audit_rows(&pool, publisher.publisher_id).len(),
        1,
        "one audit row for the whole change"
    );
}

#[test]
fn a_no_op_group_with_a_package_change_leaves_assignment_rows_byte_identical() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            start,
        ),
    )
    .expect("seed replace");
    let rows_before = all_rows(&pool, publisher.publisher_id);
    let seeded_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Pyramid,
            &[DistributionPlatform::Doab],
            seeded_token,
        ),
    )
    .expect("package replace");

    assert_eq!(all_rows(&pool, publisher.publisher_id), rows_before);
    assert!(token(&pool, publisher.publisher_id) > seeded_token);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 2);
}

#[test]
fn oclc_and_ex_libris_remain_independently_configurable() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[
                DistributionPlatform::OclcKb,
                DistributionPlatform::ExLibrisKb,
            ],
            start,
        ),
    )
    .expect("seed replace");
    let rows = all_rows(&pool, publisher.publisher_id);
    assert_ne!(
        rows[0].activation_id, rows[1].activation_id,
        "unlinked platforms receive independent activations"
    );
    let seeded_token = token(&pool, publisher.publisher_id);

    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::OclcKb],
            seeded_token,
        ),
    )
    .expect("partial disable");

    assert_eq!(
        enabled(&pool, publisher.publisher_id),
        vec![DistributionPlatform::OclcKb]
    );
}

#[test]
fn requesting_jisc_nbk_fails_before_any_write() {
    let (_guard, pool) = test_db::setup_test_db();

    // (a) A request that would otherwise have changed nothing.
    let quiet = test_db::create_publisher(&pool);
    let quiet_before = publisher_row(&pool, quiet.publisher_id);
    let outcome = replace(
        &pool,
        &input(
            quiet.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::JiscNbk],
            quiet_before.service_configuration_updated_at,
        ),
    );
    assert!(matches!(
        outcome,
        Err(ThothError::DistributionPlatformNotAssignable(_))
    ));
    assert_eq!(publisher_row(&pool, quiet.publisher_id), quiet_before);
    assert!(all_rows(&pool, quiet.publisher_id).is_empty());
    assert!(audit_rows(&pool, quiet.publisher_id).is_empty());

    // (b) A request that would otherwise have changed the package and enabled a
    // valid platform: pre-validation of the whole set precedes the first
    // lifecycle call, so nothing is written and nothing relies on rollback.
    let busy = test_db::create_publisher(&pool);
    let busy_before = publisher_row(&pool, busy.publisher_id);
    let outcome = replace(
        &pool,
        &input(
            busy.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo, DistributionPlatform::JiscNbk],
            busy_before.service_configuration_updated_at,
        ),
    );
    assert!(matches!(
        outcome,
        Err(ThothError::DistributionPlatformNotAssignable(_))
    ));
    assert_eq!(publisher_row(&pool, busy.publisher_id), busy_before);
    assert!(all_rows(&pool, busy.publisher_id).is_empty());
    assert!(audit_rows(&pool, busy.publisher_id).is_empty());
}

// --------------------------------------------------------------------------
// Concurrency
// --------------------------------------------------------------------------

/// Two clients holding the same token both submit; exactly one commits.
fn concurrent_replacements(
    pool: &Arc<PgPool>,
    publisher_id: Uuid,
    first: ReplacePublisherServiceConfigurationInput,
    second: ReplacePublisherServiceConfigurationInput,
) -> (
    ThothResult<PublisherServiceConfiguration>,
    ThothResult<PublisherServiceConfiguration>,
) {
    let one = {
        let pool = Arc::clone(pool);
        std::thread::spawn(move || replace(&pool, &first))
    };
    let two = {
        let pool = Arc::clone(pool);
        std::thread::spawn(move || replace(&pool, &second))
    };
    let _ = publisher_id;
    (
        one.join().expect("thread one"),
        two.join().expect("thread two"),
    )
}

fn assert_one_winner_one_stale(
    outcomes: (
        ThothResult<PublisherServiceConfiguration>,
        ThothResult<PublisherServiceConfiguration>,
    ),
) {
    let (first, second) = outcomes;
    let winners = usize::from(first.is_ok()) + usize::from(second.is_ok());
    assert_eq!(winners, 1, "exactly one client may commit");
    for outcome in [first, second] {
        if let Err(error) = outcome {
            assert!(
                matches!(error, ThothError::StalePublisherServiceConfiguration),
                "the loser must fail as stale, got {error:?}"
            );
        }
    }
}

#[test]
fn two_clients_holding_one_token_produce_one_winner_and_one_stale_loser() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let shared = token(&pool, publisher.publisher_id);

    let outcomes = concurrent_replacements(
        &pool,
        publisher.publisher_id,
        input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            shared,
        ),
        input(
            publisher.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Crossref],
            shared,
        ),
    );
    assert_one_winner_one_stale(outcomes);

    // Exactly one committed change: the loser wrote nothing.
    let rows = audit_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 1);
    let committed = publisher_row(&pool, publisher.publisher_id);
    let enabled_now = enabled(&pool, publisher.publisher_id);
    assert_eq!(
        platforms_in(&rows[0].after_state),
        enabled_now
            .iter()
            .map(|platform| platform.to_string())
            .collect::<Vec<_>>(),
        "the final state is exactly what the winner committed"
    );
    assert_eq!(
        rows[0].after_state["subscriptionPackage"],
        JsonValue::from(committed.subscription_package.to_string())
    );
}

#[test]
fn concurrent_linked_replacements_leave_no_one_sided_pair() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let shared = token(&pool, publisher.publisher_id);

    let outcomes = concurrent_replacements(
        &pool,
        publisher.publisher_id,
        input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            shared,
        ),
        input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Doab],
            shared,
        ),
    );
    assert_one_winner_one_stale(outcomes);

    let rows = all_rows(&pool, publisher.publisher_id);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.enabled));
    assert_eq!(rows[0].activation_id, rows[1].activation_id);
    assert_eq!(rows[0].enabled_at, rows[1].enabled_at);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn two_concurrent_membership_equal_repairs_produce_one_repair_and_one_stale() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        Uuid::new_v4(),
        "now()",
    );
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "DOAB",
        Uuid::new_v4(),
        "now()",
    );
    let shared = token(&pool, publisher.publisher_id);

    let outcomes = concurrent_replacements(
        &pool,
        publisher.publisher_id,
        input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            shared,
        ),
        input(
            publisher.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Doab],
            shared,
        ),
    );
    assert_one_winner_one_stale(outcomes);

    let rows = all_rows(&pool, publisher.publisher_id);
    assert_eq!(rows[0].activation_id, rows[1].activation_id);
    assert_eq!(rows[0].enabled_at, rows[1].enabled_at);
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);
}

#[test]
fn concurrent_replacements_for_different_publishers_do_not_contend() {
    let (_guard, pool) = test_db::setup_test_db();
    let first = test_db::create_publisher(&pool);
    let second = test_db::create_publisher(&pool);
    let first_token = token(&pool, first.publisher_id);
    let second_token = token(&pool, second.publisher_id);

    let outcomes = concurrent_replacements(
        &pool,
        first.publisher_id,
        input(
            first.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            first_token,
        ),
        input(
            second.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Crossref],
            second_token,
        ),
    );

    assert!(outcomes.0.is_ok() && outcomes.1.is_ok(), "both must commit");
    assert_eq!(audit_rows(&pool, first.publisher_id).len(), 1);
    assert_eq!(audit_rows(&pool, second.publisher_id).len(), 1);
}

#[test]
fn a_replacement_concurrent_with_a_direct_be02_transition_serializes_without_deadlock() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::ProjectMuse,
    )
    .expect("seed enable");
    let current = token(&pool, publisher.publisher_id);

    let coordinator = {
        let pool = Arc::clone(&pool);
        let publisher_id = publisher.publisher_id;
        std::thread::spawn(move || {
            replace(
                &pool,
                &input(
                    publisher_id,
                    ThothPackage::Sphinx,
                    &[DistributionPlatform::Zenodo],
                    current,
                ),
            )
        })
    };
    let direct = {
        let pool = Arc::clone(&pool);
        let publisher_id = publisher.publisher_id;
        std::thread::spawn(move || {
            PublisherDistributionPlatform::disable(
                &pool,
                publisher_id,
                DistributionPlatform::ProjectMuse,
            )
        })
    };

    coordinator
        .join()
        .expect("coordinator thread")
        .expect("coordinator replace");
    direct
        .join()
        .expect("direct thread")
        .expect("direct disable");

    // Both took the same publisher row lock, so they serialized. Every row is
    // internally consistent whichever order they ran in.
    for row in all_rows(&pool, publisher.publisher_id) {
        assert_eq!(row.enabled, row.disabled_at.is_none());
    }
}

#[test]
fn the_token_is_strictly_monotonic_per_publisher_across_a_sequence_with_a_repair() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let mut observed = vec![token(&pool, publisher.publisher_id)];

    // Ordinary committed changes.
    for (package, platforms) in [
        (ThothPackage::Obelisk, vec![DistributionPlatform::Oapen]),
        (ThothPackage::Sphinx, vec![DistributionPlatform::Oapen]),
    ] {
        let current = *observed.last().expect("token");
        replace(
            &pool,
            &input(publisher.publisher_id, package, &platforms, current),
        )
        .expect("replace");
        observed.push(token(&pool, publisher.publisher_id));
    }

    // A membership-equal repair is also a committed change.
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "DOAB",
        Uuid::new_v4(),
        "now() - interval '1 hour'",
    );
    let current = *observed.last().expect("token");
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen],
            current,
        ),
    )
    .expect("repair replace");
    observed.push(token(&pool, publisher.publisher_id));

    for pair in observed.windows(2) {
        assert!(
            pair[1] > pair[0],
            "the token must be strictly increasing: {pair:?}"
        );
    }
    let unique: HashSet<String> = observed.iter().map(|value| value.to_rfc3339()).collect();
    assert_eq!(unique.len(), observed.len());
}

// --------------------------------------------------------------------------
// Publisher and work trigger cascade (specification sections 6.4 and 18.4)
// --------------------------------------------------------------------------

struct CascadeFixture {
    publisher_id: Uuid,
    target_work_ids: Vec<Uuid>,
    control_work_id: Uuid,
}

/// One target publisher with two imprints and two works across them, plus a
/// control work belonging to a different publisher, so the trigger's
/// `work -> imprint -> publisher` join is genuinely traversed and its bound is
/// proven rather than assumed.
fn cascade_fixture(pool: &PgPool) -> CascadeFixture {
    let publisher = test_db::create_publisher(pool);
    let first_imprint = test_db::create_imprint(pool, &publisher);
    let second_imprint = test_db::create_imprint(pool, &publisher);
    let first_work = test_db::create_work(pool, &first_imprint);
    let second_work = test_db::create_work(pool, &second_imprint);

    let other_publisher = test_db::create_publisher(pool);
    let other_imprint = test_db::create_imprint(pool, &other_publisher);
    let control_work = test_db::create_work(pool, &other_imprint);

    CascadeFixture {
        publisher_id: publisher.publisher_id,
        target_work_ids: vec![first_work.work_id, second_work.work_id],
        control_work_id: control_work.work_id,
    }
}

fn work_freshness(pool: &PgPool, work_id: Uuid) -> Timestamp {
    let mut connection = pool.get().expect("connection");
    work::table
        .filter(work::work_id.eq(work_id))
        .select(work::updated_at_with_relations)
        .first::<Timestamp>(&mut connection)
        .expect("work freshness")
}

struct CascadeSnapshot {
    configuration_token: Timestamp,
    publisher_updated_at: Timestamp,
    target_freshness: Vec<Timestamp>,
    control_freshness: Timestamp,
}

fn cascade_snapshot(pool: &PgPool, fixture: &CascadeFixture) -> CascadeSnapshot {
    let publisher = publisher_row(pool, fixture.publisher_id);
    CascadeSnapshot {
        configuration_token: publisher.service_configuration_updated_at,
        publisher_updated_at: publisher.updated_at,
        target_freshness: fixture
            .target_work_ids
            .iter()
            .map(|work_id| work_freshness(pool, *work_id))
            .collect(),
        control_freshness: work_freshness(pool, fixture.control_work_id),
    }
}

fn assert_cascade(before: &CascadeSnapshot, after: &CascadeSnapshot, moved: bool) {
    if moved {
        assert!(
            after.configuration_token > before.configuration_token,
            "the configuration token must move"
        );
        assert!(
            after.publisher_updated_at > before.publisher_updated_at,
            "publisher.updated_at must move"
        );
        for (index, (before_value, after_value)) in before
            .target_freshness
            .iter()
            .zip(after.target_freshness.iter())
            .enumerate()
        {
            assert!(
                after_value > before_value,
                "target work {index} freshness must move"
            );
        }
    } else {
        assert_eq!(after.configuration_token, before.configuration_token);
        assert_eq!(after.publisher_updated_at, before.publisher_updated_at);
        assert_eq!(after.target_freshness, before.target_freshness);
    }
    assert_eq!(
        after.control_freshness, before.control_freshness,
        "a work of another publisher must never move"
    );
}

#[test]
fn a_committed_package_only_change_moves_all_three_timestamps() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    let before = cascade_snapshot(&pool, &fixture);

    replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Sphinx,
            &[],
            before.configuration_token,
        ),
    )
    .expect("package-only replace");

    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), true);
}

#[test]
fn a_committed_platform_only_change_moves_all_three_timestamps() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    let before = cascade_snapshot(&pool, &fixture);

    // Merged BE-02 alone would have moved neither publisher.updated_at nor any
    // work freshness value for this change. BE-03 does, because the same
    // transaction writes the configuration token to the publisher row. This is
    // asserted deliberately so a later reader does not mistake it for a defect.
    replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Zenodo],
            before.configuration_token,
        ),
    )
    .expect("platform-only replace");

    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), true);
}

#[test]
fn a_committed_linked_repair_moves_all_three_timestamps() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    write_raw_assignment(
        &pool,
        fixture.publisher_id,
        "OAPEN",
        Uuid::new_v4(),
        "now()",
    );
    write_raw_assignment(&pool, fixture.publisher_id, "DOAB", Uuid::new_v4(), "now()");
    let before = cascade_snapshot(&pool, &fixture);

    replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Oasis,
            &[DistributionPlatform::Oapen],
            before.configuration_token,
        ),
    )
    .expect("repair replace");

    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), true);
}

#[test]
fn a_true_no_op_moves_no_timestamp_anywhere() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    let seed_token = token(&pool, fixture.publisher_id);
    replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            seed_token,
        ),
    )
    .expect("seed replace");
    let before = cascade_snapshot(&pool, &fixture);

    replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            before.configuration_token,
        ),
    )
    .expect("no-op replace");

    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), false);
}

#[test]
fn a_stale_request_moves_no_timestamp_anywhere() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    let stale = token(&pool, fixture.publisher_id);
    replace(
        &pool,
        &input(fixture.publisher_id, ThothPackage::Obelisk, &[], stale),
    )
    .expect("seed replace");
    let before = cascade_snapshot(&pool, &fixture);

    let outcome = replace(
        &pool,
        &input(
            fixture.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            stale,
        ),
    );

    assert!(matches!(
        outcome,
        Err(ThothError::StalePublisherServiceConfiguration)
    ));
    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), false);
}

#[test]
fn a_rolled_back_transaction_moves_no_timestamp_anywhere() {
    let (_guard, pool) = test_db::setup_test_db();
    let fixture = cascade_fixture(&pool);
    let before = cascade_snapshot(&pool, &fixture);

    let outcome = {
        let _injection = InjectedFailure::on_audit_insert(&pool);
        replace(
            &pool,
            &input(
                fixture.publisher_id,
                ThothPackage::Sphinx,
                &[DistributionPlatform::Zenodo],
                before.configuration_token,
            ),
        )
    };

    assert!(outcome.is_err());
    assert_cascade(&before, &cascade_snapshot(&pool, &fixture), false);
}

// --------------------------------------------------------------------------
// Staff report
// --------------------------------------------------------------------------

fn summaries(
    pool: &PgPool,
    publishers: Vec<Uuid>,
    packages: Vec<ThothPackage>,
    enabled_platforms: Vec<DistributionPlatform>,
) -> Vec<PublisherServiceConfigurationSummary> {
    PublisherServiceConfiguration::all_summaries(
        pool,
        100,
        0,
        PublisherOrderBy::default(),
        publishers,
        packages,
        enabled_platforms,
    )
    .expect("report")
}

#[test]
fn the_report_filters_by_publisher_package_and_enabled_platforms() {
    let (_guard, pool) = test_db::setup_test_db();
    let first = test_db::create_publisher(&pool);
    let second = test_db::create_publisher(&pool);
    let third = test_db::create_publisher(&pool);

    let first_token = token(&pool, first.publisher_id);
    replace(
        &pool,
        &input(
            first.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Oapen, DistributionPlatform::Zenodo],
            first_token,
        ),
    )
    .expect("first replace");
    let second_token = token(&pool, second.publisher_id);
    replace(
        &pool,
        &input(
            second.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            second_token,
        ),
    )
    .expect("second replace");

    let all = summaries(&pool, vec![], vec![], vec![]);
    assert_eq!(all.len(), 3, "three publishers exist");

    let by_publisher = summaries(&pool, vec![third.publisher_id], vec![], vec![]);
    assert_eq!(by_publisher.len(), 1);
    assert_eq!(
        by_publisher[0].configuration.publisher_id(),
        third.publisher_id
    );

    let by_package = summaries(&pool, vec![], vec![ThothPackage::Sphinx], vec![]);
    assert_eq!(by_package.len(), 2);

    // AND semantics: both platforms must be enabled.
    let both = summaries(
        &pool,
        vec![],
        vec![],
        vec![DistributionPlatform::Zenodo, DistributionPlatform::Oapen],
    );
    assert_eq!(both.len(), 1);
    assert_eq!(both[0].configuration.publisher_id(), first.publisher_id);

    let single = summaries(&pool, vec![], vec![], vec![DistributionPlatform::Zenodo]);
    assert_eq!(single.len(), 2);

    assert_eq!(
        PublisherServiceConfiguration::count(
            &pool,
            vec![],
            vec![],
            vec![DistributionPlatform::Zenodo, DistributionPlatform::Oapen]
        )
        .expect("count"),
        1,
        "the count query applies the same predicates as the list query"
    );
    assert_eq!(
        PublisherServiceConfiguration::count(&pool, vec![], vec![], vec![]).expect("count"),
        3
    );
}

#[test]
fn the_report_reports_the_latest_change_or_null() {
    let (_guard, pool) = test_db::setup_test_db();
    let changed = test_db::create_publisher(&pool);
    let untouched = test_db::create_publisher(&pool);

    let start = token(&pool, changed.publisher_id);
    replace(
        &pool,
        &input(
            changed.publisher_id,
            ThothPackage::Obelisk,
            &[DistributionPlatform::Zenodo],
            start,
        ),
    )
    .expect("first replace");
    let second = token(&pool, changed.publisher_id);
    replace(
        &pool,
        &input(
            changed.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            second,
        ),
    )
    .expect("second replace");

    let report = summaries(&pool, vec![], vec![], vec![]);
    let for_changed = report
        .iter()
        .find(|summary| summary.configuration.publisher_id() == changed.publisher_id)
        .expect("changed publisher");
    let last = for_changed.last_change.as_ref().expect("last change");
    assert_eq!(last.actor, ACTOR);
    assert_eq!(
        last.source,
        PublisherServiceConfigurationSource::SuperuserApi
    );
    let rows = audit_rows(&pool, changed.publisher_id);
    assert_eq!(rows.len(), 2);
    assert_eq!(last.changed_at, rows[1].created_at);

    let for_untouched = report
        .iter()
        .find(|summary| summary.configuration.publisher_id() == untouched.publisher_id)
        .expect("untouched publisher");
    assert!(
        for_untouched.last_change.is_none(),
        "a publisher with no recorded change reports null, never a placeholder"
    );
}

#[test]
fn the_report_orders_deterministically_with_a_publisher_id_tie_breaker() {
    let (_guard, pool) = test_db::setup_test_db();
    // `publisher_uniq_idx` is a unique index on `lower(publisher_name)`, so
    // literally duplicate names cannot exist. The equivalent ordering tie is
    // produced on a nullable sort field: every fixture publisher has a NULL
    // shortname, so only the mandatory `publisher_id ASC` tie-breaker can make
    // offset pagination deterministic.
    for _ in 0..12 {
        let publisher = test_db::create_publisher(&pool);
        assert!(publisher.publisher_shortname.is_none());
    }

    let mut paged: Vec<Uuid> = Vec::new();
    for offset in [0, 5, 10] {
        let page = PublisherServiceConfiguration::all_summaries(
            &pool,
            5,
            offset,
            PublisherOrderBy {
                field: PublisherField::PublisherShortname,
                direction: crate::graphql::types::inputs::Direction::Asc,
            },
            vec![],
            vec![],
            vec![],
        )
        .expect("page");
        paged.extend(
            page.iter()
                .map(|summary| summary.configuration.publisher_id()),
        );
    }

    assert_eq!(paged.len(), 12, "offset pagination must not skip or repeat");
    let unique: HashSet<Uuid> = paged.iter().copied().collect();
    assert_eq!(unique.len(), 12);
    let mut sorted = paged.clone();
    sorted.sort();
    assert_eq!(paged, sorted, "the publisher_id tie-breaker is ascending");
}

// --------------------------------------------------------------------------
// Migrated database contract
// --------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct CatalogText {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

fn catalog_values(pool: &PgPool, query: &str) -> Vec<String> {
    let mut connection = pool.get().expect("connection");
    sql_query(query)
        .load::<CatalogText>(&mut connection)
        .expect("catalog query")
        .into_iter()
        .map(|row| row.value)
        .collect()
}

#[test]
fn the_migrated_database_matches_the_schema_contract() {
    let (_guard, pool) = test_db::setup_test_db();

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT e.enumlabel AS value FROM pg_type t JOIN pg_enum e ON e.enumtypid = t.oid \
             WHERE t.typname = 'publisher_service_configuration_source' ORDER BY e.enumsortorder"
        ),
        vec!["SUPERUSER_API", "MIGRATION_BACKFILL"]
    );

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT conname AS value FROM pg_constraint \
             WHERE conrelid = 'public.publisher_service_configuration_history'::regclass \
             ORDER BY conname"
        ),
        vec![
            "publisher_service_configuration_history_actor_check",
            "publisher_service_configuration_history_pkey",
            "publisher_service_configuration_history_publisher_id_fkey",
        ]
    );

    // The actor constraint's catalog *definition*, not merely its name: the
    // invariant is that an actor contains at least one non-whitespace
    // character, so a narrower `btrim` predicate under the same name must fail
    // this assertion.
    assert_eq!(
        catalog_values(
            &pool,
            "SELECT pg_get_constraintdef(oid) AS value FROM pg_constraint \
             WHERE conname = 'publisher_service_configuration_history_actor_check'"
        ),
        vec!["CHECK ((actor ~ '[^[:space:]]'::text))"]
    );

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT indexname AS value FROM pg_indexes \
             WHERE tablename = 'publisher_service_configuration_history' ORDER BY indexname"
        ),
        vec![
            "publisher_service_configuration_history_pkey",
            "publisher_service_configuration_history_publisher_created_idx",
        ]
    );

    assert_eq!(
        catalog_values(
            &pool,
            "SELECT column_name AS value FROM information_schema.columns \
             WHERE table_name = 'publisher' AND column_name = 'service_configuration_updated_at'"
        ),
        vec!["service_configuration_updated_at"]
    );

    // No capability state and no job table is created anywhere by BE-03.
    assert!(catalog_values(
        &pool,
        "SELECT table_name AS value FROM information_schema.tables \
         WHERE table_schema = 'public' AND (table_name ILIKE '%capabilit%' OR table_name LIKE '%job%')"
    )
    .is_empty());
    assert!(catalog_values(
        &pool,
        "SELECT column_name AS value FROM information_schema.columns \
         WHERE table_schema = 'public' AND column_name ILIKE '%capabilit%'"
    )
    .is_empty());

    // The audit table is append-only: no `updated_at` column, so no
    // `diesel_manage_updated_at` trigger.
    assert!(catalog_values(
        &pool,
        "SELECT tgname AS value FROM pg_trigger \
         WHERE tgrelid = 'public.publisher_service_configuration_history'::regclass \
           AND NOT tgisinternal"
    )
    .is_empty());
}

#[test]
fn deleting_a_publisher_cascades_to_its_configuration_audit() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let start = token(&pool, publisher.publisher_id);
    replace(
        &pool,
        &input(
            publisher.publisher_id,
            ThothPackage::Sphinx,
            &[DistributionPlatform::Zenodo],
            start,
        ),
    )
    .expect("replace");
    assert_eq!(audit_rows(&pool, publisher.publisher_id).len(), 1);

    let mut connection = pool.get().expect("connection");
    diesel::delete(publisher::table.filter(publisher::publisher_id.eq(publisher.publisher_id)))
        .execute(&mut connection)
        .expect("delete publisher");

    assert!(audit_rows(&pool, publisher.publisher_id).is_empty());
}
