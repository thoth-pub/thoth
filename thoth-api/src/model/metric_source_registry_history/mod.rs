//! Metrics source administration audit history (`MET-WP1-13`).
//!
//! This module owns the persisted `metric_source_registry_history` model: the
//! Metrics-local, append-only, before/after audit record of every committed
//! `metric_source` and `metric_source_account` administration mutation.
//!
//! It is a **parallel** audit to [`metric_registry_history`], not an extension
//! of it. `MET-WP1-12` deliberately closed that table's entity inventory to
//! `PLATFORM`, `MEASURE` and `PLATFORM_MEASURE` and rejected turning it into a
//! generic cross-programme abstraction; this table has the same shape and the
//! same discipline, and records exactly the two source entities and the two
//! actions the approved source-administration surface can perform.
//!
//! The generic [`Crud`] trait/macro is deliberately **not** implemented for
//! this surface or for the two registries it audits.
//!
//! The table is append-only: it has no `updated_at` column, no
//! `diesel_manage_updated_at` trigger, and nothing in the repository rewrites
//! or deletes a row. `entity_id` deliberately carries **no** foreign key,
//! because a row is polymorphic evidence spanning two canonical tables and must
//! not be cascade-deleted with the state it describes.
//!
//! Audit rows are never exposed through GraphQL by `MET-WP1-13`: there is no
//! audit query, no audit object type and no audit index, because no approved
//! audit-history access path exists yet.
//!
//! [`metric_registry_history`]: crate::model::metric_registry_history
//! [`Crud`]: crate::model::Crud

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::metric_source_registry_history;

/// Which source-administration entity a recorded mutation changed.
///
/// The inventory is closed and matches exactly the two entities `MET-WP1-13`
/// administers. There is deliberately no `OTHER`, `UNKNOWN`, `Default` or
/// checkpoint variant: an unrecognised database, serde or string value must
/// fail rather than silently resolve to a nearest entity.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricSourceRegistryHistoryEntity"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricSourceRegistryHistoryEntity {
    /// A `metric_source` row.
    #[cfg_attr(feature = "backend", db_rename = "SOURCE")]
    Source,
    /// A `metric_source_account` row.
    #[cfg_attr(feature = "backend", db_rename = "SOURCE_ACCOUNT")]
    SourceAccount,
}

/// What a recorded administration mutation did.
///
/// There is deliberately no `DELETE` value: the approved administration surface
/// exposes no delete mutation. Rows are retired by setting `enabled = false`,
/// which is an ordinary audited `UPDATE`.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricSourceRegistryHistoryAction"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricSourceRegistryHistoryAction {
    /// A row was created. `before_state` is NULL.
    #[cfg_attr(feature = "backend", db_rename = "CREATE")]
    Create,
    /// A row's approved mutable fields were replaced. `before_state` is the
    /// exact state the update overwrote.
    #[cfg_attr(feature = "backend", db_rename = "UPDATE")]
    Update,
}

/// One persisted source-administration audit row.
///
/// `before_state` and `after_state` hold the exact persisted canonical row as
/// serialized by the audited model type, so an audit entry records what the
/// database actually stored rather than what the request asked for. For a
/// source account that includes its canonical non-secret configuration, which
/// the coordinator has already passed through the closed stored-configuration
/// decoder before it may be serialized here.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceRegistryHistory {
    pub metric_source_registry_history_id: Uuid,
    pub entity: MetricSourceRegistryHistoryEntity,
    pub entity_id: Uuid,
    pub action: MetricSourceRegistryHistoryAction,
    pub actor: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: serde_json::Value,
    pub created_at: Timestamp,
}

/// One audit row to append.
///
/// `metric_source_registry_history_id` and `created_at` are database-owned.
#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = metric_source_registry_history)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricSourceRegistryHistory {
    pub entity: MetricSourceRegistryHistoryEntity,
    pub entity_id: Uuid,
    pub action: MetricSourceRegistryHistoryAction,
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
        MetricSourceRegistryHistoryAction, MetricSourceRegistryHistoryEntity,
        NewMetricSourceRegistryHistory,
    };
    use crate::schema::metric_source_registry_history;

    /// Append the `CREATE` audit row for one committed source-administration
    /// creation.
    ///
    /// This takes a `&mut PgConnection` rather than a pool, so it can only be
    /// called from inside a caller's open transaction: an audit row can never
    /// be committed independently of the canonical row that justified it.
    ///
    /// `after` is the **persisted** canonical row read back from the database,
    /// never the request.
    pub(crate) fn record_create<T: Serialize>(
        connection: &mut PgConnection,
        entity: MetricSourceRegistryHistoryEntity,
        entity_id: Uuid,
        actor: &str,
        after: &T,
    ) -> ThothResult<()> {
        insert(
            connection,
            NewMetricSourceRegistryHistory {
                entity,
                entity_id,
                action: MetricSourceRegistryHistoryAction::Create,
                actor: actor.to_string(),
                before_state: None,
                after_state: serde_json::to_value(after)?,
            },
        )
    }

    /// Append the `UPDATE` audit row for one committed source-administration
    /// change.
    ///
    /// Both states are the exact persisted canonical rows: `before` as read
    /// under the row lock the update took, `after` as returned by the update
    /// itself.
    pub(crate) fn record_update<T: Serialize>(
        connection: &mut PgConnection,
        entity: MetricSourceRegistryHistoryEntity,
        entity_id: Uuid,
        actor: &str,
        before: &T,
        after: &T,
    ) -> ThothResult<()> {
        insert(
            connection,
            NewMetricSourceRegistryHistory {
                entity,
                entity_id,
                action: MetricSourceRegistryHistoryAction::Update,
                actor: actor.to_string(),
                before_state: Some(serde_json::to_value(before)?),
                after_state: serde_json::to_value(after)?,
            },
        )
    }

    fn insert(
        connection: &mut PgConnection,
        row: NewMetricSourceRegistryHistory,
    ) -> ThothResult<()> {
        diesel::insert_into(metric_source_registry_history::table)
            .values(&row)
            .execute(connection)?;
        Ok(())
    }
}

#[cfg(feature = "backend")]
pub(crate) use backend::{record_create, record_update};

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
