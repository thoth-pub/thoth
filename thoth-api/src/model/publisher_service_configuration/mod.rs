//! Protected publisher service configuration (`BE-03`).
//!
//! This module owns the protected representation of one publisher's **desired**
//! service configuration — current subscription package, effective capability
//! codes and enabled distribution platforms — the canonical optimistic
//! concurrency token that versions it, the append-only configuration audit
//! record, and the single authoritative write coordinator through which every
//! committed production configuration change passes
//! ([`crud::replace_publisher_service_configuration`]).
//!
//! `BE-03` owns desired configuration only. It creates no distribution job, no
//! job target and no job attempt; it performs no dissemination; and it activates
//! no destination.
//!
//! Effective capabilities are **derived on read** from the canonical
//! `publisher.subscription_package` through `BE-01`'s code-owned
//! [`ThothPackage::capabilities`] and are persisted nowhere: there is no
//! capability column, table, override or cache, so no package/capability
//! inconsistency is representable (`ADR-0001` sections 4.1, 4.2 and 4.4).

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::distribution_job::DistributionJobCreation;
use crate::model::publisher::{Publisher, ThothPackage};
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::publisher_service_configuration_history;

/// How a recorded service-configuration change entered the system.
///
/// The inventory is closed. There is deliberately no `OTHER`, no `UNKNOWN` and
/// no `Default`: an unrecognised value must fail rather than resolve to a
/// nearest source.
///
/// `source` and `actor` form **one contract**: `source` fixes the namespace and
/// the required provenance of `actor` (specification section 9.1).
///
/// `BE-03` writes only [`Self::SuperuserApi`]. [`Self::MigrationBackfill`] is
/// defined here so the separately approved and separately specified `MIG-01`
/// controlled historical backfill has a coherent value to write, and no `BE-03`
/// execution path emits it.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "How a recorded service-configuration change entered the system"),
    ExistingTypePath = "crate::schema::sql_types::PublisherServiceConfigurationSource"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PublisherServiceConfigurationSource {
    #[cfg_attr(
        feature = "backend",
        db_rename = "SUPERUSER_API",
        graphql(
            description = "A committed superuser replacePublisherServiceConfiguration call; the actor is the authenticated account identifier"
        )
    )]
    SuperuserApi,
    #[cfg_attr(
        feature = "backend",
        db_rename = "MIGRATION_BACKFILL",
        graphql(
            description = "A separately approved controlled historical backfill; the actor is the authorized control identity"
        )
    )]
    MigrationBackfill,
}

/// The desired service configuration of one publisher.
///
/// The publisher row is the single source of truth for every value this type
/// exposes: the package and the configuration version token are columns on it,
/// and the effective capabilities are computed from that same package. A
/// response can therefore never report a package and a capability set that
/// disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherServiceConfiguration {
    pub publisher: Publisher,
}

impl PublisherServiceConfiguration {
    pub fn new(publisher: Publisher) -> Self {
        Self { publisher }
    }

    pub fn publisher_id(&self) -> Uuid {
        self.publisher.publisher_id
    }
}

// The `subscriptionPackage`, `effectiveCapabilities` and `updatedAt` accessors
// are the GraphQL resolvers in `crate::graphql::model`, which read them from
// this same `publisher` row. They are deliberately not duplicated here: one
// definition means a response can never report a package and a capability set
// that disagree, and the derivation has exactly one site.

/// Metadata of one recorded service-configuration change.
///
/// The audit `before_state`/`after_state` JSON is deliberately absent: it is
/// never exposed through any GraphQL surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherServiceConfigurationChange {
    pub changed_at: Timestamp,
    pub actor: String,
    pub source: PublisherServiceConfigurationSource,
}

/// One publisher's service configuration together with its latest change
/// metadata, as returned by the superuser-only staff report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherServiceConfigurationSummary {
    pub configuration: PublisherServiceConfiguration,
    pub last_change: Option<PublisherServiceConfigurationChange>,
}

/// One persisted configuration audit row.
///
/// The table is append-only: it has no `updated_at` column and therefore no
/// `diesel_manage_updated_at` trigger, and nothing rewrites or deletes a row.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherServiceConfigurationHistory {
    pub publisher_service_configuration_history_id: Uuid,
    pub publisher_id: Uuid,
    pub actor: String,
    pub source: PublisherServiceConfigurationSource,
    pub before_state: serde_json::Value,
    pub after_state: serde_json::Value,
    pub created_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = publisher_service_configuration_history)
)]
pub struct NewPublisherServiceConfigurationHistory {
    pub publisher_id: Uuid,
    pub actor: String,
    pub source: PublisherServiceConfigurationSource,
    pub before_state: serde_json::Value,
    pub after_state: serde_json::Value,
}

/// The bounded canonical service-configuration state recorded in an audit row.
///
/// The key set is exactly these three keys and must not be widened. Activation
/// identifiers, per-row `enabled_at`/`disabled_at` timestamps, disabled-platform
/// history, effective capabilities, credentials, endpoints and unrelated
/// publisher metadata are all deliberately absent (specification section 8.2).
/// A linked-state repair is already distinguishable through the differing
/// `configurationVersion` values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalServiceConfigurationState {
    pub subscription_package: ThothPackage,
    /// Serialized in canonical `DistributionPlatform::ALL` declaration order, so
    /// two equal sets always serialize identically.
    pub enabled_distribution_platforms: Vec<DistributionPlatform>,
    pub configuration_version: Timestamp,
}

/// The audit provenance the caller supplies to the write coordinator.
///
/// This is a parameter, never inferred from ambient state and never a GraphQL
/// input. The coordinator makes no authentication or authorization decision of
/// its own; its callers authorize first and then supply this context
/// (specification section 7.6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceConfigurationWriteContext<'a> {
    pub source: PublisherServiceConfigurationSource,
    pub actor: &'a str,
    /// Whether a qualifying activation in this transaction may create a durable
    /// distribution job (`BE-04`).
    ///
    /// This is supplied by the resolver from the request context, exactly as
    /// `source` and `actor` are. The coordinator makes **no** ambient
    /// environment lookup of its own, which is also what lets every creation
    /// test drive the switch directly with no environment mutation.
    pub job_creation: DistributionJobCreation,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete desired service configuration to store for a publisher. This is a replace, not a patch: the platform list is the complete desired enabled set, and an empty list means no destination is enabled"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacePublisherServiceConfigurationInput {
    pub publisher_id: Uuid,
    pub subscription_package: ThothPackage,
    pub enabled_distribution_platforms: Vec<DistributionPlatform>,
    pub expected_updated_at: Timestamp,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
mod tests;
