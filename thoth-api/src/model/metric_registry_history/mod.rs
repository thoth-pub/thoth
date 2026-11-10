//! Metrics registry administration audit history (`MET-WP1-12`).
//!
//! This module owns the persisted `metric_registry_history` model: the
//! Metrics-local, append-only, before/after audit record of every committed
//! `metric_platform`, `metric_measure` and `metric_platform_measure`
//! administration mutation.
//!
//! It follows the newer publisher service-configuration coordinator precedent
//! — one explicit `before_state`/`after_state` pair plus an actor, written in
//! the same transaction as the canonical change — rather than the older
//! per-entity `*_history` convention. The generic [`Crud`] trait/macro is
//! deliberately **not** implemented for this surface or for the three
//! registries it audits: `Crud` carries unrelated `all`/`count`/`delete`
//! behaviour, and its generic create path does not provide this selected audit
//! contract.
//!
//! This is **not** a generic cross-programme audit abstraction. It records
//! exactly the three Metrics registry entities and exactly the two actions the
//! approved administration surface can perform.
//!
//! The table is append-only: it has no `updated_at` column, no
//! `diesel_manage_updated_at` trigger, and nothing in the repository rewrites
//! or deletes a row. `entity_id` deliberately carries **no** foreign key,
//! because a row is polymorphic evidence spanning three canonical tables and
//! must not be cascade-deleted with the registry state it describes.
//!
//! Audit rows are never exposed through GraphQL by `MET-WP1-12`: there is no
//! audit query, no audit object type and no audit index, because no approved
//! audit-history access path exists yet.
//!
//! [`Crud`]: crate::model::Crud

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::metric_registry_history;

/// Which registry a recorded administration mutation changed.
///
/// The inventory is closed and matches exactly the three registries
/// `MET-WP1-12` administers. There is deliberately no `OTHER`, `UNKNOWN` or
/// `Default` variant: an unrecognised database, serde or string value must fail
/// rather than silently resolve to a nearest entity.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricRegistryHistoryEntity"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricRegistryHistoryEntity {
    /// A `metric_platform` row.
    #[cfg_attr(feature = "backend", db_rename = "PLATFORM")]
    Platform,
    /// A `metric_measure` row.
    #[cfg_attr(feature = "backend", db_rename = "MEASURE")]
    Measure,
    /// A `metric_platform_measure` mapping row.
    #[cfg_attr(feature = "backend", db_rename = "PLATFORM_MEASURE")]
    PlatformMeasure,
}

/// What a recorded administration mutation did.
///
/// There is deliberately no `DELETE` value: the approved administration surface
/// exposes no delete mutation, and a value nothing can write would be a
/// standing invitation to add one without review. Registry rows are retired by
/// setting `enabled = false`, which is an ordinary audited `UPDATE`.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricRegistryHistoryAction"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricRegistryHistoryAction {
    /// A registry row was created. `before_state` is NULL.
    #[cfg_attr(feature = "backend", db_rename = "CREATE")]
    Create,
    /// A registry row's approved mutable fields were replaced. `before_state`
    /// is the exact state the update overwrote.
    #[cfg_attr(feature = "backend", db_rename = "UPDATE")]
    Update,
}

/// One persisted registry-administration audit row.
///
/// `before_state` and `after_state` hold the exact persisted canonical row as
/// serialized by the audited model type, so an audit entry records what the
/// database actually stored rather than what the request asked for.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRegistryHistory {
    pub metric_registry_history_id: Uuid,
    pub entity: MetricRegistryHistoryEntity,
    pub entity_id: Uuid,
    pub action: MetricRegistryHistoryAction,
    pub actor: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: serde_json::Value,
    pub created_at: Timestamp,
}

/// One audit row to append.
///
/// `metric_registry_history_id` and `created_at` are database-owned.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = metric_registry_history)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricRegistryHistory {
    pub entity: MetricRegistryHistoryEntity,
    pub entity_id: Uuid,
    pub action: MetricRegistryHistoryAction,
    pub actor: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: serde_json::Value,
}

#[cfg(feature = "backend")]
mod backend {
    use diesel::pg::PgConnection;
    use diesel::RunQueryDsl;
    use serde::Serialize;
    use thoth_errors::ThothResult;
    use uuid::Uuid;

    use super::{
        MetricRegistryHistoryAction, MetricRegistryHistoryEntity, NewMetricRegistryHistory,
    };
    use crate::schema::metric_registry_history;

    /// Append the `CREATE` audit row for one committed registry creation.
    ///
    /// This takes a `&mut PgConnection` rather than a pool, so it can only be
    /// called from inside a caller's open transaction: an audit row can never
    /// be committed independently of the canonical row that justified it.
    ///
    /// `after` is the **persisted** canonical row read back from the database,
    /// never the request, so generated identifiers and database-authored
    /// timestamps are recorded exactly as stored.
    pub(crate) fn record_create<T: Serialize>(
        connection: &mut PgConnection,
        entity: MetricRegistryHistoryEntity,
        entity_id: Uuid,
        actor: &str,
        after: &T,
    ) -> ThothResult<()> {
        insert(
            connection,
            NewMetricRegistryHistory {
                entity,
                entity_id,
                action: MetricRegistryHistoryAction::Create,
                actor: actor.to_string(),
                before_state: None,
                after_state: serde_json::to_value(after)?,
            },
        )
    }

    /// Append the `UPDATE` audit row for one committed registry change.
    ///
    /// Both states are the exact persisted canonical rows: `before` as read
    /// under the row lock the update took, `after` as returned by the update
    /// itself.
    pub(crate) fn record_update<T: Serialize>(
        connection: &mut PgConnection,
        entity: MetricRegistryHistoryEntity,
        entity_id: Uuid,
        actor: &str,
        before: &T,
        after: &T,
    ) -> ThothResult<()> {
        insert(
            connection,
            NewMetricRegistryHistory {
                entity,
                entity_id,
                action: MetricRegistryHistoryAction::Update,
                actor: actor.to_string(),
                before_state: Some(serde_json::to_value(before)?),
                after_state: serde_json::to_value(after)?,
            },
        )
    }

    fn insert(connection: &mut PgConnection, row: NewMetricRegistryHistory) -> ThothResult<()> {
        diesel::insert_into(metric_registry_history::table)
            .values(&row)
            .execute(connection)?;
        Ok(())
    }
}

#[cfg(feature = "backend")]
pub(crate) use backend::{record_create, record_update};

#[cfg(all(test, feature = "backend"))]
mod tests;
