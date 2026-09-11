//! The protected `metric_source_account` administration coordinator
//! (`MET-WP1-13`).
//!
//! [`Crud`] is deliberately **not** implemented for source accounts: there is
//! no generic create/update/delete surface, no `all`/`count` listing and no
//! delete at all. The two supported writes are [`create_metric_source_account`]
//! and [`update_metric_source_account`], and they are the only places in the
//! repository that commit a change to a `metric_source_account` row.
//!
//! Neither function makes an authorization decision. Authorization is the
//! caller's responsibility and happens before any of this module is reached.
//!
//! **Lock topology.** Source and platform codes are resolved with ordinary
//! non-locking reads. The single application-requested `FOR UPDATE` lock is
//! taken on the account row alone. The referenced `metric_source`,
//! `metric_platform` and `publisher` rows are never locked by this module, and
//! no joined multi-table `FOR UPDATE` exists anywhere in it.
//!
//! **Fail-closed stored configuration.** Every account this module returns has
//! passed [`MetricSourceAccountConfiguration::decode_stored`] and the source
//! compatibility rule. An update against an account whose stored value cannot
//! be decoded safely performs no canonical `UPDATE` and writes no audit row, and
//! the stored value is never embedded in the error.
//!
//! [`Crud`]: crate::model::Crud

use std::borrow::Cow;

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};

use super::{
    check_source_compatibility, ConfigurationError, MetricSourceAccount,
    MetricSourceAccountConfiguration, MetricSourceAccountConfigurationKind, NewMetricSourceAccount,
    PatchMetricSourceAccount,
};
use crate::db::PgPool;
use crate::model::metric_platform::crud::by_code as platform_by_code;
use crate::model::metric_source::crud::{by_code as source_by_code, by_id as source_by_id};
use crate::model::metric_source::MetricSource;
use crate::model::metric_source_registry_history::{
    record_create, record_update, MetricSourceRegistryHistoryEntity,
};
use crate::schema::metric_source_account;

impl From<ConfigurationError> for ThothError {
    fn from(error: ConfigurationError) -> Self {
        ThothError::DatabaseConstraintError(Cow::Borrowed(error.message()))
    }
}

/// The same exact-equality read, on a caller-supplied connection, with no
/// decoding: callers that return the row to a client must decode it first.
fn by_code(connection: &mut PgConnection, code: &str) -> ThothResult<MetricSourceAccount> {
    metric_source_account::table
        .filter(metric_source_account::code.eq(code))
        .first::<MetricSourceAccount>(connection)
        .map_err(Into::into)
}

/// Prove one persisted account is safe to expose: its stored configuration
/// decodes, and the decoded kind is the one its immutable source requires.
fn ensure_supported(
    account: &MetricSourceAccount,
    source: &MetricSource,
) -> ThothResult<MetricSourceAccountConfiguration> {
    let decoded = account.decoded_configuration()?;
    check_source_compatibility(source, decoded.kind)
        .map_err(|_| ConfigurationError::UnsupportedStored)?;
    Ok(decoded)
}

/// Look one metric source account up by its exact stable code.
///
/// PostgreSQL `TEXT` equality on the `UNIQUE(code)` column with no trimming,
/// case folding, `ILIKE`, Unicode or whitespace normalization or aliasing.
/// The row is returned only after its stored configuration has passed the
/// closed decoder against its immutable source; otherwise the bounded
/// unsupported-configuration failure is returned and nothing stored is exposed.
///
/// This takes no row lock.
pub(crate) fn metric_source_account_by_code(
    db: &PgPool,
    code: &str,
) -> ThothResult<MetricSourceAccount> {
    let mut connection = db.get()?;
    let account = by_code(&mut connection, code)?;
    let source = source_by_id(&mut connection, account.source_id)?;
    ensure_supported(&account, &source)?;
    Ok(account)
}

/// Create one metric source account and its `CREATE` audit row atomically.
///
/// One transaction on one connection:
///
/// 1. resolve the exact `source_code` and `platform_code` with ordinary
///    non-locking reads; an unknown code fails here as `EntityNotFound`;
/// 2. enforce the closed compatibility matrix against the resolved source's
///    immutable `acquisition_type` and `driver_key`; for
///    `CLOUDFRONT_LEGACY_S3_V1` also require a non-null `expected_publisher_id`
///    (Specification Amendment 2's safety pin for the merged managed `DRIVER`
///    ingestion coordinator); then validate the typed configuration and
///    canonicalize it, with the hostname compared to the supplied
///    `external_key`;
/// 3. insert the canonical row, returning what PostgreSQL actually persisted;
/// 4. append exactly one audit row with `before_state = NULL`.
///
/// PostgreSQL remains the authority on code uniqueness, `(source_id,
/// external_key)` uniqueness, the nonblank CHECKs and the publisher foreign
/// key. No application-requested `FOR UPDATE` lock is taken.
pub(crate) fn create_metric_source_account(
    db: &PgPool,
    actor: &str,
    data: &NewMetricSourceAccount,
) -> ThothResult<MetricSourceAccount> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let source = source_by_code(connection, &data.source_code)?;
        let platform = platform_by_code(connection, &data.platform_code)?;
        check_source_compatibility(&source, data.configuration.kind)?;
        if data.configuration.kind == MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1
            && data.expected_publisher_id.is_none()
        {
            return Err(ConfigurationError::CloudFrontRequiresExpectedPublisher.into());
        }
        let (configuration, _) = data.configuration.canonicalize(&data.external_key)?;

        let created: MetricSourceAccount = diesel::insert_into(metric_source_account::table)
            .values((
                metric_source_account::code.eq(&data.code),
                metric_source_account::source_id.eq(source.source_id),
                metric_source_account::platform_id.eq(platform.platform_id),
                metric_source_account::external_key.eq(&data.external_key),
                metric_source_account::expected_publisher_id.eq(data.expected_publisher_id),
                metric_source_account::configuration.eq(&configuration),
                metric_source_account::enabled.eq(data.enabled),
            ))
            .returning(metric_source_account::all_columns)
            .get_result(connection)?;

        record_create(
            connection,
            MetricSourceRegistryHistoryEntity::SourceAccount,
            created.source_account_id,
            actor,
            &created,
        )?;

        Ok(created)
    })
}

/// Replace one metric source account's mutable fields and audit the change
/// atomically.
///
/// One transaction on one connection, under `serialized last-write-wins`:
///
/// 1. resolve and lock **exactly one** row — the `metric_source_account` row
///    named by the exact `code` — with `FOR UPDATE`;
/// 2. read the account's immutable source with an ordinary non-locking read,
///    and decode the current stored configuration through the closed decoder;
///    an unsupported or inconsistent stored value stops here, before any
///    canonical write and before anything is copied into audit history;
/// 3. enforce the compatibility matrix and canonicalize the requested typed
///    configuration, comparing its hostname to the immutable `external_key`;
/// 4. if the requested `enabled` and the canonical JSON are semantically equal
///    to the persisted state, return the current row unchanged: no `UPDATE`
///    runs and no audit row is written. JSONB/`serde_json::Value` equality is
///    value equality, so key order and whitespace never count as a change;
/// 5. otherwise update **only** `configuration` and `enabled` and append
///    exactly one audit row carrying the exact before and after states.
pub(crate) fn update_metric_source_account(
    db: &PgPool,
    actor: &str,
    data: &PatchMetricSourceAccount,
) -> ThothResult<MetricSourceAccount> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let current: MetricSourceAccount = metric_source_account::table
            .filter(metric_source_account::code.eq(&data.code))
            .for_update()
            .first::<MetricSourceAccount>(connection)?;

        let source = source_by_id(connection, current.source_id)?;
        ensure_supported(&current, &source)?;

        check_source_compatibility(&source, data.configuration.kind)?;
        let (configuration, _) = data.configuration.canonicalize(&current.external_key)?;

        if current.enabled == data.enabled && current.configuration == configuration {
            return Ok(current);
        }

        let updated: MetricSourceAccount =
            diesel::update(metric_source_account::table.find(current.source_account_id))
                .set((
                    metric_source_account::configuration.eq(&configuration),
                    metric_source_account::enabled.eq(data.enabled),
                ))
                .returning(metric_source_account::all_columns)
                .get_result(connection)?;

        record_update(
            connection,
            MetricSourceRegistryHistoryEntity::SourceAccount,
            updated.source_account_id,
            actor,
            &current,
            &updated,
        )?;

        Ok(updated)
    })
}
